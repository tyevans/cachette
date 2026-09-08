# Reinforcement Learning Parameters (Register)

This document is a **register**. It holds every value the learner seat needs
and does not have. A design document and a decision record cite a row here and
hold no figure.[^1] [^2]

**Every value in this register is unset.** The reward weighs the things a
faction gains. What those things are worth, and what winning is worth, are
rules of the downstream game, and one blocker holds those rules.[^3] A guessed
weight is a rule of a game nobody has written down.

A control plane module applies the weights. It states none of them, and it
refuses to run while one of them is unset. The refusal names each unset weight
and names the blocker.[^4]

## Format for a row

| Column | Holds |
|---|---|
| Value | The name of the value, as the module names it |
| Read by | The reader that reads it |
| Set | `unset`, and what sets it |
| Blocker | The blocker that governs it |
| Derivation | Empty until a value is written. Then how the value was reached, and the commit |

## The shaped terms

A shaped term weighs the change of one quantity since the previous decision.
Each name below names one field of the observation array of a faction, and the
schema of the world declares where that field sits.[^5]

**A weight multiplies the bounded value the schema declares.** Every position
of the observation array is a Q16.16 value between minus one and one, so 65536
is one unit and no row carries a raw count. A term that names a compressed
magnitude moves less at a large quantity than at a small one, because the map
is a logarithm. A caller that writes a value into a row below states the unit
in the derivation column.

| Value | Read by | Set | Blocker | Derivation |
|---|---|---|---|---|
| `held_tiles`, the weight of the compressed held tile count | The reward of a faction | unset, the project owner | BLK-050 | |
| `domination_progress`, the weight of the seat share of the faction | The reward of a faction | unset, the project owner | BLK-050 | |
| `live_units`, the weight of the compressed live unit count | The reward of a faction | unset, the project owner | BLK-050 | |
| `population`, the weight of the compressed population total | The reward of a faction | unset, the project owner | BLK-050 | |
| `store_total`, the weight of the compressed store total | The reward of a faction | unset, the project owner | BLK-050 | |
| `best_renown`, the weight of the compressed best renown | The reward of a faction | unset, the project owner | BLK-050 | |
| `wonder_progress`, the weight of the wonder work share | The reward of a faction | unset, the project owner | BLK-050 | |
| `wonder_track_progress`, the weight of the wonder share of the faction | The reward of a faction | unset, the project owner | BLK-050 | |

A caller may weigh any other field of the observation array that holds one
position. The rows above are the terms the project reserved, and the module
names the same eight.

## The terminal outcomes

A terminal outcome is paid once, when the run ends. Three outcomes end a run.

**There is no elimination outcome.** A faction that loses every unit and every
person keeps its held ground and its seat, so it may still win at the tick
limit. A finding holds the measurement.[^6]

| Value | Read by | Set | Blocker | Derivation |
|---|---|---|---|---|
| `won`, what the faction gets when a reader names it the winner | The reward of a faction | unset, the project owner | BLK-050 | |
| `lost`, what the faction gets when a reader names another winner | The reward of a faction | unset, the project owner | BLK-050 | |
| `drawn`, what the faction gets when the run reaches the tick limit with no winner | The reward of a faction | unset, the project owner | BLK-050 | |

## The observation layout

A draft record makes the width of the observation a constant, and it holds no
figure of its own.[^9] The width is a function of the rows below, so a change to
one of them changes the width and retires every policy trained before it. A
decision register holds the open choice of what each row should be, and a
research report derives a recommendation for each.[^10] [^11]

**Every row below is unset.** No builder produces the array the record
describes, so no value is in force. A pass that writes a first value into a row
states the derivation and marks the value provisional.

| Value | Read by | Set | Blocker | Derivation |
|---|---|---|---|---|
| Ring count of the spatial stack | The observation builder | unset, the record that adopts a layout revision | — | |
| Sector count of a ring, for each ring | The observation builder | unset, the record that adopts a layout revision | — | |
| Channel count of a spatial cell | The observation builder | unset, the record that adopts a layout revision | — | |
| Distance band of each ring, in tiles | The observation builder | unset, the record that adopts a layout revision | — | |
| Token count of each token set | The observation builder | unset, the record that adopts a layout revision | — | |
| Channel count of a token, for each token set | The observation builder | unset, the record that adopts a layout revision | — | |
| Bit cap of a compressed magnitude | The observation builder | unset, the record that adopts a layout revision | — | |
| Class count of the goods taxonomy | The observation builder | unset, the record that adopts a layout revision | — | |
| Class count of the unit type taxonomy | The observation builder | unset, the record that adopts a layout revision | — | |
| Reserved position count | The observation builder | unset, the record that adopts a layout revision | — | |
| Layout revision integer | The policy fit of the control plane | unset, the record that adopts a layout revision | — | |
| Window length of every change position, in ticks | The observation builder | unset, the record that adopts a layout revision | — | |

## What this register does not hold

**It holds no discount.** A discount belongs to the learning algorithm, and the
product record says plainly that it chooses no learning algorithm.[^7] The
person brings one.

**It holds no cost figure.** One blocker governs every cost figure this project
holds, and it says which figures are measured and which are derived.[^8] A cost
of the learner seat belongs in a row under that blocker when a run on the
target platform measures it.

## References

[^1]: Documentation Rules. `.agents/rules/documentation.md`
[^2]: Decision Record Scope, section 4.1. `.agents/rules/adr-scope.md`
[^3]: Blockers register, BLK-050. `docs/BLOCKERS.md`
[^4]: The reward of a faction. `python/cachette/learn/reward.py`
[^5]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables the engine owns, decision D1. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
[^6]: Findings register, FND-583. `docs/FINDINGS.md`
[^7]: PRD-0056, a learner plays one faction against the controllers. `docs/product/accepted/prd-0056-a-learner-plays-one-faction-against-the-controllers.md`
[^8]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^9]: ADR-0195, the observation of a faction is a fixed-width scale-free table in an egocentric frame, decisions D1 and D9. `docs/adrs/draft/adr-0195-the-observation-of-a-faction-is-a-fixed-width-scale-free-table.md`
[^10]: Decisions register, DEC-282. `docs/DECISIONS.md`
[^11]: Research report 42, what a policy should be able to see, section 9. `docs/research/reports/42-what-a-policy-should-be-able-to-see.md`
