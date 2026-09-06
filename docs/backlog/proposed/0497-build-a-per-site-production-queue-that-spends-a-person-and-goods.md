---
id: 0497
title: Build a per-site production queue that spends a person and goods
status: proposed
created: 2026-09-05
implements: [ADR-0158 D1, ADR-0158 D2, ADR-0158 D3, ADR-0158 D4, ADR-0158 D5, ADR-0158 D6, ADR-0144 D2, ADR-0144 D3, ADR-0154 D4, ADR-0004 D1, ADR-0009 D1]
changes: []
creates: []
serves: [PRD-0011, PRD-0012, PRD-0048]
blocked-by: [0059, BLK-050]
---

## Why

**No faction can make a unit it chose.** The seeding founds a group once, every
founded unit carries the worker row, and the worker row holds an attack of zero.
A meeting between two founded groups kills nobody, so the unit clause of the
domination reader can never fire. The findings register holds the reading and
its evidence.[^1]

Nothing produces a settler either, so a faction founds one city and never a
second, and the ground it holds never grows past the reach of that city.

This item builds the mechanism that answers both. A site holds a queue. A caller
or the controller queues a unit type. The store pays as the entry advances. A
finished entry takes one person of the site and the goods it costs, and gives
the world one unit of that type.

**People are the scarce middle of the economy.** A worker gathers into a store.
Food and housing grow the population. The queue spends a person and goods to
make a settler, a soldier, a worker, a merchant or a leader. So expansion and
war compete with gathering and with each other, and that competition is the
game.

## What the work does

1. A site holds a bounded, ordered queue of plain-data entries. Each entry names
   a unit type and holds the work done toward it. The whole queue enters the
   state hash.
2. One verb queues an entry. A Python caller, the built-in controller and a
   learner all reach that one verb. The engine holds no rule about what to
   queue.
3. One stage advances the front entry of each site. The advance charges the
   store, and a site that cannot pay makes no progress.
4. A finished entry takes one resident of the site and the goods it costs, then
   creates one unit of its type at the site. A site with no spare person or no
   goods refuses, and the entry stays at the front.
5. The census gains rows: what the queue produced, and each refusal reason
   counted apart.

## Impact review

Not done. This item is `proposed/`, and refining it is the work.

**What refining must answer.**

- Where the work value of a type lives. The record says it comes from the table
  the world is built with, and the type table row is a set of capability
  columns. A build work is not a capability, so the review must say whether the
  row gains a column, whether a second table holds the costs, and what that
  means for the record that fixes the row width.[^2]
- Which resident a finished entry takes, by a stable key over the residents of
  the site. The record forbids the order a collection happened to hold.[^3]
- Where the stage sits in the step, against the growth stage and against the
  pass that ends a starved unit.
- What the controller queues, and how the choice enters the action table. The
  action table derives its verb rows from the controller's choice
  enumeration.[^4]
- Whether the queue verb needs a magnitude bucket, or whether one entry for each
  call is enough.

**Governed by.** ADR-0158 D1 through D6.[^5] ADR-0144 D2 makes the verb the one
path for the controller, and D3 counts a refusal.[^6] [^7] ADR-0154 D4 puts the
verb in the action table a learner reaches.[^4] ADR-0004 D1 fixes every order.
ADR-0009 D1 makes the per-site writes disjoint.

**Blockers.** BLK-050 governs the queue bound, the work of each type, the goods
of each type, the people of each type, the charge and the schedule. This item
states none of them in the code, and the balance register holds all six.[^8]
BLK-007 governs any cost figure, so this item states none.

**Waits on item 0059**, which gives a site its housing and one reader that
answers how many people live there. The queue needs that reader to know whether
a site holds a spare person.

**Does not overlap item 0485.** Item 0485 gives a settler the verb that founds a
city, and gives the controller the choice to settle. This item makes the
settler. Neither answers the other.[^9]

## Done when

- A caller queues an entry through the public interface, and a reader shows the
  queue of a site.
- The queue is bounded. A test fills it and asserts the refusal.
- An entry advances only when the store pays. A test holds a site with an empty
  store and asserts that the entry does not move.
- A finished entry removes one resident and adds one unit of the named type. A
  test asserts the population of the site is unchanged and the type changed.
- A site with no spare person refuses a finished entry, and the entry stays at
  the front. A test asserts both, and puts the refusal back to prove the fixture
  reaches it.[^10]
- The two refusal reasons are counted apart, and the census reports what the
  queue produced. A test asserts each count.
- The built-in controller queues through the same verb, and the controller log
  holds the command. A test drives the engine rather than calling the verb.[^11]
- The engine names no unit type in a rule. A whole-tree search shows it, and the
  commit body carries the search.
- A settler produced by the queue founds a city through the verb of item 0485,
  and a soldier produced by the queue kills in a contest. Both tests drive the
  engine.
- The thread-count test asserts the same queue and the same units at 1, 2 and 12
  threads. The golden state test passes.
- No figure appears in the code or in a comment. Every value is a balance
  register row.
- The whole check command runs green.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Findings register, FND-486. `docs/FINDINGS.md`
[^2]: ADR-0120, a unit carries a type that indexes a table, decision D2. `docs/adrs/draft/adr-0120-a-unit-carries-a-type-that-indexes-a-table.md`
[^3]: ADR-0004, iteration order is explicit, and unordered reductions need slots, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^4]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decision D4. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
[^5]: ADR-0158, a site builds a typed unit from a bounded queue its store pays for. `docs/adrs/draft/adr-0158-a-site-builds-a-typed-unit-from-a-bounded-queue-its-store-pays-for.md`
[^6]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decision D2. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
[^7]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decision D3. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
[^8]: Balance register, the production queue. `docs/reference/balance.md`
[^9]: Backlog item 0485. `docs/backlog/proposed/0485-let-a-settler-found-a-city-and-let-the-controller-settle-new-ground.md`
[^10]: Testing Rules, a fixture supplies the input. `.agents/rules/testing.md`
[^11]: Testing Rules, drive the real caller. `.agents/rules/testing.md`
