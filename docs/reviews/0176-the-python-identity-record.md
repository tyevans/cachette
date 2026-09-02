# Review 0176: The Python identity record

## What was reviewed

| Item | Value |
|---|---|
| `docs/adrs/accepted/adr-0085-an-entity-crosses-to-python-as-one-opaque-identity.md` | Status `Draft` at review, and `Accepted` after it |
| Commit | `506d3d1`, the head of `main` at the time of the review |
| Code read | the value types, the world, the soldier arena, the bindings, the type stub, the agent server, and the identity resolution test |

The reviewer did not write ADR-0085 and did not write the change it
accompanies. The reviewer read the record, the rules, and the tree. The
reviewer did not read the reasoning of the author, who was not available.

**The reviewer compiled nothing.** Three other workers held the machine, so no
`cargo` command ran. Section 4 names every claim that only a run can settle.
Each of those is marked unverified and is not counted toward the verdict.

## Verdict

| Record | Verdict |
|---|---|
| ADR-0085 | ACCEPT. The record moves to `accepted/`. |

The constraint is sound and it is a constraint rather than a topic. The title
states a claim. All four decisions have code behind them. The body holds no
version pin, no count, no file table and no measured figure. The record is 997
words, the shortest of the drafts and below both reference medians.

Six objections were attempted. All six failed. Section 3 states each one and
why it failed, including the one the reviewer expected to hold.

## 1. The record against the code, decision by decision

**D1 holds.** The identity is a non-zero 64-bit value that packs the generation
above the index. The bindings return the unit column of the gather log as an
array of that whole value, taken through the method that gives the bits. The
type stub says so in its own words: the unit column holds the whole identity and
is not a slot index.

The bindings expose no column of slot indices. The reviewer searched the
bindings for an accessor that splits an identity and found none. The core crate
has an index accessor and a generation accessor, and neither is reachable from
Python, because nothing in the bindings calls them for an outgoing value.

D1's width sentence is the right treatment of a number. It says the width
follows from the decision and is not a budget, and the width it names is a
property of the identity layout that another record already fixed. That is a
structural constant rather than a figure a measurement can change.

**D2 holds, and the compiler holds it.** The function that builds an identity
from bits is private to the crate. No function of the bindings takes an index
and a generation. Python receives a value, holds it in an array, and gives it
back.

**D3 holds.** Every binding that takes an identity goes through one function
that calls the resolution and turns a refusal into the typed error for a stale
view. The reviewer checked each of the three: the set-valued despawn, the
set-valued gather order, and the singular tile read. The gather order resolves
every identity in the set before it gives any order, which is stronger than D3
asks for.

The resolution itself compares the generation the value carries against the
generation the arena holds, refuses a mismatch, and never returns the soldier
that now occupies the slot. It distinguishes three refusals: a value that is not
an identity, a slot the arena never opened, and a slot that holds a later
generation.

**D4 holds as a constraint on future work.** It states that a later binding
taking a settlement, a character or a site follows D1 to D3. It claims nothing
about code that exists. That is the correct form: a record carries the
constraint, and the constraint is what a reviewer needs in order to reject a
later change.

## 2. Why the decision needed a record

The three-condition test is met, and it is worth stating because the record is
short and the mechanism is small.

**A future contributor could reasonably choose otherwise.** The record says so
and gives the reason: the index is half the width, it indexes a column directly,
and it reads as the obvious name of a unit. The project has met that exact
choice once inside the engine, where a movement system keyed a random draw on
the slot index.[^2]

**Choosing otherwise costs more than changing it later.** The boundary is
published. A Python caller that holds an index and watches a unit across a death
reports on the wrong unit with every test green.

**The reasoning is not visible in the artefact.** A signature that reads
`unit: int` says nothing about which integer is meant. The record states this
itself.

## 3. Every objection the reviewer attempted

**Objection 1: the record claims that the resolution is what protects the
Python read, and a reinserted defect disproved that.** This is the objection the
reviewer expected to hold. **Failed.** The finding records that deleting the
generation comparison left the Python read green, because the arena compares the
generation a second time when it reads a tile, and that a test above both checks
covers neither on its own.[^1] The record makes no claim about test coverage. It
claims that every binding resolves before it acts, and that a resolution which
fails raises the typed error. Both are true of the code with the comparison in
place, and the finding does not touch either. The finding is about what a test
can prove, and this record does not claim a proof.

The reviewer looked for a weaker form of the same objection and found one
sentence worth noting rather than blocking: the consequences say each binding
costs one resolution, and a reader could take that as the only generation
comparison on the path. The tile read pays a second one inside the arena. The
sentence is still true as written, because one resolution is what the binding
costs, and the arena's own check is not a resolution. The record is not required
to describe the arena's internals, and no amendment is asked for.

**Objection 2: the record claims an enforcement that nothing performs.** The
record says the control plane that wants to act on a set builds a selector and
lets the engine resolve it, and the reserved record that would refuse a loop over
a declared tier is unwritten. **Failed.** The sentence says what the design asks
for and cites the orientation, which does ask for it. It does not say that
anything refuses a loop. The record claims exactly one enforcement, that the
identity constructor is private to the core crate, and that one is true.

**Objection 3: D4 records an intent as if it were a fact.** **Failed.** D4 binds
future bindings. That is a constraint, which is what belongs in a record, and it
is not a claim that a capability exists. The category the scope rule bans is a
declared capability nobody invokes, and D4 declares none.

**Objection 4: the width is a figure that belongs in a reference table.**
**Failed.** The width is fixed by the identity layout that another record
decided, not by a measurement. The record pre-empts the objection in its own
text and says the width follows from the decision.

**Objection 5: the record decides which fields of an event cross, which the
decisions register owns.** **Failed.** The record says in its own words that it
does not decide which fields cross or in what form the others cross, and it
cites the register row that holds that choice.

**Objection 6: the record has no citation from any source file, so it may be a
description rather than a constraint.** **Failed on the facts.** Four source
files cite it: the value types, the world, the bindings and the identity test.
The agent server cites it too, and the type stub carries the rule in prose. Low
citation is a question for review, and this record answers it.

## 4. What only a run can settle

The reviewer compiled nothing. The claims below rest on the source, and none is
counted toward the verdict.

- **The identity resolution tests pass.** Six tests read the public interface:
  a live identity resolves, a dead one refuses before the slot is reused, zero
  is not an identity, a composed index one past the end refuses, and the gather
  log names a unit that resolves. Whether they pass was not observed.
- **The Python read and the write verbs refuse a stale identity at runtime.**
  The path was read and it is correct. The behaviour was not observed.
- **The bindings build against the current core crate.** The reviewer rebased
  onto the head of `main` before reading, because the branch was one commit
  behind and that commit repaired a call site the reservation sweep missed.

## 5. What the review changed

- The record moved from `docs/adrs/draft/` to `docs/adrs/accepted/`.
- The registry row holds `Accepted`.
- The priority index no longer lists the record as waiting for review.
- A whole-tree search replaced the draft path in seven files. The search
  command is in the commit body.

No register entry opened or closed. The record needed none.

## 6. Checks run

Six document checks were run after the move. All six pass. They compile nothing.

| Check | Result |
|---|---|
| `scripts/check_adrs.py` | 0 failures |
| `scripts/check_citations.py` | 0 failures |
| `scripts/check_registers.py` | 0 failures |
| `scripts/check_priority.py` | 0 failures |
| `scripts/check_footnotes.py` | 0 failures |
| `scripts/check_conflict_markers.py` | 0 failures |

The whole gate was not run, because three other workers held the machine.

## References

[^1]: Findings register, FND-148. `docs/FINDINGS.md`
[^2]: Testing Rules, section 2. `.claude/rules/testing.md`
