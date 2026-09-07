# Report 33: What a learner can see, say and be scored on

This report asks one question. Can a policy that reads the faction observation
and writes one action integer play this game well from any seat?

The report answers it for each strategic area the engine holds. For each area
it names the decisions a competent player must take. It then names the
observation fields that serve those decisions, and the action verbs that serve
them. It then asks one more thing. Can a reward built from the fields on offer
tell a good decision from a bad one?

The report proposes no architecture. It reports capability gaps.

## 0 What the engine has done since this report

**Every measurement below is fixed to the moment section 1 names, and none of
them was edited afterwards.** The engine has moved on two of the gaps this
report ranked. A reader who plans from the table in section 7 must know which
two. A register holds each case with its evidence.[^1] [^2]

| Gap | State |
|---|---|
| The observation length does not identify the world, so a policy loads on the wrong one | Closed. A weight file states the world it fits, and a reader refuses a file that does not match |
| The cell unit count and the cell held count do not separate own from other | Closed. Each is now a pair, relative to the faction that reads |

The second change moved the field set, so it moved the layout version. Section
2 measures the field set as it stood before the change.

**Section 2.1 says that no decision record carries the number 0184. The
registry has since allocated that number**, to a record on how a verb narrows
the set it acts on.[^3] The statement in section 2.1 describes the moment it
measured, and the number was free then.

## 1 Method, and the moment this measures

The author read the engine, the binding and the learner package. The author
then built one world of the training shape. The author read the two schemas,
the legality answer and the census from it. Every number below that is marked
as measured came from that run.

**Every count in this report measures one moment.** The moment is the tip of
the main branch on 7 September 2026. A count belongs in a research report,
because a report is fixed to a moment. The same count in a decision record
decays.[^4]

The author ran no test suite and no full check.

## 2 The interface, as the engine publishes it

The engine publishes the observation as one flat array of signed integers, and
one schema that names each field.[^5] It publishes the action as one integer
over a mixed radix table, and one schema that names each verb.[^6] Two
accepted records fix both shapes.[^7] [^8]

One world of the training shape gives these lengths. The shape is 48 tiles
wide, 48 tiles high, and three factions.

| Quantity | Measured value |
|---|---|
| Observation fields | 28 |
| Observation positions | 176 |
| Action verbs | 12 |
| Action rows | 29 |
| Cells of the map lattice | 4 |
| Tiles each cell covers | 1024, 512, 512, 256 |

### 2.1 Two premises of the brief are wrong

The brief that commissioned this report states that the observation holds 45
fields. It holds 28 fields and 176 positions. The field list in the module is
the whole layout, and it names 28 entries.[^5]

The brief also cites ADR-0184 as a draft that widens the verb set. **No
decision record carries the number 0184.** The registry allocates no such
row.[^9] A completed backlog item carries that number, and it is about scoring
a forage option against food.[^10] Nothing in the tree widens the verb set.

An earlier report audited this surface and found that no source file cited the
three records that specify it.[^11] That report is fixed to 6 September 2026.
The surface is now built. The observation module, the action module and the
world all cite the records by number.

## 3 The learner acts about ten times less often than a rival

This is the largest difference between the learner and the baseline it loses
to. It is a property of the interface. No policy causes it.

The built-in controller emits several commands for one faction on one tick. It
makes a fixed number of evaluations, and each evaluation emits one gather
order or one build order. It then emits up to nine further commands, each
gated by its own keyed draw or by a schedule.[^12]

The learner emits **one** action for one decision, and the environment then
runs the world for the decision interval.[^13] The shipped interval is five
ticks.

A measurement fixes the ratio. One world of the training shape ran 300 ticks
with the learner seat under external control. The engine counted 1227
controller commands over the two remaining factions.[^14] That is 2.045
commands for one faction on one tick.

| Actor | Commands for one faction on one tick |
|---|---|
| Built-in controller, measured | 2.045 |
| Learner at the shipped interval | 0.200 |
| Learner at an interval of one tick | 1.000 |

A rival therefore takes about ten decisions for each one the learner takes. At
the best interval the environment offers, a rival still takes twice as many.
No policy class recovers a factor of ten in decision count.

**This is a real handicap and not a fair contest.** The baseline the project
calls strongest runs the built-in controller in the learner's own seat.[^13]
That baseline acts at the controller rate. The learner does not.

## 4 The strategic areas

The areas below come from the engine. Each names the decisions a player faces.
Each then names the fields that serve those decisions, the verbs that serve
them, and the legibility of the signal.

### 4.1 Choosing and pursuing a win path

The engine holds four win paths, and one reader for each. The readers run in a
fixed order: domination, territory, wonder, renown.[^14]

**What a player must decide.** Which path to pursue. When to switch. Whether a
rival is closer to a path than the player is. Whether to race a rival to a
wonder or to break the wonder instead.

**Can the policy see it?**

| Path | Own progress | Rival progress |
|---|---|---|
| Domination | `seats_held` | none |
| Territory | `held_tiles` | none |
| Wonder | `wonder_progress`, `wonder_claim` | none |
| Renown | `best_renown` | none |

The array carries no field for any rival's standing on any path. It carries no
renown target, so the renown field has no scale beside it. The array carries
`tick` and `tick_limit`. That pair is the matching threshold of the territory
path. The omission is therefore an asymmetry and not a rule.[^15] A findings
row already records it.[^16]

The domination path is the one path a policy can read to a conclusion. It sees
`seats_held`, and the relation field holds one position for each faction, so
the faction count is legible. A `seats_held` equal to the faction count is a
win.

**Can the policy act on it?** Only obliquely. No verb names a win path. The
policy pursues territory through `settle` and `project`, the wonder through
`build`, and renown through `campaign`. It cannot state a target.

**Is the signal legible?** Partly. The reward reads the observation array, and
it weighs a change in `held_tiles`, `seats_held`, `wonder_progress`,
`wonder_claim` and `best_renown`.[^17]

Every one of those is an own-side quantity. Take two players that each gained
twenty tiles. A rival of the first gained fifty, and a rival of the second
gained none. A reward built from these fields reports the same number for
both. **The relative position, which is the whole of a race, is invisible.**

**The fogged form of the fix.** A player can see a rival's cities and the
ground around them. The observation should therefore say who holds the tiles a
faction observes, for each cell. The encoding must be relative to the
observing faction. Such an encoding holds four positions: own, allied, hostile
and unheld. It therefore satisfies the record that refuses a field indexed by
faction.[^18]

### 4.2 Expansion, and holding ground

**What a player must decide.** Where to put the next city. Whether to expand
or to consolidate. Which direction to grow toward. When land runs out.

**Can the policy see it?** Weakly. Nine fields answer for each cell of the map
lattice: `cell_seen_now`, `cell_seen_ever`, `cell_tiles`, `cell_open_tiles`,
`cell_units`, `cell_held_tiles`, `cell_value_total`, `cell_height_total` and
`cell_food_total`. On the training world these hold four positions each.
Section 5 measures what that costs.

Two gaps stand apart from the resolution question.

**`cell_held_tiles` counts the tiles that anybody holds.** It does not say
who. A measured position gives the array `[44, 184, 0, 0]` for a faction whose
own `held_tiles` reads 165. The policy cannot tell its own ground from a
rival's inside one cell.

**No field carries a settlement count, a settlement position or a site
store.** The policy cannot see how many cities it has, where they stand, or
which of them is rich.

**Can the policy act on it?** The `settle` verb founds a city from every
settler that stands on ground a city may take. **The verb names no place.**
The engine resolves the set, and the founding rule refuses a place too near an
existing city.[^14] The `cross` verb sends water-crossing units at the tile
the engine surveys, and it names no tile either.[^6]

The policy therefore chooses whether to expand. It never chooses where. Where
a settler walks is decided by the engine.

**Is the signal legible?** Yes for the fact of expansion, no for its quality.
A reward on `held_tiles` rises when a city is founded anywhere. It cannot
separate a city on good ground from a city on bad ground. The reward reads no
field that describes the ground of a site.

### 4.3 Construction

The engine holds seven upgrade categories. They are road, terrace, wonder,
store, wall, lodging and one open row. The default table leaves the open row
empty.[^19] The `build` verb declares one category position, so the policy
chooses among all seven.

**What a player must decide.** What to build, where, and in what order.
Whether to raise a wonder and race for it. Whether to build lodging to lift
the housing bound. Whether to build a store to lift the store capacity.
Whether to repair what wears.

**Can the policy see it?** Almost nothing. The array carries `wonder_progress`
and `wonder_claim`. It carries no field for any other category. It carries no
count of standing upgrades, no condition, and no project list.

The plan solver writes a bounded list of projects for each faction.[^20] The
`project` verb sends idle units to them.[^21] **No observation field describes
the plan.** The policy cannot see whether the plan is full, empty, or about to
be taken.

**Can the policy act on it?** It can name a category. It cannot name a tile,
and it cannot zone a project. The Python interface holds `zone_projects` and
`clear_projects`, and neither is a verb of the action table.[^22]

A second limit is severe. **A build order reaches every live unit of the
faction.** The verb takes the whole unit set and orders each of them to build
the named category.[^14] There is no selector. The policy cannot build a road
with half its workers and a terrace with the other half.

**Is the signal legible?** For the wonder, yes: `wonder_progress` moves. For
every other category, no. A road, a terrace, a store, a wall and a lodging all
move no field of the array directly. Their effects reach the array only
through second-order quantities such as `held_tiles` and `population`, many
decisions later. A reward cannot attribute those to the build order that
caused them.

### 4.4 Economy

The engine holds three gatherable resource kinds: food, wood and stone.[^23]
The `gather` verb declares one resource position.

**What a player must decide.** Which good to gather, and in what proportion.
Whether a shortage is coming. Whether to trade for a good rather than gather
it. Where to gather.

**Can the policy see it?** One scalar. `store_total` sums every commodity of
every live settlement of the faction, as one raw fixed-point number.[^14]
**The policy cannot see which good it lacks.**

One indirect channel exists. The five board fields hold every faction's trade
board. One board row states a good and a quantity. It also states whether the
faction offers the good or wants it.

A faction writes its board from its own site stores, against a surplus
mark.[^12] The board therefore reveals the per-good position of whoever wrote
it. The board is public state by decision, so reading a rival's board breaks
no fog rule.

The channel is weak in three ways. The board is rewritten on a schedule, so it
is stale between writes.

It reports only the sign and the size of the departure from one mark. It says
nothing at all before a faction first advertises.

**Can the policy act on it?** It can order a gather of one kind. As with
build, the order reaches every live unit, so no division of labour is
possible. It cannot set a production ratio, and no verb takes one. The Python
interface holds `prefer_at_sites` and `spend_at_sites`, and neither is a
verb.[^22]

**Is the signal legible?** Barely. A reward on `store_total` rises when any
good accumulates. It cannot reward gathering the scarce good over the abundant
one. The term that would separate the two does not exist. **A learner cannot
be taught to fix a shortage it cannot see.**

**The fogged form of the fix.** A faction knows its own stores exactly. One
position for each resource kind, over the faction's own sites, reveals nothing
a player would not have. That is three positions on the present catalogue.

### 4.5 People and growth

A site proposes a birth at a rate its store sets. The free places of the site
admit the proposal.[^24] A site at its housing bound grows nobody, whatever
food it holds. The production queue is the only consumer of people. The
`queue` verb puts one entry of one unit type into the queue of a site.[^6]

**What a player must decide.** Whether to grow or to spend. Which unit types
to make. Whether lodging is needed before more people are worth having. Which
city should produce.

**Can the policy see it?** It sees `population` and `live_units`. It sees
nothing else about people. It cannot see the housing bound, the free places, a
birth rate, or a starvation event. **A policy at the housing bound reads
exactly what a policy with room to grow reads.** That is the case where the
correct decision changes, and the field that would separate the two cases is
absent.

The policy also cannot see what unit types it holds. The type table holds
eight rows, and the worker, soldier, merchant, leader, mariner and settler
rows carry distinct capabilities.[^25] No observation field reports the
composition of the faction's units.

**Can the policy act on it?** Yes, and this is the verb with the most
strategic content in the table. The `queue` verb declares a unit type position
over all eight rows. Queuing a settler enables expansion. Queuing a soldier
enables a campaign. Queuing a leader enables the relation verb, which refuses
a faction with no unit that carries command reach.[^14]

**The verb names no site.** The engine takes the lowest-slot site of the
faction whose queue has room.[^14] A player with many cities cannot direct
production.

**Is the signal legible?** For growth, yes: `population` moves. For
composition, no. Queuing a settler and queuing a soldier both raise
`population` by the same amount. No field separates the two afterwards.

### 4.6 Military

A campaign is one row of faction, objective and cohort.[^26] The `campaign`
verb raises one against the objective the engine resolves, and it declares no
argument position.[^6]

A site changes hands when a besieger presses long enough. The taker keeps a
site it can supply and burns one it cannot, and burning costs more.[^27] The
engine decides which of the two happens. No verb offers the choice.

**What a player must decide.** Whom to attack, and where. When to attack.
Whether to defend a city under siege or to counter-attack elsewhere. How large
a force to commit. When to stop.

**Can the policy see it?** Very little.

- `cell_units` counts the units standing on the tiles of a cell the faction
  sees now. **It does not say whose.** The policy cannot distinguish its own
  army from an invading one.
- No field reports a live campaign, its objective, or its progress.
- No field reports a siege on any of the faction's own sites.
- No field reports a rival's unit count, even inside observed ground.

**Can the policy act on it?** It raises a campaign, or it does not. The engine
chooses the objective. The chooser prefers a site of the faction whose ground
a war rival now holds. It otherwise takes the nearest observed site of a war
rival.[^14] Defence and offence are therefore both engine choices.

The reader is correctly fogged: it filters the candidate sites to those the
faction has observed.[^14]

The cohort size is a world parameter, not an argument, so the policy cannot
choose a force size.[^26]

**Is the signal legible?** Poorly. A campaign that succeeds raises
`held_tiles` and `seats_held` many decisions later. A campaign that fails
lowers `live_units`.

The reward can weigh `live_units`. The correct sign of that weight is not
obvious. A faction that trades units for a seat has done well. **No field
reports what a campaign achieved.**

### 4.7 Diplomacy and trade

A relation is one signed integer for each ordered pair of factions. A band is
a threshold: alliance, peace and war.[^28] [^29]

**What a player must decide.** Whom to fight and whom to leave alone. When to
make peace. Whom to trade with, and on what terms. Whether to honour a
contract.

**Can the policy see it?** The relation field holds one position for each
faction. Each position carries what the observing faction feels toward that
faction. That is a genuine and useful signal. It does not carry what the other
faction feels back, and the matrix is not symmetric.

The five board fields hold every faction's board, which is the public market
statement. No field reports a contract, its terms, its remaining term, or
whether it is being delivered.

**Can the policy act on it?** Here the table holds a hard limitation.

**The relation verb moves a relation in one direction only.** The step the
verb applies is a constant, and its value is negative one, which is one step
toward war.[^12] [^14] **No verb makes peace, and no verb forms an alliance.**
A learner can declare war on anybody. It can never end one.

The engine returns a pair to peace only through the drift schedule. That
schedule moves each entry one step toward the peace band on a timer.[^28] A
player therefore has no agency over reconciliation at all.

The `trade` verb takes one negotiation step against the faction the engine
resolves. **It names neither a partner nor terms.** The engine answers a
pending offer when one exists. It otherwise opens an offer against the first
board match it finds.[^14] The price is a midpoint, and no draw decides
it.[^12]

The `advertise` verb rewrites the whole board from the site stores against the
surplus mark. It takes no argument, so the policy cannot state what it wants.

**Is the signal legible?** No. Nothing in the array reports a delivered
contract, a broken one, or a gain from trade. A completed contract leaves one
trace in the array, which is a change in `store_total`. Every other economic
event moves that field too.

### 4.8 Hazards

The world holds wildfire. A fire is a sparse set of burning tiles, and it
burns to a conclusion on its own.[^30] The census counts `fires_started` and
`fires_doused`, so both sides of the mechanic are live.

**What a player must decide.** Whether to send units to fight a fire, and
which fire.

**Can the policy see it?** No. **No observation field reports a fire
anywhere.** A burning tile is visible to a player who watches it, so this is
not a fog constraint. It is an omission.

**Can the policy act on it?** No. The Python interface holds `order_douse` and
`stop_dousing`.[^22] Neither is a verb of the action table. **A learner cannot
fight a fire.**

**Is the signal legible?** No. A fire that burns a faction's ground reaches
the array only later. It arrives as a fall in `held_tiles` or in `population`.
The policy could not have acted on it in any case.

The same holds for weather. The interface holds readers and writers for wind,
water and storms. No observation field and no verb reaches any of them.

### 4.9 Division of labour, which no verb permits

This is a limitation that cuts across every area above, so it gets its own
section.

**Every verb that reaches units reaches all of them.** The `gather` verb takes
the whole live unit set of the faction. So does the `build` verb. The `settle`
verb takes every settler that stands on legal ground. The `project` verb takes
every idle unit.[^14]

The design principle that a set-valued command permits a cheaper algorithm is
correct and is not in question. The gap is that **no verb takes a subset**. A
player cannot send half the workers to gather food and half to build a road.
The whole faction does one thing.

This bounds the strategy a policy can express more than any missing field
does. A competent player of any game of this shape divides labour. Nothing in
the table lets one.

The built-in controller has the same limit, so this does not explain why the
controller wins. It explains the ceiling on both.

## 5 The 48-tile world blinds the policy to space

The engine partitions the world into blocks of a fixed edge. The block edge
exponent is five, so a block is 32 tiles by 32 tiles.[^31] The observation
lattice, the fog layer and the summary level share that one lattice.[^7]

A world 48 tiles on a side therefore holds two blocks on each axis, and four
cells in total. The four cells cover 1024, 512, 512 and 256 tiles, because the
world edge cuts three of them.

**The whole map, at training resolution, is four numbers for each quantity.**
Thirty-six of the 176 observation positions describe space. The rest are
scalars, the relation row, the boards and the weight vector.

### 5.1 What this costs

It removes every spatial decision a player would take. A cell of 1024 tiles is
about 44 per cent of the world. A policy cannot see a frontier, a chokepoint,
a direction of threat, or the shape of its own territory. It reads four
buckets and one of them holds nearly half the world.

The founding rule ties the two together. The engine asserts that a settler's
reach must exceed one cell edge. A target inside the settler's own cell steers
nobody.[^14] The same argument applies to a policy. A signal that cannot name
a place finer than one cell cannot direct movement within one.

### 5.2 It also blocks the stated goal

The goal is a general playability model that plays from any seat. The
observation length is a function of the cell count. The cell count is a
function of the world size. A weight matrix has the shape actions by
features.[^32] A policy trained on one world therefore states nothing about
another.

**This paragraph first said that such a load succeeds and computes the wrong
answer. That was true of some world pairs and not of others.** A measurement
separated the two cases, and a findings row holds it.[^1]

A block is 32 tiles on a side. Every world from 33 to 64 tiles on each axis
therefore holds four cells. At three factions each of those holds an
observation length of 176. A policy trained on a world 48 tiles on a side
loads on one 64 tiles on a side. It plays that world, and nothing raises.

A world of another band holds another length. The matrix product then refuses
it, with a message that names neither world.

The engine now refuses both. A weight file states the world it was trained
against. A reader refuses a file whose statement is not the world it is asked
to play.

### 5.3 The verdict asked for

**This is a blocker for general play, not an acceptable cost.** It is not a
blocker for the present milestone.

The distinction matters. A policy trained at this resolution can learn the
non-spatial game. It can learn when to queue a settler, when to advertise, and
when to raise a campaign. That is real progress and it is worth running. It
cannot learn any spatial skill, and no reward will teach it one. The input
that would carry the skill is absent.

Two fixes exist and both are cheap to state.

1. **Train on a world large enough to give a useful lattice.** A world 256
   tiles on a side gives 8 by 8 cells, which is 64 cells and 576 spatial
   positions. This changes no code.
2. **Give the observation its own lattice pitch, finer than the block.** This
   is the stronger fix and the more expensive one. It contradicts the record
   that says the observation shares the fog and summary lattice, so it needs a
   record.[^7] A findings row already warns that a second address space over
   the same world samples the wrong cells and fails nowhere.[^33]

The author recommends the first fix now. Take the second as a deliberate
decision. It buys resolution at the price of a second address space.

## 6 The action mask carries information the array does not

The engine answers, for one faction, one byte for each row of the action
table.[^14] A measured position gave 18 legal rows of 29.

That answer is a substantial observation channel, and it is fogged correctly.
The engine computes it from the readers that answer for one faction. It tells
the policy, among other things:

- whether the faction holds a unit that carries command reach, because the
  relation rows are legal only then;
- whether a campaign objective exists that the faction has observed;
- whether a negotiation step is due;
- whether a crossing target exists;
- whether a settler stands on ground a city may take;
- whether a site of the faction has room in its queue;
- which upgrade categories at least one unit could start.

**The policy does not receive it as input.** Both shipped policies apply the
mask as a filter over the action scores, after the scores are computed. The
feature encoder reads the observation array alone.[^32]

Feeding the mask into the features costs 29 extra features on the training
world. It changes no engine code. It hands the policy several facts that no
observation field carries. This is the cheapest capability gain in the report.

## 7 The gaps, in the order they block general play

The list is ordered by how much each blocks a policy that must play well from
any seat.

| Rank | Gap | Kind | Cost to close |
|---|---|---|---|
| 1 | The learner acts about ten times less often than a rival | Interface | Low |
| 2 | No verb takes a subset of the units, so no division of labour is possible | Verb | High |
| 3 | The map is four cells at training resolution, so no spatial skill is learnable | Observation | Low or high |
| 4 | No field carries any rival's standing on any win path | Observation | Medium |
| 5 | `cell_units` and `cell_held_tiles` do not separate own from other | Observation | Medium |
| 6 | `store_total` is one number, so no shortage is visible | Observation | Low |
| 7 | No verb makes peace or forms an alliance | Verb | Low |
| 8 | The observation length follows the world size, so no policy transfers | Interface | Medium |
| 9 | No field reports housing, free places or a birth rate | Observation | Low |
| 10 | The action mask is not a feature | Learner | Very low |
| 11 | No field reports a campaign, a siege or a contract | Observation | Medium |
| 12 | The `queue` verb names no site, so production cannot be directed | Verb | Medium |
| 13 | No verb and no field reaches a fire | Both | Medium |
| 14 | No verb names a place for a founding, a build or a crossing | Verb | High |

Notes on the ranking.

**Rank 1 first, because it is a handicap rather than a gap.** Every other item
is about what a policy could learn. This one is about a contest the learner
cannot win at any skill. Settle it before anybody chooses an architecture. A
result measured against an opponent that acts ten times more often measures
the handicap, not the policy.

**Rank 2 above rank 3, because it bounds strategy rather than perception.** A
policy that saw the whole world perfectly still could not divide its labour.

**Rank 10 is out of order by value and in order by blocking.** It blocks
little and costs almost nothing. A researcher should take it first.

## 8 The records this implies

The author writes no record. These are the decisions the findings above raise.
The lead allocates the numbers.

1. **How often a learner acts, and against what.** The rate asymmetry in
   section 3 is a decision nobody has recorded. Either the learner acts at the
   controller rate, or the baseline is handicapped to the learner's rate, or
   the project accepts the asymmetry and says why. The record that says a
   controller acts only through the caller's verbs is the one this joins.[^34]
2. **Whether a verb may name a subset of the units.** This is the largest
   design question in the report. It touches the principle that a set-valued
   command permits a cheaper algorithm, and it touches the action table
   record.[^8] A selector expressed as a bounded argument position, rather than
   as a list, would keep both.
3. **Whether the observation lattice may differ from the block lattice.** The
   present record says they are one lattice.[^7] Section 5 gives the case for
   parting them and the cost of doing so.
4. **What a policy may see of a rival.** Section 4 proposes an encoding
   relative to the observing faction, which is compatible with the record that
   refuses a field indexed by faction.[^18] Somebody must decide what a player
   is entitled to know.

One existing blocker governs several rows above. The rules of the downstream
game are not written down, and every reward weight is unset under it.[^35] A
reward that cannot weigh a term is one problem. An observation that cannot
carry a term is another. Section 4 is about the second.

## 9 What the author could not determine

These are marked unverified. The author did not confirm them.

- **Whether the learner in fact loses to the controller, and by how much.** The
  brief states it. This report did not measure it and did not read the training
  loop, which another worker is editing.
- **Whether the shipped decision interval of five ticks was chosen for a
  reason.** The environment states that a learner deciding on every tick spends
  its sample budget on ticks that changed nothing. The author found no measured
  figure behind the number five.
- **Whether the board fields give a usable economic signal in practice.** The
  reasoning in section 4 is from the code. Nobody has measured how often a
  board is stale.
- **Whether an eliminated faction is legible to a policy.** A faction that
  holds no site and no unit leaves the game under a draft record.[^36] The
  author did not check whether the observation reports the departure.

One minor defect surfaced during the reading, and it is reported rather than
fixed. The doc comment on the wonder upgrade category states that its
completion ends no game. It cites the draft record that removed the
reader.[^19] A wonder reader now exists and fires.[^14] The comment is false.

## References

[^1]: Findings register, FND-635. `docs/FINDINGS.md`
[^2]: Findings register, FND-636. `docs/FINDINGS.md`
[^3]: ADR-0184, a verb narrows the set it acts on by a bounded categorical position. `docs/adrs/draft/adr-0184-a-verb-narrows-its-set-by-a-bounded-categorical-position.md`
[^4]: Decision Record Scope, section 4.3. `.agents/rules/adr-scope.md`
[^5]: The faction observation module. `crates/cachette-core/src/faction_observation.rs`
[^6]: The action table module. `crates/cachette-core/src/action.rs`
[^7]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
[^8]: ADR-0176, an action integer is a mixed radix over the positions a verb declares. `docs/adrs/accepted/adr-0176-an-action-integer-is-a-mixed-radix-over-the-positions-a-verb-declares.md`
[^9]: ADR Registry. `docs/adrs/REGISTRY.md`
[^10]: Backlog item 0184, score the forage option against food. `docs/backlog/complete/0184-score-the-forage-option-against-food.md`
[^11]: Report 31, the state of the learner surface. `docs/research/reports/31-the-state-of-the-learner-surface.md`
[^12]: The controller module. `crates/cachette-core/src/controller.rs`
[^13]: The learner environment. `python/cachette/learn/env.py`
[^14]: The world module. `crates/cachette-core/src/world.rs`
[^15]: The balance table, the renown target. `crates/cachette-core/src/balance.rs`
[^16]: Findings register, FND-582. `docs/FINDINGS.md`
[^17]: The reward module. `python/cachette/learn/reward.py`
[^18]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D3. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
[^19]: The upgrade module. `crates/cachette-core/src/upgrade.rs`
[^20]: The plan module. `crates/cachette-core/src/plan.rs`
[^21]: ADR-0152, a faction plans its roads and zones with one solver. `docs/adrs/accepted/adr-0152-a-faction-plans-its-roads-and-zones-with-one-solver.md`
[^22]: The Python binding crate. `crates/cachette-py/src/lib.rs`
[^23]: The resource module. `crates/cachette-core/src/resource.rs`
[^24]: The growth module. `crates/cachette-core/src/growth.rs`
[^25]: The unit type module. `crates/cachette-core/src/unit_type.rs`
[^26]: The campaign module. `crates/cachette-core/src/campaign.rs`
[^27]: ADR-0180, a site changes hands or the taker destroys it. `docs/adrs/draft/adr-0180-a-site-changes-hands-or-the-taker-destroys-it.md`
[^28]: The relation module. `crates/cachette-core/src/relation.rs`
[^29]: ADR-0146, a faction relation is one signed integer per ordered pair, and a pass reads a threshold. `docs/adrs/accepted/adr-0146-a-faction-relation-is-one-signed-integer-per-ordered-pair-and-a-pass-reads-a-threshold.md`
[^30]: The fire module. `crates/cachette-core/src/fire.rs`
[^31]: The block layout. `crates/cachette-core/src/bridge.rs`
[^32]: The policy module. `python/cachette/learn/policy.py`
[^33]: Findings register, FND-569. `docs/FINDINGS.md`
[^34]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
[^35]: Blockers register, BLK-050. `docs/BLOCKERS.md`
[^36]: ADR-0181, a faction that holds no site and no unit leaves the game. `docs/adrs/draft/adr-0181-a-faction-that-holds-no-site-and-no-unit-leaves-the-game.md`
