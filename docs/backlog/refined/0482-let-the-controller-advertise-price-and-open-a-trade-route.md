---
id: 0482
title: Let the controller advertise, price and open a trade route
status: refined
created: 2026-09-05
implements: [ADR-0144, ADR-0147, ADR-0149, ADR-0146]
changes: []
creates: []
serves: [PRD-0050, PRD-0051]
blocked-by: [BLK-007, BLK-036, BLK-050]
---

## Why

**A faction that holds a board never writes to it.** Item 0477 gives each
faction a board and lets a contract carry land or a relation, and it stops at
the engine. Nothing inside the step reads a board or posts to one. This item is
the controller half of pass 6 of the living world game layer.[^1]

The controller writes its rows from its site economies on a schedule. It
offers only where its own surplus meets a posted want on another board.
Surplus is the site store above a mark in the balance register. It counters at
the integer midpoint between the two asks, accepts when the counter meets its
own ask, and never draws for a price. A trade route is a contract plus
carriers: units with carry capacity above zero get a home at one site and are
sent to the other, and the delivery pass does the rest.[^1]

**This item was split from item 0477 on 5 September 2026.** It waits for the
controller stage of item 0472 and the relation of item 0474, because the
controller reads the band before it trades and the midpoint rule reads the
weight vector.

## Impact review

**Governed by.**

- **ADR-0144 D2.** Every command passes a verb a Python caller can call. The
  board write goes through the advertisement verb, the negotiation through the
  offer, counter, accept and refuse verbs, and the carrier assignment through
  the home verb and the send verb. This work adds no verb for the controller
  alone.
- **ADR-0144 D4.** The stage makes a fixed number of draws for one faction on
  one tick. This work adds two draw indexes past the campaign draw: one that
  decides whether the faction takes its negotiation step, and one that breaks a
  tie between two equally lacked goods. Neither is a convergence test and
  neither is a time budget.
- **ADR-0144 D5.** Every new command enters the one plan list, and the list is
  sorted by faction and then by draw index before any command applies. A later
  faction therefore reads the board that an earlier faction wrote on the same
  tick, and that order is a property of the data.
- **ADR-0147.** A side of a contract is a tagged kind. The controller states a
  resource on each side, so the carriers deliver both sides and no land side
  and no relation side is opened by the controller.
- **ADR-0149 D3.** The controller replaces the whole board of its faction in
  one write, and it never writes one row.
- **ADR-0149 D5.** The record says that no controller writes a board yet. This
  work makes that sentence false, and the record must lose it.
- **ADR-0146 D4.** A pair in the war band refuses an offer. The controller
  reads the same predicate the verb reads, so it never asks for a refusal it
  can see coming.

**Changes.** ADR-0149 D5 holds one paragraph that says no controller writes a
board. The paragraph states a fact about the code, and this work makes it
false. **The record is a draft, so it is edited in place.**[^3] The decision
itself does not change: the controller still writes only through the verb.

**Creates.** No record. The scope rule gives three conditions and a decision
needs a record when all three hold.[^4] The price rule is the only candidate.
The first condition holds: a contributor could price by a draw. The third
condition holds only weakly, because the reasoning is in the balance register
beside the values. The second condition fails: the midpoint rule is a
provisional shape that pass 10 measures, and a record would buy the right to
refuse a change that the balance harness is expected to make. The rows go in
the balance register instead.

**Blockers.**

- **BLK-007.** Every cost figure is derived. The advertisement period, the
  surplus mark, the carrier count and the contract term are written as
  provisional values with a derivation, and pass 10 measures them.
- **BLK-036.** Whether an upgrade changes hands with the ground is open. The
  controller therefore opens no land side at all, so it never meets the
  refusal.
- **BLK-050.** The rules of the downstream game are not written down, so the
  shape of the price rule is provisional and its row names the blocker.

**Precedent.** Recurring defect shape 3 is inert code.[^5] The board verb, the
consideration verbs and the delivery pass all exist and no engine caller
reaches them. The end-to-end test of this item is what stops that shape: it
drives the step and asserts that a contract bound between two controllers and
that a carrier moved a quantity.

**Product record.** PRD-0050 and PRD-0051 state the need. No figure is taken
from either.

## Done when

- The controller rewrites the whole board of its faction on a schedule, from
  the stores of the sites of that faction, in commodity order and site slot
  order.
- The controller reads the boards of the other factions in faction order,
  matches its own want against their offer, and opens at most one negotiation
  step for one faction on one tick.
- The controller counters at the integer midpoint of the two asking
  quantities, accepts when the counter meets its own ask, and refuses
  otherwise. No draw decides a price.
- The controller opens no negotiation with a pair in the war band, and none
  with a pair that already holds a live one.
- A bound contract with a resource side takes carriers: the lowest identities
  among the idle units of the faction whose type carries, homed at the site of
  their own faction and sent to the site of the other party. They are released
  when the contract settles or fails.
- The census holds `boards_written`, `offers_made`, `contracts_bound` and
  `carriers_assigned`, and the demonstration prints a line when a contract
  binds and when one completes.
- Each new draw has one test for each field of its key.[^6]
- A test proves that a board is rewritten only on its schedule tick, that a
  site with an empty store posts a want and a site at capacity posts an offer,
  that no offer crosses a war pair, and that no second negotiation opens with
  one pair.
- An end-to-end test drives the step and proves that a contract binds between
  two controllers and that a carrier delivers, inside a bounded tick count.
- The determinism tests pass at 1, 2 and 12 threads with a trade in flight.
- Each defect is put back and the test watched go red.[^7]
- The balance register holds a row with a filled derivation for the
  advertisement schedule, the surplus mark, the board size, the carrier count
  and the contract term.
- ADR-0149 D5 no longer says that no controller writes a board.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Design: the living world game layer, section 4. `docs/superpowers/specs/2026-09-05-living-world-game-layer-design.md`
[^2]: Balance register. `docs/reference/balance.md`
[^3]: Definition of Done, section 4. `.agents/rules/definition-of-done.md`
[^4]: Decision Record Scope, section 1. `.agents/rules/adr-scope.md`
[^5]: Recurring Defect Shapes, shape 3. `.agents/rules/recurring-defects.md`
[^6]: Testing Rules, section 2. `.agents/rules/testing.md`
[^7]: Testing Rules, section 2a. `.agents/rules/testing.md`
