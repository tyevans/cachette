# What a game attains on every win quantity

This report measures what a game of Cachette reaches on every quantity a win
condition could compare. It plays whole games, samples every faction over the
course of each game, and states a distribution for each quantity.

The report designs no win condition and changes no engine behaviour. It states
numbers. A separate audit states what the code does.

**A threshold must come from a distribution somebody measured.** The project
once carried a tick limit of 6000 that a commit body called measured, and that
commit cites no measurement.[^1] Every figure below names its sample size and
its error. A later reader can therefore see what the sample supports.

## 0. What this measured, and what it cannot say

**Every figure below measures one moment.** The moment is the working tree of
this branch on 9 September 2026. The machine is one development machine on
x86-64. It ran a gate, a training run and other agents at the same time.
No figure here is a target platform measurement.

The world is the world the training runs play. It holds 128 columns, 128 rows
and three factions. Its tick limit is 6000 ticks, and it takes a decision every
10 ticks. The seeds start at 50,000, which is the held-out start the rating
tools use. No training run learned on them.[^2]

The sample is small. **Two fields of players played 60 games between them.** A
share near one half over 24 games carries a standard error near 0.10. The same
share over 36 games carries a standard error near 0.083. Read every share
below against its own error. Do not read a gap of one error as a
difference.[^3]

The report says nothing about a trained policy. Every seat here holds the
built-in controller under one of its named configurations.

## 1. How the measurement works

Two tools produce every number. The first plays the games and records the
value of every followed quantity, for every faction, every 100 ticks.[^4] The
second reads that record and states the quantiles.[^5]

The first tool takes the seat schedule, the version family and the world
builder from the controller version tool.[^6] The players here are therefore
the players that tool ranks. The first tool adds two readings that no existing
tool takes. It reads a faction in the middle of a game. It also reads every
faction rather than the training seat.

Two tools already report on endings, and neither takes those readings. The
endings module summarises finished episodes. It reads the values a record
stored at episode end.[^7] The controller version tool reads the end record
and nothing else.[^6]

Two commands produced every figure below. A third command summarises either
file.

```
python scripts/win_quantity_trajectories.py --fields calm \
    --worlds 12 --chunk 6 --workers 6 --threads 1 --out calm.json
python scripts/win_quantity_trajectories.py --fields fierce \
    --worlds 8 --chunk 4 --workers 4 --threads 1 --out fierce.json
python scripts/win_quantity_summary.py calm.json \
    --trend counts.store_total shares.renown_progress
```

### The scale of each figure

The engine publishes a share as a fixed-point integer, and it publishes the
value that stands for one. The tools divide by that published value, so a
share below runs from zero to one.

The engine publishes a count as a compressed magnitude, and it publishes the
parameters that invert the compression. The tools call that inversion and hold
no compression rule of their own. **The inversion carries a relative error of
0.0004**, which the schema states. A difference under that error is not a
difference the observation carries.

Three of the recovered counts are raw fixed-point quantities and not whole
numbers. The store total, the best renown and the military strength each cross
at the fixed-point scale of this project, which is Q16.16. A reader divides a
raw value by 65,536 to get whole units. The held tiles, the settlements, the
live units and the finished upgrades are plain counts.

### Two fields of players

A distribution taken from one controller describes that controller. The report
therefore plays two fields, at the calm end and at the aggressive end of the
named family.[^6]

| Field | Seats | Games |
|---|---|---|
| calm | quietist, settler, mason | 36 |
| fierce | default, warlord, hunter | 24 |

The quietist holds the lowest war weight and the highest trade weight. The
settler and the mason build. The warlord holds the highest war weight, and the
hunter holds the highest war weight with the hunting ratio at parity. Each
field played 12 or 8 worlds. Each world hosted one rotation for each seat.
Every player of a field therefore held every seat the same number of times.

### The seat reading and the leader reading

Each quantity reports two readings. The own reading is the value one faction
held, and the report pools the three factions of each game. The leader reading
is the highest value over the factions of one game. A threshold must be
reachable by somebody and not by everybody, so both readings matter.

**The tool computes the leader rather than reading it.** The engine publishes
a leader signal for each of the four win paths, and none for the store total.
A computed leader therefore gives one rule for every quantity.

### The two trajectory figures

The report states two figures that an end-of-game reading cannot give.

**The arrival** is how far into a game a quantity reaches nine tenths of the
value it ends at. It runs from zero to one. A quantity with a small arrival
figure decides the game long before the game ends, and it makes a poor
threshold. A quantity that never moves reads an arrival of zero, and that
reading states a constant rather than a saturation.

**The rise share** is the share of the sample steps at which the leader value
went up. A step that holds level is out of the share. A quantity that only
accumulates reads one. A quantity that a contest moves reads below one.

## 2. Game length and ending path

| Path | calm, 36 games | fierce, 24 games |
|---|---|---|
| domination | 0.000 ± 0.000 | 0.250 ± 0.088 |
| territory | 0.056 ± 0.038 | 0.000 ± 0.000 |
| wonder | 0.556 ± 0.083 | 0.417 ± 0.101 |
| renown | 0.389 ± 0.081 | 0.333 ± 0.096 |
| no end record | 0.000 | 0.000 |

Every game of both fields ended. No game held an empty end record.

| End tick | p10 | median | p90 | max | at the limit |
|---|---|---|---|---|---|
| calm | 2256 | 3504 | 4932 | 6000 | 2 of 36 |
| fierce | 1781 | 3380 | 4712 | 5574 | 0 of 24 |

The median game uses about 58 percent of the tick limit. **The two games that
reached the limit both ended on territory.** No other game ended on territory.
The territory reader fires at the tick limit and nowhere else. A territory
ending and a game that runs out of ticks are therefore one event.

The path decides the length.

| Path | end tick median | games |
|---|---|---|
| domination, fierce | 1815 | 6 |
| wonder, calm | 2790 | 20 |
| renown, fierce | 3525 | 8 |
| wonder, fierce | 3659 | 10 |
| renown, calm | 3836 | 14 |
| territory, calm | 6000 | 2 |

A domination game is the shortest and the most variable. Its tenth percentile
end tick is 211, so a faction can take every rival seat in the first few
hundred ticks.

**The aggression of the players decides which paths fire.** The calm field
ended no game on domination, and the fierce field ended a quarter of its games
that way. The gap is 0.250 with an error near 0.088, so the sample supports
it. The fierce field ended no game on territory, and the calm field ended two.
That second gap is 0.056 with an error of 0.038, and the sample does not
support it.

## 3. The four published progress signals

Each table gives the value at the end of the game. The four figures are the
tenth percentile, the median, the ninetieth percentile and the highest value.

### The calm field, 36 games

| Signal | own | leader |
|---|---|---|
| domination_progress | 0.333 / 0.333 / 0.500 / 0.667 | 0.333 / 0.333 / 0.667 / 0.667 |
| ground_progress | 0.049 / 0.199 / 0.419 / 0.716 | 0.189 / 0.340 / 0.517 / 0.716 |
| wonder_track_progress | 0.025 / 0.327 / 1.000 / 1.000 | 0.274 / 1.000 / 1.000 / 1.000 |
| renown_progress | 0.000 / 0.362 / 1.000 / 1.000 | 0.002 / 0.640 / 1.000 / 1.000 |

### The fierce field, 24 games

| Signal | own | leader |
|---|---|---|
| domination_progress | 0.000 / 0.333 / 0.500 / 1.000 | 0.333 / 0.500 / 0.617 / 1.000 |
| ground_progress | 0.000 / 0.117 / 0.488 / 0.801 | 0.141 / 0.415 / 0.612 / 0.801 |
| wonder_track_progress | 0.000 / 0.151 / 1.000 / 1.000 | 0.077 / 0.545 / 1.000 / 1.000 |
| renown_progress | 0.000 / 0.462 / 0.997 / 1.000 | 0.020 / 0.690 / 1.000 / 1.000 |

### What each signal is, as a quantity

**The domination share is a seat count and not a track.** A faction that holds
only its own seat of three reads one third, which is the floor. The share
moves in whole seats, so a three-faction world admits three readings: 0.333,
0.667 and 1.000. The tables above hold a median of 0.500 for a leader, because
a median over an even count falls between two readings. A quantity with three
values carries no threshold that a bar could sit inside.

**The ground share is the held tiles over the passable tiles of the world.**
It is continuous, and no game of this measurement brought it near one. The
leader ended at 0.340 in the median calm game and at 0.415 in the median
fierce game. The highest leader reading over all 60 games is 0.801.

**The wonder share is a step and not a track.** The wonder reader fires when a
finished wonder stands on ground the faction holds. The share holds the work
toward the next claim until a claim completes. It then reads one. The leader
reads exactly 1.000 in the median calm game, because the calm field ends most
of its games that way.

**The renown share is the best renown of the faction over the renown target.**
It is the one path with a continuous track and a bar at the end of it. The
leader ended between 0.64 and 0.69 in the median game of both fields. It
reached 1.000 in every game the renown reader ended.

## 4. Wealth, and the other economic quantities

The engine publishes the store total of a faction. The total sums every
commodity of every live settlement, as a raw fixed-point quantity. This is the
quantity the shaped training reward calls wealth.

**No reader compares the store total.** The engine says so in its own source:
the value is reported and not read. A record states the decision that removed
the clause that compared it.[^8]

### What a game reaches

The table gives the raw fixed-point value, with the whole units beside it.

| Field | reading | p10 | median | p90 | max |
|---|---|---|---|---|---|
| calm | own | 870,849 | 20,128,447 | 60,935,466 | 224,146,722 |
| calm | leader | 10,979,685 | 46,525,257 | 89,957,322 | 224,146,722 |
| fierce | own | 0 | 15,103,349 | 77,922,048 | 155,519,948 |
| fierce | leader | 17,033,727 | 46,852,326 | 119,779,413 | 155,519,948 |

| Field | reading | p10 | median | p90 | max |
|---|---|---|---|---|---|
| calm | leader, whole units | 167.5 | 709.9 | 1372.6 | 3420.0 |
| fierce | leader, whole units | 259.9 | 714.9 | 1827.7 | 2372.9 |

The two fields agree closely at the median and part in the tail. **The
aggressive field produced the poorer seats.** Its tenth percentile own reading
is zero, because a faction that a rival eliminated holds no settlement and
therefore no store. The richest game of the measurement is a calm game, and 36
calm games against 24 aggressive games is not a sample that orders the two
tails.

### Wealth grows with the length of the game

The leader store total rises through the game. The table gives the median
leader value at each tick, over the games still running at that tick.

| Tick | calm, games | calm median | fierce, games | fierce median |
|---|---|---|---|---|
| 500 | 36 | 13,550,373 | 22 | 15,636,587 |
| 1000 | 36 | 23,086,686 | 22 | 24,057,737 |
| 2000 | 35 | 38,062,968 | 20 | 34,319,825 |
| 3000 | 23 | 44,456,276 | 16 | 46,199,657 |
| 4000 | 15 | 45,214,990 | 6 | 42,095,906 |

**Read the game count of each row.** A row past tick 3000 states the median of
the games that were still running. Those are the games no reader ended early.
The rise from tick 500 to tick 2000 rests on nearly the whole sample.

### Wealth is not a pure accumulator

The rise share of the leader store total is 0.63 in the calm field and 0.58 in
the fierce field. **Wealth therefore falls at about four steps in ten.** A
faction spends its stores, so the total is not a clock and not a monotone
count.

Two other economic quantities behave differently. The finished upgrade count
reads a rise share of 1.00 in both fields, so it never falls. The military
strength reads 0.60 and 0.57, so a contest moves it as much as it moves
wealth.

### The other economic quantities

The table gives the leader reading at the end of the game.

| Quantity | field | p10 | median | p90 | max |
|---|---|---|---|---|---|
| held_tiles | calm | 1522 | 3077 | 5044 | 9705 |
| held_tiles | fierce | 1244 | 4008 | 6464 | 8970 |
| settlements | calm | 5.5 | 12.5 | 20.5 | 46.0 |
| settlements | fierce | 4.6 | 16.0 | 28.7 | 38.0 |
| finished_upgrades | calm | 178 | 414 | 1011 | 1687 |
| finished_upgrades | fierce | 59 | 432 | 753 | 1150 |
| live_units | calm | 57.0 | 102.5 | 223.5 | 310.0 |
| live_units | fierce | 37.2 | 123.0 | 243.9 | 277.9 |
| military_strength | calm | 262,121 | 524,154 | 819,162 | 1,179,445 |
| military_strength | fierce | 327,589 | 556,988 | 851,895 | 982,852 |

**The population field and the live unit field publish one number.** Every
reading of the two agreed in every game of this measurement. The project
already holds that finding.[^9]

## 5. Which quantities saturate, and which stay contested

The table gives the median arrival and the median rise share of the leader
value. A small arrival means the quantity settles early.

| Quantity | calm arrival | calm rise | fierce arrival | fierce rise |
|---|---|---|---|---|
| domination_progress | 0.04 | 1.00 | 0.21 | 1.00 |
| ground_progress | 0.77 | 0.91 | 0.86 | 0.94 |
| wonder_track_progress | 0.93 | 1.00 | 0.95 | 1.00 |
| renown_progress | 0.95 | 1.00 | 0.93 | 1.00 |
| store_total | 0.60 | 0.63 | 0.71 | 0.58 |
| held_tiles | 0.77 | 0.91 | 0.86 | 0.94 |
| settlements | 0.70 | 1.00 | 0.81 | 0.94 |
| finished_upgrades | 0.95 | 1.00 | 0.95 | 1.00 |
| live_units | 0.72 | 0.79 | 0.79 | 0.80 |
| military_strength | 0.57 | 0.60 | 0.51 | 0.57 |

**Only the domination share settles early.** Its calm arrival of 0.04 states a
constant and not a saturation. The calm field never
took a rival seat, so the share sat at its floor for the whole game. The
fierce arrival of 0.21 is a real early settlement. A faction that takes a seat
takes it early, and the share then holds.

**Three quantities stay contested to the end.** The renown share, the wonder
share and the finished upgrade count each read an arrival above 0.90. Their
leader value is still rising when the game ends.

**Wealth and military strength settle in the middle.** Wealth reaches nine
tenths of its final value at about two thirds of the game, and it then moves
in both directions. Military strength reaches nine tenths at about half the
game, and it is the least monotone quantity measured.

## 6. Where this agrees with the larger sample, and where it does not

A training run reported the endings instrument over 3,072 episodes on the same
world. That sample seats one untrained learner against two built-in
controllers, at generation zero. Its shares are domination 0.09, territory
0.05, wonder 0.62 and renown 0.24, and it recorded no episode without an end
record.

**The two measurements agree on three statements.**

The territory share is small and it is not zero. The larger sample reads 0.05
and the calm field reads 0.056 ± 0.038. Both readings are consistent.

No game fails to end. Both measurements read a share of zero for an episode
with no end record, over 3,072 episodes and over 60 games.

The wonder path is the most common ending. The larger sample reads 0.62. The
calm field reads 0.556 ± 0.083 and the fierce field reads 0.417 ± 0.101. The
calm reading is consistent with 0.62. The fierce reading sits two of its own
errors below it. A sample of 24 games is too small to call that a difference.

**The two measurements disagree on one statement, and the disagreement is a
property of the players.**

The larger sample reads a domination share of 0.09. The calm field reads
0.000 ± 0.000 and the fierce field reads 0.250 ± 0.088. **The domination
share is not one number of the game.** It runs from zero to a quarter across
the range of the built-in controller, and the larger sample sits inside that
range. A threshold set against the larger sample alone would be set against
one point of a wide range.

The renown share reads 0.24 in the larger sample, 0.389 ± 0.081 in the calm
field and 0.333 ± 0.096 in the fierce field. The gap between the calm reading
and the larger reading is 0.149 against an error near 0.081, which is 1.8
errors. **The sample does not separate them.**

**The larger sample carries an unfinished share of 0.00 beside a territory
share of 0.05.** This measurement explains that pairing. The territory reader
fires at the tick limit, so a game that runs out of ticks ends on territory. Both games of this measurement that reached the limit ended
that way, and no game that ended early ended that way.

## 7. Which quantities are viable bases for a win condition

The report ranks the five quantities against three tests. The first test asks
whether a bar sits inside the range the quantity reaches. The second asks
whether the quantity stays contested to the end of the game. The third asks
whether somebody reaches the quantity and not everybody.

**The renown track is the strongest base.** It is continuous, it never falls,
and the leader ends the median game at about two thirds of the target. It
reaches the target in about a third of the games. Its arrival above 0.93 says
that the leader is still gaining renown when the game ends. A bar anywhere
inside the range therefore parts the games.

**The ground share is the second strongest.** It is continuous and it stays
contested to about four fifths of the game. The highest leader reading over the 60
games is 0.801, and the median is 0.34 to 0.42. A bar on held ground is
therefore a real bar. The calm leader passed 0.5 in fewer than one game in ten,
because its ninetieth percentile is 0.517. The aggressive leader passed 0.5
more often, because its median is 0.415. The reader that exists compares the factions at the tick
limit and holds no bar at all.

**The finished upgrade count is a viable base that nothing reads.** It never
falls, its arrival is 0.95, and its leader reading spans 59 to 1687 over the
two fields. It is an achievement count rather than a store, so it does not
carry the objection that applies to wealth.

**Wealth is the weakest base a bar could sit inside.** A bar sits inside its
range: the leader ends between 167 and 3420 whole units. It falls at four
steps in ten. A faction can therefore cross a bar and fall back under it, and
a reader would have to state what it does then. Its distribution moves with
the length of the game, so a wealth bar acts partly as a clock. **The project
already decided that a stock total wins no game**, and nothing in this
measurement contradicts that decision.[^8]

**The domination share is not a threshold quantity.** A three-faction world
gives it three values. A bar on it is a choice between the three, and the
engine already makes that choice in its reader.

## 8. The wealth reward reaches no win path

The shaped reward the training runs use weighs two signals. It weighs the
store total at 1.0 and the held tiles at 0.5.

**No reader compares the store total.** The engine states that in its own
source, and a record holds the decision.[^8] The dominant term of the training
reward therefore reaches no win path at all.

The minor term reaches one path. The territory reader compares held tiles, and
it fires only at the tick limit. That path ended 0.056 ± 0.038 of the calm
games and none of the fierce games.

The store total and the four win quantities do not move together. Wealth falls
at about four steps in ten, and the renown share and the wonder share never
fall. A policy that maximises the shaped return therefore raises a quantity no reader
reads. That quantity also moves against the quantities the readers do read.

**This report does not measure a trained policy, so it does not prove the
cause of any win share.** It states that the reward and the win paths compare
different quantities, and that the measured behaviour of those quantities
differs.

## 9. What a larger run costs

**The cost figure the project carried before this run understates the true
cost.** A brief stated about 45.2 processor-seconds for a game of roughly
3,660 ticks. This measurement disagrees.

One clean timed run played three calm games of 7,682 ticks in total and spent
111.13 processor-seconds. That gives 37.0 processor-seconds a game and 14.5
processor-seconds for each thousand ticks. **Those three games are short**, at
a mean of 2,561 ticks, and they are not typical.

A second run played the calm field at 12 worlds. It spent 10,566
processor-seconds and had not finished its 72 games when it was stopped. **A
calm game of this world therefore costs more than 147 processor-seconds on
this machine.** The cost of one tick rises with the unit count and the
settlement count of the game. A long game therefore costs more than the product
of its ticks and a short game's rate.

The two runs this report uses played 60 games and 207,522 simulated ticks in
total.

A larger run on a 64-core machine would cost about 147 processor-seconds a
game as a lower bound. At 60 cores in use that is about 2.5 seconds of wall
clock a game. **A run of 600 games would then take about 25 minutes**, and a
run of 3,000 games about two hours. Add a margin: the figure above is a lower
bound taken on a loaded machine.

Two design points would make a repeat cheaper. The probe writes its file only
when every game is finished, so a run that is stopped loses everything. A
chunk of worlds runs until its slowest world ends, so a chunk of 12 costs the
length of its longest game.

## 10. What this report did not measure

- **No trained policy.** Every seat holds the built-in controller.
- **No target platform figure.** The machine is a development machine on
  x86-64, and it was loaded.
- **No trade quantity.** The engine publishes a contract share, and a design
  report states that the trade subsystem records no offer and no contract over
  a whole run.[^6]
- **No world other than the training world.** A larger world or a different
  faction count would move the domination floor and the ground share.
- **No sample large enough to order the players.** The win shares of the three
  players of each field sit within one or two errors of each other.

## References

[^1]: Findings register, FND-733. `docs/FINDINGS.md`
[^2]: The rating tool of the stored policies, the held-out seed start.
`scripts/policy_league.py`
[^3]: Findings register, FND-736. `docs/FINDINGS.md`
[^4]: The trajectory probe. `scripts/win_quantity_trajectories.py`
[^5]: The trajectory summariser. `scripts/win_quantity_summary.py`
[^6]: The controller version tool, and the family it holds.
`scripts/controller_versions.py`
[^7]: The endings module. `python/cachette/learn/endings.py`
[^8]: ADR-0174, a wonder is a win path and a stock total is not, decisions D1
and D2.
`docs/adrs/draft/adr-0174-a-wonder-is-a-win-path-and-a-stock-total-is-not.md`
[^9]: Findings register, FND-702. `docs/FINDINGS.md`
