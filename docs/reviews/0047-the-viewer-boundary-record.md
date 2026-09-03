# Review 0047: The viewer boundary record

## What was reviewed

| Item | Value |
|---|---|
| `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md` | Status `Draft` at review, `Accepted` after it |
| Code read | the whole viewer crate, and the core crate for the boundary checks |

The reviewer did not write ADR-0067.

**The reviewer wrote later parts of the viewer under the record as it stands**,
and that is a weaker independence than a second reader gives. It cuts toward
accepting. Section 5 states what was attempted against it, and the one finding
below was found by reading the record's own words against the binary rather
than by trusting the code.

## Verdict

| Record | Verdict |
|---|---|
| ADR-0067 | Accept, with D1 amended |

Four of the five decisions hold against the code without a change. D1 states
its constraint over a subject that includes a program the record elsewhere
requires to write to the world, so D1 and D4 contradict each other as written.
The amendment narrows D1's subject to the path it was always about. No
constraint is weakened.

## 1. The finding: D1 and D4 contradict each other

D1 says the viewer holds a shared reference, calls no method that takes a
mutable reference, never spawns or moves an entity, and never advances a tick.

D5 says the viewer is a crate.

D4 says one loop steps the engine, then draws.

The loop D4 requires is in the crate D5 names. The demonstration binary in
that crate builds a world, spawns six hundred units into it, and calls the
step. Each of those is a method that takes a mutable reference, and one of
them advances a tick. Under D1's subject, the viewer does all three things D1
forbids.

The intent is not in doubt. D1's own rationale is that "a viewer that could
write to the world would put a person's choice of what to look at into the
simulated state". Stepping the engine is not a person's choice of what to look
at. The consequences section names the demonstration binary and says D4 ties
its drawing rate to its tick rate, so the record knows the binary steps.

**The defect is the subject, not the constraint.** D1 is about the path from
the world to the picture. It was written as though that path were the whole
crate.

This matters because the violation must be findable. A reviewer asked to
enforce D1 as written must either refuse the demonstration binary, which the
record requires to exist, or read past the words to the intent. A constraint a
reviewer has to read past is not a constraint.

### The amendment

D1's subject becomes the drawing and the reporting rather than the crate, and
a sentence says plainly that the program owning the loop is not bound by it
and why. The constraint on the drawing path is unchanged, and the compiler
still enforces it: `paint::draw` and `Readout::of` both take a shared
reference, so a write from either is a compile-time error.

A draft exists to be edited, and this one is edited rather than superseded.

## 2. The record against the code, decision by decision

**D1, as amended.** `paint::draw`, `hud::draw`, `hud::bounds` and
`Readout::of` all take `&World` or no world at all. No drawing or reporting
function takes a mutable reference. The two call sites that do are the
demonstration binary's own: it builds the world it is going to show, and it
steps it.

**D2.** A search of the core for a field named for a display finds none: no
colour, no pixel, no screen position, no camera, no zoom, no font. The palette
and the kind names live in the viewer. The counts the panel reports are the
canvas's own. The one value that looks like an exception is the tick, and the
engine holds a tick because a simulation has one, not because something draws
it.

**D3.** The float ban script passes over the core, so no floating point type
appears in simulated state. In the other direction, no value that has been a
float reaches an engine call: the camera converts a screen position to a tile
address by rounding to an integer before it names a tile, and every engine
call in the viewer takes an integer, an address or an identity.

**D4.** One loop, in `main`, steps and then draws. Nothing in the crate spawns
a thread that touches the world while the step runs. The record states the
consequence rather than leaving it to be discovered, which is the part of D4
worth keeping.

**D5.** The viewer crate depends on the core, and the core's manifest does not
name the viewer. The dependency direction is what makes D2 a compiler question
rather than a reviewer's question, which is what the decision claims for it.

## 3. The dependency, and why it does not hold this record

The registry says ADR-0067 depends on ADR-0001 and `ADR-0036`. ADR-0001 is
accepted. `ADR-0036` is `Proposed`, which means a number is reserved and no file
exists.

A record that rests on a record with no file could not be accepted, for the
reason the review of ADR-0070 gives. This is not that case.

ADR-0067 cites `ADR-0036` exactly once, and it cites it for its absence: the
alternative design "needs a snapshot mechanism that no record holds". The
citation is to the registry row, which is the documented way to name a
reserved number. Nothing in ADR-0067 is built on a decision of `ADR-0036`,
because `ADR-0036` has no decisions.

The binding content of ADR-0067 rests on ADR-0001, ADR-0002 and ADR-0017, all
of which are accepted.

**This section reads its subject as it stood at the review.** On 2 September
2026 the project retired that reserved number and repaired the footnote in
ADR-0067 and the row in the registry. The conclusion is unchanged and is
stronger: the number now holds nothing at all, and ADR-0067 never rested on
it.[^R47A]

**The registry's "Depends on" column does not distinguish the two kinds of
citation.** That is worth knowing and is not worth changing: a reviewer who
reads the record finds out in one search, and a column that tried to say it
would be a second declaration of what the footnotes already hold.

## 4. The record against the scope rule

**Five claims, separable.** Each of the five decisions has a violation the
others permit. A viewer that wrote to the world violates D1 alone. An engine
field named for a display violates D2 alone. A float handed back violates D3
alone. A viewer on its own thread violates D4 alone. A core that named a
viewer type violates D5 alone.

**No volatile material.** No count, no file table, no measured figure, no
version. D5 names a crate boundary, which the scope rule allows explicitly,
because a compiler enforces it.

**The title states the claim.** It is testable and it is not a topic.

**Length.** About 1000 words for five decisions, under both reference medians.

**Citation count.** Cited by ADR-0070, by ADR-0068, and by the viewer crate
throughout. Not an uncited record.

## 5. Objections attempted

**"D5 is not a decision, it is a module arrangement."** The scope rule names a
module arrangement as a category that must not go in a record, and D5 says
where the viewer lives.

It fails. The rule states the crate boundary as its own exception, because the
boundary is the constraint and a compiler enforces it. D5 is not "the viewer
lives in `crates/cachette-view`". It is "the core does not depend on the
viewer, and it never will", which is a constraint on what the core may name.

**"D2 and ADR-0070 D1 are one decision."** Rejected, from the other side, in
the review of ADR-0070. Each has a violation the other permits, and the
conclusion is the same read from here.

**"D4 states an intent, because the record says the project will supersede
it."** It fails. D4 states what the code does today and names what would
replace it. That is the opposite of stating an intent as a fact, and the scope
rule asks for exactly this when a decision is known to be temporary.

**"D3 is a restatement of the float ban."** It fails. ADR-0002 D4 permits
floating point outside simulated state. D3 adds the direction: a value that
has been a float may not come back, in any form. The float ban alone would
permit a viewer to compute a camera position and hand it to the engine as a
tile address without rounding, and D3 is what forbids it.

**"The record should say what the viewer may spend."** It fails, and this is
the strongest attempted objection. A boundary that says nothing about cost
lets a viewer be correct and unaffordable. But that gap is exactly the subject
of ADR-0070, which says so in its own context, and filling it here would give
two records one claim.

## 6. What the review did not check

**Nothing was measured.** The record states no cost figure and none was taken.
The blocker governs every cost figure in this project.

**Nothing was checked against a second reader.** The delegation stands and a
review by a second person supersedes this one.

## 7. For the registers

- ADR-0067 D1 is amended and the record moves from `Draft` to `Accepted`.
- ADR-0070 moves from `Draft` to `Accepted`, because the review that
  recommended it named this record as the only thing holding it.
- Both files move to the accepted directory, and every citation of their draft
  paths moves with them. The citation check finds the ones that do not.
- A finding is recorded: a record can state a constraint over a subject wider
  than the constraint, and the words then forbid something the same record
  requires elsewhere.[^1]
- No blocker opened or closed.

## References

[^1]: Findings register, FND-056. `docs/FINDINGS.md`
[^R47A]: ADR Registry, the retired numbers. `docs/adrs/REGISTRY.md`
