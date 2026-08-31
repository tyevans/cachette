---
id: 0028
title: Close out the first renderable example
status: complete
created: 2026-08-30
implements: []
changes: []
creates: []
serves: [PRD-0002]
blocked-by: []
---

## Why

The project finished a long run of viewer and engine work. The product record
for the first renderable example states nine checkable statements and two cost
properties. Nothing had checked them against the code.

A product record that nobody checks becomes a record of an intent. The audit
answers whether the need is met, and it answers each statement separately.

## Impact review

**Governed by.** No decision record governs a review. Two records hold the
work under review. The viewer record binds the boundary between the viewer and
the engine.[^1] The head-up display record binds what the panel may report.[^2]

**Changes.** None. The audit found no record that needs superseding. It found
one record that decides against the product record on purpose, and the
register holds that as a choice rather than as a defect.[^3]

**Creates.** None. The three-condition test for a decision record rejects the
one decision the audit opened.[^4] A future contributor could reasonably
choose otherwise, so the first condition holds. The second does not: the
choice is between amending a product record and writing a snapshot record, and
neither is expensive to change later. The register row is the right home until
somebody needs the two rates apart.

**Blockers.** None opened and none closed. The measurement blocker stays open,
and the product record already states shapes rather than figures under it.[^5]

**Precedent.** The findings register warns that a fixture chosen for realism
hides the defect it should show, and that the only proof a test reaches a case
is to put the defect back and watch the test stay green.[^6] The audit
followed that: every verdict below rests on a run, not on a backlog item that
claimed the work was done.

**Serves.** PRD-0002.

## Done when

- Each of the nine statements has a verdict, with the evidence used.
- A statement that could not be checked is marked unverified.
- The record moves to `shipped/` only if the need is met.
- The registry agrees with the directory.
- Each real gap has a backlog item.
- The registers hold every correction, choice and blocker the audit opened.

## Outcome

**The reviewer did not ship the record.** Two of the nine statements are not
met, and one of the two cost properties is not met. The record stays in
`shaped/` and the registry row is unchanged.

The reviewer acted as the reviewer for this audit, which is the role the
product guide requires for any status past `Shaped`.

### The nine statements

| Statement | Verdict |
|---|---|
| One command opens a window, and the world appears in it | Met |
| The world has the shape the project chose | Met |
| Entities appear on the world, each on a tile | Met |
| The entities move while the developer watches, with no input | Met |
| No tile is over its capacity, and a viewer can see that this holds | Not met |
| The window shows the simulation, not a copy | Met |
| One seed gives one behaviour on every run | Met |
| The window never makes the engine wait, and reports what it drops | Not met |
| The engine gives the same results when no window is open | Met |

The two cost properties: what the viewer reads follows the window rather than
the world, which is met. The engine costs the same when a viewer is attached,
which is not met, for the same reason as the eighth statement.

### The two gaps

**Capacity.** The reviewer put four hundred soldiers on one tile of ordinary
ground and drew the frame. The spawn accepted every one, and the viewer
painted every one and said nothing. The panel's only occupancy row is a mean
over a region, and a mean is consistent with any maximum. The first half of
the statement is an open choice the register already holds.[^7] The second
half was recorded nowhere.

**Waiting.** The viewer record ties the drawing rate and the tick rate
together and states the consequence in its own text. The demonstration binary
also caps its frame rate. Nothing drops a frame and nothing reports a drop.
The two block counts the panel shows count empty spatial blocks. This is a
contradiction the viewer record chose knowingly, so it is a register row and
not a defect.[^3]

### What changed in the tree

The audit added one test. It runs one world that steps and draws on every
frame against one that steps alone, and it compares the state hashes after
twelve steps. The crate already proved that one frame changes nothing, which
is the weaker statement. The reviewer proved the new test can fail by giving
the drawn world one extra step, and it failed on the hash.

The audit repaired the outcome of the demonstration binary item, which named
an extent and a soldier count that later work made false.

### Registers

- Two findings. One says an outcome section decays like any other document.
  One says a comment can claim that one fact has one site while a second site
  exists.
- One decision row asks whether the viewer may make the engine wait.
- Five backlog items hold the gaps and the follow-up review.
- No blocker opened and none closed. No record changed status.

## References

[^1]: ADR-0067, the viewer reads the world and never writes to it. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
[^2]: ADR-0070, the head-up display reports what the drawing pass read. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
[^3]: Decisions register, DEC-022. `docs/DECISIONS.md`
[^4]: Decision Record Scope, section 1. `.claude/rules/adr-scope.md`
[^5]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^6]: Findings register, FND-051. `docs/FINDINGS.md`
[^7]: Decisions register, DEC-020. `docs/DECISIONS.md`
