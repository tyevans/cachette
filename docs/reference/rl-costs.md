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

**A weight multiplies the raw value the schema declares.** The store total and
the best renown are Q16.16 values as raw integers, so a weight on one of them
carries a factor of 65536 that a weight on a tile count does not. A caller that
writes a value into a row below states the unit in the derivation column.

| Value | Read by | Set | Blocker | Derivation |
|---|---|---|---|---|
| `held_tiles`, the weight of one more tile the faction holds | The reward of a faction | unset, the project owner | BLK-050 | |
| `seats_held`, the weight of one more seat the faction holds | The reward of a faction | unset, the project owner | BLK-050 | |
| `live_units`, the weight of one more live unit | The reward of a faction | unset, the project owner | BLK-050 | |
| `population`, the weight of one more person | The reward of a faction | unset, the project owner | BLK-050 | |
| `store_total`, the weight of one raw Q16.16 unit of store | The reward of a faction | unset, the project owner | BLK-050 | |
| `best_renown`, the weight of one raw Q16.16 unit of renown | The reward of a faction | unset, the project owner | BLK-050 | |
| `wonder_progress`, the weight of one unit of work toward a wonder | The reward of a faction | unset, the project owner | BLK-050 | |
| `wonder_claim`, the weight of the victory claim the wonder reader compares | The reward of a faction | unset, the project owner | BLK-050 | |

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
[^6]: Findings register, FND-580. `docs/FINDINGS.md`
[^7]: PRD-0056, a learner plays one faction against the controllers. `docs/product/accepted/prd-0056-a-learner-plays-one-faction-against-the-controllers.md`
[^8]: Blockers register, BLK-007. `docs/BLOCKERS.md`
