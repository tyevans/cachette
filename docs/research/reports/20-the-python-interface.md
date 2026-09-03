# What the Python Interface Should Be

Research report 20 for the decision on what this engine offers a Python
developer, and what that offer looks like. Prepared 3 September 2026.

Cachette is a world simulation engine. The core is Rust. The control plane is
Python. A developer who builds on this engine writes Python and never opens the
core.[^1] The Python package holds a small amount of pure Python and re-exports
a compiled extension module.

Every measured figure in this report carries a footnote that says what the
author ran and against what. A decision record must not hold such material, so
this report is its only home.[^2]

## 0. Provenance, and what this report could not verify

**Every measurement here ran against the installed extension module on an
x86-64 development machine.** The target platform is a 64-bit Arm server, and a
local measurement misleads on cost.[^3] No figure in this report is a target
platform figure, and one blocker holds that gap for the whole project.[^4]

**The installed module is older than the source tree.** Its `World` class
provides no `faction_population` and no `panel_names`, and the source of this
worktree declares both. Every statement below about a member that the installed
module does provide is a measurement. Every statement about a member it does not
provide is a read of the Rust source, and this report says which.

**The author did not fetch the published reference page.** Two graded reads of
that page are the evidence for what a fresh reader meets, and this report treats
them as reports of one moment.[^5] [^6]

**Two claims in section 4 are marked unverified.** The author did not read the
source of the `pydantic-core` crate and did not confirm whether that crate is
published for a Rust consumer.

### 0.1 The findings

1. **A working first program exists today, and the reference does not show
   it.** The author wrote one and ran it. The constructor takes four arguments,
   every one has a default, and `World()` builds a world. The prose that
   explains all of this is written in Rust and reaches no Python docstring, so a
   reader graded the first action 1 of 5 against documentation that exists and
   cannot be read.[^7] The interface failed at the first step for a reason that
   is not the interface.

2. **The second program is where the interface fails, and it fails for
   everybody.** A caller who wants to know where a set of units stands has one
   route: a Python loop over a singular read. This repository's own Python takes
   that route seven times.[^8] The loop costs about 4.3 microseconds for each
   unit here, which is about 4.3 seconds for one million units, in each frame
   that asks.[^9] The design forbids the loop and offers nothing else.

3. **The selector is the whole answer, and it does not exist.** An accepted
   record specifies it in full: Python builds a lazy expression tree, the tree
   crosses once, and Rust evaluates it.[^10] No selector type exists in Python or
   in Rust. Every complaint in both graded reads is a symptom of that gap.

4. **The dangerous values are the loose scalars, not the columns.** The columns
   are typed, dense and honest. A scalar in a returned mapping carries a
   fixed-point scale, or a kind from one of three overlapping numberings, or a
   sentinel from one of three conventions, and nothing distinguishes it from a
   plain count. The repair is a type on the scalar path and nothing at all on the
   column path.

5. **The project should adopt the properties of Pydantic and not the
   dependency.** Pydantic reaches this repository already, in the development
   group, under the agent protocol server.[^11] It earns its place at that
   boundary and at no other. Section 4 gives the argument, including why linking
   its Rust core is not available.

6. **The answer is two tiers.** The compiled module stays exactly as it is, and
   a pure Python tier sits above it. The line is that the Python tier holds no
   simulation state and copies no column.

---

## 1. The jobs

This section names what a Python developer with an idea should be able to do
easily. The ranking is the argument. The first job is the one the interface
should be designed around.

**One blocker governs this ranking.** No Python developer outside this
repository has used this interface. The ranking rests on two graded reads and on
the shape of the code inside the repository, and not on a user.[^12]

### Job 1. Make a world, run it, and watch something happen

**Who.** A Python developer who has an idea for a simulation and who has never
read the source of this project. This is the audience an accepted product record
already names.[^1]

**Why they stop today.** They do not stop, and this is the report's first
correction to the graded reads. The author wrote this program and ran it:

```python
import cachette
import numpy as np

world = cachette.World(width=64, height=64, seed=1, faction_count=4)
world.found_run_for_every_faction(64)
for _ in range(10):
    world.step(4)

camera = cachette.Camera.fitting(world, 320, 200)
pixels = np.zeros(320 * 200, dtype=np.uint32)
world.draw(camera, 320, 200, pixels)
```

It seats four factions, runs ten ticks, reaches 256 soldiers and four
settlements, and fills 64,000 pixels of a 64,000 pixel picture.[^13] Eleven
lines, and every one of them works.

**So the failure is the documentation and the shape of line eleven.** A reader
cannot find the constructor, because its prose reaches no Python object. A
reader who finds the constructor still meets a raw pixel buffer whose length is
declared three times.

This job stays first because everything else is downstream of it, and because
the repair is cheap. It is not the job with the largest design content.

### Job 2. Name a set of things by description, and give it one order

**Who.** The same developer, on their second program.

**Why they stop today.** They cannot describe a set. They can name a unit they
already hold and an address they chose. The only way to act on "every idle unit
of my faction that stands on ground holding food" is to build the set in Python,
which is the data plane the project forbids.[^14]

This is the job with the design content, and an accepted record already
specifies its answer.[^10] Section 3.4 writes the code.

### Job 3. Read back what the world holds, for a set, in one call

**Who.** The same developer, and a modeller who wants a large agent count.

**Why they stop today.** Every write takes a set. Every unit read takes one
identity. Nothing enumerates the live entities. A developer who drops the
identity column that `spawn_soldiers` returned has no way to name any unit
again, although `soldier_count` will report how many are alive.

The engine's own text defends the singular read: a set form would have to choose
between failing the whole call for one dead identity and returning a value that
stands for nothing.[^15] **The engine already refutes this itself.** A founding
survey returns an `eligible` column beside its answers, and that column says
which rows are answers.[^16] The same idiom answers the objection for units.

### Job 4. Watch a running world without writing a render loop

**Who.** The developer of job 1, five minutes later.

**Why they stop today.** The frame path is correct and it is raw. The caller
declares the picture size to the camera on every camera verb, again to `draw`,
and again through the length of a NumPy array. One error class exists to catch
the caller getting those three declarations out of step.[^17] The caller must
also clamp the camera by hand after every move, and the interface's own prose
says an unclamped camera draws a picture that a person cannot tell from an empty
world.

### Job 5. Store a run, and start it again

**Who.** A researcher who must reproduce a run, and any developer who wants to
send a broken world to a colleague.

**Why they stop today.** No call stores a world and none rebuilds one. The
engine's central promise is that a run repeats, and a caller can check that with
the state hash and cannot keep the run.

This job ranks last because it is a missing capability rather than a wrong
shape, and because nothing in the design blocks it. It should not be built into
the friendly tier. It should be built into the core and exposed by both tiers.

---

## 2. The worked comparison

Every "today" block below is the code the current module accepts. The author ran
each one, except where the text says the job is impossible.

### 2.1 Job 1, the first program

**Today.** The program in section 1 runs. What a reader must supply from
somewhere other than the reference is the module to import, the four constructor
arguments, the fact that a new world holds nothing, the founding call that fills
it, a thread count with no default, the camera factory, the pixel array dtype,
and the picture size three times.

**Proposed.**

```python
import cachette

world = cachette.World.generate(width=64, height=64, seed=1, factions=4)
world.found_starting_settlements(group=64)
world.run(10)

viewer = cachette.Viewer(world, width=320, height=200)
viewer.fit()
frame = viewer.draw()
frame.save("world.png")
```

What changed, and why each change earns its place.

- **`World.generate`.** A named constructor says which of several ways of making
  a world this is, and it leaves `World.load` and `World.like` room to exist
  later without an overloaded `__init__`. Section 3.2 argues this against the
  alternative.
- **`factions` rather than `faction_count`.** The count is what the name means,
  and a plural noun taking a number is the ordinary Python spelling.
- **`world.run(10)`.** Ten ticks is the intent. The thread count moves onto the
  world, where it is a property of the machine, and `step(threads=...)` keeps
  the override that the determinism test needs. The current spelling puts a
  literal thread count into every call site: this repository writes it twenty
  times.[^8]
- **`Viewer` holds the picture size.** One declaration, in one place, for the
  life of the session. The camera verbs stop taking it. `draw` stops taking it.
  The array stops carrying it. One error class loses a trigger.
- **`viewer.fit()` and an automatic clamp.** The viewer keeps the camera inside
  the world, because the interface already knows that an unclamped camera
  produces a wrong picture that a person cannot detect.

### 2.2 Job 2, ordering a described set

**Today. This job is impossible.** There is no way to describe a set. A caller
who wants every idle unit of one faction that stands on ground holding food must
build the set in Python. The nearest thing the module permits is this, and the
project forbids it:

```python
# Forbidden. This is the data plane, and it does not scale.
# Nothing in the module refuses it.
units = every_identity_i_kept          # the caller's own registry of the population
wanted = []
for unit in units:                     # one crossing for each unit
    tile = world.soldier_tile(int(unit))
    report = world.tile_report(tile % world.width, tile // world.width)
    if report["stock"][0] > 0:
        wanted.append(unit)
world.order_gather(wanted, 0)
```

Three defects, and the interface causes all three. The caller keeps a Python
list of every identity, because nothing enumerates them. The caller crosses the
boundary twice for each unit. The caller writes `0` for food, which is also the
number of water.

**Proposed.**

```python
from cachette import Ground, Resource

fertile = world.tiles.where(world.tiles.stock[Resource.FOOD] > 0)
idle = world.units.of_faction(1).where(world.units.is_idle)

idle.standing_on(fertile).order_gather(Resource.FOOD)
```

Three statements, no crossing until the last one, and no population in Python.
Building `fertile` and `idle` reads no tile and no unit: the value of each
expression is a tree of nodes, and the record requires exactly this.[^18] The
whole tree crosses once, with the verb that consumes it.[^19]

`Resource.FOOD` is an `IntEnum` member, so it is the integer zero and it prints
as its own name. `Ground` is a different enumeration, so `Ground.PLAIN` cannot
be passed where a resource belongs.

### 2.3 Job 3, reading a set

**Today.** The loop, and only the loop.

```python
tiles = [world.soldier_tile(int(unit)) for unit in units]
```

The author ran this over 64 units and measured 4.3 microseconds for each
call.[^9] At one million units that is about 4.3 seconds, in each frame that
asks. The interface names this failure in its own prose and ships it.[^20]

**Proposed, from a selector.**

```python
columns = world.units.of_faction(1).read("unit", "tile", "carrying")
tiles = columns["tile"]        # numpy.uint32, one crossing, no Python loop
```

A selector names only live entities at the moment it is evaluated, so no
validity mask is needed and no entry stands for nothing.[^21]

**Proposed, from identities the caller already holds.** This is the case the
current prose refuses, and the mask answers it.

```python
columns = world.units.of(units).read("tile", on_missing="mask")
tiles = columns["tile"]        # numpy.uint32
live = columns["live"]         # numpy.uint8, one where the entry is an answer
```

`on_missing` defaults to `"raise"`, which keeps the transactional promise the
set-valued verbs already make: one dead identity fails the whole call and names
itself.[^22] A caller who prefers the mask asks for it at the call site, which is
where this project declares what a call does.[^23]

### 2.4 Job 4, watching

**Today.**

```python
camera = cachette.Camera.fitting(world, 800, 600)
pixels = np.zeros(800 * 600, dtype=np.uint32)
camera.look_at(32, 32, 800, 600)
camera.zoom_in(800, 600)
camera.clamp(world, 800, 600)
reading = world.draw(camera, 800, 600, pixels, panel=True)
print(reading["ticks_each_second"])
```

The size appears six times. The clamp is the caller's to remember. The rate of
ticks is only obtainable by drawing a picture.

**Proposed.**

```python
viewer = cachette.Viewer(world, width=800, height=600)
viewer.look_at(cachette.Axial(32, 32))
viewer.zoom_in()
frame = viewer.draw(panels=["census"])
print(viewer.ticks_each_second)
```

The size is declared once. The viewer clamps. The frame is an object that holds
its own pixels as a two-dimensional array whose shape is the size, so the size
cannot disagree with the array. The rate of ticks belongs to the thing that
holds the clock.

**This moves nothing that the engine holds.** The bindings crate already keeps a
separate structure for the caller's frame state, and its own doc comment says
the world holds none of it, because a field for the viewer would breach the
viewer record.[^24] That structure is welded onto the world object as a second
lock. The proposal gives it a name in Python.

### 2.5 Job 5, storing a run

**Today. This job is impossible.** The state hash proves that a run repeated. It
does not store the run.

**Proposed.**

```python
world.save("run.cachette")
again = cachette.World.load("run.cachette")
assert again.state_hash() == world.state_hash()
```

This report does not design the format. It names the job, ranks it last, and
records that the friendly tier is the wrong place to build it.

---

## 3. The shape

### 3.1 The value types

**A fixed-point type, as a subclass of `int`. Agree.** The graded read is right
that this is the hazard whose failures are silent, and silent wrong numbers are
the ones that reach a report.[^25]

```python
class Fixed(int):
    """A quantity at the fixed-point scale of the engine, as its raw integer."""

    SCALE = 65536

    @classmethod
    def ratio(cls, numerator: int, denominator: int) -> "Fixed": ...

    @classmethod
    def from_float(cls, value: float) -> "Fixed": ...

    @property
    def units(self) -> Fraction: ...

    def __repr__(self) -> str:      # "Fixed(3/2)"
        ...
```

Why a subclass of `int` and not a new class. Every existing call keeps working,
because a `Fixed` **is** an integer: `store / 65536`, `store < limit`, `sorted`
and `json.dumps` all behave as they do today. The value crosses to Rust as the
integer it already is, so nothing about the boundary changes and nothing about
determinism changes. A separate class would need a conversion at every call
site, and a conversion is the instruction that produced the loop this project
already recorded.[^26]

**This report disagrees with the graded read on one point.** The read proposes
that a verb accept a bare Python float and read it as the real share, so that a
new caller writes `target=1.5`.[^27] This report rejects that.

The reason is the hard invariant. A float multiply in Python is outside the
deterministic core, and a value that entered the simulation through one carries
no record that it did. The rounding is invisible at the call site and the result
reaches simulated state. **A verb refuses a bare float, and the error names
`Fixed.ratio`.** A caller who genuinely holds a float converts through
`Fixed.from_float`, which is a named call that a reader can see and a reviewer
can search for.

```python
world.sites.of(sites).set_work_target(Resource.FOOD, Fixed.ratio(3, 2))

world.sites.of(sites).set_work_target(Resource.FOOD, 1.5)
# VerbError: the target must be a Fixed, and 1.5 is a float. A float multiply
# happens outside the deterministic core, so this call refuses it. Write
# Fixed.ratio(3, 2) for one and a half, or Fixed.from_float(1.5) to accept the
# rounding on purpose.
```

This also closes the write hazard the graded read found: `prefer_at_sites(sites,
0, 1)` is legal today and means one part in 65536, when the caller meant
one.[^28] A bare `1` is now refused for the same reason a bare `1.5` is.

**`Fixed` wraps a scalar. It never wraps an element of a column.** A column
stays raw, and its declared type says the scale. This is the whole cost
argument, and section 6 states it as an invariant of the tier.

**An `IntEnum` for each kind. Agree, with one correction.** The graded read
proposes four enumerations: resource, ground, change, and work.[^29] The work
scale and the resource scale are the same three numbers with two names, so four
enumerations would create the defect shape this project keeps catching: one
value declared in two places, with nothing that fails when the copies
disagree.[^30] **Declare three: `Resource`, `Ground` and `Change`. Rename the
work parameter to `resource`.**

An `IntEnum` member is an integer, so every existing call and every comparison
against a column entry keeps working. It is real at run time, so `Resource(7)`
raises at the boundary. It survives the round trip through a NumPy column,
because the column stays `uint8` and the caller names the member on the scalar
path only.

The measurement supports the enumeration over more prose. The engine checks the
range and nothing else: `order_gather(units, 4)` raises and names the fault, and
`order_gather(units, 2)` is accepted in full whether the caller meant stone or
forest.[^31] The range check catches two of the five ground kinds. The reference
page restates the numbering in six places and it has not made
`order_gather(units, 2)` readable.

**The enumeration is generated from the compiled module, not written in
Python.** A Python `class Resource(IntEnum): FOOD = 0` is a second declaration
site for a numbering the Rust source already holds, and nothing fails when the
two part. The module exports the names in order, and the Python tier builds the
enumeration from them.

```python
Resource = IntEnum("Resource", _core.resource_kind_names(), start=0)
```

**`Axial` for a tile address. Agree, as a `NamedTuple`.**

```python
class Axial(NamedTuple):
    q: int
    r: int
```

A `NamedTuple` **is** a tuple, so `spawn_soldiers([(0, 0), (1, 0)])` keeps
working and `spawn_soldiers([Axial(0, 0)])` works too. It carries the hex
arithmetic that every caller writes and that the graded read says half will get
wrong: `neighbours`, `distance`, `ring`. It also carries `index(world)` and
`Axial.from_index(world, index)`, which removes the row-major conversion that
the reference explains three separate times.

**`Axial` is for the scalar path.** A bulk read returns `q` and `r` columns, or
an index column. Nothing allocates an `Axial` for each tile.

**What replaces the 65535 sentinel.** Three conventions exist today for one
idea: a scalar mapping gives `None`, one column uses 65535, and another column
uses zero.[^32] Three declaration sites, one meaning.

The rule: **a scalar that may name nobody is `None`. A column that may name
nobody ships a companion `uint8` mask and carries no in-band sentinel.** The
mask idiom is already in the interface, on the founding survey, and it is
correct there for the same reason.[^16] An in-band sentinel is a value that
means two things, and the reader cannot tell which without knowing the column.

The zero sentinel deserves its own sentence, because the graded read could not
tell whether zero is a valid identity and worried that it might be. **It is
not.** The author measured the first identity the engine hands out and it is
4,294,967,296, which is one generation above index zero.[^33] Zero is safe today
and it is safe by an accident of the generation encoding, not by a stated rule,
and a mask does not depend on the accident.

**A faction is not an enumeration.** The count is chosen at construction, so no
Python enumeration can hold it. Declare it as an integer in the type stub, add
`world.faction_count`, and give `world.factions` as a range for a caller who
wants to walk the factions. The faction population is a bounded set, so walking
it is allowed.

### 3.2 Construction

**Named constructors, and `__init__` stays.**

```python
class World:
    @classmethod
    def generate(cls, *, width, height, seed, factions=4, threads=None) -> "World": ...

    @classmethod
    def load(cls, path) -> "World": ...

    @classmethod
    def like(cls, other, *, seed) -> "World": ...
```

The argument for named constructors is not that one overloaded `__init__` cannot
be written. It is that the three calls above take different arguments and mean
different things, and a single `__init__` would have to accept the union of
them, refuse most combinations, and explain the refusals in prose.

**This report rejects the `World.from_tiles(...)` that the graded read
proposes.** An accepted record states that terrain is generated from the seed
and is never stored as a map.[^34] There is no array of tiles to hand a
constructor, so `from_tiles` would declare a capability that nothing can invoke,
which is a defect shape this project keeps catching.[^35] `World.generate` is
the right name precisely because it says what the engine does.

**The constructor round-trips its own arguments.** The world reports `width` and
`height` today and reports neither `seed` nor `faction_count`. The author
measured that gap and found a second declaration site already built to fill
it: the agent protocol server declares a frozen dataclass of the four
construction values, and its own docstring says the store keeps them because the
engine does not report them back.[^36] That is the redundant declaration shape,
in this repository, today.[^30] A researcher who must reproduce a run reads the
seed from a Python object that the engine cannot check.

**The thread count moves to the world.** It has no default, the result does not
depend on it, and every call site carries the same literal forever. Give
`World.generate` a `threads` argument that defaults to a sensible count, give
the world a settable `threads`, and keep `step(threads=...)` as an override. The
determinism test is the one caller that needs to vary it, and it says so at its
own call site.

### 3.3 The bulk read

The read side is the missing half of the design. Its shape follows from three
constraints the project has already fixed.

A read returns the answer to a question and never a buffer the caller must
decode.[^37] The number of crossings must not grow with the population.[^38] The
escape hatch returns columns and never yields entities one at a time.[^39]

**So a bulk read is one call that takes a set and a list of field names, and
returns one column for each name.**

```python
columns = world.units.where(...).read("unit", "tile", "faction", "carrying")
```

Four properties of this shape, and the reason for each.

- **The caller names the fields.** A read that returned every field would copy
  columns the caller did not ask for, and at the target scale a column is tens
  of megabytes. What copies is declared at the call site, and this project
  already binds that.[^23]
- **The result is a mapping of NumPy arrays, and the arrays are the engine's own
  values.** No Python object is allocated for any entity.
- **The order is the order the selector fixes.** Evaluation fixes its order by a
  stable key over the entities, never by a thread.[^40] Two reads of one tree
  over one world state give the same columns in the same order at any thread
  count.
- **A missing entity is a mask entry, never a sentinel.** Section 2.3 shows both
  spellings and section 3.1 gives the rule.

**One current read already breaks the spirit of this.** `tile_values()` copies a
whole `int32` column of the world and also generates it, because the world holds
no such array. At 16.7 million tiles that is a copy of about 67 megabytes for
each call, whatever the caller wanted from it. It is one crossing and one
answer, so it satisfies the boundary record in form.[^37] The friendly tier
should not offer it as a property. It should offer `world.tiles.where(...)
.read("value")`, so that a caller narrows before the engine copies.

### 3.4 The selector, concretely

The record specifies the semantics.[^10] This section writes the Python.

```python
from cachette import Axial, Ground, Resource

# Building reads nothing. Each line makes nodes and nothing else.
fertile = world.tiles.where(
    (world.tiles.ground == Ground.PLAIN)
    & (world.tiles.stock[Resource.FOOD] > 0)
)
near_home = world.tiles.near(Axial(12, 9), radius=8)
target = fertile & near_home

idle = world.units.of_faction(1).where(world.units.is_idle)

# One crossing. The whole tree goes with the verb.
idle.standing_on(target).order_gather(Resource.FOOD)
```

**The domains are separate types.** `world.tiles` builds a `TileSelector` and
`world.units` builds a `UnitSelector`. Combining them directly fails when the
caller writes it, which the record requires:[^41]

```python
>>> fertile & idle
SelectorError: a TileSelector and a UnitSelector describe different domains and
    cannot be combined. Cross the domains by name: fertile.holding(idle) for the
    tiles that hold those units, or idle.standing_on(fertile) for the units that
    stand on those tiles.
```

**The terminal operations are the only calls that cross.**

| Call | Returns | Crossings |
|---|---|---|
| `.count()` | `int` | one |
| `.any()` | `bool` | one |
| `.read(*fields)` | mapping of columns | one |
| `.identities()` | one `uint64` column | one |
| `.explain()` | a plan report | one |
| any verb | the verb's own answer | one |

`.explain()` is the reporting operation the record requires, and it exists
because a predicate over a field with no summary cannot prune, and nothing about
that is visible to a caller who sees only slowness.[^42]

**The four forbidden operations, and what each says.** The record requires that
each refusal name the method that answers the question the caller was really
asking, and it makes the text of the message part of the interface that a test
asserts.[^43]

```python
>>> if idle:
SelectorError: a UnitSelector has no truth value. Testing one would evaluate the
    whole set to answer a question about one bit. Call .any() for whether the
    selector names anything, or .count() for how many it names.

>>> len(idle)
SelectorError: a UnitSelector has no length. Taking one would evaluate the whole
    set. Call .count(), which crosses once and returns an integer.

>>> for unit in idle:
SelectorError: a UnitSelector does not iterate. A soldier is one of a million,
    so no caller walks that population. Call .read("tile", "faction") for the
    columns of the whole set, or apply a verb to the whole set:
    idle.order_gather(Resource.FOOD).

>>> idle[0]
SelectorError: a UnitSelector cannot be indexed. Call .read(...) for the columns
    of the whole set, or .first() for the one entity that the selector's own
    order puts first.
```

**`SelectorError` should also subclass `TypeError`.** Python raises `TypeError`
from all four of these operations, and a caller's `except TypeError` is the
handler that will meet them. Declaring `class SelectorError(CachetteError,
TypeError)` keeps both handlers correct and costs one line. Nothing raises
`SelectorError` today, so nothing depends on the current base.[^44]

**The refusal is a property of the type and never a check on a count.** No
message above consults the population. A small world refuses on the first
attempt, in development, and no world size makes the loop succeed. The record
requires this, and the reason is that a check on a count ships a script that
works in development and fails at scale.[^45]

### 3.5 Where the viewer goes

**Agree with the graded read. `draw`, `panel_names`, `panels`, `pointer` and the
frame telemetry move off `World` onto a `Viewer`.**

The argument is not taste. It is that the world already does not hold this
state, and the interface says otherwise. The bindings crate keeps the metrics and
the founding outcomes in a separate structure whose doc comment states that the
world holds none of it, because a field for the viewer would breach the accepted
viewer record.[^24] That structure is attached to the world object under a second
lock, so the Python surface reports a shape the Rust does not have.

Two consequences worth stating.

The rate of ticks is a property of stepping, and it is obtainable today only by
drawing a picture. A viewer that owns the clock reports it without a frame.

The camera works in floating point, and it stays that way. The float ban binds
simulated and aggregated state.[^46] A camera is neither: a frame is a pure
function of a world and a camera, and no camera value reaches the world. The one
place a camera value can reach simulated state is `tile_at`, which turns a
pointer position into an address that a caller may then pass to a verb. That
address is an integer by the time it crosses, and the person who moved the
pointer was not deterministic in the first place.

---

## 4. Pydantic, and what it actually buys

The project owner named Pydantic as where the Python community has moved for the
expression of elegance. This section separates two questions that look like one.
**Adopting Pydantic and adopting the properties the owner admires in it are
separate recommendations, and this report makes both.**

### 4.1 What is already here

Pydantic reaches this repository today. The package declares one runtime
dependency, NumPy, and one optional dependency for the demonstration window. The
development group declares the reference implementation of the agent protocol,
and that package depends on Pydantic, which depends on its own compiled
core.[^11]

**A development group dependency is not a runtime dependency of the package.** A
developer who installs Cachette gets NumPy and nothing else. The agent protocol
server is a contributor tool, and the manifest says so in its own comment.

The agent server does not use Pydantic directly. It declares frozen
dataclasses.[^36]

### 4.2 Is there a Rust core to integrate with?

**No, and the owner's reading is correct on all three points. This report adds a
fourth reason that is stronger than the three.**

**Two extension modules share no Rust type.** Each compiled Python extension is
linked independently and exports one symbol that CPython calls to build a
module. Rust has no stable application binary interface, so even one crate at one
version, compiled into two extension modules, produces two distinct and
incompatible types. Anything passing between the two must become a Python object
first, which is the crossing the integration was meant to avoid.

The two modules in this environment do not even target the same CPython
interface. The bindings crate builds against the stable interface for
CPython 3.11 and later, and ships one file for every version. The Pydantic core
ships a separate file built for one CPython version.[^47] They are different
artefacts by construction.

**The API validates a Python object against a schema that Python declares.**
This report marks the claim partly unverified: the author did not read the
source of that crate and did not confirm whether it is published for a Rust
consumer. What the author can state is that its published interface is reached
from Python, and that this project's validation question is not the question it
answers.

**Depending on it would couple this project's binding library version to
Pydantic's.** This project pins one version of the binding library in its
workspace manifest.[^48] A shared Rust dependency that also binds Python would
have to agree.

**The fourth reason, and the decisive one. This project's validation is not
schema validation.** The questions a verb must answer are: does this identity
name a live entity, does this address hold ground that admits a unit, does this
world hold that faction, does this number name a resource kind. Every one of
those is a question about the live world state. No schema knows the answer.
Only the engine does, and the engine already answers all of them and raises a
typed error that names the fault.[^49]

**The throughput argument runs the wrong way, and the owner is right about
that.** A validating core pays when a program validates many payloads. This
control plane sends a small number of set-valued commands in each frame, and the
bulk data crosses as arrays that no validator should touch. Validation is not the
cost here, so making validation faster buys nothing.

**Is there a route the owner missed?** The author found none that pays. Two
exist in principle and neither is worth the trade. The two modules could
exchange a serialised buffer, which adds a copy in order to remove nothing. Or
this project could vendor a validating crate and call it from Rust, which is a
different decision from adopting Pydantic and which section 4.4 answers.

### 4.3 Where Pydantic genuinely earns its place

**At the agent protocol server, and nowhere else in this repository.**

That boundary is different from the simulation interface in every way that
matters. Its input arrives from outside the process, from a client that a model
drives, so the input can be anything. The protocol itself is specified in JSON
Schema, and a schema is exactly what Pydantic emits. The payload count for each
second is small, so a validation cost is invisible. And the dependency is
already present, transitively, because the reference implementation of that
protocol brought it.

The recommendation is therefore narrow and it should be written down as such:
**the agent server may declare its request and response types as Pydantic
models. The `cachette` package must not depend on Pydantic.** A record already
binds how that tool surface grows, and this recommendation belongs beside
it.[^50]

### 4.4 What the friendly tier needs, and what would cost more than it returns

The owner named Pydantic for elegance. The properties the Python community
actually gets from that style are five, and this project needs four of them. It
can have all four for nothing.

| Property | Does this project need it | How it gets it |
|---|---|---|
| Declared types | Yes | Real annotations, and the type stub that already exists |
| Errors that name the field | Yes | The engine already raises a typed error that names the address or the identity that refused |
| Editor completion | Yes | The type stub, and `NamedTuple`, `IntEnum` and `TypedDict` |
| A readable schema | Yes, for one consumer | `TypedDict` for every returned mapping, and a generator over the stub |
| Validation and coercion at the boundary | **No** | The engine validates against live state, which no schema can do |

**The fifth row is the whole of what Pydantic sells, and it is the row this
project must refuse.**

The reason is not cost. It is correctness of ownership. A check that Python
performs is a check that runs outside the deterministic core. If the Python tier
validates a resource kind and the Rust tier also validates it, that is one rule
in two places with nothing that fails when they part, which is the defect shape
this project keeps catching.[^30] If the Python tier validates it and the Rust
tier stops, the check has left the core, and a caller who reaches the compiled
module directly bypasses it.

**So the line is this. Python checks the type. Rust checks the value.**

- Python refuses a `float` where a `Fixed` belongs, because that is a question
  about the Python object and the answer does not depend on the world.
- Python refuses a `TileSelector` where a `UnitSelector` belongs, because the
  record requires the failure at the moment the caller writes it and the answer
  does not depend on the world.[^41]
- Rust decides whether an identity is live, whether an address admits a unit,
  whether a faction exists, and whether a number names a kind. Python must not
  answer any of these, even when it looks easy.

`Resource(7)` raising at the boundary is the one case that sits on the line. It
is legitimate for Python to raise there, because the enumeration is generated
from the compiled module, so it is not a second declaration site. It is the same
check, reached earlier.

**And the shape of a value type must not be a validated object.** A Pydantic
model instance is a Python object with a validator call behind its constructor.
On the singular path that is waste. On any per-element path it is fatal, and the
per-element path is the one this project must never make easy.

**The right shape is a type that costs nothing at run time.**

| Concept | Shape | What it costs |
|---|---|---|
| A fixed-point quantity | `int` subclass | One small object, on the scalar path only |
| A kind | `IntEnum`, generated from the module | One interned member, shared |
| A tile address | `NamedTuple` | One tuple, on the scalar path only |
| A returned mapping | `TypedDict` | Nothing. It is the same `dict` |
| A column | Unchanged NumPy array | Nothing |

Every row above is a type the checker and the reader see, and four of the five
cost nothing at run time. That is the elegance the owner is asking for, and
Pydantic is not the route to it here.

---

## 5. The tiers

**Two interfaces, and the line falls at the module.**

- **The raw tier** is the compiled extension module. Columns, plain integers,
  opaque identities, and prose that lives in the Rust doc comment. It does not
  change.
- **The friendly tier** is pure Python, above it. Selectors, value types, named
  constructors, the viewer, and prose that lives in Python.

**Why two and not one.** Python is in this project for exactly one reason: a
designer changes behaviour without a compile.[^51] A single tier would put every
ergonomic change behind a Rust rebuild, and the ergonomic surface is the part
that will change most while the project learns what a developer needs. A pure
Python tier can be rewritten in an afternoon, and the constraint it wraps cannot.

**Why the raw tier does not go away.** It is the compiled module. The real
question is whether it is published, and the answer is yes, because a caller who
cannot get an answer out builds a worse route to it.[^39]

**But its name must change.** The module is `cachette._core`, and a leading
underscore tells every Python reader that the module is private. The published
reference documents that private module, and the first line a fresh reader wrote
was a guess at whether importing it was allowed.[^52] **Add a pure Python module
named `cachette.raw` that re-exports it.** One file, no build change, and the
reference points at a name that says what it is.

**Where exactly the line falls.** The friendly tier holds one reference to a raw
world and nothing else. Stated as two rules a reviewer can check:

1. **The friendly tier holds no simulation state.** Every value it returns is
   read from the raw tier at the moment of the call.
2. **The friendly tier copies no column.** A method returns the same NumPy array
   the raw tier returned, or a scalar, or a selector. It never builds a Python
   sequence whose length is a function of the population.

**How a caller crosses.** Downward is `world.raw`, one documented attribute that
returns the raw world. Upward is `cachette.World.wrap(raw_world)`. Both
directions are named, so a reader can search the tree for every crossing.

**One consequence needs a decision before this is built.** An accepted approach
binds the prose of a compiled member to the Rust doc comment, and generates the
reference by importing the module.[^53] A pure Python tier has its prose in
Python docstrings, which is a second provenance that the record does not cover.
The record is not wrong; it is silent. It must say so before the tier exists,
because a silent gap becomes two authoritative sites that nothing compares.

---

## 6. What it costs

### 6.1 What this breaks

**Nothing outside this repository.** No caller exists beyond it. This is the
cheapest moment there will ever be to change the surface.

**Inside the repository, sixteen Python files reach the interface**, plus the
orientation and the read-me.[^54] The changes that touch them are the thread
count moving to the world, the picture size moving to a viewer, and the singular
soldier read becoming a set read. The last of those is the change worth making
for its own sake: the repository's own Python takes the forbidden route seven
times.[^8]

**What becomes expensive later.** Three things, and they compound.

- **A published wheel adds a deprecation cycle to every one of these changes.**
  Today a rename is a rename.
- **The generated reference will pin a printed value.** A `Fixed` prints as
  `Fixed(3/2)` and a `Resource` member prints as `Resource.FOOD`. Any doctest or
  golden file that captured a printed integer changes. That is the whole of the
  migration cost the graded read identified, and it is cheapest before the
  reference has readers.[^55]
- **The type stub is load-bearing.** A record makes the declared types part of
  the constraint, so a stub that disagrees with the code is a breach and not an
  inconvenience.[^56] Every shape in this report is a stub change, and a stub
  change after publication is an interface change.

### 6.2 What the design costs at the target scale

**A friendly wrapper that copies a column is a wrong answer, and this design
never copies one.** Stated as the invariant of the tier:

> No method of the friendly tier allocates a Python object for each entity or
> for each tile.

Where each proposed type sits against that invariant.

| Type | Where it is allocated | How many, at the target scale |
|---|---|---|
| A selector node | When the caller composes | The size of the expression, never the world |
| `Fixed` | On a scalar in a returned mapping | Bounded by the key count of the mapping |
| An `IntEnum` member | Interned once | One for each kind, for the process |
| `Axial` | On the singular path | One for each call, never one for each tile |
| A column | Not allocated | The engine's own array, handed through |

**Building a selector costs nothing that depends on the world.** The record
requires it: building reads no tile, no unit and no column, and allocates nodes
and nothing else.[^18] A caller composes an expression of any complexity without
paying for the world once.

**Evaluating one is the cost this report cannot state.** No measurement of a
selector evaluation exists on the target platform, because no selector exists.
Every cost figure in this project is derived rather than measured, and one
blocker holds that for the whole project.[^4]

**The invariant needs a test, and the test must not assert on time.** Asserting
on elapsed duration is forbidden here, and rightly.[^57] The check that works is
a count of allocated objects: call each method of the friendly tier against a
world of one size and against a world of four times that size, and assert that
the number of Python objects created is the same. A method that copies a column
fails it, and a method that allocates for each entity fails it, and neither
failure depends on how loaded the machine is.

---

## 7. What is recommended

### 7.1 The product record to write

**One record, and its audience is the game developer.** The registry allocates
the number. The report recommends the title:

> **A Python developer runs their first world without reading the source.**

This is a different need from the accepted record about documentation.[^58] That
record asks the package to explain itself. This one asks the package to have a
shape worth explaining. Neither answers the other, and the difference is visible
in the measurement: a working first program exists today and a reader could not
find it, which is a documentation failure; a second program that names a set is
impossible, which is an interface failure.

The six gate answers it should give.

1. **Who this is for.** A Python developer who has an idea for a simulation, who
   has never read this source, and who is not a systems programmer. Not the
   modeller and not the researcher: both reach the engine through the same
   package and both gain, and neither states the need as sharply.
2. **What the person cannot do today.** They cannot describe the things they
   want to act on. They can name a unit they already hold and an address they
   chose. Everything else costs them a loop that the project forbids and that
   nothing refuses.
3. **What good looks like.** Three checkable statements. A developer writes a
   program that founds a world, names a described set of units, gives that set
   one order, reads back where the set stands, and draws a picture, in twenty
   lines and with no loop over any entity. A developer who writes such a loop
   meets an error at that line, and the error names the call that answers the
   question. A developer who passes a ground kind where a resource belongs meets
   an error rather than a wrong result.
4. **What this does not do.** It does not store a run, it does not move a unit
   to a place the caller chooses, and it does not give the caller control over
   how the engine finds a set. The last is deliberate and a record already binds
   it.[^59]
5. **What it costs at the target scale.** Building a description costs the size
   of the description and never the size of the world. No call in the friendly
   tier allocates a Python object for each entity or for each tile. What one
   evaluation costs is unmeasured, and the record must express it parametrically
   and cite the blocker.[^4]
6. **Which blockers govern it.** The cost blocker governs every figure.[^4] A new
   blocker governs the ranking of the jobs, because no developer outside this
   repository has used the interface.[^12]

### 7.2 The decision records that should follow

The registry allocates each number. Each title below states the claim, as the
scope rule requires.[^60]

1. **The package ships a friendly Python tier, and it holds no state and copies
   no column.** The tier boundary, the two rules, and the two named crossings.
   This is the record everything else rests on.
2. **A selector refuses truth, length, iteration and indexing, and every refusal
   names the call that answers.** The Python surface of the accepted selector
   record. The four messages are part of the interface and a test asserts each
   one.
3. **A kind is an enumeration generated from the compiled module, never declared
   in Python.** One declaration site for one numbering.
4. **A fixed-point argument refuses a bare float, and the refusal names the
   exact constructor.** The determinism argument, and the reason the graded
   read's proposal is rejected.
5. **A column that may name nobody carries a mask, never an in-band sentinel.**
   Replaces three conventions with one, and gives the bulk read its answer to
   the dead identity.
6. **The viewer is an object that holds the picture size, and the world holds no
   frame state.** Moves the drawing surface off the world, and makes the Python
   shape agree with the Rust shape that already exists.
7. **An amendment to the record on documentation provenance**, saying where the
   prose of a pure Python member lives. This is a small record and it must exist
   before the tier does.

### 7.3 The order of the work

**First, and before any design work: publish the constructor's prose.** The
prose is written and reaches no Python object. Move it onto the class doc
comment and check that the reference build shows it. This is one change, it
repairs the failure both graded reads led with, and it is independent of every
other recommendation here.

**Second: the value types and the missing properties.** `Fixed`, the three
generated enumerations, `Axial`, `world.seed`, `world.faction_count`, and the
thread count on the world. Every one is small, none needs the selector, each
closes a hazard that a measurement or a graded read demonstrated, and together
they remove the second declaration site that the agent server was forced to
build.

**Third: the viewer.** It moves existing behaviour behind a better name and it
needs no new engine work.

**Fourth: the selector, with the read side built at the same time.** This is the
large piece and it is the one that answers job 2 and job 3. **Do not build the
write side first.** The write verbs already take sets. The gap is that nothing
describes a set and nothing reads one back, and a selector that only feeds verbs
would leave the loop exactly where it is.

**What can wait.** Persistence. The bulk read for the site and character
populations, because those tiers are bounded and a caller may walk them. The
plan report, which the selector record requires but which a first caller does not
need on the first day.

**What must not wait.** The record that says where the Python tier's prose
lives. It costs an afternoon before the tier exists and it costs a sweep
afterwards.

---

## References

[^1]: PRD-0021, a developer can use the control plane without reading its source. `docs/product/accepted/prd-0021-a-developer-can-use-the-control-plane-without-reading-its-source.md`
[^2]: Decision Record Scope, section 4.1. `.claude/rules/adr-scope.md`
[^3]: Project orientation, the target platform. `CLAUDE.md`
[^4]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^5]: Fresh reader review of the published reference, 3 September 2026. `~/cachette-reader-rounds/round-1-reference.md`
[^6]: Interface review of the published reference, 3 September 2026. `~/cachette-reader-rounds/round-1-interface.md`
[^7]: Findings register, FND-350. `docs/FINDINGS.md`
[^8]: Measured 3 September 2026 by `grep -rhoE "world\.[a-z_]+\(" python tests --include "*.py" | sort | uniq -c | sort -rn`, which reported 20 uses of `world.step(` and 7 of `world.soldier_tile(`.
[^9]: Measured 3 September 2026 on an x86-64 development machine, against the installed extension module. A list comprehension over 64 identities calling `World.soldier_tile` took 198.7 microseconds in total, which is 4.263 microseconds for each call. The multiplication to one million units is arithmetic on that figure and is not itself a measurement.
[^10]: ADR-0051, a selector is a lazy expression tree that Rust evaluates. `docs/adrs/accepted/adr-0051-a-selector-is-a-lazy-expression-tree.md`
[^11]: Measured 3 September 2026 by `uv pip tree`, which reported `mcp-types 2.1.1` depending on `pydantic 2.13.5` depending on `pydantic-core 2.46.5`. The project manifest declares `mcp` in the development group and declares `numpy` as the one runtime dependency. `pyproject.toml`
[^12]: Blockers register, BLK-045. `docs/BLOCKERS.md`
[^13]: Measured 3 September 2026 against the installed extension module. The program reported four factions seated of four, tick 10, 256 soldiers, four settlements, 4096 tiles painted, 256 soldiers painted and 64,000 non-zero pixels of 64,000.
[^14]: ADR-0040, Python is a control plane, not a data plane, decision D1. `docs/adrs/draft/adr-0040-python-is-a-control-plane-not-a-data-plane.md`
[^15]: The doc comment of `World.soldier_tile`. `crates/cachette-py/src/lib.rs`
[^16]: The `SurveyColumns` declaration, the `eligible` column. `python/cachette/_core.pyi`
[^17]: The `FrameError` class and the signature of `World.draw`. `crates/cachette-py/src/lib.rs`
[^18]: ADR-0051, decision D1. `docs/adrs/accepted/adr-0051-a-selector-is-a-lazy-expression-tree.md`
[^19]: ADR-0051, decision D2. `docs/adrs/accepted/adr-0051-a-selector-is-a-lazy-expression-tree.md`
[^20]: The package docstring, which states that no type refuses a loop. `python/cachette/__init__.py`
[^21]: ADR-0051, decision D4. `docs/adrs/accepted/adr-0051-a-selector-is-a-lazy-expression-tree.md`
[^22]: The doc comment of `World.despawn_soldiers`, which states that the set is all or nothing. `crates/cachette-py/src/lib.rs`
[^23]: ADR-0044, what copies and what does not is declared at the call site. `docs/adrs/draft/adr-0044-what-copies-and-what-does-not-is-declared-at-the-call-site.md`
[^24]: The `Presenter` structure and its doc comment. `crates/cachette-py/src/lib.rs`
[^25]: Interface review, section 11. `~/cachette-reader-rounds/round-1-interface.md`
[^26]: Findings register, FND-147. `docs/FINDINGS.md`
[^27]: Interface review, section 11, the migration paragraph. `~/cachette-reader-rounds/round-1-interface.md`
[^28]: Interface review, section 6a. `~/cachette-reader-rounds/round-1-interface.md`
[^29]: Interface review, section 8. `~/cachette-reader-rounds/round-1-interface.md`
[^30]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
[^31]: Findings register, FND-352. `docs/FINDINGS.md`
[^32]: Interface review, section 6c. `~/cachette-reader-rounds/round-1-interface.md`
[^33]: Measured 3 September 2026. The first entry of the column that `World.spawn_soldiers` returned was 4,294,967,296, and the arena hands out an index in the low half of the identity with a generation in the high half. `crates/cachette-py/src/lib.rs`
[^34]: ADR-0068, terrain is generated from the seed and is never stored as a map. `docs/adrs/accepted/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
[^35]: Recurring Defect Shapes, shape 3. `.claude/rules/recurring-defects.md`
[^36]: The `WorldSettings` dataclass and its docstring. `python/cachette/agent/session.py`
[^37]: ADR-0040, decision D1. `docs/adrs/draft/adr-0040-python-is-a-control-plane-not-a-data-plane.md`
[^38]: ADR-0040, decision D2. `docs/adrs/draft/adr-0040-python-is-a-control-plane-not-a-data-plane.md`
[^39]: ADR-0043, decision D4. `docs/adrs/draft/adr-0043-a-declared-tier-enforces-the-no-loop-rule.md`
[^40]: ADR-0051, decision D5. `docs/adrs/accepted/adr-0051-a-selector-is-a-lazy-expression-tree.md`
[^41]: ADR-0051, decision D3. `docs/adrs/accepted/adr-0051-a-selector-is-a-lazy-expression-tree.md`
[^42]: ADR-0051, decision D6. `docs/adrs/accepted/adr-0051-a-selector-is-a-lazy-expression-tree.md`
[^43]: ADR-0043, decision D2. `docs/adrs/draft/adr-0043-a-declared-tier-enforces-the-no-loop-rule.md`
[^44]: Measured 3 September 2026 by a search of the bindings crate for `SelectorError`, which found the import and the module registration and no raise. `crates/cachette-py/src/lib.rs`
[^45]: ADR-0043, decision D3. `docs/adrs/draft/adr-0043-a-declared-tier-enforces-the-no-loop-rule.md`
[^46]: ADR-0002, state holds no floating point number. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^47]: Measured 3 September 2026. The installed engine module is `cachette/_core.abi3.so` and the installed Pydantic core is `pydantic_core/_pydantic_core.cpython-312-x86_64-linux-gnu.so`. The bindings crate declares the stable interface feature for CPython 3.11 and later. `crates/cachette-py/Cargo.toml`
[^48]: The workspace manifest, which pins one version of the binding library. `Cargo.toml`
[^49]: ADR-0046, every error is typed. `docs/adrs/draft/adr-0046-every-error-is-typed.md`
[^50]: ADR-0092, the agent tool surface grows against a stated need. `docs/adrs/draft/adr-0092-the-agent-tool-surface-grows-against-a-stated-need.md`
[^51]: ADR-0040, the context section. `docs/adrs/draft/adr-0040-python-is-a-control-plane-not-a-data-plane.md`
[^52]: Fresh reader review, section B. `~/cachette-reader-rounds/round-1-reference.md`
[^53]: ADR-0107, the Python reference is generated from the compiled module, decisions D1 and D2. `docs/adrs/draft/adr-0107-the-python-reference-is-generated-from-the-compiled-module.md`
[^54]: Measured 3 September 2026 by `grep -rl "cachette" python tests --include "*.py" | wc -l`, which reported 16.
[^55]: Interface review, section 8, the cost paragraph. `~/cachette-reader-rounds/round-1-interface.md`
[^56]: ADR-0040, decision D3. `docs/adrs/draft/adr-0040-python-is-a-control-plane-not-a-data-plane.md`
[^57]: Testing Rules, section 3. `.claude/rules/testing.md`
[^58]: PRD-0021, what the person cannot do today. `docs/product/accepted/prd-0021-a-developer-can-use-the-control-plane-without-reading-its-source.md`
[^59]: ADR-0051, the consequences section. `docs/adrs/accepted/adr-0051-a-selector-is-a-lazy-expression-tree.md`
[^60]: Decision Record Scope, section 3. `.claude/rules/adr-scope.md`
