# Review 0029: The storage records and the tile index record

## What was reviewed

Four draft decision records.

| Record | Path | Implementation state |
|---|---|---|
| ADR-0017 | `docs/adrs/accepted/adr-0017-the-world-is-a-rhombus-so-a-tile-index-is-raw-axial.md` | Implemented by `crates/cachette-core/src/hex.rs` and `crates/cachette-core/src/world.rs` |
| ADR-0012 | `docs/adrs/accepted/adr-0012-tiles-are-dense-columns-and-units-are-a-generational-arena.md` | Not implemented |
| ADR-0014 | `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md` | Partly implemented: `Entity` in `crates/cachette-core/src/types.rs` |
| ADR-0018 | `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md` | Not implemented |

**The records moved while this review ran.** The review began at commit
`d19c872`. During it, commit `c512bb4`, `Apply the review amendments to
ADR-0012, ADR-0014 and ADR-0017`, rewrote parts of three records, and ADR-0018
changed in the working tree without a commit.

Every finding below is stated against the working tree at commit `c512bb4`,
with ADR-0018 as it stands uncommitted. Where an amendment this reviewer
raised has already landed by another route, the finding says so and states
whether the landed text is sound. Nothing below is a repeat of an amendment
that is already in the tree.

The source files did not change during the review.

## Who reviewed

An agent that wrote none of the four records and none of the source files. The
delegated-review conditions in the registry therefore hold: the reviewer read
the records and the rules, not the reasoning that produced them.

## Verdicts

| Record | Verdict |
|---|---|
| ADR-0017 | ACCEPT WITH AMENDMENT |
| ADR-0012 | ACCEPT WITH AMENDMENT |
| ADR-0014 | ACCEPT WITH AMENDMENT |
| ADR-0018 | ACCEPT WITH AMENDMENT |

No record is rejected. Each states a real constraint and each passes the
three-condition test. Eleven amendments remain outstanding. Four of them
repair reasoning that is false as written, and two repair a value that two
records now claim independently.

**ADR-0018 must not be accepted before its conflict with ADR-0056 is
settled.** That is the one finding here that another record has to move for.

---

## ADR-0017: The world is a rhombus, so a tile index is raw axial

### A. The three-condition test

**Passes all three.**

1. *Could a contributor choose otherwise?* Yes. An offset index is the common
   choice, and the record says why a project reaches for one.
2. *Does choosing otherwise cost more than changing it later?* Yes. Every
   storage decision above the index inherits the address, and the world shape
   is not a runtime parameter.
3. *Is the reasoning invisible in the artefact?* Yes. `Grid::index_of` shows
   the arithmetic. It cannot show why no offset conversion exists, and the
   absence of a function is what code cannot document.

### B. One claim, and length

780 words, against a reference median near 1300. The shortest of the four and
well inside the healthy range.

The title states a claim. D2, D3 and D4 all follow from D1 rather than
standing apart, so this is one rejectable claim.

For the registry: the scope rule's standing warning that `the Cachette drafts
are all above both reference medians`, quoting 1777 to 4570 words, does not
describe this batch. All four sit below the median. The warning is stale here.

### C. Forbidden material

One item outstanding.

> The report measures the aspect ratios of both.

The word `measures` is the exact word this project polices. BLK-007 states
that no measurement exists on the target platform, and the definition of done
forbids claiming a measurement that was not taken. An aspect ratio is derived
from the block geometry, not measured.

**Amendment 17-1 (mandatory, outstanding).** Replace that sentence with
`The report derives the aspect ratio of each.`

One item examined and cleared. `the shear wastes about half of it` reads as a
percentage, which section 4.1 bans. It survives: it is a property of a
parallelogram bounding a rectangle, which is the class the scope rule exempts
as a structural constant. It is not a budget and no measurement can change it.

No version pin, no file table, no count, no module arrangement.

### D. Does the record match the code?

Checked decision by decision against `hex.rs` and `world.rs`.

**D1 — honoured, and the code is now *more* specific than the record.** The
current D1 says only that the index derives from the address by arithmetic and
explicitly declines to state the index function. `Grid::index_of` computes
`(r * width) + q` and `Grid::address_of` is the matching remainder and
division. Row-major is one arithmetic derivation from the address, so the code
is inside what D1 permits. This is the right relationship: the record
constrains, the code chooses.

**D1's absence claim — verified.** I searched the whole of `crates/` and
`python/` for `offset`, `axial_to`, `to_offset`, `screen` and `pixel`. Every
hit on `offset` is a loop variable, one of the six neighbour offsets, or the
FNV offset basis in `hash.rs`. **No axial-to-offset conversion function exists
anywhere in the tree.** The claim is true at this commit.

**D2 — honoured.** `NEIGHBOURS` is six constant `Axial` values in a fixed
order. `Grid::neighbour` returns `None` outside the grid, so the world does
not wrap. `Grid::neighbours` returns a fixed six-long array, so the direction
index is stable whatever the position.

**D3 — honoured, with a wording gap.** Every coordinate is `i32` or `u32`.
`distance` widens to `i64` before it subtracts. No coordinate is `Fix32` and
none is a float.

The gap: `Axial::add` and `Axial::s` saturate. Saturation is not exact
arithmetic. I attempted this as an objection to D3 and it half fails —
saturation is deterministic and D3's subject is the number system — but the
word `exact` is doing real work elsewhere in this project, and a reader is
entitled to know the overflow policy is clamping rather than wrapping or
panicking.

**Amendment 17-2 (recommended, outstanding).** After `so the arithmetic that
derives one is exact.[^3]` add:

> An address operation saturates at the range of its type. A saturated address
> lies outside the world, so it has no index and no tile.

**D4 — verified.** The engine holds no screen position. The only occurrence of
`screen` in the crate is a doc comment on `World::grid` stating that the
viewer needs the grid *because* the engine holds no screen position. There is
no skew, no projection, and no float in the geometry module. The claim is true
at this commit.

### E. Reasoning errors

**The linearisation collision is repaired, and the repair is sound.**

At `d19c872`, D1 read `The index is the row multiplied by the row length, plus
the column`. That is row-major, and ADR-0016 is titled `Tiles are stored in
block-tiled order at the aggregation block size`. One value — the map from an
address to an index — was declared in two records with nothing to fail when
the copies disagreed. That is the first recurring defect shape, and it is the
shape this project has a recorded local instance of.

The current D1 removes the index function and adds:

> This record does not state the index function. The order in which tiles sit
> in memory is a separate claim, and the record that holds it may choose a
> block order rather than a row order.[^4] Both derive an index from the same
> axial address by arithmetic, which is what this record constrains.

This is correct, and it is the right cut. It keeps the claim the record exists
to make — no conversion, no offset index — and it stops the record from
deciding a second thing that ADR-0016 already claims. The footnote to ADR-0016
is present. No further amendment.

Every remaining `because` traces:

- *Offset coordinates do not support vector arithmetic because the neighbour
  offsets change from one row to the next.* Correct and standard.
- *A wrapping world would make the index a ring and every distance need a
  shortest-arc rule.* Correct.
- *A conservative bounding radius around a longer, thinner block admits more
  false positives.* Correct. A parallelogram has a larger circumradius than a
  square of the same area.
- *A projection that suits one display does not suit another.* Correct, and it
  is a real reason rather than a restatement of D4.

### F. Blocker hygiene

Clean. The record cites BLK-014 as the resolved blocker that fixed the shape,
and BLK-014 is `Resolved`. It calls no closed blocker open and states no value
BLK-007 governs. The `about half` figure is geometric, not a platform
measurement.

### G. What is missing

The registry dependency now points the wrong way. Row 0017 lists 0016 as a
dependency, but after the D1 amendment it is ADR-0016 that depends on
ADR-0017: 0016 chooses an index function over the address that 0017 fixes.
ADR-0017 is `Draft` with a file and an implementation; ADR-0016 is `Proposed`
with no file. I do not touch the registry. This is for whoever does.

### H. Verdict: ACCEPT WITH AMENDMENT

17-1 mandatory, 17-2 recommended. The acceptance may state that the record is
implemented, which is the strongest position of the four.

**Objections attempted, and why each failed:**

1. *The record restates a blocker resolution, so BLK-014 is the record.*
   Fails. BLK-014 records the answer. It records neither the forces, nor the
   rejected offset index, nor the consequence that the viewer now owns the
   skew. A register row cannot carry the reasoning that lets a reviewer reject
   a future offset conversion.
2. *D4 is a scope statement, so it belongs in the consequences.* Fails. A
   contributor could reasonably put the skew in the engine, and D4 forbids it.
   It states a constraint a reviewer can check, and I checked it against the
   tree.
3. *D3 duplicates ADR-0002 and adds nothing.* Fails narrowly. ADR-0002 forbids
   floating point. D3 is stronger and geometry-specific: a coordinate is not
   fixed-point either. Without D3 a contributor could store an address in
   `Fix32`, honour ADR-0002, and break the index.
4. *`about half` is a forbidden figure.* Fails. See section C.
5. *The record names a module arrangement.* Fails. It names no module. It
   states where a conversion may not appear, which constrains every
   arrangement rather than fixing one.
6. *D1 is now so weak it constrains nothing, since it declines to state the
   index function.* Fails. It still forbids the offset index, forbids a
   conversion in a tile access path, and fixes the world shape. Those are
   three checkable prohibitions, and one of them is checkable by grep.

---

## ADR-0012: Tiles are dense columns and units are a generational arena

### A. The three-condition test

**Passes all three.**

1. *Could a contributor choose otherwise?* Yes, and the record cites engines
   that do. Making a tile an entity is a mainstream design.
2. *Does choosing otherwise cost more than changing it later?* Yes, and the
   record's own argument is right: every system reads a tile by index and a
   unit by identity, so merging them afterwards rewrites every system.
3. *Is the reasoning invisible in the artefact?* Yes, doubly. The entity
   storage does not exist. There is nothing to read.

### B. One claim, and length

884 words, well below the reference median.

The title carries two claims joined by `and`. I attempted to split the record
on that basis and the objection failed: the two halves are one decision seen
from two sides. A reviewer cannot accept the tile half and reject the unit
half, because the reason a tile is not an entity is that a unit is. D1, D2 and
D3 are three faces of one rejectable claim.

### C. Forbidden material

Clean of figures. No percentage, no byte budget, no version, no count, no file
table.

Two items sit near the line and both survive.

> The location table would grow with the tile count, which is the largest
> count in the project.

A comparison of counts with no number. It is the argument itself; removing it
removes the decision. It stays.

> The entity arena is chunked for each shape

An arrangement, but it is ADR-0066's arrangement, cited rather than restated,
and used only to state the consequence for the zero-copy path. It stays.

### D. Does the record match the code?

**Nothing implements it.** `World` in `world.rs` holds two tile columns as
separate `Vec` values, which is consistent with D2, but the record's subject —
entity storage — has no code at all. The registry's review condition requires
the acceptance to say plainly that nothing implements this record yet.

The current D1 also gained this text at `c512bb4`:

> The derivation is arithmetic whichever order the tiles sit in. A block order
> adds a shift and a mask; it adds no lookup.[^7]

This is sound and it is the correct pairing with the ADR-0017 D1 repair. Both
records now say the same thing about the same value — that the address-to-
position map is arithmetic — and neither one fixes which arithmetic. No
amendment.

### E. Reasoning errors

**Error 1: D3 conflates `a unit` with `an entity`, and the two are not the
same set. Outstanding.**

D3 is headed `A unit is an entity in the generational arena`, then says `The
arena holds every one of the four fixed shapes`. Two of those four are not
units in any ordinary sense: a living character carries no tile position, and
a tile upgrade is not mobile. The record's own title says `units are a
generational arena`.

This is not pedantry. ADR-0018 has to spend a consequence undoing the
confusion, explaining that the bridge covers the soldier only. The
documentation rule requires one word for one meaning, and this record uses
`unit` for two sets in three consecutive sentences.

**Amendment 12-1 (mandatory).** Retitle D3 to `An entity of any shape lives in
the generational arena`, and replace its first paragraph with:

> Every entity lives in the entity storage, whatever its shape. An entity
> carries an identity that pairs a slot index with a generation.[^5] Its
> columns are the columns of its shape.[^2] A unit is one such entity, and the
> word `unit` in this record means an entity of the soldier shape.

**Error 2: D3's marker claim is incomplete, and the word `only` makes it
false. Outstanding.**

> A tile upgrade is an entity in the arena, and the tile side of the split
> holds only the marker that says which tiles carry one.

A marker answers *whether* a tile carries an upgrade. It does not answer
*which* upgrade. Something must map a marked tile to the identity of the
upgrade entity, and no record in this batch holds that structure:

- ADR-0018 excludes the tile upgrade from the bridge.
- ADR-0018 justifies the exclusion by saying `A settlement and a tile upgrade
  are fixed to a tile, so their tile field is already the answer`. **That is a
  non-sequitur.** The tile field is the forward map, from the entity to its
  tile. The reader of the marker needs the reverse map, from the tile to the
  entity — the exact direction the bridge record exists to supply.
- ADR-0015 is cited for the sparse form, and ADR-0015 has no file.

The word `only` is what turns an omission into an error. As written, D3
asserts that a bit is sufficient. It is not.

**Amendment 12-2 (mandatory).** Replace that sentence with:

> A tile upgrade is an entity in the arena. The tile side of the split holds a
> marker that says which tiles carry one. The marker answers whether, not
> which. A structure that maps a marked tile to the identity of its upgrade is
> a separate decision, and this record does not hold it.[^4]

**Error 3, minor, no amendment.** The rejected reverse alternative says a
dense column indexed by tile `would give a unit an address instead of an
identity`, so `every reference to it would have to move with it`. The
inference is right but it silently assumes references exist across the frame
barrier, which is ADR-0014's premise. It survives because ADR-0014 is cited
two paragraphs earlier.

Everything else traces. `A tile needs no identity, because its address is
stable and total` is the correct reason, and `total` is load-bearing in it.

### F. Blocker hygiene

Clean. Cites no blocker, states no value BLK-007 governs, calls nothing open.

### G. What is missing

**The reverse map from a tile to a fixed entity on it** — the settlement and
the tile upgrade. ADR-0012 D3 needs it, ADR-0018 explicitly declines it with a
bad reason, and ADR-0015 has no file. No record holds this, and each of the
two records assumes the other does. Amendment 12-2 at least makes the gap
visible instead of asserting it away.

**The batch leans on unwritten records.** Four of this record's seven
footnotes point at records with no file: ADR-0015, ADR-0016, and through
ADR-0018 the chain to ADR-0020. Each cites the registry rather than a missing
path, which is the honest form and satisfies the check. The count is high
enough to state.

### H. Verdict: ACCEPT WITH AMENDMENT

12-1 and 12-2 mandatory. The acceptance must state that nothing implements
this record.

**Objections attempted, and why each failed:**

1. *This should be two records, one for tiles and one for units.* Fails. See
   section B. The halves are not separately rejectable.
2. *Condition 3 fails, because the reasoning becomes visible in the code once
   the arena exists.* Fails. The code will show that tiles are arrays and
   units are an arena. It cannot show that making tiles entities was
   considered and refused, and that refusal is what a reviewer needs in order
   to reject a later patch that adds a tile component.
3. *D2 records an arrangement — structure-of-arrays is a layout, and section
   4.4 forbids recording a layout.* Fails. Section 4.4 forbids recording
   *where code lives*. A layout that determines the memory cost of every
   whole-world pass is the constraint itself, and D2 explicitly refuses to fix
   the column widths, which is where the volatile part sits.
4. *The record duplicates ADR-0066, which already assumes the split.* Fails,
   and the relationship runs the right way. ADR-0066 says it assumes the split
   and does not state it. Without ADR-0012 the split is an assumption with no
   record, which is the failure mode the definition of done names.
5. *D1's new block-order paragraph re-opens the ADR-0017 collision from the
   other side.* Fails. It states that the derivation is arithmetic in either
   order and cites ADR-0016 for the choice. It claims the property, not the
   value.

---

## ADR-0014: Entity identity is an index plus a generation

### A. The three-condition test

**Passes all three.**

1. *Could a contributor choose otherwise?* Yes. A bare index is the obvious
   cheaper choice, and the record's whole argument addresses a contributor who
   reaches for it.
2. *Does choosing otherwise cost more than changing it later?* Yes. Widening
   an identity after the fact changes every structure that stores one.
3. *Is the reasoning invisible in the artefact?* Yes for D3, D4, D5 and D6.
   The doc comment on `Entity` states that the generation makes a stale
   reference detectable, so part of D1 is visible. Nothing in the code can
   show why the generation advances on the free rather than the allocation, or
   why the free list is first-in first-out.

The scope rule's counter-test applies directly: this record governs
determinism, so it needs a record even where it looks obvious.

### B. One claim, and length

1327 words, still just above this batch but around the reference median. The
record grew by 208 words at `c512bb4` when D6 was added.

Seven decisions is a lot for one claim, and I tested whether they separate.
They do not. D2 through D6 are all forced by D1: once an identity is an index
plus a generation, the failure mode of resolution, the moment the generation
advances, the reuse order, the exhaustion rule and the zero case are the same
decision worked through. D7 is the weakest attachment — a dense table against
a hash map is arguably its own claim — but it is four sentences, it is
determinism-governed, and splitting it would produce a record no one reads
alone.

Watch the growth. This record has taken one new decision in one commit. If it
takes a second, split D7 out.

### C. Forbidden material

Clean, and notably careful. The record says `The generation has a finite
range` and never names the width. That is right: the width is a storage figure
that belongs in the type and the reference tables, and naming it here would
violate section 4.1.

D6 states that a generation starts at one. That is a structural constant of
the identity encoding, not a budget, and no measurement can change it. It
stays.

No version, no count, no file table, no module arrangement.

### D. Does the record match the code?

`Entity` in `crates/cachette-core/src/types.rs`, checked against D1 and D6.

**D6 — landed during this review, and it is sound.** At `d19c872` the code had
a constraint that no record held: `Entity::new` returns `None` when the index
and the generation are both zero, because the inner type is `NonZeroU64`, so
the pair (slot 0, generation 0) is not representable. Something had to be true
about the allocator and no record said which. The new D6 records it.

The reasoning in D6 is correct on every step, including the part that is easy
to get wrong:

> The failure is silent in the worst way. It appears once, at the first
> allocation, and only for one slot. Every test that allocates a second entity
> first would pass.

That is true and it is the right justification. D6 also rejects the
alternative — forbidding slot zero — for the right reason: the rule would live
in each future allocator rather than once in the identity. And the closing
line, `Generation zero therefore means that a slot has never been used`, turns
the constraint into a usable invariant. No amendment. This is the strongest
decision added to the batch.

**D1 sentence 3 — still contradicted by the code. Outstanding.**

> A caller reads the two parts through accessors and never constructs an
> identity from parts.

`Entity::new(index: u32, generation: u32) -> Option<Self>` is public. Any
caller can construct an identity from parts. The sentence is also
unachievable as a universal: the arena *must* construct an identity from
parts, because that is what allocation does. The record forbids something that
has to happen, and the code offers it to everyone.

**Amendment 14-1 (mandatory).** Replace the third paragraph of D1 with:

> The identity is one opaque value. The engine never stores the index and the
> generation as two separate fields that a caller can set apart.
>
> The entity storage is the only thing that builds an identity. Every other
> caller receives one, reads its parts through accessors, and passes it on. A
> caller that assembles an identity from an index it chose has defeated the
> generation, because the index it chose came from somewhere that could not
> have known the generation.

**One fact about the code that still no record holds.** The layout puts the
generation in the high 32 bits and the index in the low, so the natural
integer order of an identity is generation-major, not slot-major. `to_bits`
returns the whole word and its doc comment says `The sort key uses this`. This
collides with ADR-0018 D3. See ADR-0018 error 3; the amendment is filed there,
because that is the record that makes the false claim.

D2 through D5 and D7 have no code to check. The arena does not exist.

### E. Reasoning errors

**The D4 repair is sound.** I read the current D4 as asked. Its argument is:

- D3 already makes a stale identity fail, whichever slot is allocated next.
  **Correct.** The generation advances at the free, so the dead entity's
  identity no longer matches its own slot, and it never matched any other.
- Therefore last-in first-out is safe for correctness. **Correct**, and the
  record says so plainly instead of claiming a safety benefit it does not
  have. This is the repair, and it holds.
- The reason for first-in first-out is the generation range, not correctness.
  **Correct.** Under last-in first-out one hot slot absorbs every increment
  while its neighbours absorb none, so it reaches the end of its range while
  the arena as a whole has spare range everywhere. First-in first-out spreads
  the increments.

I tried to break this and could not. The repair is sound and the record is now
honest about what its own rule buys.

One residual, no amendment: the argument has force only through D5, because
without retirement, exhaustion is a wraparound rather than a leak. D4 does
cite D5 through footnote 8, which is a self-citation to this record. Unusual,
but it satisfies the documentation rule and keeps the dependency visible.

**D3, D4, D5 and D6 as a system: sound, with one hole.**

- D3 makes a stale identity fail at the death rather than at the next claim.
- D4 spreads the wear so D5 fires as late as possible.
- D5 removes the wraparound case rather than handling it.
- D6 removes the unrepresentable case rather than working around it.

The four compose, and each does work the others do not. D5 and D6 are the same
move applied at the two ends of the generation range, which is why they read
as a pair.

The hole: no decision says what a resolution attempt sees *between* the free
and the barrier at which the slot becomes available. D4's last paragraph says
the slot becomes available at the barrier; during the frame the slot holds a
dead entity whose generation already advanced, so resolution fails, which is
right. But the location table entry is in an intermediate state that no
decision names, and ADR-0020, which owns the tombstone, has no file. This is
a gap, not an error. Section G.

**Error 1: a false implication in the consequences. Outstanding.**

> The location table therefore keeps an entry for every slot ever allocated,
> and it grows with the high water mark of the population rather than with the
> live population.

The first half does not imply the second, and the first half is stronger than
the truth. The table is indexed by slot, and D4 recycles slots, so it does not
keep an entry for every entity ever allocated. It keeps one entry per *slot*,
and the slot count is the high water mark of the live population plus the
retired slots. As written, the sentence invites a reader to think the table
grows without bound with allocation history — the opposite of what the free
list achieves.

**Amendment 14-2 (mandatory).** Replace that sentence with:

> The location table therefore holds one entry for each slot the arena has
> ever opened. That count is the high water mark of the live population plus
> the retired slots, and it never falls.

**Error 2: the record calls the same leak bounded and unbounded. Outstanding.**

D5 says `The project trades a bounded leak for the removal of the case.` The
consequences say `**The retirement rule leaks, and the leak is unbounded in
principle.**` Both describe the retirement leak.

D5 is closer to right. The leak is bounded by the slot count, which is bounded
by the high water mark plus the retirements. What is unbounded is the *rate*
at which a pathological workload consumes slots, not the leak. Two sentences
in one record calling one thing bounded and unbounded is the internal
disagreement a reader resolves by picking one at random.

**Amendment 14-3 (mandatory).** Replace the heading and first sentence of that
consequence with:

> **The retirement rule leaks, and a hostile workload can grow the leak.** A
> workload that recycles one slot without pause retires it eventually, and
> then retires its successor.

**What else traces.** The rejection of the globally unique identifier is
correct and names the real cost — the hash on the hot path, not the width. D7's
argument against a hash map correctly names the determinism cost as well as
the speed cost, and the determinism cost is the one that binds.

### F. Blocker hygiene

Clean. The record cites no blocker. Crucially it names no generation width,
which is the value a storage measurement would govern, and no cost figure. It
calls no closed blocker open.

### G. What is missing

1. **The state of a slot between the free and the barrier.** ADR-0020 owns the
   tombstone and has no file. D4's last paragraph depends on it.
2. **The integer order of an identity.** The layout decides it, ADR-0018 D3
   depends on it, and no record states it. Filed against ADR-0018.
3. The former gap — the first generation value — is closed by the new D6.

### H. Verdict: ACCEPT WITH AMENDMENT

14-1, 14-2 and 14-3 mandatory. The acceptance must state that only D1 and D6
have an implementation, and that D2 through D5 and D7 have none.

**Objections attempted, and why each failed:**

1. *D7 is a separate claim and should be its own record.* Fails on the length
   evidence rather than on principle. The record is near the reference median,
   D7 is four sentences, and the cost of a separate record — a registry row, a
   footnote chain, a document that must stand alone — exceeds the instability
   that keeping it here can plausibly cause. Revisit if the record takes an
   eighth decision.
2. *D5 fails condition 1, because retirement is the only workable option.*
   Fails. Wraparound-and-hope is a real choice that real arenas make, and a
   contributor under memory pressure reaches for it. The record has to exist
   in order to refuse that patch.
3. *D6 is an implementation detail of the packing, not a decision.* Fails, and
   this is the objection I most wanted to hold. D6 constrains the allocator,
   not the encoding: it says the first generation is one. An engine could
   instead reserve slot zero, and D6 rejects that alternative explicitly and
   for a stated reason. That is a decision with a rejected alternative.
4. *D6 states a value, and section 4.5 forbids inventing a value a blocker
   governs.* Fails. No blocker governs it. The value follows from the identity
   encoding, which the code already fixes, and D6 derives it rather than
   choosing it.
5. *The record states a storage figure by implication, because a finite
   generation range implies a width.* Fails. It states that the range is
   finite and refuses to name it, which is what section 4.5 asks for when the
   value sits elsewhere.
6. *D4 is an optimisation, not a constraint.* Fails. D4 changes when D5 fires,
   and D5 leaks. A reuse order that determines when the engine begins leaking
   slots is a constraint.
7. *The `test can prove that a stale identity fails` consequence is a test
   plan, and a test plan is not a decision.* Fails. It states the falsifier
   for D2, D3 and D4 together, which is what makes the record checkable rather
   than aspirational. The testing rule requires a determinism claim to have a
   proven failure mode.

---

## ADR-0018: The unit-to-tile bridge is derived, and it rebuilds at the barrier

Reviewed as it stands uncommitted in the working tree. The title changed
during the review; the file name did not.

### A. The three-condition test

**Passes, but condition 2 is the weakest in the batch.**

1. *Could a contributor choose otherwise?* Yes, emphatically. The compressed
   sparse row offset array is the textbook structure for this problem, and the
   record says so.
2. *Does choosing otherwise cost more than changing it later?* Weakly yes. The
   bridge is derived state, rebuilt every frame from the unit columns, so
   replacing it changes no persistent format. What saves the condition is the
   last line of the consequences: the bridge shares its partition with the
   aggregation block of the pyramid, so changing the bridge's block form drags
   the pyramid with it. **That coupling is the whole answer to condition 2,
   and it appears in the final sentence of the record**, where a contributor
   deciding whether to change the bridge will not find it.

**Amendment 18-1 (mandatory).** Move the partition coupling into the context,
as its own paragraph, before `The reverse map is derived.`:

> The bridge partitions the world by the same block that the level of detail
> pyramid aggregates over.[^9] One partition serves both. A change to the
> bridge's block form is therefore a change to the pyramid, which is what
> makes this structure expensive to replace and worth recording.

3. *Is the reasoning invisible in the artefact?* Yes. Nothing is implemented.

### B. One claim, and length

1291 words, below the reference median.

**The title repair is correct, and it is incomplete in the tree.** The old
title, `The bridge is three structures, and units stay sorted by tile`, named
an arrangement and a count, which sections 4.3 and 4.4 both reach. The new
title, `The unit-to-tile bridge is derived, and it rebuilds at the barrier`,
names the claim: the bridge is rebuilt whole by a sort and never updated
incrementally. That is what a contributor would choose otherwise on, and what
determinism depends on. Good repair.

Three things did not follow the title:

- The file name still reads
  `adr-0018-the-unit-to-tile-bridge-is-three-structures-and-units-stay-sorted-by-tile.md`.
- ADR-0012 footnote 6 and ADR-0014 footnote 2 both still cite `the unit-to-tile
  bridge is three structures, and units stay sorted by tile`.
- Two consequences still say `three structures`, and D1 now lists **four**
  arrays: a key array, a unit array, a block range array, and a block
  occupancy bitplane.

That last one is the point of the scope rule's section 3 landing in real time.
The record was titled with an arrangement, the arrangement gained a member,
and the count is now wrong in three places. The number never changes when a
title changes, but everything else must.

**Amendment 18-2 (mandatory).** Rename the file to match the new title. Update
the footnote text in ADR-0012 and ADR-0014 in the same commit. Put the
whole-tree search command in the commit body, as the sweep rule requires:
`grep -rn "three structures" .` must come back clean.

**Amendment 18-3 (mandatory).** In the consequences, replace `must give the
same three structures` with `must give the same arrays`, and replace `**The
evidence for the three structures is a research report.**` with `**The
evidence for the block form is a research report.**`

### C. Forbidden material

One part is exemplary and should be the model for the other records:

> No figure appears here, because no measurement exists on the target
> platform.[^6]

That is the correct handling of section 4.1 and of BLK-007 together.

Outstanding items: the counts in the consequences, covered by 18-3, and one
more.

> Of the four fixed entity shapes, the bridge indexes the soldier.

`four` is a count. It is ADR-0066's structural claim, cited not restated, and
ADR-0066 is accepted, so it survives the rule. It should still go: if ADR-0066
ever takes a fifth shape this sentence is false, and the phrase adds nothing.

**Amendment 18-4 (recommended).** Replace it with `The bridge indexes the
soldier shape only.[^5]`

**A dangling footnote.** D1 cites `[^10]` for the bitplane. The references
section ends at `[^9]`. There is no `[^10]` definition. The record check
script should catch this; fix it whichever way — either define `[^10]` as the
ADR-0022 pyramid reference the bitplane belongs to, or point the bitplane at
D5 in this record and drop the marker.

**Amendment 18-5 (mandatory).** Resolve the `[^10]` marker.

No version, no percentage, no byte budget, no file table.

### D. Does the record match the code?

**Nothing implements it.** The acceptance must say so.

One forward-looking mismatch will otherwise be introduced silently. See error
3 below.

### E. Reasoning errors

**Error 1: the rejection of the offset array rules out a structure that
ADR-0056 already assumes. Outstanding, and it requires another record to
move.**

ADR-0018 rejects the compressed sparse row form with:

> the offset array grows with the tile count while the occupancy it describes
> grows with the unit count. The tile count is the larger of the two by a wide
> margin ... The rebuild would touch the whole array once for each frame, so
> the per-frame cost would follow the tile count rather than the work.

ADR-0056 D4 says:

> The count array that stores the occupancy of a tile bounds the capacity,
> because the count is one byte for each tile.

That is a dense array with one entry per tile, describing occupancy, which
movement reads and writes every frame. **It is the thing ADR-0018 just
rejected**, at one byte per entry instead of four, and ADR-0056 treats its
existence as settled rather than deciding it.

Either the count array exists, in which case ADR-0018's rejection argument is
false — the project already pays a per-tile occupancy array, so the offset
array is a widening rather than a new class of cost — or it does not, in which
case ADR-0056 D4's capacity bound has no basis. Both records cannot stand as
written.

I judge the count array survives, because ADR-0056 D3 sub-step 1 needs a
per-tile departure count and ADR-0018 offers no way to produce one. That makes
ADR-0018's rejection argument the part that must change.

**Amendment 18-6 (mandatory).** Replace the rejection paragraph with:

> The project rejects it for the offset array, not for the per-tile array. A
> per-tile array of counts already exists, because admission needs the
> occupancy of a target tile and its departure count in the same tick.[^2]
> What the compressed sparse row form adds is an offset array that must be
> exact everywhere, so its rebuild repairs every entry once for each frame
> even where nothing moved. The block range array is exact at the block and
> silent below it, so its rebuild cost follows the occupied blocks rather than
> the tile count. The search inside a block is what buys that.

If the project instead concludes that the count array should not exist, then
ADR-0056 D4 changes and ADR-0018 stands. **One of the two must move before
either is accepted.**

**Error 2: the stale-identity consequence contradicts D3. Outstanding.**

> **A stale identity can appear in the unit array.** The array holds
> identities across the barrier, so a reader resolves before it acts.[^7]

D3 says the engine rebuilds the whole bridge once for each frame, at the
barrier, from the occupying units. Structural change also batches at the
barrier, so no entity dies while systems run. If the rebuild happens at the
barrier after the structural apply, every identity in the unit array names a
live entity for the whole frame, and a stale identity cannot appear.

The consequence is either false, or true only because of an ordering the
record does not state: that the rebuild runs *before* the structural apply
within the barrier. Which of the two runs first is a real decision, it decides
whether every caller pays a resolution branch on the hot path, and no record
holds it.

A consequence that tells a reader to handle a case that cannot occur trains
that reader to distrust the record.

**Amendment 18-7 (mandatory).** State the ordering. If the rebuild runs after
the structural apply, replace the consequence with:

> **Every identity in the bridge is live for the whole frame.** The rebuild
> runs after the structural apply at the barrier, and no entity dies while
> systems run.[^11] A reader of the unit array therefore resolves an identity
> that cannot be stale. The resolution stays, because the bridge is not the
> only holder of an identity, but this reader never sees an absent entity.

Add `[^11]: ADR-0020, structural change batches at the barrier and applies by
tombstone and compact. `docs/adrs/REGISTRY.md``. If the rebuild runs first,
keep the consequence and say so in D3.

**Error 3: D3's tie-break does not match the identity layout. Outstanding.**

> Units that share a bridge key break the tie on the entity slot index, so the
> order is fixed and no two runs disagree.

`Entity` packs the generation into the high 32 bits and the index into the
low. `to_bits` returns the whole word, and its doc comment says `The sort key
uses this`. Sorting on `to_bits`, or on the derived `Ord`, orders by
**generation first** and by slot index second.

That order is total and deterministic, so this is not a determinism defect. It
is a record that will not describe the code. Two units on one tile with slots
5 and 9 sort as 9 then 5 whenever slot 9 carries the lower generation. A
reviewer holding D3 calls that a defect; a reviewer holding the code calls D3
wrong.

**Amendment 18-8 (mandatory).** Replace `break the tie on the entity slot
index` with `break the tie on the identity, taken as one integer`, and add to
the same paragraph:

> The identity is opaque, so this record does not say which of its parts
> orders first.[^7] It says only that the order is total, that it comes from
> stored state, and that it never comes from a thread finishing first.

This states the property the record needs — totality from stored state — and
stops it asserting a field order it does not own.

**Error 4: the motivating sentence is not supported by the record it cites.
Outstanding.**

> Movement needs the reverse map, because admission must know what already
> occupies a target tile.[^2]

ADR-0056 D3 does not get that from the bridge. It reduces intents by source
tile, sorts intents by target tile, and reads capacity from the terrain table
and occupancy from the count array. **The bridge does not appear in ADR-0056's
admission at all.**

So either movement is not the caller that motivates the bridge, or ADR-0056 is
missing a step. This does not sink the record — combat and adjacency queries
need the reverse map — but a record should not open with a motivation that its
own cited record does not use.

**Amendment 18-9 (mandatory).** Replace the sentence with:

> The reverse map is what a system asks for whenever it must act on the units
> that share a tile. Any system that joins a tile to the units on it needs it,
> and none of them can build it from the unit columns at the moment of the
> query.

Then settle with the author of ADR-0056 whether admission reads the bridge or
only the count array. If only the count array, footnote 2 moves to a different
caller and the `**Movement reads the bridge and never writes it**` consequence
needs the same treatment.

**Error 5: D1's new second paragraph claims something ADR-0014 owns.
Outstanding.**

> The arena is never sorted and never compacted, because the slot index is
> half of the identity.[^5]

This is a claim about the arena, in the bridge's record, and ADR-0014's
consequences already say `The engine can never compact the slot index space`.
One fact, two records, nothing failing when they disagree — the first
recurring defect shape, introduced by the amendment that was meant to sharpen
D1. It also cites `[^5]`, ADR-0066, which is not the record that holds the
claim.

**Amendment 18-10 (mandatory).** Replace it with:

> The bridge is wholly derived. It holds no fact that the entity columns do
> not already hold, and destroying it loses nothing. It reorders nothing that
> it does not own: the arena is not sorted to build it.[^7]

Footnote 7 is ADR-0014, which is where the non-compaction claim lives.

**What traces cleanly.** D2's argument for a derived key is the strongest
reasoning in the batch: a stored key would be a value declared twice with
nothing to fail when the copies diverge, which is this project's first
recurring defect shape, applied correctly and named. D3's rejection of the
incremental update is right and gives the real reason — the merge order of the
writes. D4's scratch index is correctly marked scratch, and `Nothing stores it
between frames` is the sentence that keeps it from becoming a second
declaration site. D5 is sound.

### F. Blocker hygiene

**Clean, and the best in the batch.** The record cites BLK-007, the only open
blocker, and cites it for exactly what it governs: the absence of a cost
figure. It calls no closed blocker open and states no value a closed blocker
resolved.

### G. What is missing

1. **The order of the rebuild against the structural apply at the barrier.**
   Error 2 turns on it and no record holds it.
2. **The reverse map for a fixed entity.** This record declines it with a
   non-sequitur; ADR-0012 D3 assumes it exists. See amendment 12-2.
3. **Whether admission reads the bridge.** See amendment 18-9.
4. **The relationship to the per-tile count array.** See amendment 18-6. Two
   records describe per-tile occupancy and neither cites the other.

### H. Verdict: ACCEPT WITH AMENDMENT

18-1, 18-2, 18-3, 18-5, 18-6, 18-7, 18-8, 18-9 and 18-10 mandatory. 18-4
recommended.

**This record should not be accepted before amendment 18-6 is settled with
whoever owns ADR-0056.** The two records currently make incompatible claims
about the same array, and accepting one binds the project to a reason that the
other falsifies.

**Objections attempted, and why each failed:**

1. *The bridge is derived state, so it is an implementation detail and needs
   no record.* Fails on the determinism counter-test. D3 forbids the
   incremental update, which is exactly what a contributor optimising the
   rebuild writes. Without the record that patch has nothing to fail against,
   and its nondeterminism appears only under a thread count and a movement
   pattern a test may not reach.
2. *Condition 2 fails, because derived state is cheap to change.* Fails, but
   only through the shared partition with the pyramid — hence amendment 18-1,
   which moves that argument to where it can be read.
3. *D4 records an algorithm, and an algorithm is an arrangement.* Fails. D4
   states what a caller must pay — a search, not a subscript — and that cost
   shapes every calling system's access pattern. That constrains callers
   rather than describing code.
4. *D5's bitplane duplicates ADR-0012's tile upgrade marker.* Fails. They are
   different bitplanes over different partitions: one bit per block for
   occupancy here, one bit per tile for upgrades there. Worth watching — a
   third bitplane forces a general decision — but not a duplication now.
5. *The record names a radix sort, which pins an implementation.* Fails.
   Section 4.2 bans a version and a named release. The determinism argument
   depends on the sort being on an integer key and total, and that is the
   claim. No crate and no version is named.
6. *D1 and D5 overlap, since D1 now lists the bitplane that D5 decides.*
   Fails, narrowly. D1 enumerates, D5 states what the bit means and who reads
   it. The enumeration is the weaker half, and amendment 18-3 keeps it from
   becoming a count that goes stale.

---

## Cross-cutting observations

Not verdicts. For whoever maintains the registers.

**One value, two records — twice.** The per-tile occupancy array is described
by ADR-0018's rejected alternative and by ADR-0056 D4, with no citation
between them. The non-compaction of the arena is claimed by ADR-0014's
consequences and by ADR-0018 D1. Both are the first recurring defect shape:
one fact in two places, with nothing that fails when the copies disagree. The
tile linearisation was a third instance and it is now repaired.

**The batch leans on unwritten records.** ADR-0015, ADR-0016 and ADR-0020 have
no files and all three carry load here. Every citation points at the registry
rather than a missing path, which is the honest form, but ADR-0014 D4 and
ADR-0018 error 2 both need ADR-0020's barrier ordering in order to be
checkable at all.

**Length is not the risk in this batch.** All four records sit at or below the
reference median: 780, 884, 1327 and 1291 words. The scope rule's standing
warning about Cachette drafts running from 1777 to 4570 words does not
describe them. ADR-0014 grew 208 words in one commit and is the one to watch.

**Two records have an implementation and two do not.** The registry's review
condition requires the acceptance of ADR-0012 and ADR-0018 to state plainly
that nothing implements them, and the acceptance of ADR-0014 to state that
only D1 and D6 do.

**A title change is a sweep.** ADR-0018's title changed and three other places
still hold the old one, including its own file name and its own consequences.
The sweep rule applies: the commit that lands the rename carries the
whole-tree search command in its body.

---

## Addendum: a second wave of edits during the review

The records moved again after the body above was written. This addendum states
the status of every mandatory amendment against the working tree as it stands
now. Nothing in the body is withdrawn.

### Landed

- **18-2, partly.** The file is renamed to
  `adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`.
  The sweep is **not** finished. ADR-0012 footnote 6 and ADR-0014 footnote 2
  both now carry the new path with the **old title text**, `the unit-to-tile
  bridge is three structures, and units stay sorted by tile`. A path and a
  title updated apart is the half-sweep the commit rule warns about. Finish it,
  and run `grep -rn "three structures" .` before the commit.
- **18-5.** The `[^10]` marker is defined, as ADR-0007.
- **18-10, partly.** D1's non-compaction sentence now cites `[^7]`, ADR-0014,
  which is the record that owns the claim. The duplication itself remains, and
  it has since spread. See below.

### Outstanding, unchanged

14-1, 14-2, 14-3, 17-1, 12-1, 12-2, 18-1, 18-3, 18-6, 18-7, 18-8, 18-9. I
re-checked each against the current text and each still applies verbatim.

### Sharpened by the new edits

**Error 3 in ADR-0018 got worse, not better.** D3 now reads:

> Units that share a bridge key break the tie on the entity slot index, so the
> order is fixed and no two runs disagree.[^4] The key is a vector of exact
> integer fields whose last field is a stable identifier, which is the form the
> engine sorts by everywhere.[^10]

The new sentence is correct about the key-vector form, and it makes the
mismatch concrete: the stable identifier in that last field is the identity,
whose integer form is **generation-major**, because `Entity` packs the
generation into the high 32 bits. The record now says `slot index` in one
sentence and `stable identifier` in the next, and only the second is
achievable with the current type. Amendment 18-8 stands and is now the
cheapest way to make both sentences true at once.

**The non-compaction claim is now in three records.** ADR-0014's consequences
hold it, ADR-0018 D1 restates it, and ADR-0056's context has just gained
`The unit arena itself is never sorted, because the slot index is half of the
entity identity.[^11]`. Three declaration sites for one fact, each citing a
different neighbour, with nothing that fails when they disagree. ADR-0014 is
the owner. The other two should cite it and state nothing.

**ADR-0056 moved toward ADR-0018 on the sorted-array question and did not
move on the count array.** ADR-0056 dropped `The sorted unit array survives`
and its context now says the bridge is rebuilt at each barrier so a moving
system does not maintain it. That resolves a real disagreement, and it is the
right direction. **ADR-0056 D4's per-tile count array is untouched**, so
amendment 18-6 — the conflict that blocks acceptance — is exactly as it was.

### The standing recommendation

**ADR-0018 still must not be accepted before amendment 18-6 is settled.** Two
records describe a per-tile occupancy array, one rejects that class of
structure by an argument the other falsifies, and the last two waves of edits
have not touched it.

## References

[^1]: ADR Registry. `docs/adrs/REGISTRY.md`
[^2]: Decision Record Scope. `.claude/rules/adr-scope.md`
[^3]: Documentation Rules. `.claude/rules/documentation.md`
[^4]: Recurring Defect Shapes. `.claude/rules/recurring-defects.md`
[^5]: Blockers register. `docs/BLOCKERS.md`
[^6]: Reviews index, what a review must contain. `docs/reviews/README.md`
[^7]: Definition of Done. `.claude/rules/definition-of-done.md`
[^8]: Commit Message Rules. `.claude/rules/commits.md`
