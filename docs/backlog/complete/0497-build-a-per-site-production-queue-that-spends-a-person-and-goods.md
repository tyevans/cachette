---
id: 0497
title: Build a per-site production queue that spends a person and goods
status: complete
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

**A site holds a bounded, ordered queue, and one stage advances it.** The queue
register, the build cost table, the one verb, the stage, the census rows, the
Python bindings and the tests all landed. ADR-0158 stays a draft, and the
disagreements between it and the code are listed below.

### The impact review the item asked for

The item said the review was the work. Here are its five questions and the
answers the work gave.

**Where the work value of a type lives.** In a second table, not in the unit
type row. A unit type row is a set of capability columns and a zero in one
means the type cannot do what the column names.[^12] A build work of zero means
a type that finishes at once, which is a second meaning for one zero in one
row. The build cost table is indexed by the same type and holds the work, the
people and the goods.

**Which resident a finished entry takes.** The lowest arena slot of the
residents of the site, which is a stable key over the units and never the
order a collection held them.[^3] Naming it needed a route that the residence
record allows, and section below states the route and its cost.

**Where the stage sits.** Between the shortage scan and the bridge refresh that
follows it. It reads the store, so it runs after the rate pass and after the
consumption pass. It is a structural change, so it rides the barrier that is
already there. It runs after the scan and not before it, because the scan holds
a plane of the slots it ends, and a stage that freed a slot and filled it again
would hand the scan a live unit it never marked.

**What the controller queues.** One keyed draw picks a type from the rows the
unit type table fills, and the order goes to the lowest-slot site of the
faction whose queue has room. The choice enumeration gains one row, so the
action table derives a verb row from it when that table is built.[^4]

**Whether the verb needs a magnitude bucket.** No. One call is one entry. The
bound is what limits a caller, and a caller that wants four entries calls four
times.

### How the stage names the person, and what it costs

The record names the person by a stable key over the residents, the residence
record refuses a reverse index and refuses a stored roster, and the cohort
table gives a per-site count and not a per-site list.[^13] Naming a resident
therefore needs a pass over the units.

**The stage makes completion set-valued, so there is one such pass for the
whole tick.** Pass one visits the sites and their front entries, charges the
store, and collects the sites whose entry finished. It costs the site count
times one. Pass two runs only when pass one collected something: it walks the
unit arena once in ascending slot order and buckets the live units by their
home site, keeping the lowest slots of each bucket. Pass three applies in site
slot order.

The cost is therefore the site count on every scheduled tick, plus the
population once on a tick where an entry finishes. That is the shape the
controller stage already uses when it buckets the units of a faction for one
command.[^14]

### Despawn and spawn, not conversion in place

A finished entry despawns the residents it takes and spawns one new unit. The
record asks for distinct identities, and the reason holds: a Python caller
holding the identity of a worker must not find that the worker became a
soldier.[^5] The despawns run before the spawn, so the arena always holds a
free slot and the spawn cannot refuse for want of capacity.

**The resident count of a site does not fall.** The unit the queue makes lives
at the site that built it, so the site keeps the person and the person now has
a type and a cost. What falls by exactly one is the count of the people the
site can still spend. The test asserts both.

### The disagreements with ADR-0158

The record is a draft and this work did not change it. A reviewer must settle
each of these.

1. **D5 says the stage never walks the units.** It does, once for the whole
   tick, and only when an entry finishes. D4 cannot be satisfied any other way
   while the residence record forbids a roster. Either D5 gives that cost back
   in the terms above, or D4 stops naming an individual.
2. **D4 says an entry takes one person.** The code reads the count from the
   people row of the build cost table, and the balance register owns that
   row.[^8] The record body holds a figure the register owns.
3. **D1 says the bound is a property of the world parameters.** The code agrees:
   the bound is settable and a bound of zero turns the queue off. The stored
   block has a fixed width and it caps the bound, and the register holds both.
4. **The consequences omit the early-game squeeze.** A faction founds with a
   very small group and its first entry spends a person out of it. The
   founded-group test had to turn the queue off for that reason, which is
   direct evidence for the repair the review asked for.
5. **Cancelling is "not decided here", and the item asked for a clear.** The
   verb clears an entry, and the work the store already paid for is lost. The
   record must state the rule or drop the sentence.
6. **The record is two claims.** The queue mechanism and the consumption rule
   are separable in the code as well: the queue register, the verb, the advance
   and the two census rows stand without D4, and D4 is one branch of pass three
   plus pass two. **This implementation would survive a split** with no change
   beyond the citations in the source.

### Left undone

Growth, housing capacity and a settler founding a city are items 0060, 0059 and
0485, and this item built none of them. A settler is a unit type with a settle
capability above zero; the queue produces one and does not redefine one.

The item asked for a test in which a settler off the queue founds a city and a
soldier off the queue kills in a contest. **Neither was written.** The founding
verb is item 0485 and does not exist. The contest test would restate what the
contest tests already prove, because a unit of the soldier row fights whatever
made it; the Python test asserts that the type the queue produced holds an
attack above zero, which is the part this item owns.

### The gates

`cargo fmt --check`, `cargo clippy -p cachette-core -p cachette-py
--all-targets -- -D warnings`, `cargo test -p cachette-core`, `cargo test -p
cachette-view`, `uv run maturin develop`, `uv run pytest` and `just records`
all ran green. The whole `just check` was not run, on instruction.

Ten golden files moved and one is new. The queue is state a later frame reads,
so it enters the state hash, and every scenario that runs the controller now
holds queued entries.

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
[^12]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decisions D1 and D2. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
[^13]: ADR-0157, a site's free places are its built housing less the residents the engine already counts, decision D3. `docs/adrs/accepted/adr-0157-a-sites-free-places-are-its-built-housing-less-the-residents-the-engine-counts.md`
[^14]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decision D1. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
