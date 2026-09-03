# Review 0223: The influence storage record

## What was reviewed

| Item | Value |
|---|---|
| `docs/adrs/draft/adr-0060-an-influence-map-is-stored-as-a-shared-basis.md` | `Draft` at review, and `Draft` after it |
| Commit | `b60319c`, the head of the review branch |
| Code read | the influence module in full, the world construction, the pyramid cell summary, and the findings behind the cell width |

The reviewer did not write this record and has no record of its own that cites
it.

**The reviewer compiled nothing.** Other workers hold the machine, and the
influence module is one of the files another worker may hold.

## Verdict

**Accept. The status was not changed, and the reason is mechanical.**

Every decision holds against the code. Four objections were attempted and all
four failed. Section 5 records the one place where the source comments read
worse than the record does, which is a note for whoever owns that module and not
a change to this record.

**Accepting a record moves its file, and this record is cited eleven times, five
of them in the influence module itself.** This work is forbidden to touch that
tree, and moving the file without repairing those five would leave five comments
naming a path that does not resolve. The verdict above is the reviewer's
judgement. The move and its sweep are one commit for whoever may edit `crates/`,
and that is the same position two other records are already in.[^1] [^2]

**Which questions this review put explicitly, and which it answered by
inference.** Section 4 puts the first and the third conditions of the record
scope test: a contributor writes the per-faction plane without thinking about
it, and the module cannot say what was refused because the refusal is an
absence. The question of whether the record contradicts the code is sections 1
and 2. **The second condition was answered by inference.** The reviewer judged
that a storage shape every consumer reads is expensive to reverse, and named
D3's missing inverse as the thing a contributor would trade away, which is
adjacent to that condition without being it.

## 1. The decisions against the code

**D1, a value that does not depend on the faction is stored once.** Holds. The
field holds one influence plane for each faction, one source plane for each
faction, and one conductance plane that is not indexed by faction. A source is a
faction's own contribution and not a property of the ground, so it is correctly
per faction.

**D1's read-only clause is held by the compiler, not by discipline.** A pass
takes the conductance plane by shared reference while it relaxes, and every
method that writes the conductance takes the field by exclusive reference, as
the solve does. Rust therefore refuses a write to the shared plane during a
solve. The record claims this is what makes sharing safe under a weak memory
model without an atomic, and the claim is enforced rather than asserted.

**D2, a narrow unsigned integer against a fixed reference.** Holds. The cell is
a two-byte unsigned newtype whose ceiling is named as one reference unit, and it
is not the project-wide fixed-point scale.

**D3, saturating addition at the ceiling.** Holds. The combine is a saturating
unsigned add, and the type's own documentation gives the same reason the record
does: the operation is exactly associative and commutative with zero as its
identity, so a fold gives one answer whatever the order.

**D4, one scratch plane for every faction.** Holds, and the code cites this
decision for it. One scratch plane is allocated once, a plane is relaxed into it
and copied back, and the factions are visited in ascending identifier. The
record says the copy is the price and that nothing has priced it because no
measurement exists on the target platform. The module says the same thing in the
same words.

## 2. The consequences against the code

The record makes one present-tense claim about the code, and it is true.

> The shared plane fixes where the update may run. No faction may write the
> shared plane during a solve, so the rule that fills it runs outside the solve.
> Today the ground fixes it once, and nothing writes it after a world is built.

The conductance is filled once, during world construction, from the level 1 cells
that the same construction has just filled. Nothing calls it again. The comment
at the call site gives the reason the record gives.

The claim that a consumer cannot ask what every faction holds at one cell in one
read is also true: the read takes a faction and a cell.

## 3. The objections that failed

**Does the record claim a measurement that BLK-007 says nobody has?** This is
the objection worth taking seriously, because the record says the project
measured the cost of a narrow cell. **It fails.** The measurement is not a cost
figure and not a timing. It is one fixture with one stencil and one source, run
to rest, comparing how far the field reached at one cell width against another.
The finding states the result in cells reached, and states that the reach is a
property of the arithmetic and so does not depend on the machine.[^3] That is a
measurement the development machine can make honestly, and the record cites it
without quoting a number.

**Does the widening of the cell contradict D2's claim that the ceiling means one
reference unit?** **It fails, and section 5 records what is behind it.** D2
defines the cell's own scale and says in the same sentence that it is not the
project-wide scale. Whether the ceiling maps to one whole number or to some
other value inside the wider scale is a property of the conversion the kernel
uses, not of the cell's definition, and the record does not claim otherwise.

**Should D4 belong to the record about the solve rather than this one?** **It
fails.** D4 decides how much storage the write half of a pass holds. That is a
storage claim, and it is what forces the copy. The solve record governs how many
passes run and on what schedule, which D4 does not touch.

**Does the record state a figure anywhere?** **It fails**, which is to say the
record is clean. There is no width, no octave count, no reach, no share and no
percentage anywhere in it. Every quantity is named as a quantity and cited. The
record even declines to name the cell width, which is the number most likely to
change again, and the finding shows it has already changed once.

## 4. Why this record earns its place

The obvious storage is a plane for each concern for each faction, and it is what
a contributor writes without thinking about it, because it makes every read a
direct index. The code as it stands shows one conductance plane and one scratch
plane, and says nothing about the two shapes that were refused. A reader cannot
see from the module that a transposed store was considered, or that a per-plane
exponent was rejected as a determinism hazard.

That is the third condition of the scope test answered plainly: the reasoning
cannot live in the artefact, because the artefact is an absence.[^4]

D3's second half is the part a future contributor would trade away. Saturating
addition has no inverse above the ceiling, so a cell cannot be repaired by
removing a contribution, and the only repair is to solve again. Someone will
eventually want to retract one contributor's share cheaply, and this record is
what refuses it.

## 5. A note on the module, not on the record

The conversion that widens a cell into the project-wide scale shifts by the
difference between the scale's fractional bits and the width of a byte. Its
comment explains the shift by what the byte ceiling used to map to. After the
widening the comment is hard to read as a description of the current mapping,
because it describes the provenance of the constant rather than the mapping the
constant now produces.

**Nothing here is wrong and nothing here is a defect of this record.** The
conversions are shifts, they are exact, and the record's D2 disclaims the
project-wide scale explicitly. This is a comment that would read better if it
said what the mapping is rather than what it used to be, and it is written down
so that whoever next opens that module does not have to derive it twice.

The reviewer may not edit that file.

## 6. What acceptance would not settle

It does not settle what fills the conductance plane. D1 says so, and points at
two open rows for the ground rule and the content table.

It does not settle how many passes a solve runs, which is a separate record
waiting for its own review.

It does not price the copy in D4. The record names the figure that would reopen
the decision and states that no measurement exists on the target platform.[^5]

## References

[^1]: Findings register, FND-197. `docs/FINDINGS.md`
[^2]: Review 0204, the two corrected records. `docs/reviews/0204-the-two-corrected-records.md`
[^3]: Findings register, FND-159. `docs/FINDINGS.md`
[^4]: Decision Record Scope, section 1. `.claude/rules/adr-scope.md`
[^5]: Blockers register, BLK-007. `docs/BLOCKERS.md`
