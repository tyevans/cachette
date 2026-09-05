# Balance Register

This document is a **register**. It holds every game value of the living world
game layer: every threshold, rate, limit, target and share. A design document
and a decision record cite a row here and hold no figure.[^1] [^2]

**Every value in this register is unset until pass 10 measures it.** Pass 10 is
the balance harness. It runs a fixed seed set to game end and checks four
statements against the shares below.[^3] A pass before pass 10 may write a
first value into its row, and it must then fill the derivation column with how
the value was chosen and mark the value provisional. Do not invent a value, and
do not leave the derivation column empty when the value column holds a number.

**A cost figure is behind an open blocker.** A row that is a cost in ticks or a
cost in work is governed by BLK-007, and the blocker column names it.[^4] Two
rows are behind BLK-130, which asks what weather is worth.[^5] One row is behind
BLK-150, which asks what raises and lowers renown.[^6]

## Format for a row

| Column | Holds |
|---|---|
| Value | The name of the value |
| Read by | The pass or the reader that reads it |
| Set | `unset`, and the pass that sets it |
| Blocker | The blocker that governs it, or a dash |
| Derivation | Empty until a value is written. Then how the value was reached, and the commit |

## The controller

| Value | Read by | Set | Blocker | Derivation |
|---|---|---|---|---|
| Controller evaluations per faction per tick | The controller stage | unset, pass 10 | BLK-007 | Provisional default of 2 written by pass 1, the smallest count at which the draw index has a second value to differ from. Pass 10 measures it. |
| Weight vector range, the lowest and highest weight | The seeding layer, when it draws a faction vector | unset, pass 10 | — | Provisional range of 1 to 8 written by pass 1. The build weight over the range top plus the build weight gives a build order a share between one ninth and one half, so every faction both gathers and builds. Pass 10 measures it. |
| Board size, the advertisement rows per faction | The trade board | unset, pass 10 | — | |
| Advertisement schedule, period and phase | The controller, when it writes its board | unset, pass 10 | — | |
| Surplus mark, the store above which a site offers | The controller pricing rule | unset, pass 10 | — | |
| Land list bound, the most tiles in one land consideration | The trade verbs, when they refuse a land offer | unset, pass 10 | — | |
| Campaign register size per faction | The campaign register | unset, pass 10 | — | |

## The relation

| Value | Read by | Set | Blocker | Derivation |
|---|---|---|---|---|
| Alliance edge | Every reader of a band | unset, pass 10 | — | |
| Peace edge | Every reader of a band | unset, pass 10 | — | |
| War edge | Every reader of a band, and the declaration event | unset, pass 10 | — | |
| Step on a contract delivered in full | The delivery pass | unset, pass 10 | — | |
| Step on a contract that fails | The delivery pass at a deadline | unset, pass 10 | — | |
| Step when a unit falls to the other side | The contest pass | unset, pass 10 | — | |
| Step when a unit converts away | The conversion pass | unset, pass 10 | — | |
| Step when a storm falls on the ground of the other | The weather verb | unset, pass 10 | BLK-130 | |
| Drift step toward peace | The drift | unset, pass 10 | — | |
| Drift schedule, period and phase | The drift | unset, pass 10 | — | |
| Bound on one `move_relation` step | The relation verb | unset, pass 10 | — | |
| Permitted bands for a conversion | The conversion pass | unset, pass 10 | — | |
| Band below which a holder refuses a guest | The movement pass | unset, pass 10 | — | |

## The game end

| Value | Read by | Set | Blocker | Derivation |
|---|---|---|---|---|
| Tick limit | The territory reader | unset, pass 10 | BLK-007 | Provisional default of 2000 written by pass 1, so that a run of the demonstration world ends inside a few minutes and no determinism scenario reaches it. Pass 10 measures it. |
| Stock target | The wealth-or-wonder reader | unset, pass 10 | — | |
| Renown target | The renown reader | unset, pass 10 | BLK-150 | |
| Census tick count, the ticks the gate drives before it reads the census | The census gate | unset, pass 10 | BLK-007 | |

## The seeding layer

| Value | Read by | Set | Blocker | Derivation |
|---|---|---|---|---|
| Founding group, the people each faction founds with | The seeding layer, when it founds the run | unset, pass 10 | — | Provisional default of 64 written by pass 1, the constant the demonstration passed to the founding verb before the seeding moved into the engine. Pass 10 measures it. |
| Luxury deposits, the tiles the seeding layer places a luxury on | The seeding layer, when it places the luxuries | unset, pass 10 | — | Provisional default of 8 written by pass 1, two deposits for each faction of the demonstration world. Pass 10 measures it. |

## Unit types

| Value | Read by | Set | Blocker | Derivation |
|---|---|---|---|---|
| Default table, five rows by eight columns | The seeding layer | unset, pass 10 | — | |

The eight columns are attack, armour, gather rate, build rate, carry capacity,
move cost scale, command reach and weather reach. The five rows are worker,
soldier, merchant, leader and one open row. Each cell is one value, and this
register holds them as one row until a pass writes them, because forty empty
rows say nothing that one does not.

## Upgrades

| Value | Read by | Set | Blocker | Derivation |
|---|---|---|---|---|
| Full condition, per kind | The build pass on completion, and the repair clamp | unset, pass 10 | — | |
| Wear step from a hostile unit | The wear pass | unset, pass 10 | — | |
| Wear step from wet ground | The wear pass | unset, pass 10 | BLK-130 | |
| Wall work | The build pass | unset, pass 10 | BLK-007 | |
| Wall harm absorption | The contest pass | unset, pass 10 | — | |
| Wall move cost raise for a unit whose faction does not hold the tile | The movement pass | unset, pass 10 | — | |
| Wonder work | The build pass | unset, pass 10 | BLK-007 | |
| Store work | The build pass | unset, pass 10 | BLK-007 | |
| Store capacity raise | The site store | unset, pass 10 | — | |

## Weather

| Value | Read by | Set | Blocker | Derivation |
|---|---|---|---|---|
| Flood mark, the ground water above which a cell is flooded | Every weather harm | unset, pass 10 | BLK-130 | |
| Spoil share, the integer share of a store lost per tick on a flooded cell | The spoilage | unset, pass 10 | BLK-130 | |
| Unit-loss draw bound, the most units one draw names per flooded cell | The unit loss draw | unset, pass 10 | BLK-130 | |
| Move cost step on wet ground | The movement pass | unset, pass 10 | BLK-130 | |

## Balance shares

| Value | Read by | Set | Blocker | Derivation |
|---|---|---|---|---|
| Win-path share, the most seeds one path may win | The balance harness, statement 1 | unset, pass 10 | — | |
| Seat share, the most seeds one seat may win | The balance harness, statement 2 | unset, pass 10 | — | |
| End share, the fewest seeds that must end before the tick limit | The balance harness, statement 3 | unset, pass 10 | BLK-007 | |
| Seed set | The balance harness | unset, pass 10 | — | |

## What belongs here

A value that a game reads and that a measurement can change. A threshold, a
rate, a step, a limit, a target, a share, a schedule, a bound.

## What does not belong here

- A cost shape or a scale constant. Those are in the budgets register.[^1]
- A measured cost on the target platform. Those are in the target platform
  register.[^7]
- A structural constant, such as the faction ceiling. State it in the record
  that needs it.
- A decision. A decision goes in a record.

## References

[^1]: Budgets and costs. `docs/reference/budgets.md`
[^2]: Decision Record Scope, section 4.1. `.agents/rules/adr-scope.md`
[^3]: Design: the living world game layer, section 10.2. `docs/superpowers/specs/2026-09-05-living-world-game-layer-design.md`
[^4]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^5]: Blockers register, BLK-130. `docs/BLOCKERS.md`
[^6]: Blockers register, BLK-150. `docs/BLOCKERS.md`
[^7]: Target platform costs. `docs/reference/graviton-costs.md`
