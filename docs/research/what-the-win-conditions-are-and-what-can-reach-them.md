# What the win conditions are, and what can reach them

This report states what the engine does today. It designs nothing and it
changes nothing. It reads the code and reports what the code says.

The engine names four win paths: domination, territory, wonder and renown.[^1]
This report answers six questions for each one. Which function decides it
fired. When that function runs. What quantity it compares, against what
threshold, declared where. What a player must do to reach it. What the
published progress signal measures. Whether the path can fire at all.

The report also states what happens at the tick limit, when a game ends with
no record, whether a policy can read its own distance to each threshold, and
where a document and the code disagree.

Every figure of behaviour in this report comes from reading the code. This
report took no measurement. Where it repeats a measured figure, it names the
source and marks it as somebody else's reading.

## 1. The reader table and when it runs

One function runs every reader.[^2] It is a private method of the world, and
the controller stage calls it once each tick.[^3] The frame step increments
the tick counter at the top of the step, so the step numbered N runs the
readers at tick N.[^4]

The function returns at once in two cases. It returns when a balance flag
turns the readers off.[^5] It returns when the end record is already set. The
record is written once and nothing rewrites it.

The function then runs four readers in a fixed order.

| Order | Path | Reader |
|-------|------|--------|
| 1 | Domination | `domination_winner` |
| 2 | Territory | `territory_winner` |
| 3 | Wonder | `wonder_winner` |
| 4 | Renown | `renown_winner` |

The first reader that names a faction writes the record and the function
returns. The order is fixed in the source and it is not a balance value.

Every reader walks the contenders, which are the factions that have not left
the game.[^2] A faction leaves the game when it holds no site and no
unit.[^6] A faction that has left wins nothing.

Each reader visits the contenders in ascending faction order and stops at the
first that fires. A tie therefore resolves to the lowest faction number.

**Three of the four readers can fire on any tick. One cannot.** The territory
reader returns nothing while the tick is below the tick limit. Section 3
states this exactly.

## 2. Domination

### The reader

`domination_winner` decides it.[^2]

### When it runs

Every tick, first of the four.

### The exact condition

The reader holds two clauses. A faction wins when either holds.

**The seat clause.** A seat is the tile of the first founding of a faction,
and the controller keeps one seat for each faction. The clause holds when the
candidate holds the seat tile of every faction that is still in the game, its
own seat included, and when at least one of those seats belongs to a rival.
The reader reads the holder of each seat tile. It walks no unit and no tile.

**The unit clause.** The clause holds when every rival has no live unit and
the candidate has one. The reader reads a live count that the soldier arena
keeps as a running total.

The reader returns nothing in a world of fewer than two factions.

**Neither clause has a threshold.** There is no number to declare and no
balance register row for this path. The condition is structural and it is
written into the reader.

### What a player must do

A player reaches the seat clause by taking the seat tile of every live rival.
Held ground is the ground within reach of a city of the faction.[^7] A player
therefore takes a rival seat by taking the city that holds it, or by founding
a city near enough to reach it. The campaign verb raises a war band against an
objective the engine resolves, and a war band takes a city.[^8] [^9] The
queue verb builds the soldiers the campaign spends. The settle verb founds the
city that holds ground.

A player reaches the unit clause by killing every unit of every rival, or by
waiting while the rivals lose their units to something else. The same campaign
and queue chain serves it.

Both clauses are reachable through verbs a policy holds.

### The published progress signal

`domination_progress` carries it, with `domination_leader` beside it.[^10]

The signal is the seats of live factions the faction holds, over the number of
live seated factions. The share is bounded to the closed interval from zero to
one.[^11]

**The signal reaches one exactly when the seat clause fires.** The numerator
and the denominator are computed from the same seat rule the reader reads. A
faction that holds every live seat reads one.

**The signal has a floor while the faction holds its own seat.** A faction
that holds only its own seat reads one over the live seat count. In a world of
three factions that is 0.333. The floor falls to zero if the faction loses its
own seat, and the floor rises as rivals leave the game, because the
denominator counts only live factions.

**The signal does not publish the unit clause.** The observation states the
reason: a share of that clause would state the unit count of a rival the
faction has never seen, and the layout publishes only the units the faction
has observed.[^10] [^12] A game can therefore end on domination while this
signal sits at its floor.

**A policy can see itself approach the seat clause.** It cannot see itself
approach the unit clause through this signal. The unit order statistics carry
a weaker reading of the same quantity, and those statistics are fogged.

### Reachability

The path fires. It needs two or more factions. The seat clause needs the
candidate to hold at least one rival seat.

## 3. Territory

### The reader

`territory_winner` decides it.[^2] It gathers the held tile count of each
contender and hands the pairs to a free function that picks the highest.[^1]

### When it runs

Every tick, second of the four. **It returns nothing while the tick is below
the tick limit.** The comparison is `self.tick.0 < self.controller.tick_limit()`
and the reader returns `None` when it is true.[^2]

**At or above the tick limit it fires whenever any contender exists.** The
free function returns the first pair with the highest count, and it returns
nothing only for an empty iterator.[^1] The iterator is empty only when every
faction has left the game.

### The exact condition

The quantity is the held tile count, which the holding keeps as a running
total. The comparison is between the factions, not against a number.

**This path holds no threshold of its own.** Its only parameter is the tick
limit, which decides when the reader starts to answer. The engine declares the
tick limit as `TICK_LIMIT_DEFAULT`, whose value is 5000.[^1] The balance
register holds the row and marks it unset under a blocker.[^13] The training
world sets 6000 through the environment configuration.[^14]

A tie resolves to the lowest faction number, because the free function keeps
the current leader when a later count is equal.[^1]

### What a player must do

A player holds ground by founding cities, because held ground is the ground
within reach of a city its faction owns.[^7] The settle verb founds a city
from a settler, and the queue verb builds a settler.[^8] [^9] A player also
takes ground by taking a rival city, through the campaign verb.

A player must then survive to the tick limit with more tiles than any live
rival. That is the whole condition.

**A player that does nothing can win this path.** The reader ranks the
factions and it asks for no act. The starting city of a faction holds ground
on its own.

### The published progress signal

`ground_progress` carries it, with `ground_leader` beside it.[^10]

**The signal measures held tiles over the passable tiles of the whole
world.**[^10] That denominator is not the requirement of the reader. The reader
compares the faction against the other factions, and it holds no denominator
at all.

**The signal does not reach one at the win.** A faction wins this path with
any share that beats its rivals. Held ground is bounded by the reach of the
cities of the faction, so the share stays well below one.[^7] Another agent
measured a highest reading of 0.45 over 3072 episodes, and a median of 0.02.

**A policy cannot read its distance to this threshold, because there is no
threshold.** The nearest reading a policy has is `ground_gap`, which is the
signed relation between the faction and its strongest rival, and
`ground_rank`, which is the share of rivals the faction leads.[^10] Those two
do carry the comparison the reader makes. `ground_progress` does not.

The product record already names this defect as a need, without naming the
path: one reading measures a distance against a requirement the win does not
use.[^12]

### Reachability

The path fires. It fires only at or above the tick limit, and only when at
least one faction is still in the game.

**The earlier claim that this path is unreachable by construction is
refuted.** A larger sample reported by another agent ends about 5 percent of
3072 episodes on territory. The reader is reachable, and the code gives no
reason it would not be. A game reaches it by surviving to the tick limit with
no other reader having fired.

## 4. Wonder

### The reader

`wonder_winner` decides it.[^2]

### When it runs

Every tick, third of the four.

### The exact condition

A wonder is an upgrade row that carries a victory claim above zero. The reader
walks the sparse upgrade map, reads the victory claim of the row that stands
at each improved tile, and attributes the claim to the faction that holds the
tile. It names the first contender whose largest standing claim is above
zero.[^2]

A claim on ground nobody holds counts for nobody.

Two values govern the path, and the upgrade table declares both.[^15]

| Value | Constant | Value today | Register row |
|-------|----------|-------------|--------------|
| The work that finishes a wonder | `WONDER_WORK` | 14400 | Wonder work |
| The claim the wonder row carries | `WONDER_VICTORY_CLAIM` | 1 | Wonder victory claim |

The balance register holds both rows and marks them unset under a
blocker.[^13] A caller sets either at run time through the world.[^2] The
record that governs them calls the work a rate and the claim a
threshold.[^16]

The wonder row fits every land tile and it asks for the builder's own
ground.[^15]

### What a player must do

A player builds a wonder with the build verb, naming the wonder category.[^8]
The verb is legal when any unit of the faction passes the build refusal for
that category.[^9] The verb orders the whole unit set of the faction to build
it.

A builder adds work to the tile it stands on, and to no other tile.[^17] One
unit of a type whose build rate is full adds one work each tick.[^15] [^18] The
worker row carries a full build rate. The soldier row carries zero, so a
soldier adds nothing.[^18]

A wonder therefore costs 14400 unit-ticks of worker work on one tile. Ten
workers standing on one tile finish it in 1440 ticks.

The player must also still hold the tile when the reader runs. The wonder row
asks for own ground, and the reader attributes the claim to the current
holder.

### The published progress signal

`wonder_track_progress` carries it, with `wonder_track_leader` beside it.[^10]

The signal is the work toward a wonder over the work the wonder row asks
for.[^10] The numerator is the largest work figure over the tiles of the
faction. A tile whose standing row already carries a claim reports the work of
that row, which equals the requirement.[^2]

**The signal reaches one exactly at the win.** The numerator equals the
denominator on the tick the wonder finishes.

**A policy can see itself approach this threshold.** The signal rises with
each tick of work, so it is dense and it is monotone while nobody destroys the
site.

The numerator is the maximum over tiles and not a sum. A faction that spreads
its builders over several tiles reads the best of them, not the total.

### Reachability

The path fires. Another agent measured it ending 0.62 of 3072 episodes.

## 5. Renown

### The reader

`renown_winner` decides it.[^2]

### When it runs

Every tick, fourth and last of the four.

### The exact condition

The reader takes the highest renown among the live characters of each faction,
and names the first contender whose value is at or above the renown
target.[^2] The renown column holds raw Q16.16 values.

The engine declares the target as `RENOWN_TARGET`, whose value is 50 whole
units.[^5] The balance register holds the row and marks it unset under
BLK-150.[^13] [^19] A caller sets it at run time. The record that governs it
calls it a threshold.[^16]

### What a player must do

**The engine holds one source of renown.**[^20] The contest of each frame
states, for each pair of factions, how many units one felled of the other. The
killer earns one share for each unit it felled. The share is a balance value,
declared as `RENOWN_PER_FELL`, whose value is one quarter of a whole
unit.[^21] [^5]

**The renown goes to one character, not to the faction.** The champion of a
faction is its live character with the highest renown, and a tie goes to the
lowest identity.[^20] A faction with no live character earns nothing.

A target of 50 against a share of one quarter asks for 200 enemy units to fall
to the faction while one champion lives.

The chain a player must run is therefore: build soldiers with the queue verb,
raise campaigns with the campaign verb, and kill 200 enemy units without
losing the champion that carries the renown.

**No verb creates a character.** The engine promotes a character from a
soldier on its own schedule, under a promotion budget.[^20] A player cannot
promote, cannot choose the champion, and cannot protect it. The player's only
lever on this path is the number of enemy units its faction fells.

### The published progress signal

`renown_progress` carries it, with `renown_leader` beside it.[^10]

The signal is the best renown of the faction over the renown target the
balance holds.[^10]

**The signal reaches one exactly at the win.** The reader and the signal
compare the same numerator against the same denominator.

**A policy can see itself approach this threshold.** The signal rises by one
two-hundredth of its range for each unit the faction fells, so it is dense.

The signal falls if the champion leaves the arena. The reader takes the
highest renown among the live characters, so a lesser character then carries
the reading.

### Reachability

The path fires. Another agent measured it ending 0.24 of 3072 episodes, with a
highest reading of 1.00.

The balance register records an earlier reading that the path was out of reach
by a factor near 110 under the built-in controller, because a faction of
workers fells nothing.[^13] A learned policy that fights reaches it.

## 6. What happens at the tick limit

Nothing in the step treats the tick limit as special. The step runs as it runs
on any other tick, and the controller stage calls the reader function.[^3]
[^4]

At the tick limit the four readers run in the same fixed order.

1. `domination_winner` runs first. If a faction holds every live seat, or if
   every rival is annihilated, the game ends on domination at the tick limit.
2. `territory_winner` runs second. It now answers, because the tick is no
   longer below the limit. It names the contender with the most held tiles.
   A tie goes to the lowest faction number.
3. `wonder_winner` and `renown_winner` run third and fourth. **At the tick
   limit they are unreachable in practice.** Territory returns a faction
   whenever any contender exists, and the two later readers use the same
   contender list. Territory therefore pre-empts them at that tick. The only
   case where territory returns nothing is the case where they also return
   nothing.

A game that reaches the tick limit therefore ends on domination or on
territory, and it never ends on wonder or renown at that tick.

## 7. When a game holds no end record

The learner names an episode with no end record `unfinished`.[^22] The
measured share of that name is 0.00, while territory is 0.05. The two agree,
and the code says why.

**A game that reaches the tick limit is always decided.** Territory resolves
it. So the tick limit does not produce an unfinished game.

Three cases produce a game with no end record.

1. **A caller turned the readers off.** The balance flag stops the reader
   function before it runs anything.[^5] [^2] The world then runs to the tick
   limit and records nothing.
2. **Every faction has left the game.** The contender list is then empty.
   Domination finds nobody, territory returns nothing from an empty iterator,
   and wonder and renown find nobody. No reader can ever fire again, because a
   faction that has left never returns.[^6]
3. **The learner stops the episode before the world reaches its tick limit.**
   The environment truncates when the decision count reaches the
   horizon.[^23] The training world sets the horizon to the tick limit divided
   by the decision interval, so the world does reach its limit there.[^14] A
   caller that sets a shorter horizon truncates instead.

**Case 2 is the only one that can happen in an ordinary configured game.** It
needs every faction to lose every site and every unit. That is an unlikely
state, and the measured share of 0.00 is consistent with it.

## 8. Whether a policy can read its distance to each threshold

| Path | Signal | What it measures | Reaches one at the win |
|------|--------|------------------|------------------------|
| Domination | `domination_progress` | Live seats held over live seats | Yes, for the seat clause. The unit clause is not published |
| Territory | `ground_progress` | Held tiles over world passable tiles | No. The reader holds no threshold |
| Wonder | `wonder_track_progress` | Work done over the work the row asks for | Yes |
| Renown | `renown_progress` | Best renown over the renown target | Yes |

Every signal is a bounded share in the closed interval from zero to one.[^11]
Each has a leader twin that carries the same share for the leading faction.

The question matters because a win is a threshold event and the training
reward is continuous. The reward pays a weight for the outcome once, and it
pays shaped terms for the published fields on every decision.[^24] A policy
that cannot read its distance to a threshold cannot learn to cross it on
purpose.

**Two paths are fully legible to a policy.** Wonder and renown publish exactly
the ratio their reader compares. A policy that climbs either signal walks
toward the win, and it reads one on the tick the reader fires.

**One path is half legible.** Domination publishes the seat clause exactly and
publishes nothing of the unit clause. A fog rule justifies the omission.[^10]
[^12] A policy that wins by annihilation cannot have steered toward that win
through this signal.

**One path is illegible.** Territory publishes a share of the world that its
reader never compares. A policy that climbs `ground_progress` does move toward
the win, because the reader ranks ground. The value it climbs toward is not
one. The policy therefore cannot tell how far it has to go. Two other readings
do carry the comparison the reader makes: `ground_gap` and `ground_rank`.[^10]
Neither is named as the progress signal of the path, and the reporting
instrument reads `ground_progress`.[^22]

## 9. Whether the paths are mutually reachable

**Renown and domination are close to the same strategy.** Both need soldiers,
campaigns and kills. A player that fells 200 enemy units in pursuit of renown
also destroys the armies that defend the rival seats. A player that takes
every seat also fells the units that defended them. The two paths differ in
which quantity crosses first, and not in what the player does.

**Territory and domination share a mechanism and differ in patience.**
Both grow held ground. Domination asks for particular tiles, which are the
seats, and it ends the game at once. Territory asks for the most tiles and it
ends the game only at the limit. A player that pursues domination and fails
still holds ground, so it is a candidate for territory. Territory is therefore
the fallback of domination and not a rival plan.

**Wonder is the one path with a different mechanism.** It needs workers on one
tile for a long time, and it needs the faction to keep that tile. It spends no
soldiers. It is the only one of the four that a peaceful player can reach.

**Pursuing wonder forfeits nothing outright, but it competes for units.** The
build verb orders the whole unit set of the faction.[^9] A tick spent
building is a tick not spent gathering or moving to a front.

**Territory can be won by a player that chose nothing.** The reader ranks the
factions at the limit and asks for no act. The product record names this as a
defect of the current game.[^12]

## 10. What the documents claim and the code does not do

**One doc comment states the opposite of what the code does.** The comment on
`WONDER_VICTORY_CLAIM` says that no reader compares it and that a finished
wonder ends no game.[^15] The wonder reader compares it and a finished wonder
ends a game.[^2] The comment describes the state before the wonder gained a
reader.[^25] The findings register holds the correction.[^26]

**No document claims a fifth win path.** The wealth path is retired. Two
records record the retirement, and the registry marks the earlier of them
superseded.[^25] The balance register marks the stock target row retired.[^13]
The stored path number of the wonder is the number the wealth path held, so
the number did not move.[^1]

**One completed backlog item names four paths as domination, wealth, wonder or
renown.**[^27] That is the title of the work that built the readers. A
completed item records one moment, so it is not a live claim.

**The product record states the needs this report measures against, and it
does not contradict the code.** It asks that each path be reachable by
arithmetic, that the distance a faction reads reach the winning value at the
win, and that no path be the outcome of making no act.[^12] Section 3 and
section 8 report that territory fails the second and the third of those.

## 11. The claims this report was asked to check

| Claim | Verdict |
|-------|---------|
| The territory reader fires only at the tick limit | **Verified.** The reader returns nothing below the limit |
| Territory is unreachable by construction | **Refuted.** It fires whenever a game reaches the limit with any faction left |
| A game reaching the limit is never unfinished, because territory decides it | **Verified.** Territory returns a faction whenever any contender exists |
| The domination share has a floor of one over the faction count | **Verified, with a correction.** The denominator is the live seated factions, not the configured faction count. The floor rises as factions leave, and it falls to zero if the faction loses its own seat |
| The domination share reaches one at the win | **Verified for the seat clause.** The signal is well formed against that clause |
| A second domination clause is unpublished for fog reasons | **Verified.** The unit clause is absent, and the observation states the fog reason |
| A game can end on domination while the share sits at its floor | **Verified.** The unit clause fires with no change to the share |
| The renown weight of the controller is read by no decision | **Verified.** The war, trade, build and settle weights each bias one keyed draw. The renown weight is read only by the observation, by the Python faction report and by the viewer panel |
| Every path's reach maximum is one in the larger sample | **Not checkable from the code.** The report repeats it as another agent's measurement |

## 12. What is unverified

- The measured shares in this report are another agent's readings. This report
  took no measurement and confirms none of them.
- Whether a build order concentrates workers on one tile in practice is not
  established here. The code says a builder works on the tile it stands on,
  and the signal reports the best tile. What a trained policy actually does is
  a measurement, not a code fact.
- Whether a champion survives long enough to reach the renown target under a
  given policy is a measurement. The code says only that renown leaves the
  arena with the character.

## References

[^1]: The win path enumeration, the territory comparison and the tick limit default. `crates/cachette-core/src/controller.rs`
[^2]: The win path readers and the standing. `crates/cachette-core/src/world/victory.rs`
[^3]: The controller stage, which calls the readers. `crates/cachette-core/src/world/controller.rs`
[^4]: The frame step, which increments the tick. `crates/cachette-core/src/world/step.rs`
[^5]: The balance values a world holds. `crates/cachette-core/src/balance.rs`
[^6]: The rule that removes a faction from the game. `crates/cachette-core/src/world/sites.rs`
[^7]: ADR-0150, held ground is the ground within reach of a city its faction owns. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
[^8]: The verb enumeration and the action table. `crates/cachette-core/src/action.rs`
[^9]: The verb legality and the verb application. `crates/cachette-core/src/world/actions.rs`
[^10]: The faction observation, the win tracks and the field table. `crates/cachette-core/src/faction_observation.rs`
[^11]: The bounded share. `crates/cachette-core/src/sim_math.rs`
[^12]: PRD-0057, more than one way to win decides a game. `docs/product/shaped/prd-0057-more-than-one-way-to-win-decides-a-game.md`
[^13]: Balance register, the tick limit, the renown target, the wonder work and the retired stock target. `docs/reference/balance.md`
[^14]: The training world constants. `python/cachette/learn/__main__.py`
[^15]: The upgrade table, the wonder row and the build rate. `crates/cachette-core/src/upgrade.rs`
[^16]: ADR-0175, a win threshold decides when a reader fires. `docs/adrs/draft/adr-0175-a-win-threshold-decides-when-a-reader-fires.md`
[^17]: The build pass and the build intent. `crates/cachette-core/src/world/upgrades.rs`
[^18]: The unit type table. `crates/cachette-core/src/unit_type.rs`
[^19]: Blockers register, BLK-150. `docs/BLOCKERS.md`
[^20]: The renown award and the champion rule. `crates/cachette-core/src/world/character.rs`
[^21]: The renown share for one felled unit. `crates/cachette-core/src/contest.rs`
[^22]: The ending report and the path to signal join. `python/cachette/learn/endings.py`
[^23]: The learner environment and its horizon. `python/cachette/learn/env.py`
[^24]: The reward and its terminal outcomes. `python/cachette/learn/reward.py`
[^25]: ADR-0174, a wonder is a win path and a stock total is not. `docs/adrs/draft/adr-0174-a-wonder-is-a-win-path-and-a-stock-total-is-not.md`
[^26]: Findings register, FND-737. `docs/FINDINGS.md`
[^27]: Backlog item 0479, end the game on domination, wealth, wonder or renown. `docs/backlog/complete/0479-end-the-game-on-domination-wealth-wonder-or-renown.md`
