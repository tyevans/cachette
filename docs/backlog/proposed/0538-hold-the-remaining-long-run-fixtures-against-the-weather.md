---
id: 0538
title: Hold the remaining long-run fixtures against the weather
status: proposed
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

## The architectural impact review

This item is not refined. Refining it is the work. The review must answer
three questions.

**Which records govern it.** No decision record governs a fixture. ADR-0177
governs the latitudes of a world, and it is what changed the sky.[^5] ADR-0001
governs the two determinism tests, and a repair must not weaken a comparison
across thread counts.[^6]

**Whether any fixture is red rather than latent.** Two of the seven that broke
were found red at the tip during the survey rather than by the gate. A worker
must run each file before it assumes the file is green.

**Which repairs are engine work in disguise.** A fixture that cannot state its
claim under the new sky is telling the project something about the engine. The
survey found one: a deposit under the region sky recovers on a period of tens
of ticks, so a crowd on one deposit starves for food that a single gatherer
would have had.[^2]

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

### The rows, highest risk first

| Rank | File | What it assumes |
|---|---|---|
| 1 | `crates/cachette-core/tests/carrying_a_load_home.rs` | Every carrier of the shared fixture is laden after 2400 ticks, and a dead carrier can never satisfy that gate |
| 2 | `crates/cachette-core/tests/a_city_changes_hands.rs` | A road finishes in the work plus one tick, and the road and the defender both stand through a 200 tick siege |
| 3 | `crates/cachette-core/tests/a_sent_set_walks_to_its_destination.rs` | Four units are alive on each of 4000 frames, read through a bare panic |
| 4 | `crates/cachette-core/tests/plan.rs` | Two cities end joined by a road after 900 ticks, and a road counted at the last tick still stands |
| 5 | `crates/cachette-core/tests/starvation.rs` | One watched unit answers a reader on each of 264 ticks, and every fed unit lives |
| 6 | `crates/cachette-core/tests/an_upgrade_that_wears_away_says_so.rs` | A dead builder is replaced by a call that answers false, so the run ends on a budget rather than on the cause |
| 7 | `crates/cachette-core/tests/a_short_cohort_loses_part_of_itself.rs` | A cohort of 32 on one open tile keeps somebody after 400 ticks, against a measured fall to three |
| 8 | `crates/cachette-core/tests/a_faction_remembers_what_happened_to_it.rs` | Two exact casualty equalities hold over 240 ticks |
| 9 | `crates/cachette-core/tests/ground_changes_hands.rs` | One invader stands on an island for 325 ticks and holds a lease |
| 10 | `crates/cachette-core/tests/a_settler_takes_the_best_place_it_can_reach.rs` | A dead settler falls out of the loop and is reported as an open last mile |
| 11 | `crates/cachette-core/tests/a_game_ends_on_three_paths.rs` | One rival unit lives through 200 ticks, and a wonder finishes in the work plus eight ticks |
| 12 | `crates/cachette-core/tests/a_faction_reads_its_way_to_a_win.rs` | The same rival-lives assumption, under a fixture that freezes movement and not the sky |
| 13 | `crates/cachette-core/tests/a_faction_that_does_nothing_is_hunted.rs` | Three factions cross a relation band inside 400 ticks, one of them asserted negative |
| 14 | `crates/cachette-core/tests/a_faction_reads_what_conflict_costs_it.rs` | A fire reaches a unit inside 400 ticks, while rain is the antagonist of the mechanism under test |
| 15 | `crates/cachette-core/tests/the_population_grows.rs` | Eight of eight seeds reach a cohort in 2500 ticks, and a fed site holds the population it grew |
| 16 | `crates/cachette-core/tests/an_island_faction_leaves_its_island.rs` | A mariner cohort lives on open water for 400 ticks and founds a second site |
| 17 | `crates/cachette-core/tests/a_gatherer_walks_to_the_stock_it_was_ordered.rs` | A gatherer lives 64 frames, and the exit field agrees with a tile column the storm ledger writes |
| 18 | `crates/cachette-core/tests/a_step_leaves_the_world_readable.rs` | One seed founds a city inside 600 ticks |
| 19 | `crates/cachette-core/tests/relation.rs` | A leader of one faction lives to move a relation inside 200 ticks |

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
