# ADR-0205: Two factions meet when a unit has line of sight to another unit or a city

## Context

Factions in Cachette interact through relations, trade, and combat.[^1] The engine
also runs autonomous controllers that direct faction strategy, select rivals, and
declare war.[^2]

Previously, diplomacy had global visibility across the simulation world. Any faction
could move relations, propose treaties, or declare war against any other faction from
tick zero, even if neither faction had ever observed the other. The autonomous controller
could also select a distant, unseen faction as its rival or prey.[^2]

This violated the core design principle that a faction sees only what its own units
observe.[^3] Exploration gave no tactical advantage for diplomacy, and fog of war did
not constrain political interaction.

A product requirement specifies that factions must discover each other in the world
before they can interact.[^4] Units must pass within line of sight of each other or of
a settlement before diplomacy opens between their factions.

## Decision

**Two factions must meet in the world before they can interact. Meeting occurs when a
live unit of one faction observes a live unit or settlement of another faction.**

### D1. Factions start unmet and diplomacy is refused between unmet factions

Every distinct pair of factions starts in an unmet state. A faction always counts as
having met itself.

An attempt by a player or leader to move relations with an unmet faction is refused
with a typed error. Proposing a trade agreement or declaring war with an unmet faction
is refused.

A reviewer finds a violation when a caller can change relations with an unmet faction,
or when an unmet faction pair can execute a diplomatic treaty.

### D2. Contact is established symmetrically by line of sight to a unit or city

Two factions establish contact when:

1. A live unit of one faction has line of sight to a tile occupied by a live unit of
   the other faction.
2. A live unit of one faction has line of sight to a tile occupied by a settlement of
   the other faction.

Contact is symmetric. When faction A observes faction B, faction B also records contact
with faction A.

Contact is permanent. Once two factions meet, they remain in contact for the duration
of the simulation run.

A reviewer finds a violation when line of sight between two units or a unit and a city
does not record contact, or when contact is asymmetric.

### D3. Autonomous controllers ignore unmet factions for strategic interaction

An autonomous controller only evaluates factions that its faction has met.

When selecting rivals, prey, and victory threats, the controller filters out all unmet
factions. The controller never selects an unmet faction as a rival and never declares
war on an unmet faction.

A reviewer finds a violation when a controller selects an unmet faction as a rival or
prey, or attempts to declare war on an unmet faction.

### D4. Contact state is stored in the relation matrix and enters the state hash

The contact state is stored as a symmetric boolean matrix alongside the relation values.
Because the maximum faction count is capped by configuration, the storage overhead is
bounded by a small constant.

The contact matrix enters the world state hash. Two simulation runs with the same seed
and actions produce identical contact states and state hashes.

A reviewer finds a violation when contact state does not enter the state hash, or when
storage grows with entity population rather than the faction ceiling.

## Consequences

- Factions cannot engage in diplomacy or war until units make contact in the world.
- Exploration and scouting become necessary prerequisites for diplomacy.
- Autonomous controllers expand locally and engage neighbors rather than targeting
  unseen factions across the map.
- Contact checks run at the observation rebuild stage of each step, adding minimal
  overhead proportional to observed units and settlements.

## References

[^1]: ADR-0146, a faction relation is one signed integer per ordered pair, and a pass reads a threshold. `docs/adrs/accepted/adr-0146-a-faction-relation-is-one-signed-integer-per-ordered-pair-and-a-pass-reads-a-threshold.md`
[^2]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
[^3]: PRD-0001, a faction sees only what its own units observe. `docs/product/accepted/prd-0001-a-faction-sees-only-what-it-observes.md`
[^4]: PRD-0058, two factions must meet before they can interact. `docs/product/accepted/prd-0058-two-factions-must-meet-before-they-can-interact.md`
