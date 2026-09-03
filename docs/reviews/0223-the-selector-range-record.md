# Review 0223: The selector range record

## What was reviewed

| Item | Value |
|---|---|
| `docs/adrs/draft/adr-0052-a-selector-result-may-be-a-range.md` | `Draft` at review, and `Draft` after it |
| Commit | `25b6c62`, the head of the review branch |
| Code read | the hex grid index, the block layout of the bridge, the pyramid block fold, the tile values, and the holding module |

The reviewer did not write this record. The reviewer accepted ADR-0051 in the
commit before this one, and ADR-0051 names this record as the thing that carries
its cost case, so the reviewer had already spent something on this record being
sound. Section 4 states what the reviewer did about that.

**The reviewer compiled nothing.** Other workers hold the machine.

## Verdict

**Accept with amendment.** D1, D4, D5 and D6 are sound, and D6 is the best
example in the corpus of a record naming a parameter instead of inventing a
value. D2 and D3 rest on a present-tense claim about the storage that the code
contradicts.

Section 5 gives the exact text. The status stays `Draft`.

**Four objections were attempted. One held, and it is the one that matters
most, because D2 is the whole cost case.**

## 1. The objection that held: the world is not stored in blocks

The context says:

> The world already has a structure that a result can use. Tiles are stored in
> blocks at the aggregation block size, and the summary pyramid divides the
> world into the same blocks.

D2 repeats it inside a decision:

> The world is stored in blocks, and the summary pyramid uses the same blocks.

**Tiles are stored row by row.** The grid derives a tile index from an axial
address as the row times the width plus the column. That is a row-major order
and it is the only storage order the engine has.

**The block-tiled space exists, and it is a different space.** The block layout
of the unit-to-tile bridge builds a key from an address by interleaving the
block of the address with the position inside that block. That key is
block-major. It is not the tile index, and the engine converts between the two.

**The pyramid proves the two do not agree, in its own comments.** The fold that
summarises one block does not read one run. It reads one run for each row of the
block, because a block is a rectangle over a row-major column, and it says so:
one row of a block is one contiguous run of the value column. A block of edge
`n` is `n` runs, not one.

So the two halves of the claim have different truth values. The pyramid does
divide the world into blocks. The storage does not.

**This is the premise of D3, and D3 fails with it.** D3 says the blocks of the
result are the blocks of the storage, that the engine does not map one space
onto another, and that a verb therefore reads a run rather than gathering
scattered values. Under the storage that exists, a block-shaped result is
several runs, and the engine already maps one space onto another in the bridge
key.

**The rest of the corpus is careful here and this record is not.** Three
accepted records cite the same reserved row for the same layout, and all three
speak of it conditionally. The storage record says the derivation is arithmetic
whichever order the tiles sit in. The tile index record says the record that
holds the memory order may choose a block order rather than a row order. The
bridge record cites it without asserting it. This record alone states it as
present fact, and the reserved row it cites has no file and no decisions.

**What this does not do.** It does not overturn D2's requirement. A whole block
costing one entry is a property the representation must have whatever the
storage order is, and it is worth having. It makes D2's cost case conditional
rather than established, and it makes D3 a statement about a layout the project
has not chosen.

## 2. The decisions that hold

**D1, the result is a set representation and enumeration is one form.** Sound.
The definition it gives is the right one: membership and ordered yield, with no
form named in any signature.

**D4, an enumerated array exists at the boundary only.** Sound, and it is the
decision that keeps D1 honest. Without it a caller's flat array would leak
inward as the working form, which is how a lazy design becomes an eager one
without anybody deciding to change it.

**D5, one fixed order whatever the representation.** Sound, and it is the
determinism decision. Its consequence is the interesting part: it makes the
representation an implementation choice that the two determinism tests cannot
see, which is exactly why the choice may be left open.

**D6, the record names the representation as a parameter and states no measured
value.** This is the decision to hold up as an example. The scope rule says that
where a blocker governs a value, the record states the value as a parameter and
cites the blocker.[^1] D6 does precisely that, names the open choice, and points
at the decision row that carries the options and the recommendation. The row
exists and holds all three options.

**The consequence about testing is also right and rarely written.** It says a
test that compares only members passes when the engine enumerates everything, so
it measures nothing about this record, and it says to put the enumerating
implementation back and watch the test stay green. That is the project's own
rule about a fixture reaching the case, applied before the code exists.[^2]

## 3. The objections that failed

**Does accepting ADR-0051 without this record leave the accepted record resting
on an unreviewed one?** **The objection fails.** ADR-0051's consequence names
this record as carrying the argument rather than asserting the cost itself, and
it states in the same paragraph that nothing implements it. A record that names
its dependency instead of borrowing its conclusion is the shape a previous
review already accepted in another record.[^3] It is worth saying out loud
because the reverse would have been a defect.

**Does D2 smuggle a figure in, by naming the aggregation block size?** **The
objection fails.** The record names no number anywhere. It names the block size
as the thing the storage and the pyramid share, which is a structural constant
of the layout and not a budget.

**Does the negation consequence promise a bound nobody can state?** **The
objection fails.** It says the engine bounds a negation by the domain the
selector already restricts and that no free-standing complement exists in the
interface. Both are checkable at review, and neither needs a measurement.

**Does D1 conflict with the identity record, which fixes how an entity crosses?**
**The objection fails.** D1 governs the internal form of a result and D4 governs
what leaves. The identity record governs the value inside a column that crosses,
and the two do not touch.

## 4. What the reviewer did about having accepted ADR-0051 first

Accepting ADR-0051 in the previous commit gave the reviewer a reason to want
this record sound, because ADR-0051 points at it for the cost case.

The reviewer therefore did not read this record against ADR-0051 at all. It read
the context's factual claims against the source first, before reading any
decision, which is where the objection in section 1 came from. The relationship
between the two records was examined afterwards, as an objection in its own
right, and it is the first entry in section 3.

**The accept of ADR-0051 does not depend on the outcome here.** ADR-0051 names
this record rather than quoting it. That was checked before the accept and it is
checked again above.

## 5. The amendment

The reviewer did not edit the record. The text below is a proposal for the
author.

**First**, replace the context paragraph:

> The world already has a structure that a result can use. Tiles are stored in
> blocks at the aggregation block size, and the summary pyramid divides the
> world into the same blocks.

with:

> The world has half of a structure that a result can use. The summary pyramid
> divides the world into blocks and aggregates over them. The tile columns are
> stored row by row, so a block is a rectangle over a row-major column and the
> pyramid reads one contiguous run for each row of a block rather than one run
> for the block. A block-major key space exists for the unit-to-tile bridge, and
> the engine converts between it and the tile index. Storing the tiles in block
> order is a reserved number with no record and no implementation.

**Second**, replace the first sentence of D2:

> The world is stored in blocks, and the summary pyramid uses the same blocks.

with:

> The summary pyramid divides the world into blocks. Whether the tile columns
> are stored in the same blocks is an open layout question that a reserved
> number holds.

**Third**, add to D3, after "The project refuses that step":

> **This decision is conditional on the storage order, and the storage order is
> not settled.** Under a row-major column a result block is one run for each row
> of the block, and the engine already maintains a second, block-major key space
> for the unit-to-tile bridge. D3 binds the result to whatever blocks the
> storage uses. It does not assert that those are the pyramid's blocks today.

The references for the reserved row and the finding go in the reference list as
footnotes.

**If the author judges instead that the storage must be block-tiled for this
record to work**, then the dependency is real and the repair is the other way
round: this record is blocked until the layout record is written, and it says so
rather than assuming the layout.

## 6. What acceptance would settle, and what it would not

Once amended, this record settles that a whole block costs one entry and that
the working form is never an enumeration. It does not settle which concrete form
the engine uses, and D6 correctly refuses to.

It does not make the storage block-tiled. That is a separate decision, and the
finding records that the project has been reading the reserved row as though it
were settled.[^4]

## References

[^1]: Decision Record Scope, section 4.5. `.claude/rules/adr-scope.md`
[^2]: Testing rules, section 2a. `.claude/rules/testing.md`
[^3]: Review 0047, the viewer boundary record. `docs/reviews/0047-the-viewer-boundary-record.md`
[^4]: Findings register, FND-217. `docs/FINDINGS.md`
