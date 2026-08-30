# Selector Engine, Verb Vocabulary, and Data-Driven Types

Research background for ADR-0001. This report covers the public API surface:
the lazy selector query engine, the command verb vocabulary, the data-driven
type system, and the verb registry.

---

## Executive summary

These are the recommendations. Each one has detail in a later section.

1. **Build the selector as a typed, lazy expression tree.** Bind the tree to a
   world and to a domain (`units` or `tiles`). Overload `&`, `|`, and `~`. Make
   `__bool__`, `__len__`, and `__iter__` raise a `TypeError` with a message that
   names the correct method. Polars does this. It stops the most common user
   error. See [Section 1](#1-the-python-selector-api).

2. **Use an explicit field namespace (`units.f.health`), not a magic
   underscore.** The Ibis project records the usability problems of the `_` API
   in its own issue tracker. See [Section 1.2](#12-field-access).

3. **Treat selector resolution as a database query.** Use three-valued
   (Kleene) evaluation of the predicate tree against each L1 and L2 summary.
   Each node returns `All`, `None`, or `Some`. This prunes whole blocks without
   a conversion to disjunctive normal form. See
   [Section 2](#2-query-planning-and-execution).

4. **The pyramid is both the index and the statistics catalogue.** A histogram
   summary gives an *exact* count for an equality predicate over a block. So
   selectivity for a single predicate is exact, not estimated. Only conjunctions
   need an independence assumption. See [Section 2.2](#22-selectivity).

5. **Represent results as a two-level chunk mask, not as a sorted ID vector.**
   For units, use `(ChunkId, bitmask)` pairs. For tiles, use an L2 mask, an L1
   mask, and dense L0 bitplanes. This is the Roaring bitmap design, but
   specialised to a key space that the storage layout already defines. Export a
   sorted `u32` array only at the FFI boundary. See
   [Section 3](#3-result-representation).

6. **Cache resolved selectors for the duration of the Python phase, then clear
   the cache at the frame barrier.** The write model does not change during the
   Python phase. So this needs no invalidation logic at all. See
   [Section 2.5](#25-caching).

7. **Decide the evaluation time of a selector that is a verb parameter.** The
   context brief does not specify this. Snapshot semantics (resolve against the
   sealed pre-frame state) keeps the determinism guarantee. Live semantics
   breaks it. Recommend snapshot. See
   [Section 2.6](#26-the-snapshot-versus-live-problem).

8. **Keep the field registry as one declaration that generates the L0 accessor,
   the monoid, the summary slot, and the predicate constructor.** A field
   without a declared monoid gives a predicate that cannot prune. Enforce this
   at compile time. See [Section 4](#4-summary-fields-and-predicate-vocabulary).

9. **Ship about 34 verbs, of which about 12 are load-bearing.** The rest are the
   same Rust code with different data. Merge symmetric pairs such as
   `damage`/`heal` into one signed operation. See
   [Section 5](#5-the-verb-vocabulary).

10. **Split "upgrades" from "capabilities".** The context brief treats a `u64`
    bitmask as both. Use an interned `UpgradeSetId(u32)` for the authoritative,
    unbounded upgrade set. Derive a `u64` capability mask from it for the hot
    loop and the selector predicates. This removes the 64-bit ceiling. See
    [Section 6.3](#63-the-u64-ceiling).

11. **Use fixed-point arithmetic for all stat and modifier maths.** Floating
    point addition is not associative. Fixed-point makes modifier order a schema
    property, not a runtime hazard. See
    [Section 6.4](#64-modifier-stacking-and-determinism).

12. **Use a runtime function table for verb dispatch, not a static enum.**
    Dispatch happens once per command, not once per entity. So dispatch cost is
    noise. Choose on ergonomics. See [Section 7](#7-verb-registry-and-dispatch).

13. **Declare each verb's component access and its region scope.** Derive
    parallelism the way Bevy does, plus a spatial dimension that Bevy does not
    have. The region scope is dynamic, because it comes from the resolved
    selector. See [Section 7.2](#72-conflict-domain-declaration).

14. **Stage the extensibility path: fixed verbs, then composite verbs, then a
    vectorised expression DSL. Defer a bytecode VM.** If you ever build a VM,
    make each opcode operate on a whole column. A per-entity scalar VM
    re-creates the exact cost that this architecture exists to remove. See
    [Section 9](#9-the-extensibility-ceiling).

**Where I disagree with the context brief.** Four points. Item 8 conflates
upgrades and capabilities. Item 6 defines the summary schema circularly ("the
fields selectors filter on"), which will drift. Item 7 does not say when a
selector parameter is evaluated, which is a determinism hole. Item 7's "~30
verbs" is the right count, but the brief does not separate load-bearing verbs
from data-parameterised ones, which makes the number look larger than the work.

---

## 1. The Python selector API

### 1.1 Prior art

Five libraries build deferred expression trees in Python. Each one teaches a
different lesson.

**Polars `LazyFrame`.** A `LazyFrame` holds no data. It holds a query plan. The
plan is a tree of operations. An optimiser walks the tree before execution and
applies predicate pushdown, projection pushdown, expression simplification, and
slice pushdown. ([Polars lazy API
guide](https://docs.pola.rs/user-guide/concepts/lazy-api/), [Polars
internals](https://deepwiki.com/pola-rs/polars/2.3-lazyframe))

The lesson: separate the plan from the execution. Then you can optimise, and
you can also *print* the plan. Polars gives `LazyFrame.explain()` for this.
([explain
docs](https://docs.pola.rs/py-polars/html/reference/lazyframe/api/polars.LazyFrame.explain.html))
Add an equivalent. A user who cannot see that a query does a full scan will
report a performance bug that is really a schema bug.

**Ibis.** Ibis builds a parse tree by operator overloading. The tree is
translated and executed later. Ibis also offers a convenience API named `_` for
deferred attribute access. The Ibis maintainers record its usability problems in
their own tracker: the name is not searchable, and it collides with other
conventions. ([Ibis issue
4704](https://github.com/ibis-project/ibis/issues/4704), [Ibis
internals](https://ibis-project.org/concepts/internals))

The lesson: do not use a magic single-character name. Use an explicit namespace.

**SQLAlchemy Core.** Column objects overload comparison and boolean operators to
produce clause elements. The API is verbose but very predictable. Users rarely
guess wrong about what a line does.

**Django `QuerySet`.** A `QuerySet` is lazy until iterated. It offers
`.update()` and `.delete()` as set-valued terminal operations that run in the
database. This is the closest prior art to a set-valued verb. Its `Q` object
exists only because Python cannot overload `and` and `or`.

The lesson: set-valued mutation on a lazy set is a proven, familiar pattern.
Users already know `qs.filter(...).update(...)`.

**xarray and Dask.** These show the failure mode. A Dask array looks like a
NumPy array, so users write code that works but is very slow, because a hidden
operation forces a full computation. The abstraction leaks under performance,
not under correctness.

The lesson: an API that *pretends* to be eager will be used eagerly. Make the
laziness visible. Do not imitate NumPy semantics on the selector object.

### 1.2 Field access

Recommend an explicit field namespace. Create selectors from the world, so that
each selector carries its world binding and its schema.

```python
import cachette as ct

w = ct.World.load("map.ct")
units = w.units          # a UnitSelector over all units
tiles = w.tiles          # a TileSelector over all tiles
f = units.f              # the unit field namespace
t = tiles.f              # the tile field namespace
me = w.faction("blue")
```

`units.f.health` is longer than `_.health`. It is also searchable, it gives
autocompletion, and it makes the domain obvious at the point of use.

### 1.3 The core expression grammar

```python
# comparison operators build predicate leaves
wounded  = f.health < 30
mine     = f.faction == me
infantry = f.unit_type.is_in([ct.UT.SPEARMAN, ct.UT.ARCHER])
veteran  = f.has_capability(ct.Cap.VETERAN)

# set algebra uses &, |, ~ and ^
targets = mine & wounded & ~veteran

# spatial predicates
near_home = f.within(w.home_tile, radius=12)
in_forest = f.on_tiles(t.terrain == ct.Terrain.FOREST)

# the cross-domain bridge is explicit in both directions
contested_tiles = tiles.holding(units.faction(w.enemy))
enemy_units     = units.on_tiles(t.owner == me) & (f.faction != me)
```

Type the selector by domain. `UnitSelector & TileSelector` must raise a
`TypeError` at build time, not produce an empty set at run time. The bridge is
`.on_tiles(tile_selector)` and `.holding(unit_selector)`. Both are explicit, so
the reader can see where the occupancy index is used.

### 1.4 Guardrails against per-entity loops

This is the most important part of the API design. The context brief lists "users
writing per-entity Python loops" as a known risk. Close the door in the type
itself.

```python
class Selector:
    def __bool__(self):
        raise TypeError(
            "A Selector has no truth value. "
            "Use `&`, `|`, `~` instead of `and`, `or`, `not`. "
            "To test for emptiness, use `.is_empty()`."
        )

    def __len__(self):
        raise TypeError("A Selector has no length. Use `.count()`.")

    def __iter__(self):
        raise TypeError(
            "A Selector is not iterable. "
            "Use `.to_numpy(fields=[...])` to get column arrays, "
            "or apply a verb to the whole set."
        )

    def __getitem__(self, key):
        raise TypeError("A Selector does not support indexing.")
```

Polars raises on `__bool__` for the same reason. The failure is loud and it
names the fix. This one method prevents `if sel:`, `sel and other`, and
`not sel`, which are the three most common mistakes.

`.to_numpy()` is the escape hatch. It returns column arrays, not objects. A user
who then writes a Python loop at least loops over arrays, and the profiler will
show them the cost. Do **not** provide `.for_each()`, `.map()`, or
`.iter_chunks()`. Each one is an invitation.

### 1.5 Terminal operations

```python
sel.count()                     -> int
sel.is_empty()                  -> bool
sel.any()                       -> bool          # short-circuits
sel.to_numpy(["health", "q"])   -> dict[str, np.ndarray]   # zero-copy view
sel.sum("health")               -> int
sel.histogram("unit_type")      -> np.ndarray
sel.explain()                   -> str
```

`.explain()` must show the plan, the pruning result per level, and the estimated
and actual row counts. Model the output on `EXPLAIN ANALYZE`.

### 1.6 Applying a verb

Verbs are methods on the selector. This matches Django and reads well.

```python
report = (units.faction(me) & (f.health < 40)).move_to(w.home_tile)

print(report.affected)            # 812
print(report.rejections)          # {"NO_PATH": 40, "IMMOBILE": 7}
retry = report.rejected.move_to(w.fallback_tile)   # a lazy selector
```

`report.rejected` is a selector, not a list of IDs. This follows context brief
item 10, and it lets the user chain without ever seeing an entity ID.

### 1.7 Failure modes of this API

- **Selector node explosion.** A user builds a selector inside a Python loop and
  creates ten thousand `|` nodes. Mitigation: cap the node count, and memoise on
  the structural hash so that repeated subtrees collapse. Also offer
  `f.faction.is_in([...])` so that the natural form is one node.
- **Silent full scan.** A predicate on a field with no summary cannot prune. The
  user sees no error, only slowness. Mitigation: `.explain()` must label such
  nodes clearly, and a debug build should warn on the first occurrence.
- **Empty selector.** A verb on an empty set must return `affected=0` and must
  not raise. Provide `.expect_nonempty()` for users who want the error.
- **Stale selector.** A selector built in frame 10 and used in frame 40 is still
  valid, because it is a description, not a result. But its *result* differs.
  Document this. It surprises people who expect a snapshot.

---

## 2. Query planning and execution

Treat this exactly as a database problem. The vocabulary already exists, and
using it will keep the design honest.

### 2.1 Three-valued pruning

Give every predicate node two methods.

```rust
enum Trilean { All, None, Some }

trait Predicate {
    /// Evaluate against an L1 or L2 summary cell.
    fn eval_summary(&self, s: &Summary) -> Trilean;
    /// Evaluate against a block of L0 rows. Writes into a bitmask.
    fn eval_leaf(&self, block: &Block, out: &mut Mask);
}
```

Combine with Kleene logic during the descent.

| op  | rule |
|-----|------|
| AND | `None` if any child is `None`; `All` if all children are `All`; else `Some` |
| OR  | `All` if any child is `All`; `None` if all children are `None`; else `Some` |
| NOT | `All` <-> `None`; `Some` stays `Some` |

The descent then does this at each cell.

- `None`: skip the whole subtree. Do not touch its children.
- `All`: mark the whole block as a member. Do not descend. This is the run
  container case, and it is common after a faction filter.
- `Some`: descend to the children, or to L0 if this is L1.

This avoids a rewrite to disjunctive normal form. DNF can blow up
combinatorially on a user-built tree, and users will build strange trees.

### 2.2 Selectivity

Here is a result worth headlining in the ADR.

If a summary cell holds a **histogram** over a field, then the count of rows in
that block that satisfy an equality predicate on that field is **exact**, not
estimated. The same is true for range predicates over a bucketed histogram, up
to the bucket width. For a bitmask summary such as a faction mask, the answer
`None` is exact and the answer `Some` is exact as "at least one".

So the pyramid is not only an index. It is also the statistics catalogue that a
query planner needs, and it is always current, because the dirty-pyramid update
maintains it.

Conjunctions still need an assumption. Two predicates over different fields are
not independent in a real world map. Recommend the standard independence
estimate `sel(A and B) = sel(A) * sel(B)` as the starting point, with two
protections.

- Clamp the estimate to `min(sel(A), sel(B))`, which is an exact upper bound.
- Track actual against estimated in `.explain()`, so that a bad estimate is
  visible rather than mysterious.

Do not build a correlation model in version 1. Measure first.

### 2.3 Plan choice

Three strategies. Choose per query, from the estimate.

1. **Hierarchical descent.** Start at L2. Prune. Descend. Use when the estimated
   selectivity is low to medium and at least one predicate is prunable.
2. **Flat SIMD scan.** Ignore the pyramid. Sweep the L0 arrays with vector
   comparisons. Use when the estimated selectivity is high, or when no predicate
   is prunable. A branch-free scan over a `u8` column runs at memory bandwidth.
   Above some selectivity, the descent's branching costs more than the scan it
   saves.
3. **Index probe.** Keep a small number of explicit inverted indexes, for
   example faction to chunk list, and unit type to chunk list. Use when a
   leading equality predicate has very low selectivity.

The switch points are tunable constants. Do not assert numbers in the ADR.
Write the benchmark and measure. As a starting hypothesis, use a flat scan above
about 25 percent estimated selectivity, and an index probe below about 1 percent.

### 2.4 Predicate pushdown

Push each predicate to the highest level at which it can be evaluated. This is
the same rewrite that Polars performs, where filters move down toward the scan.
Here the direction is up the pyramid, but the principle is identical: evaluate
the cheapest and most selective test first, on the fewest rows.

Order the conjuncts by `estimated_selectivity / estimated_cost`, ascending. Test
the cheapest and most selective conjunct first. Short-circuit as soon as a
conjunct returns `None`.

### 2.5 Caching

The context brief says commands queue during the Python phase and seal at the
frame barrier. That gives a very strong and very simple caching rule.

**The write model does not change during the Python phase. So a selector
resolved during the Python phase stays valid for the whole Python phase. Clear
the cache at the frame barrier.**

This needs no invalidation logic, no epoch counters, and no dependency tracking.
It is correct by construction. It is also the case that matters, because the
common pattern is to resolve the same selector several times in one frame.

Key the cache on a 128-bit structural hash of the normalised expression tree.
Normalise by sorting the children of commutative nodes on their own hashes. Use
128 bits so that a collision check is unnecessary in practice. Store the tree
next to the entry anyway, and compare on a hit if you want certainty.

A cross-frame cache is possible later. It needs the dirty pyramid as its
invalidation source: an entry stays valid while no dirty bit under its touched
subtree is set. Do not build this in version 1. It adds a whole class of
correctness bug for an unmeasured gain.

### 2.6 The snapshot versus live problem

The context brief does not address this, and it is a determinism hole.

A verb may take a selector as a parameter. Consider `units.faction(me).attack(
units.faction(enemy) & (f.health < 20))`. When does the parameter selector
resolve?

- **Snapshot semantics.** Resolve every selector against the sealed pre-frame
  state. The result does not depend on which commands already ran this frame.
- **Live semantics.** Resolve at apply time. The result depends on earlier
  commands in the same frame.

These give different answers. Live semantics reintroduces the order dependence
that context brief item 9 exists to remove. It also blocks parallel command
application, because a command's *read set* would then depend on another
command's writes.

**Recommend snapshot semantics as the only semantics for version 1.** Say so in
the documentation, in bold. If live semantics is ever needed, add it as an
explicit `.live()` marker that forces the command into a serial phase.

### 2.7 Short-circuiting and order

`.any()` and `.first_n(k)` can stop early. `.first_n(k)` is only deterministic
if the iteration order is defined. Define a canonical order.

- Tiles: Morton (Z-order) index over the axial coordinates.
- Units: `(archetype_id, chunk_id, slot_index)`.

Document this order as stable within a released version, and **not** stable
across versions. If you promise cross-version stability you can never change the
chunk allocator.

---

## 3. Result representation

### 3.1 The candidates

**Sorted ID vector.** A `Vec<u32>`. Simple. Compact when sparse. Set algebra is
a branchy merge. Random membership tests need a binary search.

**Dense bitset.** One bit per possible ID. Set algebra is branch-free word
operations. It is the fastest option on dense data, but it is more than ten
times slower on sparse data, because it must scan empty words. ([Chambi et al.,
"Better bitmap performance with Roaring
bitmaps"](https://arxiv.org/pdf/1402.6407)) At 16.7 million tiles a dense L0
bitset is about 2.1 MB. That is affordable once, but not per intermediate node
of a large expression tree.

**Roaring bitmap.** A hybrid. It splits the key space into chunks of 2^16. A
dense chunk uses a bitmap container. A sparse chunk uses a packed sorted array
of 16-bit integers. A third container type stores runs. Roaring is four to five
times faster than the WAH and Concise compressed schemes for intersections at
all tested densities, and it compresses better. ([Chambi et
al.](https://arxiv.org/pdf/1402.6407); [Lemire et al., "Consistently faster and
smaller compressed bitmaps with Roaring"](https://arxiv.org/pdf/1603.06549);
[implementation
paper](https://arxiv.org/pdf/1709.07821))

The Roaring papers also state the honest conclusion: no single structure is best
for all data and all applications.

### 3.2 Recommendation

**Build a purpose-specific two-level mask. Do not use a general Roaring library
on the hot path.**

The reasoning is that Roaring's high 16 bits are an arbitrary split of an
arbitrary key space. Here the key space is not arbitrary. The storage layout
already defines the natural split.

For units:

```rust
struct UnitSet {
    /// Sorted by chunk_id. One entry per chunk with at least one member.
    entries: Vec<ChunkEntry>,
}
struct ChunkEntry {
    chunk_id: u32,
    kind: MaskKind,   // Full | Bits(SmallVec<[u64; 16]>)
}
```

- `ChunkId` plays the role of Roaring's high bits, and it is *already* the unit
  of storage. An archetype chunk is 16 KB, so it holds a few hundred to about a
  thousand units. A full bitmask for 1024 slots is 128 bytes.
- Iteration follows chunk order, which is memory order. A verb then walks the
  struct-of-arrays columns in sequence, with no gather. This is the property
  that a sorted ID vector loses.
- `MaskKind::Full` is Roaring's run container, specialised to the one run that
  actually occurs. After a faction filter or a hierarchical descent, "every unit
  in this chunk matches" is very common. It makes the common case free.
- Set algebra is a merge join on `chunk_id`, then branch-free `u64` operations.

For tiles:

```rust
struct TileSet {
    l2: BitVec,                       // one bit per L2 cell
    l1: HashMap<L2Id, BitVec>,        // sparse: only for partial L2 cells
    l0: HashMap<L1Id, Box<[u64; N]>>, // sparse: only for partial L1 cells
    full_l1: BitVec,                  // "every tile under this L1 cell matches"
}
```

This mirrors the pyramid exactly, so a descent writes its result in the same
shape it produced. There is no conversion step.

**Export a sorted `u32` array only at the FFI boundary**, because NumPy needs a
flat array. Never use it as the internal working form.

**Use a general Roaring library for cold, sparse side tables**: tag membership,
upgrade-set membership, and named-entity lookup. There the key space really is
arbitrary and the access is not on the hot path.

**Caveat.** The claim that a purpose-built mask beats general Roaring here is a
design argument, not a measurement. Write the benchmark before you commit. The
benchmark should cover: an intersection of two 1-percent-dense sets, an
intersection of two 40-percent-dense sets, a union of ten sets, and a full
iteration with a column read.

---

## 4. Summary fields and predicate vocabulary

Context brief item 6 says the summaries carry "the fields selectors filter on".
That definition is circular. It will drift, because the summary schema and the
predicate list will live in different files and change at different times.

**Recommend one declarative field registry that generates both.**

```rust
declare_fields! {
    units {
        // name      type   monoid            summary slot
        faction:     u8   => BitOr<u64>     @ L1, L2,   // faction presence mask
        unit_type:   u16  => Histogram<64>  @ L1, L2,
        health:      u16  => (Min, Max, Sum)@ L1, L2,
        capabilities:u64  => BitOr<u64>     @ L1, L2,
        order_state: u8   => Histogram<16>  @ L1,
        cargo:       u16  => Sum            @ L1, L2,
        name:        Str  => None,                       // leaf-only
    }
    tiles {
        terrain:     u8   => Histogram<32>  @ L1, L2,
        owner:       u8   => BitOr<u64>     @ L1, L2,
        elevation:   i16  => (Min, Max)     @ L1, L2,
        upgrade:     u16  => (Histogram<32>, Count) @ L1, L2,
        fertility:   u8   => (Sum, Count)   @ L1, L2,    // mean at read
    }
}
```

The macro generates four things from one declaration.

1. The L0 accessor for the struct-of-arrays column.
2. The monoid combine function used by the dirty-pyramid update.
3. The summary struct layout and its slot offsets.
4. The Python-visible predicate constructors, and their `eval_summary`
   implementations.

This makes the invariant of context brief item 4 mechanical: an attribute
appears at L1 or L2 **only** if it declares an associative combine with an
identity. And it makes item 6 mechanical too: a predicate can prune **only** if
its field declares a monoid at that level.

A field with `=> None` still gets a predicate. That predicate returns
`Trilean::Some` at every summary, so the descent will not prune on it. That is
correct, and `.explain()` should say so.

### Which fields make pruning effective

Rank by pruning power, which is the fraction of blocks a predicate can reject.

**Very high value.**

- **Faction presence mask (`BitOr<u64>`).** Almost every gameplay query filters
  on ownership. A world is spatially segregated by faction, so this rejects most
  blocks. It also makes "contested block" a single bitwise `AND` and a
  population count. That directly accelerates combat target acquisition.
- **Unit presence count (`Sum` of unit count).** Rejects every empty block. Most
  of a 4096 by 4096 map is empty of units at any time. This is the single
  cheapest and highest-value summary.
- **Terrain histogram.** Terrain is highly spatially clustered, so a histogram
  rejects most blocks for a terrain predicate.

**High value.**

- **Capability mask (`BitOr<u64>`).** Rejects blocks that contain no unit with a
  required capability. This is why capabilities must be a small fixed bitmask.
- **Health min and max.** Range predicates on health are common. Min and max
  give exact `All` and `None` answers for one-sided ranges.
- **Elevation min and max.** The same, for terrain queries and line of sight.

**Medium value.**

- **Upgrade or improvement count.** Sparse, so `count == 0` rejects most blocks.
- **Order-state histogram.** Useful for "all idle units". Changes often, so it
  raises the dirty-update cost.

**Low value. Do not summarise.**

- Anything with high cardinality and low clustering, such as a per-unit
  identifier, or a name.
- Anything that changes every frame for every entity, such as an exact position
  within a block. The dirty-update cost exceeds the pruning gain.

**Budget note.** Every summary field costs memory at L1 and L2, and it costs
update time on every dirty cell. A 64-bucket histogram at `u16` per bucket is
128 bytes per cell. Set a hard byte budget per summary cell, for example 256
bytes, and make the macro fail the build if the declaration exceeds it. This
forces the trade-off to be explicit.

---

## 5. The verb vocabulary

### 5.1 What real games expose

**OpenRA.** Orders carry an `OrderString` such as `"Move"`, `"Attack"`, or
`"Deploy"`. An `OrderManager` distributes, processes, and synchronises them.
Frame-based scheduling makes every client process the same orders on the same
frame. ([OpenRA order
processing](https://deepwiki.com/OpenRA/OpenRA/2.3-order-processing-and-networking))
A reinforcement-learning wrapper exposes a small, higher-level set:
`deploy_unit`, `repair_building`, `guard_target`, `set_stance`.
([OpenRA-RL](https://openra-rl.dev/docs/agents/))

The lesson: the wire-level order set is small, and each order is a string plus a
target plus a few flags. Complexity lives in the trait that receives the order,
not in the order set.

**Paradox (CK3, EU4, Stellaris).** The scripting layer exposes hundreds of
"effects". But almost all of them are one of a few shapes: set a flag, add a
modifier, change a scalar, change ownership, spawn, or destroy. The apparent
size comes from the number of *fields*, not from the number of *operations*.

The lesson: a generic `set_field` and `add_modifier` pair replaces a hundred
named effects, provided the field registry is data.

**Civilization.** Unit commands are a small fixed set: move, attack, fortify,
build improvement, found city, upgrade, disband, and a few special ones.

**Factorio.** There is almost no unit command layer. Instead the player edits the
*world* (place entity, remove entity, set recipe) and the simulation runs. This
is worth noting, because it suggests that tile verbs matter as much as unit
verbs.

**Dwarf Fortress.** The player designates *areas* for work and sets *priorities*.
The player does not command individual dwarves. This is the purest set-valued
model in the survey, and it is the closest match to this project's philosophy.

### 5.2 Proposed verb set

Organised by conflict domain. `L` marks load-bearing verbs that need genuinely
new Rust code. `D` marks verbs that are an existing implementation with different
data or a different parameter.

#### Domain A: unit spatial state (writes position, occupancy index)

| # | Verb | | Note |
|---|------|---|------|
| 1 | `move_to(dest, stance=?)` | **L** | One hierarchical flow field for the whole set. |
| 2 | `flee_from(sel)` | **L** | Inverse flow field. Different gradient, same machinery. |
| 3 | `move_along(waypoints)` | D | `move_to` in sequence. `patrol` is this with a cycle flag. |
| 4 | `follow(sel, distance)` | D | `move_to` with a target that updates. |
| 5 | `teleport(dest)` | **L** | No pathing, but it needs the same occupancy-index update and collision resolution. |
| 6 | `stop()` | D | Clears the order state. |
| 7 | `enter(sel)` | **L** | Moves a unit out of the tile grid into a container. Changes the occupancy invariant. |
| 8 | `exit()` | D | The inverse of `enter`. |

`attack_move` is **not** a verb. It is `move_to(dest, stance=Stance.AGGRESSIVE)`.
`guard` is `follow(sel, stance=Stance.DEFENSIVE)`. Say this explicitly in the
ADR, because these are the two commands that reviewers will ask for by name.

#### Domain B: unit vital state (writes health, morale, stamina)

| # | Verb | | Note |
|---|------|---|------|
| 9 | `adjust_vital(field, delta, clamp)` | **L** | One verb. `damage` is a negative delta. `heal` and `repair` are positive. Do not ship three verbs. |
| 10 | `attack(sel, mode)` | **L** | Batched target acquisition. See section 8.1. |
| 11 | `bombard(tile_sel, profile)` | **L** | Area effect. Writes both unit vitals and tile state. |
| 12 | `kill()` | D | `adjust_vital(health, -inf)`, but it is worth a name for clarity and for the event it emits. |

#### Domain C: unit intent and order state

| # | Verb | | Note |
|---|------|---|------|
| 13 | `set_stance(stance)` | D | A scalar write. |
| 14 | `set_priority(n)` | D | A scalar write. |
| 15 | `queue(verb, params)` | **L** | Appends to a per-unit order queue. Needs a queue structure and a bounded size. |
| 16 | `clear_queue()` | D | |

#### Domain D: unit inventory and cargo

| # | Verb | | Note |
|---|------|---|------|
| 17 | `transfer(resource, amount, to)` | **L** | Conserving. See section 8.3. |
| 18 | `load(sel)` / `unload()` | D | `transfer` with a container target. |

#### Domain E, F, G: tile state

| # | Verb | | Note |
|---|------|---|------|
| 19 | `set_terrain(type)` | **L** | Must invalidate path caches and mark the dirty pyramid. |
| 20 | `build(upgrade, cost_pool)` | **L** | Validation, cost, partial failure. See section 8.4. |
| 21 | `demolish()` | D | `build` with a null upgrade. `raze` is this plus a vital effect. |
| 22 | `claim(faction)` | **L** | Ownership change. Triggers contiguity and border recomputation. |
| 23 | `unclaim()` | D | |
| 24 | `harvest(resource, into)` | **L** | Reads tiles, writes a faction pool. Set-valued sum. |
| 25 | `reveal(faction, radius)` | **L** | Visibility stamping. See section 8.2. |
| 26 | `conceal(faction)` | D | The inverse mask operation. |

#### Domain H: lifecycle

| # | Verb | | Note |
|---|------|---|------|
| 27 | `spawn(type, count, in=tile_sel)` | **L** | Placement allocation. See section 8.5. |
| 28 | `despawn()` | **L** | Chunk compaction, generation increment, index repair. |
| 29 | `transform(new_type)` | **L** | Changes `UnitType(u16)` in place. Stays in the same archetype. This is the verb that justifies the data-driven type system. |
| 30 | `split(n)` / `merge()` | **L** | Only if the design has unit stacks. Decide early; retrofitting stacks is expensive. |

#### Domain I and J: faction state and metadata

| # | Verb | | Note |
|---|------|---|------|
| 31 | `set_owner(faction)` | D | A scalar write plus an index update. |
| 32 | `grant_upgrade(id)` / `revoke_upgrade(id)` | **L** | Rewrites the interned upgrade-set ID and re-derives the capability mask. |
| 33 | `add_modifier(id, duration)` / `remove_modifier(id)` | **L** | Writes the sparse modifier table. Needs expiry handling. |
| 34 | `tag(id)` / `untag(id)` | D | A bitplane write. No simulation effect. |

**Count: 34 named verbs, of which 17 are load-bearing.** Several load-bearing
entries are optional (`split`/`merge`, `queue`), so a version 1 core is closer
to twelve. This matches the context brief's estimate of about 30 verbs, and it
shows that the implementation cost is roughly a third of the surface area.

### 5.3 Orthogonality tests

Apply these three tests before adding any verb.

1. **The symmetric-pair test.** If a proposed verb is the mirror of an existing
   verb, it is a sign, not a verb. `heal` and `damage` are one verb.
2. **The parameter test.** If a proposed verb differs from an existing verb only
   in a constant, it is a parameter. `attack_move` is a stance parameter.
3. **The composition test.** If a proposed verb is two existing verbs in
   sequence, it is a composite. Handle it at the composite-verb layer (see
   [Section 9](#9-the-extensibility-ceiling)), not with new Rust.

---

## 6. Data-driven types

### 6.1 Prior art

**Factorio.** Prototypes are defined in a data stage that runs before the game.
`data.raw` is a dictionary from prototype type to a dictionary from name to
prototype. `data.raw` exists **only** during the data stage. At run time the code
reads processed, read-only values through typed accessors such as
`LuaEntityPrototype`. ([Factorio data
docs](https://lua-api.factorio.com/latest/types/Data.html), [modding
tutorial](https://wiki.factorio.com/Tutorial:Modding_tutorial/Gangsir))

This is exactly the model to copy. There is a mutable authoring phase, a
one-time bake, and then an immutable run-time table. The bake step is where you
assign the dense `u16` indices, validate references, and lay out the
struct-of-arrays stat table.

**RimWorld.** Content is defined in XML `Def` files that map onto C# classes. A
`DefDatabase` provides name lookup. `Def` files support abstract bases and
inheritance, so common values are not repeated.
([RimWorld XML defs](https://rimworldwiki.com/wiki/Modding_Tutorials/XML_Defs),
[abstracts and
inheritance](https://spdskatr.github.io/RWModdingResources/abstracts.html))

The lesson: give the authoring format inheritance. Without it, a content author
copies fifty fields to change one, and the content set becomes unmaintainable.

**Paradox.** Content is script files that the engine parses at load. Almost all
gameplay content is data. This is the extreme end, and it is why Paradox games
have very large mod communities.

### 6.2 The stat table

```rust
/// Immutable after the bake step. Shared by every unit.
struct UnitTypeTable {
    count: usize,
    move_speed:    Vec<u16>,     // fixed point, 1/256 tiles per tick
    attack:        Vec<u16>,
    defence:       Vec<u16>,
    max_health:    Vec<u16>,
    capabilities:  Vec<u64>,     // the base capability mask for this type
    terrain_cost:  Vec<u8>,      // [type][terrain], row-major, flattened
    // ...
}
```

Access is `table.attack[unit.unit_type as usize]`. This is a gather, not a
sequential read. That is the one real cost of this design.

**Mitigation.** Most chunks are type-homogeneous or nearly so, because units are
usually spawned in batches. Add a per-chunk "dominant type" field and a
"homogeneous" flag. When a chunk is homogeneous, hoist the stat lookup out of the
inner loop entirely. This turns the gather into one load per chunk. Measure the
homogeneity rate in a real scenario before you rely on this.

### 6.3 The u64 ceiling

The context brief says upgrades are "a `u64` bitmask plus a sparse modifier
table". **This conflates two different things, and only one of them fits in 64
bits.**

- An **upgrade** is authored content. A content author will define hundreds or
  thousands of them. CK3 has thousands of traits and perks.
- A **capability** is a predicate that hot code tests. `CAN_SWIM`, `IS_RANGED`,
  `IGNORES_ZOC`. There are few of these, because each one is a branch that
  someone wrote in Rust.

**Recommendation: store both, and derive one from the other.**

```rust
struct UnitRow {
    unit_type:   u16,           // index into the stat table
    upgrades:    UpgradeSetId,  // u32, interned; authoritative; unbounded content
    capabilities: u64,          // derived; hot; tested by selectors and verbs
}
```

- `UpgradeSetId(u32)` interns into a deduplicated table of upgrade sets. Real
  populations have few distinct sets, because units are upgraded in batches. The
  table is a flyweight. A grant is "look up or insert the union", which is a hash
  lookup, not a bit operation.
- `capabilities: u64` is recomputed whenever `upgrades` or `unit_type` changes.
  It is `type_base_caps | OR(caps of each upgrade in the set)`. The capability
  bit assignment is owned by the Rust code, so it is bounded by construction.

This gives unlimited upgrade content and a fixed-size hot mask. The `u64`
ceiling then applies only to capabilities, where 64 is genuinely enough, and
where exceeding it means someone wrote 65 special branches, which is a design
smell worth catching.

**Alternatives considered.**

- `[u64; 4]` fixed-width mask, 256 bits. At one million units this is 32 MB.
  Affordable, and it keeps the pure-bitmask model. But it only moves the ceiling
  from 64 to 256, and it costs four times the bandwidth in every hot loop that
  tests one bit. Reject.
- A Roaring bitmap per unit. Correct and unbounded, but it puts a heap
  allocation and a pointer chase in the entity row. That breaks the chunked
  struct-of-arrays layout. Reject for the hot path; it is fine as the interned
  set table's internal representation.

**Failure mode of the interning scheme.** If content lets each unit accumulate a
different upgrade set, the intern table grows toward one entry per unit, and the
memory saving disappears. Monitor the intern table size and its hit rate. If the
distinct-set count exceeds a threshold, that is a content design problem, and the
engine should say so.

### 6.4 Modifier stacking and determinism

**Floating-point addition is not associative.** If modifiers are collected from a
hash map, or accumulated across threads in a non-fixed order, the result differs
between runs and between machines. This interacts directly with the context
brief's open question about the determinism target.

**Recommendation: use fixed-point integer arithmetic for all stat maths.** Use
`i32` in units of 1/1024. Integer addition is associative and commutative, so
accumulation order stops mattering. This removes the whole problem class instead
of managing it.

If floating point is unavoidable for some quantity, sort the modifiers on a
stable key of `(category, source_id)` and fold in that order. Never fold in hash
order.

**Recommendation: use a fixed pipeline, in the Paradox style.** Within a
category, modifiers add. Across categories, they multiply. Community
documentation of the Clausewitz behaviour describes the shape as
`output = base × additive_sum × multiplicative_product`, where modifiers in the
same category add and modifiers in different categories multiply.
([Paradox forum discussion of additive versus multiplicative
modifiers](https://forum.paradoxplaza.com/forum/threads/additive-bonuses-vs-multiplicative-bonuses.1144836/),
[multiplicative modifier
thread](https://forum.paradoxplaza.com/forum/threads/add-multiplicative-modifiers.1905655/))

Define the stages in the schema, not in the data.

```
stage 0  base        = stat_table[unit_type][field]
stage 1  flat        = base + sum(flat modifiers)         // integer, order-free
stage 2  percent     = flat * (1024 + sum(pct modifiers)) / 1024
stage 3  multiplier  = fold(mult modifiers, in declared category order)
stage 4  clamp       = clamp(result, field_min, field_max)
```

Because the stage order comes from the schema, and because each stage sums
integers, the result does not depend on the order in which modifiers were
applied or discovered. That is the determinism property you want.

**Memoise the resolved stat block** on the key
`(unit_type, capabilities, modifier_set_id)`. The number of distinct combinations
present at any time is small, so this converts a per-unit fold into a table
lookup.

---

## 7. Verb registry and dispatch

### 7.1 Dispatch mechanism

Three options are usually compared: a static enum with a `match`, trait objects,
and a function table.

**The comparison does not matter here, and the ADR should say so.** Dispatch
happens once per `(selector, verb)` command, not once per entity. A frame might
carry a few thousand commands. Even a virtual call costs a few nanoseconds. That
is far below the noise floor of a frame that touches millions of entities.

So choose on ergonomics. **Recommend a function table.**

```rust
pub struct VerbDescriptor {
    pub name:   &'static str,
    pub id:     VerbId,          // u16
    pub domain: Domain,          // Units | Tiles
    pub params: ParamSchema,     // validated at command-issue time, in Python
    pub access: AccessSet,
    pub apply:  fn(&mut World, &ResolvedSet, &Params) -> VerbReport,
}

pub struct VerbRegistry { verbs: Vec<VerbDescriptor>, by_name: HashMap<&'static str, VerbId> }
```

This gives four things that a static enum does not.

- Runtime registration, which tests need for stub verbs.
- Introspection, so that the Python bindings and the documentation generate from
  one source.
- A parameter schema that validates in Python at issue time, which produces a
  good error message instead of a Rust panic.
- A path to plugin-registered verbs later, with no change to the call site.

### 7.2 Conflict domain declaration

Bevy derives parallelism from declared data access. If a system accesses data
mutably, no other system that reads or writes that data may run at the same
time; such systems are incompatible. Systems that do not conflict run in parallel
automatically. ([Bevy ECS system
docs](https://docs.rs/bevy_ecs/latest/bevy_ecs/system/index.html), [Bevy cheat
book on exclusive systems](https://bevy-cheatbook.github.io/programming/exclusive.html),
[Bevy parallelism
discussion](https://github.com/bevyengine/bevy/discussions/2875))

Copy this rule, and add a dimension that Bevy does not have.

```rust
pub struct AccessSet {
    pub reads:  ComponentMask,   // u64 bitmask over component ids
    pub writes: ComponentMask,
    pub region: RegionScope,
}

pub enum RegionScope {
    /// Confined to the bounding region of the resolved selector.
    Local,
    /// Touches one faction's global state.
    Faction,
    /// Touches everything. Runs alone.
    World,
}
```

Two commands may run in parallel if **either** condition holds.

1. Their component accesses do not conflict, in the Bevy sense:
   `writes(A) ∩ (reads(B) ∪ writes(B)) = ∅` and the same with A and B swapped.
2. Both are `Local` and their resolved regions are disjoint.

**The important difference from Bevy.** Bevy's analysis is static, because a
system's access is fixed at compile time. Here the *region* is dynamic, because
it comes from the resolved selector. So the scheduler must run after selector
resolution and must build the conflict graph each frame. This is the interesting
design point, and the ADR should call it out.

The cost is real but bounded. With a few thousand commands, a pairwise conflict
check is a few million bitmask comparisons, which is under a millisecond.
Reduce it further by bucketing on the L2 cell first: two commands whose L2 masks
do not intersect cannot conflict on `Local` region, and that test is one bitwise
`AND`.

**Failure modes.**

- **Under-declaration is unsound.** A verb that writes a component it did not
  declare causes a data race. Mitigation: in debug builds, wrap component access
  in a tracking guard and assert that every access was declared. Run this in CI
  on every verb.
- **Over-declaration is slow and silent.** A verb that declares `World` scope
  because it was easier serialises the frame. Mitigation: report the achieved
  parallelism per frame, and list the top serialising verbs. Make the cost
  visible.
- **Region computation cost.** The bounding region of a resolved selector must be
  cheap. It is, if the result representation is the hierarchical mask from
  [Section 3](#3-result-representation): the L2 mask *is* the region.

---

## 8. Set-valued verbs that enable better algorithms

The context brief gives the flow-field example. Here are five more. Each one is a
case where the set-valued form is not just a batched loop, but a different and
asymptotically better algorithm, or a case where the per-entity form is *wrong*.

### 8.1 `attack` — contested-block detection

A naive loop does nearest-enemy search for each of N attackers over M possible
targets. That is O(N·M), or O(N log M) with a spatial structure that must be
rebuilt each frame.

The set-valued form uses the pyramid. The L1 faction presence mask is a `u64` per
cell. A block can contain combat only if it holds units of two hostile factions.
That test is one bitwise `AND` and a population count per L1 cell.

```
contested = l1.faction_mask[c] & hostile_mask(attacker_faction)
if contested == 0 { skip the whole block }
```

For a typical front line, well under one percent of L1 cells are contested. So
the search cost drops from "every attacker against every target" to "every
attacker in a contested block against every target in the same block and its
neighbours". The per-tile occupancy index then resolves the final targets with a
local ring scan.

The pyramid pays for itself here, and this is a stronger argument for it than the
selector pruning case, because it is an algorithm change, not a constant-factor
win.

### 8.2 `reveal` — visibility stamping without overdraw

Naive: for each of N units, write every tile in a radius-r disc. Cost is
O(N·r²) writes, with heavy overdraw when units cluster. At 5000 units and r=8
that is about a million writes, most of them redundant.

Set-valued form, using a scanline delta buffer.

1. Collect the tile coordinates of the whole set. Sort by row. The result set is
   already in Morton order, so this is close to sorted.
2. For each unit and each row of its disc, write `+1` at the row's start column
   and `-1` one past its end column, into an `i16` delta buffer. Cost is
   O(N·r) writes, not O(N·r²).
3. Prefix-sum each row once. Cost is O(tiles touched).
4. Convert to a bitplane with a word-wise comparison against zero.

Overdraw disappears, because overlapping discs simply add their deltas. The
result is exact, and the cost drops by a factor of about r. It is also
deterministic regardless of unit order, because integer addition commutes.

This algorithm is not expressible one unit at a time. It needs the whole set.

### 8.3 `transfer` — conservation and correct integer rounding

Naive per-entity loop: for each receiver, take what it needs from the pool. The
result depends on iteration order. The last receivers may get nothing. Worse,
under parallel application the pool can go negative.

Set-valued form.

1. One pass sums total demand `D` across the receiver set.
2. Compute the scale `s = min(1, available / D)` once.
3. One pass writes `floor(demand_i * s)` to each receiver.
4. Distribute the remainder by the **largest-remainder method**, in the canonical
   iteration order, until the pool is exactly empty.

This conserves exactly, it is independent of iteration order except for the
deterministic remainder step, and it is fair. Step 4 is impossible in a
per-entity loop, because it needs all the remainders at once. This is a case
where the set-valued form is not faster but *correct*, and the loop form is
subtly wrong.

### 8.4 `build` — partial failure with a shared budget

The same shape as `transfer`, applied to a shared cost pool.

1. Count the candidate tiles and compute the total cost.
2. If the budget covers it, apply to all.
3. If not, sort by a declared priority key, take the affordable prefix, apply,
   and return the remainder as a rejected selector with the reason
   `INSUFFICIENT_FUNDS` and its count.

This implements context brief item 10 directly. It also makes the result
independent of which thread got which tile, because the priority key is data, not
schedule.

### 8.5 `spawn` — placement without a rejection loop

Naive: pick a random tile, test whether it is free, retry on failure. Worst-case
cost is unbounded, and it is not deterministic under parallelism.

Set-valued form, using the L1 free-capacity summary.

1. Descend the pyramid over the target tile selector. Each L1 cell reports its
   free capacity from the existing occupancy summary.
2. Split N across the eligible L1 cells by a single deterministic multinomial
   draw, seeded from `(frame, command_seq)`. Use the largest-remainder method for
   the integer split.
3. Fill each L1 cell with one linear scan of its free tiles.

Work is bounded by `O(N + eligible cells)`. There is no retry loop, the result is
deterministic, and the L1 cells fill in parallel because they are disjoint.

### 8.6 `adjust_vital` — the trivial case that still matters

A signed delta over a `u16` column, clamped. Algorithmically this is nothing. But
because the set is a chunk mask and the storage is struct-of-arrays, the compiler
auto-vectorises it. That is roughly an 8x to 16x gain over a scalar loop, plus
the removal of all Python interpreter overhead.

Include this example in the ADR precisely *because* it is trivial. It shows that
the set-valued form has no floor: even the simplest verb benefits from the
representation.

---

## 9. The extensibility ceiling

The context brief defers a modding DSL and a bytecode VM. That is correct for
version 1. But the ADR should record the staged path, so that the deferral is a
decision and not an omission.

### Stage 1 — a fixed verb set, parameterised by data

**Cost: zero extra. It is the plan already.**

**Ceiling:** a user cannot express a rule the engine does not have. They compose
between frames, from Python, by issuing several commands.

**How far it goes:** further than it sounds. `set_field`, `adjust_field`,
`add_modifier`, and `transform` between them cover most "new rule" requests,
because the *fields* are data.

### Stage 2 — composite verbs

A named, serialisable sequence of `(verb, parameter binding)` applied to one
selector. The access set is the union of the members. The scheduler needs no
change.

```python
w.define_composite("raid", [
    ("move_to",       {"dest": "$target"}),
    ("attack",        {"sel": "$defenders", "mode": "focus"}),
    ("adjust_vital",  {"field": "stamina", "delta": -20}),
])
```

**Cost estimate: one to two weeks.** It is a data structure, a validator, and a
loop. There is no new evaluation machinery.

**Value:** covers a large fraction of "I need a new verb" requests, because most
of them are really "I need these three verbs together, atomically, in a defined
order".

**Recommend this as the first extension, and only after real usage shows which
compositions repeat.**

### Stage 3 — a vectorised expression DSL for data

Let the user write arithmetic over component columns, and evaluate it in the same
way the selector engine already evaluates predicates.

```python
sel.set_field("morale", ct.expr(f.morale + 0.1 * f.nearby_allies - f.damage_taken))
```

**Cost estimate: four to eight weeks.**

The key point: this is cheap **relative to its value**, because the selector
engine already has an expression tree, a Python builder, operator overloading, a
type checker, and a vectorised evaluator. This stage extends the node set from
predicates to arithmetic. It reuses everything else.

**Risks:** division by zero, overflow, and NaN in user expressions. Handle them
with a defined saturating or clamping semantics, decided once and documented.
Never let a user expression panic the simulation.

**This is the highest value-to-cost step in the whole ladder.** Note it in the
ADR even while deferring it, because it argues for building the selector
evaluator in a way that generalises: separate the node types from the boolean
combination logic.

### Stage 4 — a bytecode VM

**Cost estimate: three to six months**, plus ongoing cost for determinism
guarantees, sandboxing, a resource budget, error reporting, and a debugger. The
debugger is not optional. A VM without one produces bug reports that no one can
act on.

**Recommend deferring this indefinitely.**

**The design note that matters most.** If you ever build a VM, make each opcode
operate on a **whole column**, in the style of APL or NumPy. A scalar,
per-entity VM re-creates exactly the cost this architecture exists to remove: an
interpreter dispatch per entity per operation. A vectorised VM amortises the
dispatch over an entire chunk, so the interpreter overhead becomes negligible.
This single decision separates a viable VM from an unviable one, and it must be
made at the start.

### The alternative: a native plugin ABI

A user compiles a Rust `cdylib` that registers verbs into the `VerbRegistry`.

**Cost estimate: two to four weeks** for the loading mechanism.

**Advantage:** full speed, and no new language.

**Serious drawback:** the Rust ABI is not stable, so a plugin must be compiled
against the exact engine version and the exact compiler version. It also gives a
plugin the power to break determinism and to corrupt memory, with no sandbox.

**Recommendation:** this is acceptable for the first audience (the author
dogfooding a game) and unacceptable for the third (researchers who install a
wheel). If you build it, mark it clearly as an unstable, unsupported interface,
and never let it be the documented extension path.

---

## 10. Open questions for the ADR author

1. **Snapshot or live evaluation of selector parameters?** The brief does not
   say. Recommend snapshot. This must be decided before any verb takes a
   selector parameter, because it changes the scheduler.
2. **Is the canonical iteration order part of the public contract?** Recommend
   "stable within a released version, not across versions". Anything stronger
   freezes the chunk allocator.
3. **Fixed point or floating point for stats?** Recommend fixed point. This
   depends on the answer to the brief's determinism open question, and it also
   partly *answers* it.
4. **What is the capability bit budget, and who allocates the bits?** 64 is
   enough only if bit allocation is disciplined. Recommend a central registry
   with a compile-time check.
5. **May a verb issue commands?** If yes, the sealed-batch model of item 9 needs
   a defined fixed-point or a depth limit. Recommend "no" for version 1.
6. **How is per-faction visibility budgeted?** At 16.7 million tiles a bitplane
   is about 2.1 MB per faction. That is fine for eight factions and bad for two
   hundred. Decide the faction ceiling now, because it changes the visibility
   representation.
7. **Is `.explain()` output part of the tested interface?** It will be, in
   practice, because people will assert on it. Decide whether to give it a stable
   machine-readable form alongside the human-readable one.
8. **What is the summary byte budget per L1 and L2 cell?** This caps how many
   fields can prune. It should be a hard, checked number, not a guideline.

---

## Sources

- [Polars — Lazy API user guide](https://docs.pola.rs/user-guide/concepts/lazy-api/)
- [Polars — LazyFrame internals (DeepWiki)](https://deepwiki.com/pola-rs/polars/2.3-lazyframe)
- [Polars — `LazyFrame.explain`](https://docs.pola.rs/py-polars/html/reference/lazyframe/api/polars.LazyFrame.explain.html)
- [Ibis — internals and expression trees](https://ibis-project.org/concepts/internals)
- [Ibis — issue 4704, usability problems with the `_` deferred API](https://github.com/ibis-project/ibis/issues/4704)
- [Ibis — chaining expressions](https://ibis-project.org/how-to/analytics/chain_expressions)
- [Chambi, Lemire, Kaser, Godin — "Better bitmap performance with Roaring bitmaps"](https://arxiv.org/pdf/1402.6407)
- [Lemire et al. — "Consistently faster and smaller compressed bitmaps with Roaring"](https://arxiv.org/pdf/1603.06549)
- [Lemire et al. — "Roaring Bitmaps: Implementation of an Optimized Software Library"](https://arxiv.org/pdf/1709.07821)
- [Julia Evans — "Fast integer sets with Roaring Bitmaps"](https://jvns.ca/blog/2016/01/23/fast-integer-sets-with-roaring-bitmaps/)
- [Bevy — `bevy_ecs::system` documentation](https://docs.rs/bevy_ecs/latest/bevy_ecs/system/index.html)
- [Bevy Cheat Book — exclusive systems and parallelism](https://bevy-cheatbook.github.io/programming/exclusive.html)
- [Bevy — parallelism discussion 2875](https://github.com/bevyengine/bevy/discussions/2875)
- [OpenRA — order processing and networking (DeepWiki)](https://deepwiki.com/OpenRA/OpenRA/2.3-order-processing-and-networking)
- [OpenRA-RL — agent action types](https://openra-rl.dev/docs/agents/)
- [OpenRA — Lua scripting API](https://docs.openra.net/en/release/lua/)
- [Factorio — `Data` and `data.raw` prototype documentation](https://lua-api.factorio.com/latest/types/Data.html)
- [Factorio — modding tutorial, prototype stage](https://wiki.factorio.com/Tutorial:Modding_tutorial/Gangsir)
- [Factorio — `LuaEntityPrototype` runtime API](https://lua-api.factorio.com/1.1.68/LuaEntityPrototype.html)
- [RimWorld — XML Defs modding tutorial](https://rimworldwiki.com/wiki/Modding_Tutorials/XML_Defs)
- [RimWorld — abstracts and inheritance](https://spdskatr.github.io/RWModdingResources/abstracts.html)
- [Paradox forum — additive versus multiplicative modifiers](https://forum.paradoxplaza.com/forum/threads/additive-bonuses-vs-multiplicative-bonuses.1144836/)
- [Paradox forum — add multiplicative modifiers](https://forum.paradoxplaza.com/forum/threads/add-multiplicative-modifiers.1905655/)
