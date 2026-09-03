# What a Unit Does in a Tick

This document reads the engine and reports what one unit does in one tick,
end to end. It names the verbs in order. It says where the chain of cause
breaks, how large each break is, and which repair makes the rest visible.

It is an analysis note, not a record. It holds counts and file names, which a
decision record must not hold.[^1] It sits with the other analysis notes
rather than with the record reviews, because it reviews the engine and not a
record.[^2]

Nothing here was run. The claims come from reading the source. Each claim that
only a run can settle is marked.

## 1. The finding

**The engine computes a decision for every unit on every tick, and then
throws the decision away.**

The choice pass scores four options for a unit and writes the winner into a
column. The movement pass reads that column, tests whether it holds a value,
discards the value, and draws a uniform direction from the keyed
generator.[^3] A unit that decides to forage therefore walks the same way as a
unit that decides to climb.

Everything upstream of that column is correct and costs a full pass. The level
1 rebuild, the cell summary, the need column, the option weights and the
stagger schedule all feed one bit: whether the unit moves at all.

Two more chains end the same way.

**The field that the viewer paints is noise.** The tile value pass draws a
number for every tile on every tick and adds one, minus one, or nothing to the
tile.[^4] The value is a random walk. It is the field the viewer reads for the
colour of a tile, the field the cell summary averages, and the field the
`forage` option scores.[^5] [^6] No other system reads it and no other system
writes it.

**The resources are meaningful and nothing can see them.** Every tile carries
a generated stock of food, wood and stone, and the founding survey reads that
stock to choose a place and to set the production rate of the new site.[^7]
The cell summary carries no resource field, so a unit cannot see food. The
viewer draws no resource, so a watcher cannot see food either.

## 2. What a unit does in a tick, verb by verb

The step runs these stages in this order.[^8]

1. **The tile value pass.** Each thread walks a range of tile indices and
   draws a delta for each tile. This is the noise field of section 1.
2. **The bridge refresh.** The unit-to-tile structure rebuilds over the
   spawns and despawns that happened between two frames.
3. **The choice pass.** A unit whose level 1 cell chooses on this frame scores
   four options against the summary of that cell and writes the winner.
4. **The movement intent pass.** A unit that holds any winner draws a uniform
   direction. A neighbour outside the world refuses the step. Ground that
   admits no unit refuses the step.
5. **The admission pass.** Two passes admit the intents against the capacity
   of each target tile.
6. **The bridge refresh.** The structure rebuilds over the movement.
7. **The deposit recovery.** The depleted set ages, and each entry gives a
   part of its take back to its tile.
8. **The gather resolve.** Each unit that holds a gather order takes from the
   tile it stands on.
9. **The holding spread.** Each unit stamps its faction onto the ground near
   it, and the tile holder column changes.
10. **The rate pass.** Each site adds its production and subtracts its upkeep,
    on a schedule.
11. **The consumption pass.** Each unit draws its ration from the store of its
    home site. A unit that draws nothing gains a deficit.
12. **The death scan.** A unit whose deficit reaches the bound is removed.
13. **The bridge refresh.** The structure rebuilds over the removals.
14. **The level 1 rebuild.** Every cell summary is built again from level 0.
15. **The influence solve.** Every faction plane runs a fixed number of
    relaxation passes.

Stages 7 and 8 do no work in the demonstration, because no unit ever holds a
gather order. Stage 15 does full work and produces nothing, because no source
term is ever set.

The live chain of cause is therefore short.

> The survey reads the ground. The founding sets a production rate from it.
> The rate fills a store. The store feeds the units of that site. A unit that
> is not fed gains a deficit and dies. The death removes a unit from the cell
> summary, which changes what the units of that cell choose, which changes
> whether they move.

That chain works. It is the only closed loop in the engine, and it closed six
commits ago.[^9] A watcher cannot see it, because the viewer shows no store,
no ration and no rate.

## 3. Where the chain breaks

Each break below is stated with its size. A size is the shape of the change,
not a measurement.

### 3.1 The option does not steer the step

**The break.** The movement pass reads the presence of the intent and not its
value.[^3]

**The size.** One pass. The movement pass must read the option and turn it
into a direction. Section 5 gives the shape.

**What it costs today.** Every statement of the product record that this
project points at fails at the action, although each one passes at the
choice.[^10] A watcher who changes the world sees the intent column change and
sees no unit behave differently.

### 3.2 Nothing in the world tells a unit where the food is

**The break.** The cell summary holds six fields: the tile count, the open
tile count, the unit count, the held tile count, the value total and the
height total.[^5] None of them is a resource, a store, a settlement or a
need. The `forage` option scores the mean of the noise field.

**The size.** One accumulator on the summary, one argument on the rebuild, and
one field on the option table. The rebuild already walks the tiles of each
cell, so the pass count does not change.

**What it costs today.** A unit cannot prefer good ground, because no field of
the summary says which ground is good.

### 3.3 A unit is faction-blind

**The break.** The held tile count says how many tiles a faction holds, and it
never says which faction.[^11] The unit count is the same. Both fields are
faction-blind by decision, because a summary indexed by the faction would
multiply the world by the faction count.

**The size.** No change to the summary. The influence field is the route that
the project already chose: it is one plane for each faction over the same
level 1 lattice that the choice pass already reads, and its own module says a
consumer reads the cell it already reads.[^12]

**What it costs today.** One statement of the product record cannot be met at
all from the summary: a unit of another faction nearby must change what a unit
does.[^10] Two backlog items hold this work and both are open.[^13] [^14]

### 3.4 The influence field has no source and no reader

**The break.** The only verb that raises the field is called by three tests
and by nothing else. No file outside the core crate reads the field.

**The size.** A settlement or a ruler writes a source term. One backlog item
holds it.[^14]

**What it costs today.** The solve runs its full pass count over every plane
on every tick, on an all-zero field. Another item already asks for the
cadence.[^15]

### 3.5 The resource loop has no sink

**The break.** A unit gathers into a carry column. **No verb anywhere moves a
carry load into the store of a site.** A load leaves the world when the unit
dies, and the world counts it as departed so that the ledger balances.[^16]

**The size.** One verb and one stage. A unit that stands on its home site and
holds a load gives the load to the store.

**What it costs today.** Gathering cannot feed anybody. The store rises only
by the fixed rate that the founding set from the survey, so the economy is a
constant and the ground the units stand on does not change it.

### 3.6 No unit ever orders a gather

**The break.** The gather order is a control-plane verb. The engine issues
none. The `forage` option is named for an act that never happens.

**The size.** One rule in the step: a unit whose intent is `forage`, and whose
tile holds the resource, takes it. The testing rule already names this shape:
when the engine is obligated to invoke a thing, the test starts at the
engine.[^17]

**What it costs today.** The resource module, the ledger, the depletion set
and the recovery pass are all correct and all idle.

### 3.7 A unit is not a person

**The break.** The character arena holds no tile and none of the unit
columns, and it says so.[^18] Nothing links a unit to a character. Descent,
renown and a ruler therefore reach no unit.

**The size.** Large, and outside this analysis. The promotion of a unit into
the character tier is a separate open item.[^19]

**What it costs today.** Descent and renown are complete and reach nothing
that moves.

### 3.8 The engine can explain a choice and nobody can ask

**The break.** The engine holds a verb that reports every score, the value
each option read, and the winner. No file outside the core crate calls
it.[^20]

**The size.** One panel row in the viewer.

**What it costs today.** The product record asks that a watcher can ask why a
unit did what it did, and get an answer from the engine. The answer exists and
the question cannot be put.[^10]

## 4. Which couplings exist

These couplings are live in the demonstration today.

| From | To | Through |
|---|---|---|
| Terrain | Movement | Passability and capacity |
| Survey | Site production | The founding |
| Site store | Unit need | The consumption pass |
| Unit need | Choice | The drive of an option |
| Level 1 cell | Choice | The cell summary |
| Units | Tile holder | The holding spread |
| Tile holder | Level 1 cell | The held tile count |
| Tile holder | The viewer | The border paint |
| Unit need | The viewer | The unit condition |

These couplings are missing.

| From | To | Section |
|---|---|---|
| Choice | Movement direction | 3.1 |
| Resources | Level 1 cell | 3.2 |
| Influence | Choice | 3.3 |
| Settlement or ruler | Influence | 3.4 |
| Carried load | Site store | 3.5 |
| Choice | Gather order | 3.6 |
| Unit | Character | 3.7 |
| Choice explanation | The viewer | 3.8 |
| Resources | The viewer | 3.2 |

**Two of them carry the rest.** The first is 3.1, because every other coupling
that reaches the choice pass reaches nothing further until the option steers
the step. The second is 3.2, because the choice pass scores noise until the
summary carries a quantity another system writes.

## 5. The smallest slice that produces a living world

The slice is ordered. Each step is visible on its own, and each step needs the
step before it.

**Step 1. The cell summary carries the food of its cell.** Add one 64-bit
accumulator, in the way the pyramid record requires of a field summed over the
target tile count.[^21] The rebuild takes the resource field as one more
argument and reads the stock of each tile as it walks the cell.

**Step 2. The `forage` option scores food, not noise.** Change the field of
one row of the option table, and add the intensive accessor beside the others.
A watcher can now check the choice against the ground: the explanation reports
a food value, and the deposit under the unit holds that food.

**Step 3. The option steers the step.** After the level 1 rebuild, derive one
exit direction for each cell and each option. The movement pass reads the
intent of the unit and the exit direction of its cell, and takes that
direction. A cell that ranks highest against its own neighbours has no exit
direction, and a unit there keeps the uniform draw it takes today.

This shape is the one the project already prefers. A set-valued command
permits a cheaper algorithm, and a flow field over the cells costs the cell
count rather than the unit count.[^22] The alternative, where each unit scores
its six neighbours, costs the population and gives the same answer for every
unit of one cell.

**This step needs a decision record, and this document reserves no number for
it.** A contributor would reasonably make each unit score its own neighbours.
The reason the cheap method gives the same answer is that the option value is
a property of the cell and not of the unit, and that reasoning is invisible in
the loop.

Step 3 turns the random walk into a migration. A watcher sees crowds leave
poor ground and stream toward good ground. This is the smallest change that
answers the project owner.

**Step 4. A unit that forages takes what it stands on.** The engine issues the
gather order for a unit whose intent is `forage` and whose tile holds food.
The gather resolve, the ledger, the depletion set and the recovery pass all
start to run. Food falls where the crowd stands, the cell summary falls with
it, and the flow field of step 3 turns the crowd away. That is the negative
feedback that stops the migration from being one rush in one direction.

**Step 5. A load reaches a store.** A unit that stands on its home site gives
its carry load to the store of that site. The economy then depends on what the
people fetched, and not only on the rate that the founding set.

**Step 6. The viewer shows the food and the reason.** Paint the food stock of
a tile instead of the noise, and add a panel row that reports the choice
explanation for one unit. A watcher then sees a deposit drain, sees it recover,
and can ask why a unit walked where it walked.

**If only three steps are taken, take 1, 2 and 3.** They convert every pass
that already runs into behaviour that a watcher can see, and they add no new
subsystem.

The slice deliberately leaves the influence field, the characters, the
descent, the households and the positions alone. Other work holds each of
them.

## 6. What stops this recurring

### 6.1 The obvious rule does not catch the case

A clause that says a feature does not merge unless something a person can run
reaches it would have passed the choice pass. The demonstration runs the
choice pass on every tick, on every unit. Nothing about it is unreachable.

A check that derives the callers from the tree and reports a public verb that
nothing calls would also have passed it. The choice pass has a caller. So does
the intent column: the movement pass reads it.

A rule that makes a backlog item name its caller before it is refined would
also have passed it. The item named the caller, and the caller is the right
one.[^23]

**All three candidates look for an absent caller. The defect is a present
caller that discards the payload.** The engine writes a value into state, and
no decision anywhere reads that value. The rules the project already holds
look for inert code, and this is inert data.[^24]

### 6.2 The rule that does catch it

The testing rule already states the discipline in one direction. For a keyed
draw, it says to test what the value depends on: change a field of the key,
and the draw must change.[^25] The repair is the same discipline in the other
direction.

**For each value that the work writes into state, name the stage that reads it
to decide something, and write a test that changes the value and asserts that
the decision changes.**

The falsification is the one this project already trusts. Pin the value to a
constant and run the suite. A suite that stays green proves that nothing reads
it.[^26] Applied to the choice pass, that test fails today: pin the intent
column to one option, and every test still passes, because no test asserts a
consequence of which option a unit chose.

The cost is one line in the impact review and one test for each new column.
The catch is exact, and the gate is one that can go green.

### 6.3 What to reject, and why

**Reject the reachability check as a gate.** It needs a baseline, because some
public verbs are legitimately inert: a binding exists for a control plane that
nobody has written yet, and a reader exists for a test. The project already
carries one baseline of this shape, and it records what a baseline costs: it
can only shrink, and it does not shrink by itself.[^27] The project has also
already stated the rule that governs the case. A gate nobody can turn green is
a gate everybody learns to skip.[^28]

**Do not file inertness one item at a time.** Four rows of the priority index
name a capability with no caller, and every one of them sits under `Later`,
where nothing blocks and nothing moves.[^29] A row for each instance is not a
structural change. It is a list of the instances.

## 7. What this document did not verify

- Nothing here was compiled or run. Another worker holds the machine.
- The visible effect of each slice step is reasoned, not observed. Whether a
  migration reads as a migration on the screen can only be settled by running
  the demonstration.
- The cost of every step is stated as a shape. No cost figure appears, because
  one blocker holds every cost figure a step would state.[^30]
- The counts in this document are counts of the tree on this branch. Another
  worker may have changed any of them. Five workers hold the influence field,
  the households, the descent, the positions and the world construction while
  this document is written.

## References

[^1]: Decision Record Scope, section 4.3. `.claude/rules/adr-scope.md`
[^2]: Movement timing note. `docs/research/movement-timing.md`
[^3]: The movement intent pass. `crates/cachette-core/src/world.rs`
[^4]: The tile value pass. `crates/cachette-core/src/world.rs`
[^5]: The cell summary. `crates/cachette-core/src/pyramid.rs`
[^6]: The option table. `crates/cachette-core/src/choose.rs`
[^7]: The founding provisions a site from the survey. `crates/cachette-core/src/world.rs`
[^8]: The step. `crates/cachette-core/src/world.rs`
[^9]: Commit 910dec0, feed a founded group from the ground its survey measured.
[^10]: PRD-0009, a unit acts on the world it can see. `docs/product/accepted/prd-0009-a-unit-acts-on-the-world-it-can-see.md`
[^11]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D3. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
[^12]: Decisions register, DEC-040. `docs/DECISIONS.md`
[^13]: Backlog item 0068, give a faction a ruler and a succession. `docs/backlog/refined/0068-give-a-faction-a-ruler-and-a-succession.md`
[^14]: Backlog item 0104, carry the writ of a ruler in the influence field. `docs/backlog/refined/0104-carry-the-writ-of-a-ruler-in-the-influence-field.md`
[^15]: Backlog item 0169, choose the cadence of the influence solve. `docs/backlog/proposed/0169-choose-the-cadence-of-the-influence-solve.md`
[^16]: The carry ledger invariant. `crates/cachette-core/src/world.rs`
[^17]: Testing Rules, section 5. `.claude/rules/testing.md`
[^18]: The character arena. `crates/cachette-core/src/character.rs`
[^19]: Backlog item 0088, promotion into the character tier. `docs/backlog/PRIORITY.md`
[^20]: The choice explanation. `crates/cachette-core/src/world.rs`
[^21]: ADR-0023, an aggregate combines exactly, in any order, decision D3. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
[^22]: Project orientation, the design principles. `CLAUDE.md`
[^23]: Backlog item 0064, choose an action by scoring a fixed option set. `docs/backlog/complete/0064-choose-an-action-by-scoring-a-fixed-option-set.md`
[^24]: Recurring defect shapes, shape 3. `.claude/rules/recurring-defects.md`
[^25]: Testing Rules, section 2. `.claude/rules/testing.md`
[^26]: Testing Rules, section 2a. `.claude/rules/testing.md`
[^27]: Findings register, FND-130. `docs/FINDINGS.md`
[^28]: The footnote check script. `scripts/check_footnotes.py`
[^29]: Backlog priority index. `docs/backlog/PRIORITY.md`
[^30]: Blockers register, BLK-007. `docs/BLOCKERS.md`
