---
id: 0542
title: Aim the controller at a rival that nears a win
status: complete
created: 2026-09-10
implements: [ADR-0144 D1, ADR-0144 D2, ADR-0144 D4, ADR-0144 D5, ADR-0146 D5, ADR-0174 D1, ADR-0181 D5, ADR-0204 D4, ADR-0206 D2]
changes: []
creates: []
serves: [PRD-0057]
blocked-by: []
---

## Why

In a game of three factions, a trained policy wins about a third of its games
with one tactic. It puts every unit on one wonder, and it does nothing else.

The built-in controller names two targets. The rival is the other faction with
the most held ground. The prey is the weakest faction that the controller
overmatches. A faction on one wonder is neither, so no controller moves against
it, and nothing disrupts the wonder.

The product record asks that a player who chooses one way to win can lose to a
player who chooses another.[^1] The project owner approved the behaviour on
10 September 2026. When a rival nears any win path, that rival becomes the
target. On the wonder path, the march aims at the city whose ground holds the
wonder.

## Impact review

**Governed by.** ADR-0144 D1 keeps the evaluation of one faction off a pass over
the units or the tiles. D2 sends every command through a verb a caller can
call. D4 and D5 fix the draws and the order of the commands.[^2] ADR-0146 D5
sends every relation move through the one gated verb.[^3]

ADR-0174 D1 makes a finished wonder a win path.[^4] ADR-0204 D4 defines the
reading of each win path as a bounded share that reaches one on the tick its
reader fires.[^5] ADR-0181 D5 keeps a faction that left the game out of every
win reader.[^6] ADR-0206 D2 resets part-built wonder work when its ground
changes holder, so a march that takes the city undoes the wonder.[^7]

ADR-0199 D2 lets a controller choice carry the whole-frame place value. The
campaign choice keeps that value, so the verb set does not move.[^8]

**Changes.** None. The rule is a policy of the built-in controller. It is cheap
to change, and a reader of the code sees why, so it needs no record.[^9]

**Creates.** None. The item took no record number and no register number.

**Blockers.** BLK-050 governs the share at which a rival counts as near a win.
The share is a balance row with a provisional value, and the code states it as
a parameter.[^10] BLK-160 holds whether the standing of a rival toward a win
is public. **The controller reads the whole world today**, for its rival and
its prey. The threat test reads the whole world as well, and this item does
not change that rule.[^11]

**Precedent.** FND-739 records one weight that decided two choices.[^12] The
move against a win threat draws nothing, so it takes no weight, and the war
weight and the renown weight each still decide one choice.

## Done when

- A rival whose reading on the wonder path reaches the share becomes the target
  of the relation move, on every tick.
- A rival whose reading on another win path reaches the share becomes the
  target, ahead of a prey.
- A rival below the share does not change the target.
- A march on a rival that nears the wonder path aims at the city whose ground
  holds the wonder, and not at a nearer city of that rival.
- A share of zero takes the rule out of the game.
- The reading the controller takes equals the progress the observation
  publishes to each faction about itself.
- Each defect put back turns a named test red.
- The golden state hash is regenerated from the merged source, because the
  change moves the controller row and the controller behaviour.
- The whole check command runs green.

## Outcome

The built-in controller aims at a rival that nears a win. One commit holds the
work.[^13]

**What was done.** The world computes the reading of every faction on every win
path. It uses the readers that already state each numerator and each
requirement. Each faction row holds a win threat share. The rival with the
highest reading at or above that share is the win threat. The threat outranks
the prey and the rival by held ground, and the move against it draws nothing.
The war weight and the renown weight therefore each still decide one
choice.[^12]

A campaign against a threat in the war band aims at one city. When the threat
nears the wonder path, that city is the city that the wonder lookup of item
0540 names. Otherwise it is the nearest city of the threat.

**The share.** The default share is five eighths. The controller module states
it once, as `WIN_THREAT_SHARE_DEFAULT`.[^14] The balance register holds the row
and its derivation, and BLK-050 governs the value.[^15] [^10] A share of zero
takes the rule out of the game.

**The fog.** The controller reads the full world state for its rival, its prey
and its win threat. BLK-160 still holds whether the standing of a rival toward
a win is public. This item leaves that row open.[^11]

**The golden state hash.** The controller row gained the share, so the golden
state hash moved. A separate commit regenerated it after the merge.[^16]

**What changed from the plan.** The controller reads two readers that were
private to the observation and to the game end. Each became visible to the
crate, and neither changed.

**What was left undone.** No binding exposes the share to Python. A live
campaign against another faction runs until it closes, and the controller does
not re-aim it at a new threat. The territory reader still ranks the factions at
the tick limit, so the territory reading reaches one only when a faction holds
all passable ground.[^5]

**Registers.** The balance register holds the win threat share row. The item
used no register number. No finding, decision or blocker opened or closed.

**Gates.** The dispatcher runs the whole check command on the settled tree.
This item does not state the result.

## References

[^1]: PRD-0057, more than one way to win decides a game. `docs/product/shaped/prd-0057-more-than-one-way-to-win-decides-a-game.md`
[^2]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decisions D1, D2, D4 and D5. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
[^3]: ADR-0146, a faction relation is one signed integer per ordered pair, decision D5. `docs/adrs/accepted/adr-0146-a-faction-relation-is-one-signed-integer-per-ordered-pair-and-a-pass-reads-a-threshold.md`
[^4]: ADR-0174, a wonder is a win path and a stock total is not, decision D1. `docs/adrs/draft/adr-0174-a-wonder-is-a-win-path-and-a-stock-total-is-not.md`
[^5]: ADR-0204, every win path holds a bar of its own, decision D4. `docs/adrs/draft/adr-0204-every-win-path-holds-a-bar-of-its-own.md`
[^6]: ADR-0181, a faction that holds no site and no unit leaves the game, decision D5. `docs/adrs/draft/adr-0181-a-faction-that-holds-no-site-and-no-unit-leaves-the-game.md`
[^7]: ADR-0206, a part-built wonder decays when nobody works it, decision D2. `docs/adrs/draft/adr-0206-a-part-built-wonder-decays-when-nobody-works-it.md`
[^8]: ADR-0199, a verb names a place by a cell of the egocentric frame, decision D2. `docs/adrs/draft/adr-0199-a-verb-names-a-place-by-a-cell-of-the-egocentric-frame.md`
[^9]: Decision Record Scope, section 1. `.agents/rules/adr-scope.md`
[^10]: Blockers register, BLK-050. `docs/BLOCKERS.md`
[^11]: Blockers register, BLK-160. `docs/BLOCKERS.md`
[^12]: Findings register, FND-739. `docs/FINDINGS.md`
[^13]: Commit 21d7a834, aim the controller at a rival that nears a win. It merged in e25c343b.
[^14]: The controller module. `crates/cachette-core/src/controller.rs`
[^15]: Balance register, the win threat share. `docs/reference/balance.md`
[^16]: Commit d80477a1, regenerate the golden state hashes for the win threat and the wonder watch.
