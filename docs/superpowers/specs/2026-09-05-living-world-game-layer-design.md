# Design: The Living World Game Layer

Date: 2026-09-05
Status: Draft for review

## The problem

The Python demonstration steps a world and draws it. Nothing in that world
wants anything. A unit gathers where the choice pass sends it. No faction
plans, no faction fights for a reason, and no run ends. A watcher sees motion
and sees no game.

This design adds a game layer inside the engine. Factions make choices. The
choices drive toward four win conditions. The world pushes back with weather,
wear and war. Python watches and reports. Python drives nothing in this work.

This document is a design specification. It holds structure. It holds no tuned
figure. Every threshold, rate, limit and target is a value in a reference
table. Two tables hold them. The budgets table holds every cost shape and every
scale constant.[^1] A new balance table holds every game value.[^2] Two open
blockers govern parts of this design, and each section that they touch cites
them.[^3] [^4]

## The decisions already taken

The project owner took these decisions before this document was written. This
document does not reopen them.

- Controllers live in Rust, inside the step, as one system at a fixed stage.
- Four win paths exist: domination, territory at a tick limit, wealth or
  wonder, and renown.
- Diplomacy is a graded relation. One signed integer stands for each ordered
  faction pair. Threshold bands name alliance, peace, tension and war.
- Weather may harm upgrade condition, stores, production, units at full
  strength, and movement speed.
- No player control exists in this work. A per-faction flag says "externally
  controlled". Nothing sets it.
- Balance is checked four ways over a fixed seed set.
- One controller exists. A seeded weight vector per faction biases it. The
  vector holds four weights: war, trade, build, renown. No named archetype
  exists.
- The upgrade set keeps road and terrace and adds wall, wonder and store.
- Unit types are capability columns.
- A trade route requires a unit that carries.
- The work proceeds as a tracer bullet widened in passes. Every pass leaves a
  demonstration that runs, ends, and is measured.

## 1 The controller stage

### 1.1 Where it runs

One system runs at one fixed stage of the step, in the core crate. The step
today opens its stages in one fixed order, from the tile scan to the presence
fold.[^5] The controller stage opens directly after the presence fold. At that
point every derived structure of the frame describes the frame, so the
controller reads a settled world. The commands it emits take effect in the
next frame, through the same verbs a Python caller uses between steps.

The stage is a new variant of the stage enumeration. The step register gains
one row for it. The stage takes no thread count. Its cost follows the faction
count and the seat count, never the population.

### 1.2 What it reads

The controller reads a fixed set of aggregates that the engine already
exposes, or that this design adds. It starts no pass over the world.

| Reading | Exists today | Section that adds it |
|---|---|---|
| Faction population | Yes | — |
| Region summary around each seat | Yes | — |
| Window census around each seat | Yes | — |
| Site economy of each own site | Yes | — |
| Trade book of the faction | Yes | — |
| Weather totals | Yes | — |
| Relation row of the faction | No | 3 |
| Market board of each other faction | No | 4 |
| Score of each faction | No | 5 |

A seat is the tile a faction founded its first settlement on. The founding
report already names it.

### 1.3 What it emits

The controller emits commands only through verbs. The existing verbs are
`order_gather`, `order_build`, `set_unit_type`, `send_units_to`, the six trade
verbs, and `inflict_weather`. This design adds `move_relation`, `advertise`,
and the widened trade terms.

**No verb exists for the controller alone.** Whatever the controller can call,
Python can call. A verb that only the controller reached would be a capability
that no caller invokes, and the defect rules name that shape.[^6]

The core crate today exposes the gather, build and type verbs for one entity.
The Python binding loops over a set in Rust and calls the one-entity form.
The controller needs the set form inside the core crate. This design moves the
loop into the core crate. The binding then calls the same set form, so one loop
serves both callers.

### 1.4 How it chooses

The controller visits the factions in identifier order. For each faction it
makes a fixed number of evaluations. Each evaluation draws once from the keyed
generator. The key is the tuple (controller system, tick, faction, draw).[^7]
The faction weight vector biases the draw. A high war weight makes a campaign
more likely, and a high build weight makes an upgrade order more likely.

The evaluation count per faction per tick is a value in the balance table.
There is no convergence test and no time budget.

A faction whose externally-controlled flag is set receives no evaluation. The
flag is one `u8` per faction in simulated state. It is off by default. Nothing
in this work sets it. It exists so that a later player hook has a place to
stand, and so that the balance harness can prove the hook is inert.

### 1.5 How commands apply

Each evaluation that produces a command pushes it to a list. The list is sorted
by (faction, sequence) before any command applies. The sequence is the draw
index. The sort makes the order explicit and stable, so the result never
depends on the visit order.[^8]

Each command applies through the same code path a Python caller takes. A
command the verb refuses is dropped, and the refusal counts in the subsystem
census.

### 1.6 Cost

The cost shape is: evaluations per faction, multiplied by factions, multiplied
by the bounded cost of one reading. No term follows the unit count or the tile
count. The figure belongs in the budgets table, and it stays derived until the
target platform measures it.[^1] [^3]

## 2 Unit types as capability columns

### 2.1 The row

A unit type is an index into a shared table today. A row holds two values,
attack and armour, in the fixed-point scale. No pass branches on a type
name.[^9] This design widens the row. Every column is numeric. Zero means
"cannot".

| Column | Meaning |
|---|---|
| attack | As today |
| armour | As today |
| gather rate | Scales what the unit takes from a tile in one tick |
| build rate | Scales the work the unit adds to an upgrade; also the repair rate |
| carry capacity | What the unit carries under a contract; zero means it never carries |
| move cost scale | Scales the movement cost the unit pays on a tile |
| command reach | Nonzero means the unit may move a relation and may lead a campaign |
| weather reach | Nonzero means the faction may inflict weather while it holds this unit |

Every column is an integer or a Q16.16 value.[^10] No pass reads a type name.
A pass reads a column of the row the unit indexes.

### 2.2 The default table

One Rust constant holds the default table. It holds five rows: worker,
soldier, merchant, leader, and one open row. The seeding layer instantiates the
table from the constant. The values of each row are in the balance table.

The panel labels and the generated Python reference derive from the same
constant. A check fails when a label list and the constant disagree, because a
second declaration site rots when nothing fails.[^11]

The Python verb `define_unit_type` takes the full row. The core function takes
the full row too. The two-value form is removed. Every caller of the old form
moves in the same commit, and the whole-tree search goes in the commit body.

### 2.3 A trade route

A trade route is a contract plus carriers. A contract moves a quantity only
when a unit carries it onto the ground of the other party.[^12] The delivery
pass exists today and runs directly after the contest stage. It is not a stage
of its own.

To open a route the caller gives units with carry capacity above zero a home
at one site, then sends them to the other. The delivery pass does the rest. No
new movement machinery exists.

### 2.4 An army

Raising an army is two verbs. `set_unit_type` moves a cohort to the soldier
row. `send_units_to` sends the cohort to a destination field. Nothing else is
needed.

## 3 Diplomacy as a graded relation

### 3.1 The matrix

A dense matrix of signed integers covers the ordered faction pairs. The entry
for the pair (A, B) is what A feels toward B. The matrix is simulated state
and enters the state hash. Its size follows the faction ceiling, never the
population.

Four bands cover the integer range: alliance, peace, tension, war. The band
edges are values in the balance table. No band name appears in code. A pass
compares the integer to an edge that it reads from the table.

### 3.2 What reads the relation

- **Contest.** The contest pass resolves a meeting between two factions and
  fires wherever two factions are adjacent.[^13] This design gates it. Two
  factions fight only when at least one of the pair is in the war band toward
  the other. This changes the contest from always-on to gated.
- **Conversion.** A unit converts only when the leading faction is in a stated
  band toward the faction of the unit. The permitted bands are in the balance
  table.
- **Trade.** The engine refuses an offer when either side is in the war band
  toward the other.
- **Movement.** A unit may not enter ground another faction holds when the
  holder is below a stated band toward the guest. The band is in the balance
  table. Nothing refuses this today.

### 3.3 What moves the relation

Each move is an integer step. Every step value is in the balance table.

| Cause | Direction |
|---|---|
| A contract delivers in full | Up |
| A contract fails | Down |
| A unit of one side falls to the other | Down |
| A unit converts away | Down |
| A storm falls on ground the other holds | Down |
| Drift, one step toward peace on a fixed schedule | Toward the peace band |

The drift schedule is a period and a phase, in the shape the economy and
position schedules already use.

### 3.4 The verb

`move_relation(speaker, other, step)` moves the entry for (speaker, other) by
the step. The verb refuses when the speaker faction holds no unit whose type
has command reach above zero. The step is bounded by a value in the balance
table.

### 3.5 The event

A crossing of the war edge, in either direction, writes one event. The event
is plain data with `repr(C)`, declared padding, and no `bool`.[^14] It holds
the tick, the two factions, and the direction. The panel and the demonstration
read it and announce a declaration or a peace.

### 3.6 A treaty

A treaty is a trade whose consideration is a relation move. Section 4 defines
the tagged consideration.

## 4 The trade board and land as a good

### 4.1 The board

Each faction holds one small fixed-size table of advertisements. A row holds
(good, quantity, offers-or-wants, asking good, asking quantity). The row count
is a value in the balance table. The board is simulated state, enters the
hash, and is bounded by the row count times the faction ceiling.

The controller writes its rows from its site economies on a schedule. Python
writes rows through a verb `advertise(faction, row)`. A reader
`market(faction)` returns the board of any faction. Reading has no cost to
standing and moves no relation.

### 4.2 The tagged consideration

Each side of a contract becomes a tagged consideration. The tag is one of
three kinds.

| Kind | Content | How it delivers |
|---|---|---|
| Resource | A resource kind and a quantity | As today; carriers move it |
| Land | A bounded set of tiles the offerer holds | The holder changes on full delivery of the other side; no carrier |
| Relation | A step on the pair | The step applies on full delivery of the other side |

The land set is one level 1 cell, or a bounded list of tiles. The list bound
is a value in the balance table. The engine refuses a land offer when the
offerer does not hold every tile in the set.

The status machine is unchanged. Offer, counter, accept, refuse, close and
reopen keep their meanings and their code paths.

### 4.3 The open question about upgrades on traded ground

Whether an upgrade on a tile goes with the tile when the holder changes is a
question the project owner holds, and it is open.[^4] This design does not
answer it. The land kind is written parametrically. Until the blocker closes,
the engine refuses a land offer whose tiles carry an upgrade. When the blocker
closes, one commit removes the refusal and applies the answer. That commit
searches the tree for the blocker number and repairs every record that calls
it open.

### 4.4 Controller pricing

The controller offers only where its own surplus meets a posted want on
another board. Surplus is the site store above a mark in the balance table. It
counters at the integer midpoint between the two asks. It accepts when the
counter meets its own ask. It never draws for a price.

## 5 Win conditions and game end

### 5.1 The four readers

| Path | Condition |
|---|---|
| Domination | One faction holds every seat, or every other faction has no units |
| Territory | At the tick limit, the faction with the most held tiles |
| Wealth or wonder | A faction stock total reaches a target, or a wonder upgrade completes |
| Renown | A character of the faction reaches a renown target |

The tick limit and every target are values in the balance table. The held
tile count is a running total the engine already keeps.[^15] The stock total
sums the stores of the own sites in a 64-bit accumulator. The renown reader
reads the character column that exists today.

### 5.2 The game end

A game end record holds (winner, path, tick). The step writes it once, at the
first tick a reader fires. The controller stage checks the readers directly
before it evaluates, in the order of the table above. A tie on the territory
path resolves by the lowest faction identifier.

After the game end the world keeps stepping. The controller stage emits
nothing. Weather, wear, gathering and every other pass continue, so a watcher
can keep watching.

Two Python readers exist: `score(faction)` returns the four running values for
one faction, and `game_end()` returns the record or nothing.

## 6 Upgrade condition, wear and repair

### 6.1 Condition

Each upgrade entry gains an integer condition. Completion sets it to full. The
full value per kind is in the balance table. Condition is simulated state.

### 6.2 Wear

Two sources wear an upgrade. Weather wears an upgrade on a cell whose ground
water is above the wet mark (section 7). A hostile unit standing on the tile
wears it, so an army damages what it stands on. A hostile unit is one whose
faction is in the war band toward the holder. Each source takes one integer
step per tick, and the steps are in the balance table.

Condition zero means the upgrade is gone. The engine removes it through the
existing destroy path, so one code path removes an upgrade whatever the cause.

### 6.3 Repair

Repair is `order_build` on a tile that already carries an upgrade. The build
pass adds the build rate of the unit type to the condition, clamped at full.
No new verb exists.

### 6.4 The new kinds

The upgrade kind is an enumeration with a per-kind work table. Three variants
join it.

| Kind | Effect |
|---|---|
| Wall | Raises the movement cost for a unit whose faction does not hold the tile; absorbs contest harm on its tile before any unit falls |
| Wonder | Large work; completion fires the wealth-or-wonder win path |
| Store | Raises the store capacity of the site on its tile; a flood spoils it |

The work of each kind and the wall absorption are values in the balance
table. The map paints the condition as a tint (section 9).

## 7 Weather harm

The weather field holds air water and ground water on the level 1 cell
lattice.[^16] The engine exposes the wet mark and the wet-cell test today.
This design adds four harms. Every value is in the balance table.

| Harm | Rule |
|---|---|
| Upgrade wear | One step per tick on a cell above the wet mark |
| Store spoilage | An integer share of the store of a site on a flooded cell, per tick |
| Production halt | The rate pass skips a site on a flooded cell |
| Unit loss | One bounded keyed draw per flooded cell names units at full strength that fall |
| Movement | The movement cost on wet ground rises by a step |

A flooded cell is a cell whose ground water is above a second mark, higher
than the wet mark. The unit loss draw is keyed on (weather harm system, tick,
cell, draw), and it names units by a keyed rotation in the shape the contest
uses.[^13]

The verb `inflict_weather` takes a faction, places and a strength, and it
refuses ground the faction does not hold.[^17] This design adds one refusal.
The verb refuses when the faction holds no unit whose type has weather reach
above zero. The controller calls the same verb.

## 8 Campaigns

A campaign is (faction, objective, cohort). Each faction holds a small bounded
register of campaigns. The register size is in the balance table. The register
is simulated state.

Three objective kinds exist: take a site, wear an upgrade, relieve an own site.
Raising a campaign is `set_unit_type` to the soldier row and `send_units_to`
the objective tile. The controller chooses an objective from the relation band
and the war weight. It raises no campaign against a faction in the alliance or
peace band. No new movement machinery exists. A campaign closes when its
objective holds or when its cohort is empty.

## 9 The picture

The viewer paints three new layers on the map.

- Upgrades: a glyph or a tint per kind, with the condition as the tint depth.
- Weather: air water as an overlay, and wet ground darker than dry ground.
- Luxuries: one mark per tile that holds a deposit.

Five panels join the deck: weather, relations, market, economy, score. The
viewer holds the deck in one registration, and the Python `panel_names` reader
derives from it. The new panels appear with no Python edit. A panel reads a
bounded number of addresses and starts no pass over the world.[^18]

## 10 The census and the balance harness

### 10.1 The subsystem census

An engine reader `subsystem_census()` returns one count per subsystem. The
subsystem list derives from one Rust table. Each row names the subsystem and
the reader that counts it. A list written by hand would be a second
declaration site.[^11]

This resolves the open questions of the backlog item on what the
demonstration never produced.[^19] The census reports counts, not zeros. It
lives in the engine. A gate test drives the demonstration world for a tick
count in the balance table and asserts every count is nonzero. The
demonstration prints the census at its end.

### 10.2 The balance harness

A recipe `just balance` runs a fixed seed set to game end. It checks four
statements against thresholds in the balance table.

1. No win path wins more than its share of the seeds.
2. No seat wins more than its share of the seeds.
3. Every game ends before the tick limit in more than a stated share of the
   seeds.
4. Every subsystem count is nonzero in every seed.

The harness is long, so it is not a merge gate. It runs in the slow test
recipe, on the schedule that recipe runs, and before any commit that changes
a balance value. Its output names the seed set and every failing seed.

## 11 The demonstration

The demonstration adds no verb. It builds the world from the seed and steps
it. The seeding layer runs inside the engine at construction, so the
demonstration calls no seeding verb.

It prints game events as they happen: a declaration, a treaty, a signed
contract, a storm, wonder progress, a repair, and the game end. Each line comes
from an event log or a reader. It prints the subsystem census at its end.
Under a flag it ends at the game end and names the winner and the path. The
function keys follow the panel deck as they do today.

## 12 Determinism, across everything

Every pass in this design satisfies each line.

- No floating point number in simulated or aggregated state.[^10]
- Every arithmetic step goes through the arithmetic module.
- Every random draw is keyed on (system, tick, entity, draw).[^7]
- Every parallel result is sorted by a stable key.[^8]
- Every solver and every controller runs a fixed evaluation count.
- Every event type is plain data with `repr(C)`, declared padding, and no
  `bool`.[^14]
- Every accumulator at level 1 is 64 bits wide.
- The thread-count test and the golden state hash test pass at every
  pass.[^20]

## 13 The passes

Every pass leaves a demonstration that runs, ends, and is measured.

| Pass | Content | Touches `fn step` |
|---|---|---|
| 1 | Seeding layer; controller stage with gather and build only; territory win; subsystem census; demonstration end | Yes |
| 2 | Unit type columns and the default table | No |
| 3 | Relation matrix; war gating of contest, conversion, trade and movement; `move_relation`; declaration event | Yes |
| 4 | Upgrade condition; wear by armies; repair; wall | Yes |
| 5 | Weather harm; weather-reach gate | Yes |
| 6 | Trade board; tagged considerations; land; treaties; carriers as routes | No |
| 7 | Campaigns | No |
| 8 | Wonder; store; remaining win paths; renown | No |
| 9 | Map layers and five panels | No |
| 10 | Balance harness | No |

Passes 1, 3, 4 and 5 touch `fn step`. Only one worker at a time may hold a
pass that touches it, so those four run in sequence. Pass 9 runs in parallel
with passes 3 to 8, one panel as each subsystem lands. Passes 6, 7 and 8 read
what earlier passes wrote and may run in parallel with each other once pass 5
lands.

## 14 The records this work needs

The scope rule gives a three-condition test.[^21] A decision needs a record
when a contributor could choose otherwise, when choosing otherwise costs more
than changing it later, and when the reasoning is not visible in the code.

### 14.1 Product records

One record per audience need. Each states a need and no structure.

- A developer watches factions play a game to an end.
- A god trades land.
- A god advertises what it will trade.
- A god declares war and makes peace.
- An upgrade wears and a worker repairs it.
- A game is balanced across seeds.

### 14.2 Decision records that pass the test

- A faction controller runs inside the step and acts only through the
  caller's verbs. A contributor could put it in Python or give it private
  verbs. Both break the control plane rule silently.
- A unit type is a row of capability columns, and no pass reads a type name.
  A contributor could add a branch on a name. The code does not show why that
  is refused.
- A faction relation is one signed integer per ordered pair, and bands are
  thresholds in a table. A contributor could add named states. The hash and
  the gating depend on the integer shape.
- A contract consideration is a tagged kind. A contributor could add a second
  contract table for land. The delivery pass depends on one table.
- A game end is recorded once and stops the controllers. A contributor could
  stop the step instead. The reasoning is not in the code.

### 14.3 Decision records that fail the test

- "An upgrade holds a condition and zero destroys it." Only one workable shape
  exists once wear exists, and the destroy path is visible in the code. The
  code is the record.
- "The subsystem census is derived from one table." The defect rule already
  states the constraint, and the check that fails on a stale list is the
  reasoning.[^11] A record would restate a rule.

## Corrections to the brief

The code disagreed with the brief at these points, and this document follows
the code.

1. The core crate exposes `order_gather`, `order_build` and `set_unit_type`
   for one entity. The set form exists only in the Python binding. Section 1.3
   moves the loop into the core crate so the controller and the binding share
   it.
2. `inflict_weather` takes a faction, places and a strength. It takes no unit.
   Section 7 gates it on the faction holding a unit with weather reach, and
   keeps the signature.
3. The name `census` is taken three times: the `census.rs` module, the
   `window_census` reader, and the `just census` recipe. Section 10 names the
   new reader `subsystem_census`.
4. No "faction census", "relation row" or "score reader" exists. Section 1.2
   lists the readers that exist and marks the three this design adds.
5. Trade settlement is not a stage. It runs inside the step between the
   contest stage and the rate stage. Section 2.3 says so.
6. The upgrade kind is an enumeration with two variants, not a data table.
   Section 6.4 adds variants and rows of the per-kind work table.
7. Founding and luxury placement are Python verbs today. Section 11 moves
   seeding into the engine at construction, so the demonstration calls no
   seeding verb.
8. The demonstration prints the census at its end, not throughout, because the
   census is a reader and the events are the running report.

## References

[^1]: Budgets and costs. `docs/reference/budgets.md`
[^2]: Balance table, to be created. `docs/reference/balance.md`
[^3]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^4]: Blockers register, BLK-036. `docs/BLOCKERS.md`
[^5]: The step, stage order. `crates/cachette-core/src/world.rs`
[^6]: Recurring Defect Shapes, shape 3. `.agents/rules/recurring-defects.md`
[^7]: ADR-0003, every random draw is keyed, never stateful. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
[^8]: ADR-0004, iteration order is explicit. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^9]: ADR-0120, a unit carries a type that indexes a table. `docs/adrs/draft/adr-0120-a-unit-carries-a-type-that-indexes-a-table.md`
[^10]: ADR-0002, state holds no floating point number. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^11]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
[^12]: ADR-0128, a contract moves a quantity only when a unit carries it. `docs/adrs/draft/adr-0128-a-contract-moves-a-quantity-only-when-a-unit-carries-it.md`
[^13]: ADR-0121, a meeting between two factions resolves at the tile. `docs/adrs/draft/adr-0121-a-meeting-between-two-factions-resolves-at-the-tile.md`
[^14]: ADR-0001, one binary gives one answer at any thread count. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^15]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D4. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
[^16]: ADR-0141, a weather pass moves water and never scales it. `docs/adrs/draft/adr-0141-a-weather-pass-moves-water-and-never-scales-it.md`
[^17]: ADR-0142, a god inflicts weather only on ground it holds. `docs/adrs/draft/adr-0142-a-god-inflicts-weather-only-on-ground-it-holds.md`
[^18]: ADR-0070, the head-up display reports what the drawing pass read. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
[^19]: Backlog item 0278, say what the demonstration world never produced. `docs/backlog/proposed/0278-say-what-the-demonstration-world-never-produced.md`
[^20]: Testing Rules, section 1. `.agents/rules/testing.md`
[^21]: Decision Record Scope, section 1. `.agents/rules/adr-scope.md`
