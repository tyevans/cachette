---
id: 0477
title: Give each faction a trade board and let a contract carry land or a relation
status: refined
created: 2026-09-05
implements: [ADR-0147 D1, ADR-0147 D3, ADR-0147 D4, ADR-0147 D5, ADR-0126 D1, ADR-0126 D2, ADR-0127 D1, ADR-0128 D2, ADR-0128 D4, ADR-0002 D1, ADR-0004 D1, ADR-0001 D4]
changes: [ADR-0128 D1]
creates: []
serves: [PRD-0050, PRD-0051]
blocked-by: [BLK-007, BLK-036]
---

## Why

**A faction cannot say what it wants, and a contract moves only a resource.**
No god can trade land, and no treaty exists. This item is the engine half of
pass 6 of the living world game layer.[^1]

Each faction holds one small fixed-size table of advertisements. A row holds
(good, quantity, offers-or-wants, asking good, asking quantity). The row count
is a row in the balance register, and it is unset.[^2] Python writes rows
through `advertise(faction, rows)`, which replaces the whole board of one
faction, and `market(faction)` returns the board of any faction as columns.
Reading a board costs nothing and moves no relation.

Each side of a contract becomes a tagged consideration of one of three kinds:
a resource, a bounded set of tiles that the offerer holds, or a step on the
relation pair. A land set is one level 1 cell or a bounded list of tiles. The
holder of every tile in the set changes on full delivery of the other side, at
the barrier, and no carrier moves land. A relation step is stored now and
delivers as a logged no-op, because the relation matrix arrives in pass 3. The
status machine is unchanged.

**This item was split on 5 September 2026.** The controller half is item 0482:
advertising from site economies on a schedule, pricing at the integer
midpoint, opening contracts and assigning carriers. That half waits for the
controller stage of item 0472 and the relation of item 0474. This half touches
no stage and does not touch `fn step`.

## Impact review

**Governed by.** ADR-0147 D1 holds that a consideration is one tag and the
content the tag names, and that every field is a whole number. The row gains
one tag byte for each side, and the tile list of a land side sits beside the
row in the same plane, indexed by the row. ADR-0147 D3 holds that a land set
and a relation step deliver on full delivery of the other side, without a
carrier, in the same pass. ADR-0147 D4 holds that the engine refuses a land
offer whose tiles the offerer does not hold, and that the list bound is a
balance value. ADR-0147 D5 holds that the status machine is unchanged.
ADR-0126 D1 holds that the plane enters the state hash and that a walk reads
it in pair order; the board follows the same shape, lazy until the first
write. ADR-0126 D2 holds that every term is an exact integer. ADR-0127 D1 holds
that a terminal refusal closes a direction for every kind. ADR-0128 D2 holds
the sort-then-transfer order for a resource, and D4 holds that a contract with
a debt at the deadline fails for every kind. ADR-0002 D1 holds that no column
is a floating point number. ADR-0004 D1 holds that the order of every write is
explicit: tiles change holder in ascending tile index, and contracts apply in
pair order. ADR-0001 D4 holds that the two determinism tests protect every
change to simulated state.

**Changes.** ADR-0128 D1, which states that the only path from one faction's
goods to another's is a unit that carries a load. ADR-0147 widens it: a
quantity still moves only when a unit carries it, and a consideration that is
not a quantity applies when the quantity it was priced against has arrived.
ADR-0128 D1 already cites the widening. A holder change is inside the widened
decision because it moves no store and no quantity. ADR-0147 stays at `Draft`,
and the Outcome lists every disagreement between the record and the code.

**Creates.** None. ADR-0147 is allocated and written.[^3]

**Blockers.** BLK-036 governs upgrades on traded ground, and it is open.[^4]
Until it closes, the engine refuses a land offer whose tiles carry an upgrade,
and the error text names the blocker. The commit that removes the refusal
searches the tree for the blocker number. BLK-007 governs the cost of the
board and of a land set at the target scale; both figures stay derived. The
board size and the land list bound are rows in the balance register, and both
are unset.[^2] The engine holds each as a world parameter with a setter, and
the value in the code is a stand-in and not a decision. BLK-120 and BLK-121
govern the trade plane and are unchanged by this item.[^4]

**Precedent.** FND-320 records that nothing regenerates the type stub, so the
stub is edited by hand in the same commit as `advertise` and `market`.[^5]
FND-048 and FND-051 record that a determinism test cannot see a broken
invariant and that a fixture chosen for realism hides the defect, so every
rule here has a test that a restored defect turns red.[^5] Recurring defect
shape 3 records that a capability nothing invokes ships inert; the relation
kind is accepted as inert for one pass, and the Outcome says so.

**Serves.** PRD-0050, a god advertises what it will trade, and PRD-0051, a god
trades land.[^6]

## Done when

- Each faction holds one fixed-size board, and `advertise(faction, rows)`
  replaces the whole board. A call with more rows than the bound is refused
  and no row changes.
- `market(faction)` returns the board of any faction as columns, and no column
  is a floating point number.
- The board enters the state hash. Two worlds that differ only in one board
  row have different hashes.
- Each side of a contract carries a tag in the closed set: resource, land,
  relation. The Python verbs take the tag and the target as keyword arguments,
  and every existing positional call keeps working.
- A land side names one level 1 cell or a bounded list of tiles. The engine
  refuses a land side whose tiles the debtor does not hold, a land side with
  more tiles than the bound, and a land side whose tiles carry an upgrade. The
  upgrade refusal names BLK-036 in its text.
- On full delivery of the other side, every tile of a land set changes holder
  to the creditor, in ascending tile index, at the barrier, with no carrier. A
  test proves that exactly the listed tiles moved and no other, with a fixture
  at the extreme: a cell on the world edge and a list at the bound.
- A relation side is stored and delivers as a no-op that logs an event. A test
  proves the event is logged.
- Every new event type is plain data with `repr(C)`, declared padding and no
  `bool`.
- Each new test is proven able to fail: the defect is put back, the test goes
  red, and the commit body records it.
- The thread-count test and the golden state hash test pass at 1, 2 and 12
  threads, and a determinism test drives a land trade at all three counts.
- The type stub is edited by hand in the same commit as the verbs.
- The whole check command runs green.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Design: the living world game layer, sections 4 and 12. `docs/superpowers/specs/2026-09-05-living-world-game-layer-design.md`
[^2]: Balance register. `docs/reference/balance.md`
[^3]: ADR Registry. `docs/adrs/REGISTRY.md`
[^4]: Blockers register, BLK-036, BLK-007, BLK-120 and BLK-121. `docs/BLOCKERS.md`
[^5]: Findings register, FND-320, FND-048 and FND-051. `docs/FINDINGS.md`
[^6]: Product records PRD-0050 and PRD-0051. `docs/product/`
