# Decisions (Register)

This document is a **register**. It lists the choices this project has made, the
options that were considered, and the outcome.

A decision needs **judgement**. The options are known and work can continue
under a stated assumption. Compare the blockers register, which lists work that
is stopped for want of information.

Numbers are permanent. Never reuse one. A closed decision keeps its row, with
the outcome recorded.

When a decision closes, and it corrected something the project believed, record
the correction in the findings register as well.


## Allocating a number

**Claim the next number below before you write the row.** Increment it in the
same change that adds the row.

A writer that numbers a row by reading the last row collides with any other
writer working at the same time. That happened, and it is recorded as
precedent.[^ALLOC]

**Next number: DEC-107**

## Open

### DEC-105 — Is the memory-for-speed trade now open, and does ADR-0088 still hold?

**Open. A reviewer owns it, with the project owner. It governs the frame budget
work.**

**The premise under several accepted decisions is measured and it was wrong.**
A world at the target scale holds 876 MB resident at 12 threads and peaks at
957 MB. The measured machine holds 32 GB. The engine uses about three percent
of the target machine and misses the frame budget by a factor of five.[^DEC105A]
[^BLK7]

**The split says where the room is.** The same world with no unit holds 456 MB,
so a population of one million adds 89 MB. A tile costs 27 bytes and a unit
costs 89 bytes. Nothing the engine holds is close to the machine.

**One accepted record spends time to save space, and says so.** It states that
a tile field is a generated base and a stored change, and that building the
field visits no tile and allocates nothing for it.[^DEC68D] That record is
sound as written. It was written when every cost figure in this project was
derived, and the derivation that made space the scarce quantity is the one the
benchmark contradicts.

**Option A. Open the trade, and treat the records one at a time.** State that
space is not the scarce resource at the target scale, and let each field
decide. This does not overturn any record by itself. It changes what a new
record must argue: a choice that spends time to save space now needs a reason
beyond the size of the tile count.

**Option B. Supersede the generated-base record.** Decide that a tile field is
stored, and keep generation only where a measurement shows it wins. This is the
larger move and it discards a property that the record earns: a build that
visits no tile. That property serves a product requirement that the build still
fails for other reasons.

**Option C. Leave it. Reduce memory first, then spend it.** Refuse the trade
until the engine meets the frame budget by other means. This keeps every record
as written and forgoes the cheapest measured wins.

**Recommendation: A.** B decides a general question from four specific cases,
and the four cases can be argued on their own merits under A. C is the option
that costs something real, because three of the four candidate trades cost under
70 MB each against 31 GB of unused memory.

**What A does not decide.** It does not say any particular field changes shape.
Each of those is a record or an item, and each must measure the trade rather
than assume it. A change that spends 67 MB and saves nothing is worse than no
change, because it also costs the reader an explanation.

**One candidate trade has now been measured, and it was refused.** A
tile-indexed exit direction costs an order of magnitude more than it saves, and
the read it proposes is itself slower than the read it replaces.[^DEC105B] The
figure this row states is a capacity figure. It says the machine has room and it
says nothing about cache, and the refused trade lost on cache. **Every remaining
candidate under this row must answer the cache question separately from the
capacity question.**

**Revisit when** a measurement shows a world at the target scale that holds
settlements and characters, since every memory figure the project holds is a
lower bound taken on a world that holds neither.[^BLK7]

### DEC-088 — What share of its own cell must a tile keep before the window gives room to a gap?

**Open. The window ships one half, and one half is the only value a sentence
forces.**

The window leaves a gap of one pixel between two tiles. A watcher reads the gap
as a black grid, because it shows the colour of the ground outside the world. A
tile `w` pixels across keeps `w - 1` pixels of colour in each direction, so the
gap takes `1 - ((w - 1) / w)^2` of the cell. The share grows as the tile
shrinks, and at the smallest tile the camera allows it reaches three quarters.
At that zoom the map is mostly grid, and a watcher reported it as a defect.

**What ships.** The window gives room to the gap only while the tile keeps at
least half of its own cell. That is the point at which a separator stops
covering more of the picture than the thing it separates, so the rule follows
from what a separator is. The bound sits near three and a half pixels. Below
it, the drawing leaves the gap out and the colour change from one tile to the
next is what separates them.

**Why the row is open.** Half is forced by a sentence. Any larger share is a
matter of taste, and taste is the project owner's. At a tile of four pixels the
gap passes the bound and still takes forty-four parts in a hundred of the cell,
and the picture at that width still reads as a grid to the eye that reported
the defect.

**Options.**

1. **Keep one half.** Derived, defensible, and stated in one sentence. It fixes
   the extreme and leaves the middle widths gridded.
2. **Keep three quarters.** The bound then sits near seven and a half pixels,
   so the gap appears only at the zooms where a tile is comfortably readable.
   The share is chosen, not derived.
3. **Draw no gap at any width.** The tiles then meet, and the ground reads as a
   continuous map. The tile grid stops being visible at all, which a watcher
   reading a tile-discrete world may want or may not.

**The recommendation is to ask the owner rather than to choose.** The register
already holds two repairs at this layer that were proposed from a rendered
picture and refuted by a count.[^DEC88A] A third value picked because one render
looked better would be the same mistake. The finding holds the measurement.[^DEC88B]

### DEC-086 — Does the agent protocol server ship in the installed package?

**Open. Engineering owns it. The recommendation is Option B.**

The server lives in the Python package, and the library that runs the protocol
sits in the development dependency group. An installed wheel therefore carries
the server module and not the thing that runs it. Importing the module from an
installed wheel fails on the import of the protocol library. Nothing fails at
build time and nothing fails at install time, so the first person to learn this
is a person who tried.

The question was left open when the server was built.[^DEC86A] It is sharper
now: the surface has grown, and a growth policy binds it.[^DEC86B] A module that grows is a module somebody will find and import.

**Option A. Move the protocol library into the runtime dependencies.** Every
installation of the engine then carries a protocol server it will never run.
The engine is a simulation core, and the people who install it are not the
people this server serves.

**Option B. Move the server out of the package, into the contributor tools.**
The package then holds the control plane and nothing else. The server sits
beside the checks and the scripts, where the other things that serve a
contributor sit. What is lost is the import path: a contributor runs the server
by a different route than a module of the package.

**Option C. Leave it, and make the failure say what happened.** The module
catches the import error and raises one that names the dependency group to
install. This is the smallest change and it keeps the shape that caused the
question.

**Recommendation: B.** The record that governs the surface states plainly that
it serves an agent working on this repository, and the product record names
that audience.[^DEC86B] [^DEC86C] A tool for a contributor belongs with the
contributor tools. Option A prices every installation for an audience that is
not installing. Option C documents the confusion rather than removing it.

**What follows either way.** Nothing is blocked. The server runs from the
repository today, which is the only place anybody runs it from.

### DEC-083 — Should a citation of a record name the directory the record sits in?

**Open. Engineering owns it. The recommendation is Option A.**

A record moves from one directory to another when a reviewer accepts it, and a
citation of that record names its path. Every citation written while the record
was a draft then names a path that does not resolve, and the citation check
fails on each one.[^DEC83A] Accepting a record is therefore a whole-tree sweep,
and the sweep grows with how well the record is cited. A finding holds the
evidence.[^DEC83B]

**Option A. Hold every record in one directory and let the registry hold the
status.** A path then never changes, and a citation never decays. The registry
already claims to be the only document that holds a status, and a directory
that also holds it is a second site for the same fact.[^DEC83C] What is lost is
the reading of a directory listing: a browser of the tree can no longer see
which records bind. The registry answers that, and the priority index answers
which drafts wait.

**Option B. Cite a record by number and let a check resolve the path.** The
body then names no directory. It needs a resolver in every check that reads a
citation, and it makes a path unreadable to a person who greps.

**Option C. Keep the move and pay the sweep.** It costs one sweep for each
acceptance, and the sweep touches source comments, so a worker who may not edit
source cannot accept a well-cited record.

**Recommendation: A.** It removes the sweep rather than automating it, and it
takes one declaration site away from the status of a record, which is the shape
this project meets most often. Option B pays a resolver in five checks. Option
C prices acceptance by how well a record is cited, which is the opposite of the
incentive the project wants.

**What follows either way.** Two records are ready and neither status has
moved, because a reviewer that may not edit source could not sweep them.[^DEC83D]

### DEC-081 — Does a caller that bounds a site read the composed capacity or the ground?

**Open. Engineering owns it. The recommendation is Option A.**

One function composes the capacity of the ground with the capacity a finished
upgrade gives, and returns the larger. Admission calls it, and so does the
public reader that reports what a tile holds.

Four callers do not. The position table bounds the positions of a site by the
ground alone. The founding seats a group over a disc by the ground alone. The
founding survey estimates the room of a place by the ground alone. The drawing
pass counts a tile as at its capacity by the ground alone, and paints an
over-full marker above that number, so a watcher reads a correctly filled made
way as over-full. A finding holds the evidence.[^DEC81A]

**The width is the reason it matters.** The row width of the position table is
a fold over the terrain capacity table. A made way states a capacity above
every value in that table, so a position table that followed the composition
would be asked to hold more positions than its row carries.

**Option A. Say that the four callers ask a different question, and fold both
tables into the width.** The positions of a site are the work it opens, and the
ground is the right bound for that. The founding seats conservatively, and
seating fewer than a tile holds is safe under the record that lets only
admission enforce a capacity.[^DEC81B] What must change is the fold: it claims
to report the largest number of units that can stand on a tile, and it walks
one of the two tables that state one. The drawing pass is the exception the
option must still answer, because a false marker is what a watcher sees.

**Option B. Make every caller compose.** One question then has one answer
everywhere. It widens the position row to the largest capacity any tile can
reach, which multiplies the storage of every site by that ratio, and no
measurement supports paying it.[^BLK7]

**Option C. Leave it, and remove the claim from the record.** The review
already removes the claim. Nothing else changes, and the fold keeps saying
something that is false of a roaded tile.

**Recommendation: A.** It costs one fold and one comment, it removes the false
statement rather than the true behaviour, and it leaves the position bound
where the movement record puts it. Option B pays the storage of every site for
a case that no run reaches. Option C leaves a helper whose own words are wrong,
and this project has recorded what that costs.

### DEC-079 — Does a whole cell that steps in one direction read as a crowd?

**Open. Engineering owns it. The recommendation is Option A.**

A record states that movement takes its direction from a per-cell field, and
never from a per-unit search over the neighbouring cells.[^DEC79A] One
consequence follows from the shape and not from a parameter. Every unit of one
cell that holds one option takes one direction, so a cell moves as a block.

The block edge is the level 1 block edge, which is a property of the partition
that level 1 and the derived unit structure share.[^DEC79B] A watcher
therefore sees a rectangle of ground step together, and the step of one
rectangle does not agree with the step of the next.

Nothing was run. Whether that reads as a migration or as a grid of marching
squares is a question about a screen, and only a run answers it.

**Option A. Accept the block, and settle the question with a run.** Build the
field, run the demonstration, and look. If a block reads as a block, open a
second choice against evidence rather than against argument.

**Option B. Give a share of the units the uniform draw instead.** A share that
deviates turns a hard edge into a soft one. The draw already exists, so the
cost is one comparison. The share is a value, and no measurement and no run
supports a value for it today.[^BLK7]

**Option C. Rank the neighbours of a tile rather than of a cell.** A field at
the tile pitch has no visible edge. It costs the tile count instead of the
cell count, and the record rejects the shape that pays the world for a
quantity that is correctly sampled at level 1.[^DEC79A]

**Recommendation: A.** Option B invents a parameter to repair a defect that
nobody has seen. Option C pays the tile count against the same defect. The
cheap move is to look first, and the item that builds the field carries the
question to whoever runs it.[^DEC79D]

**What the run showed. The row stays open, and the reason has changed.** The
field was built and the demonstration was run and rendered. A block of marching
squares was not visible, and neither was a crowd, because the demonstration
supplies no unit that the field moves as a group. Every unit chooses the option
that reads the share of a cell which admits a unit, and that value is a
property of the ground, so the field it steers by never changes and the
population walks to a local maximum and settles there.[^DEC79E] The founded
groups hold thirty people each, spread over several cells, so no cell ever
holds the crowd that the question is about.

The run did settle the smaller half of the question. The step is directed and
not a walk: over three hundred ticks of the demonstration world the mean
distance from the starting tile rose from 13 tiles under the uniform draw to
36, and the furthest unit from 40 tiles to 74. Both figures are one run each,
on a development machine.[^BLK7]

**The question needs a run that puts a crowd in one cell and gives it a reason
to move.** That needs a demonstration that makes a unit hungry, and an item
holds it.[^DEC79F] Nobody should choose Option B against the run above, because
the run did not reach the case.

### DEC-080 — Does a founding keep its fixed production rate once a carried load can reach a store?

**Open. Engineering owns it. The recommendation is Option A.**

A founding surveys the ground, and it sets the production rate of the new site
from the food that the survey found.[^ADR75] That rate is the only thing that
fills a store today, so what a settlement holds does not depend on what its
people do.

A separate item lets a unit give its carried load to the store of its home
site.[^DEC80B] After it, two mechanisms fill one store from one ground. The
survey reads the deposits of the disc around the place. The units gather from
those same deposits. The same food therefore reaches the store twice, once as
a rate and once as a load.

This is one quantity declared in two places, which is the shape this project
records as a recurring defect.[^DEC73B] Nothing fails when the two disagree,
because neither is wrong on its own.

**Option A. Keep the rate, and restate what it means.** The rate is the yield
of the ground that the site works without anybody walking to it. The carried
load is what a unit fetched from further away. The two are then different
quantities, and the survey stops being a promise about the deposits that units
can also gather.

**Option B. Drop the rate to zero once delivery exists.** The store then rises
only by what units fetched, which is the strongest form of the chain from the
ground to the store to the ration. It also starves a site whose people cannot
reach food in time, and no run has shown whether they can.

**Option C. Subtract the delivery from the rate.** The store then rises by a
fixed total whatever the units do. It restores the double count as a rule
instead of removing it, and it makes gathering pointless.

**Recommendation: A.** Option B changes the survival of every site against no
evidence, and this project has no run that says the units can feed themselves.
Option C is the double count with a subtraction on top. Option A costs one
sentence and leaves the choice open until a run shows what delivery alone
does.

### DEC-074 — How does the project find a value that nothing reads?

**Open. Engineering owns it. The recommendation is Option A.**

The engine writes a value into state, and no stage reads that value to decide
anything. The option column is the instance: movement reads whether a unit
chose and not what it chose.[^DEC74FND] The influence field and the tile stub
value are two more.[^DEC74NOTE]

**The rules the project already holds do not cover it.** Both look for an
absent caller, and this defect has one.[^DEC74SHAPE] So do the three repairs
that suggest themselves first. A rule that a person must be able to run the
feature passes, because the demonstration runs the choice pass on every tick.
A check that reports a public verb with no caller passes, because the pass and
the column both have callers. A rule that a backlog item names its caller
before it is refined passes, because the item named the right caller.

**Option A. State the test against the value.** Add a section to the testing
rule and one line to the impact review: for each value the work writes into
state, name the stage that reads it to decide something, and write a test that
changes the value and asserts that the decision changes. The falsification is
the one the rule already trusts: pin the value to a constant and watch the
suite stay green.[^DEC74TEST] The cost is one line in a review and one test for
each new column.

**Option B. Add a reachability check as a gate.** Derive the callers of every
public verb from the tree and fail when one has none outside the tests. It
needs a baseline, because a binding for a control plane nobody has written and
a reader that a test needs are both legitimately inert. The project carries one
baseline of this shape already, and its own text records what a baseline costs:
it can only shrink, and it does not shrink by itself.[^DEC74BASE] It would also
not catch the instance that opened this row.

**Option C. Derive a register of what the demonstration reaches.** Write the
set of verbs that the viewer and the bindings call to a reference table, derive
it from the tree, and fail when the table and the tree disagree. It reports and
does not judge, so it cries no wolf, and the number moves in the diff where a
reviewer sees it. It catches nothing by itself.

**Option D. File each instance as a backlog item.** The project does this
today. Four rows of the priority index name a capability with no caller, and
each sits under `Later`.[^DEC74PRI] A list of the instances is not a structural
change.

**Recommendation: A.** It is the only option that catches the instance that
opened the row, and it costs least. C is cheap and may be taken beside A, but
not instead of it. B buys a baseline that the project has already learned to
carry rather than repair. D is what happens if nothing is chosen.

**The first work to apply Option A found that the falsification is not
enough on its own.** The option column was pinned to a constant and the whole
suite of the new tests stayed green, because the fixture gave one answer for
the pinned value and for the value under test. The pin reached the consumer and
still proved nothing. A finding holds the case and the rule it adds: separate
the value under test from every other value the consumer could read, and assert
that the others do not give the answer under test.[^DEC74SEP] Option A stands.
The line it adds to the review must ask for the separation as well as for the
pin.

### DEC-071 — Does the world draw the sex of a character, or does content supply it?

**Open. The recommendation is that the world draws it, and the current code
draws it.**

The work that recorded descent needed a birth draw, because the item it
implemented requires a keyed one and asks for a test on each field of the
key.[^DEC71A] Nothing else in that item draws. The work therefore gave the
birth one drawn value, and chose the sex of the child, because the character
report carries the sex on the character row and the succession work reads
it.[^DEC71B]

**This is a concept that no product record asked for.** The product record for
descent excludes a heritable trait and excludes resemblance between a parent
and a child.[^DEC71C] The sex is neither of those, so the record does not
forbid it. It also does not ask for it.

The options are three.

1. **The world draws the sex at the birth.** This is what the code does. The
   draw keys on the mother and on the number of children she has borne on this
   tick, because the child holds no identity when the draw happens.[^KEYED] A
   character who founds a line already holds an identity, so that draw keys on
   the character.
2. **Content supplies the sex.** A caller that bears a child names it. The
   birth then makes no draw, and the item that requires a keyed birth draw has
   nothing to key.
3. **The world holds no sex at all.** The succession work then needs another
   filter, and the birth draw needs another subject.

**The recommendation is option one.** It gives the birth draw a real subject,
it matches the character report, and it costs one byte in the slot columns. A
reviewer who rejects it should say what else the birth draws, because the item
that governs this work requires that the birth draws something.

## Open

### DEC-073 — What does a kind of work fill, and what does a site want of it?

**Open. The engine holds a placeholder table, and content replaces it.**

A site opens a number of positions of each kind of work, and it decides that
number from what it wants of each kind less what it holds.[^DEC73A] Two values
feed that rule, and neither is an engineering choice.

The first is the map from a kind of work to the commodity that the work fills.
A kind of work is the resource kind that the work gathers, because the engine
already enumerates those and a second enumeration of one set is one fact in two
places.[^DEC73B] The store of a site holds commodities, and the commodity set
holds one entry today, so every kind of work maps onto that one entry. The map
is therefore correct and it carries no information.

The second is what a site wants of each kind before anybody tells it
otherwise. The engine declares a value so that a founded site opens positions
without a command, and the control plane replaces it with one command over a
set of sites.[^DEC73C]

**The options.**

1. Leave both in the engine as a table, and grow the commodity set when the
   economy needs one. The table is data the engine reads, never a function it
   calls, so this stays inside the rule that content supplies values and not
   comparators.[^DEC73D]
2. Move both into a content pipeline when one exists. The kernel reads the
   table either way, so the move costs no change to the pass.

**The recommendation is option 1 until a content pipeline lands, then option
2.** The values are content and the engine must not own them for ever. Nothing
is gained by inventing a pipeline for two tables.

**What settles it.** The economy needs more than one commodity. Until then the
map has one destination and any value in it is the same value.

### DEC-069 — Which representation does the result of a selector take?

**Open. The recommendation is a two-level form built for this storage, and the
choice waits for a measurement nobody has taken.**

A selector describes a set and the engine evaluates it.[^DEC69A] The evaluation
produces a result, and the record that governs the result requires one property
of it: a block that a predicate satisfies as a whole costs one entry, whatever
the block holds.[^DEC69B] That record states the property and names no form,
because no run separates the forms.[^BLK7]

**The options.**

1. A form built for this storage, in two levels. The upper level names a block
   of the storage, and the lower level is either a statement that the whole
   block matches or a bitmask over the block. The blocks are the blocks the
   storage and the summary pyramid already use, so a descent writes its answer
   in the shape a verb consumes and nothing converts.
2. A general compressed bitmap library. It arrives tested and needs no work. It
   splits the key space at a fixed width that has no relation to the storage, so
   a conversion sits between the descent and the verb.
3. A sorted list of indices, with a bitmask above a density threshold. This is
   the smallest amount of new code. The threshold is a value a measurement
   decides, and nobody has taken one.

**The recommendation is option 1.** The key space here is not arbitrary, and
option 2 treats it as if it were. The gain the record claims comes from the
agreement between the result and the storage, and only option 1 has that
agreement by construction.

**What holds it back.** Nothing holds the decision, and the work it governs is
not started. The research states plainly that the claim is a design argument
and not a measurement, and it names the benchmark that would settle it: an
intersection of two sparse sets, an intersection of two dense sets, a union of
many sets, and a full iteration with a column read.[^DEC69C]

**What is not open.** The property in the record is not one of the options. Any
form the project takes must state a whole block in one entry, must follow the
storage layout, and must yield one fixed order.[^DEC69B] A form that fails those
supersedes the record rather than choosing inside it. The interface never names
the form, so this choice does not reach a caller.

## Open

### DEC-068 — Should the dense column record be superseded, now that three tile fields sit outside it?

**Open. The recommendation is to supersede it, and to state the rule that
picks the shape rather than to state one shape. The recommendation is a weak
one, and the argument against it is below.**

ADR-0012 D2 states that a tile field is one contiguous array with one element
for each tile.[^DEC68A] Three accepted or drafted records now describe a tile
field that is not one. The ground is generated and stored nowhere.[^DEC68B]
The tile stock is generated, and only what was taken is stored.[^DEC68C] The
tile value field is a generated base and a stored change.[^DEC68D]

None of the three superseded ADR-0012, and none of them says whether it still
holds for the fields they do not cover. A contributor who adds a tile field
reads ADR-0012 first, because it is the general record, and writes a dense
column.

**D2 carries an exception clause, and it was checked.** The clause says that
the record fixes that a tile field is a column, and that it does not fix the
width of a column, the encoding of a boolean field, or the form a rare field
takes. It gives those to a separate record, and its footnote names which
one: the record that holds a narrow tile column, with bitplanes and sparse
side tables.[^DEC68G]

**The clause does not reach the three records.** It delegates the
representation of a column. A bitplane is a column one bit wide. A sparse
side table is a column for a field that most tiles lack. Each presupposes
that a column exists and asks what shape it takes. None of the three fields
has a column at all. The stored change of a tile value is a sparse side
table, and the clause covers that half, but the clause offers nothing for the
half that is generated and stored nowhere. The sentence the three records
leave is the first one, which the clause does not qualify.

**The strongest argument for leaving ADR-0012 alone is not the clause. It is
ADR-0068.** That record met D2 directly. It names the dense column as the
first of the two available shapes, states that the tile storage record
provides for it, and rejects it. It was accepted, it depends on ADR-0012, and
it superseded nothing. If a record may do that once, this one may do it
again, and the question below is whether three times is still an aside.

**The options.**

1. Supersede ADR-0012 D2 with a record that states the rule. A tile field
   whose base is a function of the seed and the tile index is generated. Every
   other tile field is a dense column. The new record carries both branches,
   and one record then answers the question a contributor asks.
2. Leave ADR-0012 D2 and let ADR-0088 narrow it. A reader who meets the
   disagreement is served by the later record, which resolves it. This is what
   the project did for the earlier disagreement between ADR-0018 and ADR-0012,
   where superseding over an aside was judged disproportionate.[^DEC68E]
3. Amend ADR-0012 D2 in place. This is not available. ADR-0012 is accepted and
   has dependents, and an accepted record changes by supersession.[^DEC68F]

**The recommendation is option 1, and it is not a strong one.** The precedent
for option 2 was a record that named a mechanism in an aside. D2 is not an
aside. It is the general record for tile storage, it states the shape of
every tile field in its first sentence, and three fields now contradict that
sentence. One exception is an exception. Three is the rule, stated nowhere.

**What is at stake, and it is not symmetric.** Superseding is expensive.
ADR-0012 has dependents, an accepted record changes only by supersession, and
every citing record must be re-read. ADR-0012 also holds a second decision
about the unit arena, which a supersession must carry forward or leave where
it is. The cost of option 2 is smaller and recurring: one wrong first answer
for every contributor who adds a tile field, and a foundational record whose
first sentence is false of three fields.

A reviewer who takes option 2 is not making a mistake. The argument for it is
in the paragraph above about ADR-0068, and it is a real one.

**Who decides.** A reviewer. The author of ADR-0088 is not one.

## Open

### DEC-095 — Is a strategy field derived from nothing each rebuild, or carried between frames?

**Open. Engineering owns it. It waits on DEC-067 and does not decide before
it.**

A strategy that names a place takes its direction from a field over the block
lattice, seeded at the destination.[^DEC95A] How far that field reaches depends
on where it starts each time.

**Derived from nothing.** The engine clears the field at every rebuild of level
1 and spreads it a fixed number of passes. The field is then a pure function of
level 0 and states no fact of its own, which is what the movement field
does.[^DEC95B] It reaches as far as its passes and no further, so a unit more
than that many blocks from its destination reads nothing and behaves as it does
today.

**Carried between frames.** A rebuild applies its passes to the field the last
rebuild left, so the reach grows until the field spans the world. This is what
the influence plane does, and it is why the writ of a ruler reaches further than
one tick of passes.[^DEC67ADR] It also stores a value that appears nowhere at
level 0.

**The two are not equally available.** Carrying is exactly the case that DEC-067
holds open: a plane above level 0 that is not a summary and never claims to
be.[^DEC95D] That row already blocks one record, and a strategy field would be
the second plane to raise it. **This row cannot close before that one**, and if
DEC-067 refuses the carried plane, this row has one option and not two.

**Option A. Derive from nothing, and accept the reach limit.** The field is a
projection like every other derived structure, no record has to change, and a
unit beyond the reach falls back to the behaviour it already has. The cost is
that a destination further than the pass count is unreachable, so the pass count
becomes the maximum useful world radius for a strategy.

**Option B. Carry, and take whatever DEC-067 decides for the influence plane.**
The reach grows to the whole world for a fixed cost per tick. The cost is a
second plane holding solver state above level 0, and a second instance of the
problem that has already stopped one record.

**Option C. Derive from nothing, and seed densely enough that the reach does not
matter.** A strategy that sends a unit to its nearest site seeds every site, and
sites are spread across the world, so the distance from any cell to the nearest
seed is bounded by how far apart sites are rather than by the size of the world.
The reach limit then binds only a strategy with one distant seed.

**Recommendation: C, falling back to A.** It needs no change to any record and
no answer from DEC-067, and the multi-source seeding it relies on is already how
the record states the mechanism.[^DEC95E] It does not serve a strategy with a
single far destination, and that case can wait for somebody to ask for it.

**No figure appears here.** The pass count, the reach it buys and the spacing of
sites are all quantities no measurement supports on the target
platform.[^BLK7]

**Revisit when** DEC-067 closes, or when a strategy with one distant destination
is asked for.
### DEC-094 — How does a clone get the pre-commit hook installed?

**Open. Engineering owns it. Nothing is blocked by it.**

The project now has a hook, and it is the first one.[^DEC94HOOK] It lives in a
checked-in directory rather than in `.git/hooks`, so it is versioned and a
change to it reaches whoever has installed it. Installing it is one line, and
a recipe runs that line.

**A hook nobody installs is worse than no hook.** It reads as protection that
is not there, and a reader who sees the directory reasonably assumes the
commits were checked. Nothing today tells a clone that the hook exists.

**The obvious remedy has a real cost, and it is not obvious which way it
falls.** Worktrees share the repository config, so arming the hook arms it for
every worktree of that clone at once. During a session with several workers in
several worktrees, one worker installing the hook changes what happens when
another worker commits. That is the reason this decision exists rather than
being made in passing.

**Option A. Leave it manual and documented.** The recipe exists and the
orientation names it. A contributor who wants the early warning takes it. The
gate is the enforcement either way, so nobody who skips it can merge a defect.

**Option B. Install it from the setup recipe.** A clone that runs the setup
step gets the hook. This is the smallest change that makes the hook real, and
it is also the one that arms every sibling worktree without saying so.

**Option C. Make the gate check that the hook is installed.** This turns a
convenience into a requirement and fails a gate for a local config, which is
the shape that trains people to disable a check.

**Recommendation: A until a defect reaches the gate that the hook would have
caught.** The hook saves minutes, and the gate loses nothing when the hook is
absent. B becomes right if the count of hook-catchable defects reaching the
gate stops being zero. C is the one to refuse.

**Revisit when** a merge defect reaches the gate on a branch whose author had
the recipe available and had not run it.

### DEC-092 — Can a caller ask anything about a character who has died?

**Open. A reviewer owns it. The recommendation is Option A.**

The record of descent outlives every character in it, and that is why it exists:
a parent edge cannot live in a structure keyed on a slot, because a watcher must
read a parent after that parent has died.[^DEC92A] [^DEC71C]

**No reader delivers it.** All four world-level readers take an entity, and an
entity that names a dead character resolves to nothing. The parents of a dead
character return nothing. Its ancestors and its descendants return an empty
list. Its relation to anybody returns zero.[^DEC92C]

**Zero already means two things, and a dead character makes three.** Two
characters with no common ancestor stand at zero, and so do two whose only
common ancestor is beyond the stated depth. A caller cannot tell any of the
three apart.

**The answer names things the caller cannot ask about.** The ancestor walk
returns descent identities, and an ancestor is usually dead. No world-level
reader takes a descent identity, so a caller holds an answer it can do nothing
with. That is the shape a finding already records from the control plane
side.[^F147]

**Option A. Add a reader keyed on a descent identity, beside each reader keyed
on an entity.** The entity reader keeps its meaning, which is a question about a
living character, and the new reader answers a question about a row of the
record. A caller that walked to an ancestor can then ask about it. The cost is
four more readers and a rule about which one a caller wants.

**Option B. Make the existing readers take either, and resolve internally.** One
reader for each question. The cost is that a caller cannot tell whether it asked
about somebody alive, and the readers that return zero or an empty list for a
dead identity would stop distinguishing dead from absent, which is the
distinction this row is about.

**Option C. Narrow the record's force to what the readers give.** Say that
descent outlives a character so that a living character's line stays correct
after its ancestors die, and not so that a watcher can ask about the dead.
Nothing changes in the code. The accepted product record has to agree, and it is
the thing that asks for the watcher.[^DEC71C]

**Recommendation: A.** The storage already carries the answer, the walk already
hands out the key, and the missing part is four functions. Option C is the
cheapest and it gives up the reason the storage is shaped as it is.

**Whichever option closes it, a backlog item follows**, and it needs a number
that this work did not hold.

**Revisit when** anything exposes a character to the control plane. No binding
exposes a character, a parent, a walk or a relation today, so the need is served
in Rust and not in Python, and the shape of the Python answer may decide this
one.

### DEC-091 — Does a group store its members, or does a member store its group?

**Open. A reviewer owns it. The recommendation is Option A. The author of
ADR-0065 is not the reviewer.**

The project holds two answers to one question, in two places, and nothing fails
when they disagree.

**The register says the member stores the group.** The blocker on formations is
resolved, and its text says formation membership is an ownership column plus a
reverse index. The unit carries the identity of its group, and the list of
members is derived from that column.[^DEC91A]

**The record says the group stores the members, and the code does that.**
ADR-0065 D1 gives the group one row of entries, each naming a member by its
whole identity or naming nobody.[^DEC91B] The position table implements it.

**The record calls these one answer.** Its own context quotes the register
correctly, and its D1 then says the register already resolved the case this way.
A review found otherwise.[^DEC91C]

**Option A. The group stores its members, and the register entry is
superseded.** A stored entry that names nobody is a state and not an absence, so
a group can say what it lacks, and what it lacks is what makes it ask for a
member. A reverse index derived from an ownership column cannot hold a vacancy,
because nothing owns an empty seat. This is what the code does today.

**Option B. The member stores its group, and the record changes direction.**
This is one fact in one place, and the project already chose it once for a
different relation: a unit carries the slot of the site it lives in, and a
reverse index from the site was refused because it adds a structure that the
spawn, the death and the home change must all maintain.[^DEC91D] Against that,
a vacancy needs somewhere else to live, and the resize pass needs to know what
a group lacks.

**Option C. Two directions on purpose, with a rule that says which relation
takes which.** Work held is a group-stored row because it has vacancies.
Residence is a member-stored column because it does not. The cost is that a
contributor must learn the rule, and a rule with two answers is the shape this
project meets most often.

**Recommendation: A**, and say so in both places. The vacancy argument is
decisive for a workforce, and it is the reason the code is shaped as it is. A
formation may still want the register's shape, and if it does, the answer is
Option C stated deliberately rather than Option A stated by accident.

**Closing this costs a register repair.** The blocker is resolved, so its text
carries the authority of a settled answer. A resolved row that states a
superseded answer is worse than an open one, because nothing marks it as stale.

**Revisit when** a formation exists. Today only the workforce case is built, so
the disagreement costs nothing yet and will cost the first person who builds a
formation from the register.

### DEC-093 — Does the forest kind or the hill kind carry a step multiplier of its own?

**Open. Engineering owns it. Nothing is blocked by it.**

The terrain table states a step multiplier for every kind of ground. The
mountain kind carries two and every other kind carries one. The mountain value
follows from the ratio of the two accepted crossing times.[^DEC93TIMES] No
accepted crossing time separates a forest tile or a hill tile from level
ground, so both carry the baseline.

**A value for either kind would be invented.** The project holds that a cost
figure is derived and never invented, and one blocker states that no
measurement exists on the target platform.[^BLK7] An intermediate
multiplier chosen because it looks reasonable would be a figure with no
derivation behind it, wearing the authority of the table.

**Option A. Leave both at the baseline.** Ground is level or it is a mountain,
and nothing between them costs more to cross.

**Option B. Accept a crossing time for one of the two kinds, then derive the
multiplier from it.** This is the route the mountain value took. It needs an
owner decision about how long a formation should take to cross a wood or a
ridge, and the multiplier follows.

**Option C. Derive both from the height field.** The generator already gives a
height, and the classifier already partitions it. A multiplier that rose with
the height would need no separate figure. It would also make the multiplier a
function rather than a table entry, which puts one crossing lever in code and
the other in content, and the project already rejected that split for this
value.

**Recommendation: A until a watcher asks for B.** Option A states no figure
that nobody derived, and it is honest about what the calibration covers. Option
C is the one to refuse: it reintroduces the content-and-code split that this
same question already settled.[^DEC93HOME]

**Revisit when** the project accepts a crossing time for ground between level
and mountain, or when a watcher reports that a wood reads as level ground.

### DEC-067 — Does a plane that carries the state of a solver belong above level 0?

**Open. Engineering owns it. It blocks the acceptance of ADR-0087.**

ADR-0022 D1 states that level 0 is the only source of truth, that every fact
about the world is stored once at level 0, and that a value which appears only
at level 1 is a defect.[^DEC67LEVEL] D2 states that a level 1 cell equals the
exact combination of the level 0 tiles it covers.

The influence field breaks the first sentence and makes no claim under the
second. It is a plane over the level 1 cell lattice. Its value at a cell is the
result of a relaxation that reads the neighbours of that cell, so it is not the
combination of the tiles the cell covers and it never claims to be. It also
carries from one tick to the next: a solve applies a fixed number of relaxation
passes to the field the last solve left, and that is what makes the writ of a
ruler reach further than the passes of one tick.[^DEC67ADR] Remove the source
and the field falls from the edge inward, which is the behaviour the project
already chose for a faction with no ruler.[^DEC67041]

**Why a solve from zero does not answer it.** A solve that starts at zero on
every tick is a pure function of level 0 and satisfies D1. It also reaches only
as far as its pass count, so a field with a small fixed pass count would cover
a few cells and no more. Raising the pass count until the field spans the world
puts the whole propagation into one tick, which is the cost the research
removes by keeping the plane.[^DEC67REPORT] A solve from zero also erases the
decay behaviour, because a field with no source is zero at once rather than
falling from the edge.

**Option A. Amend ADR-0022 D1 to name the case.** A record that supersedes it
says that a plane which is not a summary may hold solver state above level 0,
and states what such a plane must not do: it must not be read as a summary, and
no consumer may treat it as the exact combination of anything.

**Option B. Give the plane a level 0 home.** Store the field at level 0 and
summarise it into level 1. That restores D1 by making the level 1 value a
combination, and it multiplies the storage by the tiles that a cell covers.

**Option C. Leave D1 as it is and read it narrowly.** D1 governs the summary
pyramid, and an influence plane is not part of it. Nothing changes except the
reading.

**Recommendation: A.** Option B pays the tile count for a quantity the research
shows is correctly sampled at level 1, and it stores a field at a resolution no
consumer reads.[^DEC67REPORT] Option C leaves the reading in the head of a
reviewer, which is the failure this project has already recorded: a decision
that nobody wrote down becomes an assumption, and a later contributor trades it
away.[^DEC67DOD]

### DEC-062 — Do the settlement arena and the character arena reserve their storage too?

**Open. The recommendation is that both reserve, and that the character arena
reserves its target rather than its ceiling.**

The unit arena reserves its columns at construction, and a spawn past the
reservation gets a typed refusal.[^DEC59] The settlement arena and the
character arena hold the same shape and have not taken the same answer. Both
carry a capacity that refuses, and neither reserves any memory for it, so both
grow a column under a running simulation.

The character arena adds a question the unit arena does not have. It carries a
tier ceiling that is larger than the target the owner answered, so a
reservation has two candidate values rather than one.[^BLK4] The settlement
count is answered and has one candidate.[^BLK5]

**The options.**

1. Both arenas reserve at construction, in the way the unit arena does. The
   character arena reserves its target and keeps its ceiling as the refusal
   bound, so the ceiling stays the property of the identity layout and the
   target stays the property of the settings.
2. Both arenas reserve at construction, and the character arena reserves its
   ceiling. A run then never refuses a character below the ceiling, at the
   cost of reserving several times the target.
3. Neither reserves. The unit arena is the only one that does, because it is
   the only one whose population reaches a million.

**The recommendation is option 1.** A reservation that a run cannot exceed is
the property the product record states, and it states it of the storage the
world reserves rather than of the units alone.[^PRD12] Option 2 spends the
difference between the target and the ceiling on every world in the process,
and no run on the target platform says what that costs.[^BLK7] Option
3 leaves two arenas that reallocate inside a step, which is the shape the
closed row rejected for the third.

**What holds it back.** Nothing holds the choice. It was separated from the
closed row because that row asks about the unit arena alone, and answering
two arenas inside one row would put two claims in one place.

### DEC-057 — Does a site store its resident count, or read the one the engine keeps?

**Open. The recommendation is to read the count the engine already keeps.**

Housing needs the number of units that live at a site. A review of the housing
draft found that the engine answers the question today.[^FND128] The cohort
table holds one row for each faction at each site, and each row holds a
headcount derived from the home column of the soldier arena. The residents of a
site are the sum of its rows.

**The options.**

1. Read the resident count from the cohort table. Nothing new is stored. The
   table is rebuilt inside the consumption pass, so a reader between frames sees
   the count the frame settled on.
2. Store a per-site occupancy count and maintain it by the change, with a check
   that fails when it disagrees with the home column. This is what the housing
   draft states.
3. Store the count and retire the cohort headcount, so that one site holds the
   fact.

**The recommendation is option 1.** It adds no declaration site. Option 2 makes
three sites hold one fact, and a check between two copies does not guard
three.[^SHAPE1] Option 3 is coherent, but the cohort headcount is split by
faction and the pooled draw needs that split, so retiring it moves the cost
rather than removing it.

**What holds it back.** Very little. The table is already public, and the check
that compares it against the home column is already public. What option 1 needs
is a reader that sums the rows of one site, because the table splits the count
by faction. It needs no new store and no new check.

**What follows either way.** The housing draft states decision D3 as option 2,
and it must be rewritten against whatever this row decides.[^ADR81]

### DEC-044 — Should the default ration be above the decay?

**Open. The recommendation is to raise the default ration above the decay.**

The default need rule sets the ration equal to the decay, so a unit that
receives its whole ration holds the need it has.[^DEC34REF] A unit whose need
reached zero therefore holds at zero, even when its site feeds it again. Its
need never climbs back over the threshold, so its deficit never falls.[^FND089]

The consequence changed when a shortage gained an end. A deficit that only
rises reaches the bound, so every shortage that empties a need is fatal, and
the recovery rate of the rule reaches nothing.

Three options. Raise the ration above the decay, and a fed population climbs
back to a full need; this makes a fully served population drift up, which the
clamp at the top of a need absorbs. Feed the recovery from the ration a unit
received rather than from the need it holds, which changes a kernel. Leave the
rule and accept that a shortage which empties a need is fatal.

The values are content, and no content pipeline exists.[^DEC34REF] The row
therefore asks for a default, not for a rule.

**A refined item now waits on this row.** Growth adds mouths to the same
store, so under the default rule a site that grows into a shortage loses the
population it grew and does not recover. The item states its behavioural
tests against a rule it chooses rather than against the default, so the work
can proceed, and it records the fatal default as a test of its own.[^DEC44ITEM]


### DEC-038 — Which slot does the faction take in the founding draw key?

**Decided. The faction identity fills the entity slot, and the candidate
ordinal moves to the draw index.** The decision belongs in ADR-0076, which
replaces the slot assignment in ADR-0075 D2.[^ADR75D2] ADR-0075 keeps its
number and its status.

ADR-0075 D2 puts the candidate ordinal in the entity slot. That record was
written for one founding, and one founding has no actor, so the ordinal
occupied the slot that names the actor. With one founding for each faction,
the faction is the actor. The key then means what the names of its fields say.

**Why this is the cheap answer as well as the correct one.** Every faction
that keys alike draws alike. A separation rule against an identical sample
narrows the pool that every founding after the first draws from, for a reason
that belongs to the key and not to the world. Keying on the faction gives each
faction its own sample, so a founding after the first chooses from the whole
pool rather than from what the foundings before it left. The project buys that
with a field assignment rather than with a mechanism, and DEC-037 keeps the
fixed sample it chose.

**Corrected on 1 September 2026.** This paragraph said that an identical sample
seats the first faction and refuses the rest. It does not. The sample holds many
places, they stand far apart at this extent, and a founding after the first takes
a lower-ranked place that still keeps the distance. The work that implemented
this decision removed the faction from the key and counted the factions seated
at four, six, eight and twelve factions; every faction was seated every
time.[^FND106] The decision is unchanged and the reasoning above is narrowed to
what is true. A consequence test built on the stronger claim was written,
found to catch nothing, and deleted.

**What this opens, and does not decide.** A per-faction sample can later carry
a per-faction bias, so that factions value different ground. Nothing decides
that here.

**The test that holds it.** Change the faction and assert that the sample
changes. A draw keyed on the wrong field draws the same wrong value on every
thread and every run, so both determinism tests pass while the defect
stands.[^TESTKEY]

### DEC-039 — Is a household the same thing as a place to live?

**Decided. No. A dwelling is stored and a household is derived.**

A dwelling is a structure that stands on a tile and holds a capacity. A unit
carries the slot of the dwelling it lives in. A household is every unit that
carries one slot. Nothing stores a household, and nothing declares one.

This follows the rule that level 0 is the only truth and every level above it
is derived.[^LEVEL0] A stored roster of a household would be a second
declaration of where a person lives, and nothing would fail when the roster
and the slot disagreed.[^SHAPE1]

**What follows, and it is the reason to prefer this.** A household forms when
people share a roof and dissolves when they stop. A child who takes a dwelling
of their own splits a household by moving, not by a rule that splits
households. An inheritance is a transfer of a slot. None of that needs a
kinship rule, because a household is a fact about a place rather than a fact
about a family.

### DEC-040 — How does the decision of a ruler reach a unit?

**Decided. Through the world. The ruler writes a field, and a unit reads the
level 1 cell it already reads.** No unit asks who rules it, and nothing walks
from a unit to its faction.

A unit already gathers from level 1 planes, and those gathers are the reason a
decision is cheap.[^AGENCY] A ruler contributes a source term to the influence
field of its faction. The field carries the writ. How that field is stored is
a separate question, and a proposed record holds it.[^ADR60]

**Why this is the good answer.** The influence solve carries terrain
conductance, so influence flows around a mountain rather than through
it.[^DEC5REF] The writ of a ruler therefore runs strongly near the seat and
weakly far from it, and a mountain range obstructs it. A distant province is
less governed than a near one, and the engine spends nothing to achieve that,
because the field and its conductance already exist for another reason.

**The bound.** A ruler sets a field. A ruler does not command a unit. Nothing
here gives a ruler a per-unit order, and a per-unit order would be a data
plane in Python by another route.[^ORIENT]

### DEC-041 — What does a faction with no ruler do?

**Decided. Nothing special. An absent ruler is an absent source term.**

The engine holds no branch for a faction without a ruler, and no rule asks
whether a ruler exists. The influence field of that faction simply has nothing
writing into it. The solver runs its fixed iteration count either
way.[^FIXEDITER]

**What a reader sees.** The writ relaxes from the edge inward, because the
periphery is the part the field held least strongly. The far provinces stop
being governed first, and the seat is the last place to lose its hold. An
interregnum is drift rather than a state, and whoever takes the seat inherits
the drift.

**Why it is worth choosing.** The felt behaviour a crisis needs comes free
from a solver the project already runs. A branch on the absence of a ruler
would cost a check on every pass and would produce a worse result, because it
would make the loss of a ruler instant everywhere rather than gradual from the
edge.


### DEC-072 — What does a build cost, and how fast does one unit build?

**Open. Engineering recommends the values below and asks the owner to confirm
them.**

An upgrade asks for an amount of work before it is finished, and one builder
adds a fixed amount of it in one tick. Both are content. Neither is a cost
figure: they say what the world asks for, not what the engine spends.[^DEC72A]

**The recommendation.** One builder adds one unit of work in one tick. A made
way asks for eight, and worked ground asks for twenty-four. The two differ so
that the catalogue does not read as one number with two names.

**What the values must satisfy.** Every kind must ask for more work than one
builder adds in one tick, or nothing takes several ticks and the storage that
carries a build between ticks has no reason to exist. A test asserts this
against the whole catalogue rather than against a named kind.

**What is not decided.** No run on the target platform has priced a build, so
no value here is derived from one.[^BLK7] One tick is a fixed
span of simulated time, and the scale constants table holds it, so a content
author who wants a build to take a stated span can convert.[^SCALE]

**Why it is worth choosing.** A build that finishes in one tick is the defect
that makes the whole feature pointless, and it is the one the determinism
tests do not see.[^DEC72D]


## Decisions to apply at merge

These are mechanical. They do not need judgement, but they must not be
forgotten.

### DEC-009 — Renumber the colliding decision ranges

Reports 10, 11 and 12 all claim D51. Report 15 overlaps report 14 at D90 to
D95. Every decision number becomes local to its record, so the collision
disappears when the records are written.

### DEC-010 — The needs report must adopt the agency report's decision cost

The needs report's cohort decision line is 16.00 core-ms and is 92 percent of
its subsystem. Corrected, it is under 0.05 core-ms. See DEC-002.

### DEC-011 — Re-run the vector storage argument

The vector report computed against a stale copy of the character report. It
used 8-byte edges at mean degree 8, giving 33.6 MB at the ceiling. The real
figure is 168 MB. The storage argument for vectors is stronger than the report
concluded, and it called that argument its weakest.

## Closed

### DEC-097 — What does the project do with an accepted record whose imported fact a register made false?

**Closed. Reword it in place. A reviewer closed it, beside DEC-096.**

**The case.** A blocker narrowed, and seven accepted records still say that no
measurement exists on the target platform and that every cost figure in this
project is derived.[^BLK7] A benchmark now runs on the target platform and a
reference table holds measured rows.[^DEC97B] A sweep repaired 49 documents and
stopped at the seven, because the retcon window is shut on all of them and
rewording a sentence reads as an amendment.[^DEC96A]

**DEC-096 does not reach this case, and stretching it would cost more than it
saves.** That decision covers a consequence a record derived from its own claim.
These sentences derive from the condition of a register. If "derived" grows to
mean "anything a later fact made false", nothing is left of the amendment rule.

**The answer is a sibling rule and not a wider reading.** A record that states
the condition of the project imported that condition. The register owns it, the
register moved, and the record is false through no fault of its own. Reword the
sentence in place and keep the citation. The registry holds the rule beside the
other two.[^DEC96A]

**The test is the one the citation finding gives.**[^DEC96B] A reader decides
from the record's constraint. Every one of the seven states no figure, and the
scope rule forbids the figure whether or not a measurement exists.[^SCOPE] So
correcting the reason for an absence changes no decision that anybody makes from
the record.

**The boundary. A claim that rested on the condition is not an imported fact.**
Ask whether the record would have decided the other way if the register had said
the other thing. None of the seven would. Each one chose its option on
determinism, on the shape of the traffic, or on the structure of the storage,
and each says separately that it states no figure.

**The seven are not false in the same way, and a single replacement would make
some of them false again.** Every one holds a universal clause and a narrow one.
The universal clause says that every cost figure in this project is derived, and
it is now wrong in all seven. The narrow clause says that the quantity this
record needs is unmeasured, and it is still right in all seven, because the
blocker stayed open for a stage inside a step and for a world that holds
settlements.[^BLK7] Strike the universal clause. Keep the narrow one. Reword
each record against its own subject, which is the second recurring defect
shape.[^SWEEP]

**A record gains no figure in exchange.** The rule that keeps a measured figure
out of a record body does not depend on whether the measurement exists.[^SCOPE]

**What the repair costs.** One commit for each record, or one commit for the
group, that says which sentence changed, which register row moved, and why the
freeze did not apply. The footnote marker stays, because the blocker is narrowed
and not closed.


### DEC-096 — How does the project repair a stale consequence inside an accepted record?

**Closed. Option A. A reviewer closed it, in the review of ADR-0096.**

**The rule. A consequence that a record derived from its own claim may be
struck or corrected in place, in an accepted record, whatever depends on it.** A
claim, a force and a rejected alternative may not. The test is the one the
citation finding gives: does the edit change what a reader would decide?[^DEC96B]
The registry now holds this rule beside the citation rule, because a reader who
looks for one looks for the other in the same place.[^DEC96A]

**The case that raised it.** ADR-0064 D1 ends with a sentence that says the cost
of the choice pass is the option count times the population, and nothing
else.[^DEC96D] The sentence follows "therefore". It derives from the two
sentences before it, which say that a unit scores a fixed option set and reads
one cell. Nothing binds on it. A reviewer refuses a change because a unit
searched the world, and never because a cost clause named a quantity. The clause
is also the shape that the scope rule keeps out of a record body.[^SCOPE]

**So the sentence is a derived consequence and not a decision.** Strike it.
Supersession would cost a whole record, restate five decisions that nobody
disputes, and put every citation of the number under review.

**The boundary, so that nobody stretches it.** ADR-0064 D1 also says that every
unit scores every option in the set. That sentence states the selection, and not
the place the work runs. ADR-0064 says so in its own second force: the engine
precomputes the option values for each cell rather than for each unit. A later
record that moves where the scoring runs therefore leaves the sentence true. A
record that makes a unit select by a different rule falsifies a claim, and the
retcon window governs that.[^DEC96A]

**The strike waits for the record that stales the sentence.** ADR-0096 is a
draft, and this review returned it for correction.[^DEC96C] A draft binds
nobody, so it stales nothing. The sentence is struck in the commit that accepts
ADR-0096, and that commit says which sentence went and why the edit is a repair.

**What this settles beyond one sentence.** The project has met three cases: a
pointer that decays, a consequence that decays, and a claim that is wrong. The
first and the third were settled. This closes the gap between them.

### DEC-106 — How wide is a need bucket in the choice?

**Closed. The width matches the rate at which a need moves.** A measurement
settled it.

**Context.** The choice is decided for each cell and each bucket of need, and a
unit reads the answer of its bucket.[^D106A] **The width is the mechanism of that
decision and not a detail of it.** Unbucketed, the key is the exact need, and the
distinct keys in a cell are bounded by the cohorts standing in it rather than by
anything the engine holds. A review of the record that governs the pass returned
this defect and placed the choice of the width on the item that implemented
it.[^D106E] **The review gave a different reason, and the measurement refuted the
reason and not the conclusion.**[^D097F]

**What was missing.** No measurement existed of how many need values coexist in
one cell in a world that consumes. The measurement register said so in its own
words, and named the three things a fixture would need.[^D106C]

**The measurement now exists.** A world of 64 level 1 cells, 64 settlements, a
home for every unit, and about 75 units in the median cell, which is the density
the project states for the target scale. The findings register holds the table
and the fixture is in the tree.[^D097F]

**The answer, and it is not the one the author first chose.** The spread rises
to a peak while the stores empty and falls afterwards, because a cohort whose
share is below the decay falls to the floor and one whose share is above it rises
to the ceiling. At the peak, a width four times finer than the decay gives 41
distinct keys in a cell of 75 units. The matched width gives 17. **The finer
bucket separates two needs that the rule cannot separate inside a tick, so it
buys nothing and it is not free.**

**The outcome.** A bucket is the amount the default need rule takes off a unit in
one tick, so a unit crosses one bucket in one tick. A finer bucket resolves a
distinction the dynamics do not make. A coarser one lets a need change without
the bucket changing, so the choice lags the need.

**The decay is a parameter of the need rule, so the two are coupled.** A caller
who changes the decay and leaves the width alone has unmatched them. **That is
why the width is a parameter of the world and not a constant of a module**, and
the reference table holds the value a world starts with.[^D106B]

**What the outcome rests on, and what it does not.** It rests on a measured
distribution and on an argument about the dynamics. It does not rest on a cost
figure, and one blocker still governs every cost figure this project holds.[^BLK7]
Nobody has measured what the choice pass costs under this width on the target
platform.

**No gate can see this value.** The width was varied over most of its range and
no golden file moved, because no golden scenario reaches a case that the width
changes. A finding holds that, and it is the reason this row states its evidence
rather than pointing at a green pipeline.[^D097G]

### DEC-085 — Does a shipped product record constrain a design, or follow it?

**Closed. It follows. The project owner ruled it.**

The window outgrew a shipped product record. The record lists what the window
states, and a new design met four of those statements on demand rather than
continuously. The question was whether to bend the design to keep the record
literally true, or to change the design and amend the record.

**The ruling is to design for the best result and amend the record.** The owner
gave two reasons. The first is that the demonstration should look its best. The
second is general: a product record falls out of date as the product grows, and
amending it is the expected maintenance, not a failure.

**This does not make a record advisory.** The project already holds that a
record the code contradicts is worse than no record, because it lies.[^DEC85DOD]
The ruling says which of the two things changes when they disagree. It does not
say that they may disagree. A design that outgrows a record carries the
amendment as a deliverable of the same work.

**The scope is product records.** An architecture decision record is a
different instrument: it states a binding constraint and it is superseded by a
new record rather than amended.[^DEC68F] Nothing here changes that.

**Amend, or supersede.** Amend when the statements move and the need holds.
Supersede with a new number when the audience or the need itself changes. The
first case is a product that grew. The second is a different product.


### DEC-084 — Does a legend on a key satisfy a record that asks the window to name its colours?

**Closed. It does. The record asks that the window names its colours. It does
not ask that the naming is always on the glass.**

The window now draws cards and not a panel. The cards hold what changes moment
to moment, and every other number goes to a rendered picture and to the
terminal.[^DEC78A] Four statements of a shipped product record were at
risk.[^DEC84B] The record asks that the window name every colour it draws, that
it state where the person is looking, that it state the cost of the step and of
the drawing as two numbers while the run continues, and that it state how many
units the world holds beside how many it shows.

**The population pair stays on the glass.** Both numbers, both labelled. That
statement asks for something a watcher reads continuously, and it costs two
rows.

**The other three appear while a watcher holds a key.** One key reveals the
faction legend, the ground legend, the camera position and the cost. They are
the quantities a watcher checks occasionally rather than continuously, and one
mechanism serves all three.

**The options.**

1. **A legend on a key.** Keeps every statement true. Adds no state that lives
   between frames: the caller passes what the keyboard says, in the same way it
   passes the camera.
2. **Supersede the product record.** Write a record saying the window shows the
   moment and the inspection path holds the record. Honest, and it spends a
   shipped product record.
3. **A thin strip of swatches, always on.** Names the colours permanently in
   about twelve pixels, and states no count.

**Option one, on looks.** A legend that appears on a key is the window naming
its colours, so the record holds either way. The reason to take option one is
that it costs nothing on the glass: a layer that is hidden by default takes no
space from the map. Option three spends twelve pixels permanently to say what
the key says for free.

**The first version of this row gave a different reason, and it was wrong.**
It said option one was right because it needed no product decision. The project
owner then ruled the other way round: design for the best-looking
demonstration, and amend the product record where the design outgrows it. Their
words were "rule of cool", and they added that product records fall out of date
with growth and must be amended as necessary.[^DEC85REF]

That ruling reverses the order of the argument. Compliance is not the reason to
choose a layout. Looks are, and the record follows. Option one still wins,
because it also looks better, and the record was amended anyway to describe the
product as it now is.[^DEC84B]

**What would reverse this.** A watcher who cannot find the key. The window says
what the key does on a line that is always visible, so the test is whether a
person reads that line.


### DEC-075 — Does the cell summary carry one resource total, or one for each kind?

**Closed. Food alone. The summary carries one resource total.**

The engine holds three resource kinds: food, wood and stone.[^DEC75KIND] The
level 1 summary carried none of them, so a unit could not see any of them. The
question was whether to add one total or three.

**Three is the shape the rest of the engine uses.** The founding survey reads
all three. The ledger holds an entry for each kind. A summary that carried one
kind is the odd one out, and a later item that wants wood must widen the type
rather than fill a field.

**One is what a reader exists for.** The `forage` option reads food. Nothing
scores wood and nothing scores stone. A wood total and a stone total would be
written on every rebuild, hashed, and read by nobody, which is the shape the
project keeps meeting and has no rule that catches.[^DEC74SHAPE] [^DEC75SHAPE]
Each extra total also costs eight bytes on every cell of the level.

**The outcome.** The summary carries a food total. The type is a struct with
named fields, not an array indexed by the kind, so adding a wood total later is
one field and one line of the combine operation. That is a small change, and
paying for it now against no reader is the larger cost.

**What would reverse this.** An option row, a viewer panel, or a control-plane
verb that scores wood or stone. Add the total in the same change as the reader,
and not before it.
### DEC-077 — Which unit does the panel explain, when the viewer has no cursor?

**Closed. The unit nearest the middle of the window.**

The engine holds a verb that reports every option score, the value each option
read from the level 1 cell, the weight each option carried, and the option the
scores select.[^DEC77A] The panel must name one unit to ask it. The viewer has
no cursor and no selection, so nothing said which unit.

**The middle of the window is the pointer.** A watcher who wants another unit
scrolls until that unit is in the middle. This costs no new input handling, it
reuses the reading the panel already makes for the region under the crosshair,
and it needs no state that lives between frames.

**The drawing pass fixes it while it paints.** The pass already computes the
screen position of every unit it draws. It compares that position against the
middle of the canvas, which costs one comparison for each unit the pass was
already painting, so the panel starts no pass to find the unit.[^DEC77B] The
comparison is strict and the drawing order is fixed, so the same world and the
same camera name the same unit.

**The options rejected.**

1. **A cursor.** The viewer takes no pointer input today. A cursor is a real
   feature and it belongs to a watcher who wants to inspect rather than watch.
   It is not free, and this work needed a unit today.
2. **A fixed identity, chosen at the founding.** A watcher who scrolls away
   then reads a panel about a unit off the screen, and the panel says nothing
   about what is in front of them.
3. **The first unit the drawing pass paints.** Cheapest, and useless: it names
   a unit at the top corner of the window, which is not where a watcher is
   looking.

**Why this is not a record.** A later contributor could reasonably choose a
cursor, so the first test for a record passes. The second fails: changing this
costs one function in the viewer and no caller outside it. The third fails
too, because the reasoning fits in a doc comment beside the code.[^SCOPE1]

### DEC-078 — How does a panel longer than the window make room for a new section?

**Closed for this work. By the order of the sections. The reach mechanism
stays open.**

The panel is longer than the window and cuts at the foot, with a notice on the
last line. A separate item holds the reach problem, and it lists three
candidate mechanisms: a fold, a scroll, and a second column. It also asks
whether the order of the sections is the cheaper answer, and says that
question should be measured against the others.[^DEC78A]

**This work is that measurement.** Three sections were added to a panel that
already cut. The order was changed instead of adding a mechanism: every
section that reports the world as it stands now comes first, and the sections
that report the founding go last. The founding is history. It cost twelve rows
for each faction, and it filled the panel from the middle down.

**What the measurement showed.** Before the change, in the demonstration
window, the cut fell inside the second founding section. The region rows, the
ground legend and the cost rows were already below the edge, which is the loss
the reach item names. After the change the three new sections are above the
cut and the founding detail is below it. The demonstration window is also
taller, which reaches further down.

**The order is not a full answer, and the reach item is not closed by it.**
The ground legend and the cost rows are still below the edge. The order buys
one placement and it cannot buy a second: the next section forces the same
choice again, which the reach item already predicted. The order should now be
read as evidence in that item rather than as a solution.

**The recommendation for the reach item was the scroll.** The fold needs a name
and a state for each section, the second column needs a width the
demonstration window does not have, and the scroll needs one number that the
viewer already has a place for beside the camera.

**The project owner chose otherwise, and the recommendation is superseded.**
They asked for a heads-up overlay: the window shows only what changes moment to
moment, drawn over the map, and every other number moves to a rendered picture
and to the terminal. They accepted the trade that goes with it, which is the
best-looking demonstration and a worse window for diagnosis.

A scroll would have made thirteen sections reachable. The owner did not want
thirteen sections in the window at all, which is a different answer to a
different question, and it is theirs to give. The register row stays, because a
later contributor who reaches for a scroll should find the reasoning and the
outcome together rather than the reasoning alone.[^DEC78B]


### DEC-063 — Which verb puts a unit in the world from the control plane?

**Closed. Spawn is set-valued. The selector tree is the destination, and it is
not built now.**

The project owner decided this on 1 September 2026. He took none of the four
options below. He took a fifth path, which removes the question instead of
answering it: a verb that takes a set has no per-unit form for a caller to
repeat, so the rule about looping stops being a rule a caller can break.

**What the project already reserved is the answer, and nobody wrote it.** Four
registry rows sit reserved with no file: Python is a control plane, a declared
tier enforces the no-loop rule and the API refuses the loop, a selector is a
lazy expression tree that Rust evaluates, and a selector result may be a range
rather than an enumerated set.[^DEC63G] Under a selector, the control plane
never asks where to act. It says where to act as part of the command, a
predicate crosses once, and the engine evaluates it over the columns. That is
why a per-tile read was the wrong repair for the finding below: it answers the
question instead of removing it.[^F147]

**What changed now.** The spawn verb takes a collection of addresses and
returns an identity column in one crossing. The remove verb and the gather
order verb take a collection of identities. Each set is all or nothing, so a
caller never receives half a population and an error. The identity column is
the read side that DEC-060 already built, so the change adds no new mechanism.

**What did not change.** The read that resolves one identity stays singular. A
set form would have to choose between failing the whole call for one dead
identity and returning a value that stands for nothing, and that value is the
false answer the identity record forbids.[^DEC63C]

**The selector tree is not built.** Nothing needs it today, and building it
before a caller exists is the inert capability shape. A separate item holds the
work, and the records that govern it want an author who is not their
reviewer.[^DEC63D]

**Honesty about what the change buys.** The verb is set-valued at the boundary
and is still a loop inside the engine. Spawning has no cheaper whole-set
algorithm today. What the change removes is the crossing and the worked example,
not the per-unit cost. The principle that a set-valued command permits a cheaper
algorithm is satisfied in form here and not yet in substance.

**The options that were weighed, and the recommendation that was not taken.**

1. Expose the founding run, and withdraw the per-unit pair.
2. Expose the founding run as the verb a caller should reach for, and keep the
   per-unit pair.
3. Keep the per-unit pair alone, and add a set-valued verb when a caller wants
   many units.
4. Mark the per-unit pair as a test fixture and keep it out of the package.

The recommendation was option 2. Option 1 does not hold today, because a
founding never frees a slot that a later founding reuses: its only despawn is
the rollback of a founding that failed, and the one real death path is
starvation, which needs a large world, a long run and a verb that removes the
production rate the founding set.[^DEC63B] Option 3 left the engine's own
population path unreachable. Option 4 put one verb in two places.

**Why the tension was real.** A soldier is the mass tier, and the shape says
why: a soldier is one of a million, so no caller walks the population.[^DEC63E]
The per-unit verbs took a mass-tier entity one at a time, which is the case the
rule exists to protect. The evidence was in this work's own test, which spawned
a unit on every open tile and ordered each one. Reserved row 0043 holds the
claim that the API refuses the loop for a declared tier, and nothing implements
it, so prose was the only thing between a caller and a population built one
call at a time.[^DEC63F]

### DEC-059 — Does the world reserve unit storage, or grow it during a run?

**Closed. The world reserves the unit columns at construction, and a spawn past
the reservation gets a typed refusal.**

The project owner decided this on 1 September 2026, taking option 1.

The accepted product record for a founding states that the storage the world
reserves is sized for the target population, that it does not change during a
run, and that a run does not stop to grow.[^PRD12] The engine does the
opposite. The unit arena opens as many slots as the slot index holds, reserves
no memory for them, and appends one entry to each of its columns at each
spawn.[^FND135]

The record and the code disagree, so one of them changes.[^DOD3]

**The options.**

1. The world reserves the unit columns at construction, from a population the
   settings name. A spawn past the reservation gets a typed refusal. The record
   then describes the code.
2. The arena keeps growing, and the product record loses the statement. A run
   then reallocates a column under a running simulation, at a moment nobody
   chose.
3. The world reserves at construction and grows past the reservation as a
   fallback. This holds both behaviours, so it holds two answers to one
   question.

**The recommendation is option 1.** A reallocation of ten columns at one million
units is a cost that arrives inside a step, and no run on the target platform
says what it costs.[^BLK7] A reservation moves that cost to
construction, where a developer can see it. It also gives the refusal path a
real case, rather than one somebody adds after the first stall.

**What holds it back.** Nothing holds the choice. The value of the reservation
is a separate matter, and the target population is already answered.[^BLK3]

**How to state the reservation.** Take it from the world settings, in one
place. A second copy in the arena and in the settings is the defect shape this
project keeps meeting.[^SHAPE1]

A proposed item carries the work.[^ITEM0150]

### DEC-060 — How does Python read an event?

**Closed. The bindings return a column for each field.**

The project owner decided this on 1 September 2026, taking option 1. Option 3,
which keeps the log opaque and answers a question in Rust, was put to him as the
stricter reading of the control plane rule and he did not take it.

The bindings return the event log as raw bytes. The layout of an event lives in
the Rust source, which declares the field order, the field widths and the
padding.[^DEC60A] Python holds no description of it, so a Python reader must
repeat the layout. Two declaration sites hold one fact, and nothing fails when
they disagree.[^SHAPE1]

The consequence is real today. The agent-facing protocol server returns the
bytes and a digest of them, because it refuses to hold a second copy of the
layout. An agent can prove that two runs emitted the same log. It cannot see
which tile changed.[^DEC60C]

**The options.**

1. The bindings return a column for each field, in the way the tile column is
   returned today. Python never sees a byte offset. This costs a copy for each
   column.
2. The bindings return a description of the layout, derived from the type.
   Python decodes against the description. The description cannot disagree with
   the type, because it comes from it.
3. The log stays opaque, and the engine answers a question about it instead.
   Python asks; Rust reads.

**The recommendation is option 1.** It matches how the tile column already
crosses the boundary, so it adds no new mechanism. It keeps Python out of the
data plane, because Python receives an answer and does not walk a buffer. It
costs a copy, and the log of one step is small next to the world.

Option 3 fits the control plane rule best and costs the most, because each new
question is a new verb. Option 2 is the cheapest and puts a decoder in Python,
which is the shape the recommendation is trying to avoid.

**What every option must satisfy.** An event names an entity. The identity
packs an index and a generation, and the crate keeps its constructor private so
that no caller can assemble one from an index it chose.[^DEC60E] Whatever
crosses to Python must carry the generation and must round-trip back to the
same entity or fail. A bare index is not an identity. An agent that holds one
watches a unit, the unit dies, a new unit takes the slot, and the agent reports
on the wrong one with nothing failing. A fixed-point field crosses as its raw
integer, never as a float.[^DEC60F]

**What holds it back.** Nothing. Work continues on the bytes, and only a reader
is missing. A backlog item holds the work.[^DEC60D]

### DEC-056 — May a decision record cite a product requirement record?

**Closed. The rule is dropped.** A decision record may cite a product
requirement record for the need that made a choice hard. It must not take a
figure, a budget or a date from one.

The project owner decided this on 1 September 2026.

**What was believed.** A decision record cites no product requirement record.
The product guide stated it and the project orientation repeated it.[^FND129]

**Why it is dropped.** Nothing checked the rule and the project did not follow
it. ADR-0064, ADR-0067, ADR-0074 and ADR-0075 are accepted and each cites a
product record. The reason the rule gave is sound: a product direction changes
more often than a constraint does. The records that broke it do not show that
harm, because each cites a need that is stable and none quotes a figure from
one. A rule that nobody follows and nothing checks is worse than no rule,
because it hands a reviewer an objection the project will not support.

**What survives.** The part of the reason that has teeth. A decision must not
rest on a value that moves, and that is already the record scope rule, which
bans a budget, a figure and a date wherever they come from.[^SCOPE]

**What follows.** The two statements of the rule are removed, in the project
orientation and in the product guide. No record is repaired, because none was
in breach of a rule that no longer exists. No check is added. A refined backlog
item still cites both the records that govern it and the need it serves.

### DEC-055 — Does a period return one unit or the whole deposit?

**Open. The recommendation is one unit, and the engine holds that today.**

A recovery period must say what it is the period of. Two readings are
available, and the record for recovery states the shape and not the
reading.[^ADR80]

**The options.**

1. The period returns one unit of stock. A deposit that lost more takes longer
   to return, in proportion to what it lost.
2. The period returns the whole deposit. Every depleted deposit is whole again
   after one period, whatever it lost.

**The recommendation is option 1.** It makes heavy extraction cost more than
light extraction, which is the statement the product record asks the world to
be able to make.[^PRD18] It also keeps the arithmetic a whole-number division
with no reference to the generated stock of the tile.

Option 2 stays available and costs the same to compute. The row exists because
a reader of the code cannot tell which reading was chosen on purpose.

**What holds it back.** Nothing. Work continues under either reading, and only
the meaning of one parameter changes.

### DEC-054 — What period does each recovering kind take?

**Open. The recommendation is one simulated day for food and several for wood.**

The decision that food and wood recover and that stone does not is closed, and
it states the period as a parameter of the kind.[^DEC49REF] It states no value.
The engine now needs one, because a default rule set must hold something.

The engine carries a default in one place, and a caller replaces the whole rule
set. The value is therefore cheap to change and no second site holds it.

**The options.** Any pair of periods. The shape does not change with the value,
so this row asks for a judgement about how a run should feel and not for
information.

**The recommendation.** Food returns fast enough that a worked patch is worth
returning to within a run, and wood returns several times slower, because a
felled wood is a longer loss than a grazed field. The engine holds that pair
until the owner or a content pipeline replaces it.

**What holds it back.** Nothing. No measurement governs a content value, and no
blocker names one.

### DEC-049 — Which resource kinds recover, and how fast?

**Decided under delegated authority, 1 September 2026. Food and wood recover.
Stone does not.** This is option 1, as recommended below.

The project owner delegated the decision for this session and left the run
unattended. This row states that work continues under any of the options and
that only the value of a parameter changes, so deciding it is cheap to reverse
and blocking the work overnight was not. **The owner may reverse this without
supersession, because it is a parameter and not a constraint.**

The reasoning is the one the recommendation gives. It matches what a player
expects. It needs no value for stone. It makes the absent case a real case that
the engine carries from the first day, rather than one somebody adds later and
then discovers the shape does not hold.

The row as it stood follows.

The product record for a deposit that comes back states the recovery period as
a parameter of the resource kind and states no value.[^PRD18]

The world holds three kinds: food, wood and stone. Two of them are alive in
the ordinary meaning and one is not, so the shape of the answer is probably
one period for each kind, with one of the periods absent.

**The options.**

1. Food and wood recover. Stone does not. A period for each of the two.
2. Every kind recovers, with stone far slower than the others.
3. One period for the whole world, and no difference between the kinds.

**The recommendation is option 1.** It matches what a player expects, it needs
no value for stone, and it makes the absent case a real case that the engine
must carry from the first day rather than a case somebody adds later.

**How to state a period.** State it in simulated time and derive the tick
count. One tick is a fixed span of simulated time, and the register holds
that constant.[^SCALE] A period given in ticks alone would go stale if the
tick span ever moved.

**What holds it back.** Nothing. Work can start under option 1 with the two
periods as parameters, and the parameters carry the same name in the engine
and in this row.

### DEC-050 — Does a deposit that reached nothing recover?

**Decided under delegated authority, 1 September 2026. A deposit that reached
nothing recovers, in the same way as any other depleted deposit.** This is
option 1, as recommended below. **The owner may reverse this without
supersession.**

The same reasoning applies as for the row above: this row states that work
continues under either option and that only the value of a parameter changes.

Option 2 stays interesting and stays unchosen. Permanent ruin is a separate
need, and the product record names it as one on purpose, so it should arrive as
a need rather than as a side effect of this parameter. Option 3 reads a
neighbourhood, so its cost stops following the depleted set alone, and no
measurement exists on the target platform to justify that.[^BLK7]

The row as it stood follows.

The product record states this as a parameter and states no value.[^PRD18]

A deposit that units emptied is a different case from one they reduced. The
question is whether the world treats it as a wound that heals or as a thing
that is gone.

**The options.**

1. It recovers, in the same way as any other depleted deposit. Nothing in the
   world is ever permanently spent.
2. It never recovers. A deposit that reached nothing stays at nothing.
3. It recovers only when a neighbouring tile of the same kind still holds
   something.

**The recommendation is option 1 for now, and option 3 is a later need.**
Option 1 is the cheap one and the one that keeps the rule uniform. Option 2
gives a player a way to ruin ground, which is interesting, and the product
record names permanent ruin as a separate need on purpose. Option 3 reads a
neighbourhood, so its cost does not follow the depleted set alone, and this
row should not choose it without a measurement the project cannot take
today.[^BLK7]

**Why this is a decision and not a blocker.** Both options are known and work
continues under either. Only the value of one parameter changes.

### DEC-043 — What deficit ends a unit?

**Outcome. One default bound in the engine, carried by the need rule, until a
content pipeline exists.**

A unit that fails its draw builds a deficit. The deficit is a rate against a
bound, and the unit ends when it reaches the bound. The bound is content, in
the same way that the decay, the ration, the threshold and the recovery are
content.[^DEC34REF]

The engine holds the bound as a fifth value of the need rule and refuses a
value below zero. A caller replaces the whole rule, so the bound is a
parameter and no kernel holds one. The condition of a unit is read against the
bound in one place, so a watcher never compares a deficit against a rule of
its own.[^SHAPE1]

A bound written into the death pass was rejected, because it is the value a
world tunes most and a kernel is the hardest place to reach. A bound derived
from the threshold was rejected, because a value derived from another value
rots silently.[^FND015]

**Revisit when** a unit type table exists. The bound then belongs to the unit
type, with the rest of the rule.

### DEC-001 — The commodity split

**Outcome. 64 commodities exist. 16 take part in the transport solve. An
individual carries 8.** The remainder stay local to a settlement.

Two reports set different ceilings, and they bound different things.

| Report | Ceiling | Reason |
|---|---|---|
| Entity economy | 64 | A presence mask is one `u64`. 64 `i64` values fill exactly 8 cache lines. |
| Trade and flow | 16, hard limit 32 | Cache residency during the flow solve. |
| Individual agency | 4 to 8 | What one individual can carry. |

The three limits are compatible, because they bound existence, participation
and carriage separately. The project therefore takes all three rather than one
of them.

### DEC-002 — Do units make individual decisions?

**Outcome. Both tiers decide. An individual chooses where to go. A cohort
chooses what to buy.**

The needs report concluded that units do not decide, because a decision cost
400 nanoseconds and one million decisions would take four times the tick
budget. The agency report measured 4.1 nanoseconds. The gathers are sequential,
not random, because units are sorted by tile index and the fields are level-1
planes that stay in cache.

The corrected cost made this a design choice and not a budget one. The project
owner asked for individual experiences, and two tiers deliver them.

### DEC-003 — Do dead characters keep relation edges?

**Outcome. A dead character drops its relation edges.**

Retention costs 531 MB at 100,000 living characters and 1.39 GB at the ceiling.
The target is now 50,000 living characters, so retention costs roughly half the
first figure at the target.[^TARGET] That scaling is derived, not measured.

The cost still exceeds the whole living character layer. Dropping the edges
loses the ability to reason about a dead person's former ties, and the project
accepts that loss.

### DEC-004 — One fog layer or two

**Outcome. Two fog layers. Explored and visible stay separate.**

The fog report specifies the two as separate layers, and asked whether both are
needed. The answer depends on whether the game shows explored terrain
differently from currently visible terrain. It does, so the project keeps both.

### DEC-005 — Does the military influence plane need terrain conductance?

**Outcome. The plane includes terrain conductance.**

With conductance the solve costs 150 microseconds. Without it, 12 microseconds.
The difference is whether influence flows around mountains or through them.
Twelve times a small number is still a small number, and influence that ignores
terrain looks wrong.

### DEC-006 — Simulated or procedural weather

**Outcome. A procedural base with a simulated perturbation, if weather is
built.** Weather is not yet in scope.

Procedural weather is a deterministic function of position, tick and seed. It
needs no storage and no update cost, and it reproduces exactly, but it gives no
feedback. Simulated weather supports an orographic rain shadow and fire-driven
weather at real cost. The base carries the cheap part, and the perturbation
buys the feedback where the project needs it.

### DEC-007 — Retained or transient event log

**Outcome. The event log stays transient.**

Retention costs 3.2 MB for each frame, which is 11.5 GB for each minute. It
would buy rollback, time travel and audit. Events are already serialisable and
the apply step is pure, so retention stays additive. The project can take it
later at the same price.

### DEC-008 — Is a 50-second mountain crossing acceptable?

**Outcome. The project accepts the 50-second mountain crossing.**

The approved calibration puts an ordinary crossing at 12.5 seconds and a
mountain crossing at 50 seconds. The project owner rejected 50 seconds as the
ordinary case, and the recalibration relocated it to mountains. A mountain pass
must be a serious obstacle.

### DEC-012 — Does a product record cite a decision record?

**Outcome. No.** Recorded here because the reasoning is easy to lose.

A product record states a need. A decision record answers to a constraint. A
product direction changes more often than a constraint does, so a citation from
a decision record to a product record would place changing material inside a
historical document, which the scope rule forbids.

The join runs the other way and through one place only: a refined backlog item
names both the record that governs it and the product record it serves. A check
enforces that a product record contains no decision record citation.

**Revisit if.** The backlog stops being the only route from a need to the work,
or a reader cannot answer "which need does this record serve" and needs to.

### DEC-013 — Which toolchain version does the project pin?

**Outcome. The project pins the current toolchain version and records the
reason. The reason is that the project tracks a recent stable release.**

**The owner chose against the recommendation.** The recommendation asked the
project to state the property it needs from the toolchain first, and then to
pin the lowest version that provides it. The owner chose the simpler rule.

What follows is that the pin carries no property statement, so a later reader
cannot tell which toolchain behaviour the project depends on. The float ban
depends on toolchain behaviour today, because the reassociating methods do not
resolve on the current pin. A later toolchain may make them resolvable, and
therefore bannable by a lint rather than by a script. Whoever raises the pin
must check that case.

The record scope rule forbids a version in a record body, so the pin belongs
here and not in a record.

### DEC-014 — Which hash does the golden state test use?

**Outcome. The project confirms FNV-1a.**

The scaffolding chose it and nothing had ratified it. The choice is
load-bearing for determinism. The golden file is written by the hash, so a
change to the hash invalidates every stored hash.

The hash must be exact, order-sensitive, and stable across the platforms the
project builds on. FNV-1a meets all three. This decision earns a record when
someone writes one, because it is cheap to change now and expensive later.

### DEC-015 — The Python mutation gate is off

**Outcome. The gate is off, and the choice is reversible.** The gate was
removed rather than left failing, which the definition of done requires. The
Python package only re-exports the compiled module, so no mutant is covered and
the tool exits non-zero.

Turn the gate on when the Python package holds logic of its own. The testing
policy says how.

### DEC-016 — Type checking uses mypy, not pyright

**Outcome. Type checking uses mypy.** The project chose it to avoid a second
language runtime in continuous integration. Recorded because the choice was
made in passing and no record holds it.

### DEC-017 — Is a tile crossing time content-configurable, or fixed by the engine?

**Outcome. The terrain step multiplier is content. It sits in the terrain table
beside the terrain capacity.**

A crossing time depends on the terrain multiplier that scales the step cost of
a tile. The alternative was to fix the multiplier in engine code, which bounds
the dwell range at compile time.

The terrain capacity table is already content, and the capacity and the
multiplier describe the same tile. A split across content and code would put
one crossing's two levers in two places. A validated range in content buys the
same compile-time bound.

**Related, and now closed.** The mountain multiplier had no recorded value.
The accepted 50-second mountain crossing implies a multiplier of 2 against
ordinary ground.[^MOVETIME] The terrain table now states that value, the scale
constants table holds the row, and the row says the value is derived from the
two accepted crossing times.[^SCALE] The value is derived, not measured, and
nobody decided it directly.[^BLK7]

**What the terrain table does not answer.** The forest kind and the hill kind
carry the ordinary multiplier, because no accepted crossing time distinguishes
them from level ground. A separate row holds that open choice.[^DEC17NEXT]

### DEC-018 — Where does movement sit in the frame schedule?

**Outcome. Movement runs after the needs system and before the combat system.**

The frame schedule is static and known before the frame runs. The order of the
systems inside it was recorded nowhere. The movement design session proposed
this order, and nobody argued for another.

Movement reads what the needs system produces, so a unit acts in the same frame
on the need that raised it. Combat then sees the positions of this frame. The
read-after-write dependency between needs and movement is real.

**Why this is a register row and not a record.** Neither the needs system nor
the combat system exists. An order between systems that nobody has written is
an intent, and a record must not state an intent as a fact. Write the order
into the schedule when the schedule exists. Promote it to a record only if a
contributor could reasonably choose otherwise and the reasoning does not show
in the schedule itself.

### DEC-019 — How many admission passes does one frame run?

**Outcome. One frame runs two admission passes.** A unit may follow one
departure.

The admission step runs a fixed number of passes. Each pass admits what it can
against the room the previous pass confirmed. The engine never runs to a
fixpoint, because a fixpoint needs a convergence test and a solver in this
project runs a fixed count.[^FIXEDITER] The record states that the count is
content and declared before the frame runs. It states no value, and no value
follows from the tile scale.

One pass admits no chain. A unit cannot follow another out of a full tile in
the same frame, so a column of units on a road advances one unit for each
frame however long the column is. Each further pass admits a chain one unit
longer, and costs one more scan of the intents. A chain longer than the pass
count waits for the next frame, which is a delay and never a wrong answer.

Two passes admit the case that a person watching the world calls obviously
correct: a unit stepping into the tile a neighbour just left. Two is the
smallest count that admits any chain. More than two buys a longer chain, and
nobody has measured what a chain costs or how long a real column is. The value
is content, so raising it later costs nothing.

**Revisit when** a measurement exists of how often a chain longer than two
appears in a run, on the target platform. That measurement waits on the
benchmark harness.[^BLK7]

### DEC-020 — Must a spawn respect the tile capacity?

**Outcome. No. A spawn may over-fill a tile, and admission is the only rule
that enforces the capacity. The engine holds no dense per-tile count.**

**This row was decided twice, and the second answer stands.** The row first
recorded that a spawn refuses a tile at capacity and that the engine holds a
dense occupancy array. The owner reversed that before any code was written.
The first answer is kept here because a reader who finds the reversed record
needs to know the project held both positions and why it moved.

Admission never raises a tile above the capacity of its ground.[^ADR56D3] A
spawn reads the faction ceiling and the passability of the ground, and it does
not read the capacity. A caller may therefore place a hundred units on one
tile and the engine accepts it. The capacity is a rule that movement obeys,
not a property of the world at rest.

**The guarantee this gives is monotone, and that is the point.** Admission
computes the room of a target as the capacity less the occupancy, and it
saturates at zero. A tile that stands above its capacity therefore admits
nobody, while its units may still leave. An over-full tile drains and never
fills further. Crowding is a state the world can reach and then relieve,
rather than a state the engine refuses to represent.

**What the project gives up.** The capacity is not a world invariant, so no
single check can state it. A test may assert that no tile gains a unit beyond
its capacity. It may not assert that no tile is ever above one.

**What this costs nowhere.** The engine already behaves this way, so the
reversal changed no code. The dense array is not written, so the occupancy
storage question that the movement record defers stays deferred, and the
project pays no second declaration of where units stand.[^SHAPE1]

### DEC-021 — Where does a structural change made outside a frame get its barrier?

**Outcome. The step opens by rebuilding a stale bridge.**

The bridge rebuilds at the barrier, after the structural apply.[^ADR18D3]
Admission reads the occupancy of a target from the bridge, so the bridge must
describe the arena before the intents are admitted. A spawn or a despawn made
between two frames is a structural change that has passed no barrier. It leaves
the bridge stale, and the first step after it then has nothing to read.

The step gives the caller's changes the barrier they never had. It costs a
revision comparison when nothing changed. The rebuild at the end of the step
stays last and stays the barrier of that frame, so one operation has two call
sites.

Two options were rejected. A rebuild by the caller before it steps makes a
correct program depend on a convention that nothing enforces, and the error it
raises names the bridge rather than the spawn that caused it. A spawn that
maintains the bridge itself stops the bridge being derived at the barrier,
which the record forbids.

**Why this is a register row and not a record.** The two call sites are the
part a reviewer might object to, and the objection is about one function rather
than about a constraint on the project. Promote it to a record if a second
structural apply lands inside the frame, because the ordering between the two
is then a real decision.

The ordering of the barrier itself is settled and enforced. A test reads it
from outside: a rebuild that ran before the structural apply leaves the derived
structure stale when the step ends, and four tests fail on that.[^ITEM0030]

### DEC-022 — May the viewer make the engine wait?

**Outcome. The project amends the product record now. It separates the two
rates when a caller needs them apart.**

The product record for the first renderable example states that the window
keeps up with the engine, or drops what it cannot draw and reports the drop,
and that it never makes the engine wait. It also states that the engine costs
the same when a viewer is attached.

The viewer record decides the opposite for now. One loop steps and then draws,
so the drawing rate and the tick rate are one number. Its consequences section
says plainly that a slow drawing slows the simulation in the demonstration
binary, and that this is acceptable for a demonstration.[^ADR67D4] The binary
also caps its own frame rate, so the engine waits on every frame that finishes
early. Nothing drops a frame and nothing reports a drop. The two block counts
the panel shows count empty spatial blocks, not dropped frames.

**This was a real contradiction, not a defect in either document.** The viewer
record knew it was choosing against the product record, and it named what would
supersede the choice.

The amendment makes the statement about waiting a statement about the engine
when a viewer is attached through a snapshot, and it excludes the demonstration
by name. The product record then describes what the project built, and it can
reach `Shipped`.

The rejected option was to separate the two rates now. The engine would run on
its own thread and publish a frame the viewer reads. That needs the snapshot
record, which does not exist. Writing the snapshot record to serve a
demonstration is the wrong order, which the viewer record already argues. Take
it when a person must watch a world that steps faster than a screen refreshes.

### DEC-023 — What rate does a unit gather at?

**Outcome. One rate, high against the stock of a tile, until a content pipeline
exists. Then a rate that the unit type carries.**

A unit told to gather takes an amount from its tile in each step. The engine
holds one rate for every unit and every ground, and the value is
content.[^ADR73D1]

The value interacts with the stock tables. A rate far below the stock of a tile
makes a deposit last many frames, and two units on one deposit then never
contend. A rate at or above the stock empties a deposit in one frame, so the
contested case is ordinary and every test meets it.

A high rate makes the contested case the normal case, so every scenario
exercises the resolve. A deposit lasts one frame, which makes gathering feel
instant. A low rate is the better game and it reads better, but it makes the
case this subsystem exists for rare, which is the wrong trade before the
subsystem has a second reader.

A rate on the unit type is the shape the project ends at, because a unit type
is data.[^ORIENT] It needs a unit type table, and none exists.

### DEC-030 — Is the founding the only way to people a world?

**Outcome. It is one of two ways.** The founding is a call a caller makes. The
direct spawn stays as it is, and every fixture that spawns a unit keeps
working.

The alternative was to make the founding the only entry, and to remove the
direct spawn or to hide it. That was rejected for three reasons.

The founding is built on the direct spawn. A founding that placed a unit by
some other route would be a second write path into one arena, which is the
first recurring defect shape.[^SHAPE1]

A test needs to place a unit where the test chooses. A fixture that must ask
the engine where to put its units cannot build the extreme the assertion needs,
and a fixture that supplies no extreme measures itself.[^TEST2A]

Every golden file would be re-recorded, and a re-recorded golden file proves
nothing about the change that caused it. A new scenario for a founded world is
the cheaper and the stronger test, because the old files stay as the control.

**What follows.** No existing fixture changes and no existing golden file
moves. The founding adds one scenario and one golden file. The demonstration
binary founds a run rather than spawning a full world, because the
demonstration is what a watcher looks at.

### DEC-031 — What does a founding score read?

**Outcome. It reads the ground and the stock the ground carries.**

The founding happens before the first frame, so the only properties that exist
are the ones the seed fixes. The score therefore reads the terrain kind of a
place, the food and the wood and the stone within a small radius of it, how
much of that radius admits a unit, and whether open water touches it.

The product record says plainly that it does not decide which properties make a
place good, and it names water, food, high ground and reachable ground as
candidates.[^PRD12] This row records the set that was taken, so that a later
change to it is a change to something written down.

**What is not in the score.** Nothing that a run produces. No faction holding,
no neighbour settlement, no route. Each of those is a property of a world that
has stepped, and the founding runs before any of them exists.

**Revisit when** a second founding exists. A group that splits off from a
settlement chooses against a world that has stepped, and the set above is then
too small.

### DEC-032 — What layout does the character arena hold?

**Outcome. The character arena keeps struct-of-arrays.** The trait record holds
array-of-structs, and it is a separate structure that nothing has written.

The character arena holds its columns as struct-of-arrays, in the same style as
the soldier arena and the settlement arena. A register row said the character
tier wants array-of-structs, and it gave a difference of twelve cache lines
against one for a random graph gather.[^FND022]

**That premise is misattributed, and the finding records the
correction.**[^FND072] The twelve-against-one figure belongs to the vector
report and it covers the personality influence pass over a separate 64-byte
trait record.[^REP18] The character report covers descent and succession, and
it recommends struct-of-arrays for the character row.[^REP14] The two reports
do not conflict, because they describe two structures.

Every descent and succession kernel is a column pass: a map to a mask and a
compaction scan for eligibility, a map to a key tuple and a sort for ranking, a
counting sort for the child list, and a map over a contiguous range for a cadet
split.[^REP14] The two operations that gather at random, the lowest common
ancestor walk and the kinship recursion, read two or three columns for each
node.

Array-of-structs would charge every column pass a full row read to serve a
gather that reads two columns. It would also break the zero-copy column view
that the Python control plane takes for each shape. A hybrid, with the hot
descent fields in one row, declares one value at two sites unless the split is
exact, and the split cannot be exact while nothing has written the
pass.[^SHAPE1]

A gather benchmark on a development machine measured the crossover as a
function of the column count, and the crossover sits well above the two columns
that descent reads. The figures are in the commit body, because the machine is
not the target and a measured figure decays.[^BLK7]

**The record is written, and this row closes.** When this row was opened, the
scope rule refused a record: the arena held five columns and no parent edge,
so a later change was cheap and the record would have stated an intent as a
fact.[^SCOPE1] The descent columns now exist and a pass reads them, so the
claim points at something. ADR-0021 sits on the reserved row and holds the
claim that a layout claim names one structure and one pass, and never a
tier.[^REG21] [^DEC32ADR] Its status is `Draft`. A reviewer sets `Accepted`.

The record states the claim in a stronger form than this row did. This row
said layout follows the access pattern, which is true and which the misread
register row also said.[^FND022] The record says what a layout claim must name
in order to be checkable, because naming the tier is what caused the
error.[^FND072]

**The columns went to the record of descent and not to the character arena.**
The backlog item said the arena. A separate finding records why the record is
the right home and what to read into an item that names a structure.[^DEC32FND]

### DEC-033 — Does the project keep a performance path for the development machine?

**Outcome. The project keeps two performance paths, and they have different
standing.** The target owns every claim about how the engine performs. The
development machine owns a local gate-time budget, and that budget is never
evidence about the target.

Most cost figures in this project are derived and belong to the target, and one
open blocker says which of them a run has measured.[^BLK7] The rule that
follows is that a measurement taken on a development machine proves nothing
about the target, because the two differ in cache line size.

That rule is correct and it is not the whole picture. Development happens on
the development machine. The gate suite runs there many times a day, and its
cost is paid there and nowhere else. No rule owned that cost, so it grew
without anything noticing. The golden state hash test is the live instance: it
grew as each subsystem entered the state hash, and it is now the slowest gate
in a debug build.

The two quantities are not the same kind of thing. How fast the engine runs at
the target scale is a property of the engine, and the target owns it. How long
a contributor waits for the gates is a property of the development loop, and
the machine that runs it owns that. To measure both and treat them alike was
rejected, because the cache line difference makes it unsound, and that is the
mistake the platform rule exists to prevent.

**What follows.** A development budget must state that it is local and must
never be cited as evidence about the target. The gate cost gets a stated budget
and a home in the reference tables, and a change that exceeds it is visible
rather than silent. The work is filed.[^ITEM0098] The blocker stays open,
because it is about the target and this decision does not touch it.

### DEC-034 — What does a unit need, and how fast?

**Outcome. One default rule in the engine, until a content pipeline exists.
Then a rule that the unit type carries.**

A unit carries a need that falls at an interval, and it draws a ration against
the store of the site it belongs to. Four values govern the rule: the decay of
the need, the ration, the threshold below which a unit is in deficit, and the
rate at which the deficit recovers. Every one of them is content.[^ADR73D1]

The engine holds the four as one rule and refuses a rate below zero. The rule
is a parameter, so a caller replaces it without touching a kernel.

The values interact. The ration equals the decay today, so a unit that receives
its whole ration holds its need level. Any other relation between the two makes
a fully served population drift up or down, which is a design choice and not an
engine constraint.

The engine default is what the engine does today. The demonstration runs, and
every test states the case it needs by choosing the production of a site rather
than the rule. To give the rule to the control plane for each world moves the
choice without settling it. A rule on the unit type is the shape the project
ends at, because a unit type is data.[^ORIENT] It needs a unit type table, and
none exists.

### DEC-035 — Does a settlement need a ground rule of its own?

**Outcome. The tile kind carries a second suitability property. A settlement
reads its own rule.**

**The owner chose against the assumption in force.** Work proceeded on the
assumption of one ground property, and item 0092 is written against the
passability reader. The second property is a widening of that item rather than
a rewrite, but the item and the tile kind both need the new value.

Item 0092 refuses a settlement the ground that cannot carry one, and it reads
the passability of a tile to do it. Passability answers whether a unit may
stand on a tile. It does not answer whether a place may be built there. The two
questions come apart on ground a unit crosses and a settlement cannot occupy. A
mountain is the obvious case. The project had one ground property, so the two
answers were the same by accident rather than by decision.

What follows is that every new ground kind is priced at two values instead of
one. The project accepts that price, because the mountain case is real. Item
0092 states the question as out of scope and settles nothing, so the question
would otherwise live only in an item body.[^ITEM0092]

### DEC-036 — How does a unit find the units of a lost site?

**Outcome. The engine keeps the scan.**

A unit carries the slot of the site it belongs to. When a settlement is
destroyed, every home naming that slot must be cleared, or the settlement
founded next in that slot feeds a population it never took. The engine clears
them by scanning every unit.[^ADR14D7]

The scan is correct and it is the whole population for one destruction. No
figure is stated here, because no run has priced the scan.[^BLK7] A destruction is rare, and the scan needs no second structure
to maintain. It is one fact in one place.[^SHAPE1]

A reverse index from a site to its units would touch only the units that named
the site. It adds a structure that the spawn, the death and the home change
must all maintain, and nothing fails when it disagrees with the home column.

**Revisit when** a rule destroys sites in bulk rather than one at a time.

### DEC-051 — Which slot of the draw key holds the faction?

**Outcome. The frame slot holds the faction.** The candidate ordinal keeps the
entity slot, and the axis keeps the draw slot.[^ADR75D2] A record holds the
decision.[^ADR76]

A founding happens before the first frame, so the frame slot carried a
constant. It now carries the faction, and two factions read two samples. The
key keeps the shape the determinism record fixes, and no slot carries two
meanings.[^KEYED]

Two options were rejected. **An amendment to the founding record** would state
the key for several foundings inside a record written for one. That record is
accepted and it is still true, and an accepted record changes only by
supersession. **A fold of the faction and the ordinal into the entity slot**
puts two values in one slot, so a later change to either one can collide with
the other, and nothing would fail when it did.[^SHAPE1]


### DEC-037 — How far apart are two foundings, and may a founding widen its sample?

**Outcome. A fixed minimum separation, and a fixed sample.** A founding that
finds no admissible place fails, and a failed founding is a correct outcome.

Every faction founds one group.[^BLK18] That answer needs two rules the project
did not have, and item 0094 could not be refined without them.[^ITEM0094]

**The separation.** Two groups drawn from one bounded sample can land on one
tile, or within one disc of each other. Whether a second founding refuses a
place near the first, and by how much, was a rule no record held. A world of
sixty-three factions founding into one region makes the question sharper than a
world of four.

**The sample.** The founding record refuses a sample that widens until it
succeeds, because a sample that grows on failure has no bound.[^ADR75] A second
founding that must avoid the first fails more often than the first did. The
fixed sample keeps the bound and accepts the failure.

Two options were rejected. A separation that scales with the faction count
seats everybody in a crowded world, but it introduces a second value derived
from the faction count, which is a declaration site to watch.[^SHAPE1] A
partition of the world into one region for each faction seats every faction by
construction, but it decides map structure, which is a larger claim than a
founding rule and would need its own record.

The chosen option adds no mechanism, and the product record already states that
a failed founding is correct.[^PRD12]

## References

[^DEC73A]: ADR-0065, a group is a site membership, not a region, decision D3. `docs/adrs/draft/adr-0065-a-group-is-a-site-membership-not-a-region.md`
[^DEC73B]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
[^DEC73C]: ADR-0040, Python is a control plane, not a data plane, decision D1. `docs/adrs/draft/adr-0040-python-is-a-control-plane-not-a-data-plane.md`
[^DEC73D]: ADR-0007, content supplies a key vector, never a comparator, decision D3. `docs/adrs/accepted/adr-0007-content-supplies-a-key-vector-never-a-comparator.md`

[^ADR75D2]: ADR-0075, the founding choice reads a bounded sample of the world, decision D2. `docs/adrs/accepted/adr-0075-the-founding-choice-reads-a-bounded-sample-of-the-world.md`
[^ADR76]: ADR-0076, a founding keeps a fixed distance from the foundings before it, decision D3. `docs/adrs/accepted/adr-0076-a-founding-keeps-a-fixed-distance-from-the-foundings-before-it.md`
[^KEYED]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
[^TESTKEY]: Testing rules, section 2. `.claude/rules/testing.md`
[^LEVEL0]: ADR-0022, level 0 is the only truth and every level above it is derived. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
[^AGENCY]: Individual agency and occupations, the decision cost, and DEC-002 above. `docs/research/reports/16-individual-agency-and-occupations.md`
[^ADR60]: ADR Registry, proposed row 0060, an influence map is stored as a shared basis. `docs/adrs/REGISTRY.md`
[^DEC5REF]: See DEC-005 in this document.
[^PRD18]: Product record PRD-0018, a depleted deposit comes back. `docs/product/shaped/prd-0018-a-depleted-deposit-comes-back.md`
[^SCALE]: Budgets and costs, the scale constants. `docs/reference/budgets.md`

[^D106A]: ADR-0098, the choice is decided for each cell and each bucket of need, decision D1. `docs/adrs/draft/adr-0098-the-choice-is-decided-for-each-cell-and-each-bucket-of-need.md`
[^D106B]: Budgets and costs, the choice pass. `docs/reference/budgets.md`
[^D106C]: Target platform costs, would the choice pass collapse if it decided for each cell. `docs/reference/graviton-costs.md`
[^D106E]: Review of ADR-0096, correction 1. The review artefact sits on the branch that holds it, so this branch cannot resolve its path and the citation names it instead.
[^D097F]: Findings register, FND-259, and the need spread measurement. `crates/cachette-core/tests/need_spread.rs`
[^D097G]: Findings register, FND-258, in this document.
[^ALLOC]: Findings register, FND-038. `docs/FINDINGS.md`
[^TARGET]: Blockers register, BLK-004, and the scale constants. `docs/reference/budgets.md`
[^MOVETIME]: The movement timing note, and DEC-008 above. `docs/research/movement-timing.md`
[^FIXEDITER]: ADR-0005, a solver runs a fixed iteration count, decision D1. `docs/adrs/accepted/adr-0005-a-solver-runs-a-fixed-iteration-count.md`
[^BLK7]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^ADR56D3]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D3. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
[^ADR56D4]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D4. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
[^ADR18D3]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D3. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
[^ITEM0030]: Backlog item 0030. `docs/backlog/complete/0030-enforce-the-barrier-ordering.md`
[^ADR67D4]: ADR-0067, the viewer reads the world and never writes to it, decision D4 and its consequences. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
[^ADR73D1]: ADR-0073, gathering is admitted by sort-then-admit against the tile, decision D1. `docs/adrs/accepted/adr-0073-gathering-is-admitted-by-sort-then-admit-against-the-tile.md`
[^ORIENT]: Project orientation, the design principles. `CLAUDE.md`
[^SHAPE1]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
[^DEC63B]: The founded group tests. `crates/cachette-core/tests/founded_group_survives.rs`
[^DEC63C]: ADR-0085, an entity crosses to Python as one opaque identity that the engine resolves, decision D3. `docs/adrs/accepted/adr-0085-an-entity-crosses-to-python-as-one-opaque-identity.md`
[^DEC63D]: Backlog item 0161. `docs/backlog/proposed/0161-let-a-selector-say-where-to-act.md`
[^DEC63E]: ADR-0054, an entity belongs to one of three tiers, declared at creation, decision D1. `docs/adrs/accepted/adr-0054-an-entity-belongs-to-one-of-three-tiers-declared-at-creation.md`
[^DEC63F]: ADR Registry, row 0043. `docs/adrs/REGISTRY.md`
[^DEC63G]: ADR Registry, rows 0040, 0043, 0051 and 0052. `docs/adrs/REGISTRY.md`
[^F147]: Findings register, FND-147. `docs/FINDINGS.md`
[^TEST2A]: Testing rules, section 2a. `.claude/rules/testing.md`
[^PRD12]: PRD-0012, a world starts small and grows. `docs/product/accepted/prd-0012-a-world-starts-small-and-grows.md`
[^FND022]: Findings register, FND-022. `docs/FINDINGS.md`
[^FND072]: Findings register, FND-072. `docs/FINDINGS.md`
[^REP18]: Vector entity representation, section 9 and decision D155. `docs/research/reports/18-vector-entity-representation.md`
[^REP14]: The character graph and inheritance, sections 2.1, 3.3 and 15.3. `docs/research/reports/14-character-graph-and-inheritance.md`
[^SCOPE1]: Decision Record Scope, section 1. `.claude/rules/adr-scope.md`
[^REG21]: ADR Registry, reserved row 0021. `docs/adrs/REGISTRY.md`
[^ITEM0098]: Backlog item 0098. `docs/backlog/complete/0098-give-the-gate-suite-a-development-budget.md`
[^ITEM0092]: Backlog item 0092. `docs/backlog/complete/0092-refuse-a-settlement-on-the-ground-that-cannot-carry-one.md`
[^ADR14D7]: ADR-0014, entity identity is an index plus a generation, decision D7. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
[^BLK18]: Blockers register, BLK-018. `docs/BLOCKERS.md`
[^ITEM0094]: Backlog item 0094. `docs/backlog/complete/0094-decide-how-many-groups-found-a-world.md`
[^FND128]: Findings register, FND-128. `docs/FINDINGS.md`
[^FND129]: Findings register, FND-129. `docs/FINDINGS.md`
[^ADR81]: ADR-0081, a residence is a stored column and occupancy is a maintained count, decision D3. `docs/adrs/draft/adr-0081-a-residence-is-a-stored-column-and-occupancy-is-a-maintained-count.md`
[^ADR75]: ADR-0075, the founding choice reads a bounded sample of the world. `docs/adrs/accepted/adr-0075-the-founding-choice-reads-a-bounded-sample-of-the-world.md`
[^FND106]: Findings register, FND-106. `docs/FINDINGS.md`
[^DEC44ITEM]: Backlog item 0060. `docs/backlog/proposed/0060-grow-the-population-from-the-store-and-the-housing.md`
[^SCOPE]: Decision Record Scope, section 4.1. `.claude/rules/adr-scope.md`
[^FND135]: Findings register, FND-135. `docs/FINDINGS.md`
[^DOD3]: Definition of Done, section 3. `.claude/rules/definition-of-done.md`
[^BLK3]: Blockers register, BLK-003. `docs/BLOCKERS.md`
[^ITEM0150]: Backlog item 0150. `docs/backlog/complete/0150-reserve-the-unit-columns-at-construction.md`
[^DEC60A]: The event types. `crates/cachette-core/src/event.rs`
[^DEC60C]: Findings register, FND-137. `docs/FINDINGS.md`
[^DEC60D]: Backlog item 0153. `docs/backlog/refined/0153-let-python-read-an-event-without-repeating-its-layout.md`
[^DEC60E]: The identity type. `crates/cachette-core/src/types.rs`
[^DEC60F]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^BLK4]: Blockers register, BLK-004. `docs/BLOCKERS.md`
[^BLK5]: Blockers register, BLK-005. `docs/BLOCKERS.md`
[^DEC59]: Decisions register, DEC-059, in this document.
[^DEC34REF]: Decisions register, DEC-034, in this document.
[^DEC49REF]: Decisions register, DEC-049, in this document.
[^FND015]: Findings register, FND-015. `docs/FINDINGS.md`
[^FND089]: Findings register, FND-089. `docs/FINDINGS.md`
[^ADR80]: ADR-0080, a depleted deposit recovers by ageing the stored take. `docs/adrs/accepted/adr-0080-a-depleted-deposit-recovers-by-ageing-the-stored-take.md`
[^DEC69A]: ADR-0051, a selector is a lazy expression tree that Rust evaluates, decision D1. `docs/adrs/accepted/adr-0051-a-selector-is-a-lazy-expression-tree.md`
[^DEC69B]: ADR-0052, a selector result may be a range, not only an enumerated set, decision D2. `docs/adrs/draft/adr-0052-a-selector-result-may-be-a-range.md`
[^DEC69C]: Selector engine and verbs, section 3.2. `docs/research/reports/04-selector-engine-and-verbs.md`
[^DEC68A]: ADR-0012, tiles are dense columns and units are a generational arena, decision D2. `docs/adrs/accepted/adr-0012-tiles-are-dense-columns-and-units-are-a-generational-arena.md`
[^DEC68B]: ADR-0068, terrain is generated from the seed and is never stored as a map, decision D1. `docs/adrs/accepted/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
[^DEC68C]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D4. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
[^DEC68D]: ADR-0088, a tile field is a generated base and a stored change, decision D1. `docs/adrs/draft/adr-0088-a-tile-field-is-a-generated-base-and-a-stored-change.md`
[^DEC68E]: Findings register, FND-081. `docs/FINDINGS.md`
[^DEC68F]: Decision Record Scope, section 7. `.claude/rules/adr-scope.md`
[^DEC67LEVEL]: ADR-0022, level 0 is the only truth, and every level above it is derived, decisions D1 and D2. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
[^DEC67ADR]: ADR-0087, an influence solve runs a fixed iteration count over the whole plane. `docs/adrs/draft/adr-0087-an-influence-solve-runs-a-fixed-iteration-count.md`
[^DEC67041]: Decisions register, DEC-041, in this document.
[^DEC67REPORT]: Influence maps, sections 4 and 6. `docs/research/reports/09-influence-maps.md`
[^DEC67DOD]: Definition of Done, section 2. `.claude/rules/definition-of-done.md`
[^DEC68G]: ADR Registry, row 0015. `docs/adrs/REGISTRY.md`
[^DEC71A]: Backlog item 0067, record a parent and walk a line. `docs/backlog/complete/0067-record-a-parent-and-walk-a-line.md`
[^DEC71B]: The character graph and inheritance, section 2.1. `docs/research/reports/14-character-graph-and-inheritance.md`
[^DEC71C]: PRD-0015, a unit has parents and children. `docs/product/accepted/prd-0015-a-unit-has-parents-and-children.md`

[^DEC72A]: ADR-0090, a tile upgrade is stored sparsely, as the difference from the generated world, decision D2. `docs/adrs/draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md`
[^DEC72D]: Findings register, FND-174. `docs/FINDINGS.md`
[^DEC74FND]: Findings register, FND-180. `docs/FINDINGS.md`
[^DEC74NOTE]: What a unit does in a tick, sections 1 and 3.4. `docs/research/what-a-unit-does-in-a-tick.md`
[^DEC74SHAPE]: Findings register, FND-181. `docs/FINDINGS.md`
[^DEC74TEST]: Testing Rules, section 2a. `.claude/rules/testing.md`
[^DEC74BASE]: Findings register, FND-130. `docs/FINDINGS.md`
[^DEC74PRI]: Backlog priority index. `docs/backlog/PRIORITY.md`
[^DEC79A]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, decisions D1 and D4. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
[^DEC79B]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D2. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
[^DEC79D]: Backlog item 0185, steer a step by the option the unit chose. `docs/backlog/complete/0185-steer-a-step-by-the-option-the-unit-chose.md`
[^DEC79E]: Findings register, FND-226. `docs/FINDINGS.md`
[^DEC79F]: Backlog item 0240, let the demonstration make a unit hungry. `docs/backlog/complete/0240-let-the-demonstration-make-a-unit-hungry.md`
[^DEC74SEP]: Findings register, FND-227. `docs/FINDINGS.md`
[^DEC80B]: Backlog item 0187, give a carried load somewhere to go. `docs/backlog/refined/0187-give-a-carried-load-somewhere-to-go.md`
[^DEC81A]: Findings register, FND-193. `docs/FINDINGS.md`
[^DEC81B]: ADR-0074, a spawn may over-fill a tile, and only admission enforces the capacity, decision D1. `docs/adrs/accepted/adr-0074-a-spawn-may-over-fill-a-tile-and-only-admission-enforces-the-capacity.md`
[^DEC75KIND]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D3. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
[^DEC75SHAPE]: Recurring defect shapes, shape 3. `.claude/rules/recurring-defects.md`
[^DEC83A]: The citation check script. `scripts/check_citations.py`
[^DEC83B]: Findings register, FND-197. `docs/FINDINGS.md`
[^DEC83C]: ADR Registry, the status vocabulary. `docs/adrs/REGISTRY.md`
[^DEC83D]: Review 0204, the two corrected records. `docs/reviews/0204-the-two-corrected-records.md`
[^DEC77A]: ADR-0064, a unit chooses by scoring a small fixed option set, decision D2. `docs/adrs/accepted/adr-0064-a-unit-chooses-by-scoring-a-small-fixed-option-set.md`
[^DEC77B]: ADR-0070, the head-up display reports what the drawing pass read, decision D1. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
[^DEC78A]: Backlog item 0133. `docs/backlog/complete/0133-let-a-watcher-reach-a-panel-longer-than-the-window.md`
[^DEC78B]: Decisions register, DEC-084, in this document.
[^DEC84B]: PRD-0005, a watcher can tell what is happening and why. `docs/product/shipped/prd-0005-a-watcher-can-tell-what-is-happening-and-why.md`
[^DEC85REF]: Decisions register, DEC-085, in this document.
[^DEC85DOD]: Definition of Done. `.claude/rules/definition-of-done.md`
[^DEC94HOOK]: The pre-commit hook. `.githooks/pre-commit`
[^DEC86A]: Backlog item 0152, what is still open. `docs/backlog/complete/0152-let-an-agent-drive-the-engine-through-a-protocol-server.md`
[^DEC86B]: ADR-0092, the agent tool surface grows one tool at a time, against a stated need. `docs/adrs/draft/adr-0092-the-agent-tool-surface-grows-against-a-stated-need.md`
[^DEC86C]: PRD-0019, an agent can ask the running engine what it holds. `docs/product/shaped/prd-0019-an-agent-can-ask-the-running-engine-what-it-holds.md`
[^DEC88A]: Findings register, FND-206. `docs/FINDINGS.md`
[^DEC88B]: Findings register, FND-207. `docs/FINDINGS.md`

[^DEC91A]: Blockers register, BLK-010. `docs/BLOCKERS.md`
[^DEC91B]: ADR-0065, a group is a site membership, not a region, decision D1. `docs/adrs/draft/adr-0065-a-group-is-a-site-membership-not-a-region.md`
[^DEC91C]: Review 0223, the group membership record. `docs/reviews/0223-the-group-membership-record.md`
[^DEC91D]: Decisions register, DEC-036, in this document.
[^DEC92A]: ADR-0078, descent is a bounded record, and a relation is a bounded recursion, decision D1. `docs/adrs/draft/adr-0078-descent-is-a-bounded-record-and-a-relation-is-a-bounded-recursion.md`
[^DEC92C]: Review 0223, the descent record. `docs/reviews/0223-the-descent-record.md`

[^DEC32ADR]: ADR-0021, a layout claim names one structure and one pass, and never a tier. `docs/adrs/draft/adr-0021-layout-follows-the-access-pattern.md`
[^DEC32FND]: Findings register, FND-236. `docs/FINDINGS.md`
[^DEC17NEXT]: Decisions register, DEC-093, in this document.
[^DEC93TIMES]: Decisions register, DEC-008, in this document.
[^DEC93HOME]: Decisions register, DEC-017, in this document.
[^DEC95A]: ADR-0095, a behavioural strategy arrives as a field over cells, never as a search from a unit, decision D3. `docs/adrs/draft/adr-0095-a-behavioural-strategy-arrives-as-a-field-over-cells.md`
[^DEC95B]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, decision D2. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
[^DEC95D]: Decisions register, DEC-067, in this document.
[^DEC95E]: ADR-0095, a behavioural strategy arrives as a field over cells, never as a search from a unit, decision D4. `docs/adrs/draft/adr-0095-a-behavioural-strategy-arrives-as-a-field-over-cells.md`
[^DEC96A]: ADR Registry, the retcon window. `docs/adrs/REGISTRY.md`
[^DEC96B]: Findings register, FND-218. `docs/FINDINGS.md`
[^DEC96C]: ADR-0096, cost follows the lattice, not the population, and a unit is a reader, decision D4. `docs/adrs/draft/adr-0096-cost-follows-the-lattice-not-the-population.md`
[^DEC105A]: Target platform costs, the resident memory rows. `docs/reference/graviton-costs.md`
[^DEC105B]: Findings register, FND-281. `docs/FINDINGS.md`
[^DEC96D]: ADR-0064, a unit chooses by scoring a small fixed option set, decision D1. `docs/adrs/accepted/adr-0064-a-unit-chooses-by-scoring-a-small-fixed-option-set.md`
[^DEC97B]: Target platform costs. `docs/reference/graviton-costs.md`
[^SWEEP]: Recurring Defect Shapes, shape 2. `.claude/rules/recurring-defects.md`
