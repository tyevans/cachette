---
id: 0420
title: Let two factions bind each other to a future delivery
status: complete
created: 2026-09-03
implements: [ADR-0120 D1, ADR-0120 D2, ADR-0120 D3, ADR-0120 D4, ADR-0120 D5, ADR-0121 D1, ADR-0121 D2, ADR-0121 D3, ADR-0121 D4, ADR-0122 D1, ADR-0122 D2, ADR-0122 D3, ADR-0122 D4]
changes: []
creates: [ADR-0120, ADR-0121, ADR-0122]
serves: [PRD-0034]
blocked-by: []
---

## Why

The project owner asked for contractual trades between two players, with
counteroffers, and with a refusal that says no and asks for no more
counteroffers.

Nothing in the engine let two factions agree on anything. Goods had no path
from one faction to another. A promise could not be recorded, so it could not
be broken. And a refusal could not say how final it was, which is the gap that
costs the most: a language model that cannot tell a no from a closed door asks
again for ever.

A product record holds the need.[^1]

## Impact review

**Governed by.** ADR-0002 D1 forbids a floating point number in simulated
state, so every quantity, term and deadline here is an exact integer. ADR-0003
D1 keys every random draw; nothing in this work draws, and the record says so
in its own text. ADR-0004 D1, D3 and D4 fix the iteration order; the plane is
walked in pair order and the deliveries are sorted on a total key before
anything moves. ADR-0006 D1 and D2 make an event plain data delivered at the
barrier. ADR-0053 D2 states that a relation between factions is a plane, which
is the shape of the negotiation. ADR-0062 D3 and D5 govern the order of a pass
that moves a quantity. ADR-0072 D5 holds the conservation equality that a
transfer must not break. ADR-0073 D2 gives the sort-then-transfer shape.
ADR-0090 D1 is the precedent for a store that holds nothing until somebody
writes to it.

**Changes.** No record changes. Nothing here contradicts one.

**Creates.** Three records, and each states a claim a future contributor could
reasonably choose otherwise on.

- ADR-0120 says the negotiation is engine state and the words are not. A
  contributor could reasonably put the whole conversation in the control plane,
  and one of the two players already keeps a transcript there.
- ADR-0121 says a terminal refusal closes an ordered pair until a named tick.
  A contributor could reasonably make a refusal permanent, or make it clearable
  by either party, and both are wrong for different reasons.
- ADR-0122 says a contract moves a quantity only when a unit carries it. A
  transfer between two stores is shorter and a contributor will reach for it.

Each passes the three conditions of the scope rule.[^2] Choosing otherwise
costs more than changing it later in every case, because each answer shapes
what a game built on the engine can be. And the reasoning is not visible in the
code: nothing in a delivery pass says why there is no store-to-store transfer.

**Blockers.** Two opened. BLK-120 holds what a contract means when the other
party has no settlement, and the engine states one answer plainly. BLK-121
holds whether a player may read a negotiation it is not party to, and the
engine answers any pair. Neither stops work.

**Product record.** PRD-0034, and this item answers it.[^1]

**Precedent.** The findings register holds three entries this work
produced.[^3] [^4] [^5]

## What was built

The engine holds one row for each ordered pair of factions, and the plane holds
no row until somebody speaks. Six verbs open a negotiation, restate its terms,
agree to them, decline them, decline them terminally, and open a direction the
speaker closed. Three reads answer one pair, answer every pair one faction is a
party to, and answer what the last step said.

A pass in the step moves a carried load into the store of another faction's
settlement when a contract obliges it, and then fails every contract that
reached its deadline with a debt.

Every verb and every read crosses the Python boundary, and every test starts
there.

## Done when

- Two factions offer, counter and agree, and the goods move. **Done.**
- A terminal refusal is distinguishable from a refusal, and an attempt to
  reopen states the step that opens the direction again. **Done.**
- A contract whose party cannot deliver fails at its deadline, and the failure
  costs the defaulting party the direction. **Done.**
- The result does not change with the thread count. **Done.**
- Each rule was broken on purpose and the test that covers it went red. **Done
  for five rules. The review names each one.**
- The whole check command runs green. **The review states each gate and its
  output.**

## Outcome

The work is in the review.[^6] The review states where the negotiation lives
and why, how a terminal refusal differs from a refusal, that goods move by
carrier and never instantly, what a failure to deliver does, the five defects
that were put back, and every gate.

Two things were left undone on purpose. The second product number that was
allocated was not used, because one record states the need and a second would
have split one need in two. And the pass opens no stage, which follows the
precedent of the delivery pass beside it; item 0421 holds the repair for
both.[^4]

## References

[^1]: PRD-0034, two players hold each other to a future delivery. `docs/product/shaped/prd-0034-two-players-hold-each-other-to-a-future-delivery.md`
[^2]: Decision Record Scope, section 1. `.claude/rules/adr-scope.md`
[^3]: Findings register, FND-430. `docs/FINDINGS.md`
[^4]: Findings register, FND-431. `docs/FINDINGS.md`
[^5]: Findings register, FND-432. `docs/FINDINGS.md`
[^6]: Review of item 0420. `docs/reviews/0420-contracts-and-counteroffers.md`
