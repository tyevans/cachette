---
id: 0540
title: Name the city that holds each wonder, and publish a rival wonder the reader sees
status: refined
created: 2026-09-10
implements: [ADR-0174 D1, ADR-0150 D1, ADR-0150 D2, ADR-0004 D1, ADR-0154 D3, ADR-0195 D7, ADR-0195 D9, ADR-0199 D1, ADR-0199 D2]
changes: []
creates: []
serves: [PRD-0057]
blocked-by: []
---

## Why

**A trained policy wins about a third of its three-faction games with one
tactic.** It puts every unit into one wonder from the first tick and does
nothing else. The built-in controllers grow large and let the wonder finish.
The project owner approved three linked changes. This item is the first half
of the second one: a policy can see where a wonder is.

**Nothing states where a wonder stands.** The observation publishes the wonder
work of a rival as one number for each faction, and it names no place. The
reader of the win path folds the work into one maximum for each faction and
drops the tile. The settlement token declares a wonder channel, and nothing
writes it.[^1]

A player that cannot find a rival wonder cannot stop it. A controller that
cannot name the city that holds a wonder cannot march on it. The second half
of this change aims the controller at that city, so it needs one lookup that
names it.

## Impact review

**Governed by.** ADR-0174 D1 states what a wonder is: an upgrade row that
carries a victory claim above zero.[^2] ADR-0150 D1 and D2 state which city a
tile belongs to, and the tie rule between two cities at one distance.[^3]
ADR-0004 D1 states that every walk has a stated order.[^4] ADR-0154 D3 and
ADR-0195 D7 state that a faction reads only what it observes, and that the fog
never masks its own state.[^5] [^6] ADR-0195 D9 states that the revision rises
when a value changes, and that a new signal takes its positions from the
reserve.[^6] ADR-0199 D1 and D2 state that a place is a cell of the egocentric
frame, and that a place narrows the region of a verb.[^7]

**Changes.** No record. The layout changes under the revision rule of
ADR-0195 D9, and that rule asks for no new record.

**Creates.** No record. The work closes one choice: which city a wonder
belongs to when two cities reach it. The choice is cheap to change later,
because one function states it, so it goes to the decisions register and not
to a record.[^8]

**Blockers.** BLK-160 asks whether a policy may read the win standing of a
rival it has never observed.[^9] This item publishes a rival wonder only where
the reader sees it, so it answers a narrower question and leaves that row open.
BLK-050 governs the wonder work, and every test states its own work.[^10]
BLK-007 governs every cost figure, and this item states none.[^11]

**Precedent.** FND-671 says a faction is not fogged from its own ground.[^12]
FND-689 says a raise of the version refuses every stored file, so a raise must
follow a changed position or a changed value.[^13]

## Done when

- One lookup states each wonder of the world: its tile, its work, its
  requirement, the faction that holds the tile, and the city the tile belongs
  to. It walks in ascending tile order.
- The reader of the win path folds that lookup, and a test proves that the
  fold gives the values of the walk it replaced.
- A test proves that the lookup names the nearest city of the holder, and that
  a tie goes to the lower slot.
- The settlement token of the reader carries the progress of the wonder on its
  ground.
- The rival token carries the progress of the rival wonder the reader sees,
  and the place value of the city that holds it.
- A test proves that a learner can find a rival wonder it sees and aim a
  campaign at it: the published place makes the campaign verb march on the
  city that holds the wonder.
- A test proves that a rival wonder the reader does not see publishes nothing.
- A test proves that the observation is one array at one, two and twelve
  threads.
- The observation version rises, the stored policies at the old version move
  to the archive, and the index says so.
- The whole check command runs green.

## Outcome

The code, the tests and the registers are on the branch of this item. The
gates have not run, because a worker does not run them. The dispatcher runs
them and then moves this item to `complete/`.

The lookup is `World::wonder_sites`. The reader of the win path folds it. The
settlement token and the rival token read it. The rival token gained two
channels, and the positions came from the reserve, so the length holds. The
version rose from 7 to 8, and the four stored policies at version 7 moved to
the archive.

A survey had said that a rival settlement gets a settlement token while the
reader sees it. The settlement set holds the reader's own settlements only. The
place of a rival wonder therefore went to the rival token. FND-764 records
it.[^14] DEC-283 records the tie rule.[^8]

## References

[^1]: The audit of the observation, section 3.4. `docs/research/what-a-policy-cannot-see.md`
[^2]: ADR-0174, a wonder is a win path and a stock total is not, decision D1. `docs/adrs/draft/adr-0174-a-wonder-is-a-win-path-and-a-stock-total-is-not.md`
[^3]: ADR-0150, held ground is the ground within reach of a city its faction owns, decisions D1 and D2. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
[^4]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^5]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decision D3. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
[^6]: ADR-0195, the observation of a faction is a fixed-width scale-free table, decisions D7 and D9. `docs/adrs/draft/adr-0195-the-observation-of-a-faction-is-a-fixed-width-scale-free-table.md`
[^7]: ADR-0199, a verb names a place by a cell of the egocentric frame the observation publishes, decisions D1 and D2. `docs/adrs/draft/adr-0199-a-verb-names-a-place-by-a-cell-of-the-egocentric-frame.md`
[^8]: Decisions register, DEC-283. `docs/DECISIONS.md`
[^9]: Blockers register, BLK-160. `docs/BLOCKERS.md`
[^10]: Blockers register, BLK-050. `docs/BLOCKERS.md`
[^11]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^12]: Findings register, FND-671. `docs/FINDINGS.md`
[^13]: Findings register, FND-689. `docs/FINDINGS.md`
[^14]: Findings register, FND-764. `docs/FINDINGS.md`
