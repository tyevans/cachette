---
id: 0538
title: Hold the remaining long-run fixtures against the weather
status: refined
created: 2026-09-10
implements: []
changes: []
creates: []
serves: []
blocked-by: []
---

## Why

**A map is now one region of a stormy planet, and no ground of any world stays
dry.** A survey of the seats that admit a city, at nine latitudes and four
spans, found none that stays dry and clear of storms for twelve hundred
ticks.[^1] A storm ends a unit that stands in the open, it kills a resident of
a site, it wears a level that stands, and it writes the food it flattens into
the depletion ledger that a gather also writes.

**Seven fixtures have already broken on this, one at a time.** Each cost a
gate run and a diagnosis, and each failure named the wrong subsystem: a slow
world, a missing precondition, a starvation, or a builder that left its
tile.[^2] [^3] [^4]

A survey of every test of the core crate that steps a world past about a
hundred ticks found many more that pass today by luck. Five were repaired
under this survey. The rest are listed below, in the order a worker should
take them.

**The tick count was the wrong criterion, and the list below is therefore not
the list of the files at risk.** A storm reaches a unit in the open on any
tick, so exposure is what a fixture depends on and not how long it runs. One
file that steps twenty-four ticks was red at the tip and the survey never
listed it.[^12] Each row below is still worth taking. A worker that finishes
them has not finished the sweep, and the sweep ends when a search of the whole
crate for a stepping test comes back clean.

## The architectural impact review

The review is done, and the three questions have answers.

**Which records govern it.** No decision record governs a fixture. ADR-0177
governs the latitudes of a world, and it is what changed the sky.[^5] ADR-0001
governs the two determinism tests, and a repair must not weaken a comparison
across thread counts.[^6] No repair made under this item touches either.

**Whether any fixture is red rather than latent.** Four were red at the tip and
the gate had not reported three of them. A worker must run each file before it
assumes the file is green. Running the nineteen files one at a time takes about
five minutes.

**Which repairs are engine work in disguise.** Two are. The first is the
deposit that no longer recovers inside a short run, which the survey
found.[^2] The second is the road: a way of sixteen tiles is never finished
under the sky of a region, at nine hundred ticks, at two thousand four hundred
and at four thousand alike.[^9] Neither is a defect. The engine wears what a
faction builds, and a fixture that wants to measure a solver stands where the
weather is quiet.

## What the work found

Three findings came out of the repairs so far.

**A polar region with a wide span leaves the ground dry, and a storm still
forms over a world of full extent.**[^10] The earlier survey read one tile. A
fixture that needs a quiet sky over a corridor reads the corridor, and it
states the regime it needs rather than the widest one it can name.

**A budget is not the repair when the state is stable.**[^9] Read what a run
reached at two budgets before raising one.

**The weather breaks one fixture twice, and the second cause is the
ground.**[^11] The casualty log covers the units. It covers nothing about the
ground they stand on, and a fixture that must hold a rate over a window states
the recovery period.

## What the work delivers

A worker takes the rows below in order. It repairs each with the idioms the
project already holds, and it proves each repair able to fail before it hands
back.

The idioms are four.

1. Read the log of what the storms took, and skip the units the log names and
   no others. Count what the run read, and refuse a count of zero.
2. Derive a tick budget from the work table, and give it a margin the fixture
   documents as slack rather than as a count of ticks the test takes.
3. Write an input on every tick of a window that must hold, in the way a
   fixture already refills a store.
4. State that the unit a fixture depends on is alive on each tick, so a
   failure names the storm rather than the subsystem under test.

Where no span and no input can hold a term still, hold it by comparison: run
the same seed twice, change one thing, and assert the difference.[^7]

### What is repaired

The table below holds the files that are repaired. Each repair was proven able
to fail. The commit bodies hold the probes and their output.

| File | What it now proves |
|---|---|
| `crates/cachette-core/tests/carrying_a_load_home.rs` | The delivery, the coarse return field and the last mile, over a carrier set the storms thin and ground that each carrier has to itself |
| `crates/cachette-core/tests/plan.rs` | Two cities end joined by a road, under a stated sky the run reads on every tick, inside a derived budget whose slack the run proves |
| `crates/cachette-core/tests/a_sent_set_walks_to_its_destination.rs` | Every surviving sent unit reaches the destination cell and one reaches the tile |
| `crates/cachette-core/tests/starvation.rs` | The condition, the end, the bound and the recovery, each read over the units the storm log does not name and each refusing a count of zero |
| `crates/cachette-core/tests/an_upgrade_that_wears_away_says_so.rs` | The build states that its builder lives on every tick, so a death names the cause |
| `crates/cachette-core/tests/a_settler_takes_the_best_place_it_can_reach.rs` | A settler that leaves the world is named as such, and not as an open last mile |
| `crates/cachette-core/tests/barrier_ordering.rs` | The barrier ordering, over a watched set the storms thin, with an exact population equality |
| `crates/cachette-core/tests/passable_ground.rs` | The movement pass refuses water, over a company seated on the shoreline and sent across a lake, and thinned by the storms the log names |

The last two files were not on the list below, and both were red at the tip.

### The rows that remain, highest risk first

| Rank | File | What it assumes |
|---|---|---|
| 1 | `crates/cachette-core/tests/a_city_changes_hands.rs` | A road finishes in the work plus one tick, and the road and the defender both stand through a 200 tick siege |
| 2 | `crates/cachette-core/tests/a_short_cohort_loses_part_of_itself.rs` | A cohort of 32 on one open tile keeps somebody after 400 ticks, against a measured fall to three |
| 3 | `crates/cachette-core/tests/a_faction_remembers_what_happened_to_it.rs` | Two exact casualty equalities hold over 240 ticks |
| 4 | `crates/cachette-core/tests/ground_changes_hands.rs` | One invader stands on an island for 325 ticks and holds a lease |
| 5 | `crates/cachette-core/tests/a_game_ends_on_three_paths.rs` | One rival unit lives through 200 ticks, and a wonder finishes in the work plus eight ticks |
| 6 | `crates/cachette-core/tests/a_faction_reads_its_way_to_a_win.rs` | The same rival-lives assumption, under a fixture that freezes movement and not the sky |
| 7 | `crates/cachette-core/tests/a_faction_that_does_nothing_is_hunted.rs` | Three factions cross a relation band inside 400 ticks, one of them asserted negative |
| 8 | `crates/cachette-core/tests/a_faction_reads_what_conflict_costs_it.rs` | A fire reaches a unit inside 400 ticks, while rain is the antagonist of the mechanism under test |
| 9 | `crates/cachette-core/tests/the_population_grows.rs` | Eight of eight seeds reach a cohort in 2500 ticks, and a fed site holds the population it grew |
| 10 | `crates/cachette-core/tests/an_island_faction_leaves_its_island.rs` | A mariner cohort lives on open water for 400 ticks and founds a second site |
| 11 | `crates/cachette-core/tests/a_gatherer_walks_to_the_stock_it_was_ordered.rs` | A gatherer lives 64 frames, and the exit field agrees with a tile column the storm ledger writes |
| 12 | `crates/cachette-core/tests/a_step_leaves_the_world_readable.rs` | One seed founds a city inside 600 ticks |
| 13 | `crates/cachette-core/tests/relation.rs` | A leader of one faction lives to move a relation inside 200 ticks |

Every one of these thirteen passes today. None is red at the tip.

## What this item does not do

It changes no engine code. The engine behaves as its records state, and the
fixtures assumed a sky that no longer exists.

## References

[^1]: Findings register, FND-744. `docs/FINDINGS.md`
[^2]: Findings register, FND-745. `docs/FINDINGS.md`
[^3]: Findings register, FND-725. `docs/FINDINGS.md`
[^4]: Findings register, FND-726. `docs/FINDINGS.md`
[^5]: ADR-0177, the row axis of a world is a latitude that the world states. `docs/adrs/draft/adr-0177-the-row-axis-of-a-world-is-a-latitude-that-the-world-states.md`
[^6]: ADR-0001, one binary gives one answer at any thread count. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^7]: Findings register, FND-728. `docs/FINDINGS.md`
[^9]: Findings register, FND-749. `docs/FINDINGS.md`
[^10]: Findings register, FND-748. `docs/FINDINGS.md`
[^11]: Findings register, FND-750. `docs/FINDINGS.md`
[^12]: Findings register, FND-751. `docs/FINDINGS.md`
