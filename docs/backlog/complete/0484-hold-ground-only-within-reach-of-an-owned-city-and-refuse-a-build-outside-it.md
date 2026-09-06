---
id: 0484
title: Hold ground only within reach of an owned city, and refuse a build outside it
status: complete
created: 2026-09-05
implements: [ADR-0150 D1, ADR-0150 D2, ADR-0150 D3, ADR-0150 D4, ADR-0053 D2, ADR-0053 D4, ADR-0004 D1, ADR-0001 D4]
changes: [ADR-0053 D5, ADR-0053 D6]
creates: []
serves: [PRD-0054]
blocked-by: [BLK-007, BLK-050]
---

## Why

**A holding spreads from wherever a unit stands and grows until something
stops it.** A faction cannot lose ground by losing a city, and cannot gain
ground by founding one. The project owner asked that a faction's ground exist
only around a city it owns, extended by the upgrades finished inside it up to a
bound, and that a unit build outside that ground only a road. The product
record states the need, and a draft record states the rule.[^1] [^2]

This item replaces the spread pass at the holding stage with a rewrite from the
cities. For each settlement, the pass computes a reach from three balance
values and the finished upgrades on the ground the settlement held last step.
It decides every tile within reach of any settlement by nearest city, then
lowest slot index, and it clears every held tile that no city reaches. The
changes go through the apply path that the spread and the land transfer use, so
the running total, the block masks and the held list repair as they do
today.[^3]

The build intent pass and the build verb both gain one test: the holder of the
builder's tile is the builder's faction, or the kind is a road. A refused order
is dropped and, where the controller gave it, counted.[^4]

**Two readers change without changing their code.** The territory score reader
reads the running total, so a faction's score becomes the ground its cities
reach. The controller's build orders meet the new refusal, so a faction with no
city sees every build but a road refused, and the refusal count rises.

**This item touches `fn step` in `world.rs`. Only one worker may hold it at a
time.** It waits for the pass that holds `fn step` before it to merge.

## Impact review

**Governed by.** ADR-0150 D1 holds that a tile is held by the faction whose
city is nearest within reach, that two cities at one distance resolve by the
lower settlement slot index, and that no tile is held with no city in reach.
ADR-0150 D2 holds that the reach of a city is a base plus one step for each
block of finished upgrades on the ground the city held at the end of the
previous step, capped at a bound, and that the three values are balance rows.
ADR-0150 D3 holds that the holder column stays stored and hashed, and that the
rewrite runs at the holding stage and writes through the apply path that
repairs the running total, the block masks and the held list. ADR-0150 D4 holds
that the build verb and the build intent pass refuse a build outside the
builder's own held ground unless the kind is a road, and that both apply one
test. ADR-0150 D6 holds that traded land keeps the new holder only where D1
gives it. ADR-0150 D5, the settler, is item 0485 and not this item. ADR-0053
D2 holds that a tile carries one holder, D4 holds that what a faction holds is
a running total the changing rule maintains, and D3 and D7 stand untouched.
ADR-0004 D1 holds that iteration order is explicit. ADR-0009 D1 to D3 hold that
a parallel stage writes disjoint outputs, joined in an order the data fixes.
ADR-0001 D4 holds that the two determinism tests protect the claim. ADR-0111
D1 and D2 hold that the presence relation is derived at the end of the step
from the holder column, so the presence gate now reads city-held ground and
neither record changes.

**Changes.** ADR-0053 D5 and D6, the spread rule and the contest key. ADR-0150
supersedes both, and its registry row says so.[^5] The other decisions of
ADR-0053 stand. ADR-0053 is not edited.

**Creates.** None. ADR-0150 is allocated and drafted beside this item. The
item leaves it in draft and lists every disagreement in its outcome.

**Blockers.** BLK-050 governs the base reach, the finished upgrades that earn
one step of reach, and the bound.[^6] Each is a balance row, and this item
writes a provisional value into each row with its derivation filled. BLK-007
governs the cost of the stage. The cost shape is the cities multiplied by the
area of the largest reach, plus the ground held, and the figure stays derived.

**The holder column stays in the hash.** The three reasons of ADR-0150 D3
hold: the state hash folds the column, the tile event stamps the holder as the
frame left it, and the land side of a contract writes the column between two
rewrites. The item confirms D3 and does not supersede it.

**Golden files.** Every golden scenario that holds a settlement and every
scenario whose units held ground moves, because the rule that writes the
holder column changed. The commit body lists the files regenerated.

**Precedent.** FND-285 records that the spread reaches a large share of the
world at the target population, which is the cost this rule removes. FND-320
records that nothing regenerates the type stub, so the two new readers edit
the stub by hand in the same commit. FND-051 and FND-048 record that a fixture
chosen for realism hides the defect, so every fixture here is built for its
extreme.[^7]

**Item 0370.** It asks for a refusal on ground another faction holds. This
item refuses every build but a road on ground the builder's faction does not
hold, which is a wider set. Item 0370 closes with this one.[^8]

**Serves.** PRD-0054, a god's ground is the ground around its cities.[^1]

## Done when

- At the holding stage the holder column is rewritten from the settlements.
  For every passable tile, the holder is the faction of the nearest settlement
  whose reach covers it, tie by the lowest settlement slot, and none in reach
  means nobody. The stored column, the tile event stamp and the hash are
  written by the existing apply path.
- The reach of a settlement is the base plus the finished upgrades on the
  ground the settlement held at the end of the previous step divided by the
  upgrades that earn one step, capped at the bound. All three are read from
  one rule struct whose defaults are the provisional balance values.
- The rewrite computes one reach for each settlement, then decides the tiles
  the settlements reach in parallel over disjoint chunks, and joins in slot
  order. Distances are hex distances in whole numbers.
- One function states the build rule: the holder of the builder's tile is the
  builder's faction, or the kind is a road. The build verb and the build intent
  pass both call it.
- A settlement at the world edge holds the tiles inside the world within its
  reach and the test asserts the count.
- Two settlements at one distance from a tile: the lower slot holds it, and
  the test proves it with the higher slot founded first.
- A faction with held tiles and no settlement holds nothing after one step.
- The reach grows by exactly one step when the finished upgrade count crosses
  the block, an unfinished upgrade adds nothing, and the reach never passes
  the bound.
- A build off own ground is refused and a road is permitted, from the verb and
  from the controller order through `controller_refused`.
- A traded tile outside the reach of every creditor city is held by nobody
  after the next step.
- The tests that tested ADR-0053 D5 and D6 are removed or rewritten, and the
  commit body names each and says why.
- Each defect is put back once: drop the tie rule, read the upgrades of the
  current step, forget the cap, skip the refusal for a terrace. Each test goes
  red, and the commit body says so.
- The thread-count test runs a world of several settlements at 1, 2 and 12
  threads, and the golden state hash test passes with regenerated files.
- The Python binding holds `city_reach(site)` and `holds(faction, q, r)`, the
  stub is edited by hand, and a pytest file drives both from the boundary.
- `cargo fmt --check`, the clippy gate on both crates, the named test
  binaries and the pytest file run green.

## Outcome

Built. The holding stage rewrites the holder column from the settlements. The
holder of a passable tile is the faction of the nearest city whose reach
covers it, a tie goes to the lower settlement slot, and a tile no city reaches
is held by nobody. The reach of a city is the base, plus one step for each
block of finished upgrades on the ground the city held at the end of the
previous step, capped at the bound. The three values are one rule struct in
the holding, and they enter the state hash beside the column they decide.

The pass computes one reach for each city, then decides the candidate tiles in
contiguous chunks on several threads. Each tile reads the city table and never
another tile. The candidate list is the tiles the cities reach plus the tiles
the held list names, in ascending tile order. The write goes through the apply
path the land transfer uses, so the running total, the block masks and the
held list repair as before.

**One decision this item made, and the record does not hold it.** A finished
upgrade counts for the nearest city of the faction that holds its tile, and a
tie goes to the lower slot. The record says the count reads the ground the
city held, and the holder column names a faction and not a city, so the
attribution had to be chosen. A reviewer of ADR-0150 should decide whether
that rule belongs in D2.

One function states the build rule, and the build verb and the build intent
pass both call it. A build on a tile the builder's faction does not hold is
refused unless the kind is a road. The Python binding raises a typed refusal
instead of asserting, and the controller's own build order is refused through
the same set verb and counted.

The Python control plane gained `holds(faction, q, r)` and `city_reach(site)`,
and the type stub was edited by hand.

Nine golden files moved, and the commit body names the command that
regenerated them. Six tests of the superseded spread rule were deleted with
their machinery, and the commit body names each one and why. Fifteen fixtures
took ground by standing units on it, and each founds a city now. That cost is
recorded as a finding.

Item 0370 asked for a refusal on ground another faction holds. This item
refuses a wider set, so item 0370 closes with it.

Left undone. The demonstration was not run to the tick limit, because this
wave forbids the whole check command and the demonstration gate sits inside
it. ADR-0150 stays a draft, and D5, the settler, belongs to item 0485.

## References

[^1]: PRD-0054, a god's ground is the ground around its cities. `docs/product/shaped/prd-0054-a-gods-ground-is-the-ground-around-its-cities.md`
[^2]: ADR-0150, held ground is the ground within reach of a city its faction owns, and upgrades extend the reach to a bound. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
[^3]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D4. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
[^4]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decision D3. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
[^5]: ADR Registry. `docs/adrs/REGISTRY.md`
[^6]: Balance register. `docs/reference/balance.md`
[^7]: Findings register, FND-285, FND-320, FND-051 and FND-048. `docs/FINDINGS.md`
[^8]: Backlog item 0370, refuse a build on ground another faction holds. `docs/backlog/complete/0370-refuse-a-build-on-ground-another-faction-holds.md`
