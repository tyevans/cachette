# Character Graph, Offices and Inheritance

Research report 14 for the foundational architecture decision record.

## 0. Context

Cachette is a world simulation engine. The core is Rust. The control plane
is Python. The engine simulates a hex world at three levels of detail.
Level 0 holds 16,777,216 tiles in a 4,096 by 4,096 grid. Level 1 holds
65,536 cells. One level 1 cell summarises 256 tiles. Level 2 holds 256
blocks. One level 2 block summarises 65,536 tiles. The target scale is
16,777,216 tiles and one million units.[^1]

The engine runs a fixed timestep at 10 Hz. The frame loop has five
barriers. Phases 1 to 4 read the world and write only events. Phases 5 to 8
write the world and read only events. Python attaches before phase 0 and
after phase 8. Python never runs while the simulation runs.[^1]

Three rules bind every recommendation in this report.

- **No floating point in simulated or aggregated state.** The fixed-point
  scale is Q16.16. `Fix32` is `i32`. `Fix64` is `i64`. `Accum` is `i64`.
- **Every random draw is a keyed hash of the tuple (system, frame, entity,
  draw).** No thread-local generator is allowed.
- **Every parallel result is ordered by a stable key.** Thread completion
  order, work-stealing order and hash iteration order are forbidden.

This report covers a character and dynasty layer over the mass simulation.
The layer holds named individuals. It holds their parents, their houses,
their offices, the assets they own, and the laws that move those assets on
death.

### 0.1 The five findings

**Finding 1. The identity and field split holds, with one exception.** A
character is a graph node. A character has an identity, a set of edges, and
a history. None of that is a quantity over space. The field and operator
framework does not apply to it. The exception is the **spatial reach of
renown**, which is a scalar field and which belongs in the existing
influence-map machinery. Section 7 gives the link, and the link is one
integer column. No part of character identity enters the field.

**Finding 2. The scale regime inverts, but the Rust side is not the cost.**
At 100,000 living characters the whole Rust character tier costs under
50 ms for each simulated year. A simulated year is 1,200 ticks, so that is
under 0.05 ms for each tick. The running per-tick budget lists trade at
1.1 ms and economy at 0.4 to 0.6 ms.[^2] The Rust character tier is free
against that budget. **The Python decision pass is the cost**, at about
1.7 ms for each tick at 100,000 characters. Section 8 derives both numbers.

**Finding 3. The lead's succession hypothesis is nearly right, and the
correction matters.** Succession is **filter, sort, then allocate**. Take
the first candidate is one allocator. Partible inheritance is a second
allocator over the same filtered and sorted list. Partition therefore does
not break the model. It shows that the model was stated one step too
narrowly. Section 6 gives both allocators.

**Finding 4. Do not accept a comparator function from a policy author.** A
user-supplied comparator can be intransitive. An intransitive comparator
makes the output of a sort depend on the sort algorithm and on the input
order. That is a determinism hole and no tie-break on entity identifier
closes it. Accept an ordered vector of integer key extractors instead. Then
totality is structural, and the sort is an integer radix sort. Section 6.3
states this rule.

**Finding 5. The no-loops rule in the API is a function of the tier, not of
the count.** Enforce the exception with a **declared tier on the entity
class**, not with a cardinality check at call time. A cardinality check
makes the same script work on a small world and fail on a large world. That
is the worst available failure mode. Keep a hard cardinality ceiling as a
load-time backstop only. Section 8 sets the ceiling at 262,144 and derives
the number from the Python budget.

**Finding 6. Promotion from the mass tier is the same algorithm as
succession, and formations need no new structure at all.** A soldier who
crosses an achievement threshold becomes a character. That is filter, then
sort, then allocate, with a different key vector and a budget instead of
`Take(1)`. Section 9 gives it. A formation is an organisational node that
**owns** its soldiers, so it reuses the ownership machinery of section 4
without change, and an order to a formation is an ordinary set-valued verb
over one extra selector leaf. Section 10 gives it. Neither addition needs a
new kernel and neither is a budget line.

**Finding 7. Opinion storage, not the character arena, decides the target
population.** A promoted soldier cannot inherit by blood, so he must rise by
appointment, and appointment needs a model of favour. Favour is a directed
relation, and a dense one is impossible at 68 billion pairs. A hard
out-degree cap makes it linear rather than quadratic, and that cap is the
whole mechanism. Even so, opinion costs 640 bytes for each living character
against 200 for the arena. **It more than quadruples the per-character
memory cost.** Section 11 gives the arithmetic and revises the recommended
target down to 20,000 to 50,000 living characters, well below the hard
ceiling of 262,144.

**Finding 8. The promotion mechanism is close to a granted patent, and the
report states that as a constraint.** Section 12 summarises the published
claim text of the Nemesis patent, which runs to 2036. Every independent
claim recites a player-operated avatar and non-player characters defined by
opposing that avatar. The design in this report has neither. **That section
is a factual summary and not legal advice.**

---

## 1. Terms

**Character.** A named individual with an identity that persists after
death. A character is not a unit. A unit is a mass-tier entity with a
position.

**Character identifier.** A `u32` index into the character arena, paired
with a `u32` generation, exactly as the unit identifier is.[^1]

**House.** A group of characters that share a patrilineal founder. The
report uses "house" for the group and "dynasty" for the same idea in prose
about other games.

**Genealogy.** The directed graph in which an edge runs from a child to
each of its two parents. The graph is acyclic. The graph is not a tree,
because a character has two parents.

**Father tree.** The subgraph of the genealogy that holds only the
child-to-father edges. This subgraph **is** a tree. The mother tree is the
same construction on the child-to-mother edges.

**Title.** A named, inheritable object. A title carries a succession
policy. A title may anchor to a place.

**Office.** An entity that exists because another entity exists. A court
exists because a castle exists. The castle is the **anchor**. The office is
the **dependant**.

**Anchor graph.** The directed graph in which an edge runs from a dependant
to its anchor.

**Kinship coefficient.** The probability that one gene drawn at random from
one character and one gene drawn at random from a second character are
identical by descent. The symbol is `f(i, j)`.

**Inbreeding coefficient.** The kinship coefficient of the two parents of a
character. The symbol is `F(i)`, and `F(i) = f(father(i), mother(i))`.

**Numerator relationship matrix.** The matrix `A` in which
`A(i, j) = 2 f(i, j)`. Animal breeding uses this matrix.

**Euler interval label.** A pair of integers `(tin, tout)` assigned to each
node of a tree by a depth-first search. Node `X` is an ancestor of node `Y`
if and only if `tin[X] <= tin[Y]` and `tout[Y] <= tout[X]`.

**Character tier.** The set of entity classes that expose a per-character
object model to Python. The mass tier is the set that does not.

---

## 2. The budget, established before any optimisation

The brief asks for the real budget first. This section gives it.

### 2.1 The character row

The row is struct-of-arrays, in the same style as the unit row. The table
gives one logical row.

| Field | Type | Bytes | Purpose |
|---|---|---|---|
| `father` | `u32` | 4 | Parent edge. `u32::MAX` means none. |
| `mother` | `u32` | 4 | Parent edge. |
| `house` | `u32` | 4 | House membership. |
| `liege` | `u32` | 4 | Chain of command parent. |
| `primary_title` | `u32` | 4 | Held title. |
| `seat` | `u32` | 4 | Tile index of the held seat. |
| `birth_tick` | `u32` | 4 | Absolute tick. |
| `death_tick` | `u32` | 4 | `u32::MAX` while alive. |
| `father_tin` | `u32` | 4 | Euler label, father tree. |
| `father_tout` | `u32` | 4 | Euler label, father tree. |
| `renown` | `i32` | 4 | `Fix32`, an owned scalar. |
| `inbreeding` | `i32` | 4 | `Fix32`, `F(i)`, written once at birth. |
| `traits` | `u32` | 4 | Interned `TraitSetId`. |
| `culture` | `u16` | 2 | |
| `faith` | `u16` | 2 | |
| `depth` | `u16` | 2 | Depth in the father tree. |
| `flags` | `u8` | 1 | Sex, alive, legitimate. |
| `fame_plane` | `u8` | 1 | Index of the renown field plane. 255 means none. |
| **Total** | | **64** | Section 9.6 adds `unit: u32`, taking it to 72. |

The row is 64 bytes. Every field is a fixed-width integer, so the row is
`bytemuck::Pod` with declared padding and no `bool`.

Section 9.6 adds one column, `unit: u32`, which links a character back to
the soldier that carries him. That takes the row to 68 bytes and padding
takes it to 72. The storage table of section 2.4 rounds a living character
to 200 bytes and a dead character to 80, and both figures still hold. The
arena is struct-of-arrays, so the row size is an accounting figure and not a
claim about locality.

### 2.2 Side structures

| Structure | Size rule | Note |
|---|---|---|
| Child list, compressed sparse row | 8 bytes for each row, living and dead | One offset and one entry for each child. |
| Relation edges | 8 bytes for each edge, stored at both endpoints | Edge is `(other: u32, kind: u8, strength: i8, since: u16)`. |
| Office rows | 16 bytes for each office | `(anchor, holder, kind, policy, flags)`. |
| Ownership reverse index | 8 bytes for each owned asset | Not for each character. |

Assume a mean of 8 non-kin relation edges for each living character. A
symmetric edge is stored at both endpoints, so the cost is 128 bytes for
each living character and no reverse index is needed. Assume one office for
each four characters, which is 4 bytes for each character.

A living character therefore costs 64 + 128 + 4 = 196 bytes. Round to 200.

A dead character keeps its row, its parent edges and its child list. It
drops its relation edges, because a dead character has no living social
graph. A dead character therefore costs 64 + 8 = 72 bytes. Round to 80.

### 2.3 The dead-to-living ratio

Assume a stationary population and a mean lifespan of 60 years. Then the
number of deaths for each year is the living count divided by 60. Over 500
simulated years the dead count is therefore about 8.3 times the living
count.

### 2.4 The storage table

| Living | Dead at 500 years | Living bytes | Dead bytes | Arena total |
|---|---|---|---|---|
| 10,000 | 83,000 | 2.0 MB | 6.6 MB | **8.6 MB** |
| 100,000 | 833,000 | 20 MB | 67 MB | **87 MB** |
| 1,000,000 | 8,300,000 | 200 MB | 666 MB | **866 MB** |

Compare these figures with the mass simulation. The tile side alone is
about 268 MB at 16 bytes for each tile. At 100,000 living characters the
whole character layer costs less than one third of the tile side. **The
character arena is not a storage problem at 10,000 or at 100,000.**

### 2.5 The one structure that grows without bound

The biography log is the only structure with unbounded growth. Assume 20
recorded life events for each character over a whole life. An event row is
24 bytes: `(tick: u32, subject: u32, object: u32, place: u32, kind: u16,
pad: [u8; 6])`.

| Living | Total characters at 500 years | Events | Log bytes |
|---|---|---|---|
| 10,000 | 93,000 | 1.86 M | 45 MB |
| 100,000 | 933,000 | 18.7 M | 448 MB |
| 1,000,000 | 9,300,000 | 187 M | **4.5 GB** |

**Prune at the 1,000,000 case, not before.** Section 16.2 gives the rule.

### 2.6 The conclusion of the budget section

At 10,000 and at 100,000 living characters the binding constraint is not
the character arena and it is not Rust compute. Two other things bind, and
both are established later in this report. The wall-time cost of the Python
decision pass binds first. Opinion storage binds second, and section 11.10
shows that it costs 640 bytes for each living character against the 200
bytes that this section accounts for. Section 8 derives that number and sets
the tier ceiling from it. Every structural choice below is therefore
optimised for **expressiveness and determinism**, and only checked against
throughput.

---

## 3. Data structures for hierarchy and genealogy

### 3.1 The survey

The table compares the six standard representations. `N` is the node count.
`d` is the tree depth. The query column names the cost of "is X an ancestor
of Y". The update column names the cost of adding one leaf.

| Representation | Ancestor query | Leaf insert | Subtree range | Verdict |
|---|---|---|---|---|
| Parent pointer | O(d) | O(1) | Not available | **Accept** for mutable trees |
| Adjacency, compressed sparse row | O(d) upward, O(1) child list | Rebuild, O(N) | Not available | **Accept** for child lists |
| Euler interval label | O(1), two comparisons | O(1) into a gap, else O(N) rebuild | O(1), contiguous | **Accept** for stable trees |
| Nested set | O(1) | O(N) relabel | O(1) | Same as the interval label. The interval label is the general form. |
| Materialised path | O(1) prefix test | O(1) | O(1) prefix scan | **Reject.** Variable length breaks the `Pod` row. |
| Closure table | O(1) | O(d) rows inserted | O(1) | **Reject.** See the arithmetic below. |

**Reject the closure table on arithmetic.** A closure table stores one row
for each ancestor-descendant pair. On a genealogy DAG each character has up
to `2^k` distinct ancestors at generation `k`. At depth 20 that is over one
million ancestor rows for a single character. At 100,000 living characters
the table does not fit in memory and it cannot be maintained.

**Reject bitset ancestry.** A 64-bit founder mask answers "does X descend
from founder F" in one instruction. It requires 64 or fewer founders. A
world with 100,000 living characters has thousands of founders. The
technique is correct and it does not apply here.

**Reject Euler tour plus range-minimum-query for the lowest common
ancestor.** The technique gives an O(1) query after O(N) preprocessing, and
it is implementable.[^3] It does not fit the update pattern. The command
hierarchy and the vassalage hierarchy change on every death and on every
grant. A full O(N) preprocess for each change is worse than the naive
query, because the trees are shallow. Section 3.3 gives the number.

### 3.2 The recommended split

The domain holds three different graphs. They have different shapes and
they need different structures. Do not use one structure for all three.

**Graph 1: genealogy.** A DAG with in-degree 2. It is append-only. A
character gains parents at birth and never changes them. It is large: it
holds every dead character.

**Graph 2: the father tree and the mother tree.** Each is a tree, because
each holds only one of the two parent edges. Each covers the same node set
as the genealogy.

**Graph 3: the mutable hierarchies.** Chain of command, vassalage and the
office hierarchy. These are trees. They are shallow. They change often, and
they hold only living characters.

Index them as follows.

- Store `father` and `mother` as two `u32` columns on the character row.
  This is the authoritative genealogy. It costs 8 bytes and it is exact.
- Build a **compressed sparse row child list** over all characters, living
  and dead, at the character-tier barrier. This answers "who are the
  children of X" as one contiguous range. Rebuild is one counting sort by
  `father`, which is `O(N)`.
- Build a **gapped Euler interval label on the father tree**, stored as the
  `father_tin` and `father_tout` columns. This answers "is X a patrilineal
  ancestor of Y" in two integer comparisons, and it answers "give me every
  patrilineal descendant of X" as one contiguous range.
- Store the **mutable hierarchies as parent pointers with a depth column**.
  Do not label them. They change too often.

### 3.3 The two results that justify this split

**Result 1. Patrilineal descent becomes a range operation.** With the
interval label, the set of all patrilineal descendants of a character is a
contiguous span of the Euler order. A dynasty query is therefore a range
scan and not a graph walk. A cadet-branch split, which rewrites the `house`
column of every descendant of one character, is therefore a **contiguous
range write**. It is a `map` kernel over a span, not a traversal. At
100,000 living characters the largest plausible descendant span is a few
thousand rows, which is one or two microseconds.

**Result 2. The naive lowest common ancestor is correct here.** Cap the
depth of the mutable hierarchies at 12 and check the cap when an edge is
added. Then the lowest common ancestor of two nodes is: lift the deeper
node to the shallower depth, then walk both upward in lockstep. The cost is
at most 24 pointer reads.

Quantify it. The character arena at 100,000 living characters is 6.4 MB, so
a random row read is an L2 or L3 hit at about 15 ns. A lowest-common-
ancestor query therefore costs about 360 ns. One query for every living
character costs 36 ms. That is 36 ms for each simulated year, which is
0.03 ms for each tick. Range-minimum-query preprocessing would save that
0.03 ms and would cost an O(N) rebuild on every grant. **Do not build it.**

### 3.4 Rebuild cost of the Euler labels

The label rebuild is a depth-first search over every character, living and
dead. At 100,000 living characters that is 933,000 nodes. A pointer-chasing
depth-first search runs at about 20 ns for each node, so the rebuild costs
about 19 ms.

Run the rebuild once for each simulated year. Nineteen milliseconds
amortises to 0.016 ms for each tick. That is affordable and it needs no
further work.

A gapped label is the optimisation if measurement ever demands it. Assign
each node an interval that is twice as wide as its subtree needs. Then a
birth writes into a gap in O(1). Rebuild only when the gap pressure of some
subtree reaches zero. Record this option and do not build it in version 1.

### 3.5 Union-find for lineage grouping

Union-find with path compression and union by size is the standard
structure for grouping.[^4] It has one property that disqualifies it as the
authoritative house index: **it cannot split**. A cadet branch that leaves
its parent house is a split, and union-find has no split operation.

Recommend instead: store `house` as an explicit `u32` column on the
character row. A birth copies the father's house. A cadet split is the
range write of section 3.3.

Use union-find for one purpose only: the one-off grouping pass during world
generation, which assigns initial houses from a set of founder marks. That
pass never splits.

**Determinism rule for union-find.** Union by size. Break a size tie by
choosing the smaller root identifier as the new root. Without that rule the
forest shape depends on insertion order, and the forest shape is observable
through any iteration of the structure.

### 3.6 Kinship and relatedness

This subsection covers the material that the brief marks as probably
unknown to game developers. It is directly applicable, and one property of
it is unusually well suited to this project.

**The source domain.** Animal breeding needs the numerator relationship
matrix `A`, where `A(i, j) = 2 f(i, j)`, to predict breeding values.
Henderson published a method that builds the **inverse** of `A` directly
from the pedigree, without ever forming `A`.[^5] The method works because
`A^{-1}` is sparse: each animal contributes only to the three-by-three
block that covers itself and its two parents. Building `A^{-1}` is
therefore linear in the pedigree size.

**Why the project does not need Henderson's method.** `A^{-1}` exists to
solve the mixed-model equations of best linear unbiased prediction. This
project does not solve those equations. It needs individual entries of `A`,
on demand, for a few thousand pairs each year. Record Henderson's method as
the reason the animal-breeding literature is worth reading, and do not
implement it.

**What the project does need.** Two algorithms.

**Algorithm 1: pairwise kinship, on demand.** Karigl gives the recursion
for kinship and for the generalised kinship coefficients.[^6] The two-gene
case is:

```
f(i, i) = (1 + F(i)) / 2
f(i, j) = (f(father(i), j) + f(mother(i), j)) / 2
          when i is not an ancestor of j
f(i, j) = 0   when i or j has no recorded ancestor in common with the other
```

Order the two arguments so that the recursion always expands the younger
character. Memoise on the ordered pair `(min, max)`.

**The fixed-point result.** Every step of this recursion halves a value.
Therefore every kinship coefficient is a dyadic rational: an integer over a
power of two. `Fix32` in Q16.16 represents a dyadic rational with an
exponent down to `2^-16` **exactly**. If the recursion is truncated at 12
generations, the smallest term is `2^-12`, and every intermediate value is
exactly representable. **The kinship computation is therefore exact in
fixed point, with no rounding at any step.** This is not an approximation
that the no-float rule tolerates. It is a case where the integer form is
the correct form and the float form is the lossy one.

**Cost.** Truncate the ancestor search at generation `d`. Then each
argument has at most `2^d - 1` ancestors, and the memoised recursion visits
at most `(2^d)^2` pairs. At `d = 6` that is at most 3,969 pairs, and real
pedigrees reach a small fraction of that bound because lines converge.
Recommend `d = 6` for gameplay tests such as "may these two marry" and
`d = 12` as the ceiling that keeps the arithmetic exact.

**Algorithm 2: the inbreeding coefficient, once for each character.**
Meuwissen and Luo give an algorithm that computes inbreeding coefficients
for large populations with memory linear in the population size. It is
designed for the case where a few new individuals need a coefficient and
the ancestors' coefficients are already known.[^7] That case is exactly a
birth.

Compute `F(i) = f(father(i), mother(i))` once, at the birth event. Write it
into the `inbreeding` column. Never recompute it. A character's parents
never change, so the value is immutable. At 100,000 living characters there
are about 1,667 births for each year, and each costs about 1,000 integer
operations. The pass costs about 5 ms for each simulated year.

**The coefficient of relationship.** Wright's `r` is `2 f(i, j)` for
non-inbred individuals.[^8] Expose `r`, not `f`, in the Python API, because
`r` is the number that reads as "one half for a full sibling".

---

## 4. Ownership

### 4.1 The forward direction

One owner for each asset is one `u32` column on the asset. `u32::MAX` means
unowned. This is correct, it is 4 bytes, and it is total.

Do not make the column a union of a faction identifier and a character
identifier. Keep two columns, `owner_faction` and `owner_character`, and
make the invariant "at most one is set". A tagged union in a hot column
costs a branch in every read and it cannot be compared with a single
vector instruction.

### 4.2 The reverse direction

"Give me every asset of this owner" needs an index. Build it as a
**compressed sparse row structure keyed on the owner**, exactly as the
child list is built.

- Sort the asset identifiers by `owner_character`. This is a counting sort
  over a `u32` key, so it is `O(assets)`.
- Store an offset array over characters and an entry array over assets.
- Cost is 4 bytes for each character plus 4 bytes for each owned asset.

**Rebuild policy.** Rebuild at the character-tier barrier, not each tick.
Between rebuilds, hold changes in a small patch list and consult it after
the index. Rebuild early when the patch list exceeds one percent of the
asset count.

At one million assets a counting sort by a `u32` key costs about 5 ms on
one core. Once for each simulated year that is 0.004 ms for each tick.

### 4.3 The cost of transfer on death

Transfer on death is a bulk reassignment. The reverse index makes the
assets of one decedent a contiguous run, so the cost of one transfer is
`O(assets owned by the decedent)`.

Compute the yearly total. With a 60-year mean lifespan, one sixtieth of all
characters die each year. If ownership is spread evenly, one sixtieth of
all assets change hands each year. At one million owned assets that is
about 16,700 writes for each year. Each write is a scatter into the asset
owner column, at about 15 ns. The pass costs about 0.25 ms for each
simulated year.

**Transfer on death is not a performance concern at any scale in this
report.** Its cost is dominated by the succession algorithm that decides
the new owner, and section 6 shows that this is also cheap.

### 4.4 Shared, disputed and conditional ownership

All three are cheap, and each is cheap for a different reason. State the
reasons, because a naive implementation of any of them is not cheap.

**Shared ownership is cheap by interning.** Add an optional
`share_set: u32` column that interns into a side table of owner sets,
exactly as the upgrade set and the recipe interning already work.[^9] The
`owner_character` column stays authoritative and single-valued: it names
the managing owner. The share set carries the split. Expect under one
percent of assets to carry a share set, so the side table is small.

**Disputed ownership is cheap because it is not ownership.** Do not add a
second owner column. Model a dispute as a **claim edge**:
`(claimant: u32, asset: u32, strength: i16, since: u16)`, which is 12
bytes. The owner column stays total and single-valued, so every read stays
one load. A claim is a gameplay object that a war can resolve.

**Conditional ownership is cheap because it is a flag.** A fief held on the
condition of service is the same ownership plus a condition identifier and
a review period. Add a `condition: u16` column on the ownership record and
evaluate conditions in one pass at the character-tier barrier.

**The rule that makes all three cheap: the owner column must stay
single-valued and total.** Every extension hangs off the side. If the owner
column ever becomes multi-valued, every hot-path read of ownership becomes
an indirection, and there are many such reads.

---

## 5. Offices and lifecycle coupling

### 5.1 The model

An office is an entity with three columns:

```rust
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct OfficeRow {
    anchor: u32,   // the entity that causes this office to exist
    holder: u32,   // CharacterId, or u32::MAX when vacant
    kind:   u16,   // OfficeKindId, an index into the immutable type table
    policy: u16,   // SuccessionPolicyId
    flags:  u32,   // depth in the anchor graph, and status bits
}
```

The dependency is the `anchor` column. A court exists because a castle
exists, so the court's `anchor` is the castle.

### 5.2 The shape rule

**Constrain the anchor graph to a forest of depth at most 8.** Check the
depth when an office is created, and reject a creation that exceeds it.
Reject a creation that would form a cycle.

The bound matters for three reasons. It makes the cascade terminate in at
most 8 waves. It lets `flags` carry the depth in 4 bits. It makes the whole
cascade cost analysable in advance.

A forest, not a general DAG. A general DAG would allow one office to have
two anchors, and then the destruction rule needs a policy for partial
destruction. That policy is a design question with no obvious answer, so
remove it by construction.

### 5.3 The cascade

When an anchor is destroyed, every office below it must be vacated and
removed. This runs in phase 6, the structural phase, because it despawns
entities and invalidates indices.[^1]

**The ordering rule: run a level-synchronous wavefront, and sort each wave
by entity identifier before it expands.**

```
wave_0   = the set of destroyed anchors, sorted ascending by entity id
for k in 0 .. 8:
    wave_(k+1) = gather children of wave_k through the anchor CSR index
    sort wave_(k+1) ascending by entity id
    deduplicate wave_(k+1)
    if wave_(k+1) is empty: stop
emit vacate and despawn events in wave order, deepest wave first
```

This rule gives determinism for three reasons. The initial set is sorted.
Each wave is sorted and deduplicated, so the frontier does not depend on
the expansion order. The waves apply deepest first, so a parent never
despawns before its child, and no event refers to an entity that is already
gone.

**Kernel vocabulary.** Each wave is `gather`, then `sort`, then a unique
pass over the sorted array. Those are all in the project's kernel
vocabulary. A level-synchronous breadth-first search is therefore
expressible in the vocabulary, at the cost of one sort for each wave. With
at most 8 waves and a small frontier, the sorts are free.

**Cost.** The cascade is bounded by the number of offices below the
destroyed anchors. Even a large siege destroys a few hundred anchors and a
few thousand offices. The cost is microseconds. This is not a budget line.

### 5.4 What the cascade emits

The cascade emits two event types, not one.

- `OfficeVacated { office, former_holder, reason }`
- `OfficeDestroyed { office, anchor }`

Keep them separate. A vacancy is a fact that the succession pass consumes.
A destruction is a fact that the structural pass consumes. A single merged
event forces both consumers to filter, and it hides the case where an
office is vacated but survives.

---

## 6. Succession and inheritance as an algorithm

### 6.1 Verdict on the hypothesis

The lead's hypothesis is: succession is filter, then sort, then take the
first. **The hypothesis holds for selective succession and it is one step
too narrow.** The correct statement is:

> **Succession is filter, then sort, then allocate.**
>
> A policy is a triple: an eligibility predicate, an ordered vector of
> integer key extractors, and an allocator.

Take the first candidate is the allocator `Take(1)`. Partible inheritance
is the allocator `Partition`. Both consume the same filtered and sorted
candidate list. Partition therefore does not break the model.

### 6.2 The three parts, as data

**Part 1: the eligibility predicate.** This is the same predicate machinery
that the selector engine already has.[^9] A predicate is a tree of
comparisons over character columns and over derived quantities. Examples:
`sex == male` gives agnatic succession; `sex == female` gives enatic;
`house == title.house` restricts to the house; `faith == title.faith`
restricts by religion; `is_legitimate` excludes bastards.

**Part 2: the key extractor vector.** This is the important part and
section 6.3 covers it.

**Part 3: the allocator.** Two allocators cover the whole survey.

| Allocator | Definition |
|---|---|
| `Take(1)` | The first candidate takes every asset. |
| `Partition { primary_share }` | The first candidate takes `primary_share` of the ranked asset list. The remainder deals round-robin to the next candidates in rank order. |

The asset list must itself be sorted before the deal, or the partition is
not deterministic. Sort the assets by `(tier descending, asset id
ascending)`. Then the primary heir receives the highest-tier assets, which
matches the documented behaviour of high partition in the reference
games.[^10]

Use the largest-remainder method for the integer split when a share is
expressed as a fraction. The project's `transfer` verb already specifies
this method for the same reason: it conserves exactly and it does not
depend on iteration order.[^9]

### 6.3 The key extractor vector, and why a comparator is forbidden

**Do not accept a comparator function from a policy author.** This is the
sharpest determinism finding in this report.

A comparator is a function from a pair of candidates to an ordering. A
comparator written by a policy author can be **intransitive**. It can say
that A beats B, that B beats C, and that C beats A. When the comparator is
intransitive, the result of a sort depends on the sort algorithm, on the
pivot choice and on the input order. A tie-break on the entity identifier
does not repair this, because the failure is not a tie. It is a cycle.

The engine cannot detect an intransitive comparator cheaply. A full check
costs `O(n^3)` comparisons.

**Accept an ordered vector of integer key extractors instead.**

```rust
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct SortKeySlot {
    extractor: u16,   // an index into the immutable extractor table
    ascending: u8,    // 1 means ascending
    _pad:      u8,
}
```

A policy holds up to 8 slots. **The engine appends a final, implicit slot
that extracts the character identifier, ascending.** That slot is not
authorable and it cannot be removed.

Each extractor is a function from a candidate to an `i64`. The engine holds
the extractor table. Examples: `birth_tick`, `generation_distance_to_
deceased`, `kinship_to_deceased` in `Fix32`, `renown`, `house_membership`,
`elector_vote_total`.

Three properties follow, and all three are structural.

1. **The order is total.** Two distinct candidates differ in the final key,
   because identifiers are unique.
2. **The order is transitive.** Lexicographic order over a tuple of `i64`
   is transitive by construction. An author cannot break it.
3. **The sort is an integer sort.** The key is a fixed-width tuple of
   `i64`, so a least-significant-digit radix sort applies. That is the
   `sort` kernel, with no comparison callback and no branch.

### 6.4 The five named policies, expressed as data

| Policy | Eligibility predicate | Key vector | Allocator |
|---|---|---|---|
| Primogeniture | `is_child(deceased) and is_legitimate` | `generation_distance` ascending, `birth_tick` ascending | `Take(1)` |
| Ultimogeniture | `is_child(deceased) and is_legitimate` | `generation_distance` ascending, `birth_tick` descending | `Take(1)` |
| Seniority | `house == deceased.house` | `birth_tick` ascending | `Take(1)` |
| Elective | `is_in_candidate_pool` | `elector_vote_total` descending | `Take(1)` |
| Partition | `is_child(deceased) and is_legitimate` | `birth_tick` ascending | `Partition { primary_share: 1/2 }` |

Add `sex == male` to the predicate for the agnatic form of any row. Add
`kinship_to_deceased` as a leading descending key for the proximity-of-
blood form. **Every named succession law in the survey is a row in this
table. None of them is code.** That confirms the lead's framing.

### 6.5 How a region carries its own policy

**The policy is a column on the title, not on the character and not on the
tile.**

The reason is that the title is the thing that is inherited. The law
travels with the object that the law governs. A character who holds three
titles under three laws is a normal case, not a special case: each title
resolves independently under its own policy.

Regional variation follows for free, because a title anchors to a place. A
region with its own laws is a set of titles that carry a different policy
identifier. Nothing about the tile grid needs to know that succession
exists.

Cost: 2 bytes for each title. A world with 100,000 living characters holds
perhaps 20,000 titles, so the whole policy column is 40 KB.

### 6.6 What happens when policies conflict across a border

**Two policies cannot conflict, because each title resolves under exactly
one policy.** What appears to be a conflict is one of two things, and both
have a defined answer.

**Case 1: one heir inherits under two laws.** A character resolves as the
heir of a title in region A and of a different title in region B. This is a
union, not a conflict. The character holds both titles. No rule is needed.

**Case 2: two characters each believe they are the legal heir.** This
happens when the title's own policy names heir X, and a policy that some
other authority recognises would have named heir Y. Examples of another
authority: the liege of the title holder, or a neighbouring realm that
recognises a different law.

**The resolution rule.**

> The title's own policy is authoritative for the state transition. The
> engine transfers the title to the heir that the title's policy names.
>
> For each recognised alternative policy that names a different heir, the
> engine emits one claim edge from that candidate to that title, with a
> strength derived from the recognising authority.
>
> The set of recognised alternative policies is a column on the title. It
> is a bitmask over the policy table, capped at 16 entries.

This rule has three good properties. The state machine stays total, so
there is never an unowned title. The legal ambiguity becomes a gameplay
object with a strength, instead of an engine special case. And the cost is
bounded: at most 16 extra evaluations of the succession algorithm for each
death, on a candidate list that is already gathered.

### 6.7 Prior art, and what is verifiable

**Crusader Kings 3.** The community wiki documents the behaviour of the
succession laws in detail: primogeniture gives every title to the firstborn
of the eligible sex; ultimogeniture gives them to the lastborn; house
seniority gives them to the oldest member of the house regardless of
distance; partition splits the titles and high partition guarantees the
primary heir at least half; elective forms such as tanistry select by
vote.[^10] **This is documented game behaviour. It is not documented
implementation.** No published source describes how the game stores or
evaluates these laws. Mark any implementation claim as unverified.

**Victoria 3.** The community wiki documents interest groups as collections
of pops with political positions and with a leader whose personal views
influence the group.[^11] The published material describes design, not
implementation. No figure for the number of simulated characters is
published. Do not cite a character count.

**Dwarf Fortress.** The community wiki documents that world generation
creates historical figures, that legends mode exposes their histories, and
that an option exists to remove unimportant dead historical figures after
generation in order to save space and to speed up loading and saving. The
same source records that a large world with a long history can produce a
history dump of up to one gigabyte.[^12] **That is a documented
confirmation of the finding in section 2.5: the biography log, not the
character arena, is the structure that must be pruned.** It is the only
external figure in this report that bears directly on the storage
recommendation.

**Shadow of Mordor.** The Nemesis system is described in a granted United
States patent assigned to Warner Bros. Entertainment. The patent describes
uniquely generated non-player enemies, each with an assigned personality,
that rise and fall within a social structure over the course of a
playthrough.[^13] A patent is a verifiable public document, so this is the
one game in the survey with a citable implementation description. Note the
legal position: the patent runs to 2036. Read it as prior art and do not
copy its claimed method.

**RimWorld.** The relationship system is documented in the modding
material, in the same XML definition form as the rest of the game's
content.[^14] The lesson to take is the one the project has already taken
elsewhere: relationships are data with an inheritance mechanism in the
authoring format, not code.

**The pattern across this whole survey.** The project's citation checks have
repeatedly found that game-implementation claims are documented on community
wikis only, with no developer documentation behind them. That finding
already covers Victoria 2 formulas, Dwarf Fortress needs, The Sims decay
rates, Anno tiers and Planetary Annihilation pathfinding.[^2] **Expect the
same for Crusader Kings succession internals and for the Nemesis system.**
Every game citation in this report is marked as documented behaviour or as a
patent claim. None of them is marked as a description of shipped code,
because no such description is published. Do not let any of them become an
assertion when this report is merged.

**Historical law.** The five named policies of section 6.4 map onto real
legal systems: male-preference primogeniture in medieval western Europe,
gavelkind partition in Kent and in Irish Brehon law, tanistry election in
Gaelic Ireland, and agnatic seniority in Kievan Rus. The mapping is a
statement about vocabulary and not about engine behaviour. This report does
not cite a legal-history source, and the ADR should not claim historical
accuracy for the model.

---

## 7. Renown as a field, and identity as a graph

### 7.1 The evaluation

The lead suspects that reputation is the one part of this domain that is a
field. **The suspicion is correct for one of three quantities and wrong for
the other two.** Separate them, because merging them is what would drag
identity into the field framework.

**Quantity 1: renown.** How much this character has achieved. This is a
scalar attached to a character. It is not a field. It has no spatial
extent. Store it as one `Fix32` column on the character row. It is a stat,
and the existing modifier pipeline applies to it unchanged.

**Quantity 2: recognition.** Does character A know of character B. This is
a sparse bipartite relation over pairs of characters. It is not a field.
And it must not be stored: at 100,000 characters the full relation has
10^10 entries.

**Quantity 3: the spatial reach of renown.** How well known is this
character at this place. **This is a field.** It decays with distance. It
spreads along roads and trade routes rather than in straight lines. It has
exactly the shape of an influence map.

The link between them is: recognition is **derived**, not stored.

```
recognises(A, B) = reach_field_of_B[ level_1_cell_of(A) ] >= threshold
```

That is one field read and one integer comparison. No pair is ever stored.

### 7.2 Why one plane for each character is impossible

One `u8` plane over 65,536 level 1 cells is 64 KiB. At 100,000 characters
that is 6.4 GB. Reject it on arithmetic, in the same way that the influence
report rejected eight planes for each faction.[^15]

### 7.3 The recommendation

Apply the same correction that the influence-map report applied to threat.

1. **Materialise a reach plane for the house, not for the character.** A
   house is the unit that a distant population recognises. At 64 KiB for
   each plane and 64 materialised planes, the whole structure is 4 MiB.
2. **Cap the materialised planes at 64.** Store the index in the
   `fame_plane` column on the character row, with 255 meaning "not
   materialised". Rank houses by total renown at the character-tier
   barrier, and materialise the top 64. This reuses the `u64` mask idiom
   that the fog and influence reports already established.
3. **Derive an individual's reach from the house plane.** Scale the plane
   value by the character's share of the house renown, in fixed point. This
   is one multiply and one shift at the point of the query.
4. **A character with no materialised plane is known locally only.**
   Recognition for such a character is a radius test around the `seat`
   column. It needs no plane and no storage.
5. **Propagate on the existing conductance plane.** The cell is a `u8`. The
   combine is saturating unsigned addition, which is exactly associative
   and exactly commutative, so it meets the aggregation invariant with no
   special case.[^15] The propagation is the same diffusion kernel that the
   influence maps already run, on the same road and trade conductance that
   the resource report already builds.

### 7.4 What this buys, and what it does not touch

The field framework gains one input column, `renown`, and one output index,
`fame_plane`. Neither carries identity. No graph edge, no parent pointer,
no title and no office ever enters a field.

**The split therefore holds in both directions.** Identity is a graph and
stays a graph. Renown's spatial reach is a field and joins the existing
field machinery with **no new kernel**. The cost is one `u8` plane for each
of at most 64 houses, which is 4 MiB, and one diffusion pass for each
simulated year.

---

## 8. The Python API, and the exception to the no-loops rule

### 8.1 The rule that this section modifies

The selector API forbids `__bool__`, `__len__`, `__iter__` and
`__getitem__` on a selector. Each raises a `TypeError` whose message names
the correct method. The rule exists because a loop over one million
entities is catastrophic, and because an API that pretends to be eager will
be used eagerly.[^9]

The rule is right for the mass tier. This section proposes an exception for
the character tier and derives the limit of that exception from a budget.

### 8.2 The real cost, which is not the boundary

The lead's framing is that 10,000 PyO3 calls cost about 1 ms, so the rule
is a function of N. **The framing reaches the right conclusion through the
wrong cost.**

Measure the parts of one character AI decision.

| Part | Estimate | Note |
|---|---|---|
| One PyO3 attribute read | about 150 ns | Argument parsing and conversion dominate. Measure this on the target. |
| Reads for each decision | about 40 | A decision inspects the character, the liege, a few relations and a title. |
| Boundary cost for each decision | about 6 microseconds | 40 multiplied by 150 ns. |
| Pure Python logic for each decision | about 14 microseconds | Branches, comparisons and a small dictionary. |
| **Total for each decision** | **about 20 microseconds** | |

**The boundary is about 30 percent of the cost. The interpreter is about
70 percent.** Removing the boundary entirely would not change the order of
magnitude. The number that matters is therefore the total decision cost,
not the call count.

### 8.3 The budget, expressed per simulated year

A simulated year is 1,200 ticks at 10 Hz, which is 120 seconds of wall
time. That framing is what makes the character tier affordable, and the ADR
should state the budget in these terms.

| Living characters | Python cost for each year | Amortised over 1,200 ticks | Verdict |
|---|---|---|---|
| 10,000 | 0.2 s | **0.17 ms for each tick** | Free. |
| 100,000 | 2.0 s | **1.7 ms for each tick** | Affordable. Comparable to trade at 1.1 ms.[^2] |
| 262,144 | 5.2 s | **4.4 ms for each tick** | The ceiling. The largest single line in the budget. |
| 1,000,000 | 20 s | **17 ms for each tick** | Rejected. Exceeds the whole remaining budget. |

**Set the character tier ceiling at 262,144, which is `2^18`.** The number
is derived, not chosen. Above it the Python pass becomes the dominant cost
of the whole simulation, and 17 percent of a simulated year is spent
running character AI.

Two secondary checks agree with the figure. At `2^18` the character arena
is 16 MiB, which fits comfortably. And `2^18` leaves 14 bits of a `u32`
identifier for the generation counter if the two are ever packed.

### 8.4 How the exception is enforced

Three mechanisms were considered. Recommend the second, with the third as a
backstop.

**Rejected: a hard cardinality check at call time.** The check makes the
same script work on a small world and fail on a large world. A user
develops against 500 characters and ships against 50,000. This is the worst
available failure mode, because the failure appears far from its cause and
it appears only at scale.

**Recommended: a declared tier on the entity class.** Each entity class
declares `Mass`, `Character` or `Singleton` at registration. The tier is a
static property of the class and not of the current count.

| Tier | Classes | `__iter__`, `__getitem__`, `to_list` | Selector API |
|---|---|---|---|
| `Mass` | tiles, units | Raise `TypeError` | Yes |
| `Character` | characters, offices, titles | Available | Yes |
| `Singleton` | world, faction | Not applicable | No |

A user who writes `world.units.to_list()` receives an `AttributeError` on
the first call, in development, with a message that names the tier and the
correct method. There is no argument, no world size and no runtime state
that makes the call succeed. The mistake is unreachable rather than
discouraged.

**Recommended as a backstop: a load-time cardinality guard.** When the
world loads, check the declared character capacity against the 262,144
ceiling. Refuse to construct the tier above it, and name the limit in the
error. This catches the case where content grows past the design point. It
runs once, at load, so it never surprises a running script.

### 8.5 The proxy object

A per-character object must not be a copy of the row, and it must not be a
mutable view.

```python
class Character:
    """A generational handle into the character arena."""
    __slots__ = ("_id", "_gen", "_world")
```

Four rules govern it.

1. **A proxy holds an identifier and a generation.** It holds no data.
   Every attribute read goes to the Rust arena through one PyO3 call.
2. **Every read checks the generation.** A read through a proxy to a
   character that has since died raises `StaleHandleError`. A use after
   death is an exception and never a silently wrong value.
3. **A proxy write queues a command.** It does not mutate. This preserves
   the read and write phase split, and it means the character API needs no
   new determinism machinery: character commands enter the same sealed
   queue that selector verbs enter, and they receive sequence numbers at
   the same barrier.[^1]
4. **A proxy expires at the frame barrier.** Holding a proxy across a
   `step()` call raises on the next read. State this in the documentation,
   because users will try it.

The selector API remains available on the character tier and stays the
preferred form for anything set-shaped. `world.characters.filter(f.renown >
1000).count()` must remain the way a user counts. The proxy exists for
per-character branching logic, which is the case that a set-valued API
genuinely cannot express.

### 8.6 What character AI and court intrigue cost

The brief asks what this legitimate use costs. State it plainly.

**At 10,000 living characters it costs 0.17 ms for each tick, which is
under 0.2 percent of the frame. At 100,000 it costs 1.7 ms for each tick,
which makes it the second-largest line in the per-tick budget after
trade.[^2] At 262,144 it costs 4.4 ms for each tick and becomes the largest
line.**

Three further costs are real and are not in the table.

- **The global interpreter lock.** The character pass runs while Python is
  attached, so no simulation work overlaps it. The pass is serial with the
  rest of the frame by construction.
- **Sharding is mandatory above 10,000.** The pass must not run as one
  spike each year. Section 13.2 gives the sharding rule.
- **Determinism becomes the user's responsibility inside the pass.** The
  engine can guarantee that it delivers characters in a fixed order and
  that the draws are keyed. It cannot stop a user from iterating a Python
  dictionary or from calling `random`. Give the user a keyed generator that
  is bound to the character, and document that the standard library
  generator breaks replay.

---

## 9. Promotion from the mass tier to the character tier

### 9.1 Why this section exists

The project owner has decided that a unit is an individual soldier and not a
formation. The tile capacity is 8, which the owner describes as a skirmish
line. The owner wants individual units with individual experiences where
possible.

One million soldiers who each carry individual experience are the pool that
named characters come from. A soldier who survives enough battles, or who
crosses an achievement threshold, becomes a character. He gains a name, a
house, relationships, and per-character access from Python.

This section specifies that tier boundary crossing.

### 9.2 The headline result

**Promotion is filter, then sort, then allocate. It is the same three
kernels as succession, with a different key vector and a different
allocator.**

Succession filters the candidates of one title, ranks them, and takes the
first. Promotion filters the eligible units of the world, ranks them, and
takes as many as the character budget allows. The engine therefore needs no
new machinery for promotion. It needs one predicate, one key vector and one
budget.

### 9.3 The trigger, and how it costs 2 microseconds at one million units

Add one column to the unit row.

```rust
deeds: u16,   // an integer achievement accumulator, monotone upward
```

`deeds` increases when a soldier survives a battle, wins a duel, or takes a
position. It never decreases. That single property is what makes the cheap
design correct, and section 9.4 explains why.

Maintain a dense eligibility bitset over units, one bit for each unit. At one
million units that is 125 KB, which is 15,625 words of 64 bits.

The pass that updates `deeds` also writes the bit, in the same vectorised
loop, with one comparison and no branch:

```
eligible_bit = (deeds >= threshold[unit_type])
```

This is the project's dense-bitset-plus-sparse-ascending-scan pattern, which
the entity economy work already uses for threshold crossings.[^16] It costs
one compare and one bit write for each unit that the deeds pass already
touches.

**Cost of the scan.** The bitset is 125 KB, so it is level 2 resident. One
full scan at a memory bandwidth of about 64 GB for each second costs about
2 microseconds. Extracting the set bits costs one count-trailing-zeros
instruction for each set bit.

### 9.4 The scan is lazy, and monotonicity is the reason

Scan the bitset **only at the character-tier barrier**, which is once every
120 ticks. Do not scan each tick.

This is correct only because `deeds` is monotone upward. A monotone
accumulator makes the bit a **level** and not an **edge**. A unit that
becomes eligible at tick 3 is still eligible at tick 120, so a late scan
misses nothing. A non-monotone accumulator would need an edge bit, and an
edge bit must be collected every tick or it is lost.

**State this as a design constraint on the content, not as an
implementation detail.** If any rule ever reduces `deeds`, the lazy scan
breaks silently. Check the constraint in the deeds kernel in debug builds.

Cost of the lazy scan: 2 microseconds, 12 times for each simulated year,
which is 24 microseconds for each year. That is the whole trigger cost at
one million units.

### 9.5 Promotion is budgeted, not automatic

Do not promote every eligible unit. The character tier has a hard ceiling of
262,144, and an automatic promotion would reach it.

At each character-tier barrier:

1. Scan the eligibility bitset. The scan yields unit identifiers in
   ascending order, because a word scan walks the index space in order.
2. Sort the eligible set by the key vector `(deeds descending, unit id
   ascending)`. This is the same key-vector mechanism as a succession
   policy, so it inherits totality and transitivity by construction.
3. Compute the budget:
   `budget = max(0, target_living - current_living)`.
4. Promote the first `budget` entries.

The budget makes the character population self-regulating. Deaths open
slots; promotions fill them. The population settles at `target_living` and
can never exceed the ceiling of section 8.3.

**Expected rate.** With a mean lifespan of 60 years and a target of 100,000
living characters, about 1,667 characters die for each year. The promotion
rate therefore settles at about 1,667 for each year, which is about 139 at
each monthly barrier.

### 9.6 What a promoted soldier gains, and what it costs

The promotion writes one character row and one link.

| Field | Source |
|---|---|
| `seat` | The unit's current tile. |
| `birth_tick` | The unit's `spawn_tick` column. |
| `renown` | Derived from `deeds` by a fixed table. |
| `traits` | The unit's interned upgrade set, mapped through a table. |
| `culture`, `faith` | The owning faction. |
| `house` | A new house, founded by this character. |
| `father`, `mother` | None. See section 9.7. |
| `unit` | The unit identifier. This is a new column. |

Two new columns are needed. The character row gains `unit: u32`, where
`u32::MAX` means disembodied. The unit row gains `character: u32` and
`spawn_tick: u32`.

**The storage effect is smaller than it looks.** The character row grows
from 64 bytes to 68, and padding takes it to 72. The storage table of
section 2.4 already rounded a living character to 200 bytes and a dead
character to 80 bytes, and 72 + 128 + 4 is 204 while 72 + 8 is 80.
**Every figure in the storage table stands unchanged.** Note that the
arena is struct-of-arrays, so the row size is an accounting figure and not a
locality claim; adding a column adds one array and does not disturb the
others.

The unit side costs 8 bytes for each unit, which is 8 MB at one million
units.

**Cost of one promotion.** The write touches about 18 struct-of-arrays
columns at a scattered index, so it is about 18 cache misses at about
100 ns, which is about 2 microseconds. At 1,667 promotions for each year the
whole pass costs **about 3.3 ms for each simulated year**, which is 0.003 ms
for each tick. It does not appear in the budget.

### 9.7 Invented parents, or a null lineage

**Recommend a null lineage. Do not invent parents.**

Reject invented lineage on arithmetic. An invented ancestry of depth `d`
needs `2^d - 1` extra dead rows for each promotion.

| Invented depth | Rows for each promotion | Rows over 500 years | Bytes |
|---|---|---|---|
| 1, the parents only | 2 | 1.67 M | 120 MB |
| 2, add grandparents | 6 | 5.0 M | 360 MB |
| 4 | 30 | 25 M | 1.8 GB |

Even depth 1 costs 120 MB over 500 years and buys nothing that gameplay
reads. Depth 4 is larger than the entire character arena.

A promoted soldier therefore receives `father = None`, `mother = None`, and
a **new house of which he is the founder**. His kinship coefficient against
every existing character is exactly zero, which is both correct and
narratively right: a promoted commoner is unrelated to the nobility.

**State the one consequence plainly.** A null-lineage character is not
eligible under any kinship-keyed or house-keyed succession policy. A
promoted soldier cannot inherit a title by blood. He can hold an office by
appointment, he can found a house, and his children can inherit from him.
That is a design consequence and not a defect, and the owner should confirm
it.

The lazy option stays open at no cost. If a rule ever needs a parent,
invent one at that moment and write it. Nothing in the design forecloses
this, because a null parent and an invented parent occupy the same column.

**How the interval tree takes a new root, and what that costs.** A
null-lineage character is a root of the father forest, so the Euler interval
labelling must graft him in. The scheme handles this without a special case,
for one reason: **label a virtual super-root, not a forest.** Every real root
is a child of one virtual node, so the structure is always one tree and the
containment test of section 3.2 is unchanged.

Reserve a trailing gap inside the super-root's interval for new roots. A
promotion then takes an interval `[t, t + w)` from the tail and bumps a
cursor. That is two integer writes and one increment, so **grafting a new
root costs O(1) and adds no pass**.

Size the reserve. Give each new root `w = 1024` label slots for its future
descendants. At 1,667 promotions for each year that consumes 1.7 million
slots for each year. A `u32` label space holds 4.29 billion slots, so the
reserve only has to survive until the next yearly relabel, and it does so
with three orders of magnitude to spare.

**The yearly rebuild that section 3.4 already specifies absorbs the
fragmentation.** It renumbers every label and reclaims every unused gap. So
the promotion path adds no new index maintenance at all: it consumes reserve
during the year and the existing 19 ms rebuild returns it.

Set `depth = 0` for a new root. The kinship recursion of section 3.6
terminates immediately on a null parent and returns zero, so no special case
is needed there either.

### 9.8 Death, disembodiment, and the absence of demotion

**There is no demotion.** A promoted character never returns to the mass
tier. Two reasons. Identity does not un-happen, and a legends-mode history
that deletes its subjects is not a history. And a demotion would have to
delete a character who may have living children and inheritable assets,
which means running the full death and succession path for a routine event.

Three lifecycle transitions exist instead.

**Transition 1: disembodiment.** A character stops standing in a skirmish
line and takes a court or a command post. Set `unit = u32::MAX` and despawn
the unit. The character continues. This is the path from a soldier to a
courtier, and it costs one column write.

**Transition 2: character death at a barrier.** The character tier decides
the death from age or from an event. Emit `CharacterDied` in phase 5, then
despawn the linked unit in phase 6. The order follows the existing phase
split, because a despawn is structural.

**Transition 3: unit death between barriers.** This is the case that needs
a rule, because a soldier dies in combat at an arbitrary tick and the
character tier does not run until the next barrier.

> A unit despawn that carries a non-null `character` link sets the
> `pending_death` flag on that character in phase 5, at O(1), and emits an
> `EmbodimentEnded` event. The character-tier barrier consumes the event
> and runs the death, the succession and the asset transfer.

The character is therefore **dead but unresolved for at most 120 ticks**.
State that latency in the documentation. It is safe, because the invariant
is that a character with `pending_death` set cannot act: the decision pass
skips it, and no verb accepts it as a target. No rule reads stale state.

### 9.9 The determinism rule for promotion ordering

Many units may cross the threshold in one period. Promotion allocates new
character identifiers, so it has the same hazard as birth.

**Rule 1. The eligible set is collected in ascending unit identifier
order.** A bitset word scan yields set bits in ascending index order with no
sort at all, so the input to the ranking sort is already canonical. This is
a further reason to use a bitset rather than a per-thread candidate list.

**Rule 2. Rank by the key vector `(deeds descending, unit id ascending)`.**
The unit identifier is unique, so the order is total. No comparator function
is accepted here either, for the reason that section 6.3 gives.

**Rule 3. Allocate character identifiers in ranked order**, after the sort
and after the budget cut. Never allocate during the scan.

**Rule 4. Key every promotion draw on the unit, not on the character.** Use
`(SYSTEM_CHARACTER, tick, unit_id, draw_index)`. The new character has no
identifier when the name and the personality are drawn. This is exactly
parallel to the rule that a birth draw keys on the mother.

### 9.10 The Nemesis system as the reference point

The Nemesis system is the closest published prior art for this mechanism,
and it is the reason the promotion path matters rather than being a
curiosity.

The granted United States patent assigned to Warner Bros. Entertainment
describes non-player enemies that are generated uniquely for each
playthrough, that each receive a personality, and that rise and fall within
a social structure as the game progresses.[^13] A nameless enemy that
defeats the player is promoted into a named, ranked individual with a
history against that player. That is a tier promotion in the sense of this
section.

**Mark the limits of this citation.** A patent describes a claimed method.
It does not describe the implementation that shipped. No developer
documentation of the shipped Nemesis system exists. Treat every specific
claim about how the shipped game stores or evaluates a nemesis as
unverifiable. The patent runs to 2036, so read it as prior art and do not
copy its claimed method.

---

## 10. Formations as organisational nodes

### 10.1 The model

The owner has chosen the individual soldier as the simulated entity. A
regiment, a company or an army may still exist as an **organisational
node**: an entity that owns soldiers, that has a commander, that occupies a
place in the chain of command, and that can receive an order.

A formation is a node in the character-tier hierarchy. It is not a row in
the mass arena.

```rust
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct FormationRow {
    parent:    u32,   // FormationId, the chain of command. u32::MAX at the root.
    commander: u32,   // CharacterId
    home:      u32,   // optional recruiting region, an L1 cell. u32::MAX means none.
    l1_mask:   u32,   // index into the derived bounding level 1 cell set
    kind:      u16,   // FormationKindId, an index into the type table
    faction:   u16,
    depth:     u8,    // depth in the chain of command
    flags:     u8,
    _pad:      [u8; 2],
}
```

The row is 24 bytes. At 10,000 formations the whole table is 240 KB.

**The row holds no strength and no composition.** An earlier draft of this
report cached a member count here. Remove it. The pyramid already maintains
a per-cell unit count, a faction mask and a type histogram, so army strength
in a region is computed state and not modelled state. A cached strength
would be a second source of truth that the dirty-pyramid update does not
maintain. Derive every aggregate from the pyramid.

### 10.2 Membership without a per-member list

**Recommendation: formation membership is ownership. Reuse the ownership
machinery of section 4 without change.**

Add one column to the unit row, `formation: u32`, at 4 MB for one million
units. Build the reverse index as a compressed sparse row structure keyed on
the formation, using a counting sort, exactly as the ownership reverse index
is built. That costs 8 MB and about 5 ms for each rebuild at one million
units.

The alternative was a per-formation list of unit identifiers. **Reject it,
and reject it on maintenance and not on size.** Its size is comparable, at
about 4 MB. Its failure is that a despawn invalidates a slot, and every
formation list that holds that identifier must then be repaired. The forward
column has no such problem: a despawned unit takes its `formation` value
with it, and the next rebuild simply does not see it.

So a formation owns its soldiers in exactly the sense that a character owns
an asset. One owner column, single-valued and total, plus a rebuilt reverse
index. **No new code is required.**

### 10.3 A formation is a selector leaf, and an order is an ordinary verb

The lead asks whether a formation could be a selector predicate rather than
a membership list. **It is both, and they are the same object.**

The `formation` column makes the predicate `f.formation == F` expressible.
The reverse index makes it fast. The resolution is an **index probe**, which
is one of the three plans that the selector engine already has: the
compressed sparse row run for formation `F` is a sorted unit identifier list,
and the result representation converts a sorted list to a chunk mask in one
pass.[^9] The cost is proportional to the members of `F` and not to one
million.

The pyramid plays no part. A formation identifier has high cardinality, so
it cannot carry a useful summary, and it does not need one.

**A command to a formation is therefore not a new mechanism.**

```python
army.move_to(dest)
# desugars to
world.units.in_formation(army).move_to(dest)
```

The formation contributes one selector leaf. Everything after that is the
existing set-valued verb path: resolve, validate, plan, execute. There is no
per-member iteration in Python and none in the command layer.

**Subordinate formations.** An order to a parent formation must reach its
subordinates. Expand the formation tree first, with the same sorted
level-synchronous wavefront that section 5.3 specifies for the office
cascade, then union the compressed sparse row runs. Cap the chain-of-command
depth at 6, which covers army, corps, division, regiment, company and
platoon. The expansion is `gather`, then `sort`, then a unique `scan`, so it
stays inside the kernel vocabulary.

### 10.4 The bounding mask, and why it is needed for a different reason

The `l1_mask` field is not needed to find the members. The reverse index
already does that. It is needed for the **region scope** that the command
scheduler requires: two commands may run in parallel when both are local and
their resolved regions are disjoint.[^9]

Refresh the bounding mask at the character-tier barrier by reducing the
member tiles up to level 1. That is a `map` then a `reduce`, over the
members only.

**The tile capacity of 8 makes this mask genuinely multi-cell.** A company of
100 soldiers occupies at least 13 tiles. A formation is therefore spatially
extended by construction, and a formation order is a multi-region command.
The scheduler must expect that, and a design that assumed a formation sits
on one tile would be wrong.

### 10.5 The full hierarchy, stated once

The owner asked for unit-level hierarchies, chains of command and feudal
vassalage. Those are three uses of two structures.

```
character (liege)  --parent pointer-->  character
character (commander)  --commands-->  formation
formation  --parent pointer-->  formation
formation  --owner column + CSR-->  unit
```

Two edge types carry all of it. Both are parent pointers with a depth cap,
which is the structure that section 3.2 recommends for mutable hierarchies.
The soldier-to-formation edge is the ownership structure of section 4. **No
third structure is needed.**

### 10.6 Cost

| Item | Size | Rebuild |
|---|---|---|
| `formation` column on the unit | 4 MB at 1 M units | none |
| Reverse index, compressed sparse row | 8 MB at 1 M units | about 5 ms |
| Formation rows | 240 KB at 10,000 formations | none |
| Bounding masks | under 100 KB | under 1 ms |
| **Total** | **about 12 MB** | **about 6 ms for each barrier** |

Twelve rebuilds for each simulated year cost about 72 ms for each year,
which is 0.06 ms for each tick. Formations roughly double the Rust character
tier cost and leave it far below every other line in the budget.

### 10.7 Spatial command, evaluated against explicit membership

A larger group is a coarser view and not a new entity type. That framing is
right, and section 10.1 applies it: aggregate quantities come from the
pyramid and are never stored on a formation.

The framing suggests a stronger form. **A formation could be a place.** A
commander commands a region, the units in the region are his command, and a
unit that leaves the region leaves the command. Then a formation is a
selector predicate over an aggregate and the `formation` column disappears.

**Evaluate this seriously, because it would save 12 MB.** It fails, and it
fails on function rather than on cost.

**Failure 1: a detached unit cannot be recalled.** A scouting party that
leaves the region leaves the command. Recall is an order, and the order
cannot reach it, because the order's recipient set is the region. This is
not a loss of fidelity. It is a command that cannot be expressed.

**Failure 2: interpenetration.** Two formations of one faction in one region
cannot be separated by any spatial predicate. A faction filter separates
hostile forces and does nothing for two of a player's own armies.

**Failure 3: a garrison and a field army share tiles.** Spatial command
merges them, and the garrison then marches out with the field army.

**Failure 4: scattered levies.** Units raised across many provinces form one
army and are spatially disjoint when raised. No region contains them and
nothing else.

**Failure 5, and it is the structural one: a region is not stable under
movement.** If the command is the region, then a movement order changes its
own recipient set. Snapshot selector semantics make this safe within one
frame, because the selector resolves once against the sealed state. Across
frames it does not hold: the formation dissolves as it marches. **A command
structure must be stable across frames, and a spatial one is not.**

### 10.8 The recommended hybrid

**Explicit membership is the authority. The region is a default and a
cache.**

1. **The `formation: u32` column on the unit is authoritative.** Section
   10.2 stands unchanged. Cost 4 MB, plus 8 MB for the reverse index.
2. **The bounding level 1 mask is derived.** It exists for the scheduler's
   region scope and for nothing else. Section 10.4 stands.
3. **Every aggregate is derived from the pyramid.** Section 10.1 stands.
4. **The `home` region is a default recruiting rule, not a membership
   rule.** A unit that is spawned in a formation's home region, or that
   enters it while carrying no formation, joins that formation. After that
   the unit carries the column and the region has no further authority.

Rule 4 gives spatial grouping exactly where it is convenient, which is
garrisons and levies, and it never takes a unit out of a command.

**Cost of the hybrid over pure explicit membership.** One `u32` on the
formation row, which is nothing, plus one check in the spawn path and one
for units whose `formation` is null. Evaluate the second check at the
character-tier barrier and not each tick, and it costs nothing per tick: it
is one filtered scan over the units with a null formation column.

**Cost of pure spatial command.** It saves the 4 MB column and the 8 MB
reverse index, which is 12 MB. **Twelve megabytes does not buy five broken
cases.** Reject it.

**What a commander's node points at.** Not a level 1 cell, not a set of
cells, and not a stored member list. A commander points at a formation
identifier. The formation reaches its members through the owner column and
the compressed sparse row index. The level 1 mask hangs off the side and is
derived.

---

## 11. Appointment, patronage and opinion

### 11.1 Why appointment matters

Section 9.7 establishes that a promoted soldier has no ancestry, founds a
new house, has a kinship coefficient of zero against everyone, and therefore
cannot inherit a title by blood. The project owner has confirmed that rule.

The owner has added a mechanic: a title holder, or an upstream holder, may
assign a title to a favoured unit.

**These two rules are one design.** Blood inheritance needs lineage.
Appointment needs favour. The rule that shuts one path is what gives the
other path meaning. A risen soldier rises by merit and by patronage, and the
engine must model favour to make that work.

Favour is a relation between two characters, and relations are the most
expensive structure in this whole report. Section 11.5 gives the arithmetic.

### 11.2 Appointment as a verb

The verb is `grant_title(title, candidate)`.

**Who may appoint is policy, and it lives in the policy row that section 6.5
already puts on the title.** Add two fields to that row.

| Field | Values | Meaning |
|---|---|---|
| `appointment` | `Never`, `ByHolder`, `ByLiege`, `ByEither` | Who may grant this title. |
| `revocable` | 0 or 1 | May a granted title be taken back. |

This costs 2 bytes on a structure that already exists. **Appointment is
therefore a field of the succession policy, not a second policy object.** A
region that grants offices freely and a region that does not are two rows in
the same table, exactly as the succession laws are.

**Two cases, and both are allowed.** A grant of a vacant title is
uncontroversial. A grant of a held title is a dispossession, and the
`revocable` bit gates it. Keep the bit, because dispossession is what
creates rebellion, and a design that cannot express it loses the mechanic
that makes appointment risky.

**Does an appointment override a blood claim? Yes for the state, and it
produces a claim edge.** The engine writes the owner column to the
appointee. It then runs the ordinary filter, sort and allocate of section
6.1 to find who *would* have inherited. If that character is not the
appointee, the engine emits a claim edge from the passed-over heir to the
title, with a strength from the policy.

**This is the mechanism of section 6.6 reused without change.** Appointment
needs no new conflict machinery, because a rival law and a rival grant
produce the same object: a claim.

**Revocation** works the same way. The title returns to the appointer, and
the dispossessed holder receives a claim edge. Reject a revocation when
`revocable` is 0, and report the rejection with the reason
`TITLE_NOT_REVOCABLE`.

### 11.3 Ordering within one barrier

Several appointments may resolve in one pass, and several may target the
same title.

**Rule: process appointments in ascending order of the pair `(title
identifier, command sequence number)`. The first appointment for a title
wins. Reject every later appointment for the same title with the reason
`TITLE_ALREADY_GRANTED`.**

The sequence number comes from the seal barrier, so it is already fixed
before any Rust code runs. First-wins over a total order is deterministic
and needs no new mechanism.

**The barrier pass order matters more than the tie-break, and it must be
stated.**

```
1. deaths          resolve pending deaths and embodiment ends
2. succession      each vacated title resolves under its own policy
3. appointments    grants and revocations, in (title, sequence) order.
                   A grant that names a unit promotes that unit inline.
4. promotions      the automatic deeds path fills the remaining budget
5. indices         reverse indices, bounding masks, Euler relabel
```

Succession runs before appointment. A death vacates a title, the law names
an heir, and a grant may then override that heir. The reverse order would
grant a title whose holder is about to die.

Appointment runs before automatic promotion, because a grant that names a
unit performs its own promotion, and the automatic pass must see the result.

### 11.4 Appointment is the second promotion path

The owner's wording names a unit, not a character. So a lord who notices a
distinguished soldier and grants him a title must **cause** the promotion.
The soldier must not have to promote himself first.

There are therefore two paths into the character tier.

| Path | Driver | Budget |
|---|---|---|
| Automatic | The monotone `deeds` threshold of section 9.3 | Consumes the population budget |
| Patronage | A player or an AI issues `grant_title` | Consumes the same budget, first |

**Recommendation: an appointment consumes the same budget, and it consumes
it first, and the target is soft while the ceiling is hard.**

Work through the three options, because the lead is right that the obvious
two are both wrong.

**Option A, bypass the budget.** Rejected. It removes the self-regulating
population bound of section 9.5, and the bound is what keeps the Python cost
of section 8.3 inside its budget.

**Option B, share one budget with automatic promotion.** Rejected on play,
not on cost. An appointment would fail because an automatic pass earlier in
the same barrier had already spent the budget. A player action then fails
for a reason the player cannot see.

**Option C, recommended.** Appointments run before automatic promotions, so
an appointment always sees the whole budget. The automatic pass takes what
remains. When the budget is already zero, **the appointment still succeeds
and the population overshoots the soft target.** The next barrier computes
`budget = max(0, target_living - current_living)`, finds it zero, and
promotes nobody. The population self-corrects within one or two barriers.

Reject an appointment only at the hard ceiling of 262,144, and report the
rejection with the reason `CHARACTER_TIER_FULL`.

**The target is a control input. The ceiling is an invariant.** Under option
C an appointment never fails in normal play, the population still converges
on the target, and the hard bound of section 8.3 is preserved exactly.

The cost is unchanged at about 2 microseconds for each promotion.
Appointments are rare, so they add nothing measurable.

### 11.5 How a lord finds a favoured unit

A unit has no opinion edges. Opinion is a character-tier relation, and a
unit is a mass-tier entity. **So "well liked" cannot be read from a stored
relation for a unit at all.** The resolution by selector is not an
optimisation. It is the only available answer, and that is a stronger reason
than performance.

The lead expects a selector over `deeds` within the appointer's formation or
region, then a small sort. **Confirmed, with one addition.**

```python
pool = world.units.in_formation(lord.command) & (f.deeds > threshold)
best = pool.rank(["deeds", "service_ticks"], take=8)
```

Rank on the key vector `(deeds descending, service_ticks descending, unit
identifier ascending)`. Merit is `deeds`. Loyalty is length of service, and
it comes free: `service_ticks = current_tick - spawn_tick`, using the
`spawn_tick` column that section 9.6 already adds. Without the second key,
patronage is purely meritocratic, which loses the mechanic.

**Scope by the formation subtree or by the bounding region mask.** Both
exist already, from section 10. The cost is proportional to the members of
the command and not to one million, because the resolution is an index probe
against the formation reverse index.

The final key is the unit identifier, so the order is total. The same rule
that section 6.3 gives for succession applies here: no comparator function
is accepted.

### 11.6 Opinion storage, and the correction to the scaling claim

**A dense opinion matrix is impossible.** At the ceiling of 262,144
characters there are about 68 billion ordered pairs. Reject it immediately.

**But sparse opinion is not quadratic, and the reason matters.** Storage is
quadratic only if the number of edges for each character grows with the
population. **Cap the out-degree at `K`, and storage becomes exactly
`N × K × edge_bytes`, which is linear in `N`.** The cap is the mechanism
that converts the quadratic into the linear, and it must be a hard,
enforced number rather than an expectation about behaviour. Without it, a
long-lived and well-connected character accumulates edges without bound.

**The edge.**

```rust
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct OpinionEdge {
    other:      u32,   // the target character
    last_tick:  u32,   // when this edge was last written
    respect:    i16,   // competence. Drives appointment.
    affection:  i16,   // personal warmth. Drives marriage and friendship.
    fear:       i16,   // coercion. Drives compliance.
    obligation: i16,   // debt owed. Drives feudal service.
}
```

Opinion is **directed**. What A thinks of B is not what B thinks of A. Store
the outgoing edges only, and build a reverse index of 4-byte identifiers
when an incoming query is needed.

**How many axes, and the surprising answer on cost.** The key and the
timestamp cost 8 bytes before any axis exists. One axis gives a 12-byte
record, which pads to 16. Four axes give exactly 16. **The second, third and
fourth axes are free**, because the key overhead dominates and the padding
absorbs them. The cost multiplier from one axis to four is 1.00, not 4.00.

**Recommend exactly four axes.** Choose them on design grounds, because cost
does not discriminate. A fifth axis costs 8 more bytes for each edge, which
is a real 50 percent increase, so four is also the natural stopping point.

**Storage.** The table uses `K = 32` and includes the 4-byte reverse index,
so 20 bytes for each edge and 640 bytes for each living character.

| Living | Opinion edges | Opinion bytes | Character arena, from section 2.4 |
|---|---|---|---|
| 10,000 | 320,000 | **6.4 MB** | 8.6 MB |
| 100,000 | 3.2 M | **64 MB** | 87 MB |
| 262,144 | 8.4 M | **168 MB** | 226 MB |

**Check of the lead's figures.** The lead estimated about 4 MB at 10,000
with 50 edges for each character, and over 100 MB at 262,144. Those figures
are correct for a single-scalar 8-byte edge with no reverse index:
`10,000 × 50 × 8` is 4.0 MB and `262,144 × 50 × 8` is 105 MB. Four axes and
a reverse index take the edge to 20 bytes and a cap of 32 takes the degree
down, and the two changes nearly cancel. **The lead's order of magnitude
holds.**

**The consequence, stated plainly.** Opinion is the second-largest structure
in the character layer, and it is within 25 percent of the character arena
at every scale. It roughly doubles the memory cost of a living character,
from about 200 bytes to about 840.

### 11.7 The default opinion, which is derived and never stored

Most character pairs have never interacted and must still have an opinion.
Derive it. Do not store it.

```
default(A -> B) = w_dip  * diplomatic_relation(faction(A), faction(B))
                + w_house* (house(A) == house(B))
                + w_faith* (faith(A) == faith(B))
                + w_cult * (culture(A) == culture(B))
                + w_kin  * r(A, B)
                + w_trait* trait_compatibility[traits(A)][traits(B)]
                + w_fame * renown_band(B)
```

Every term is an integer and every weight is `Fix32`, so the sum is exact
and order-free.

**Two terms come free from work this report already specifies.** The kinship
term `r(A, B)` is the coefficient of relationship of section 3.6, computed
by the memoised truncated recursion. The diplomatic term reads the relation
plane that the fog of war work already maintains.

The trait term is a lookup on two interned trait set identifiers. Trait sets
intern, so the distinct pair count is small. Memoise the whole default on
the key `(house, faith, culture, traits)` for each side. The memo table is
probed and never iterated, which satisfies the determinism rule of section
14.

### 11.8 Edge lifecycle, and why decay must target the default

**Creation.** Create an edge when an interaction moves any axis by more than
a minimum delta. A trivial interaction must not allocate an edge, or the cap
fills with noise.

**Decay targets the derived default, not zero.** This is the important
choice, and it is not only a fidelity argument.

```
value_now = default + (value_stored - default) * decay ^ elapsed_months
```

If decay ran toward zero, a forgotten enemy would end up neutral, which is
wrong, and an edge at zero would still carry information. Decaying toward
the default means **an edge that has decayed to its default carries no
information at all and can be deleted with no loss.** Decay to default makes
pruning correctness-preserving rather than lossy. That single property
removes the whole question of what to prune.

**Apply decay lazily, at read.** Store `last_tick` and evaluate the
expression above when the edge is read. Use a lookup table over elapsed
months for the decay factor, so the operation is one table read, one
multiply and one shift, all integer.

The alternative is a periodic pass over every edge. At 100,000 characters
that is 3.2 million edges, which is about 3.2 ms for each barrier and 38 ms
for each year. **Lazy decay removes that line entirely**, and it would
otherwise be the largest single item in the Rust character budget.

**Pruning happens only on cap overflow.** When a character's edge count
reaches `K`, evict the least salient edge, where salience is the largest
absolute difference between any stored axis and its default. Order the
eviction by the key vector `(salience ascending, last_tick ascending, other
identifier ascending)` and drop the tail. The order is total.

There is therefore **no periodic decay pass and no periodic prune pass**.
The recurring cost of opinion is zero.

**Death: drop every opinion edge of the deceased, in both directions.**

This confirms the answer that section 17 already proposes for relation
edges, and it makes the answer far more expensive to get wrong. Compute the
cost of the alternative. The dead outnumber the living by about 8.3 to one
after 500 years, so retaining opinion edges on the dead costs 8.3 times the
opinion table.

| Living | Opinion on the living | If the dead keep opinion |
|---|---|---|
| 100,000 | 64 MB | **531 MB** |
| 262,144 | 168 MB | **1.39 GB** |

Nothing reads those edges. A dead character forms no new opinions and is the
subject of no new decisions. **Drop them, and let the biography log carry
the interaction that created the edge.** Section 16 already retains births,
deaths, title transfers, and the events of characters who held office or
achieved renown, which covers every interaction a narrative would want.

### 11.9 Determinism for opinion

**Rule 1. Accumulate opinion deltas by sort and segmented reduce, never by
atomics.** An opinion delta is an integer add, and integer addition is
exactly commutative and associative. Sort the pending deltas by the pair
`(source, target)` and reduce each segment. This is the same kernel the
economy pass uses, and it removes every atomic from the pass.

**Rule 2. Store each character's edge run sorted ascending by the `other`
identifier.** Then iteration order is defined by the data and a probe is a
binary search over at most 32 entries. Merge new edges into the sorted run
at the barrier.

**Rule 3. Never iterate a hash container.** The default-opinion memo table
is probed and never iterated, which is permitted. The edge store is a sorted
run and not a map.

**Rule 4. Evict on a total order.** Use `(salience ascending, last_tick
ascending, other identifier ascending)`.

### 11.10 The consequence for the target population

**Opinion changes the recommendation for the target living population, and
the change is not small.**

Restate the whole per-character cost with opinion included.

| Item | Bytes for each living character |
|---|---|
| Character row and side structures, from section 2.2 | 200 |
| Opinion edges and the reverse index, `K = 32` | 640 |
| **Total** | **840** |

Now total the character layer at three targets, including 500 years of dead
characters and the biography log at the 5 percent retention of section 16.

| Target living | Arena and dead | Opinion | Biography, retained | Total |
|---|---|---|---|---|
| 20,000 | 17 MB | 13 MB | 4 MB | **34 MB** |
| 50,000 | 43 MB | 32 MB | 11 MB | **86 MB** |
| 100,000 | 87 MB | 64 MB | 22 MB | **173 MB** |
| 262,144 | 226 MB | 168 MB | 59 MB | **453 MB** |

**Recommend a target of 20,000 to 50,000 living characters.** Three reasons,
and each is independent.

1. **Memory.** At 50,000 the whole character layer is 86 MB, against 268 MB
   for the tile side. At the ceiling it is 453 MB, which is larger than the
   tile side.
2. **Python.** At 50,000 the decision pass costs 0.85 ms for each tick,
   which sits comfortably below trade at 1.1 ms. At the ceiling it costs
   4.4 ms and becomes the largest line in the budget.
3. **Opinion is the term that grows fastest with the target**, because it is
   more than three quarters of the per-character cost. Every increase in the
   target buys four times as much memory as the arena figures alone suggest.

**The ceiling of 262,144 stays as the hard invariant.** It is the point
beyond which the design fails. The recommended target is far below it, and
the gap is deliberate.

---

## 12. The Nemesis patent as a design constraint

### 12.1 Why this is a constraint and not a citation

Section 9.10 cites a granted United States patent as prior art for the
promotion mechanism. That patent covers subject matter close to the feature
that the project owner has now asked for. **State it as a design constraint
so that the owner can act on it, rather than leaving it in a footnote.**

**This section reports the published text of the patent. It is not legal
advice, and it is not an assessment of infringement.** Claim scope is
decided by claim construction and by the prosecution history, and this
report has examined neither. The purpose here is to record what exists.

### 12.2 The facts of the document

| Field | Value |
|---|---|
| Number | US 10,926,179 B2 |
| Title | Nemesis characters, nemesis forts, social vendettas and followers in computer games |
| Assignee | Warner Bros. Entertainment Inc. |
| Earliest priority date | 26 March 2015 |
| Filing date | 25 March 2016 |
| Grant and publication date | 23 February 2021 |
| Adjusted anticipated expiration | 11 August 2036 |
| Claims | 36, of which claims 1, 6 and 9 are independent |

### 12.3 What the independent claims recite

All three independent claims recite the same combination. Claim 1 is a
method, claim 6 is an apparatus in means-plus-function form, and claim 9 is
an apparatus. Each recites four steps.[^13]

1. Controlling game events in a computer-implemented game, where the events
   involve **an avatar that is operated in response to input from a player**
   and **a first non-player character that is controlled to respond to and
   automatically oppose avatars** based on first character parameters held in
   memory.
2. Detecting the occurrence of a predefined game event that involves **an
   interaction between the avatar and the first non-player character**.
3. **Changing second character parameters** for the control of **a second
   non-player character**, based on that detection, where the second
   non-player character is also controlled to respond to and automatically
   oppose avatars.
4. **Outputting to an output device an indication of the changed second
   character parameters.**

The dependent claims add the recognisable features of the shipped system.
Claim 2 triggers the change on the death of the avatar or on the avatar
entering a new zone. Claim 3 makes the change a change of status within **a
faction** of non-player characters. Claim 4 makes that faction **a ranked
hierarchy** whose ranks correlate with game-playing power. Claims 5 to 9
export the faction data **to a remote server and into another player's
independently operating game instance**, which is the social vendetta.
Claims 11 and 12 change appearance, personality traits, ability, a player
interaction score, or a **power centre** attribute and configuration, which
is the nemesis fort. Claim 13 selects dialogue.

### 12.4 What the recited elements require, and what sits outside them

Two elements appear in every independent claim and are worth naming plainly.

- **A player-operated avatar.** Every independent claim requires game events
  that involve an avatar operated in response to player input.
- **Non-player characters defined by opposition to avatars.** Both the first
  and the second non-player character are recited as controlled to respond
  to and automatically oppose avatars.

A world simulation that has no player-operated avatar, and whose characters
are not defined by opposing one, does not present those recited elements.
The design in this report is of that kind: promotion is triggered by a
monotone achievement accumulator and by a lord's grant, and neither refers
to a player avatar.

**The general pattern is much older and much broader.** Promoting an
anonymous mass entity into a named, persistent, tracked individual is not
novel in itself. Dwarf Fortress generated named historical figures with
recorded deeds and social positions well before the March 2015 priority
date, and dynastic character simulation with named characters, ranks and
succession is older still. **Distinguish the broad pattern, which is old
prior art, from the specific claimed combination.**

### 12.5 The line the owner should watch

The design moves toward the claimed combination if it later adds all of the
following together:

1. A player-operated avatar in the simulated world.
2. Characters whose behaviour is defined by opposing that avatar.
3. A change to a **second** character's parameters that is caused by an
   interaction between the avatar and a **first** character.
4. A ranked faction of those characters that reorders as a result.
5. Output to the player of the changed parameters.

Points 3, 4 and 5 alone are ordinary. The combination with points 1 and 2 is
what the claims recite.

**Recommendation.** Record the patent in the decision record. Keep promotion
driven by simulation state and by patronage, which is where the owner has
already put it. **If the design later adds a player avatar together with
characters that rise by defeating that avatar, obtain legal counsel before
building it.** Do not rely on this report for that decision.

Note also that the patent is a **published document**, so it is a legitimate
and citable source of prior art. Section 9.10 uses it that way. Nothing in
this section restricts reading it.

---

## 13. Simulation cadence

### 13.1 The two passes

Split the character tier into two passes with different cadences. They have
different determinism requirements, and merging them forces the stricter
requirement onto the cheaper pass.

**Pass A: the event pass. One barrier, every 120 ticks.** This pass runs
births, deaths, succession, office cascades, house splits, and the Euler
label rebuild. It is Rust. It must be global and unsharded, because
succession must see one consistent world: two deaths in the same period can
compete for the same title, and the resolution must not depend on which
shard ran first.

At 120 ticks the period is 12 seconds of wall time. Call it one simulated
month.

**Pass B: the decision pass. Sharded, every tick.** This pass runs the
Python character AI and the intrigue scripts. It is sharded across the
1,200 ticks of a simulated year by `character_id modulo 1200`. Each tick
processes one shard.

The shard function is a fixed integer function of a stable identifier, so
it is deterministic. Within a shard, characters are delivered in ascending
identifier order.

### 13.2 Why the split is necessary

At 100,000 living characters, an unsharded yearly Python pass costs 2.0
seconds. The frame budget at 10 Hz is 100 ms. The pass would overrun by a
factor of 20. The frame-loop rule says to report an overrun and never to
run a catch-up loop.[^1] So an unsharded decision pass produces a
20-times overrun once each simulated year, forever.

With 1,200 shards, each tick processes 83 characters at 20 microseconds,
which is 1.7 ms. The spike disappears.

Pass A does not need sharding. Section 15 shows that its total yearly cost
is under 50 ms, and it runs 12 times a year, so each run costs about 4 ms.
That fits in one tick.

### 13.3 The interleaving rule

Add the character tier as one more staged system, using the mechanism that
the frame loop already has. Each system has a period and a phase
offset.[^1]

| Pass | Period | Offset | Frame phase | Writes |
|---|---|---|---|---|
| A, event | 120 ticks | 0 | 5, 6, 7 | World state, through events |
| B, decision | 1 tick | 0 | 0, in Python | Commands only |

Pass A runs inside the normal write phases, so it obeys the same rules as
every other write. It emits events in phase 5 and it despawns in phase 6.
The Euler label rebuild runs in phase 7, next to the pyramid rebuild,
because both are index rebuilds after structure is stable.

### 13.4 How a slow tier stays deterministic under a fast tier

**No new mechanism is required.** State the rule as three conditions.

1. **The period and the offset are compile-time constants.** A data-driven
   period is a determinism hazard, and the frame loop already forbids
   it.[^1]
2. **A slow pass reads only values that are stable at its own barrier.** It
   must not read a value that a faster system writes within the same
   period, except through the event log. Enforce this with the existing
   access declaration: a slow system declares its reads, and the scheduler
   rejects a read of a component that a faster system writes in the same
   period without an intervening barrier.
3. **A slow pass writes only commands and events.** It never mutates
   directly. Then its writes are ordered by the same sequence numbers that
   order every other write.

The result: a character death at tick 1,200 and a unit death at tick 1,200
are ordered against each other by their command sequence numbers, and by
nothing else. The rate difference between the tiers is invisible to the
ordering.

---

## 14. Determinism rules

State these eight rules in the decision record. Each closes a specific
hole, and each hole is reachable in this domain.

**Rule 1. Key every character draw on
`(SYSTEM_CHARACTER, tick, character_id, draw_index)`.** This follows the
existing counter-based generator rule.[^1] Births, deaths, trait assignment
and personality all draw from it.

**Rule 2. Key a birth draw on the mother, never on the child.** The child
does not exist when the draw happens, so it has no identifier. Use
`(SYSTEM_CHARACTER, tick, mother_id, birth_sequence)`, where
`birth_sequence` counts births to that mother within that tick.

**Rule 3. Allocate new character identifiers in sorted order.** Collect the
pending births of a tick, sort them by `(mother_id, birth_sequence)`, then
allocate identifiers in that order. Without this rule the identifiers
depend on which thread finished first, and every downstream tie-break
inherits that dependency.

**Rule 4. Sort every graph frontier before it expands.** This applies to
the office cascade, to ancestor enumeration and to any breadth-first
search over relations. Sort ascending by entity identifier, then
deduplicate.

**Rule 5. Sort every depth-first search on the child list in ascending
identifier order.** The Euler labels depend on the visit order, and the
labels are observable through every range query.

**Rule 6. Succession sorts on an integer key vector whose final key is the
character identifier.** No comparator callback is accepted from content.
See section 6.3.

**Rule 7. Never iterate a hash container.** The kinship memo table is
probed and never iterated, which is safe. Any structure that is iterated
must be a `BTreeMap`, a fixed-hasher map that is never iterated, or an
index-sorted vector. This restates the existing project ban.[^1]

**Rule 8. Union by size, and break a size tie on the smaller root
identifier.** This applies to the world-generation grouping pass of section
3.5.

Two existing project rules apply unchanged and should be repeated in the
decision block, because a reader of the character section will not have
read the platform section. All character arithmetic is integer or `Fix32`.
Every character row and every character event is `bytemuck::Pod` with
`repr(C)` and declared padding, and uses `u8` rather than `bool`.

---

## 15. Cost summary and kernel vocabulary

### 15.1 The Rust cost, for each simulated year, at 100,000 living

| Pass | Cost for each year | Amortised for each tick |
|---|---|---|
| Euler label rebuild, 933,000 nodes | 19 ms | 0.016 ms |
| Inbreeding at birth, 1,667 births | 5 ms | 0.004 ms |
| Ownership reverse index rebuild, 1 M assets | 5 ms | 0.004 ms |
| Succession, 1,667 deaths | 3 ms | 0.003 ms |
| Asset transfer on death, 16,700 writes | 0.3 ms | under 0.001 ms |
| Renown diffusion, 64 planes | 8 ms | 0.007 ms |
| Office cascades | under 1 ms | under 0.001 ms |
| Promotion trigger scan, 1 M units, 12 barriers | 0.024 ms | under 0.001 ms |
| Promotion writes, 1,667 promotions | 3.3 ms | 0.003 ms |
| Formation reverse index, 12 rebuilds at 1 M units | 72 ms | 0.060 ms |
| Opinion decay and pruning | 0 ms, see section 11.8 | 0 ms |
| Appointments, rare | under 1 ms | under 0.001 ms |
| Lowest-common-ancestor queries, 100,000 | 36 ms | 0.030 ms |
| **Rust total** | **under 160 ms** | **under 0.14 ms** |

Compare with the running per-tick budget, which lists trade at 1.1 ms,
economy at 0.4 to 0.6 ms and influence at 0.53 ms.[^2] **The Rust character
tier costs about one quarter of the cheapest existing line, with formations
and the promotion path included.** At 10,000 living characters every figure
except the formation rebuild divides by ten; the formation rebuild scales
with the unit count and not with the character count.

Two lines dominate and both are less alarming than they look. The
lowest-common-ancestor total is large only because the table charges one
query for every living character each year, and real workloads issue far
fewer. The formation rebuild is a counting sort over one million units, and
it can be halved by rebuilding only the formations whose membership
changed.

### 15.2 The Python cost

| Living | For each year | For each tick |
|---|---|---|
| 10,000 | 0.2 s | 0.17 ms |
| 100,000 | 2.0 s | 1.7 ms |

**The Python decision pass costs 25 times the whole Rust character tier at
100,000 characters.** Every optimisation effort in this domain belongs on
the Python side or on the decision cadence. None belongs on the data
structures.

### 15.3 Where the kernel vocabulary applies, and where it does not

The project's kernel vocabulary is map, gather, scatter, reduce, scan,
sort, stencil and local join. State plainly where this domain fits and
where it does not.

**Inside the vocabulary.**

| Operation | Kernels |
|---|---|
| Eligibility filter | `map` producing a mask, then a compaction `scan` |
| Succession ranking | `map` to a key tuple, then `sort` |
| Partition allocation | `sort`, then a `map` with the largest-remainder method |
| Child list rebuild | counting `sort`, which is `scatter` plus `scan` |
| Ownership reverse index | counting `sort` |
| Transfer on death | `gather` a contiguous run, then `scatter` |
| Cadet-branch split | `map` over a contiguous range |
| Office cascade wave | `gather`, then `sort`, then a unique `scan` |
| Renown diffusion | `stencil` on the conductance plane |
| Promotion eligibility | `map` to a bit, then a `scan` of the bitset |
| Promotion ranking and budget | `map` to a key tuple, `sort`, then a prefix `take` |
| Formation membership index | counting `sort` |
| Formation order expansion | `gather`, then `sort`, then a unique `scan` |

**Outside the vocabulary.** Three kernels are genuinely new, and the
decision record should say so rather than force them into the existing
list.

1. **Pointer-chasing depth-first search**, for the Euler label rebuild. It
   is inherently serial and it has no data-parallel form at this size.
   Cost: 19 ms for each year. Affordable only because the tier is slow.
2. **Memoised recursion over a DAG**, for the kinship coefficient. The
   memo table makes the traversal order matter for performance and not for
   the result. It has no vectorised form.
3. **Upward walk to the lowest common ancestor.** It is a bounded chain of
   dependent loads. It vectorises only across independent queries, and each
   lane then diverges at a different depth.

All three are affordable for exactly one reason: **the tier holds 100,000
entities and it runs 12 times a year, not 1,000,000 entities 10 times a
second.** If the character count ever approaches the unit count, all three
must be replaced. That is the strongest single argument for the cardinality
ceiling of section 8.3.

---

## 16. History retention and the transient event log

### 16.1 The interaction, stated plainly

The project's per-tick event arena is **transient**. It is a set of
preallocated `Vec<T>` arenas that are cleared each frame with one store to
a length field.[^1] A legends-mode history is **retained** and it must
survive for the life of the world.

**These are two different structures with two different lifetimes. Do not
reuse the transient arena for history.** It is cleared, so anything left in
it is gone at the next barrier.

**The rule: copy the retained subset out at the character-tier barrier.**
Pass A walks the sealed event stream, selects the events that the retention
rule keeps, and appends them to the biography log. The copy is a filtered
`gather` and it costs a few microseconds each month.

This also aligns with the merge-note principle that derived state is not
logged.[^2] A biography event is not derived. A birth, a death, a marriage
and a succession are discontinuous facts, and the log keeps discontinuous
facts.

### 16.2 The retention rule

Section 2.5 shows that the biography log reaches 4.5 GB at one million
living characters over 500 years, and 448 MB at 100,000. Prune at the
century barrier, not each year, and prune by rule.

Keep an event when any condition holds:

1. Its subject held an office or a title at any time.
2. Its subject's peak renown exceeded a threshold.
3. Its subject is an ancestor of a living character within 6 generations.
4. The event kind is a birth, a death, or a title transfer. These are
   structural and the genealogy depends on them.

Discard every other event and mark its subject as "summarised". A
summarised character keeps its 64-byte row, so the genealogy stays intact
and every ancestry query still works. Only the narrative detail is lost.

Expect a retention rate near 5 percent under this rule, because most
characters hold no office and achieve no renown. That takes the one-million
case from 4.5 GB to about 224 MB.

**Archive rather than delete when the world is large.** Append the
discarded events to a file in the existing hand-written `bytemuck` save
format, with a version field, an endianness marker and a checksum.[^1] Keep
only an index in memory. Legends mode then reads from the file, which is
acceptable because it is not a simulation path.

### 16.3 When history must be pruned

State the trigger as a number, not as a judgement.

| Living characters | Years before the log passes 512 MB |
|---|---|
| 10,000 | about 5,700 |
| 100,000 | about 570 |
| 1,000,000 | about 57 |

**At 10,000 living characters, retention never needs to run.** At 100,000 it
runs once every few centuries. At one million it runs constantly, which is
one more reason for the ceiling of section 8.3.

---

## 17. Open questions for the record

**OQ40. How many living characters must the world hold?** Every figure in
this report scales with this number. Section 11.10 revises the
recommendation down to 20,000 to 50,000, because opinion storage costs 640
bytes for each living character against 200 for the arena. The hard ceiling
stays at 262,144. Answer this question before any other in this list.

**OQ41. How deep does lineage go?** Two parts, because both ask how much
ancestry the engine stores. Is the genealogy two-parent, which needs both
the father tree and the mother tree at 8 extra bytes for each row and a
second label rebuild? And what truncation depth does the kinship recursion
use, where six generations serves a marriage rule and twelve is the ceiling
that keeps `Fix32` exact? The depth changes the per-query cost by a factor
of about 64. The promoted-soldier case is settled: the owner has confirmed a
null lineage.

**OQ42. Does a character own assets directly, or only through an office?**
Ownership through an office removes the reverse index over characters and
replaces it with a much smaller index over offices. It also changes what
inheritance moves.

**OQ43. How much succession law ships in version 1?** Two parts. Is
partible inheritance in scope? The `Partition` allocator is about 200 lines
and needs a total order over assets, while `Take(1)` alone is about 20
lines. And may one character hold titles under two lieges with different
laws? Section 6.6 assumes yes and resolves it as a union.

**OQ44. What is the wall-time budget for the Python character pass?** This
report proposes 1.7 ms for each tick at 100,000 characters, from a 20
microsecond estimate for each decision. That estimate is not measured.
Measure one realistic decision before the ceiling of section 8.3 is fixed.

**OQ45. How much history must the world retain?** How many simulated years
in full? This report proposes a 5 percent retention rule applied at a
century barrier. Section 11.8 settles the related question about relation
edges: drop every opinion edge at death, because retaining them costs 531 MB
at 100,000 living characters and 1.39 GB at the ceiling.

**OQ46. Are characters visible to fog of war?** If a character is a fog
subject, the character tier gains a per-faction visibility relation, and
that relation is large. If characters are always globally known, it costs
nothing. This report assumes globally known.

**OQ47. What are the promotion threshold, the target living population, and
the appointment reserve?** The budget rule of section 9.5 makes the
population settle at the target, so the target is the real control and the
threshold only decides who waits in the queue. Section 11.4 recommends that
an appointment may overshoot the target and self-correct, and that only the
hard ceiling rejects it. The owner must set the target, and section 11.10
recommends 20,000 to 50,000.

**OQ48. How many opinion axes, and what is the out-degree cap?** This report
recommends four axes and `K = 32`. The fourth axis is free because padding
absorbs it, and a fifth costs 50 percent more for each edge. The cap `K` is
the number that converts quadratic storage into linear storage, so it must
be a hard, enforced limit rather than an expectation. Doubling `K` doubles
the opinion table.

**OQ49. How close will the design go to the Nemesis patent, and does the
owner want counsel?** Section 12 records US 10,926,179 B2, which runs to
2036. The current design has neither of the two elements that every
independent claim recites, which are a player-operated avatar and characters
defined by opposing it. **If a player avatar is ever added together with
characters that rise by defeating it, obtain legal counsel before building
it.** This report is a factual summary of published claim text and is not
legal advice.

---

## 18. Recommended decision block

**This section is ready to apply to the foundational architecture record.
**It uses D70 to D95 and OQ40 to OQ49.** The lead assigned D70 to D89 and
later released D90 to D95, because report 15 starts at D96. It does not
continue from the end of the record.**

---

#### D70. Characters are a separate entity tier with a declared ceiling of 262,144

The world holds three entity tiers. `Mass` holds tiles and units. `Character`
holds characters, offices and titles. `Singleton` holds the world and the
factions. The tier is a static property of an entity class, declared at
registration.

The character tier ceiling is `2^18`, which is 262,144 living characters.
Check the declared capacity when the world loads, and refuse to construct
the tier above the ceiling.

The ceiling is derived. At 262,144 characters the Python decision pass costs
5.2 seconds for each simulated year, which is 4.4 ms for each tick at 10 Hz.
That makes it the largest single line in the per-tick budget. Above it the
character layer dominates the whole simulation.

#### D71. The character row holds both parents, the Euler label and the unit link

The row holds `father`, `mother`, `house`, `liege`, `primary_title`, `seat`,
`birth_tick`, `death_tick`, `father_tin`, `father_tout`, `renown`,
`inbreeding`, `traits`, `culture`, `faith`, `depth`, `flags`, `fame_plane`
and `unit`. It is 68 bytes, and padding takes it to 72.

Every field is a fixed-width integer. The row is `repr(C)` and
`bytemuck::Pod` with declared padding and no `bool`. The arena is
struct-of-arrays, so adding a column adds one array and disturbs no other.

Storage: 8.6 MB at 10,000 living characters and 87 MB at 100,000, including
500 years of dead characters. This is not a budget concern at either scale.

#### D72. Genealogy is a DAG, indexed as two trees with gapped Euler labels

Store `father` and `mother` as the authoritative genealogy. Build a
compressed sparse row child list over every character, living and dead.
Build an Euler interval label on the father tree, and on the mother tree if
OQ41 requires it.

The label answers "is X a patrilineal ancestor of Y" in two integer
comparisons. It makes the set of all patrilineal descendants of a character
a contiguous range, so a dynasty relabel is a range write and not a
traversal.

Rebuild the label once for each simulated year, inside phase 7, next to the
pyramid rebuild. The rebuild is a depth-first search over 933,000 nodes at
100,000 living characters, and it costs about 19 ms for each year.

Reject the closure table: on a DAG of depth 20 it needs up to one million
ancestor rows for a single character. Reject the materialised path: a
variable-length field breaks the `Pod` row. Reject bitset ancestry: it needs
64 or fewer founders.

#### D73. Kinship is a memoised, truncated Karigl recursion, and it is exact in fixed point

Compute the kinship coefficient `f(i, j)` on demand with the recursion
`f(i, j) = (f(father(i), j) + f(mother(i), j)) / 2`, expanding the younger
argument and memoising on the ordered pair.

Every step halves a value, so every kinship coefficient is an integer over a
power of two. `Fix32` in Q16.16 represents such a value exactly down to
`2^-16`. **Truncate the recursion at 12 generations. Then every intermediate
value is exact and no step rounds.** The integer form is the correct form
here, not a concession to the no-float rule.

Truncate at 6 generations for gameplay tests. That bounds the memoised
recursion at 3,969 pairs.

Expose Wright's coefficient of relationship, which is `2 f(i, j)`, in the
Python API. Do not expose `f`.

**The inbreeding coefficient is computed once, at birth, and stored.** `F(i)`
equals `f(father(i), mother(i))`. A character's parents never change, so the
value is immutable. Compute it in the birth event and write it to the
`inbreeding` column. Never recompute it. At 100,000 living characters there
are about 1,667 births for each year, and the whole pass costs about 5 ms
for each year.

#### D74. House membership is a column, not a union-find. A cadet split is a range write

Store `house` as a `u32` column. A birth copies the father's house.

Union-find cannot split, and a cadet branch leaving its parent house is a
split. Use union-find only for the one-off grouping pass during world
generation. In that pass, union by size and break a size tie on the smaller
root identifier.

A cadet split rewrites the `house` column over the Euler range of the
founder's descendants. That is a `map` over a contiguous span.

#### D75. Mutable hierarchies use parent pointers with a depth cap of 12

The chain of command, vassalage and the office hierarchy are shallow trees
that change often. Store a parent pointer and a depth. Cap the depth at 12
and check the cap when an edge is added.

Compute the lowest common ancestor by lifting the deeper node and then
walking in lockstep. The cost is at most 24 pointer reads, which is about
360 ns.

Do not build Euler tour plus range-minimum-query preprocessing for these
trees. It gives an O(1) query and needs an O(N) rebuild on every grant. At
this depth the naive query already costs less than the rebuild would.

#### D76. Ownership is one owner column plus a rebuilt reverse index. A dispute is a claim edge

Each asset carries `owner_character` and `owner_faction` as two `u32`
columns, with the invariant that at most one is set. Do not use a tagged
union.

Build the reverse index as a compressed sparse row structure keyed on the
owner, using a counting sort. Rebuild it at the character-tier barrier, and
early when a patch list exceeds one percent of the asset count. The rebuild
costs about 5 ms at one million assets.

Transfer on death is a bulk reassignment over a contiguous run. At one
million assets and a 60-year lifespan it moves about 16,700 assets for each
year, at about 0.3 ms.

Shared ownership is an interned `share_set: u32` on the side. A dispute is a
claim edge `(claimant, asset, strength, since)` and never a second owner
column. A conditional holding is a `condition: u16` column evaluated at the
character-tier barrier.

**The owner column must stay single-valued and total.** Every extension
hangs off the side, because the owner column is read on hot paths.

#### D77. An office is an entity anchored to another entity. The anchor graph is a forest of depth 8

An office holds `anchor`, `holder`, `kind`, `policy` and `flags`. A court
exists because a castle exists, so the court's anchor is the castle.

Constrain the anchor graph to a forest with a maximum depth of 8. Reject an
office creation that exceeds the depth or that would form a cycle. A general
DAG would need a policy for partial destruction, and that policy has no
obvious answer.

#### D78. Anchor destruction cascades as a sorted level-synchronous wavefront in phase 6

When an anchor is destroyed, expand the dependants wave by wave. Sort each
wave ascending by entity identifier and deduplicate it before it expands.
Apply the waves deepest first, so a parent never despawns before its child.

The cascade runs in phase 6, the structural phase, because it despawns
entities and invalidates indices. It terminates in at most 8 waves.

The cascade emits `OfficeVacated` and `OfficeDestroyed` as two separate
event types. A vacancy feeds the succession pass. A destruction feeds the
structural pass.

Each wave is `gather`, then `sort`, then a unique `scan`. The cascade is
therefore inside the kernel vocabulary.

#### D79. Succession is filter, then sort, then allocate

A succession policy is a triple: an eligibility predicate, an ordered vector
of integer key extractors, and an allocator.

Two allocators cover the whole survey. `Take(1)` gives every asset to the
first candidate. `Partition { primary_share }` gives the first candidate a
share of the ranked asset list and deals the remainder round-robin in rank
order.

Partible inheritance therefore does not break the model. It is a second
allocator over the same filtered and sorted list.

Sort the assets by `(tier descending, asset id ascending)` before a
partition deal, or the split is not deterministic. Use the largest-remainder
method for a fractional share, as the `transfer` verb already does.

Primogeniture, ultimogeniture, seniority, elective, partition, agnatic,
enatic and proximity of blood are all rows in a table. None of them is code.

#### D80. A succession policy is a key extractor vector. A comparator function is forbidden

A policy holds up to 8 sort key slots. Each slot names an extractor from an
immutable engine table and a direction. The engine appends a final,
implicit, unauthorable slot that extracts the character identifier
ascending.

**Do not accept a comparator function from content.** A comparator can be
intransitive, and an intransitive comparator makes the output of a sort
depend on the sort algorithm and on the input order. A tie-break on the
identifier does not repair a cycle, and detecting a cycle costs `O(n^3)`
comparisons.

The key vector makes totality and transitivity structural. It also makes the
sort a least-significant-digit radix sort over a fixed-width tuple of `i64`,
with no comparison callback and no branch.

#### D81. The succession policy is a column on the title. A rival law makes a claim, not a conflict

The policy is 2 bytes on the title, not on the character and not on the
tile. The law travels with the object that the law governs.

A character who holds three titles under three laws is a normal case. Each
title resolves independently.

Regional variation follows for free, because a title anchors to a place.
Nothing in the tile grid needs to know that succession exists.

**A losing candidate under a recognised alternative policy becomes a claim
edge.** Two policies cannot conflict, because each title resolves under
exactly one policy. One heir inheriting under two laws is a union and needs no rule.

When the title's policy names heir X and a recognised alternative policy
would have named heir Y, the title transfers to X, and the engine emits a
claim edge from Y to that title with a strength derived from the recognising
authority.

A title carries a bitmask of recognised alternative policies, capped at 16.
The cost is at most 16 extra evaluations for each death, over a candidate
list that is already gathered.

This keeps the state machine total. There is never an unowned title, and a
legal ambiguity becomes a gameplay object rather than an engine special
case.

#### D82. Renown is a character scalar. Its spatial reach is a `u8` field, capped at 64 planes

Separate three quantities. **Renown** is a `Fix32` column on the character
row and it is not a field. **Recognition** is a sparse relation over pairs
and it is never stored. **The spatial reach of renown** is a field.

One plane for each character is 6.4 GB at 100,000 characters. Reject it.

Materialise a reach plane for the house, not for the character. Cap the
materialised planes at 64, which is 4 MiB at 64 KiB for each plane. Store
the index in the `fame_plane` column, with 255 meaning not materialised.
Rank houses by total renown at the character-tier barrier.

Derive an individual's reach by scaling the house plane value by that
character's share of the house renown. A character with no plane is known
locally only, through a radius test around `seat`.

The cell is a `u8` and the combine is saturating unsigned addition, which is
exactly associative and commutative. Propagate on the existing road and
trade conductance plane, with the same diffusion kernel the influence maps
already run.

Recognition is then derived, not stored:
`recognises(A, B) = reach_of_B[cell_of(A)] >= threshold`.

**The identity and field split holds.** The field framework gains one input
column and one output index. No graph edge, no parent pointer, no title and
no office enters a field.

#### D83. The character tier exposes a per-character object model. The mass tier does not

The selector rule that forbids `__iter__`, `__len__`, `__getitem__` and
`__bool__` stays in force for the `Mass` tier without change.

The `Character` tier additionally exposes `to_list()` and per-character
proxy objects. The selector API remains available on the character tier and
stays the preferred form for anything set-shaped.

**Enforce this with the declared tier on the entity class, not with a
cardinality check at call time.** A cardinality check makes the same script
work on a small world and fail on a large world, which is the worst
available failure mode. `world.units.to_list()` must be an `AttributeError`
in development, with a message naming the tier and the correct method.

The load-time ceiling of D70 is the backstop and runs once, at load.

#### D84. A character proxy is a generational handle. Reads go to the arena. Writes queue commands

A proxy holds a character identifier, a generation and a world reference. It
holds no data.

Every attribute read goes to the Rust arena through one PyO3 call and checks
the generation. A read through a proxy to a dead character raises
`StaleHandleError`.

A proxy write queues a command. It never mutates. Character commands enter
the same sealed queue as selector verbs and receive sequence numbers at the
same barrier, so the character API needs no new determinism machinery.

A proxy expires at the frame barrier. Holding one across a `step()` call
raises on the next read.

Cost, measured per simulated year of 1,200 ticks: 0.17 ms for each tick at
10,000 characters, and 1.7 ms for each tick at 100,000. About 30 percent of
that is the PyO3 boundary and about 70 percent is the Python interpreter.
The boundary is not the binding cost.

#### D85. The character tier runs as two passes: a monthly event pass and a sharded decision pass

**Pass A, the event pass.** Rust. Period 120 ticks, offset 0. It runs
births, deaths, succession, office cascades, house splits and the Euler
label rebuild. It is global and unsharded, because two deaths in one period
can compete for the same title and the resolution must not depend on shard
order. It writes in phases 5, 6 and 7. It costs about 4 ms for each run at
100,000 living characters.

**Pass B, the decision pass.** Python. Period 1 tick. Sharded by
`character_id modulo 1200`, so one shard runs each tick and every character
runs once for each simulated year. Within a shard, deliver characters in
ascending identifier order.

Sharding is mandatory above 10,000 characters. An unsharded yearly Python
pass at 100,000 characters costs 2.0 seconds against a 100 ms frame budget,
which is a 20-times overrun once each simulated year, forever.

A slow tier stays deterministic under three conditions. Its period and
offset are compile-time constants. It reads only values that are stable at
its own barrier, or reads through the event log. It writes only commands and
events. No new mechanism is required.

#### D86. Character randomness keys on the character. A birth draw keys on the mother

Every character draw keys on `(SYSTEM_CHARACTER, tick, character_id,
draw_index)`.

A birth draw keys on `(SYSTEM_CHARACTER, tick, mother_id, birth_sequence)`,
because the child has no identifier when the draw happens.

Allocate new character identifiers in sorted order. Collect the pending
births of a tick, sort them by `(mother_id, birth_sequence)`, then allocate.
Without this rule the identifiers depend on thread completion order, and
every downstream tie-break inherits that dependency.

Sort every graph frontier ascending by entity identifier before it expands,
and deduplicate it. Visit children in ascending identifier order in every
depth-first search, because the Euler labels depend on the visit order.

#### D87. The biography log is a retained structure, separate from the transient event log

The per-tick event arena is transient and is cleared each frame. A
legends-mode history must survive for the life of the world. **Do not reuse
the transient arena for history.**

Pass A walks the sealed event stream at its barrier, selects the retained
events by rule, and appends them to the biography log. The copy is a
filtered `gather`.

Retain an event when its subject held an office or a title, or when its
subject's peak renown exceeded a threshold, or when its subject is an
ancestor of a living character within 6 generations, or when the event kind
is a birth, a death or a title transfer.

Discard every other event and mark its subject as summarised. A summarised
character keeps its 64-byte row, so the genealogy stays intact and every
ancestry query still works. Expect about 5 percent retention.

The log reaches 512 MB after about 5,700 simulated years at 10,000 living
characters, after about 570 years at 100,000, and after about 57 years at
one million. **Retention never needs to run at 10,000. It runs constantly at
one million**, which is a further reason for the ceiling of D70.

Archive rather than delete for a large world. Append discarded events to a
file in the hand-written `bytemuck` save format, with a version field, an
endianness marker and a checksum, and keep only an index in memory.

---

#### D88. A soldier is promoted into the character tier by filter, sort and budget

A unit carries a `deeds: u16` accumulator that only increases. The pass that
updates it also writes a dense eligibility bitset, one bit for each unit,
with one branch-free comparison against a per-type threshold. At one million
units the bitset is 125 KB.

**Scan the bitset only at the character-tier barrier, not each tick.** This is
correct only because `deeds` is monotone, which makes the bit a level and not
an edge. Check the monotonicity in the deeds kernel in debug builds, because
a rule that ever reduces `deeds` breaks the lazy scan silently. The scan
costs about 2 microseconds, twelve times for each simulated year.

Promotion is then the same three kernels as succession. Collect the eligible
set, which a bitset word scan yields in ascending unit identifier order with
no sort. Rank by the key vector `(deeds descending, unit id ascending)`. Take
the first `max(0, target_living - current_living)` entries.

The budget makes the character population self-regulating and bounds it by
the ceiling of D70. At a target of 100,000 living characters and a 60-year
mean lifespan the rate settles at about 1,667 for each year, which is about
139 at each monthly barrier.

**A promoted soldier receives a null lineage and founds a new house.** Reject
invented ancestry on arithmetic: depth 1 costs 120 MB over 500 years and
depth 4 costs 1.8 GB. The consequence, which the owner must confirm, is that
a promoted soldier cannot inherit a title by blood. He can hold an office by
appointment, and his own children inherit from him normally.

The character row gains `unit: u32`. The unit row gains `character: u32` and
`spawn_tick: u32`, which is 8 MB at one million units. One promotion costs
about 2 microseconds, so the whole pass costs about 3.3 ms for each year.

**There is no demotion.** A promoted character never returns to the mass
tier. Disembodiment sets `unit` to `u32::MAX` and despawns the unit; the
character continues. A unit despawn that carries a `character` link sets a
`pending_death` flag in phase 5 at O(1) and emits `EmbodimentEnded`. The next
barrier resolves the death, the succession and the asset transfer, so a
character is dead but unresolved for at most 120 ticks. A character with
`pending_death` set cannot act and cannot be a verb target, so no rule reads
stale state.

**Grafting a null-lineage character into the Euler labels costs O(1).** Label
a virtual super-root rather than a forest, so the structure is always one
tree. Reserve a trailing gap inside the super-root interval and give each new
root 1,024 label slots from it. At 1,667 promotions for each year that
consumes 1.7 million slots of a 4.29 billion slot `u32` space, and the yearly
relabel of D72 reclaims every unused gap. The promotion path therefore adds
no new index maintenance. Set `depth = 0`; the kinship recursion terminates
on a null parent and returns zero with no special case.

Key every promotion draw on `(SYSTEM_CHARACTER, tick, unit_id, draw_index)`.
The new character has no identifier when its name is drawn. Allocate
character identifiers after the sort and after the budget cut, never during
the scan.

#### D89. A formation is an organisational node that owns its soldiers

A formation is a character-tier entity with a `parent` formation, a
`commander` character, a cached strength, a bounding level 1 mask, a kind and
a faction. The row is 24 bytes, so 10,000 formations cost 240 KB.

**Formation membership is ownership. Reuse D76 without change.** Add a
`formation: u32` column to the unit row, at 4 MB for one million units, and
build the reverse index as a counting sort into a compressed sparse row
structure, at 8 MB and about 5 ms for each rebuild.

Reject a per-formation list of unit identifiers. Its size is comparable, and
its failure is maintenance: a despawn invalidates a slot and every list that
holds it must be repaired. A forward column has no such repair.

**An order to a formation is an ordinary set-valued verb.**
`army.move_to(dest)` desugars to `world.units.in_formation(army).move_to(
dest)`. The formation contributes one selector leaf. Resolution is an index
probe against the reverse index, which yields a sorted identifier list at a
cost proportional to the members and not to one million. The pyramid plays no
part, because a formation identifier has high cardinality and needs no
summary.

An order to a parent formation expands the formation tree first, with the
same sorted level-synchronous wavefront that D78 specifies for the office
cascade, then unions the runs. Cap the chain-of-command depth at 6.

The bounding level 1 mask is not needed to find the members. It is needed for
the region scope that the command scheduler uses to run disjoint local
commands in parallel. Refresh it at the barrier as a `map` then a `reduce`
over the members. **The tile capacity of 8 makes the mask genuinely
multi-cell**: a company of 100 soldiers occupies at least 13 tiles, so a
formation order is a multi-region command.

Total cost: about 12 MB and about 6 ms for each barrier, which is 0.06 ms for
each tick.

**The formation row stores no strength and no composition.** The pyramid
already maintains a per-cell unit count, a faction mask and a type histogram,
so army strength in a region is computed state. A cached strength would be a
second source of truth that the dirty-pyramid update does not maintain.

#### D90. Command is explicit membership. A region is a default and a cache, never the authority

Spatial command was evaluated seriously, because it would delete the
`formation` column and its reverse index and save 12 MB. **Reject it.** It
fails on function, in five places.

A detached unit cannot be recalled, because recall is an order and the order
cannot reach outside the region. Two formations of one faction in one region
cannot be separated by any spatial predicate. A garrison and a field army
share tiles and merge. Scattered levies form one army and occupy no common
region. And a region is not stable under movement: if the command is the
region, a movement order changes its own recipient set, which snapshot
selector semantics make safe within one frame and not across frames. **A
command structure must be stable across frames.**

Adopt the hybrid. The `formation` column is authoritative. The level 1 mask
is derived and serves the scheduler. Every aggregate comes from the pyramid.
A formation may declare a `home` region, and a unit spawned there, or
entering it while carrying no formation, joins by default; after that the
unit carries the column and the region has no further authority.

**Spatial by default, explicit thereafter.** That gives spatial grouping for
garrisons and levies and never removes a unit from a command. The hybrid
costs one `u32` on the formation row and one filtered scan over the units
with a null formation column, evaluated at the barrier and not each tick.

A commander points at a formation identifier. Not a level 1 cell, not a set
of cells, and not a stored member list.

#### D91. Appointment is a field of the succession policy, and it produces a claim like any rival law

Add two fields to the policy row that D81 puts on the title:
`appointment`, with the values `Never`, `ByHolder`, `ByLiege` and
`ByEither`, and a `revocable` bit. That is 2 bytes on a structure that
already exists, so appointment is a field of the succession policy and not a
second policy object.

The verb is `grant_title(title, candidate)`. A grant of a vacant title is
ordinary. A grant of a held title is a dispossession and needs the
`revocable` bit; reject it otherwise with `TITLE_NOT_REVOCABLE`.

**An appointment overrides a blood claim for the state and emits a claim
edge.** Write the owner column to the appointee, then run the ordinary
filter, sort and allocate of D79 to find who would have inherited. If that
character differs from the appointee, emit a claim edge with a strength from
the policy. **This is the D81 mechanism reused without change**, because a
rival law and a rival grant produce the same object. Revocation works the
same way and gives the dispossessed holder a claim.

**Ordering.** Process appointments in ascending `(title identifier, command
sequence number)` order. The first appointment for a title wins; reject later
ones with `TITLE_ALREADY_GRANTED`. The sequence number is fixed at the seal
barrier, so no new mechanism is needed.

**The barrier pass order is part of this decision.** Deaths, then succession,
then appointments, then automatic promotions, then index rebuilds.
Succession must precede appointment, or a grant lands on a title whose holder
is about to die. Appointment must precede automatic promotion, because a
grant that names a unit performs its own promotion.

#### D92. Appointment is the second promotion path. It spends the budget first, and the target is soft

A grant may name a mass-tier unit. The grant then causes the promotion; the
soldier does not promote himself first.

Appointments run before automatic promotions, so an appointment always sees
the whole budget `max(0, target_living - current_living)` and the automatic
pass takes what remains.

**When the budget is zero the appointment still succeeds and the population
overshoots the soft target.** The next barrier computes a budget of zero and
promotes nobody, so the population self-corrects within one or two barriers.
Reject an appointment only at the hard ceiling of D70, with the reason
`CHARACTER_TIER_FULL`.

Two alternatives were rejected. Bypassing the budget removes the
self-regulating bound that keeps the Python cost of D70 inside its budget.
Sharing one budget makes a player action fail because an automatic pass
earlier in the same barrier spent it, which is a failure the player cannot
see. **The target is a control input; the ceiling is an invariant.**

**Finding a favoured unit is a selector, and that is the only option**, not
an optimisation: a unit has no opinion edges, because opinion is a
character-tier relation. Resolve
`units.in_formation(lord.command) & (f.deeds > threshold)` and rank on
`(deeds descending, service_ticks descending, unit identifier ascending)`.
Length of service comes free from the `spawn_tick` column that D88 adds.
Scope by the formation subtree or the bounding mask of D89, so the cost is
proportional to the command and not to one million.

#### D93. Opinion is a directed sparse edge with a hard out-degree cap and four axes

A dense opinion matrix is 68 billion ordered pairs at the ceiling. Reject it.

**Sparse opinion is quadratic only if the degree grows with the population.
A hard out-degree cap `K` makes storage exactly `N x K x edge_bytes`, which
is linear.** The cap is the mechanism, and it must be enforced rather than
expected.

The edge is 16 bytes: `other: u32`, `last_tick: u32`, and four `i16` axes for
respect, affection, fear and obligation. Opinion is directed; store outgoing
edges and build a 4-byte reverse index when an incoming query is needed.

**Take exactly four axes.** The key and the timestamp cost 8 bytes before any
axis exists, so one axis pads to 16 bytes and four axes occupy exactly 16.
The second, third and fourth axes are free. A fifth costs 8 more bytes for
each edge, which is a real 50 percent increase.

Set `K = 32`. Storage is then 640 bytes for each living character including
the reverse index: 6.4 MB at 10,000, 64 MB at 100,000 and 168 MB at 262,144.

**The default opinion is derived and never stored.** Sum integer-weighted
terms over diplomatic relation, house, faith, culture, the coefficient of
relationship from D73, trait compatibility on interned trait sets, and a
renown band. Memoise it on a configuration key; the memo table is probed and
never iterated.

#### D94. Opinion decays toward its derived default, lazily, so it has no recurring cost

**Decay targets the derived default, not zero.** This is a correctness
property and not only a fidelity choice: an edge that has decayed to its
default carries no information and can be deleted with no loss. **Decay to
default makes pruning correctness-preserving**, which removes the whole
question of what to prune.

**Apply decay at read, not in a pass.** Store `last_tick` and evaluate
`default + (stored - default) * decay ^ elapsed` with an integer lookup table
over elapsed months. A periodic pass over 3.2 million edges would cost 38 ms
for each year at 100,000 characters and would be the largest single line in
the Rust character budget. Lazy decay removes it.

**Prune only on cap overflow.** Evict the least salient edge, where salience
is the largest absolute difference between any axis and its default, ordered
by `(salience ascending, last_tick ascending, other identifier ascending)`.
There is no periodic decay pass and no periodic prune pass, so the recurring
cost of opinion is zero.

**Drop every opinion edge at death, in both directions.** Retaining them
costs 8.3 times the table after 500 years, which is 531 MB at 100,000 living
characters and 1.39 GB at the ceiling. Nothing reads them. The biography log
of D87 already retains the interaction that created the edge.

**Determinism.** Accumulate opinion deltas by sort and segmented reduce,
never by atomics; integer addition is exactly commutative and associative.
Store each edge run sorted ascending by the `other` identifier, so iteration
order comes from the data and a probe is a binary search over at most 32
entries. Never iterate a hash container.

**The consequence for the target population.** With opinion included a living
character costs 840 bytes, not 200. The whole character layer, including 500
years of dead characters and the retained biography log, is 34 MB at a target
of 20,000, 86 MB at 50,000, 173 MB at 100,000 and 453 MB at the ceiling.
**Recommend a target of 20,000 to 50,000 living characters.** At 50,000 the
Python pass costs 0.85 ms for each tick, below trade at 1.1 ms, and the layer
is a third of the tile side. The ceiling of D70 stays as the hard invariant,
and the gap below it is deliberate.

#### D95. Record the Nemesis patent as a design constraint

US 10,926,179 B2, assigned to Warner Bros. Entertainment Inc., has an
earliest priority date of 26 March 2015, a filing date of 25 March 2016, a
grant date of 23 February 2021, and an adjusted anticipated expiration of
11 August 2036. It has 36 claims, of which claims 1, 6 and 9 are
independent.[^13]

Every independent claim recites the same combination: game events involving
**an avatar operated in response to input from a player** and **a first
non-player character controlled to respond to and automatically oppose
avatars**; detecting an interaction between the avatar and that character;
**changing the parameters of a second** such non-player character based on
that detection; and outputting an indication of the change. Dependent claims
add a ranked faction of non-player characters, export of the faction to
another player's game instance, and power-centre attributes.

**The current design presents neither of the two recited elements.**
Promotion is driven by a monotone achievement accumulator and by a lord's
grant. Neither refers to a player avatar. The broad pattern of promoting an
anonymous mass entity into a named tracked individual is much older prior
art; Dwarf Fortress generated named historical figures with recorded deeds
before the 2015 priority date.

**If the design later adds a player avatar together with characters that rise
by defeating that avatar and a ranked faction that reorders as a result,
obtain legal counsel before building it.**

**This decision records published claim text. It is not legal advice and it
is not an assessment of infringement.** Claim scope is decided by claim
construction and prosecution history, and this report examined neither.

---

## References

[^1]: ADR-0001, Foundational Architecture, decisions D1, D4, D5, D9, D17, D20, D21, D22, D28, D29, D30, D31, and the byte and per-tick budget tables. `docs/adrs/REGISTRY.md`
[^2]: ADR-0001 merge notes for the background reports, sections 4, 8 and 9, including the running per-tick budget table. `docs/research/reports/MERGE-NOTES.md`
[^3]: Bender, M. A., and Farach-Colton, M. (2000). "The LCA Problem Revisited." LATIN 2000: Theoretical Informatics, Lecture Notes in Computer Science volume 1776, pages 88 to 94. https://doi.org/10.1007/10719839_9
[^4]: Tarjan, R. E., and van Leeuwen, J. (1984). "Worst-case Analysis of Set Union Algorithms." Journal of the ACM, volume 31, issue 2, pages 245 to 281. https://doi.org/10.1145/62.2160
[^5]: Henderson, C. R. (1976). "A Simple Method for Computing the Inverse of a Numerator Relationship Matrix Used in Prediction of Breeding Values." Biometrics, volume 32, issue 1, pages 69 to 83. https://doi.org/10.2307/2529339
[^6]: Karigl, G. (1981). "A recursive algorithm for the calculation of identity coefficients." Annals of Human Genetics, volume 45, issue 3, pages 299 to 305. https://doi.org/10.1111/j.1469-1809.1981.tb00341.x
[^7]: Meuwissen, T. H. E., and Luo, Z. (1992). "Computing inbreeding coefficients in large populations." Genetics Selection Evolution, volume 24, issue 4, pages 305 to 313. https://doi.org/10.1186/1297-9686-24-4-305
[^8]: Wright, S. (1922). "Coefficients of Inbreeding and Relationship." The American Naturalist, volume 56, number 645, pages 330 to 338. https://doi.org/10.1086/279872
[^9]: ADR-0001 background report 04, Selector Engine, Verb Vocabulary, and Data-Driven Types, sections 1.4, 3, 6.3 and 8.3. `docs/research/reports/04-selector-engine-and-verbs.md`
[^10]: Crusader Kings 3 community wiki, Succession laws. Documented game behaviour only. No published implementation detail exists, so any claim about how the game stores or evaluates a succession law is unverified. https://ck3.paradoxwikis.com/Succession_laws
[^11]: Victoria 3 community wiki, Interest group and Pops. Documented game behaviour and design only. No implementation detail and no character-count figure is published. https://vic3.paradoxwikis.com/Interest_group
[^12]: Dwarf Fortress community wiki, Advanced world generation and Legends. The option to remove unimportant dead historical figures after generation, and the report of history dumps up to one gigabyte, are documented there. No implementation detail is published. https://dwarffortresswiki.org/index.php/Advanced_world_generation
[^13]: United States Patent 10,926,179 B2, "Nemesis characters, nemesis forts, social vendettas and followers in computer games." Warner Bros. Entertainment Inc. Earliest priority 2015-03-26, filed 2016-03-25, granted 2021-02-23, adjusted anticipated expiration 2036-08-11. 36 claims; claims 1, 6 and 9 are independent and were read in full for section 12. This is the one citable implementation description in the game survey, and it describes a claimed method rather than shipped code. https://patents.google.com/patent/US10926179B2/en
[^14]: RimWorld modding documentation, XML definition files and inheritance. Documented authoring format only. No implementation detail for the relationship system is published. https://rimworldwiki.com/wiki/Modding_Tutorials/XML_Defs
[^15]: ADR-0001 background report 09, Influence Maps, findings 2, 3, 4 and 5 on per-faction plane cost, the `u8` saturating cell, and level 1 as the correct resolution. `docs/research/reports/09-influence-maps.md`
[^16]: ADR-0001 background report 12, Entity Economy and Modifiers, section 8, threshold crossings by a dense bitset and a sparse ascending scan. That report's decision numbers are subject to renumbering at merge, so this note cites the section. `docs/research/reports/12-entity-economy-and-modifiers.md`
