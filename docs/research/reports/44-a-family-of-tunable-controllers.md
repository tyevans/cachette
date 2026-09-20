# Report 44: A family of tunable controllers, and how to rank them

This report answers one question. What settings turn the built-in controller
into a family of opponents of different strength, and what measurement puts
that family in order?

The project trains a policy against one opponent. That opponent wins the even
share of a three-faction world, and the policy wins about a fifth of the even
share. A policy that loses almost every game receives almost no win signal. A
set of opponents ordered by strength gives the run an opponent the policy can
beat early and a stronger one to cross later. **The value of a variant is
therefore how far it moves the family apart, and not how well it plays.**

**Four answers follow.**

**Seven of the thirteen settings sit on the world.** Only the five weights and
the external-control flag sit in the faction row. Two factions of one world
cannot hold different values of the other seven. A family of opponents that
shares one world must therefore be built out of the weight vector and the
external-control flag today.

**Six of the thirteen settings steer nothing.** The trade subsystem records no
offer, no contract and no carrier over a whole run, so the four trade settings
and the trade weight change no behaviour. The renown weight is drawn, stored
and published, and no decision reads it.

**The territory path is unreachable, and it is unreachable by design.** Its
reader fires at the tick limit and nowhere else. Every other reader fires
earlier, so the territory reader only ever sees a game that no other reader
ended. Over the 24 games the project has measured, none reached the limit.

**A controller cannot aim at a win path.** The build order picks one of seven
upgrade categories from a uniform draw, and the wonder is one of the seven. No
setting biases that draw. Two engine changes would widen this family, and this
report keeps them apart. Widening the weight bound down to zero is the cheapest
one. A weight over the build categories is the only one that would let a
controller aim at a path.

## 0 Provenance, and what this report did not measure

The author read the source in this repository, read the registers, and did
arithmetic on figures the registers already hold. The author ran no training,
started no instance, stepped no world and changed no engine behaviour.

Each claim below carries one of four marks.

**Read.** The author read the code and states what it does.

**Measured.** The figure comes from a register entry that records a run that
already happened. The entry names the machine and the world.

**Derived.** The figure is arithmetic over a read constant or a measured
figure. A blocker governs every cost figure in this report.[^blk7]

**Predicted.** The figure is a forecast that this report offers so that a
measurement can refuse it. No predicted figure belongs in a register.

**The training world is the world every figure here is about.** It is a square
world of extent 128, with three factions, a tick limit of 6000 ticks and a
decision every 10 ticks. One file declares it.[^trainworld]

The author was told three win shares from a run of 64 episodes on that world:
the built-in controller 0.333, the trained policy 0.070, and a seat that sends
no action 0.000. **The author did not verify those three figures.** A register
entry holds a passive seat at 0.000 over a smaller set, and this report cites
that entry where it uses the figure.[^fnd710]

## 1 What the controller decides, and where

The controller is one stage at the end of the step. It acts only through the
verbs that a Python caller can call, and it never reaches past them.[^adr144]
Two files hold it. One holds the decision rules and the parameter table.[^ctrl]
The other reads the world, builds one row for each faction, and applies the
plan.[^stage]

The stage runs in a fixed order on every tick. First it folds the last tick
into the census and empties the two logs. Then it runs the game end
readers.[^victory] Then it scans the unit arena once, in slot order, and finds
for each faction its speaker, its cohort and whether it holds a settler. Then
it reads the held ground of every faction, names a rival and a prey for each,
runs the road and zone solver, and builds one state row for each faction.
Then it plans, sorts the plan by faction and by draw index, and applies each
command through a verb.

**Every decision is one keyed draw or no draw at all.** A draw takes the
controller system, the tick, the faction and a draw index. There is no stored
random state.

### 1.1 The decisions, in the order of their draw index

| Decision | Draw index | What decides it |
|---|---|---|
| Gather or build, and which kind | 0 to `evaluations - 1` | The build weight splits the two. The kind comes from the high bits of the same draw, uniform over the kinds |
| Move the relation toward a rival or a prey | `evaluations` | A prey moves every tick with no draw. A rival moves on a draw the war weight biases |
| Raise a campaign against a tile | `evaluations + 1` | The war weight biases the draw. The world already chose the objective |
| Rewrite the trade board | `evaluations + 2` | The advertisement schedule. The draw only breaks a tie between two goods |
| Take one negotiation step | `evaluations + 3` | The trade weight biases the draw |
| Assign and release carriers | `evaluations + 4` | No draw. The stage acts when a contract owes a quantity |
| Send idle units to the zoned projects | `evaluations + 5` | No draw. The stage acts when the plan holds a project and no campaign runs |
| Put one unit type in a site queue | `evaluations + 6` | A uniform draw over the filled rows of the unit type table |
| Send water-crossing units over water | `evaluations + 7` | No draw. The world chose the tile from a bounded sample |
| Found a city from every settler | `evaluations + 8` | The settle weight biases the draw |

### 1.2 The economy

A gather order sends every unit of the faction at one resource kind. A build
order sends every unit of the faction at one upgrade category. **The kind and
the category both come from a uniform draw**, over three resource kinds and
over seven upgrade categories.[^upgrade] The build weight decides only which
of the two orders the faction gives.

The engine refuses a build order for each unit that cannot take it, and it
leaves that unit's earlier order standing. **A second evaluation therefore
mixes the two orders across the unit set** rather than replacing the first.
Each evaluation orders the whole set, so the last evaluation of a tick decides
the order of every unit the verb accepted.

### 1.3 Expansion and founding

A faction founds a city from every unit whose type row carries a settle group.
The settle weight biases one draw for the whole faction on each tick. The verb
refuses a unit whose settle column is zero and a place that breaks the minimum
founding distance.

A site queue builds a typed unit, and the type comes from a uniform draw over
the rows the unit type table fills.[^adr158] **One rule overrides that draw.**
A faction with no live unit that carries command reach queues a leader
instead, because a faction with no speaker can move no relation and can
therefore never reach war.

### 1.4 Military and combat

The war weight drives two separate decisions with one number. It biases the
relation move against a rival, and it biases the campaign raise. A campaign
takes a cohort of units, sets that cohort to the soldier row and marches it at
the nearest enemy site.[^campaign]

**A campaign is the only thing in a seeded run that makes a unit able to
kill.** Every founded unit carries the worker row, whose attack is zero, and
the contest pass requires the attack of the attacker to exceed the armour of
the defender. A worker never penetrates a worker.[^fnd486] The item that gives
the seeded world types that can fight is open.[^item491]

### 1.5 Relations and war

A relation is one signed integer for each ordered pair, and a pass reads a
threshold.[^adr146] The controller moves it one step toward war at a time, and
a drift moves it back toward peace. A
faction reaches the war band after enough steps, and only then does the world
offer it a campaign objective.

**A prey outranks a rival, and hunting a prey needs no draw.** A faction
overmatches another when its own held ground reaches the overmatch multiple of
the other's. It then moves the relation toward that prey on every tick,
whatever its war weight says. This is the one rule in the controller that
takes a weight out of the decision.

### 1.6 Upgrades

An upgrade is a category at a level, with a ground fit and a work cost. The
controller never names a category on purpose. It draws one of the seven, and
the wonder is one of the seven.

### 1.7 Trade

A faction writes a board of offers and wants from what its sites hold, matches
its wants against another faction's offers, and binds a contract that carriers
then serve.

**The whole chain records nothing over a run.** A sweep of 16 seeds of the
demonstration world, each played to 20000 ticks, recorded zero offers, zero
contracts and zero bound carriers in every seed. The cause is that all three
goods map onto the one commodity a store holds, so every faction offers all
three goods at the same quantity and wants none of them. A match needs one
faction to want what another offers, and none ever does.[^item181]

## 2 The knobs

The table below holds every setting of the controller and of the game end.
**Settable** says whether Python can write it today. **Scope** says whether the
value sits in the faction row or on the world. A world-level value cannot
differ between two seats of one world.

### 2.1 The settings that exist

| Setting | Scope | Settable | Range that means something | Steers |
|---|---|---|---|---|
| War weight | Faction | Yes[^bindfaction] | 1 to 8, giving a per-tick chance of 0.111 to 0.500 | The relation move and the campaign raise |
| Build weight | Faction | Yes[^bindfaction] | 1 to 8, same chance range | The share of evaluations that build rather than gather |
| Settle weight | Faction | Yes[^bindfaction] | 1 to 8, same chance range | The founding draw |
| Trade weight | Faction | Yes[^bindfaction] | 1 to 8 | Nothing that reaches the world today |
| Renown weight | Faction | Yes[^bindfaction] | 1 to 8 | Nothing. No code reads the field |
| External control | Faction | Yes[^bindfaction] | On or off | Off gives the seat to the controller. On and with no action sent gives a passive seat |
| Evaluations for each faction each tick | World | Yes[^bindfaction] | 0 upward | How many gather or build orders one faction gives in one tick |
| Overmatch ratio | World | Yes[^bindrelation] | A Q16.16 factor. At or below zero the rule leaves the game | Which faction a strong faction hunts, and how early |
| Surplus mark | World | Yes[^bindtrade] | A store quantity | Nothing that reaches the world today |
| Carriers for each contract | World | Yes[^bindtrade] | 0 upward | Nothing that reaches the world today |
| Contract term | World | Yes[^bindtrade] | Ticks | Nothing that reaches the world today |
| Advertisement schedule | World | Yes[^bindtrade] | A period and a phase | Nothing that reaches the world today |
| Tick limit | World | Yes[^bindworld] | Ticks | When the territory reader fires |

**Every setting above except the external-control flag carries a provisional
default that a register row holds, and every one of those rows is
unset.**[^balance] Pass 10 is the balance harness, and it has not run. No value
in the table above is measured.

**No Python file under the control plane package calls any of the
setters.** The training harness only flips the seat between the controller and
a policy. Every setting above therefore holds its default in every run the
project has made.

### 2.2 The rules that no setting reaches

Each rule below decides controller behaviour and holds a literal that nothing
can write.

| Rule | Where it is fixed | Why it bounds this family |
|---|---|---|
| The weight bound is 1 to 8 | Two constants in the controller[^ctrl] | A weight of zero is refused, so no faction can be made to never fight, never build or never found |
| The build category draw is uniform over seven | The evaluation function[^ctrl] | The wonder is one category of seven. No setting aims a faction at the wonder path |
| The gather kind draw is uniform over three | The evaluation function[^ctrl] | No faction can prefer food, wood or stone |
| The relation step is one toward war | One constant in the controller[^ctrl] | No faction can declare faster, and none can move toward peace |
| A prey moves the relation with no draw | The plan function[^ctrl] | The overmatch rule cancels the war weight for the faction that has a prey |
| The rival is the faction with the most held ground | The rival function[^ctrl] | No faction can pick a different enemy |
| The campaign objective is the nearest enemy site | The campaign module[^campaign] | No faction can choose a target |
| The queue type draw is uniform over the filled rows | The queue draw function[^ctrl] | No faction can build an army rather than workers |

**The first two are the ones worth changing.** Widening the weight bound down
to zero doubles the span of every weight, because a weight of zero removes a
behaviour rather than making it rare. A weight over the build categories is
the only way a controller could aim at the wonder path.

### 2.3 The settings that are rules of the game, not settings of a player

Four further values are settable and must not differ between seats: the renown
target, the renown given for each unit felled, the work a wonder costs, and
the victory claim the wonder row carries.[^adr175] A record states that a
threshold decides when a reader fires and never what the simulation does. A
family that varied one of these would compare two different games rather than
two players. **Hold all four fixed across a ranking run.**

## 3 The win paths, and whether each is reachable

Four readers run in a fixed order on every tick, inside the controller stage:
domination, territory, wonder, then renown. The first that fires writes the
end record, and the record is written once.[^adr148] After that the controller
plans nothing.

Over 24 games of the world a run trains on, at an extent of 128 and a tick
limit of 6000, the paths fell out as follows.[^fnd710]

| Path | Games | Measured or derived |
|---|---|---|
| Renown | 13 | Measured |
| Wonder | 7 | Measured |
| Domination | 4 | Measured |
| Territory | 0 | Measured |

### 3.1 Domination

**What the engine requires.** Either the faction holds the seat tile of every
faction still in the game and at least one of those seats belongs to a rival,
or every rival that is still in the game has no live unit while the faction
has one.[^victory]

**Can a controller steer toward it?** Yes, weakly. A high war weight raises
more campaigns, a campaign is the only path to a siege, and a siege is what
takes a seat. No setting names a seat as a target, so the faction takes
whichever enemy site is nearest.

### 3.2 Territory

**What the engine requires.** The world tick must reach the tick limit. The
reader then names the faction with the most held tiles.[^victory]

**Can a controller steer toward it? No, and no player can.** The reader is a
residual. It fires only in a game that the other three readers left alone for
the whole run. A faction that wanted the territory path would have to stop two
opponents from finishing a wonder and from reaching the renown target, and it
holds no verb that does either.

**This is why the reading is zero and not small.** Every one of the 24 games
resolved before tick 6000. A path that fires only when nothing else fires is
not a path a family of players can be spread along.

**A ranking run should still report it.** The endings instrument publishes the
held-ground share of the seat and of the leader on every episode, and that
share is the one quantity a settling variant should move.[^endings] The
variant that leads on held ground and still loses is the clearest evidence
this report expects.

### 3.3 Wonder

**What the engine requires.** A finished upgrade row that carries a victory
claim above zero must stand on ground the faction holds. The wonder row
carries a claim of one, and it costs 14400 work.[^upgrade]

**Can a controller steer toward it? Only by volume.** A faction reaches the
wonder category on one build order in seven, and the draw is uniform. Raising
the build weight from 1 to 8 raises the share of evaluations that build from
0.111 to 0.500, which is a factor of 4.5 on the rate of wonder work. That is
the whole of the available steering.

### 3.4 Renown

**What the engine requires.** A live character of the faction must reach the
renown target. The target is 50 renown points. The contest gives the champion
of a faction one quarter of a point for each unit that faction fells.[^balance-table]
**The target is therefore 200 fells, and nothing else in a seeded run writes
the column.**[^contest]

**Can a controller steer toward it?** Yes, and this is the strongest steering
the family has. Renown is the modal path in the measurement above, fells come
from campaigns, and campaigns come from the war weight.

**The renown weight steers none of it.** The engine draws the field from the
seed, stores it in the faction row, hashes it and publishes it to Python and
to the view panel. No decision reads it. This is an inert declaration, which
is a defect shape this project already records.[^inert]

**A blocker governs the wider renown rule.**[^blk150] This report states no
renown value and proposes no change to one.

## 4 The proposed variants

Each variant below is a seat configuration. **Every setting a variant names is
per-faction and settable today, except where the row says otherwise.** The
seven world-level settings hold their defaults in every variant, because two
seats of one world cannot differ on them.

The predicted win share is the share of games the variant wins in a
three-faction world in which the other two seats hold other members of this
family. **Every predicted figure is a forecast and none is measured.**

| Name | War | Build | Settle | Trade | Renown | Other | Predicted win share |
|---|---|---|---|---|---|---|---|
| `mute` | — | — | — | — | — | External control on, no action sent | 0.000 |
| `quietist` | 1 | 1 | 1 | 8 | 1 | — | 0.10 |
| `settler` | 1 | 2 | 8 | 1 | 1 | — | 0.20 |
| `mason` | 1 | 8 | 4 | 1 | 1 | — | 0.28 |
| `default` | Drawn | Drawn | Drawn | Drawn | Drawn | The shipped controller | 0.333 |
| `warlord` | 8 | 3 | 4 | 1 | 1 | — | 0.45 |
| `hunter` | 8 | 1 | 1 | 1 | 1 | Overmatch ratio 1.0, per faction | 0.35 to 0.55 |

### 4.1 What each is predicted to be strong and weak against

**`mute`.** The passive seat. It is the floor of the family and it is already
measured at 0.000 over 12 games.[^fnd710] It is strong against nothing. It
exists so that a rating scale has a fixed bottom.

*Hypothesis.* It wins no game in any seating. A single win refuses it.

**`quietist`.** It holds the lowest reachable war, build and settle weights
and spends its highest weight on the trade subsystem, which transacts nothing.
It is the weakest member that still plays. It is strong against `mute` alone.
It is weak against every member that fights, because it draws for a campaign
on about one tick in nine and therefore fells few units.

*Hypothesis.* Its win share is below 0.15 and above 0.02. A share at or above
0.20 refuses it, and so does a share of zero.

**`settler`.** It founds often and fights rarely. It is predicted to lead the
family on held ground and to lose anyway, because the reader that rewards held
ground fires only at a tick limit that no game reaches.

*Hypothesis.* Its ninth-decile held-ground reach is the highest in the family,
and its win share is below the `default`. A settler that leads on both refuses
it, and that outcome would mean the territory path is doing work this report
says it cannot do.

**`mason`.** It builds on half its evaluations. One build order in seven names
the wonder, so its wonder work accrues about 4.5 times as fast as the
`quietist`. It is strong against a family that fights slowly, because a wonder
ends a game that nobody else has ended. It is weak against `warlord`, which
takes its ground before the work finishes.

*Hypothesis.* Its share of episodes ending on the wonder path is at least
twice the `default`, and its win share is **not** separated from the
`default`. **This report does not predict that `mason` beats the `default`.**
A wonder share that fails to double refuses the first half.

**`default`.** The shipped controller, with the weight vector drawn from the
seed. It is the anchor of the scale and the opponent every stored policy was
trained against. **Its share in a world where every seat holds it is one third
by symmetry**, and that is a derivation and not a measurement. A run of 64
episodes reported 0.333, and the author did not verify that run.

**`warlord`.** Maximum war weight, with enough build weight to keep an
economy. It draws for a campaign on one tick in two and draws for a relation
move at the same rate, so it reaches the war band early and fells units
for the rest of the run. Renown is the modal path and fells are its source, so
this is predicted to be the strongest member that needs no engine change. It
is weak against nothing in this family, and it would be weak against a player
that walls its sites, which no member of this family does.

*Hypothesis.* Its win share is above 0.45, and renown is the modal ending of
its games. A share at or below 0.39 refuses it.

**`hunter`.** Maximum war weight and an overmatch ratio of 1.0, so it hunts
any faction that holds no more ground than it does. The overmatch rule then
moves its relation every tick with no draw, which is faster than any war
weight can reach. It
starves its own economy: it builds on one evaluation in nine, so it finishes
few upgrades and its cities stay small.

*Hypothesis.* This report offers no point prediction and states the
uncertainty instead. `hunter` either beats `warlord` or falls below the
`default`. **An outcome between the two refuses the reasoning**, because it
would mean that neither the faster declaration nor the starved economy
dominates.

**`hunter` needs the overmatch ratio to be held for each faction.** That work
is in flight and this report does not do it. Until it lands, `hunter` cannot
be seated beside a member that holds a different ratio, and the family runs
with six members.

### 4.2 Why the family spans what it spans

The four tuned members that need no engine change are separated by the weight
range alone, and that range is narrow. A weight of 1 gives a chance of
0.111 and a weight of 8 gives 0.500, so the widest available factor on any one
behaviour is 4.5. **A weight of zero would remove the behaviour instead, and
the bound refuses it.** Widening the bound down to zero is the cheapest change
that widens this family, and section 6.3 states it as work.

## 5 The measurement that ranks them

### 5.1 The instrument that already exists

The project holds a rating tool.[^league] It seats players in unordered
triples, plays each triple three times on one world with the seats turned by
one each time, fits one strength for each player from the winners alone under
the Luce choice model, and reports the strength on the Elo scale with the
built-in controller anchored at zero. It takes a fixed number of solver steps
and tests nothing for convergence, so two runs give one answer. Its error bars
come from a bootstrap over worlds rather than over games, because the three
rotations of one triple share a world.

**The tool cannot seat two controller configurations.** A player holds either
a stored policy or nothing, and nothing means the built-in controller. There
is one such player and its knobs are the defaults. Section 6 states the change.

### 5.2 How many games the ranking needs

A win share in a three-seat world has an even value of one third. The standard
error of a share near one third is the square root of two ninths over the game
count.

| Games for one player | Standard error of its share | Derived or measured |
|---|---|---|
| 64 | 0.059 | Derived |
| 120 | 0.043 | Derived |
| 180 | 0.035 | Derived |
| 256 | 0.029 | Derived |
| 576 | 0.020 | Derived |

Two players are separated when the difference of their shares exceeds about
twice the standard error of that difference, and the difference of two
independent shares carries the square root of two times one share's error. At
576 games each, that separation is 0.056.

The schedule fixes the game count. With `p` players and three seats, the tool
plays every unordered triple three times on each world. One player then plays
three times the number of triples that hold it, on each world.

| Players | Triples | Games for each world | Games for one player for each world |
|---|---|---|---|
| 5 | 10 | 30 | 18 |
| 6 | 20 | 60 | 30 |
| 7 | 35 | 105 | 45 |

A player sits in the triples that hold it, which is the number of pairs the
other players make. At five players that is six triples and 18 games for each
world. At six players it is ten triples and 30 games. At seven it is fifteen
triples and 45 games.

### 5.3 The plan

**Stage one, the screen.** Seat the six members that need no engine change over
4 world seeds. That is 240 games, and 120 games for each member. The standard
error of one share is 0.043. **Read the screen by the point estimate alone.** A
bootstrap over 4 worlds carries too few units to quote, and this report does
not quote one.

The screen refuses a hypothesis whose predicted share is more than 0.10 from
the measured one, which is a gap of about 2.3 standard errors.

Cost: 240 games. One passive episode of this world costs 23.7 seconds on
sixteen cores, measured.[^fnd710] The screen is therefore about 1.6 hours of
one such machine, derived. A blocker governs every cost figure.[^blk7]

**Add `hunter` to the screen when the per-faction overmatch ratio lands.**
Seven members give 420 games over the same 4 seeds, 180 games for each member,
a standard error of 0.035, and about 2.8 hours. The `hunter` question is the
one the screen answers best, because its two predicted outcomes are far apart.

**Stage two, the ranking.** Drop to five members: the two anchors, `mute` and
`default`, and the three that the screen leaves furthest apart. Play 32 world
seeds. That is 960 games and 576 games for each member. The standard error of
one share is 0.020 and the separation is 0.056. The bootstrap then resamples
32 worlds, which is enough units to quote an interval.

Cost: about 6.3 hours of the same machine, derived.

**The whole plan is 1200 games and about 7.9 hours of one sixteen-core
machine, derived.** With `hunter` in the screen it is 1380 games and about 9.1
hours.

### 5.4 What the plan cannot do, stated before it runs

The predicted shares of `mason` and the `default` differ by 0.05, which is
below the 0.056 that stage two separates. **The plan cannot separate those
two, and this report does not claim that it can.** The `mason` hypothesis is
therefore written against the wonder ending share rather than against the win
share, and the ending share is what the run must report for it.

The plan also cannot rank two members that differ only in a world-level
setting, because one world holds one value of each. A ranking of the
evaluation count, the overmatch ratio before it becomes per-faction, or any
trade setting needs a separate run for each value, against a fixed family.
**Do not put a world-level setting in the round robin.**

### 5.5 What the run must report

Report the strength of each member on the Elo scale with its interval, the
win share of each member, the seat share of the league itself, and the ending
share and reach of each member on all four paths.[^endings] The ending shares
are what refuse the `settler` and `mason` hypotheses, and the win share alone
refuses neither.

## 6 What this design asks somebody to build

**6.1 Let the rating tool seat a controller variant.** A player today holds a
policy or nothing. Add a third case: a named set of per-faction settings that
the tool writes to that faction before the world runs. This is the whole of
the harness work, and the setters it needs are already bound to
Python.[^bindfaction]

**6.2 Hold the overmatch ratio for each faction.** Another worker holds this,
and this report does not touch it. Until it lands, `hunter` cannot be seated.

**6.3 Widen the weight bound down to zero.** The bound refuses a weight of
zero today, so no faction can be made to never fight. A weight of zero gives a
chance of zero, which removes a behaviour rather than making it rare, and that
is what widens a family more than any other single change. It is one constant
and the validity check that reads it. **It changes the world hash of every
seeded run**, because the seeding draws each weight from the widened range.
That cost must be stated before the change, not after.

**6.4 Give the build order a category weight.** This is the only change that
would let a controller aim at a win path. It is larger than the other three
and this report does not specify it. A decision record would be a deliverable
of it, because the choice of what a faction builds is policy that a learner
may also make.[^adr156]

**6.5 Do not build a knob for the trade subsystem.** Four settings and one
weight already exist and steer nothing. A fifth would be a fifth inert
declaration. The item that gives a kind of work its own commodity is what
opens that subsystem, and it is already in the backlog.[^item181]

## 7 What surprised the author

**The renown weight reaches nothing, and renown is the modal win path.** The
field is drawn, stored, hashed, published to Python and printed by the view
panel, and no decision reads it. A reader of the faction row would reasonably
conclude that a faction can be aimed at the renown path. It cannot.

**The war weight drives two decisions with one number.** It biases the relation
move and the campaign raise. A variant cannot declare often and campaign
rarely, or the reverse.

**The overmatch rule cancels the war weight.** A faction with a prey moves its
relation on every tick with no draw. The strongest lever in the controller is
therefore the one that takes the weight vector out of the decision.

**The territory path is a residual and not a path.** The reader fires at the
tick limit and nowhere else. The reading of zero over 24 games is not a balance
problem to fix with a value. It is what a residual reader does in a game that
always resolves early.

## References

[^blk7]: Blockers register, BLK-007, most cost figures are still derived on the target platform. `docs/BLOCKERS.md`
[^trainworld]: The training world, its extent, faction count and tick limit. `python/cachette/learn/__main__.py`
[^fnd710]: Findings register, FND-710, the win share separates a passive seat from a chance seat. `docs/FINDINGS.md`
[^adr144]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decisions D1 and D2. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
[^ctrl]: The controller, its constants and its decision rules. `crates/cachette-core/src/controller.rs`
[^stage]: The controller stage, where it reads the world and applies the plan. `crates/cachette-core/src/world/controller.rs`
[^victory]: The game end readers. `crates/cachette-core/src/world/victory.rs`
[^upgrade]: The upgrade table, the categories and the wonder row. `crates/cachette-core/src/upgrade.rs`
[^adr158]: ADR-0158, a site builds a typed unit from a bounded queue its store pays for, decision D2. `docs/adrs/accepted/adr-0158-a-site-builds-a-typed-unit-from-a-bounded-queue-its-store-pays-for.md`
[^campaign]: The campaign register, the cohort and the objective. `crates/cachette-core/src/campaign.rs`
[^fnd486]: Findings register, FND-486, every founded unit carries the worker row. `docs/FINDINGS.md`
[^item491]: Backlog item 0491, seed the demonstration world with unit types that can fight, gather and carry. `docs/backlog/refined/0491-seed-the-demonstration-world-with-unit-types-that-can-fight-gather-and-carry.md`
[^adr146]: ADR-0146, a faction relation is one signed integer per ordered pair, and a pass reads a threshold, decision D3. `docs/adrs/accepted/adr-0146-a-faction-relation-is-one-signed-integer-per-ordered-pair-and-a-pass-reads-a-threshold.md`
[^item181]: Backlog item 0181, give a kind of work the commodity it fills. `docs/backlog/refined/0181-give-a-kind-of-work-the-commodity-it-fills.md`
[^bindfaction]: The faction bindings, the weight verb and the evaluation count. `crates/cachette-py/src/world/faction_view.rs`
[^bindrelation]: The relation bindings, the overmatch ratio. `crates/cachette-py/src/world/relations.rs`
[^bindtrade]: The trade bindings, the four trade settings. `crates/cachette-py/src/world/trade.rs`
[^bindworld]: The construction bindings, the tick limit. `crates/cachette-py/src/world/construction.rs`
[^balance]: Balance register, the controller section. `docs/reference/balance.md`
[^adr175]: ADR-0175, a win threshold decides when a reader fires and never what the simulation does, decisions D1 and D2. `docs/adrs/draft/adr-0175-a-win-threshold-decides-when-a-reader-fires.md`
[^adr148]: ADR-0148, a game end is recorded once and stops the controllers, decisions D2 and D3. `docs/adrs/accepted/adr-0148-a-game-end-is-recorded-once-and-stops-the-controllers.md`
[^endings]: The endings instrument, the ending share and the reach of each path. `python/cachette/learn/endings.py`
[^balance-table]: Balance register, the renown target and the renown for each unit felled. `docs/reference/balance.md`
[^contest]: The contest pass, the renown one felled unit gives. `crates/cachette-core/src/contest.rs`
[^inert]: Recurring defect shapes, shape 3, inert code that nothing invokes. `.agents/rules/recurring-defects.md`
[^blk150]: Blockers register, BLK-150, nobody has said what raises and lowers renown. `docs/BLOCKERS.md`
[^league]: The policy rating tool. `scripts/policy_league.py`
[^adr156]: ADR-0156, a faction's option weights are policy, set through one verb, decisions D1 and D3. `docs/adrs/accepted/adr-0156-a-factions-option-weights-are-policy-set-through-one-verb.md`
