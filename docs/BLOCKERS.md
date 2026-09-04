# Blockers (Register)

This document is a **register**. It lists work that is stopped and names what
must happen to start it again.

A blocker needs **information** the project does not have. Compare
`DECISIONS.md`, which lists choices that need judgement. If work can continue
under a stated assumption, it is a decision, not a blocker.

Numbers are permanent. Never reuse one. A resolved blocker keeps its row.

| Field | Meaning |
|---|---|
| Blocks | What cannot proceed |
| Owner | Who can resolve it |
| Status | `Open`, `Resolved`, or `Dropped` |


## Allocating a number

**Claim the next number below before you write the row.** Increment it in the
same change that adds the row.

A writer that numbers a row by reading the last row collides with any other
writer working at the same time. That happened, and it is recorded as
precedent.[^ALLOC]

**Next number: BLK-112**

[^ALLOC]: Findings register, FND-038. `docs/FINDINGS.md`

## Open

### BLK-110 — Nobody has said what the variety score should change

**Owner:** the project owner. **Blocks:** wiring the luxury variety score to
any simulation pass.

The engine counts how many different luxuries stand on a tile, in a level 1
cell, on the ground one faction holds, and in the whole world. Nothing in the
engine reads any of those numbers.[^BLK110A]

The project owner asked for luxury resources so that resource variety becomes
a scorable thing for research. He suggested that it could change the worker
policy of a faction, and he said that he did not know.[^BLK110B]

**What would close this.** A statement of one effect: which pass reads the
score, what the score changes, and by how much. The rule on records forbids
inventing a value that an unanswered question governs, so the engine holds the
score and changes nothing until this row closes.[^BLK110C]

**What the project does meanwhile.** The control plane reads the score. A game
built on this engine can make its own rule from it, outside the simulation.

### BLK-111 — Nobody has said whether 64 luxuries is enough

**Owner:** the project owner. **Blocks:** a catalogue wider than one word.

A set of luxuries is one 64-bit word, so the catalogue addresses 64 luxuries.
A caller that names the luxury numbered 64 gets a typed refusal, and the
engine never folds two luxuries onto one bit.[^BLK111A]

**What would close this.** A statement of how many luxuries the downstream
game needs. A catalogue above 64 needs a wider word or a second word. Both
change what the engine stores for each tile, and both change the state hash,
so the choice is cheaper now than later.

**What the project assumes meanwhile.** Sixty-four is enough. The ceiling is
the width of one machine word, and the scale constants table states that width
as the reason for the faction ceiling as well.[^SCALE] The two ceilings are not
the same number: a faction reserves one value for nobody, and a luxury reserves
none.


### BLK-007 — Most cost figures are still derived on the target platform

**Owner:** engineering. **Blocks:** confidence in most cost figures in most
reports.

**This row narrowed on 3 September 2026. It did not close.** A benchmark now
lives in the repository, and a script runs it on a Graviton instance and
destroys the instance afterwards. Every axis of the sweep is a parameter. Two
runs measured five quantities, on two instances of different size, and many
runs followed them. A register holds every row, every machine and every
commit.[^MEASURED]

**What is now measured.** The cost of one frame against the tile count and
against the unit count, up to 16,777,216 tiles and 1,000,000 units, at 1, 2,
4, 12 and 16 threads. The cost of building a world. The cost of the hash of
the whole world. The resident memory of a world. Whether one frame at the
target scale trips an integer overflow check, which it does not. Every figure
was taken on a world that holds no settlement and no character.

**What keeps this row open.**

1. **The figures in the research reports.** A cache hit rate, an allocation
   count for each frame, and the cost of a call across the language boundary
   are all derived, and the benchmark measures none of them.
2. **Any figure for a world that holds settlements or characters.** The
   measured world holds neither, so the rate pass, the consumption pass and
   the position pass did no work and the character arena was empty. Every
   measured frame figure and the memory figure are lower bounds.

**A stage inside a step no longer holds this row open.** A crate feature names
every stage of a frame, and the register holds the tables.[^MEASURED]

**The machine size no longer holds this row open, and the reason is a
result.** The unit passes stop gaining above 12 threads, so a larger machine is
not the missing measurement.[^FLOOR]

**The frame figure this row once quoted is out of date, and the row now cites
the register instead.** This row said 500 milliseconds at the target scale.
That was the first run, on 16 hardware threads, on a world whose units were
packed. Later work made the engine faster. The last whole-frame stage table
gives 177.9 milliseconds at the target scale on 12 threads, and two later runs
of a changed tree gave 167 to 169 milliseconds. The register holds every row
with the machine, the commit and the fixture of each.[^MEASURED]

**The two figures do not share a fixture, so do not read the difference as the
whole gain.** The first packed the units and ran 16 threads. The later ones
scatter the units and run 12, and a scattered unit costs about twice a packed
one. **Cite the register for a frame figure. Do not cite this row.** A finding
records how the stale figure survived here long enough to be quoted.[^STALE]

**Do not read a document that says no measurement exists as current.** That
sentence was true when it was written, and about ninety documents hold it.
This row is the current statement. A finding records how far the sentence
spread and why it was not swept.[^SPREAD]

### BLK-045 — No Python developer outside this repository has used the interface

**Owner:** the project owner. **Blocks:** confidence in the ranking of the jobs
that the Python interface must serve.

**A research report ranks five jobs and designs the interface around the
first.**[^BLK45A] That ranking rests on two graded reads of the published
reference page, and on the shape of the Python inside this repository. Both are
evidence. Neither is a user.

**The two graded reads share one method, so they share its limits.** Each reader
was given one web address and nothing else. A reader who cannot install the
package cannot say which call they would reach for on the second day, and cannot
say which of five jobs they would meet first. Both reads led with the same
complaint, which is a strong signal about the documentation and a weak one about
the ranking.

**This is information and not a judgement.** The options are not known, because
nobody has said what they wanted to build. A guess produces an interface that is
elegant against a job nobody has.

**Work continues.** Two of the five jobs are settled by records that do not
depend on this row: a selector is specified in full, and the boundary rule is
fixed.[^BLK45B] [^BLK45C] The report's ranking is the part this row governs, and
a product record that states the ranking expresses it against this row.

**What closes this.** The project owner puts the package in front of a Python
developer who has an idea and has not read this source, and records what that
person tried to do first and where they stopped.

### BLK-040 — Under which name does a reader install the package, and does the project publish it?

**Owner:** the project owner. **Blocks:** the install line in the reference, in
the tutorial and in the how-to guides.

**A reader who reaches the published reference cannot get the software.** The
page named no install command at all, and a graded review scored that gap as
the second worst failure on the page.[^BLK40A] A reader who supplies the
obvious command installs a different project, because the public Python package
index answers on the name `cachette` with an unrelated package by another
author.[^BLK40B]

**Two facts are missing, and neither is in the tree.** The first is the
distribution name this project publishes under. The second is whether it
publishes to a public index at all, or expects every reader to build from a
checkout. The package manifest states the name `cachette` and the version
0.0.0, and no job in the tree uploads anything.

**This is information and not a judgement.** A guess writes an install command
that installs somebody else's software, and the import failure that follows
names no cause.

**Work continues.** The reference now states that no public index carries this
engine and gives the commands that build it from a checkout. That is true today
and it is prose that no test can hold, so it decays the moment this row closes.

**What closes this.** The project owner names the distribution name and says
whether the project publishes to a public index. A search of the tree for this
row number then repairs every place that states the install path.

### BLK-036 — Does an upgrade change hands when the ground does?

**Owner:** the project owner. **Blocks:** any rule that ties an existing
upgrade to the faction that holds the tile under it.

The engine stores an upgrade on a tile and asks nothing about who holds the
tile.[^BLK34A] A holder column names who holds each tile, and that value moves
as units move.[^BLK34B] Nothing says what happens to an upgrade on a tile whose
holder changes.

**This row is the part of BLK-034 that stayed open when the rest of it
resolved.** It was split out on 3 September 2026, because a row that is mostly
answered reads as open and stops work that should proceed.

**The three answered questions do not answer this one.** A faction builds only
on ground it holds. Anyone may destroy an upgrade. Destruction takes work, and
a faction-level removal is instant. Every one of those is an act that somebody
invokes. This question asks what happens when nobody invokes anything and the
ground changes hands.

**Two shapes the answer could take.** The upgrade goes to the new holder, or
the upgrade stays with the faction that built it. A third shape is that the
upgrade is destroyed. Each reaches the state hash, so the engine cannot hold
two of them.

Work continues without the answer. The engine states the storage and the
arithmetic, and neither depends on a faction. A rule invented here would be a
content decision made by the wrong person.

### BLK-050 — The rules of the downstream game are not written down

**Owner:** the project owner. **Blocks:** the meaning of every verb the
downstream game asks for.

A game called Gods and Congregations is being built on this engine. The project
owner named six things a god will want to do, in one sentence, and stated one
rule in full: a god may send a message to another god only while one of its own
units stands in that god's territory.[^BLK50A]

**What is missing is what each verb does.** For each of the six, the project
does not know what counts as success, what happens on failure, what the world
looks like afterwards, or what a player sees. A research report reasons about
all six, and it needed twelve assumptions to do it. That report lists every one
of them in a single section, so that the owner can correct each.[^BLK50B]

**This is information and not a judgement.** The options are not known, because
nobody has written down what the game does. A guess produces an engine feature
that is precise against a rule the game does not have.

**Two questions inside this row matter more than the rest.** What a conversion
changes, and what a god is in engine terms. The engine holds no belief and no
allegiance that moves, and it holds nothing that is a god. Every verb has a
subject, and the subject is currently a guess.

**Work continues.** The engine gaps the report found are real whatever the game
turns out to be: the core holds a build verb the bindings do not expose, the
control plane cannot name a destination, and territory is readable one tile at a
time. None of those depends on a game rule.

**What closes this.** The project owner writes down, for each verb, what a
player asks for and what the world does in reply.

### BLK-051 — The scale the downstream game runs at is unknown

**Owner:** the project owner. **Blocks:** the ranking of the interface work
against the downstream game.

This engine targets 16.7 million tiles and one million units, and the scale
constants table holds both.[^SCALE] Every argument about what the control plane
may do is priced at that scale. A loop over the population is forbidden because
a million crossings in a frame cannot be paid.

**A congregation may be much smaller.** Nobody has said how many units the
downstream game runs, or how many gods. If it runs ten thousand units, a Python
loop is merely slow rather than impossible, and several conclusions about what
to build first change with it.

**This is information and not a judgement.** The engine's own target is
decided. What is unknown is the consumer's, and only the consumer can say.

**Work continues.** Nothing the report recommends becomes wrong at a smaller
scale. The ordering of it may.

**What closes this.** The project owner states the unit count, the god count
and the world size that the downstream game runs at.

### BLK-080 — Nobody has measured whether the engine's own movement ever brings two factions onto one tile

**Owner:** engineering. **Blocks:** confidence that a contest at the tile ever
fires in a running world.

A fight resolves at the tile, and a tile contest needs both factions on one
tile.[^BLK80A] The admission rule reads the capacity of the ground and not the
faction, so nothing refuses a mixed tile. Ordinary ground holds 8 units, and a
tile already full of one faction offers no room.[^BLK80B]

**The evidence that settled the granularity does not settle this.** The harness
placed the armies on shared tiles directly, through the placement call, which
skips admission and skips the movement pass. So the arrangement it measured is
one the engine has never produced for itself.

**A god cannot aim an army today.** A unit takes its direction from a field over
cells, and the control plane names no destination, so no call sends one army at
another.[^BLK80C] Until a seed set exists, nothing can drive the case.

**Work continues.** The granularity is decided and the band is measured. What is
unknown is how often a running world reaches a contested tile, which changes how
much a contest costs and not where it resolves.

**What closes this.** Somebody drives two factions together through the movement
pass, with a seed set the control plane named, and reports how many tiles hold
two factions.

## Resolved

### BLK-052 — Nobody has measured whether a fight at this granularity looks like a fight

**Resolved on 3 September 2026. The measurement is taken, and a fight resolves
at the tile.**

A level 1 cell summarises one block of tiles, and the block edge is 32. A design
sketch resolved a fight for each cell, and the risk was that an army smears
rather than forming a front line.[^BLK52A] The research report stated the
method, and the measurement followed it.[^BLK52B]

**What was measured.** Four arrangements of two armies, in a world of 128 by 96
tiles, on an x86-64 development machine. The band that holds the middle 90
percent of the casualties is 1 tile wide at the tile, in every arrangement. It
runs from 1 to 30 tiles wide at the level 1 cell. The furthest casualty of a
cell resolution stood 36 tiles from the nearest enemy. Between 67 and 72 percent
of the casualties of a cell resolution stood on a tile that held no enemy at
all.[^BLK52C]

**The defect was put back.** Every arrangement resolved twice, once for each
tile and once for each cell, and the two runs differ in the granularity and in
nothing else.

**One thing the measurement also showed.** An arrangement two tiles deep reports
no smear at either granularity, so a single typical fixture would have closed
this row with the wrong answer.[^BLK52D]

**What follows.** The decision row is closed on Option B.[^BLK80A] A new row
holds the question this one did not ask: whether a running world ever produces a
tile that holds two factions.[^BLK52F]

### BLK-035 — Where does the documentation site publish, and who turns the publishing on?

**Resolved on 3 September 2026. The project owner chose the host.** He stated
it directly. The documentation site publishes to GitHub Pages, from the
workflow that builds it.[^BLK35D]

**The address follows from the host and from the name of this repository.**
This repository does not carry the name of its owner, so the site is a project
site rather than an owner site, and it answers on a path below the host name of
the owner. The site configuration states the address, and it is the one
declaration site of it.[^BLK35E] A record binds how the reference is built and
says nothing about where it goes.[^BLK35A] The documentation plan holds the
structure and the order of the work.[^BLK35B]

**The second question asked for an action, and an action is not information.**
This register holds work that is stopped for want of a fact. The fact is now
in the tree. What is left is one setting of the repository, which the host
reads rather than the workflow, and which the research report names as a
requirement of the workflow it documents.[^BLK35C] Only the project owner can
change it. One backlog item holds that action and the check that follows
it.[^BLK35F]

**The publishing job carries no switch of its own.** The hosting source
setting is the switch, and the deploy step reads it. A repository variable
beside that setting would be one fact in two places, with nothing that fails
when the two disagree. The decisions register holds the choice and the
reasoning.[^BLK35G]

**What the address bought, beyond the canonical link.** The builder writes the
address into the canonical link of every page and into the sitemap. It also
writes it into the page that the host serves for an address it does not hold.
That page carried links to the root of the domain while the address was
absent, and a project site does not answer at the root. One finding holds the
evidence.[^BLK35H]

### BLK-034 — May a unit build on ground another faction holds, and who may destroy an upgrade?

**Resolved on 3 September 2026, and one question moved to its own row.** The
project owner answered the building rule and both halves of the destruction
rule. The question about an upgrade whose ground changes hands is now BLK-036,
and it is open.

The engine stores an upgrade on a tile and lets a unit build it.[^BLK34A] It
asked nothing about who holds the tile.

**A faction builds only on ground it owns.** A unit may not build on ground
another faction holds. The holder column already names who holds each tile, so
the rule reads a value the engine stores.[^BLK34B]

**Anyone may destroy an upgrade.** Destruction is not restricted to the holder
of the ground. The owner stated that the reasons to destroy one vary, so the
engine permits the act and does not encode a motive.

**A unit-level destruction takes work, and a faction-level removal is
instant.** A unit that destroys an upgrade does work over several ticks, in the
same way that a unit that builds one does. The instant removal that exists
today stays, because some needs ask for it.

**The owner named the two as two verbs rather than as one verb with an
exception, and wrote the word "perhaps".** Destroy is the unit-level verb that
takes work. Reclaim is the faction-level verb that is instant. **Read the two
names as the owner's current thinking and not as a settled interface.** The
shape is decided; the names are not.

**A unit therefore needs a verb for destruction.** The engine has no such verb
today. It has an instant removal by address and nothing else. One item holds
that work.[^BLK34C]

### BLK-018 — How many groups found a world, and does every faction found one?

**Resolved.** Every faction founds one group. A run begins with one founding
for each faction the world holds.

The product record named one group and one group for each faction as the two
candidates and declined to choose between them.[^FOUND] The owner chose the
second. The two produce different games: one founding gives a run with one
society on an empty map, and one for each faction gives a run in which the
factions meet, on a tick that follows from how far apart the engine placed
them.

**A third shape was considered and deferred.** One faction that fractures into
several as the run proceeds. It was set aside because a fracture needs a rule
for why a society splits, and no record holds one. It is a later question, and
it does not block the founding rule: a run that founds one group for each
faction can still gain a fracture rule afterwards.

The engine takes the group size and the faction at the founding call, so a
caller founds one group or several. The run-level call founds one group for
each faction the world holds, and the demonstration calls it.[^FOUNDDIST]

**Two questions this answer does not settle, and the work needs both.** How
far apart two foundings must be, and whether a second founding may widen its
sample when it fails. The founding record refuses a sample that widens until
it succeeds, so the second question is a real constraint and not a
detail.[^SEP] Both are judgement rather than missing information, so they are
decisions and not blockers.

### BLK-013 — Maximum faction count

**Resolved.** The ceiling is 63. A faction is one bit in a 64-bit mask, and
one value is reserved for no faction. The transposed level 0 grid is
therefore affordable, a relation is one plane, and a presence set is one
word. The value is in the scale constants table.[^SCALE]

### BLK-014 — The world shape

**Resolved.** The world is a rhombus. A tile index is a raw axial pair, so no
tile access converts a coordinate. The cost falls on the viewer: a rhombus is
a parallelogram on the screen, so the viewer applies the skew and the engine
does not. The registry row for the tile index was written for an offset
index and now states the rhombus claim.[^SHAPE] The finding records the
correction.[^TILEIDX]

### BLK-001 — Tile scale, and therefore world extent

**Resolved.** The tile edge is 80 metres. The world is regional, about 330 km
across. Dwell is 2 and crossing-terrain capacity is 16, which stays inside
`u8`. The 12.5-second ordinary crossing holds. The project gives up the
continental extent, not the crossing time, so no per-tick rate re-bakes. The
parametric movement constants resolve against these values, and the values are
in the scale constants table.[^SCALE]

### BLK-002 — Name three archetypes you expect to exist

**Resolved.** Four fixed shapes exist, so entity storage keeps the archetype
machinery. The shapes are the soldier, the settlement, the living character,
and the tile upgrade. A record holds the claim and its consequences.[^SHAPES]

### BLK-003 — Is one million the whole population, or one million soldiers?

**Resolved.** One million is the whole population. Soldiers are a fraction of
it, and civilians are not separate entities on top of the million. Every
storage figure in the needs report and the agency report holds as written.

### BLK-004 — Target living character population

**Resolved.** The target is 50,000 living characters, inside the range the
character report recommends. The hard ceiling of 262,144 stays. The layer cost
is in the scale constants table, and it is derived by scaling, not
measured.[^SCALE]

### BLK-005 — Settlement count

**Resolved.** The world holds 5,000 settlements. This confirms the assumption
in the entity economy report, so every storage figure in that report holds.

### BLK-006 — Tile upgrade fraction

**Resolved.** Fewer than one tile in twenty carries an upgrade, which agrees
with the entity economy report estimate. Tile upgrades therefore use sparse
storage, not one slot for each tile. A read pays one indirection.

### BLK-012 — What does one tick represent in simulated time?

**Resolved by derivation, once BLK-001 answered what a tile represents.**

The tile edge is 80 metres, so a march rate of 24 km in a simulated day
crosses 300 tiles. Each tile costs a dwell of 2 ticks, so a simulated day is
600 ticks. One tick is therefore 2.4 simulated minutes.

The engine runs at 10 ticks for each second, so a simulated day passes in one
minute of real time.[^TIMING] A content author who writes a per-tick rate now
has the figure they need. The derived values are in the scale constants
table.[^SCALE]

The derivation assumes the march rate applies to ordinary ground at dwell 2.
It is arithmetic on constants the owner approved, not a new decision.

### BLK-008 — Upkeep per unit or per formation

**Resolved.** A unit is an individual soldier. The three-tier split makes it
affordable: individual decay, pooled consumption, aggregate decisions.

### BLK-009 — Tile capacity

**Resolved.** Eight units, stored as `u8`, with capacity as a data-driven
parameter. Crossing terrain raises it to 16.

### BLK-010 — Do formations exist as entities

**Resolved.** Formation membership is an ownership column plus a reverse
index. A formation is not a spatial region: a region is not stable under
movement, so a move order would change its own recipient set across frames.

### BLK-011 — Promoted soldier lineage

**Resolved.** A promoted soldier gets no invented ancestry. He founds a new
house, his kinship to everyone is zero, and he cannot inherit a title by
blood. A title holder may **appoint** him. His children inherit from him
normally.

[^MEASURED]: Target platform costs. `docs/reference/graviton-costs.md`
[^SPREAD]: Findings register, FND-223. `docs/FINDINGS.md`
[^FLOOR]: Findings register, FND-224. `docs/FINDINGS.md`
[^STALE]: Findings register, FND-330. `docs/FINDINGS.md`
[^SCALE]: Budgets and costs, the scale constants. `docs/reference/budgets.md`
[^SHAPES]: ADR-0066, entity storage holds four fixed shapes. `docs/adrs/accepted/adr-0066-entity-storage-holds-four-fixed-shapes.md`
[^SHAPE]: ADR Registry, row 0017. `docs/adrs/REGISTRY.md`
[^TILEIDX]: Findings register, FND-042. `docs/FINDINGS.md`
[^TIMING]: Movement timing note. `docs/research/movement-timing.md`
[^FOUND]: PRD-0012, a world starts small and grows. `docs/product/accepted/prd-0012-a-world-starts-small-and-grows.md`
[^SEP]: Decisions register, DEC-037. `docs/DECISIONS.md`
[^FOUNDDIST]: ADR-0076, a founding keeps a fixed distance from the foundings before it. `docs/adrs/accepted/adr-0076-a-founding-keeps-a-fixed-distance-from-the-foundings-before-it.md`
[^BLK34A]: ADR-0090, a tile upgrade is stored sparsely, as the difference from the generated world. `docs/adrs/draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md`
[^BLK34C]: Backlog item 0262, destroy an upgrade over several ticks. `docs/backlog/proposed/0262-destroy-an-upgrade-over-several-ticks.md`
[^BLK34B]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D2. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
[^BLK35A]: ADR-0107, the Python reference is generated from the compiled module, decisions D1 and D4. `docs/adrs/draft/adr-0107-the-python-reference-is-generated-from-the-compiled-module.md`
[^BLK35B]: Backlog item 0308, the documentation plan. `docs/backlog/refined/0308-the-documentation-plan.md`
[^BLK35C]: Research report 19, the documentation toolchain, section 3.6. `docs/research/reports/19-documentation-toolchain.md`
[^BLK35D]: The documentation site job. `.github/workflows/docs.yml`
[^BLK35E]: The configuration of the documentation site. `mkdocs.yml`
[^BLK35F]: Backlog item 0321, turn the documentation site publishing on. `docs/backlog/proposed/0321-turn-the-documentation-site-publishing-on.md`
[^BLK35G]: Decisions register, DEC-118. `docs/DECISIONS.md`
[^BLK35H]: Findings register, FND-334. `docs/FINDINGS.md`
[^BLK45A]: Research report 20, what the Python interface should be, section 1. `docs/research/reports/20-the-python-interface.md`
[^BLK45B]: ADR-0051, a selector is a lazy expression tree that Rust evaluates. `docs/adrs/accepted/adr-0051-a-selector-is-a-lazy-expression-tree.md`
[^BLK45C]: ADR-0040, Python is a control plane, not a data plane. `docs/adrs/draft/adr-0040-python-is-a-control-plane-not-a-data-plane.md`
[^BLK40A]: Backlog item 0330, repair the defects a fresh reader found in the published reference. `docs/backlog/complete/0330-repair-the-defects-a-fresh-reader-found.md`
[^BLK40B]: Findings register, FND-341. `docs/FINDINGS.md`
[^BLK50A]: PRD-0031, a god knows whose ground its people stand on. `docs/product/shaped/prd-0031-a-god-knows-whose-ground-its-people-stand-on.md`
[^BLK50B]: Research report 21, what a god needs from this engine, section 8. `docs/research/reports/21-what-a-god-needs.md`
[^BLK52A]: Research report 21, what a god needs from this engine, section 4. `docs/research/reports/21-what-a-god-needs.md`
[^BLK52B]: Research report 21, what a god needs from this engine, section 4.2. `docs/research/reports/21-what-a-god-needs.md`
[^BLK52C]: Findings register, FND-390. `docs/FINDINGS.md`
[^BLK52D]: Findings register, FND-391. `docs/FINDINGS.md`
[^BLK52F]: Blockers register, BLK-080, in this document.
[^BLK80A]: Decisions register, DEC-144. `docs/DECISIONS.md`
[^BLK80B]: Findings register, FND-392. `docs/FINDINGS.md`
[^BLK80C]: Findings register, FND-363. `docs/FINDINGS.md`

[^BLK110A]: Decisions register, DEC-200. `docs/DECISIONS.md`
[^BLK110B]: Research report 21, what a god needs from this engine. `docs/research/reports/21-what-a-god-needs.md`
[^BLK110C]: Decision Record Scope, section 4.5. `.claude/rules/adr-scope.md`
[^BLK111A]: Decisions register, DEC-202. `docs/DECISIONS.md`
