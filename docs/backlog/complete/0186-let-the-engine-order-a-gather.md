---
id: 0186
title: Let the engine order a gather
status: complete
created: 2026-09-02
implements: [ADR-0002 D1, ADR-0004 D1, ADR-0022 D1, ADR-0064 D1, ADR-0064 D4, ADR-0064 D6, ADR-0072 D4, ADR-0072 D5, ADR-0073 D1, ADR-0073 D3, ADR-0073 D4, ADR-0080 D1]
changes: []
creates: []
serves: [PRD-0007, PRD-0009]
blocked-by: []
---

## Why

**No unit in the demonstration ever gathers.** The gather order is a
control-plane verb, and the engine issues none. The resource module, the
gather ledger, the depletion set and the recovery pass are all correct and all
idle. The option named `forage` is named for an act that never happens.[^1]

The testing rule already names this shape. Ask who is obligated to invoke the
thing: the user of the library, or the engine. If the engine, the test starts
at the engine.[^2] Nothing today starts at the engine.

**This closes a feedback loop.** Food falls where the crowd stands. The cell
summary falls with it. The exit direction of item 0185 then turns the crowd
away.[^3] Without this item, the field of 0185 produces one rush in one
direction, and nothing in the world turns it back. Schedule the two together,
and take this one immediately after 0185 if they must be split.

## What the work does

1. The choice pass writes a gather order beside the intent. A unit whose
   option is `forage` holds an order for food. A unit whose option is anything
   else holds no order.
2. The gather resolve, the depletion and the recovery then run in the
   demonstration, on the path that the step already holds.

## The answers this item takes, stated plainly

**The choice pass is the only engine writer of the order, and it writes the
order in the same write as the intent.** One pass writes both, for the same
units, on the same frame. A second stage that derived the order from the
option would be a second writer of one column, and that is the shape this
project records as a recurring defect.[^4]

**A control-plane order holds until the unit next chooses.** The engine writes
the order only for a unit whose cell chooses on that frame, which is the same
set the intent pass already writes.[^5] A caller that orders a gather from the
control plane therefore keeps that order for the frames until the cell of that
unit comes round. The precedence is stated, and a test asserts it. **The verb
stays.** An accepted record states that the command names a unit and a kind
and that the step resolves the whole set, so removing the verb would
contradict it.[^6]

**The choice pass clears the order.** A unit that chooses an option other than
`forage` holds no order after that choice. A unit that holds no intent at all
holds no order either.

**A `forage` option names food.** The option is driven by what the unit lacks,
and what a unit lacks is the ration that the consumption pass draws.[^7] Food
is the only kind that answers it. Wood and stone answer no need today, and a
world that holds three kinds does not make three options.

**The tile test stays in the resolve.** The choice pass runs before movement,
and the resolve runs after it, so only the resolve knows the tile the unit
stands on at the end of the frame.[^8] A unit that forages on a tile with no
food takes nothing and produces no event, which is what the resolve already
does when a deposit is empty.

## Impact review

**Governed by.**

- **ADR-0073 D1.** The command names a unit and a kind, and the step resolves
  the whole set. This item adds an engine writer of that command. It does not
  change the command, the resolve or the verb.
- **ADR-0073 D3.** The resolve runs after the barrier of its frame, because it
  reads where each unit stands. This item writes the order before movement and
  changes no stage order.
- **ADR-0073 D4.** What a unit carries is a column of the unit. This item adds
  no side table.
- **ADR-0064 D1, D4 and D6.** A unit scores a fixed option set, the choice
  runs at an interval keyed on the level 1 cell, and it writes nothing above
  level 0. The order is a column of the unit, so D6 still holds.
- **ADR-0072 D4 and D5.** The engine stores what was taken and nothing else,
  and conservation is a world invariant checked for each kind. This item makes
  that invariant carry real quantities for the first time.
- **ADR-0080 D1.** A depleted deposit recovers by ageing the stored take. The
  recovery pass runs today over an empty set. After this item it runs over a
  set that grows, which is the behaviour the record describes.
- **ADR-0004 D1.** Iteration order is explicit. The order column is written by
  slot, and the resolve already sorts the intents by the deposit and then by
  the identity of the unit.
- **ADR-0002 D1.** No floating point. An amount of a resource is an exact
  whole number.

**Changes.** No record changes. This item contradicts no accepted record.

**Creates.** No record. A contributor could reasonably put the write in a
stage of its own, and the change would be cheap to make later, so the
constraint does not earn a record.[^9] The precedence between the two writers
is stated in this item and asserted by a test.

**Blockers.** None. BLK-007 governs any cost figure, and this item states
none.[^10]

**Open choices this item carries.** DEC-074 asks how the project finds a value
that nothing reads, and its recommendation applies here: name the stage that
reads the order to decide something, and test that changing the order changes
the decision.[^11]

**Precedent.** FND-181 records that the rules against inert work look for an
absent caller, and that this defect has one.[^12] FND-191 records that the
engine writes the number of the food commodity wherever it needs one, and this
item adds a second map from an option to a resource kind. Declare that map
once.[^13]

## Done when

- A unit whose option is `forage` holds a gather order for food after the
  choice pass, and the demonstration produces gather events without any caller
  ordering a gather.
- A unit whose option is not `forage` holds no gather order after it chooses.
  A test asserts both halves in one run.
- A unit whose cell does not choose on a frame keeps the order it held. A test
  asserts it.
- A control-plane order survives until the cell of that unit next chooses, and
  the choice then replaces it. A test asserts both halves, so the precedence
  between the two writers is checked and not described.
- The test drives the step. It does not call the resolve directly.[^2]
- A unit that forages on a tile that holds no food takes nothing and produces
  no event.
- The depletion set grows as units gather, and the recovery pass returns a
  part of a take. A test drives several ticks and asserts that a deposit falls
  and then rises.
- The conservation invariant holds across the whole run, for each kind.
- The gather log is identical, byte for byte, at 1, 2 and 12 threads.
- **A test pins the option column to one value and fails.** A suite that stays
  green proves that nothing reads the column.[^11] [^14]
- The fixture is built for this test and is not copied from the demonstration
  world. It holds a tile with food, a tile with none, and a unit whose option
  is `forage`. The commit body says how that was checked: the engine write was
  removed, and each gather test was watched to fail.[^14]
- The two determinism tests pass, and `just check` runs green.

## Outcome

The choice pass writes the gather order beside the intent, in the same write.
A unit whose option is `forage` holds an order for food. A unit that chose any
other option, and a unit that chose nothing, holds no order. The demonstration
world of a test now produces gather events with no caller ordering a gather.

**The map from an option to a resource kind is declared once.** The option row
carries it, beside the field the option reads. No second site names the kind of
a gathering option, so nothing can disagree with it.

**The precedence is asserted and not described.** A control-plane order
survives every frame on which the cell of the unit does not choose, and the
choice replaces it on the frame that it does. One test drives both halves.

**An existing probe fixture had to absorb the new writer, and FND-211 records
it.** The gather probe set the choice interval to every tick and then ordered a
gather from outside, so the choice replaced that order before the resolve read
it and the contest disappeared. The fixture now puts the choice far enough
apart that no cell of a gatherer chooses on the frame under test, and it
asserts that.

**Registers.** FND-209 and FND-211 were added. No blocker opened or closed.

**Evidence.** Two defects were put back separately, and the source was restored
after each. Removing the engine write of the gather order failed five of the
six gather tests. Setting the map from the option to the resource kind to
nothing failed four of them.

**The loop is inert in the demonstration.** Every unit there is fed, so the
`forage` row scores zero and the engine orders no gather at any tick that was
measured. The engine tests drive a hungry unit and it forages, gathers, and
works a deposit down. FND-209 holds the measurement, and item 0216 holds the
repair.

## References

[^1]: What a unit does in a tick, section 3.6. `docs/research/what-a-unit-does-in-a-tick.md`
[^2]: Testing Rules, section 5. `.claude/rules/testing.md`
[^3]: Backlog item 0185, steer a step by the option the unit chose. `docs/backlog/complete/0185-steer-a-step-by-the-option-the-unit-chose.md`
[^4]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
[^5]: ADR-0064, a unit chooses by scoring a small fixed option set, decision D4. `docs/adrs/accepted/adr-0064-a-unit-chooses-by-scoring-a-small-fixed-option-set.md`
[^6]: ADR-0073, gathering is admitted by sort-then-admit against the tile, decision D1. `docs/adrs/accepted/adr-0073-gathering-is-admitted-by-sort-then-admit-against-the-tile.md`
[^7]: ADR-0063, a need is a rate with a threshold, and crossing it is a fact, decision D2. `docs/adrs/accepted/adr-0063-a-need-is-a-rate-with-a-threshold-and-crossing-it-is-a-fact.md`
[^8]: ADR-0073, gathering is admitted by sort-then-admit against the tile, decision D3. `docs/adrs/accepted/adr-0073-gathering-is-admitted-by-sort-then-admit-against-the-tile.md`
[^9]: Decision Record Scope, section 1. `.claude/rules/adr-scope.md`
[^10]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^11]: Decisions register, DEC-074. `docs/DECISIONS.md`
[^12]: Findings register, FND-181. `docs/FINDINGS.md`
[^13]: Findings register, FND-191. `docs/FINDINGS.md`
[^14]: Testing Rules, sections 2 and 2a. `.claude/rules/testing.md`
