# Review: ADR-0017, the world is a rhombus, so a tile index is raw axial

**Reviewed:** `docs/adrs/draft/adr-0017-the-world-is-a-rhombus-so-a-tile-index-is-raw-axial.md`,
against `crates/cachette-core/src/hex.rs`, `crates/cachette-core/src/world.rs`
and `crates/cachette-core/tests/hex_geometry.rs`, at commit `6fa3a44`.

**Reviewer:** an agent that wrote no part of the record and no part of the
code. **Verdict:** `ACCEPT WITH AMENDMENT`. The amendment is applied.

## The three-condition test

Passes all three.

The first condition is the interesting one. A contributor could reasonably
choose otherwise, and would: the project's own research recommends the
alternative. Report 02 says to use an offset index for the array position.
A contributor who reads the research and not the record implements offset
indexing. That reader is the reason this record exists.

The third condition is the strongest. The code shows an index derivation of a
multiply and an add. Nothing in the code explains why there is no offset
step, because the absence of a function cannot be seen.

## The record against the code

Both hard claims were verified rather than accepted.

**"A conversion function does not exist in the engine" is true.** A whole-tree
search over the crate and the Python package found no conversion. The two
conversion functions that report 02 supplies by name were not copied in. The
only matches for the word are the neighbour offsets, which are a different
thing.

**"The engine never holds a screen position" is true, and currently vacuous.**
No screen, pixel or projection type exists. Nothing has yet tried to add one
and no viewer exists, so this is a constraint on the future rather than a
description of a tension that was resolved. The record is accepted on that
understanding. A later reader should not conclude that a projection was
removed from the engine.

D2 and D3 hold. The neighbour tests that a wrapping defect would fail first
exist and pass, and the symmetry test is a property rather than an example.

## The evidence

The premise that the record disagrees with its own evidence is only half
right, and the review corrected the approver on this.

Report 02 recommends offset indexing for a rectangular world, and then
explicitly offers this record's choice: storing raw axial and accepting a
rhombus world is called a valid choice that removes the conversion entirely,
to be taken if the design allows a rhombus map. The report forwards to the
block aspect ratio as the cost. The record takes that option, for that
reason, and pays that cost. The section of the report that discusses rhombus
blocks is itself titled as the recommended one.

So the record does not contradict its evidence. It resolves an option that
the evidence deliberately left to the project owner, which a blocker then
closed.

The report names one further failure mode, which is that parallelogram blocks
cannot interoperate with a geospatial dataset. The record omits it. The
objection fails, because the report's next sentence rules it inapplicable to
a game world with no geographic reference frame. It was considered and
dismissed on the report's own grounds.

## The amendment that held

**D1 stated the index function, which is an arrangement rather than a
constraint.** It said the index is the row multiplied by the row length plus
the column. That is row-major order. The registry row for ADR-0016 claims
tiles are stored in block-tiled order, and ADR-0017 names 0016 among its
dependencies. Those are different index functions.

Nothing was broken yet, because ADR-0016 has no file and the code is
row-major. The defect is what happens next: once ADR-0016 is written, either
it supersedes a sentence inside an accepted ADR-0017, or ADR-0017 reads as
forbidding it. A record cannot both depend on a record and constrain it.

This is the recurring shape of one fact declared twice with nothing failing
when the copies disagree.

**Applied.** D1 now says the index is derived from the address by arithmetic
and nothing else, and states that the record does not fix the index function.
The order in memory belongs to ADR-0016. The constraint that survives is the
one the record is about: the address is a raw axial pair and no access
converts a coordinate.

## Objections attempted that failed

Sixteen were attempted. Fifteen failed, each for a reason.

- Fails condition one, no real alternative. Failed: the project's own report
  recommends the alternative.
- Fails condition two, cheap to change later. Failed: every storage record
  above it inherits the index shape.
- Fails condition three, the reasoning is visible in the code. Failed: the
  absence of a conversion is invisible by construction.
- Holds a volatile figure. Failed: the record cites the aspect ratio
  measurement and quotes no number, which is what the rule asks for.
- Holds a version pin, a count, or a file table. Failed: none present.
- Holds a value an open blocker governs. Failed: the blocker is resolved and
  the value lives in the scale constants table.
- Records a capability nothing invokes. Failed: every function has a real
  caller, and the test drives the world rather than the grid.
- The no-conversion claim is aspirational. Failed: confirmed by search.
- The no-screen-position claim is unverified. Failed: confirmed, though
  vacuous, which this review states.
- Overstates the evidence or hides the disagreement. Failed: see above.
- Omits the geospatial failure mode. Failed: the report rules it
  inapplicable.
- Missing decision, the saturating addition. Failed: the reasoning is visible
  in the code, so condition three is not met.
- Missing decision, the refusal of a tile count that overflows the index.
  Failed: that is a property of the index width and belongs to ADR-0011.
  Putting it here would give the record a second claim.
- Missing decision, the neighbour order and the declared layout. Failed: both
  are held by ADR-0004 and ADR-0006 and are cited as such in the code.

## References

[^1]: Decision Record Scope. `.claude/rules/adr-scope.md`
[^2]: Reviews index. `docs/reviews/README.md`
