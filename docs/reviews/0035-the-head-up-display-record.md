# Review 0035: The head-up display record

## What was reviewed

| Item | Value |
|---|---|
| `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md` | Status `Draft` |
| Branch | `docs/review-adr-0070-for-acceptance` |
| Code read | `crates/cachette-view/src/hud.rs`, `crates/cachette-view/src/paint.rs`, `crates/cachette-view/src/metrics.rs`, `crates/cachette-view/src/main.rs` |

The reviewer did not write ADR-0070 and did not write the code that first
implemented it.

**The reviewer wrote two later sections of the panel**, the ground legend and
the region readout, under the record as it stands. That is a weaker
independence than a second reader gives, and it cuts one way: a reviewer who
has worked under a record is more likely to accept it. Section 4 states what
was attempted against that bias.

## Verdict

| Record | Verdict |
|---|---|
| ADR-0070 | Sound, unchanged. Do not accept it yet. |

The record states two constraints, both of which a reviewer can find a
violation of. The code honours both, name by name. Nothing in the body decays.
No amendment is recommended.

**It stays a draft because its dependency is a draft.** ADR-0070 rests on
ADR-0067 and cites three of its decisions as the boundaries it extends. A
draft is not binding, and a record that nothing may cite as binding cannot
carry an accepted record on top of it. Section 5 holds the argument and what
would close it.

This is the finding the review was asked to produce. It is a sequencing
defect, not a defect in the record, and the fix is to review the dependency
rather than to change anything here.

## 1. The record against the code, decision by decision

### D1. The panel adds no pass over the world

The decision names three sources and says there is no fourth. Each number the
panel states was traced to one of them.

**A value the engine gives at once.** The tick is `world.tick()`. The live
population is `world.soldiers().len()`, which returns a stored count and reads
no unit. The extent and the faction count come from the configuration. The
region readout is `world.summary_covering(address)`, which converts an address
to an index, an index to a block, and reads one array element.

**A value the viewer computes for itself.** The centre tile, the zoom, and the
extent shown all come from the camera and the canvas. None of them touches the
world.

**A count the drawing pass produced.** The tiles painted, the units painted,
the blocks read, the blocks skipped, the units by faction and the tiles by
kind are all incremented on the path that paints the thing they count.

There is no loop over the world anywhere in the reporting code. Every loop in
`hud.rs` walks the panel's own arrays: six faction slots, five ground kinds,
and the list of lines the panel holds. The cost of the panel follows the
window, which is what the decision asks.

### D2. A number the panel cannot afford is absent, never estimated

The panel shows no world-wide count that would need a scan. The counts it
shows are labelled by what they count: `FACTIONS IN THE WINDOW`, `GROUND IN
THE WINDOW`, `REGION UNDER THE CROSSHAIR`. The world section shows the tick,
the extent and the live population, and every one of those is a value the
engine gives at once.

No sampling, no extrapolation and no carried-over figure appears anywhere. A
reading the engine cannot give prints a dash rather than a number, which is
the strongest form of what this decision asks.

## 2. ADR-0070 D1 against ADR-0067 D2

The item required this check, and the two are distinct.

ADR-0067 D2 constrains the **engine**: it holds no value that exists for the
viewer. ADR-0070 D1 constrains the **viewer**: it spends no pass over the
world to work a value out.

Each has a violation the other permits. A running census kept on the world,
updated as units spawn, violates ADR-0067 D2 and satisfies ADR-0070 D1,
because the panel would then read it at once. A loop over the population
inside the reporting code violates ADR-0070 D1 and satisfies ADR-0067 D2,
because the engine gained no field.

The record's own rejected-alternatives section names both, and rejects them
under different records. That is the clearest evidence that the two decisions
are separable, and it is in the record already.

**Recommendation: keep them separate.** Merging them would produce one
decision with two violation shapes, which is the shape the scope rule warns
against.

## 3. The record against the scope rule

**One claim, or two separable ones.** D1 says where a number may come from.
D2 says what to do when no source can afford it. A panel can honour D1 and
still estimate: scale the window count by the ratio of the world to the
window, and no pass over the world is added. That violates D2 alone, so D2 is
not a corollary of D1.

**No volatile material.** The body holds no count, no file table, no measured
figure and no version. It names the unit count at the target scale by footnote
into the reference table rather than quoting it, which is what section 4.1
requires.

**No module arrangement.** The record names no file. It names the panel, the
drawing pass and the engine, which are roles rather than locations.

**The title states the claim.** "The head-up display reports what the drawing
pass read" is a claim a reader can test. It is not a topic.

**Citation count.** The record is cited by the viewer crate and by four
backlog items. It is not one of the uncited records the scope rule asks a
reviewer to question.

## 4. Objections attempted

A review that lists no attempted objection did not happen. Five were tried and
all five failed.

**"The region readout is a fourth source."** The panel now states a summary of
a block of tiles, and a block reaches beyond the window. The number is
therefore about part of the world that the drawing pass did not paint, which
looks like the thing D1 forbids.

It fails. The consequences section already answers it: "Showing one needs a
new engine reader, or a structure the engine already maintains. It never needs
a new engine field." Level 1 is a structure the engine maintains for its own
reasons, and the panel reads one cell of it at once. The record anticipated a
case that did not exist when it was written, and the case arrived and fitted.
That is the strongest evidence found for accepting it.

**"D1 and ADR-0067 D2 are one decision."** Rejected in section 2. Each has a
violation the other permits.

**"D2 is a corollary of D1."** Rejected in section 3. An estimate scaled from
the window adds no pass and violates D2 alone.

**"The record describes an intent."** It does not. The three sources are in
the code, the counters are fields of the canvas, and the labels that D2
requires are the headings the panel prints. A test reads the same lines the
panel prints and fails when a value is cut.

**"The record is too long for what it decides."** It runs about 1100 words
against a reference median near 1300, and about half of it is the context that
explains why the cheap wrong answer is attractive. That context is the part
that stays true, and it is what a future contributor reaching for the scan
needs to read.

## 5. Why it stays a draft

ADR-0070's context opens by naming the boundaries a separate record already
fixes, and its rejected alternatives reject one option outright by citing that
record. Three of ADR-0067's decisions carry it: the viewer holds a shared
reference and never writes to the world, the engine holds no field that exists
because something draws it, and the demonstration binary steps and draws in
one loop.

ADR-0067 is a draft. The project's own words are that an author may set
`Draft`, only a reviewer may set `Accepted`, and a draft binds nothing. An
accepted record whose foundation binds nothing is a record that can be made
false by an edit to a file nobody is required to leave alone.

The failure mode is the one the findings register already holds twice: a
record correct on the day it was written that states something false the
moment the thing under it moves.[^3]

**What would close this.** A review of ADR-0067 against the viewer crate, by
an agent that did not write it. If ADR-0067 is accepted, ADR-0070 follows
without any change to its text, because this review found nothing to change.
An item carries that work.[^4]

**The alternative was rejected.** Accepting ADR-0070 now with a note that its
dependency is a draft would put the condition in prose that nothing checks. The
registry holds status, and a status that means "accepted, but" is a status the
vocabulary does not have.

## 6. What the review did not check

**The record's cost claims are not measured.** The record argues that a scan
of the population would cost more than the picture. Nobody has measured either
on the target platform, and the blocker governs every cost figure in this
project.[^1] The record does not state a figure, so it does not depend on one.

**Nothing was checked against a second reader.** The delegation stands and a
review by a second person supersedes this one.[^2]

## 7. For the registers

- The registry row for 0070 stays `Draft`. The review recommends acceptance
  and the dependency holds it.
- No finding. The review corrected nothing that the project believed. The
  sequencing defect is a condition on one record, not a correction to
  something the project held true.
- No blocker opened or closed.
- Backlog item 0047 carries the review of ADR-0067.

## References

[^1]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^2]: ADR Registry, who reviews. `docs/adrs/REGISTRY.md`
[^3]: Findings register, FND-042 and FND-055. `docs/FINDINGS.md`
[^4]: Backlog item 0047. `docs/backlog/complete/0047-review-adr-0067-for-acceptance.md`
