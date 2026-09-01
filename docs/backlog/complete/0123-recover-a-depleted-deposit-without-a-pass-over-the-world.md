---
id: 0123
title: Recover a depleted deposit without a pass over the world
status: complete
created: 2026-08-31
implements: [ADR-0072 D1, ADR-0072 D4, ADR-0002 D1, ADR-0003 D1, ADR-0004 D1, ADR-0001 D4]
changes: []
creates: [ADR-0080]
serves: [PRD-0018]
blocked-by: []
---

## Why

A unit takes from a tile and the amount never rises again. Every resource in
the world is a budget that only falls, so a run has one direction and worked
ground is worth less than untouched ground for ever. The product record states
the need.[^1]

The world already holds the fact that makes this cheap. A starting stock
follows from the seed and the tile address, and the engine stores only what
units took, and only for the tiles they took from.[^2] Recovery is therefore
not growth of an amount. It is the ageing away of a stored take. A tile that
nobody touched holds no take, so recovery has no work to do on it. The cost
follows the number of depleted deposits and never the tile count.

**Do not step every tile.** The product record rejects that shape by name, and
so does the record for gathering that came before it.[^1]

## Impact review

**Governed by.**

- **ADR-0072 D1** states that a tile stock is generated from the seed and the
  address, and is never stored as a map. Recovery must not create a map of
  amounts. What a deposit holds now stays a computed answer.
- **ADR-0072 D4** states that the engine stores only what was taken, sparsely.
  Recovery reduces a stored take. It adds no entry for a tile that nobody
  gathered from.
- **ADR-0002 D1** forbids a floating point number in simulated state. A
  recovery amount is an exact whole number, and every fixed-point value goes
  through the arithmetic module.
- **ADR-0003 D1** keys every random draw on system, frame, entity and draw. If
  recovery draws at all, it keys the draw. A recovery that varies by tile with
  no draw key is a defect that both determinism tests pass over.
- **ADR-0004 D1** fixes the iteration order. Any pass over the depleted set
  runs in key order, never in completion order.
- **ADR-0001 D4** hashes the world each frame. A stored take gains a tick, so
  the ledger contribution to the state hash changes. Write the new golden file
  in the same change and say so in the commit body.
- **ADR-0073 D1** admits gathering by sort-then-admit against the tile. The
  amount a tile offers on a tick is now a function of the tick. Admission must
  read the recovered amount, not the raw stored take, or a unit takes what the
  tile does not hold.

**Changes.** None. No accepted record states that a stock only falls, so this
supersedes nothing. Confirm that with a whole-tree search before you claim it,
and put the search command in the commit body.

**Creates.** ADR-0080, reserved in the registry, claiming that a depleted
deposit recovers by ageing the stored take and never by a pass over the world.
The three-condition test passes for it. A contributor could reasonably write
the per-tile pass; choosing it costs a structural property that is expensive to
recover; and a reader of a recovery function cannot see from the code why the
per-tile pass was refused. **Write the record with this work, not before it.**
A record for a subsystem nobody has built is the failure the scope rule opens
with.

**Blockers.** BLK-007 governs every cost figure, so state a shape and no
number. DEC-049 holds the recovery period of each kind and DEC-050 holds
whether an emptied deposit recovers. **Neither stops this work.** Express both
as named parameters in one place, and take the recommendation of each row as
the value until the owner answers. Do not spread either parameter over two
declaration sites.

**Precedent.** FND-104 records that the deferral in the earlier product record
waited on a cost that the sparse store had already answered. Shape 1 of the
recurring defects rule warns about a value declared twice: the recovery period
must not exist both in the engine and in the control plane without a check that
fails when the copies disagree.

## Done when

- A deposit that a unit took from holds more at a later tick than at the tick
  of the take, when nothing takes from it again.
- No deposit ever holds more than its generated starting stock.
- The total recovered over a deposit never exceeds the total taken from it. A
  conservation property test states this and passes.
- A world in which nothing was gathered stores nothing, and one tick of
  recovery over it touches nothing. A test asserts the stored count stays at
  zero.
- Recovery reads no tile that holds no stored take. A test drives a world with
  one depleted tile and asserts that the work done does not change when the
  world extent grows.
- Reading the world does not change it. Two reads at one tick give one answer,
  and a read at tick N followed by a read at tick N gives the same answer as a
  single read.
- Gathering at a tick takes what the deposit holds at that tick, and no more.
- The recovery period is one named parameter for each resource kind, declared
  in one place, and a kind may declare that it does not recover.
- The thread-count equivalence test passes at 1, 2 and 12 threads over a run
  that gathers and then recovers.
- The golden state hash test passes against a regenerated golden file.
- **Put the defect back and watch the test stay green.** Perturb the recovery
  so that it ignores the elapsed ticks, and confirm that a test fails. A test
  that passes under that perturbation measures nothing.
- ADR-0080 is written, the registry row moves from `Reserved` to `Draft`, and
  the priority index row moves with it.
- The whole check command runs green.

## Outcome

**Done. A depleted deposit recovers by ageing the stored take.**

The ledger entry now carries the tick that its amount was last brought up to
date at. A pass ages every stored take forward to the tick, in the key order the
ledger holds, and it takes no grid and no tile count. The step runs the pass
before the gather resolve, so a unit takes what the deposit holds at that tick.

The recovery period is one parameter for each kind. It is stated in simulated
days and converted in one place. A kind may state that it does not recover, and
stone does. A caller replaces the whole rule set, so no second site holds a
period.

The conservation check gained a second term, because recovery gives a part of
the take back to the tile. The stored take alone no longer balances what the
units hold.

ADR-0080 was written with the code, and it is a draft. Two rows opened in the
decisions register: the period value of each recovering kind, and whether a
period returns one unit or the whole deposit. Two findings were recorded.

**What is left undone.** An entry that owes nothing stays in the ledger, so the
depleted set grows and never shrinks. That is item 0124, and the priority index
now states the cost. Nothing shows a watcher a deposit recovering, which is item
0125. The control plane cannot read or replace the recovery rules.

## References

[^1]: PRD-0018, a depleted deposit comes back. `docs/product/shaped/prd-0018-a-depleted-deposit-comes-back.md`
[^2]: ADR-0072, a tile stock is generated, and only what was taken is stored. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
