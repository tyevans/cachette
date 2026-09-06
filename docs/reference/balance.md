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
cost in work is governed by BLK-007, and the blocker column names it.[^4] Several
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
| Weight vector range, the lowest and highest weight | The seeding layer, when it draws a faction vector | unset, pass 10 | — | The vector now carries five weights, the fifth being the settle weight of item 0485. Provisional range of 1 to 8 written by pass 1. The build weight over the range top plus the build weight gives a build order a share between one ninth and one half, so every faction both gathers and builds. Pass 10 measures it. |
| Board size, the advertisement rows per faction | The trade board | unset, pass 10 | — | Provisional default of 8 written by pass 6, above the three goods a board can name today, so a faction states every good it holds and still has room when the good set grows. A caller sets it with `set_board_rows`. Pass 10 measures it. |
| Advertisement schedule, period and phase | The controller, when it writes its board | unset, pass 10 | BLK-007 | Provisional default of period 10 and phase 0 written by pass 6, the economy schedule default, so a board is rewritten once for each application of the rates that fill the store it reads. A caller sets it with `set_advertisement_schedule`. Pass 10 measures it. |
| Surplus mark, the store above which a site offers | The controller pricing rule | unset, pass 10 | BLK-050 | Provisional default of 8 written by pass 6, the default founding group, so a faction offers only when it holds more than one whole unit for each person it settled with. The rules of the downstream game are not written down, so BLK-050 governs it. A caller sets it with `set_surplus_mark`. Pass 10 measures it. |
| Carriers per contract, the units one faction sends | The controller, when a contract binds | unset, pass 10 | BLK-007 | Provisional default of 2 written by pass 6, the smallest count above one, so a contract survives the loss of one carrier. A count of zero assigns none. A caller sets it with `set_carriers_per_contract`. Pass 10 measures it. |
| Contract term, the ticks a controller contract runs for | The controller, when it opens a negotiation | unset, pass 10 | BLK-007 | Provisional default of 200 written by pass 6, twenty applications of the default economy schedule, so a carrier has time to cross the ground between two sites before the deadline. A caller sets it with `set_contract_term`. Pass 10 measures it. |
| Land list bound, the most tiles in one land consideration | The trade verbs, when they refuse a land offer | unset, pass 10 | — | |
| Campaign register size per faction | The campaign register | unset, pass 10 | — | Provisional default of 2 written by pass 7: one row for the live campaign, and one so that the last closed campaign stays readable after the next raise. A faction holds at most one live campaign, so one row would serve and two keeps the outcome. Pass 10 measures it. |
| Campaign cohort size, the units one raise takes | The controller, when it raises a campaign | unset, pass 10 | BLK-050 | Provisional default of 4 written by pass 7. Pass 7 took half the founding group of a test fixture, so that a raise leaves the site peopled. The project owner set the founding group to 2 on 5 September 2026, so a cohort of 4 now takes more units than a faction founds with. The rules of the downstream game are not written down, so BLK-050 governs it. A caller sets it with `set_campaign_cohort_size`. Pass 10 measures it. |
| Campaign cadence, the draw that raises one | The controller, when it draws for a campaign | unset, pass 10 | BLK-050 | Provisional shape written by pass 7: one keyed draw for each faction on each tick at war, yes with probability war weight over the weight range top plus the war weight, the shape the relation move already uses. No separate period exists. BLK-050 governs it. Pass 10 measures it. |
| Campaign deadline, the ticks a live campaign runs before it closes unmet | The campaign register | unset, pass 10 | BLK-007 | Provisional default of 500, declared as `DEADLINE_DEFAULT`. It governs the stuck campaign alone: a campaign that takes its objective or loses its cohort closes earlier. The greatest hex distance across the harness world at extent 256 is below 256 tiles and the movement pass admits one tile step a tick, so 500 ticks is about twice the worst crossing of the whole world. A cohort that has not arrived in that time is stuck and not slow. The earlier default of 2000 left a faction at war for one tick in four. A caller sets it with the world accessor. Pass 10 measures it. |
| Settle weight, what a faction gives the settle option | The controller, when it draws for a settling | unset, pass 10 | BLK-050 | Provisional weight drawn from the seeded faction weight vector, in the way the war weight and the build weight are drawn. It is the fifth weight of the vector. Written by item 0485. Pass 10 measures it. |

## The choice

| Value | Read by | Set | Blocker | Derivation |
|---|---|---|---|---|
| Own-ground weight, what a unit of a faction gives an option on ground its own faction holds | The choice pass, when it scores an option | unset, pass 10 | BLK-050 | Written by the item that implements ADR-0156. The engine states no value, and the built-in controller draws the strength of its preference from the seeded faction weight vector. The rules of the downstream game are not written down, so BLK-050 governs it. Pass 10 measures it. |
| Rival-ground weight, what a unit of a faction gives an option on ground another faction holds | The choice pass, when it scores an option | unset, pass 10 | BLK-050 | Written by the item that implements ADR-0156. It is the lowest of the three, and it is never zero, because a zero would make the term a fence and ADR-0156 D5 refuses a fence. The rules of the downstream game are not written down, so BLK-050 governs it. Pass 10 measures it. |
| Unheld-ground weight, what a unit of a faction gives an option on ground nobody holds | The choice pass, when it scores an option | unset, pass 10 | BLK-050 | Written by the item that implements ADR-0156. It sits between the other two, and a cell that names more than one faction takes it as well. The rules of the downstream game are not written down, so BLK-050 governs it. Pass 10 measures it. |

## The plan

| Value | Read by | Set | Blocker | Derivation |
|---|---|---|---|---|
| Plan bound, the projects one faction may hold | The plan register, when a write asks for a row | unset, pass 10 | BLK-050 | Provisional default of 40 written by item 0488. A path across the default search radius of 8 spans at most 17 tiles, so one whole path fits twice over and a hand-zoned project still finds a row. A caller sets it with `set_plan_rules`. The rules of the downstream game are not written down, so BLK-050 governs it. Pass 10 measures it. |
| Projects one solver pass writes | The solver, at the controller stage | unset, pass 10 | BLK-050 | Provisional default of 17 written by item 0488, the longest path the default path pass count resolves exactly, so one pass writes one whole path and never part of one. A caller sets it with `set_plan_rules`. BLK-050 governs it. Pass 10 measures it. |
| Solver pass count | The solver, at the controller stage | unset, pass 10 | BLK-050 | Provisional default of 2 written by item 0488, the smallest count at which a second pass reads what the first wrote. A caller sets it with `set_plan_rules`. BLK-050 governs it. Pass 10 measures it. |
| Path relaxation pass count | The road path search | unset, pass 10 | BLK-050 | Provisional default of 17 written by item 0488, twice the default search radius plus one. A pass advances the frontier by at least one step, so every path of that many steps is the cheapest one. A longer detour gives a path that is deterministic and is not the cheapest. A caller sets it with `set_plan_rules`. BLK-050 governs it. Pass 10 measures it. |
| Road search radius, the hex steps a path may span | The road path search, when it builds its window | unset, pass 10 | BLK-050 | Provisional default of 8 written by item 0488, twice the base reach of a city, so two cities whose ground touches can be joined and a pair further apart yields no project. The window it describes holds 217 tiles. A caller sets it with `set_plan_rules`. BLK-050 governs it. Pass 10 measures it. |

## The relation

| Value | Read by | Set | Blocker | Derivation |
|---|---|---|---|---|
| Alliance edge | Every reader of a band | unset, pass 10 | — | Provisional default of 8 written by pass 3, the same distance above the peace edge as the war edge is below it, so the drift takes equal time back from either side. Pass 10 measures it. |
| Peace edge | Every reader of a band | unset, pass 10 | — | Provisional default of 0 written by pass 3. Every entry of a new world holds it, so two factions that never met are at peace and the contest resolves nothing between them. Pass 10 measures it. |
| War edge | Every reader of a band, and the declaration event | unset, pass 10 | — | Provisional default of -8 written by pass 3: eight controller moves of one step from peace declare a war, and eight drift periods end it. Pass 10 measures it. |
| Step on a contract delivered in full | The delivery pass | unset, pass 10 | — | Provisional default of 2 written by pass 3, twice the fallen step, so one honoured contract outweighs one skirmish casualty. Both directions move. Pass 10 measures it. |
| Step on a contract that fails | The delivery pass at a deadline | unset, pass 10 | — | Provisional default of 2 written by pass 3, equal to the delivered step, so a default undoes one delivery. Only the party that was owed moves. Pass 10 measures it. |
| Step when a unit falls to the other side | The contest pass | unset, pass 10 | — | Provisional default of 1 for each unit written by pass 3, the smallest whole step, because a battle of many units already sums to a large move. The victim moves toward the faction that delivered the most harm. Pass 10 measures it. |
| Step when a unit converts away | The conversion pass | unset, pass 10 | — | Provisional default of 1 for each unit written by pass 3, for the reason the fallen step has. The old faction moves toward the leader. Pass 10 measures it. |
| Step when a storm falls on the ground of the other | The weather verb | unset, pass 10 | BLK-130 | Unset and unwired. The rules struct holds the field at zero and nothing reads it. A god inflicts weather only on ground its own faction holds, so no source for the cause exists before pass 5. |
| Drift step toward peace | The drift | unset, pass 10 | — | Provisional default of 1 written by pass 3, the smallest whole step. An entry below the peace edge moves up and stops at it. An entry at or above the alliance edge moves down and stops one below it. Pass 10 measures it. |
| Drift schedule, period and phase | The drift | unset, pass 10 | — | Provisional default of period 10 and phase 0 written by pass 3, the economy schedule default, so the drift and the rates share a cadence. Pass 10 measures it. |
| Bound on one `move_relation` step | The relation verb | unset, pass 10 | — | Provisional default of 4 written by pass 3, half the distance from peace to war, so a leader needs two declarations to reach war from peace and one drift period undoes a quarter of one. Pass 10 measures it. |
| Permitted bands for a conversion | The conversion pass | unset, pass 10 | — | Provisional default written by pass 3: the leader converts only below the peace edge, which is the tension band and the war band. A leader at peace with a faction converts none of its units. The code holds it as one edge equal to the peace edge. Pass 10 measures it. |
| Band below which a holder refuses a guest | The movement pass | unset, pass 10 | — | **The refusal is a window and no longer a floor, and the earlier reading of this row is stale.** It said that a border in tension or at war closes. War now opens the border that tension closes, so a holder refuses a guest only while the pair sits between the guest edge and the war edge, and a pair at or below the war edge admits a guest without protecting one. The code holds the guest edge equal to the peace edge, declared as `PEACE_EDGE_DEFAULT`, and the war edge below it as `WAR_EDGE_DEFAULT`. ADR-0167 holds the record. Pass 10 measures both. |

## The game end

| Value | Read by | Set | Blocker | Derivation |
|---|---|---|---|---|
| Tick limit | The territory reader | unset, pass 10 | BLK-007 | Provisional default of 5000. **The territory reader fired in no seed at a horizon of 20000 ticks**, because the wealth-or-wonder reader ended every game first, between tick 1059 and tick 16850. The engine constant holds 5000 and this row holds 5000, so the two agree today; the project owner asked for 20000 on 5 September 2026 and neither site holds it. The findings register holds the sweep.[^14] The project owner set the horizon of a game to 5000 ticks on 5 September 2026, and named domination and territory as the primary win paths. The earlier default of 2000 came from pass 1, so that a run of the demonstration world ends inside a few minutes. A run of 8 seeds at 5000 ticks costs 2 minutes 46 seconds on one development machine (ty001-ubuntu, x86-64). Pass 10 measures it. |
| Stock target | The wealth-or-wonder reader | unset, pass 10 | — | Provisional default of 28672 whole units, as a raw Q16.16 sum over every commodity of every settlement of the faction. The project owner asked for a much higher bar on 5 September 2026, because the wealth path fired too early, and set the horizon of a game to 5000 ticks. The earlier default of 4096 came from pass 8, and its derivation said the demonstration store climbs to hundreds. That derivation was wrong. Measured on 5 September 2026 over the 8 default seeds on one development machine (ty001-ubuntu, x86-64), extent 256, four factions: at 4096 the wealth path ended every game between tick 437 and tick 1340, and no wonder finished, so the stock clause fired every time. At 28672 the wealth path ends 3 of the 8 games, between tick 2960 and tick 3980, and the territory reader ends the other 5. The new value is seven times the old one. **The path has a hard ceiling.** One faction founds one settlement, the commodity count is one, and a store is a `Fix32`, so the stock total of a faction clamps at 32767 whole units. A target above that never fires, which bounds the multiple below eight. The value is seven eighths of that ceiling, which keeps the path reachable and leaves room below the clamp. **No target the engine permits puts this path out of reach.** Measured on 5 September 2026 by item 0481 over 32 seeds on one development machine (ty001-ubuntu, x86-64), extent 256, four factions, tick limit 20000, with `scripts/balance_sweep.py`. A development-machine run, and no evidence about the target platform. At 28672 the stock clause ended 29 of the 32 seeds and the wonder clause ended the other 3, so the wealth-or-wonder path ended 32 of 32. The store rises steadily, reads a median of 22 percent of the target at tick 2000, and reaches the clamp, so every legal target is crossed given enough ticks. The findings register holds the reading and names the three engine changes that could answer it.[^15] Pass 10 measures it. |
| Renown target | The renown reader | unset, pass 10 | BLK-150 | Provisional default of 100 whole units, declared as `RENOWN_TARGET`, written by pass 8. **The earlier reading of this row is stale.** It said that no pass writes renown, so the reader could fire only when the control plane wrote the column. The contest now gives the champion of a faction one share for each unit that faction fells, so the path has a source in the engine. The value is a placeholder under the blocker and not a choice. Pass 10 measures it. |
| Renown for each unit felled, the share a champion earns | The renown pass | unset, pass 10 | BLK-150 | Provisional default of one quarter of a point, declared as `RENOWN_PER_FELL` in the contest module. Against a renown target of 100 points a champion reaches the target after 400 enemy units fall to its faction while that champion lives. A whole point for each unit would put renown ahead of domination in most runs, and domination is the path the game is meant to resolve on. A hundredth of a point would leave the source inert. The renown goes to one character and never to the faction, because the reader takes the highest renown among the live characters of a faction, and renown spread over every character would never reach the target. Written by item 0479. Pass 10 measures it. |
| Census tick count, the ticks the gate drives before it reads the census | The census gate | unset, pass 10 | BLK-007 | |

## The holding

| Value | Read by | Set | Blocker | Derivation |
|---|---|---|---|---|
| Base reach, the hex steps a city holds with no finished upgrade | The holding rewrite | unset, pass 10 | BLK-050 | Provisional default of 4 written by item 0484. A disc of radius 4 holds 61 tiles. Item 0484 chose the radius against a founding group of 64, so a founded faction held about one tile for each person it founded with. The project owner set the founding group to 2 on 5 September 2026, so that derivation no longer holds and the radius now has no reason behind it. Pass 10 measures it. |
| Upgrades per reach step, the finished upgrades inside the ground that earn one step | The holding rewrite | unset, pass 10 | BLK-050 | Provisional default of 4 written by item 0484, the base reach again, so the first step of growth asks for as many upgrades as the base has steps. Pass 10 measures it. |
| Reach bound, the reach a city never passes | The holding rewrite | unset, pass 10 | BLK-050 | Provisional default of 8 written by item 0484, twice the base, so upgrades at most double how far a city reaches and a disc of radius 8 holds 217 tiles. Pass 10 measures it. |
| Lease raise step, the count one tick of use adds | The lease pass | unset, pass 10 | BLK-050 | Provisional default of 1, declared as `LEASE_RAISE_STEP_DEFAULT`. It is the smallest step above nothing, so a lease is built by staying rather than by arriving. A caller sets it with `set_lease_rules`. Pass 10 measures it. |
| Lease lower step, the count one tick of another faction takes away | The lease pass | unset, pass 10 | BLK-050 | Provisional default of 4, declared as `LEASE_LOWER_STEP_DEFAULT`, four times the raise step, so a tile in contest changes hands faster than it was won. A caller sets it with `set_lease_rules`. Pass 10 measures it. |
| Lease decay step, the count one decay takes away | The lease decay | unset, pass 10 | BLK-050 | Provisional default of 1, declared as `LEASE_DECAY_STEP_DEFAULT`, the raise step again, so one decay undoes one tick of use. A caller sets it with `set_lease_rules`. Pass 10 measures it. |
| Lease decay schedule, period and phase | The lease decay | unset, pass 10 | BLK-050 | Provisional default of period 32 and phase 0, declared as `LEASE_DECAY_PERIOD_DEFAULT` and `LEASE_DECAY_PHASE_DEFAULT`. A tile that nobody stands on therefore loses one count every 32 ticks, so a lease outlives a passing unit and not an abandonment. A caller sets it with `set_lease_rules`. Pass 10 measures it. |
| Lease bound, the count a lease never passes | The lease pass | unset, pass 10 | BLK-050 | Provisional default of 192, declared as `LEASE_BOUND_DEFAULT`, three times the claim threshold, so a long occupation buys a lease that survives about two thousand ticks of decay. A caller sets it with `set_lease_rules`. Pass 10 measures it. |
| Lease claim threshold, the count at or above which the lease holds the tile | The holding rewrite | unset, pass 10 | BLK-050 | Provisional default of 64, declared as `LEASE_CLAIM_THRESHOLD_DEFAULT`, which is 64 ticks of unopposed standing at the raise step. A caller sets it with `set_lease_rules`. Pass 10 measures it. |
| Closure pass count, the passes that run after the holder of every tile is decided | The holding rewrite | unset, pass 10 | BLK-050 | **No value, because no closure pass exists.** ADR-0153 D6 states the closure and nothing implements it. Item 0493 holds the work, and this row stays empty until that pass runs. |
| Closure neighbour threshold, the neighbours of one faction that give it an unheld tile | The holding rewrite | unset, pass 10 | BLK-050 | **No value, because no closure pass exists.** See the row above. |

## The seeding layer

| Value | Read by | Set | Blocker | Derivation |
|---|---|---|---|---|
| Founding group, the people each faction founds with | The seeding layer, when it founds the run | unset, pass 10 | — | The project owner set this to 2 on 5 September 2026. The earlier default of 64 came from pass 1. It was the constant the demonstration passed to the founding verb before the seeding moved into the engine. Pass 10 measures it. |
| Luxury deposits, the tiles the seeding layer places a luxury on | The seeding layer, when it places the luxuries | unset, pass 10 | — | Provisional default of 8 written by pass 1, two deposits for each faction of the demonstration world. Pass 10 measures it. |

## The population

| Value | Read by | Set | Blocker | Derivation |
|---|---|---|---|---|
| Housing, the quantity that stands at one site | The growth stage, when it counts the free places of a site | unset, pass 10 | BLK-050 | Named by the growth and queue design call of 5 September 2026, which makes housing the bound on the people a site holds at once. Item 0059 made it a stored column of the settlement arena, and the ground never sets it. It is a quantity of housing and not a count of people, so the housing per person row converts it. A founded site starts at the founding housing row. **The earlier reading of this row is stale: an upgrade raises it now.** Item 0498 added a lodging category, and a finished lodging level raises the housing of the site that reaches the tile. A caller sets it with `set_site_housing`. Pass 10 measures it. |
| Housing per person, the housing one person takes | The growth stage, when it counts the free places of a site | unset, pass 10 | BLK-050 | Named by the growth and queue design call of 5 September 2026. It is stated apart from the housing, so that the housing is a quantity of housing and not a count of people. Provisional default of 1 written by item 0059, the smallest whole quantity above zero, so the housing of a site reads directly as the people it holds until a reason to separate the two arrives. A value of zero leaves no site with a free place. A caller sets it with `set_housing_per_person`. Pass 10 measures it. |
| Food per birth, the store one birth costs | The growth stage, when it proposes a birth | unset, pass 10 | BLK-050 | Named by the growth and queue design call of 5 September 2026, which makes food and housing the two things a site needs to grow. Provisional default of one whole unit of every good written by item 0060: the smallest whole quantity above zero, so a birth is never free and the cost converts exactly in the fixed-point scale. One whole unit is sixteen ticks of the default ration of one person. A caller sets it with `set_food_per_birth`. Pass 10 measures it. |
| Birth rate, the chance that one proposal becomes a birth | The growth stage, when it proposes a birth | unset, pass 10 | BLK-050 | Named by the growth and queue design call of 5 September 2026. The store sets how many proposals a site makes, which is what it can pay for, and this row is the chance that one proposal takes. Provisional default of one half written by item 0060, so growth is a rate and not a step, and a site with food does not fill its housing on one application. A chance at or above one makes every proposal a birth. A caller sets it with `set_birth_chance`. Pass 10 measures it. |
| Growth schedule, period and phase | The growth stage, when it decides whether this tick acts | unset, pass 10 | BLK-007 | Named by the growth and queue design call of 5 September 2026. Provisional default of period 10 and phase 0 written by item 0060, the economy schedule default, so a site grows once for each application of the rates that fill the store it reads. A caller sets it with `set_growth_schedule`. Pass 10 measures it. |
| Founding housing, the housing a founded site starts with | The seeding layer, when it founds a site | unset, pass 10 | BLK-050 | Named by the growth and queue design call of 5 September 2026. A founded site must house the group that founds it, or a run starts crowded. Provisional default of 16 written by item 0060, which is four times the campaign cohort size, so a faction may raise a cohort and still hold people at home. No upgrade raises the housing yet, so this value is the ceiling on the people one site holds. **Every faction reaches that ceiling and stays at it.** Measured on 5 September 2026 by item 0481 over 32 seeds on one development machine (ty001-ubuntu, x86-64), extent 256, four factions, tick limit 20000, with `scripts/balance_sweep.py`. A development-machine run, and no evidence about the target platform. The population of every faction reads 16 from tick 500 to the end of the run, in 31 of the 32 seeds, and the lowest population any faction held at an end was 11. The world is not starving; it is full. A caller sets it with `set_founding_housing`. Pass 10 measures it. |

The six rows above are unset and every one of them is behind a blocker. Each
now holds a provisional default that item 0059 or item 0060 chose, and the
derivation column names which. The growth stage reads every one of them from
the world and holds no literal of its own. ADR-0157 cites this section and
states no figure of its own.[^8]

## The ground and what it gives

**This section was missing until item 0508 measured the ground.** Four values
decide whether a worked tile is a stock that empties once or a flow a faction
returns to. All four are declared in the core and no register held any of them,
so the table said nothing about the one part of the world that can renew
itself.[^17]

| Value | Read by | Set | Blocker | Derivation |
|---|---|---|---|---|
| Gather rate, the stock one unit takes from a tile in one tick | The gather resolve | unset, pass 10 | BLK-050 | Provisional default of 4. The core comment states the intent: the rate is high against the stock of a tile, so a full tile of gatherers always empties a deposit. Pass 10 measures it. |
| Wet gather bonus, the extra stock a unit takes on wet ground | The gather resolve | unset, pass 10 | BLK-130 | Provisional default of 2. Pass 10 measures it. |
| Recovery period, the ticks one unit of a kind takes to come back | The resource ageing | unset, pass 10 | BLK-050 | Provisional defaults of 240 ticks for food and 300 for wood, declared as `RECOVERY_TICKS_FOR_ONE_UNIT` in the resource module. Stone declares no period and never comes back. **The period is declared in ticks and not in units for each day**, because the rate is below one unit a day and a rate in units for each day cannot reach that without a period of zero. The earlier reading of this row, at 600 ticks for food and 2400 for wood, is stale: the owner reported that food came back too fast on unimproved ground, and item 0511 raised the unimproved period and gave a terrace a speedup instead. A caller sets it with `set_recovery_rules`. Pass 10 measures it. |
| Moisture band scale, the sixteenths a moisture band applies to a recovery period | The resource ageing | unset, pass 10 | BLK-130 | Provisional table of 7 bands, indexed by the water on the ground of the level 1 cell that covers the tile. Food peaks in a narrow middle band and suffers at both ends. Wood peaks higher, holds a plateau over the wet middle and collapses only in the parched band. Stone declares no period, so no entry of its row is ever read. **The band edges come from a measurement and not from a guess.** A probe walked 2000 ticks of a 192 by 192 world and sorted the ground plane every 50 ticks; of 1440 cell samples, most held under 32 drops and the tail reached 1395, so even steps would have put almost every cell in one band. Written by item 0511. Pass 10 measures it. |
| Simulated day, the ticks a recovery period counts in | The resource ageing | unset, pass 10 | BLK-007 | Provisional default of 600 ticks. Pass 10 measures it. |

## The effective rate

**A site derives its production and its upkeep from the world at each
application, and the stored rate is the base.** The four production terms and
the two upkeep terms below are shares of one. A record states how they
compose, and this register states what each one is worth.[^18]

| Value | Read by | Set | Blocker | Derivation |
|---|---|---|---|---|
| Holding share, the part of what a store holds that the store costs to keep each tick | The effective upkeep | unset, pass 10 | BLK-050 | Provisional default of one sixty-fourth, declared as `HOLDING_SHARE`. This is the term that turns a store from a thing that climbs into a thing that settles: a store settles at its net production divided by the share, which is 64 times the net production. The share is the reciprocal of a relaxation time of 64 ticks, and the rate pass applies every 10 ticks by default, so a store crosses most of the distance to its settling point in about six applications and a watcher sees it move. At one sixteenth it would settle inside two applications and read as a constant again. At one five hundred and twelfth it would still be climbing after a whole simulated day, which is the behaviour this replaced. Written by item 0508. Pass 10 measures it. |
| Resident share of the ration, the part of a resident's food that the site pays to house it | The effective upkeep | unset, pass 10 | BLK-050 | Provisional default of one quarter, declared as `RESIDENT_SHARE_OF_RATION`. It multiplies the ration of the need rule rather than restating it, so the food a person costs keeps one declaration site. Feeding and housing a person together take five quarters of the ration, so a site runs out of store a little before its people run out of food, and the store is the early warning. Written by item 0508. Pass 10 measures it. |
| Ground term, what the disc of a site still holds of food against what it held untouched | The effective production rate | unset, pass 10 | BLK-050 | Provisional weight written by item 0513, declared as `GROUND_WEIGHT`. It is the sink: gatherers draw the ground down and the recovery brings it back, so the term rises and falls over a period of a few hundred ticks. Pass 10 measures it. |
| Moisture term weight, what wet ground adds to a site's output | The effective production rate | unset, pass 10 | BLK-130 | Provisional weight written by item 0513, declared as `WET_WEIGHT`. The term takes the same wet mark that the gather resolve takes, so the project holds one moisture reader and not two, and it stays discontinuous for that reason. Pass 10 measures it. |
| Terrace term ceiling, the terraces of a disc that the term counts before it stops | The effective production rate | unset, pass 10 | BLK-050 | Provisional ceiling written by item 0513, declared as `TERRACE_CEILING` against a weight `TERRACE_WEIGHT`. A standing terrace is worked ground and worked ground yields more. Pass 10 measures it. |
| People term, what a site that has lost its residents works of its ground | The effective production rate | unset, pass 10 | BLK-050 | Provisional weight written by item 0513, declared as `PEOPLE_WEIGHT`. The term takes away and never adds. Pass 10 measures it. |

**No temperature term exists.** The production pipeline composes four terms and
the project owner asked for a fifth. A separate item holds it, and it is held by
a measurement rather than by a choice: a weight written against a range nobody
has taken would be an invented value.[^19]

## The production queue

| Value | Read by | Set | Blocker | Derivation |
|---|---|---|---|---|
| Queue bound, the entries one site holds | The queue verb, when it refuses an entry | unset, pass 10 | BLK-050 | Named by the growth and queue design call of 5 September 2026. It bounds the cost of the advance stage together with the settlement count. Provisional default of 4 written by item 0497, the filled rows of the default unit type table, so a caller may queue every type the world defines at once. The bound is a parameter of the world, and a caller sets it with `set_queue_bound`. A bound of zero turns the queue off. The width of the stored block is `QUEUE_BOUND` and it caps the bound. Pass 10 measures it. |
| Work to finish a unit, by type | The queue advance, when it finishes an entry | unset, pass 10 | BLK-050 | Named by the growth and queue design call of 5 September 2026. One value for each row of the unit type table, so a settler and a soldier may take different work. Provisional default of 8 for every row written by item 0497, which is eight advances of the default schedule, or eighty ticks against the tick limit row, so a build is a choice and not a formality. One advance adds `WORK_PER_ADVANCE`, which is one, so the value counts the advances a build takes. A caller sets the row with `define_build_cost`. Pass 10 measures it. |
| Goods to finish a unit, by type and by good | The queue advance, when it charges a finished entry | unset, pass 10 | BLK-050 | Named by the growth and queue design call of 5 September 2026. A settler costs food and a soldier costs the goods that arm it. Provisional default of 8 whole units of every good for every row, written by item 0497 as the charge row multiplied by the work row, so the completion costs as much again as the advances did. A caller sets the row with `define_build_cost`. Pass 10 measures it. |
| People to finish a unit, by type | The queue advance, when it takes a resident | unset, pass 10 | BLK-050 | Named by the growth and queue design call of 5 September 2026. The call fixes the value at one person for every type, and the row exists so that a later type may cost more without a change to the record. Provisional default of 1 for every row, written by item 0497. The advance reads this row and holds no literal, so a row above one takes that many residents. A caller sets the row with `define_build_cost`. Pass 10 measures it. |
| Queue charge, the store one advance costs | The queue advance, when it advances the front entry | unset, pass 10 | BLK-050 | Named by the growth and queue design call of 5 September 2026, which requires that a queue is never free. Provisional default of one whole unit of every good, written by item 0497: the smallest whole quantity above zero, so a queue is never free and the charge converts exactly in the fixed-point scale. A caller sets it with `set_queue_charge`. Pass 10 measures it. |
| Queue schedule, period and phase | The queue advance, when it decides whether this tick acts | unset, pass 10 | BLK-007 | Named by the growth and queue design call of 5 September 2026. Provisional default of period 10 and phase 0 written by item 0497, the economy schedule default, so a queue advances once for each application of the rates that fill the store it reads. A caller sets it with `set_queue_schedule`. Pass 10 measures it. |

The six rows above are unset and every one of them is behind a blocker. Each
now holds a provisional default that item 0497 chose, and the derivation column
gives the reasoning for each one. ADR-0158 cites this section and states no
figure of its own.[^9]

## Unit types

| Value | Read by | Set | Blocker | Derivation |
|---|---|---|---|---|
| Default table, seven rows by ten columns | The seeding layer | unset, pass 10 | — | |

The ten columns are attack, armour, gather rate, build rate, carry capacity,
move cost scale, command reach, weather reach, water crossing and settle group.
The seven rows are worker, soldier, merchant, leader, one open row, mariner and
settler. Each cell is one value, and this register holds them as one row until a
pass writes them, because seventy empty rows say nothing that one does not.

| Value | Read by | Set | Blocker | Derivation |
|---|---|---|---|---|
| Water crossing of the mariner row and the settler row | The terrain capacity table | unset, pass 10 | BLK-050 | Provisional placeholder written by the water crossing work. The column states the room a water tile holds for a unit of the type, and the capacity table takes it as a parameter, so the engine holds one statement of passability and the column sets it. Pass 10 measures it. |
| Settle group, the people a founding by a settler seats at the new city | The settle verb | unset, pass 10 | BLK-050 | Provisional placeholder of one. It is the smallest group the founding path admits, because a survey refuses a group of nobody, and it keeps the unit count of a faction level across a founding, since the settler is spent. Pass 10 measures it. |

## Upgrades

| Value | Read by | Set | Blocker | Derivation |
|---|---|---|---|---|
| Full condition | The build pass on completion, and the repair clamp | unset, pass 10 | — | Provisional default of 1,000,000, declared as `CONDITION_FULL` in the upgrade module. One scale serves every category, and it is not per kind. The scale is large because a repair buys `CONDITION_FULL * work / level_work` a tick, and a smaller scale would round the repair of a costly category to nothing a tick, so that category could never be mended. Written by item 0475. Pass 10 measures it. |
| Wear step from a hostile unit | The wear pass | unset, pass 10 | — | Provisional default of 2000, declared as `ARMY_WEAR_FOR_EACH_UNIT` in the world module, which is 500 ticks of one hostile unit from full condition to collapse. Neither wear rate reads the category, because the upgrade row holds no column that resists wear, so a wall and a road wear alike. Written by item 0475. Pass 10 measures it. |
| Wear step from wet ground | The wear pass | unset, pass 10 | BLK-130 | Provisional default of 500, declared as `WEATHER_WEAR_FOR_EACH_TICK` in the world module, which is 2000 ticks of unbroken storm from full condition to collapse. Written by item 0475. Pass 10 measures it. |
| Recovery change, the divisor a standing improvement applies to a recovery period | The resource ageing | unset, pass 10 | BLK-050 | Provisional defaults of 8 at terrace level 1 and 16 at level 2, held in the `recovery_change` column of the upgrade row. The rule scales the speedup by the condition of the site, so a neglected terrace falls back toward the unimproved rate and reaches it exactly at no condition. Written by item 0511. Pass 10 measures it. |
| Wall work | The build pass | unset, pass 10 | BLK-007 | Provisional default of 16 written by item 0486, twice the first level of a road, so a defence costs more than a way and less than worked ground. **The earlier reading of this row is stale: item 0475 has landed.** The wall row still holds no effect column, and it wears at the same rate as every other category, because no column of the upgrade row resists wear. Pass 10 measures it. |
| Wall harm absorption | The contest pass | unset, pass 10 | — | |
| Wall move cost raise for a unit whose faction does not hold the tile | The movement pass | unset, pass 10 | — | |
| Road ground fit, the ground kinds a road fits | The build resolve | unset, pass 10 | BLK-050 | Provisional fit of plain, forest and hill written by item 0486. A road is a made way over ground a unit walks, and high ground is not it. Water holds nobody, so no row fits it. Pass 10 measures it. |
| Terrace ground fit, the ground kinds a terrace fits | The build resolve | unset, pass 10 | BLK-050 | Provisional fit of plain, forest and hill written by item 0486, the road fit again, because the ground a unit works is the ground a unit walks. Pass 10 measures it. |
| Wonder, store and wall ground fit | The build resolve | unset, pass 10 | BLK-050 | Provisional fit of every ground a unit stands on, written by item 0486, so that a great work, a storehouse and a defence stand wherever a faction holds ground. Pass 10 measures it. |
| Road work, by level | The build pass | unset, pass 10 | BLK-007 | Provisional defaults of 8 and 24 written by item 0486. The first level is the value the catalogue held before the table. The second is three times the first, so a raise costs more than the build that reached it. Pass 10 measures it. |
| Road capacity, by level | The capacity composition | unset, pass 10 | BLK-050 | The first level is the crossing capacity of the scale constants table, and the module reads it from there rather than restating it. The second level is provisionally twice the first, written by item 0486, so a watcher reads the level from what the tile holds. Pass 10 measures it. |
| Terrace work, by level | The build pass | unset, pass 10 | BLK-007 | Provisional defaults of 24 and 72 written by item 0486. The first level is the value the catalogue held before the table. The second is three times the first, as the road is. Pass 10 measures it. |
| Terrace yield, by level | The gather resolve | unset, pass 10 | BLK-050 | Provisional defaults of 2 and 4 written by item 0486. The first level is the value the catalogue held before the table, and the second is twice it. Pass 10 measures it. |
| Wonder victory claim | The wealth-or-wonder reader | unset, pass 10 | BLK-050 | Provisional default of 1 written by item 0486, the smallest value above zero, because the reader asks whether a claim stands and not how large it is. Pass 10 measures it. |
| Wonder work | The build pass | unset, pass 10 | BLK-007 | Provisional default of 2400, a hundred times the first level of a terrace. The project owner asked for a much higher bar on 5 September 2026. The earlier default of 240 came from pass 8. The multiple is ten, one order, because no measurement supports a finer choice. **No measurement supports the raise, and none refutes it.** Measured on 5 September 2026 over the 8 default seeds on one development machine (ty001-ubuntu, x86-64), extent 256, four factions: no wonder finished in any seed, and the wonder work done stalled between 18 and 61 units before tick 500 and did not move again by tick 20000. **That reading is stale.** It was taken before the repair that closed the road chain. Measured on 5 September 2026 by item 0481 over 32 seeds on one development machine (ty001-ubuntu, x86-64), extent 256, four factions, tick limit 20000, with `scripts/balance_sweep.py`. A development-machine run, and no evidence about the target platform. At 2400 the wonder work reaches the bar in 3 of the 32 seeds and ends those three games. The work done at the end of a run runs from 24 to 2400, with a median of 192. The findings register holds the reading.[^16] Pass 10 measures it. |
| Store work | The build pass | unset, pass 10 | BLK-007 | Provisional default of 48, twice the terrace, written by pass 8. Pass 10 measures it. |
| Lodging work, by level | The build pass | unset, pass 10 | BLK-007 | Provisional defaults of 144 and 432, declared as `LODGING_LEVEL_1_WORK` and `LODGING_LEVEL_2_WORK`. The second is three times the first, as the road and the terrace are. Written by item 0498. Pass 10 measures it. |
| Lodging housing, the housing one finished level gives | The build pass on completion | unset, pass 10 | BLK-050 | Provisional default of half the founding housing, declared as `LODGING_LEVEL_HOUSING` against `FOUNDING_HOUSING_DEFAULT`, so two levels double what a site was founded with. **It reads the founding housing rather than restating it**, so the project holds one declaration of what a site starts with. Written by item 0498. Pass 10 measures it. |
| Lodging ground fit, the ground kinds a lodging fits | The build resolve | unset, pass 10 | BLK-050 | Provisional fit declared as `LODGING_FIT`. Written by item 0498. Pass 10 measures it. |
| Store capacity raise | The site store | unset, pass 10 | — | Provisional default of 64 whole units, as a raw Q16.16 quantity, written by pass 8. The engine holds no store capacity, so nothing reads the raise yet and the boundary exposes the sum for a site. Pass 10 measures it. |

## Weather

| Value | Read by | Set | Blocker | Derivation |
|---|---|---|---|---|
| Flood mark, the ground water above which a cell is flooded | Every weather harm | unset, pass 10 | BLK-130 | |
| Spoil share, the integer share of a store lost per tick on a flooded cell | The spoilage | unset, pass 10 | BLK-130 | |
| Unit-loss draw bound, the most units one draw names per flooded cell | The unit loss draw | unset, pass 10 | BLK-130 | |
| Move cost step on wet ground | The movement pass | unset, pass 10 | BLK-130 | |
| Wet mark, the ground water at which a cell counts as wet | The gather resolve, the production moisture term, and the upgrade wear pass | unset, pass 10 | BLK-130 | Provisional default of 64 drops, declared as `WET_MARK` in the weather module. It is a content constant that no measurement chose. Item 0501 set it against a measured ground plane, so that a cell can be wet while its neighbour is dry rather than every cell reading wet. Pass 10 measures it. |
| Lift quantity, the water one lift raises into the air of a cell | The weather solve | unset, pass 10 | BLK-130 | |
| Lift period, how rarely a cell of open water lifts | The weather solve | unset, pass 10 | BLK-130 | |
| Heat weight of the mean height of a cell | The heat of a cell | unset, pass 10 | BLK-130 | |
| Heat weight of the water share of a cell | The heat of a cell | unset, pass 10 | BLK-130 | |
| Pressure scale, the pressure one step of heat gives | The wind pass | unset, pass 10 | BLK-130 | Provisional default declared as `PRESSURE_DIVISOR` in the weather module. The value is the same at every lattice pitch, because the temperature holds the terrain of the cell and a coast therefore holds one step between two neighbours whatever the pitch. Written by item 0499. Pass 10 measures it. |
| Wind acceleration step, the most the wind of a cell changes in one pass | The wind pass | unset, pass 10 | BLK-130 | Provisional default of two lattice steps, declared as `WIND_STEP` in the weather module. The step is what makes the wind lag the pressure, and the lag is why the field carries a front at all. Written by item 0499. Pass 10 measures it. |
| Wind drag share, the share of the speed a cell loses in one pass | The wind pass | unset, pass 10 | BLK-130 | Provisional default of one quarter, declared as `DRAG_NUMERATOR` over `DRAG_DENOMINATOR` in the weather module. The drag settles the wind where the step and the share balance, and under the step above the field settles below the speed ceiling. Written by item 0499. Pass 10 measures it. |
| Wind speed ceiling | The wind pass, and the air transport | unset, pass 10 | BLK-130 | Provisional default of six lattice steps, declared as `SPEED_CEILING` in the weather module against the wind fineness `WIND_FINE`. A settled field never reaches it, so it is a guarantee and not a working part. Written by item 0499. Pass 10 measures it. |
| Wind pass count | The weather solve | unset, pass 10 | BLK-007 | Provisional default of 2, declared as `WIND_PASSES_FOR_EACH_SOLVE` in the weather module. The count is fixed and no pass tests whether the field settled. Written by item 0499. Pass 10 measures it. |
| Transport share, the share of the air a cell sends for each step of wind speed | The air transport | unset, pass 10 | BLK-130 | Provisional default declared as `SEND_FOR_EACH_WIND_STEP` over `SEND_DENOMINATOR` in the weather module, with a base share `SEND_BASE_NUMERATOR` that a still cell sends. A compile-time assertion holds the total send below one, so no cell sends more air than it holds. Written by item 0500. Pass 10 measures it. |
| Evaporation share by heat, the share of the water of a cell that its heat lifts | The weather solve | unset, pass 10 | BLK-130 | |
| Rain share by cooling, the share of the air that falls for each step of cooling | The weather solve | unset, pass 10 | BLK-130 | |
| Lattice pitch, the tiles a weather cell covers along one side | The weather solve, and every weather reader | unset, pass 10 | BLK-130 | Provisional default of the level 1 pitch, declared as `WeatherScale::DEFAULT` in the weather module. The pitch is a parameter of the world and runs from one tile a cell to the ceiling `SCALE_BITS_CEILING`, which is 256 tiles a side. `REFERENCE_BITS` names the pitch the tuning was chosen at, and the four derived quantities return their stated values there exactly, so a world at the default behaves as it did before the pitch became a parameter. Written by item 0499. **The cost of each pitch was measured, and it is the reason the default is the level 1 pitch rather than the tile pitch.** Measured on 6 September 2026 on one development machine, x86-64, over a 256 by 256 world, 200 ticks, one thread, as the nanoseconds the weather stage took for each tick: 29,022 at the level 1 pitch, 193,596 at half of it, 1,116,622 at a quarter of it, and 120,809,461 at one cell for each tile. A development-machine run, and no evidence about the target platform. The demonstration picture runs in 2.8 seconds at the default and 19.1 seconds at the tile pitch. Pass 10 measures it. |
| Season swing, the degrees the sun adds at the warm centre of its band | The temperature pass | unset, pass 10 | BLK-130 | Provisional default of 80, declared as `SEASON_SWING` in the weather module. The season takes the same count away at the cold edge, so the term runs from this above zero to this below it. Written by item 0499. Pass 10 measures it. |
| Season period, the ticks the sun takes for one whole swing | The temperature pass | unset, pass 10 | BLK-007 | Provisional default of 2048, declared as `SEASON_PERIOD_TICKS` in the weather module. The period is a time and not a distance, so it follows neither the extent of the world nor the pitch of the lattice. Half of it lasts about a thousand ticks, which is long enough for a place to hold a wet part of the year and a dry part. Pass 10 measures it. |
| Tilt, the part of the way to a pole the sun reaches at the top of its swing | The temperature pass | unset, pass 10 | BLK-130 | Provisional default of one half, declared as `TILT_NUMERATOR` over `TILT_DENOMINATOR` in the weather module. The Earth reaches about a quarter. This world reaches half, because a larger tilt gives a larger seasonal difference at every latitude and the swing must be readable in a picture. Pass 10 measures it. |
| Cloud swing, the degrees a saturated sky takes away from a cell | The temperature pass | unset, pass 10 | BLK-130 | Provisional default of 32, declared as `CLOUD_SWING` in the weather module. The term rises with the water in the air over the cell and stops at the saturation mark, so it is bounded whatever a god puts into the sky. Pass 10 measures it. |
| Warmth share, the part of the way to the asked temperature one pass moves | The temperature pass | unset, pass 10 | BLK-130 | Provisional default of one eighth, declared as `WARMTH_NUMERATOR` over `WARMTH_DENOMINATOR` in the weather module. The share is what makes the temperature lag its driver, and the lag is why the field carries the temperature rather than deriving it. Pass 10 measures it. |
| Temperature pass count | The weather solve | unset, pass 10 | BLK-007 | Provisional default of 1, declared as `WARMTH_PASSES_FOR_EACH_SOLVE` in the weather module. The count is fixed and no pass tests whether the field settled. Pass 10 measures it. |
| Deflection share, the part of a sixth of a turn the wind gains each pass | The wind pass | unset, pass 10 | BLK-130 | Provisional default of one third, declared as `DEFLECT_NUMERATOR` over `DEFLECT_DENOMINATOR` in the weather module. It is the one term that turns a flow, so a circulation forms without anyone naming one. A large share would spin every cell whatever the flow, which is a stirred field and not a weather field. Pass 10 measures it. |
| Temperature carry share, the part of a difference one step of wind carries | The temperature pass | unset, pass 10 | BLK-130 | Provisional default declared as `CARRY_FOR_EACH_WIND_STEP` over `CARRY_DENOMINATOR` in the weather module. A compile-time assertion holds the total carry below one, so no cell hands away more than the difference it holds. Pass 10 measures it. |

## Balance shares

| Value | Read by | Set | Blocker | Derivation |
|---|---|---|---|---|
| Win-path share, the most seeds one path may win | The balance harness, statement 1 | unset, pass 10 | — | Observed on 2026-09-05 by item 0481 over 8 seeds on one development machine (ty001-ubuntu, x86-64), extent 256, four factions, tick limit 2000: territory won 8 of 8, and it was the only path that existed. Observed again on 2026-09-05 on the same machine and the same seed set, at the stock target, wonder work and tick limit the project owner asked for: territory won 5 of 8 and wealth or wonder won 3 of 8. Domination and renown won none. **Both readings above are stale, and the second is wrong about territory.** Measured on 5 September 2026 by item 0481 over 32 seeds on one development machine (ty001-ubuntu, x86-64), extent 256, four factions, tick limit 20000, with `scripts/balance_sweep.py`. A development-machine run, and no evidence about the target platform. Wealth or wonder won 32 of 32. Territory, domination and renown won none. Value unset under BLK-050. |
| Seat share, the most seeds one seat may win | The balance harness, statement 2 | unset, pass 10 | — | Observed on 2026-09-05 by item 0481 over 8 seeds on one development machine (ty001-ubuntu, x86-64), extent 256, four factions, tick limit 2000: seat 0 won 2 of 8, seat 1 won 3 of 8, seat 2 won 2 of 8, seat 3 won 1 of 8. Observed again on 2026-09-05 on the same machine and the same seed set, at tick limit 5000: seat 0 won 4 of 8, seat 1 won 2 of 8, seat 2 won 2 of 8, and seat 3 won none. Measured on 5 September 2026 by item 0481 over 32 seeds on one development machine (ty001-ubuntu, x86-64), extent 256, four factions, tick limit 20000, with `scripts/balance_sweep.py`. A development-machine run, and no evidence about the target platform. Over 32 seeds at tick limit 20000: seat 0 won 8, seat 1 won 10, seat 2 won 9 and seat 3 won 5. A development-machine run, and no evidence about the target platform. Value unset under BLK-050. |
| End share, the fewest seeds that must end before the tick limit | The balance harness, statement 3 | unset, pass 10 | BLK-007 | Observed on 2026-09-05 by item 0481 over 8 seeds on one development machine (ty001-ubuntu, x86-64), extent 256, four factions, tick limit 2000: 0 of 8 ended before the tick limit, because the territory path fires at the limit. Observed again on 2026-09-05 on the same machine and the same seed set, at tick limit 5000: 3 of 8 ended before the limit, and each of those three ended on wealth. Measured on 5 September 2026 by item 0481 over 32 seeds on one development machine (ty001-ubuntu, x86-64), extent 256, four factions, tick limit 20000, with `scripts/balance_sweep.py`. A development-machine run, and no evidence about the target platform. Over 32 seeds at tick limit 20000: 32 of 32 ended before the limit. A development-machine run, and no evidence about the target platform. Value unset under BLK-050. |
| Seed set | The balance harness | unset, pass 10 | — | The harness default is 8 seeds derived from the demonstration seed by one stated rule in the code, and `--seeds` overrides it. The default is a fixture and not a value. Observed on 2026-09-05 by item 0481 over those 8 seeds on one development machine (ty001-ubuntu, x86-64), extent 256, four factions, tick limit 2000: contracts, controller_commands and controller_refused were zero in every game at its end. Observed again on 2026-09-05 on the same machine and the same seed set, at tick limit 5000: thirteen rows were zero in every game, and they include campaigns_raised, wars_declared, relation_moves, seats_filled and wonders_complete. **Both readings above are stale for every row that counts an act.** Those rows read the last tick and not the run, and the controller emits nothing after a game end, so each such zero is an artefact of the reader. The finding holds the detail.[^13] A development-machine run, and no evidence about the target platform. Value unset under BLK-050. |

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
[^8]: ADR-0157, a site's free places are its built housing less the residents the engine already counts. `docs/adrs/accepted/adr-0157-a-sites-free-places-are-its-built-housing-less-the-residents-the-engine-counts.md`
[^9]: ADR-0158, a site builds a typed unit from a bounded queue its store pays for. `docs/adrs/accepted/adr-0158-a-site-builds-a-typed-unit-from-a-bounded-queue-its-store-pays-for.md`
[^13]: Findings register, FND-498. `docs/FINDINGS.md`
[^14]: Findings register, FND-542. `docs/FINDINGS.md`
[^15]: Findings register, FND-543. `docs/FINDINGS.md`
[^16]: Findings register, FND-547. `docs/FINDINGS.md`
[^17]: Findings register, FND-549. `docs/FINDINGS.md`
[^18]: ADR-0055, a site derives its effective rate from the world at each application. `docs/adrs/draft/adr-0055-a-site-derives-its-effective-rate-from-the-world.md`
[^19]: Backlog item 0512, add the temperature term to the production pipeline. `docs/backlog/proposed/0512-add-the-temperature-term-to-the-production-pipeline.md`
