---
id: 0120
title: Draw what a unit suffers and where each faction founded
status: refined
created: 2026-08-31
implements: [ADR-0067 D1, ADR-0067 D2, ADR-0067 D3, ADR-0070 D1, ADR-0070 D2, ADR-0063 D3, ADR-0076 D2]
changes: []
creates: []
serves: [PRD-0003]
blocked-by: [0057, 0094]
---

## Why

The engine gains state that nobody can see. A watcher opens the window and
reads terrain, soldiers, tile ownership and a panel of counts. Every rule
added since then is invisible.

Two items now in flight add state of exactly this kind. One gives a unit a
condition that gets worse under a shortage and ends the unit when it lasts too
long. The other founds one group for each faction at a minimum distance. Both
state their result as a value a watcher reads through the public interface.
Neither draws anything.

The product record asks for a world worth looking at, and its index note
already says the viewer trails the engine.[^1] No open item answered that
note. Two items name the record as what they serve, and neither draws: one is
a world-build cost item and the other is a terrain regression test.[^2] [^3]

A finding records the shape.[^4] The word "watcher" covers two interfaces, the
library and the window, and an item can satisfy its whole acceptance list
against the first while adding nothing to the second.

## What the work does

1. The viewer draws the condition of a unit, so a watcher sees a shortage
   spread through a group and sees which units it takes.
2. The panel says how many units the shortage holds and how many it has ended.
3. The viewer marks each founded place, so a watcher sees the factions apart
   from each other and can judge the distance between them.
4. The panel names each faction that founded and each faction that failed to
   found.

## The questions this item must answer before it is refined

**Which interface supplies each value.** The engine holds no value that exists
because something draws it. Every value here must already be readable, or the
item must say why the engine gains a reader rather than the viewer gaining a
pass.

**Whether the condition is a colour, a mark or a panel row.** A condition on
every unit competes with the faction colour a unit already carries. The
viewer holds one faction colour table and must not gain a second.

**What the founding marks cost after the founding frame.** A founded place is
history, not state that changes. The panel reads counts of the pass that just
ran, and a mark that persists is a different thing from a count.

**Whether a picture test can hold either of these.** A stored picture of the
panel cannot read a clock, and the condition changes every tick.

## What this item does not do

It adds no rule to the engine, and it changes no simulated value. It does not
widen into the resource display, which a later item owns.[^5] It writes no
decision record; the boundary between the engine and the viewer is already
recorded, and this item obeys that record rather than restating it.

## The answers

### Which interface supplies each value

Every value already has a public reader. The engine gains nothing.[^6]

- **The condition of one unit.** The world reads it back by identity and
  returns the name of the condition rather than the accumulator. The rule
  carries the bound, so the viewer never compares a number against a rule of
  its own.[^7]
- **How many drawn units are short, and how many are starved.** The drawing
  pass counts them while it paints them. The panel says what the pass read,
  and it starts no pass of its own.[^8]
- **How many units a shortage ended.** The world holds the log of the scan
  that just ran. The panel states its length. That is a count of the world,
  not of the window, and the label says so.[^9]
- **Where each faction founded, and which faction was refused.** The run
  returns one outcome for each faction. The caller that founded the run owns
  those outcomes and lends them to the frame. The world keeps no copy, because
  a field that existed to be drawn is the violation the boundary record
  names.[^10]

The viewer therefore gains a pass and the engine gains no reader.

### The condition is a mark, not a colour

A unit keeps the faction colour it already carries, from the one table the
viewer owns.[^11] The condition is drawn over that disc in one mark colour.

A unit that is short takes a dot at half the radius. A unit that a shortage
has taken to the bound takes the whole disc. One colour, two areas: the viewer
gains no second table, and a watcher still reads the faction of every unit
that is not about to end.

At a tile size of a few pixels the two marks reach the same pixel, because
half of a radius of one is one. The picture cannot separate the two conditions
there. The panel counts them apart, and the item states this rather than
leaving a watcher to find it.

### What the founding marks cost after the founding frame

A founded place is history. Nothing in the world says that a place was
founded, and this item adds nothing that does.

The mark comes from the outcomes the caller holds, one for each faction, on a
pass that visits that list and nothing else. The cost follows the faction
count. It does not follow the tiles, the units or the ticks, and it does not
change after the founding frame, because the list does not change.

The mark is not a tile read. The pass asks the camera where a place sits and
paints there. A place outside the window paints nothing.

### A picture test can hold both

A picture test cannot read a clock, and two rows of the panel divide by a wall
clock span. The viewer already answers that: a caller states a fixed span, and
only a test may state one.[^12]

The condition changes every tick, and a tick is not a clock. One seed, one
need rule and one count of steps give one condition for every unit, because
the engine gives one answer for one input.[^13] A picture test pins all three.

## Impact review

**Governed by.**

- ADR-0067 D1. Every new read takes a shared reference to the world. A test
  asserts that the state hash does not move over a frame.[^14]
- ADR-0067 D2. The engine gains no field, no method and no colour.[^6]
- ADR-0067 D3. The mark geometry is floating point, and no scaled or formatted
  value returns to the engine.[^15]
- ADR-0070 D1. The condition of a unit is read at the unit the pass is
  painting, inside the loop that already runs. The panel starts no pass.[^8]
- ADR-0070 D2. Two of the new counts are of the window and one is of the
  world. Each label says which, so a reader never learns it from the
  heading.[^9]
- ADR-0063 D3. The bound belongs to the rule of the world. The viewer reads
  the named condition and holds no threshold.[^7]
- ADR-0076 D2. A run reports one outcome for each faction, and a refusal
  stands beside a founding. The panel states both, so a watcher sees the
  faction that failed rather than a shorter list.[^16]

**Changes.** No record changes.

**Creates.** No record. The claim a record would carry is that the viewer
draws history the caller holds, and the boundary record already carries it in
a stronger form.[^6] The choice between a mark and a colour fails the second
condition of the scope rule, because reversing it costs one function.[^17]

**Blockers.**

- BLK-007 governs every cost figure, so this item states none. The cost of the
  condition follows the units the window paints. The cost of the founding
  marks follows the faction count.
- No blocker governs the shortage rule or the founding distance. The engine
  applies both already.

**Precedent.**

- FND-100 records that "watcher" names two interfaces, and that an item can
  satisfy its whole list against the library while the window shows
  nothing.[^4] Every line below names the window or the panel.
- FND-107 records that a four-faction world put its foundings tens of tiles
  apart by chance, and that a distance test stayed green with the whole rule
  removed.[^18] The fixture here is crowded on purpose. The world is small
  enough that the minimum distance refuses at least one faction.
- FND-051 records that a fixture chosen for realism hides the defect it must
  show.[^19] The shortage fixture feeds half its sites and starves the other
  half, and it asserts that it produced both.
- FND-061 records that a fixture asserts over the outcome and not over the
  inputs.[^20] Each test reads the conditions back before it draws.
- FND-093 records that a test which passes because an earlier filter excluded
  the bad case is a guard.[^21] The report says which of these is one.

**Serves.** PRD-0003. The record asks for a world worth looking at, and its
index note says that the viewer trails the engine. This item is the first that
answers that note.

**Conflict surface.** `crates/cachette-view/src/paint.rs` at the unit loop and
at the colour constants. `crates/cachette-view/src/hud.rs` at the readout and
at the line list. `crates/cachette-view/src/lib.rs` and
`crates/cachette-view/src/main.rs` at the frame call, which takes a list of
outcomes in place of a list of foundings. No engine file changes.

**It cannot run beside item 0106**, which draws over the same units. It cannot
run beside any item that changes the frame call in this crate.

## Done when

- A watcher who opens the window sees a unit that a shortage holds, and sees a
  unit that the shortage has taken to the bound. The two marks differ. A
  picture test reads the pixels of a named unit for each of the three
  conditions.
- A unit that is fed carries no mark, and its pixels hold the faction colour
  alone.
- The panel says how many drawn units are short and how many are starved, and
  both labels name the window.
- The panel says how many units the last scan ended, and that label names the
  world.
- The window marks each place a faction founded, in that faction's colour from
  the one table the viewer owns. A picture test reads the mark at each founded
  place and finds the colour of the faction that founded there.
- The panel names each faction that founded and each faction that was refused,
  and gives the reason for the refusal.
- The fixture is crowded. The world is small enough that the minimum distance
  refuses at least one faction, and the fixture asserts that it did.
- A whole-tree search finds no second faction colour table and no second
  reader that maps a faction to a colour. The search command is in the commit
  body.
- The engine gains no field, no method and no colour. No file under
  `crates/cachette-core/src/` changes.
- A test asserts that a drawn frame leaves the world where it found it, by the
  state hash.
- Each rule above is put back as a defect, one at a time, and the test that
  defends it is watched failing. The report says which test caught which
  defect, and which tests stayed green.
- The two determinism tests are unaffected, and the commit body says so.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Product priority index. `docs/product/PRIORITY.md`
[^2]: Backlog item 0112. `docs/backlog/proposed/0112-build-a-world-without-a-pass-over-every-tile.md`
[^3]: Backlog item 0034. `docs/backlog/proposed/0034-measure-the-generated-terrain-against-a-stored-one.md`
[^4]: Findings register, FND-100. `docs/FINDINGS.md`
[^5]: Backlog item 0106. `docs/backlog/proposed/0106-show-a-watcher-what-is-moving-and-where-it-goes.md`
[^6]: ADR-0067, the viewer reads the world and never writes to it, decision D2. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
[^7]: ADR-0063, a need is a rate with a threshold, and crossing it is a fact, decision D3. `docs/adrs/accepted/adr-0063-a-need-is-a-rate-with-a-threshold-and-crossing-it-is-a-fact.md`
[^8]: ADR-0070, the head-up display reports what the drawing pass read, decision D1. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
[^9]: ADR-0070, the head-up display reports what the drawing pass read, decision D2. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
[^10]: ADR-0076, a founding keeps a fixed distance from the foundings before it. `docs/adrs/draft/adr-0076-a-founding-keeps-a-fixed-distance-from-the-foundings-before-it.md`
[^11]: Backlog item 0085. `docs/backlog/complete/0085-show-a-watcher-who-holds-the-ground.md`
[^12]: The measurement module of the viewer. `crates/cachette-view/src/metrics.rs`
[^13]: ADR-0001, one binary gives one answer at any thread count. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^14]: ADR-0067, the viewer reads the world and never writes to it, decision D1. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
[^15]: ADR-0067, the viewer reads the world and never writes to it, decision D3. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
[^16]: ADR-0076, a founding keeps a fixed distance from the foundings before it, decision D2. `docs/adrs/draft/adr-0076-a-founding-keeps-a-fixed-distance-from-the-foundings-before-it.md`
[^17]: Decision Record Scope, section 1. `.claude/rules/adr-scope.md`
[^18]: Findings register, FND-107. `docs/FINDINGS.md`
[^19]: Findings register, FND-051. `docs/FINDINGS.md`
[^20]: Findings register, FND-061. `docs/FINDINGS.md`
[^21]: Findings register, FND-093. `docs/FINDINGS.md`
