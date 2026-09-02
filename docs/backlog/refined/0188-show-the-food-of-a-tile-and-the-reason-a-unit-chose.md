---
id: 0188
title: Show the food of a tile and the reason a unit chose
status: refined
created: 2026-09-02
implements: [ADR-0067 D1, ADR-0067 D2, ADR-0067 D3, ADR-0070 D1, ADR-0070 D2, ADR-0064 D2, ADR-0072 D4]
changes: []
creates: []
serves: [PRD-0005, PRD-0009]
blocked-by: []
---

## Why

**The viewer paints noise.** The colour of a tile comes from the stub value
field, which is a random walk that no other system reads or writes. The
resources, which the ground generates and the founding survey reads, are drawn
by nothing.[^1]

**The engine can explain a choice and nobody can ask.** A verb reports every
score, the value each option read, and the winner. No file outside the core
crate calls it. The product record asks that a watcher can ask why a unit did
what it did and get an answer from the engine. The answer exists and the
question cannot be put.[^2]

**The engine grew and the viewer did not.** The engine now holds households,
influence, descent, tile upgrades, ranked positions at a site, and a closed
loop from the survey to the production rate to the store to the ration to a
death. A watcher sees none of the loop, because the panel shows no store, no
rate and no ration.[^1]

## What the work does

1. The tile colour reads the food stock of the tile.
2. The panel reports the choice explanation for one unit.
3. The panel reports what the tile under the middle of the window holds, and
   what each of the first few sites produces, holds and owes.
4. The viewer keeps reading the world and never writes to it.[^3]

## Impact review

**Governed by.**

- **ADR-0067 D1.** The viewer reads the world through the public interface and
  writes nothing to it. Every new reader takes a shared reference.[^3]
- **ADR-0067 D2.** The engine holds no value that exists for the viewer. The
  work adds no engine field. Every value it draws is one the engine already
  holds for its own reasons.[^3]
- **ADR-0067 D3.** Floating point begins at the viewer boundary and never
  returns. The brightness arithmetic and the decimal formatting stay in the
  viewer.[^3]
- **ADR-0070 D1.** The panel adds no pass over the world. The tile section
  reads one address. The choice section reads one unit, which the drawing pass
  named while it painted. The site section stops at a fixed number of rows.[^5]
- **ADR-0070 D2.** A number the panel cannot afford is absent, never
  estimated. A tile the ground gave nothing of says so rather than printing a
  pair of zeroes. A unit the engine will not explain gets a sentence, not four
  zeroes. The site section states how many sites the world holds beside the
  few rows it read.[^5]
- **ADR-0064 D2.** A unit chooses by scoring a small fixed option set, and the
  engine recomputes the explanation on demand because it stores no score. The
  panel restates that answer and derives no part of it.[^6]
- **ADR-0072 D4.** A tile stock is generated, and only what was taken is
  stored. The colour and the panel read the stock through that reader, so a
  tile nobody touched costs a generation and a search that finds nothing.[^7]
- **ADR-0002 D4.** Rendering sits outside simulated state, so the float
  arithmetic here is free.[^8]

**Changes.** No record changes. The work contradicts none of them.

**Creates.** No record. The choice of which unit the panel names fails the
test for a record: a later contributor could choose otherwise, but changing it
later is cheap and the reasoning fits in a doc comment.[^9] It is recorded as
a decision instead.[^10]

**Blockers.** None.

**Precedent.** FND-119 records that a watcher cannot see a unit at the moment
a shortage ends it. The panel states a count instead.[^11] The recurring
defect rule warns against one value in two places, so the option names, the
condition names and the tile capacity all come from the engine's own tables
rather than from a second table in the viewer.[^12]

## The questions the proposal left open

**Which unit the explanation names, when the viewer has no cursor.** The unit
nearest the middle of the window. The middle of the window is the pointer, and
a watcher who wants another unit scrolls until that unit is in the middle. The
drawing pass fixes it while it paints, so the panel starts no pass to find
it.[^10]

**Whether the stub value keeps any reader.** After this work the viewer reads
it no more. Two readers are left: the level 1 cell summary, which averages it,
and the `forage` option, which scores that average. Items 0183 and 0184
replace both with food. When they land, nothing reads the field and the pass
that computes it can go. A separate item holds that work.[^13]

**Where the row goes, given that the panel cuts.** The panel is ordered by
what a watcher needs now. Every section that reports the world as it stands
comes first, and the sections that report the founding, which is history and
which cost twelve rows for each faction, go last. That is the cheapest of the
three candidates the panel-reach item lists, and this work is the measurement
of it.[^14] It is not a full answer: the ground legend and the cost rows still
fall below the edge, as they did before this work. The demonstration window is
also taller, which reaches further down the panel.

## Done when

- The colour of a tile rises with the food the tile still holds, and a test
  proves it by comparing tiles of one kind of ground.
- A gather darkens the tile it took from, and the test drives the engine.
- A tile whose stock nobody touched draws the same on two ticks.
- The panel names the drawn unit nearest the middle of the window, and a test
  finds the same unit by a full scan.
- The panel restates the engine's answer field for field, and a test compares
  it against the verb.
- The panel states what is left of a tile beside what the ground gave, and a
  test separates the two by gathering first.
- The panel states the store and the rate of each site it read, and how many
  sites the world holds.
- Every test above has a proven failure mode.
- `just check` passes.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: What a unit does in a tick, section 1. `docs/research/what-a-unit-does-in-a-tick.md`
[^2]: What a unit does in a tick, section 3.8. `docs/research/what-a-unit-does-in-a-tick.md`
[^3]: ADR-0067, the viewer reads the world and never writes to it. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
[^5]: ADR-0070, the head-up display reports what the drawing pass read. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
[^6]: ADR-0064, a unit chooses by scoring a small fixed option set. `docs/adrs/accepted/adr-0064-a-unit-chooses-by-scoring-a-small-fixed-option-set.md`
[^7]: ADR-0072, a tile stock is generated, and only what was taken is stored. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
[^8]: ADR-0002, simulated and aggregated state holds no floating point number. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^9]: Decision Record Scope, section 1. `.claude/rules/adr-scope.md`
[^10]: Decisions register, DEC-077. `docs/DECISIONS.md`
[^11]: Findings register, FND-119. `docs/FINDINGS.md`
[^12]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
[^13]: Backlog item 0194. `docs/backlog/proposed/0194-retire-the-tile-value-pass-when-nothing-reads-it.md`
[^14]: Backlog item 0133. `docs/backlog/proposed/0133-let-a-watcher-reach-a-panel-longer-than-the-window.md`
