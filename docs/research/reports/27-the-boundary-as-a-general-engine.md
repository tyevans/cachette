# What The Python Boundary Would Have To Be For A Stranger

Research report 27. It asks what the Cachette Python boundary would have to
become to serve two audiences it was not written for: a researcher who trains
agents, and a developer who builds a different game. Prepared 6 September 2026.

Cachette is a world simulation engine. The core is Rust. The control plane is
Python. The engine simulates a hex world at two levels of detail, and it holds
one property it cannot recover once lost, which is determinism.[^1] The Python
package re-exports one compiled extension module, and a developer who builds on
this engine writes Python and never opens the core.[^2]

This report reads the compiled bindings, the Python package and the records that
govern the boundary. Every count in it comes from a command the author ran on
the `integration` branch at commit `3d40cff`. No claim here is a performance
measurement. The author ran no benchmark, no test and no build, because three
other agents held the machine.

## 0. The conclusion

**The boundary is not a general engine wearing a game's name. It is one game,
and most of its surface is that game.** Of the 180 methods the world exposes,
109 name a noun of this game: a trade, a campaign, a character, a luxury, an
upgrade category, a settlement, a housing count, a relation, a deed. A stranger
who wants a different game does not find a general mechanism under those names.
Almost every one of them is a distinct entry point with a distinct signature.

**The project already knows the right answers, and it wrote them down before it
built them.** Three accepted records describe exactly the general shape this
report would otherwise have to propose: a selector is a lazy expression tree, an
observation and an action are schema-declared bounded tables the engine owns,
and a batch of worlds steps in one call.[^3] [^4] [^5] None of the three has any
code. The binding registers a `SelectorError` class whose own doc comment says
the selector interface is not written. The word `observation` appears zero times
in the binding, while the core carries a complete fog module. The largest
generalisation available to this project is therefore not a new idea. It is the
three ideas it has already accepted and not built.

**The two audiences share one need and diverge after it.** Both want a stable,
schema-declared table of numbers in and one integer out, and both want the world
to be reproducible from a value they can write down. Past that they pull apart.
A researcher wants that table frozen and cheap. A developer wants to change what
is in it. The single boundary that serves both is the schema mechanism, not the
schema.

## 1. What I measured

The binding is one file of 7857 lines plus a 63-line helper.[^6] It exposes two
classes and four module functions. `World` carries 180 methods. `Camera` carries
nine. The module functions are `version`, `event_schema`, `stock_target` and
`stock_ceiling_of_one_settlement`.

Inside the 180 world methods:

- 33 begin with `set_`. Each is a separate entry point for one scalar.
- 12 end with `_columns`. Each returns the columns of one event kind.
- 11 end with `_count`.
- 3 begin with `define_`. These are the only calls that write a table row.
- 109 name a noun of this game, by a search over the words `weather`, `trade`,
  `campaign`, `relation`, `character`, `luxury`, `upgrade`, `settlement`,
  `site`, `housing`, `queue`, `convert`, `deed`, `influence`, `score`,
  `standing` and their neighbours.

The step runs 35 named stages. One macro declares them once, in order, and
derives the enumeration, the count, the list and three property tables from that
one list.[^7] The list is a compile-time constant. Nothing in Rust and nothing in
Python composes it.

The game end reads four clauses. The reader table is a Rust array of four
function pointers paired with a four-variant enumeration, and a comment above it
says the order of the table is a rule of the game.[^8]

The world holds five terrain kinds and three resource kinds, both as compile-time
counts. A two-dimensional constant table of five rows by three columns states
what ground carries what.[^9] A faction is a bit in a mask, and the ceiling is 63
factions.[^10]

The unit type table holds eight named capability columns. The upgrade table holds
eight named columns. A caller supplies rows through `define_unit_type` and
`define_upgrade_row`. The rows are data. The column set is code.

Six setters take a period and a phase: the queue, the positions, the growth, the
economy, the characters and the advertisement. That is one scheduling mechanism
with six copies at the boundary.

The step releases the interpreter. Exactly one call to `Python::detach` exists in
the binding, and it wraps the whole step.[^11] Every other call holds the
interpreter, including `draw`, which fills a whole frame of pixels, and the
plan solver.

Every array crossing copies. The binding constructs 95 NumPy arrays and borrows
none. At the target scale of 16.7 million tiles, one `tile_values` call copies
about 67 megabytes.

The world reports no seed and no faction count. The agent session store keeps
both in Python for that reason, and its own docstring says so.[^12] The binding
holds no save, no load, no snapshot, no restore and no clone.

The agent server exposes 18 tools. All of them build, step or read. None reaches
fog, and none offers an action space.

## 2. Where the design is already right

A report that finds everything wrong is not useful, and several parts of this
boundary are better than what a stranger would expect.

**One declaration site, derived everywhere.** The stage list, the event layouts
and the unit type columns each live in exactly one place, and the boundary
derives from them. `event_schema` returns the field names and NumPy type names of
every event, so a caller decodes the log without holding a byte offset. That is
the correct pattern, and it is the pattern the observation record asks for. The
project has already built the mechanism it needs; it has pointed it at events
rather than at observations.

**An entity crosses as one opaque identity.** A caller passes an identity back and
the engine resolves it, and it refuses the identity of a unit that has died.[^13]
This is what lets a stranger hold references safely without learning the arena.

**A field solver takes a caller-named seed set.** The control plane names a set of
tiles and the engine seeds a plane at every one of them.[^14] The unit carries the
plane it obeys and no address. This is genuinely general. It is a flow field with
a caller-supplied boundary condition, and a different game uses it unchanged.

**A unit type is a row, and zero means cannot.**[^15] The instinct is right, and it
is the instinct the rest of the boundary should follow.

**Nine typed error classes.** A stranger who does something wrong gets a class,
not a string.

**The world is frozen and guarded by a mutex.** Many worlds live in one
interpreter without a global. That is the precondition for batching, and it is
already met.

## 3. The generalisations

### 3.1 Build the observation and action tables that the record already describes

The record is accepted and complete.[^4] The engine returns a schema for the
observation and a schema for the action table. The observation is one flat array
of signed integers whose length follows the world parameters and never the
population. It holds only what the faction has observed, and the interface offers
no argument that asks for the truth. An action is one integer into a bounded
table that factorises into a verb, a target and a magnitude by a mixed radix the
schema states. One byte for each row says whether the verb would refuse it now.

**What it buys.** It is the whole researcher story in one mechanism. It is also
the only place in the boundary where fog reaches Python, and a faction-scoped
reader is what a game developer needs for a human player as well. The verb rows
derive from the controller's choice enumeration, which already holds nine
arms.[^16] The engine gets an action space for free from a table it maintains for
another reason.

**What it costs.** The observation reader must read aggregates the engine already
keeps and the level 1 summary the pyramid already rebuilt. The fog module exists
but no reader answers for one faction, and a backlog item holds that work.[^17]
The legality byte must read the same candidate lists the verbs read and duplicate
no rule, which is the expensive part, because a duplicated rule is the defect
shape this project meets most often.

**What it breaks.** Nothing, if the reader touches no state. The risk to
determinism is the candidate list order. The record already binds it to a stable
key, and a violation is visible to a reviewer.

### 3.2 Write the selector, and route the set-valued verbs through it

The record is accepted.[^3] Python builds a lazy expression tree, the tree crosses
once, Rust evaluates it, and the evaluation order never comes from a thread. The
boundary today routes around this entirely: a verb takes a `Sequence[int]` or a
NumPy array of identities that the caller assembled.

**What it buys.** It is the difference between a control plane and a thin remote
procedure call. A developer describes a set by a property instead of holding every
identity, which is a stated need of the developer record.[^18] It also removes the
one place where the no-loop rule is unenforced: the package docstring admits that
no type refuses a loop and that the declared tier reaches no code outside the
core.[^19] A selector that Rust evaluates makes the loop unnecessary rather than
forbidden.

**What it costs.** It is real work: a tree type in Python, a stable encoding, an
evaluator in Rust, and a domain check that fails when the tree is built. The
result-may-be-a-range record adds a second representation.[^20]

**What it breaks.** It is the largest determinism surface in this report. Every
evaluation must fix its order by a stable key, and a set-valued verb must apply in
an order the data fixes, never the order a caller wrote. The record says this. A
test must prove it at more than one thread count.

### 3.3 Make the win condition a caller-supplied clause set

Four readers sit in a fixed array. A researcher wants to define an objective. A
developer wants different ones. The general shape is already visible in
`standing`, which returns the five running values that the four readers compare.

Turn that inside out. The engine keeps the running values, which is the part it
must do because it owns the aggregates. The caller supplies a small set of clauses
over those values: a named quantity, a comparison and a threshold. The engine
evaluates the clauses in the order the caller gave them and records the first that
fires. A game with no end supplies none.

**What it buys.** A researcher gets an episode boundary they chose. A developer
gets a different game. The reward function follows for free, because a reward is a
weighted read of the same quantity vector.

**What it costs.** Small. The readers are already pure functions of the world, the
values are already computed, and the record that governs the end says the record is
written once and stops the controllers, which stays true.[^8] The work is defining
the quantity vector as a schema rather than as a typed dictionary.

**What it breaks.** The clause set enters the state hash, because the game end
record does. Two worlds with different clause sets then hash differently, which is
correct but must be stated. Nothing else.

### 3.4 Make the tables a stranger supplies wider than the two that already exist

Unit types and upgrades are rows a caller writes. Terrain kinds, resource kinds and
the ground-to-yield table are not. The five-by-three constant table is exactly the
shape of the two tables that are already data, and it is the one a developer
building a different game hits first. A game with ore and mana and no wood cannot
say so.

The honest ordering here is: the resource-to-ground table is a table and should be
one. The terrain kind count is not, because the terrain is generated from the seed
by a pure function and never stored, and the generator's arithmetic is bound to the
kinds it produces.[^21] Widening the kind count is a change to the generator, not to
a table.

**What it buys.** The commodity set is the vocabulary of an economy. A developer who
can name it can build a different economy on the production, upkeep, store, trade
and consumption machinery that already exists.

**What it costs.** Several fixed-width arrays are sized by the resource count at
compile time. A runtime count means either a bound with padding, or a generic
parameter, or an allocation. A bound with padding is the cheapest and the least
surprising, and it matches how the faction count already works at 63.

**What it breaks.** The state hash covers the stores. A padded array must declare its
padding, which is the fifth hard invariant of this project, and an undeclared byte
puts false nondeterminism into the hash.

### 3.5 Give one setter a name instead of thirty-three

Thirty-three `set_` methods, six of them the same period-and-phase pair, is a
boundary that grows one entry point for every value. A stranger cannot enumerate the
configuration, cannot serialise it, cannot diff two worlds' settings, and cannot see
which setting governs which stage.

Replace them with one schema and two calls: the engine declares its parameters with
a name, a kind, a bound and a default, and a caller reads and writes by name. This
is the same mechanism as `event_schema`, applied to configuration.

**What it buys.** A researcher writes an experiment configuration as a dictionary and
reproduces it. A developer sees the whole tuning surface at once. The balance
harness, which today parses markdown rows of a register, reads it from the engine
instead.[^22]

**What it costs.** Low, and mechanical. It is also the change most likely to be
undone by an agent adding a thirty-fourth setter, so it needs a check that fails when
a `set_` method appears outside the schema.

**What it breaks.** Nothing. A name lookup replaces a symbol lookup, and neither
enters the simulation.

### 3.6 Make reproducibility a thing a caller can hold

The world does not report the seed that built it. It cannot be saved, loaded,
snapshotted or cloned. The state hash is a `u64` that says two runs agree, and
nothing more.

This is the gap between determinism being true and determinism being usable. A
researcher who cannot serialise a world cannot checkpoint a run, cannot restore a
failing episode, cannot fork a world at a decision point, and cannot ship a
reproduction to a colleague. A game developer cannot save the game.

The minimum is small and worth doing first: report the seed, the extent and the
faction count back, and expose the whole configuration by the schema of 3.5. That
alone makes a run reproducible from values a caller can write to a file.

The full version is a byte-exact serialisation of the world. That is a real
project. Every simulated structure is already plain data with declared padding,
because the event rule and the hash rule both demand it, so the ground for it is
prepared. The cost is that the format becomes a compatibility surface.

**What it breaks.** Nothing about determinism. A save that round-trips to a
different hash is a defect the existing hash test finds immediately, which is a
better position than most engines start from.

### 3.7 Stop copying every array, and release the interpreter more than once

Every one of the 95 array crossings copies. One `tile_values` call at target scale
moves about 67 megabytes. A researcher who reads an observation each tick pays that
in the inner loop.

The observation table of 3.1 removes most of this need, because it is bounded and
small. For the readers that stay, a read-only borrow of a column that the engine
owns is the right answer, and the record on what copies and what does not already
says the call site must declare which.[^23]

Separately, `draw` and the plan solver hold the interpreter while they run. Two
Python threads therefore cannot render two worlds at once, and the batch record's
promise that the interpreter is released for the whole batch call needs the same
treatment applied more widely.[^5]

**What it costs.** A borrowed array aliases engine memory, and a caller who holds it
across a step reads a changed world. That is a correctness hazard the copy currently
prevents. The declaration at the call site is what makes it safe, and it must be in
the doc comment the reference generates.

## 4. What I would do first, second and third

**First: the observation and action tables, faction-scoped, with the legality
byte.** It is accepted, it is designed, it has a backlog item under it, and an
accepted product record states the need it answers.[^24] It is the
single largest step toward both audiences. It also drags fog across the
boundary, which nothing else does. Do the schema mechanism properly, because
everything after this reuses it.

**Second: the configuration schema, the seed reader and the win clause set.** These
three are cheap, they are mechanical, and together they are what turns a run into an
experiment. A researcher can then state a world, state an objective, and state that
two runs are the same run. None of the three risks determinism.

**Third: the selector.** It is the deepest change and the one with the most
determinism surface, so it should follow the two above rather than lead them. Doing
it third also means the action table already exists, and the action table's candidate
lists are the first real consumer of a set-valued evaluation. Build the selector
against a caller that already needs it.

Serialisation sits alongside the third, not before it. The minimum part of it belongs
in the second.

## 5. What I reject

**A caller-composed stage list.** The task asked whether a caller could compose the
35 stages. I recommend against it, and this is the clearest no in the report. The
stage order is not a list; it is a set of read-write dependencies that the step's
comments spend hundreds of lines justifying. The bridge rebuild must sit at the
barrier or a dead identity survives a frame. The gather resolve must follow the
barrier or it takes from the tile the unit left. Recovery must precede the resolve.
Admission must read the structure the last barrier rebuilt. A caller who reorders
these does not get a different game; they get a world whose invariants are silently
false, and the two determinism tests will not notice, because a consistently wrong
order is still deterministic. That is the defect shape this project already knows it
cannot see.[^25] If stages must be composable, the unit of composition is a stage that
declares what it reads and what it writes, with a check that derives the order, and
that is a much larger piece of work than a list.[^26] Do not offer a list.

**A Python callback inside the step.** The crate split makes it a compile error, and
the record's whole value is that the compiler enforces it. A researcher will ask for
it, because most environments allow it. The answer is the action table: the learner
acts at the frame barrier, not inside a system.

**Floating point in the observation.** A researcher will ask for this too, and the
record already answers: a fixed-point value crosses as its raw integer and the learner
scales it on its own side. Giving way here costs the one property the project cannot
recover.

**A general scripting or rules language for content.** The instinct that unit types
are data is right and should be extended. The instinct that content should be code is
where engines go to die, and it is incompatible with keying every random draw and
fixing every iteration order. Extend the tables. Do not add an interpreter.

**Deleting the game verbs to make room.** The 109 game-named methods are not waste.
They are the proof that the general machinery underneath carries a real game, and a
general engine with no game on it is untested. Add the general surface beside them.
Retire a named verb only when the general one has a caller.

**Removing the 63-faction ceiling.** A faction is a bit in a mask and a relation is a
plane, and that choice buys exact integer set operations at target scale.[^10] Sixty-three
sides is more than any strategy game and more than any multi-agent training run this
project will do. Leave it.

## 6. Where the two audiences genuinely conflict

They agree on more than they disagree on, and the agreement is the mechanism: a
schema the engine declares, integers in both directions, and a world reproducible
from a written value. Both benefit from every item in section 4.

They diverge in three places, and each is a real trade rather than a matter of
sequencing.

**A stable observation space against a changing one.** A researcher needs the
observation layout to be fixed for the life of a training run, because a policy's
first layer is that layout. A developer changes what the world holds, and every change
moves the layout. These are opposed. The resolution is that the schema is versioned
and the caller pins it: a researcher records the schema with the checkpoint and
refuses to load a policy against a different one. That costs the developer nothing
and gives the researcher the guarantee. It does not make the conflict go away; it
makes it visible at load time instead of at inference time.

**Speed and batching against expressiveness.** A researcher's cost is dominated by
steps per second across many worlds, and the batch record's answer is many small
worlds stepping in one call with the interpreter released. A developer runs one large
world and wants it rich. Every general mechanism this report proposes — a schema
lookup instead of a symbol, a clause set instead of four functions, a selector tree
instead of an array of identities — trades a little speed for expressiveness. At
16.7 million tiles the trade is invisible, because the step is dominated by the
per-tile passes. In a researcher's batch of small worlds the per-call overhead is a
larger share, and a schema lookup in the inner loop is exactly the wrong place for
it. The resolution is that a researcher resolves the schema once and holds the
resulting offsets, which is what the record's mixed-radix decoding already assumes.

**A fixed objective against no objective.** A researcher needs an episode to end and
a scalar to maximise, and needs both to be stable. A developer often wants a world
that does not end at all, and wants the thing a player optimises to be unstated. The
clause set of 3.3 serves both only if the empty clause set is a first-class case, and
if the reward is a caller-side read of the quantity vector rather than an engine-side
number. Build it that way. An engine that owns the reward serves one audience and
fights the other.

There is a fourth divergence that is not a conflict, and it is worth naming because it
sets the priority. A researcher's needs are almost entirely reads: an observation, a
legality mask, a reward, a reset, a batch. A developer's needs are almost entirely
writes: new verbs, new content, new rules. The read side is smaller, it is fully
specified in an accepted record, and it risks determinism least. That is why it goes
first.

## References

[^1]: Project orientation, the hard invariants. `CLAUDE.md`
[^2]: Research report 20, what the Python interface should be. `docs/research/reports/20-the-python-interface.md`
[^3]: ADR-0051, a selector is a lazy expression tree that Rust evaluates, decisions D1, D2 and D5. `docs/adrs/accepted/adr-0051-a-selector-is-a-lazy-expression-tree.md`
[^4]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decisions D1 to D5. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
[^5]: ADR-0155, a batch of worlds steps in one call, in index order, decisions D1 and D2. `docs/adrs/accepted/adr-0155-a-batch-of-worlds-steps-in-one-call-in-index-order.md`
[^6]: The Python bindings. `crates/cachette-py/src/lib.rs`
[^7]: The stage declaration. `crates/cachette-core/src/stage.rs`
[^8]: ADR-0148, a game end is recorded once and stops the controllers, decisions D1 to D3. `docs/adrs/accepted/adr-0148-a-game-end-is-recorded-once-and-stops-the-controllers.md`
[^9]: The resource module, `CEILING` and `PRESENCE`. `crates/cachette-core/src/resource.rs`
[^10]: ADR-0053, a faction is a bit in a mask and a relation is a plane. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
[^11]: ADR-0042, the interpreter is released for the whole step. `docs/adrs/draft/adr-0042-the-interpreter-is-released-for-the-whole-step.md`
[^12]: The agent session store. `python/cachette/agent/session.py`
[^13]: ADR-0085, an entity crosses to Python as one opaque identity that the engine resolves. `docs/adrs/accepted/adr-0085-an-entity-crosses-to-python-as-one-opaque-identity.md`
[^14]: ADR-0125, the control plane names the seed set of a destination field, decisions D1 and D3. `docs/adrs/draft/adr-0125-the-control-plane-names-the-seed-set-of-a-destination-field.md`
[^15]: ADR-0145, a unit type is a row of capability columns and zero means cannot. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
[^16]: The controller choice enumeration. `crates/cachette-core/src/controller.rs`
[^17]: Backlog item 0495, build the observation plane and let every reader answer for one faction. `docs/backlog/proposed/0495-build-the-observation-plane-and-let-every-reader-answer-for-one-faction.md`
[^18]: PRD-0030, a developer builds a game the engine did not anticipate. `docs/product/shaped/prd-0030-a-developer-builds-a-game-the-engine-did-not-anticipate.md`
[^19]: ADR-0043, a declared tier enforces the no-loop rule, and the API refuses the loop, decision D5. `docs/adrs/draft/adr-0043-a-declared-tier-enforces-the-no-loop-rule.md`
[^20]: ADR-0052, a selector result may be a range, not only an enumerated set. `docs/adrs/draft/adr-0052-a-selector-result-may-be-a-range.md`
[^21]: ADR-0068, terrain is generated from the seed and is never stored as a map, decisions D1 and D2. `docs/adrs/accepted/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
[^22]: The balance harness. `python/cachette/balance/__init__.py`
[^23]: ADR-0044, what copies and what does not is declared at the call site. `docs/adrs/draft/adr-0044-what-copies-and-what-does-not-is-declared-at-the-call-site.md`
[^24]: PRD-0056, a learner plays one faction against the controllers. `docs/product/accepted/prd-0056-a-learner-plays-one-faction-against-the-controllers.md`
[^25]: Recurring Defect Shapes, shape 4. `.agents/rules/recurring-defects.md`
[^26]: Backlog item 0237, declare what each stage reads and writes. `docs/backlog/proposed/0237-declare-what-each-stage-reads-and-writes.md`
