---
id: 0478
title: Let a faction raise a campaign against a faction at war
status: refined
created: 2026-09-05
implements: [ADR-0144 D1, ADR-0144 D2, ADR-0144 D3, ADR-0144 D4, ADR-0144 D5, ADR-0145 D3, ADR-0146 D2, ADR-0003 D1, ADR-0004 D4, ADR-0001 D4, ADR-0006 D1]
changes: []
creates: []
serves: [PRD-0049]
blocked-by: [BLK-050, BLK-080, BLK-007]
---

## Why

**A faction at war does nothing about it.** The relation says war and no
cohort moves. This item is pass 7 of the living world game layer.[^1]

A campaign is (faction, objective, cohort). Each faction holds a small bounded
register of campaigns, and the register size is a row in the balance
register.[^2] The register is simulated state. Three objective kinds exist:
take a site, relieve an own site, and wear an upgrade.

Raising a campaign is two verbs that exist: the set form of `set_unit_type` to
the soldier row, and `send_units_to` the objective tile. The controller chooses
an objective from the war band and the war weight. It raises no campaign
against a faction outside the war band. A campaign closes when the objective
tile changes holder or when every unit of the cohort has fallen. No new
movement machinery exists.

**This pass does not touch `fn step` in `world.rs`.** It adds no stage. It
extends the controller stage that pass 1 added, and it runs beside passes 6
and 8.

## Impact review

**Governed by.** ADR-0144 D1 holds that the controller is one system at one
fixed stage and that no evaluation starts a pass over the units. The campaign
reading rides on the one scan of the arena the stage already makes for the
speaker, and it adds no scan. ADR-0144 D2 holds that the controller acts only
through verbs a caller can call. The raise is one core function,
`raise_campaign`, that the Python binding and the controller both call, and it
acts through the set form of the type verb and through `send_units_to`. ADR-0144
D3 holds that a refused command is dropped and counted. ADR-0144 D4 holds that
each evaluation draws once from the keyed generator. The campaign draw is one
more keyed draw for each faction on each tick, at the index past the relation
draw, so it collides with no other. ADR-0144 D5 holds that commands apply in
the order of faction and then draw index. The campaign command joins the same
plan list and the same sort. ADR-0145 D3 holds that the soldier row is a row of
the type table. ADR-0146 D2 holds that a pass reads the war edge and never a
band name. The controller asks `war_between` and holds no name. ADR-0003 D1,
ADR-0004 D4 and ADR-0001 D4 hold the keyed draw, the stable sort and the two
determinism tests. ADR-0006 D1 holds that an event is plain data with declared
padding, and the campaign event and the campaign row are both of that shape.

**Is the register a controller reading or a world fact?** It is a world fact.
A later frame reads it, because the stage closes a campaign by comparing the
holder of the objective tile against the holder it held when the campaign was
raised, and because the raise refuses while a campaign is live. It therefore
enters the state hash.[^6]

**Does a campaign need a record?** The scope test has three conditions.[^4] A
contributor could choose otherwise, and the choice is cheap to change: the
register shape, the objective choice and the cohort rule are each one function
with one caller. The reasoning is in the code and in this item. Two of the
three conditions fail, so no record is written. The decisions ADR-0144 already
holds govern every constraint that matters here.

**Changes.** None.

**Creates.** None.

**Blockers.** BLK-050 governs the cohort size and the campaign cadence,
because the rules of the downstream game are not written down.[^5] Both are
rows of the balance register, and the pass writes a provisional value into each
with its derivation. BLK-080 governs whether a cohort ever reaches a contested
tile in a running world.[^5] The pass sends the cohort and measures nothing
about the arrival. BLK-007 governs the cost figure of the raise, which follows
the population as the verbs it calls do.[^5]

**Precedent.** FND-320 records that nothing regenerates the type stub, so each
new reader and verb edits the stub by hand in the same commit.[^3] FND-048
records that a determinism test cannot see a draw keyed on the wrong field, so
the campaign draw needs a test for each field of its key.[^3] FND-051 records
that a fixture chosen for realism hides the defect, so the fixture here reaches
the extremes: two factions at peace, two at war, a busy unit beside an idle one,
and a campaign whose objective changes holder.[^3]

**Serves.** PRD-0049.[^7]

## Done when

- The world holds a bounded register of campaign rows for each faction. A row
  is plain data with declared padding and no boolean, and the register enters
  the state hash. The register size and the cohort size are rows of the
  balance register with a provisional value and a derivation.
- Three objective kinds are declared. The take kind and the relieve kind are
  raised. The wear kind is declared and nothing raises it, and the code says
  so.
- The controller draws once more for each faction on each tick, at the draw
  index past the relation draw, only when `war_between` holds for some pair and
  the faction holds no live campaign. One test for each key field proves that a
  change to the tick, the faction, the draw index and the seed changes the
  draw.
- The objective is chosen with no draw: the nearest enemy settlement by hex
  distance from the seat, and a tie goes to the lowest settlement slot.
- The cohort is the lowest identities among the idle units of the faction. An
  idle unit is one that nobody has sent anywhere. A test proves that a sent
  unit is not taken and that the lowest identities are.
- The raise sets the cohort to the soldier row through the set form of the type
  verb and sends it through `send_units_to`. The Python binding calls the same
  core function, so a caller can do what the controller does.
- A faction holds at most one live campaign. A test raises one and asserts that
  a second raise is refused.
- A campaign closes when the objective tile changes holder or when every unit
  of the cohort has fallen. A test changes the holder and asserts the close.
- No campaign is raised at peace, and one is raised at war. The fixture is two
  seated factions, and the pair is set to war through `set_relation`.
- The census holds `campaigns_raised` and `campaigns_won`. The demonstration
  prints a line when a campaign is raised and a line when one takes its
  objective, from one event log.
- The thread-count test and the golden state hash test pass at 1, 2 and 12
  threads with a campaign in flight.
- Every new test was put back to red once, and the commit body names the
  defect and the test.
- The type stub is edited by hand in the same commit as the new reader and the
  new verb.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Design: the living world game layer, sections 8 and 13. `docs/superpowers/specs/2026-09-05-living-world-game-layer-design.md`
[^2]: Balance register. `docs/reference/balance.md`
[^3]: Findings register, FND-320, FND-048 and FND-051. `docs/FINDINGS.md`
[^4]: Decision Record Scope, section 1. `.agents/rules/adr-scope.md`
[^5]: Blockers register, BLK-050, BLK-080 and BLK-007. `docs/BLOCKERS.md`
[^6]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^7]: Product registry. `docs/product/REGISTRY.md`
