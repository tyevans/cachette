# What a God Needs From This Engine

Research report 21. It asks what a downstream game needs from the Python
interface of this engine, and what the engine must gain to offer it. Prepared
3 September 2026.

Cachette is a world simulation engine. The core is Rust. The control plane is
Python. A developer who builds on this engine writes Python and never opens the
core.[^1]

The consumer is a real project. It is a game called Gods and Congregations. In
it a god directs a congregation. A person or a language model plays the god.
**A god may send a message to another god only while one of its own units
stands in that god's territory.** The project owner stated that rule and named
the verbs a god will want: move units somewhere, explore, build things, gather
units in a place, attack another god's units, and convert people.

He asked one question. Is this definable largely in Python, and if not, what
must the project enhance?

**The short answer is that the engine models more than it offers.** The write
surface of the bindings is eight callables. The core holds verbs that no
binding calls. The central mechanic of the game is a read that the engine can
already answer, and the answer fits in one word for each faction.

## 0. Provenance, and what this report could not verify

Every claim about the engine in this report is a read of the source tree of
this worktree on 3 September 2026. **No claim here is a measurement taken by
this author.** One measured figure appears, and it belongs to research report
20, which states what was run and against what.[^2]

One blocker governs every cost figure in this project.[^3] No figure in this
report is a target platform figure.

**The rules of the game are one paragraph.** The project owner named seven
verbs in one sentence and stated one rule in full. Section 8 lists every
assumption this report makes about the rest, so that he can correct each one.
This report invents no game rule outside that list.

**This report did not run the engine.** It reads the bindings crate, the core
crate and the Python package. A claim that a call does not exist is a search of
those files, and section 9 gives each search command.

### 0.1 The findings

1. **The write surface is eight callables, not seven.** The eighth is
   `found_group`, which founds a group and hands back a site identity. The
   register holds the corrected count as precedent.

2. **The core holds a general build verb and nothing binds it.** `order_build`,
   `stop_build`, `build_order` and `destroy_upgrade` are public on the world
   type. No line of the bindings crate names one. So the build verb the game
   wants is a binding, and not an engine feature.

3. **Movement to a named place is built, and the control plane cannot name the
   place.** The core answers a return direction for a faction and a cell. The
   seed set is fixed: every live site of the faction. So a god can send its
   people home and can send them nowhere else.

4. **Territory is modelled well and read badly.** One tile carries one holder.
   A block carries a mask of the factions inside it. Each faction carries a
   running total of what it holds. Python reads one tile at a time, and one
   count for a window.

5. **The messaging rule is a relation between two factions, and it fits in one
   word for each faction.** The world holds at most 63 factions. The answer to
   "does any unit of A stand on ground that B holds" is one bit for each
   ordered pair. That is 63 words for the whole world, and it does not grow
   with the population.

6. **Two of the seven verbs need a mechanism the engine does not have.**
   Attack needs a contest. Convert needs a rule that changes who a unit belongs
   to. Everything else is a binding, a seed set, or policy that Python already
   owns.

---

## 1. The verb table

Each row names a verb as a caller would write it. The engine column says what
is true today. The gap column says what is missing.

| The verb | Today | What is missing |
|---|---|---|
| `move(units, to=place)` | **No.** A unit takes its direction from a per-cell field. The control plane sets no destination. | A settable seed set for the field that already exists. |
| `explore(units, area)` | **No.** Nothing is hidden, so nothing can be revealed. | A record of what a faction has observed, and a rule that reveals it. |
| `build(units, kind, at=place)` | **Partly.** The core orders a build and removes an upgrade. No binding calls either. Two kinds exist. | A binding, and a catalogue the game supplies. |
| `gather(units, at=place)` | **Partly.** A founding seats a group at a place the engine chose. Nothing assembles existing units. | A settable seed set, the same one `move` needs. |
| `attack(units, target)` | **No.** No unit carries a strength. No pass resolves a meeting. | A contest, and the state it reads. |
| `convert(units, to=faction)` | **No.** A unit carries a faction and nothing changes it after a spawn. | A rule that changes the column, and a verb that asks for it. |
| `standing_in(territory_of=god)` | **Partly.** The engine holds the answer. Python reads one tile at a time. | A set-valued read, or a derived relation. |

The rest of this section states each row in full.

### 1.1 Move units somewhere

**The engine forbids the obvious mechanism and supplies a better one.** A unit
never searches for a route. A record states that movement takes its direction
from a per-cell field, and never from a per-unit search.[^4] A second record
states that any strategy that names a place arrives as a field over cells.[^5]
A third record builds one such field: the engine derives a reach for each
faction and each cell, seeded at every live site of that faction, and a laden
unit climbs it home.[^6]

**That field is built.** The core answers a return direction for a faction and
an address. So the mechanism the game needs exists and runs.

**The seed set is what is missing.** The record fixes the seeds at the live
sites of the faction.[^7] A god who wants its people at a mountain has no way
to say so, because a mountain is not a site.

The repair is small in shape and it is a decision before it is a change. The
control plane names a set of addresses. The engine seeds a plane at those
addresses and derives the directions. Nothing about the per-unit rule changes,
and no unit gains a search.

**One consequence stays.** A cell moves as a block, so two units in one cell
take one direction.[^8] A god cannot send half a cell one way and half the
other. That is the architecture and it is not a gap.

### 1.2 Explore

**This verb has nothing to act on.** The engine hides nothing. Every tile is
readable by every caller and by every unit. A product record asks that a
faction sees only what its own units observe, and the record is accepted and
unbuilt.[^9]

So explore is not a missing verb. It is a missing model. A god cannot uncover
what is not covered.

**The cost of covering it is stated and not measured.** Fog storage that grows
with observed area is a reserved record number with no file. The backlog holds
two items for it and both sit under `Later`.

### 1.3 Build things

**The core builds and the bindings do not.** The world type orders a build for
one unit and one kind, stops that order, reports it, and removes a finished
upgrade at an address. A search of the bindings crate finds none of the four.

The store is sparse. It holds one entry for each tile that carries an upgrade,
so a world in which nobody built holds no entry.[^10] Several units add to one
progress accumulator in one tick, and the terms combine exactly.

**`found_settlements` is a build of one kind, and it is not this one.** It
creates a settlement entity: a generational identity, a tile, a faction and a
pooled store. It does not mark the ground. The upgrade store and the settlement
arena are different structures and they answer different questions. A god who
wants a temple on a hill wants the upgrade store. A god who wants a village
wants the settlement arena.

**The catalogue is two kinds and they are a Rust enumeration.** A road and a
terrace. A god game wants a shrine, a temple and an altar, and it cannot add
one from Python. Section 3 says whether that is the architecture or a choice.

### 1.4 Gather units in a place

**This is the move verb with a different name.** A god who assembles a
congregation at a place is giving one destination to a set of units. The
mechanism is the seed set of section 1.1, and nothing else is needed.

**The engine has one thing close to it, and it is not this.** A founding
surveys the ground, takes the best place its sample offered, and seats a group
over the disc around that place.[^11] The engine chooses the place. The caller
chooses how many people. And the people are new, not gathered.

**A record already holds the membership shape.** A group is a site membership
and not a region.[^12] That record is a draft. A god who wants a congregation
that is not seated at a site should be read against it before anybody builds
one.

### 1.5 Attack another god's units

**The engine models nothing that a fight reads or writes.** A soldier carries
a generational identity, a tile address, and a faction. It carries no strength,
no health, and no state that a contest could change.

**One thing exists and it is not combat.** The control plane removes a set of
units at once, and the removal is all or nothing.[^13] A god could use it to
delete another god's people. Nothing contests that, nothing costs it, and no
event records why.

**What a contest needs is state before it is a pass.** A contest reads
something about each side and writes something to each side. Neither exists.
This is the row in the table that most clearly needs new Rust, and section 3
says why Python cannot hold it.

**Two properties of this project make the pass harder than it looks.** The
result must not depend on the thread count, so a contest between two units
resolves against a fixed order and never against completion order. And a
solver runs a fixed iteration count, so a fight cannot run until one side is
gone. The project owner and the coordinator supplied a sketch for this verb,
and section 4 tests it against the code.

### 1.6 Convert people

**Half of this is one column write.** A soldier carries a faction. Changing it
changes who the unit belongs to, and every downstream reader already reads that
column: the holding rule, the population count, the return field and the
founding.

**The other half is the whole game and the engine holds none of it.** A
conversion in a god game is not a column write. It is the outcome of belief,
proximity, persuasion and time. The engine models none of those. Section 5
says what the engine must represent before a religion could be built above it.

**So convert splits into two needs.** The first is a verb that moves a set of
units from one faction to another. That is small, and it is a binding over a
column the engine already keeps. The second is a rule that decides when it
happens. That is content, and the project owner has not stated it.

### 1.7 Send a message, gated on presence

**The message is not the engine's business.** Two gods exchanging text is
Python talking to Python. The engine holds no channel and should hold none.

**The gate is the engine's business, and it is a read.** Section 2 designs it.

---

## 2. The territory read

This is the central mechanic of the game. It is a read and not a verb, which
makes it the sharpest test of the set-valued read.

### 2.1 What the engine already holds

**One tile carries one holder, and the holder is a faction or nobody.** The
holder is one dense column over the tiles, so exclusivity is a property of the
storage: one tile holds one value.[^14]

**A faction is one bit in a 64-bit mask, and the world stores one mask for each
block of tiles.** A query that asks where a faction holds reads the masks,
passes over every block that does not name the faction, and walks only the
blocks that do.[^15]

**The count of what a faction holds is a running total.** The rule that changes
a holder adds one to the total of the faction that gained and takes one from
the total of the faction that lost.[^16]

The core offers all three. It answers the holder of an address, the mask near
an address, the count for a faction, the blocks a faction holds, and the tiles
a faction holds.

### 2.2 What Python can do today, and what it costs

**Python reads the holder of one tile at a time.** One call returns a report
for one address, and the holder is one entry of that report.

**Python cannot list the units of a faction at all.** One call returns a
population count for each faction. Nothing enumerates the identities. A caller
who wants the units of its own god must keep every identity that a spawn
returned, in a Python list, for the life of the run.

So the read today is this, and the project forbids it.

```python
# Forbidden. This is the data plane, and it does not scale.
# Nothing in the module refuses it.
present = False
for unit in every_identity_i_kept:
    tile = world.soldier_tile(int(unit))
    report = world.tile_report(tile % world.width, tile // world.width)
    if report["holder"] == other_god:
        present = True
        break
```

**Two crossings for each unit, and a Python object for each one.** Research
report 20 measured one of the two calls at 4.263 microseconds against the
installed module on a development machine.[^2] The other call builds a
dictionary of about a dozen entries. Neither figure is a target platform
figure.

**The loop also carries the population in Python.** That is the failure the
control-plane rule exists to prevent.[^17] A god game with a congregation of
any size cannot pay it, and a god game at this engine's target scale cannot pay
it at any speed.

### 2.3 The read, designed

The question has two forms and they cost different things. Separate them,
because a game asks the cheap one every frame and the expensive one rarely.

**Form one. Is any unit of A standing on ground that B holds?** This is the
gate. It is one bit.

**Form two. Which units of A are standing on ground that B holds?** This is the
answer a god needs when it wants to act on those units. It is a set.

#### Form one: the presence relation

**The answer for the whole world is one word for each faction.** The world
holds at most 63 factions, and the reference table states that the ceiling was
chosen so that a relation is one plane and a presence set is one word.[^18] A
faction is one bit in a mask, and a relation between factions is a plane of
such masks.[^15]

So the whole diplomatic gate of the game is 63 masks. One mask for each
faction, naming every faction that has a unit standing on its ground.

**The derivation costs one column read for each unit, once each frame.** The
engine already rebuilds a bridge from units to tiles at every barrier, and that
rebuild visits every unit and the tile it stands on. A pass that also reads the
holder of that tile and sets one bit costs one more read for each unit, and it
allocates nothing.

**It combines exactly and in any order.** The combine is a bitwise union of
masks. That is associative, commutative, and has zero as its identity, so a
fold over a set of units gives one answer whatever the order and whatever the
thread count.

**Python reads it in one call and gets 63 integers.** No object for each unit.
No object for each tile. The size of the answer does not change when the
population changes.

```python
# One crossing. The whole diplomatic state of the world.
presence = world.presence_masks()      # numpy.uint64, one entry for each faction
may_speak = bool(presence[other_god] & (1 << my_god))
```

**This is what the design principle asks for.** A set-valued command permits a
cheaper algorithm, and the engine should choose an algorithm that uses the
whole set rather than batching a per-entity loop.[^19] The presence relation is
that algorithm. It replaces a loop over the population with a fold whose
result is a fixed size.

#### Form two: the selector

**The set answer is the selector, and the selector does not exist.** An
accepted record specifies it in full: Python builds a lazy expression tree, the
tree crosses once, and Rust evaluates it.[^20] Research report 20 records that
no selector type exists in Python or in Rust.[^21]

Written against that record, the read is three statements and one crossing.

```python
mine = world.units.of_faction(my_god)
inside = mine.where(world.tiles.holder == other_god)

columns = inside.read("unit", "tile")   # one crossing, no Python loop
```

**The cost of form two follows the smaller of two terms, and the engine can
choose.** A walk over the units of A costs the population of A. A walk over the
tiles B holds, using the bridge to name the units on each, costs the territory
of B. The engine holds the running total for both, so it can pick the smaller
one before it starts. A caller can express neither choice and should not have
to.

**The block masks make the second walk cheap.** The query passes over every
block that names neither faction. A god whose territory is one corner of the
world is answered from that corner.

### 2.4 Which of the two to build

**Build form one first.** It answers the stated mechanic completely, it is a
fixed-size read, its derivation rides on a pass that already exists, and it
needs no selector. A god game can ship its diplomacy on it.

**Form two waits for the selector, and it should.** The selector is the large
piece of the Python interface and report 20 already ranks it.[^22] A presence
read built inside it would arrive with it, and the game does not need it to
open a conversation.

---

## 3. The line between data and code

This section answers the project owner's question directly. For each verb it
says whether Python defines it, whether Rust runs it under parameters a game
supplies, or whether it needs Rust that no parameter reaches.

### 3.1 The bargain, stated plainly

**Per-entity logic cannot run in Python, and that is not negotiable.** Three
things force it together. Python must never loop over entities, because the
control plane is not a data plane.[^17] The simulation must release the global
interpreter lock for the whole step, so no Python runs while the world
runs.[^23] And one binary must give one answer at any thread count, which a
Python callback inside a parallel pass cannot promise.

**A small verb vocabulary is not forced by any of that.** Nothing in those
three rules says the engine may offer only eight verbs. The write surface is
small because it was built against what the engine already ran, one call at a
time, and not because a record bounds it. Four of the seven things the game
wants are missing for that reason and for no other.

**Say which is which, because the difference decides what to ask for.** A god
game developer who is told "the architecture forbids it" will design around a
wall that is not there.

### 3.2 The table

| The verb | Its kind | What Python supplies |
|---|---|---|
| Move to a place | Mechanism, parameterised | The seed addresses, and which units hold the option |
| Explore | Mechanism, new Rust | Nothing yet. The model is absent |
| Build | Mechanism, parameterised | The set of units, the kind, and the catalogue |
| Gather at a place | Mechanism, parameterised | The seed addresses. Same mechanism as move |
| Attack | Mechanism, new Rust | Nothing yet. The state is absent |
| Convert | Mechanism, parameterised, once the rule exists | The set, the target faction, and the rate |
| Presence gate | Mechanism, parameterised | The two factions, and nothing else |
| Who, where, when | Policy | All of it |

### 3.3 Policy, and Python owns all of it

**Every choice of which, where and whether is Python's.** Which units act,
which place they act on, which condition holds before a god sends an order, and
what the god says in its message. None of that touches the simulation.

This is the part that is already true and already large. A god is a program
that reads the world, decides, and sends one command. The engine never asks
Python a question mid-frame, and Python never walks the population.

### 3.4 Mechanism parameterised by data

**Move and gather.** The parameter is the seed set: a list of addresses that
seeds the field. The derivation, the relaxation, the tie-break and the per-unit
read are all fixed by records and none of them becomes a parameter. A second
parameter is the pass count, which sets how far the field reaches, and a record
already states that the count is fixed and is the reach.[^24]

**Build.** The parameters are the set of units, the kind, and the catalogue.
The catalogue is the interesting one. Today a kind is a Rust enumeration with
two members, and each member has a fixed work cost and a fixed effect. **The
design principle says a unit type is an index into a shared table and an
upgrade set is an interned identifier.**[^19] A catalogue of kinds, each with a
work cost and a declared effect on the tile, is exactly that shape. Nothing in
the determinism rules refuses it, because a table entry is data and a table
lookup is not a callback.

**Convert, once the rule exists.** The parameters are the set, the target
faction, and whatever rate governs it. The column write itself is trivial. The
rule that decides is content.

**The presence gate.** The only parameters are the two factions. There is
nothing to configure, which is why it is the cheapest of the seven to build.

### 3.5 Mechanism that needs new Rust

**Attack.** A contest reads a property of each side and writes a property of
each side. The engine holds neither property. A parameter cannot supply a
column that does not exist, and a Python function cannot run inside the pass
that would read it. So this is new Rust, and the new part is the state before
it is the arithmetic.

**Explore.** The same argument, one level up. A reveal writes to a record of
what a faction has observed. The engine holds no such record. A parameter
cannot supply it.

**Both are unbuilt rather than forbidden.** Neither contradicts a record. Both
are ordinary engine work with ordinary decisions attached.

### 3.6 What a game genuinely cannot have

One thing on the list is refused by the architecture and not merely unbuilt.

**Two units in one cell cannot be sent different ways.** The direction belongs
to the cell, and the mechanism that would give one unit its own direction is
the per-unit search that the record forbids.[^4] A god who wants to split a
crowd must split it by cell, by faction, or by the option each unit holds.

**And two units with the same need always choose alike.** The engine holds one
weight profile for every unit alive, and a finding records it. A god game that
wants a zealot and a farmer to want different things is asking for a need that
no record answers today.

---

## 4. The attack sketch, evaluated

The project owner and the coordinator supplied a sketch for the attack verb.
This section tests it against the code. It is not a design and it records no
decision.

**The requirement, in the owner's words.** Attack should be "some sort of flow
field with probabilities". Something must happen "in particles to understand
the crowd flow that results in a powerful army overpowering things
realistically". The acceptance test is that **one tank still kills four
bowmen**.

**The sketch.** Apply a penetration threshold for each attacker type before
anything is aggregated. A type whose effect does not exceed the defender's
threshold contributes exactly zero. Combat then resolves for each cell as a
small table over unit types rather than for each pair of units. The field
decides how an army arrives. Probability enters as one keyed draw for each
cell, never as a draw for each unit.

**The verdict in one line.** The sketch is sound in three of its four parts.
The fourth is wrong, and the correction is that a fight belongs at the tile
rather than at the cell.

### 4.1 Does the threshold satisfy the tank test?

**Yes, and it satisfies it structurally rather than by tuning a constant.**
Zero is the identity of integer addition. A sum of zeroes is zero at any count,
so no number of bowmen ever reaches the tank. That property does not depend on
a rate, a cap, or a balance figure, so no later measurement can weaken it.

**It also keeps every determinism rule.** The threshold applies to a pair of
types, and the pair is fixed before the fold begins. So the fold is still
integer addition over a set, it is still associative and commutative, and its
identity is still zero. An aggregate must combine exactly in any order, and
this one does.[^25] Nothing about it depends on a thread count.

**Three cases give a result a player would call wrong.**

**First, the cliff.** An effect of 99 against a threshold of 100 does nothing,
and an effect of 101 does everything. One point of armour makes a unit immune
to a whole class of attacker, and one point of upgrade ends that immunity. A
player who improves a unit by the smallest possible step, and then watches a
whole war change, reads that as a defect.

**This project has met that shape before and named it.** The consumption module
says that a pure cohort has a cliff: a place is fine a little above its demand
and starves entirely a little below it. It removes the cliff by keeping a
per-unit deficit accumulator, so that a shortage degrades before it kills.[^26]
A contest with a hard threshold has the same shape and no such accumulator.

**Whether the cliff is acceptable is the owner's judgement, and the register
holds it.** The tank test asks for the cliff. A player asks for the cliff to be
soft. Those two pull opposite ways, and no engineering answer settles it.

**Second, no attrition means no crowd behaviour.** The owner asked for crowd
flow that shows a powerful army overpowering things. A tank that cannot be
hurt at all produces no flow: the bowmen arrive, nothing resolves, and they
stay. The picture the owner asked for comes from an army that grinds down, and
a hard threshold removes the grinding for every pair it applies to.

**Third, a table over counts holds no arrangement, so a unit cannot screen
another.** A cell holding one tank and one hundred bowmen of one faction
resolves as counts. The tank is immune and the bowmen are not, so the tank
stands untouched while its own escort dies. A player expects a heavy unit to
shield the units behind it. A table over counts cannot express "behind".

**None of the three refutes the sketch.** The first is a game-design question,
the second follows from the first, and the third is a property of any aggregate
resolution. It is the price of not running a fight for each pair.

### 4.2 Does a per-cell resolution produce a front line?

**The risk is real, and the number is worse than the question implies.** The
block edge is 32 tiles, and the layout is a power of two set by one constant in
the bridge. So one level 1 cell covers 1024 tiles. A fight resolved for the
whole cell kills units spread over all of them.

**The correction: resolve the fight at the tile, and keep the field at the
cell.** The engine already splits exactly this way for movement. The exit field
gives one direction to a whole block, and admission enforces the capacity at
the tile, one tile at a time.[^27] The bridge already lists the units standing
on one tile, and it rebuilds at every barrier, so the input a tile-level
resolution needs exists today.

**The cost of the tile form follows the contested tiles and not the world.** A
tile is contested when it holds units of two factions. That set is small in
every world that is not entirely at war, and the block masks already say which
blocks hold more than one faction, so the scan skips the rest.[^15]

**The cell keeps the job it is good at.** It decides where an army goes. That
is the sketch's own second point, and it survives the correction unchanged.

**How to measure it before anybody builds it.** Do not ask whether the picture
looks right. Measure the depth of the killing.

1. Build a fixture that seeds two factions on opposite sides of a world and
   runs them into contact. **Do not copy the demonstration world**, because
   that world is chosen to look right rather than to produce an edge
   value.[^28]
2. For each frame after contact, take each casualty. Measure its distance in
   tiles to the nearest tile the other faction holds.
3. Report the band that holds the middle 90 percent of the casualties, in
   tiles. A front line gives a narrow band. A smear gives a band whose width
   approaches the block edge.
4. **Put the defect back and watch the measurement move.** Resolve at the cell
   on purpose, and confirm that the band widens to the block edge. A fixture
   that cannot show the bad case is measuring itself, and this project has
   recorded that failure twice.[^28]

That measurement is cheap, it is a number rather than an opinion, and it
settles the question before any combat pass is written. A blocker holds it.

### 4.3 Where the probability goes

**A draw for each unit is wrong, and the project already holds the exact rule
that replaces it.** A record states that a cohort serves whole rations to a
keyed subset, never an equal share to everybody, and that the subset is the
ordinals of the cohort rotated by a keyed offset.[^29] [^30] One draw serves a
whole cohort.

**Casualties should reuse that rule rather than invent a second one.** The
arithmetic module already floors a share, and its own text says the caller
hands out the remainder.[^31] So the pass computes the exact expected
casualties in the fixed-point scale, floors them into a whole count, and
selects that many units by the rotation the ration rule already uses. That is
one keyed draw for each contested tile in each frame.

**Two properties make this the right shape rather than merely a cheap one.** A
whole-unit outcome is what a watcher can see, and a fractional casualty is not
a thing a player can be shown. And the order of the served set never depends on
a thread.[^32]

**One warning, and it is the warning the testing rule exists for.** A keyed
draw is invisible when it is keyed wrongly. A draw keyed on the tile but not on
the frame kills the same units for ever. A draw keyed on the slot rather than
the identity kills whoever now occupies a dead unit's slot. Both defects are
deterministic, so both pass the thread-count test and the golden hash. **Write
one test for each field of the key**, which is what closed the same defect in
the movement system.[^33]

### 4.4 Is attack one verb or several?

**It is none of them. It is a destination and a posture.**

Take the three candidates. "Attack these units" is a destination that follows
an enemy. "Hold this ground" is a destination that does not move. "Raid and
withdraw" is a destination, then a second destination, with a condition
between them.

**All three are the seed set of section 1.1, plus one small piece of unit
state.** The seed set says where the army goes. A posture says what a unit does
when it stands beside an enemy: engage, hold, or refuse. Neither needs a verb
of its own.

**This is the project's own principle, applied.** Unit types and upgrades are
data, and types parameterise the verbs rather than multiplying them.[^19] A
posture is the same kind of thing. Three verbs that differ only in a stored
value are one verb and a value.

**The condition in "raid and withdraw" belongs to Python and should stay
there.** A god watches, decides, and sends a second destination. Putting the
condition in the engine would give the engine a rule to evaluate for a set of
units in every frame, and the control plane evaluates it between frames for
nothing.

**One case does not fit, and it should be stated.** A unit that must withdraw
inside the frame in which it meets something cannot wait for Python, because
Python runs at the frame barrier and never inside a step.[^23] If the game
needs that, it needs a rule in the engine, and the posture column is where it
goes.

### 4.5 What the engine must represent before any of this exists

Five things. Four are absent and one is a table.

1. **A type on a unit.** A soldier carries a generational identity, a tile, a
   faction, a carried load, a gather order and a build order. It carries no
   type. Nothing in the tank test is expressible until it does. The arena is
   struct-of-arrays, so a column is additive rather than a rewrite.[^34]
2. **A count for each faction and each type, at whatever granularity the fight
   uses.** The cell summary holds one total unit count and nothing for each
   faction. This is the largest new structure in the sketch, and its size is
   the product of three counts, so it must be sized on purpose.
3. **A table from a pair of types to an effect, with the threshold applied.**
   This is data. The world is built with it, it holds no code, and a lookup in
   it is not a callback.
4. **A posture on a unit, and a seed set that the control plane names.**
   Section 1.1 already needs the second, for a reason that has nothing to do
   with fighting.
5. **An event that says who died and to what.** The engine writes a log when a
   unit starves and when a unit is promoted, and the bindings expose neither. A
   fight that nobody can read is a fight that nobody can repair.

**Nothing on that list contradicts a record.** Every item is unbuilt rather
than forbidden, and each is ordinary engine work with an ordinary decision
attached.

### 4.6 What the sketch got right

Stated plainly, because the corrections above run longer than the agreements.

- **The threshold before aggregation is the right mechanism for the tank
  test.** It is exact, it is order-independent, and it needs no constant to
  hold it up.
- **A table over types, rather than a fight for each pair, is the right cost
  shape.** It follows the type count and never the population.
- **The field is how an army arrives and not how it fights.** That separation
  is correct, and it is the one the engine already uses for movement.
- **One draw for a group, rather than one for each unit, is right**, and the
  project already holds the record that says how.

## 5. What a god game needs that this engine does not model

The project owner named three examples. This section says which the engine has
and which it lacks. It designs no religion. It says what must be representable
before one could be built above the engine.

### 5.1 Territory ownership: the engine has it

**This is the surprise of the report.** Territory is the best-modelled of the
three. One holder for each tile, one mask for each block, a running total for
each faction, and a spread rule that reads the terrain and resolves a contest
by a stable key. No faction ever holds water.

**The gap is the boundary, not the model.** Python reads one tile and one
window count. Section 2 designs the read.

**One question about territory is open and the owner holds it.** Whether an
upgrade changes hands when the ground does is unanswered. A god game in which
a temple sits on ground that changes hands meets that question at once.

### 5.2 Belief and allegiance: the engine has one half

**Allegiance exists in the crudest possible form.** A soldier carries a
faction. It is exclusive, it is one value, and it never changes.

**Belief does not exist and nothing resembles it.** The engine holds no
continuous state on a unit that a neighbour could influence. The nearest thing
is the influence field, which is a plane over the cell lattice with one plane
for each faction, and it carries the reach of a faction rather than the
conviction of a person.

**What a religion needs the engine to represent, before anybody writes one.**

1. **A value on a unit that can move.** Allegiance today is a hard switch. A
   conversion mechanic needs something that can be partly one thing and partly
   another. It must be an exact integer or a fixed-point value, because the
   engine holds no floating point number in simulated state.
2. **A source that raises that value at a place.** The influence field is the
   shape, and the core already accepts a source term for a faction. Nothing
   sets one today.
3. **A read that turns the field into a per-unit change without a search.** A
   unit reads the cell it stands in and nothing else. That constraint is
   already the whole shape of the choice pass, so the pattern exists.
4. **A rule that decides when the value crosses.** A need is a rate with a
   threshold, and crossing it is a fact. That record is accepted and it is the
   precedent a conversion threshold should be written against.

**Note what is not on that list.** None of it requires a new kind of storage,
and none of it requires Python inside a frame. A conversion system is ordinary
work on top of structures that exist.

### 5.3 Messaging: the engine should never have it, and it already has the gate

**A message between two gods is Python to Python.** The engine holds no text,
no channel, no delivery and no ordering between gods. It should hold none of
them, because none of it is simulated state and all of it would have to be
hashed if it were.

**What the engine owes the game is the gate, and section 2 designs it.** The
one thing the engine must not do is answer the gate wrongly or slowly, because
the game asks it whenever a god wants to speak.

**One property matters and it is easy to lose.** The gate must be answered
against a barrier and never from a half-built state. A caller that changed the
population and did not step must be refused rather than answered from a stale
bridge, which is the rule the window census already follows.

### 5.4 The three the owner did not name

**A god itself.** The engine has factions, settlements, characters and
soldiers. It has nothing that is a god. Whether a god is a faction, or a
character who owns a faction, or a thing outside the world, is a choice nobody
has made. Every verb in section 1 has a subject, and until this is chosen the
subject is a guess.

**A congregation.** A group is a site membership and not a region, under a
draft record.[^12] A congregation that follows its god across the world is not
a site membership. Read that record before assuming either shape.

**A miracle.** Every verb in the table acts through units. A god who acts
directly on the ground acts through a verb that no record anticipates, and the
upgrade store is the only structure that records what somebody did to a tile.

---

## 6. The recommendation

### 6.1 The product records

**Two records, and they are genuinely separate.** The registry allocates the
numbers. Both name the god-game developer as the audience.

**Record 0030 is the verb vocabulary.** The need is that a developer can say
what a set of units should do. It is open-ended: each game adds a verb, and the
record must bound itself or it grows without limit. Its cost at the target
scale differs for each verb.

**Record 0031 is the presence read.** The need is that a god knows whose ground
its people stand on. It is closed: one question, one answer, one fixed size.
Its cost is stated exactly and does not follow the population.

**The argument for two rather than one.** They fail different gate questions.
The bound of 0030 is a judgement about how many verbs is enough, and the bound
of 0031 is a fact about a relation between 63 factions. The cost of 0030 cannot
be stated without naming a verb, and the cost of 0031 is one word for each
faction. A single record would have to answer each gate twice, and a reader
could not tell which answer governed which half.

**They also ship apart.** The presence read needs no selector and no new verb.
It could be built and shipped while the verb vocabulary is still being argued.
Folding them together would hold the cheap one behind the expensive one.

### 6.2 The decision records that should follow

The registry allocates each number. Each title states the claim, as the scope
rule requires. Each has a row in the decisions register.

1. **A presence relation between factions is derived at the barrier and never
   stored as a fact.** The whole shape of section 2: a mask for each faction,
   folded by union during the bridge rebuild, cleared and derived again each
   frame. This is the record the game's central mechanic rests on.

2. **The control plane names the seed set of a strategy field, and never a
   destination for a unit.** What makes move and gather possible without
   giving any unit a search. It is an amendment in spirit to the record that
   fixes the seeds at the live sites of a faction.

3. **An upgrade kind is a row in a catalogue the world is built with, never a
   variant in the engine.** What turns building from two kinds into a game's
   own vocabulary. It is the design principle applied to one table.

4. **A verb that changes the faction of a unit is a command, and no pass takes
   it without one.** The conversion column write, kept apart from any rule
   that would decide it.

5. **A god is a faction, and the control plane holds everything else about
   it.** Or whichever answer the owner gives. The record matters more than the
   answer, because every verb has a subject.

Five more follow from the attack sketch. Write them in this order, because each
one bounds the next.

6. **A unit carries a type, and the type is an index into a table the world is
   built with.** Nothing in the tank test is expressible until a unit has a
   type, and the design principle already states the shape.

7. **A meeting between two factions resolves at the tile, and a field decides
   only where an army goes.** The correction of section 4.2. It is the record
   that stops a fight from smearing across a block of 1024 tiles.

8. **An attacker whose effect does not exceed the defender's threshold
   contributes exactly zero.** The tank test, stated as a constraint a reviewer
   can find a violation of. The threshold itself is a table value and belongs in
   a reference table, never in the record.

9. **Casualties are whole units served to a keyed subset, never a fraction of
   everybody.** The randomness rule. It reuses the ration rule rather than
   inventing a second one, and it is the record that keeps one draw for a group
   instead of one draw for each unit.

10. **A posture is a column on a unit, and attack is a destination and a
    posture, never a verb of its own.** What collapses three candidate verbs
    into one mechanism and one value.

**Record 9 governs determinism, so it needs a record even where it looks
obvious.** A later contributor who wants a smoother fight will reach for a draw
for each unit, and only a written constraint refuses it.

### 6.3 What to build first

**First: the presence relation, and the read that exposes it.** It is the
game's central mechanic. It answers a question the engine can already answer.
It rides on a pass that already runs. Its result is a fixed size, so it needs
no selector and it cannot be made slow by a large world. It is the one item on
this list that a god game cannot open a conversation without.

**Second: bind the build verbs the core already has.** Four public methods, no
binding. This is the cheapest ratio of value to work in the whole report, and
it closes a capability that ships inert today.

**Third: the seed set for the strategy field.** It gives move and gather at
once, and both are on the owner's list. It is a decision before it is a change,
because it amends a record that is a draft.

**Fourth: the units of a faction, read as a set.** A god that cannot list its
own people cannot use any verb on a described set. This is the selector, and
report 20 already ranks it and says to build the read side with it.[^22]

**Combat runs beside all four, and it starts with a measurement rather than
code.** Measure the width of the casualty band before anybody writes a combat
pass, by the method in section 4.2. It is cheap, it decides between a tile
resolution and a cell resolution, and building either one first risks throwing
it away. Then give a unit a type, because nothing about the tank test can be
stated until it has one.

**Later: the catalogue, the conversion verb, and the fog.** Each is real. None
of them blocks a first playable god.

**What should not be built.** No messaging, no text, no channel between gods.
The engine holds the gate and holds nothing else about a conversation.

---

## 7. Where this report disagrees with what was believed

Four corrections go to the findings register. Each is stated here with the
search that produced it, and section 9 holds every command.

1. **The write surface was counted as seven callables and it is eight.**
   `found_group` was the one missed. It founds a group and returns a site
   identity.

2. **The core has a general build verb and the bindings expose none of it.**

3. **Movement to a named place is built rather than proposed.** The draft
   record that describes it is implemented in the core.

4. **Territory is present in the engine, not absent from it.** It was listed as
   an example of what a god game needs and the engine does not model.

**One correction goes to the sketch rather than to the register.** The sketch
resolves combat for each cell. A cell covers 1024 tiles, so a fight resolved
there kills units across a block wider than any front line. The bridge already
lists the units on one tile, so the tile form needs no new input. Section 4.2
holds the argument and the measurement that would settle it.

---

## 8. The assumptions this report makes

**Every assumption about Gods and Congregations is listed here and nowhere
else.** The project owner stated one rule in full and named seven verbs. Each
line below is a guess this report needed in order to reason. None of them is a
game rule the owner stated.

1. **A god maps onto a faction.** Assumed for the whole report. Every verb
   needs a subject, and a faction is the only thing the engine holds that owns
   both units and ground.
2. **A congregation is the set of units of one faction.** Assumed. The engine
   has no other grouping that follows a god.
3. **Territory means the ground the holding rule gives a faction.** Assumed.
   The rule spreads a claim from what a faction holds and reads the terrain.
4. **The presence gate is asked between two gods, and it is symmetric in form
   but not in fact.** Assumed. A unit of A in B's land lets A speak to B, and
   this report does not assume it lets B speak to A.
5. **"Attack" means one unit reduces another, rather than one god removes a
   unit outright.** Assumed. An instant removal already exists.
6. **"Convert people" means a unit changes which god it belongs to.** Assumed.
   It could instead mean a unit gains a belief while keeping its faction.
7. **"Build things" means marking a tile, and not only founding a
   settlement.** Assumed from the plural and from the word "things".
8. **The game runs many gods rather than two.** Assumed. The presence relation
   is sized for 63 either way.
9. **A god's orders arrive between frames, and never inside one.** Assumed
   from the engine's own rule. If a language model must be asked mid-frame,
   several conclusions here change.
10. **"One tank still kills four bowmen" means that no number of bowmen kills
    the tank.** Assumed. It could instead mean that the tank wins against four
    and loses against forty.
11. **A tank and a bowman are unit types, and a god fields several of them.**
    Assumed from the test. The engine has one unit shape and no type at all.
12. **A fight happens because two factions stand on one place, and not because
    a god named a target.** Assumed. It is what makes attack a destination and
    a posture rather than a verb.

**Three of these are open enough to stop work, and each has a row in the
blockers register.** The first is that the rules of the game are not written
down, which covers assumptions 1 to 8 and 10 to 12. The second is the scale the
game runs at. The third is whether a fight at this engine's granularity looks
like a fight, which nobody has measured.

---

## 9. The searches behind each claim

Each command ran in this worktree on 3 September 2026.

| The claim | The command | What it reported |
|---|---|---|
| The write surface is eight callables | `grep -n "let mut world = self.lock()" crates/cachette-py/src/lib.rs` | Nine sites. One is `step` |
| No binding calls the build verbs | `grep -c "order_build\|destroy_upgrade\|stop_build" crates/cachette-py/src/lib.rs` | 0 |
| No Python calls them either | `grep -rn "order_build\|destroy_upgrade\|return_direction" python tests --include "*.py"` | 0 lines |
| The core has them | `grep -n "    pub fn " crates/cachette-core/src/world.rs` | `order_build`, `stop_build`, `build_order`, `destroy_upgrade`, `return_direction`, `exit_direction`, `tiles_held_by` |
| Nothing models a fight or a belief | `grep -rin "attack\|combat\|damage\|belief\|allegiance\|diplomac" crates/cachette-core/src/*.rs` | One line, and it is about coordinate conversion |
| The upgrade catalogue is two kinds | `grep -n "enum UpgradeKind" -A 10 crates/cachette-core/src/upgrade.rs` | `Road` and `Terrace` |
| The faction ceiling is 63 | `grep -n "FACTION_CEILING" crates/cachette-core/src/types.rs` | `pub const FACTION_CEILING: u16 = 63` |
| Python reaches for the singular read | `grep -rhoE "world\.[a-z_]+\(" python tests --include "*.py" \| sort \| uniq -c \| sort -rn` | 7 uses of `world.soldier_tile(` |
| A cell covers 1024 tiles | `grep -rn "BLOCK_BITS_DEFAULT" crates/cachette-core/src/bridge.rs` | `pub const BLOCK_BITS_DEFAULT: u32 = 5`, and the edge is two raised to it |
| A soldier carries no type | `grep -n "pub" crates/cachette-core/src/soldier.rs` | An identity, a tile, a faction, a carry, a gather order, a build order |
| A cell summary holds no count for each faction | `grep -n "pub struct CellSummary" -A 10 crates/cachette-core/src/pyramid.rs` | One `units` total, and six other totals |
| The arithmetic module floors a share | `grep -n "pub const fn share" -B 8 crates/cachette-core/src/sim_math.rs` | Its own text says the caller hands out the remainder |

---

## References

[^1]: PRD-0021, a developer can use the control plane without reading its source. `docs/product/accepted/prd-0021-a-developer-can-use-the-control-plane-without-reading-its-source.md`
[^2]: Research report 20, what the Python interface should be, section 0. `docs/research/reports/20-the-python-interface.md`
[^3]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^4]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, decision D1. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
[^5]: ADR-0095, a behavioural strategy arrives as a field over cells, never as a search from a unit, decision D1. `docs/adrs/draft/adr-0095-a-behavioural-strategy-arrives-as-a-field-over-cells.md`
[^6]: ADR-0110, a unit returns by climbing a reach field seeded at every site of its faction, decision D1. `docs/adrs/draft/adr-0110-a-unit-returns-by-climbing-a-reach-field.md`
[^7]: ADR-0110, a unit returns by climbing a reach field seeded at every site of its faction, decision D2. `docs/adrs/draft/adr-0110-a-unit-returns-by-climbing-a-reach-field.md`
[^8]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, the consequences section. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
[^9]: PRD-0001, a faction sees only what its own units observe. `docs/product/accepted/prd-0001-a-faction-sees-only-what-it-observes.md`
[^10]: ADR-0090, a tile upgrade is stored sparsely, as the difference from the generated world, decision D1. `docs/adrs/draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md`
[^11]: ADR-0075, the founding choice reads a bounded sample of the world, decision D5. `docs/adrs/accepted/adr-0075-the-founding-choice-reads-a-bounded-sample-of-the-world.md`
[^12]: ADR-0065, a group is a site membership, not a region. `docs/adrs/draft/adr-0065-a-group-is-a-site-membership-not-a-region.md`
[^13]: ADR-0085, an entity crosses to Python as one opaque identity that the engine resolves, decision D3. `docs/adrs/accepted/adr-0085-an-entity-crosses-to-python-as-one-opaque-identity.md`
[^14]: ADR-0012, tiles are dense columns and units are a generational arena, decision D2. `docs/adrs/accepted/adr-0012-tiles-are-dense-columns-and-units-are-a-generational-arena.md`
[^15]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D2. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
[^16]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D4. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
[^17]: ADR-0040, Python is a control plane, not a data plane, decision D1. `docs/adrs/draft/adr-0040-python-is-a-control-plane-not-a-data-plane.md`
[^18]: Budgets and costs, the scale constants. `docs/reference/budgets.md`
[^19]: Project orientation, the design principles. `CLAUDE.md`
[^20]: ADR-0051, a selector is a lazy expression tree that Rust evaluates, decision D1. `docs/adrs/accepted/adr-0051-a-selector-is-a-lazy-expression-tree.md`
[^21]: Research report 20, what the Python interface should be, section 0.1. `docs/research/reports/20-the-python-interface.md`
[^22]: Research report 20, what the Python interface should be, section 7.3. `docs/research/reports/20-the-python-interface.md`
[^23]: Project orientation, the hard invariants. `CLAUDE.md`
[^24]: ADR-0110, a unit returns by climbing a reach field seeded at every site of its faction, decision D5. `docs/adrs/draft/adr-0110-a-unit-returns-by-climbing-a-reach-field.md`
[^25]: ADR-0023, an aggregate combines exactly, in any order, decision D1. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
[^26]: ADR-0106, a cohort serves whole rations to a keyed subset, the context section. `docs/adrs/draft/adr-0106-a-cohort-serves-whole-rations-to-a-keyed-subset.md`
[^27]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D3. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
[^28]: Testing Rules, section 2a. `.claude/rules/testing.md`
[^29]: ADR-0106, a cohort serves whole rations to a keyed subset, decision D1. `docs/adrs/draft/adr-0106-a-cohort-serves-whole-rations-to-a-keyed-subset.md`
[^30]: ADR-0106, a cohort serves whole rations to a keyed subset, decision D2. `docs/adrs/draft/adr-0106-a-cohort-serves-whole-rations-to-a-keyed-subset.md`
[^31]: The share operation of the arithmetic module. `crates/cachette-core/src/sim_math.rs`
[^32]: ADR-0106, a cohort serves whole rations to a keyed subset, decision D3. `docs/adrs/draft/adr-0106-a-cohort-serves-whole-rations-to-a-keyed-subset.md`
[^33]: Testing Rules, section 2. `.claude/rules/testing.md`
[^34]: ADR-0012, tiles are dense columns and units are a generational arena, decision D3. `docs/adrs/accepted/adr-0012-tiles-are-dense-columns-and-units-are-a-generational-arena.md`
