# Cachette

**A world of one million people, each with their own hunger. The same run,
every time.**

Cachette is a simulation engine for very large worlds. The core is Rust.
You drive it from Python.

> **Status: design.** There is no engine yet, and nothing here has been
> measured on hardware. This document says what the project is building
> and why.

The world is a hex grid of 16.7 million tiles. It holds one million units.
A unit is a person, not a number inside a formation. It stands on a tile,
carries what it owns, gets hungry, and works a trade.

## Build this with it

A grand strategy game where a famine is not an event you scheduled. The
harvest failed in one province, so the price of grain rose, so the garrison
there could not be fed, so it marched. You did not write those arrows. You
wrote what a farm makes, what a soldier eats, and what a cart costs to
move.

Or an economic model with a million households instead of a thousand. Or
any study where you run the world a thousand times, change one number, and
know that everything else stayed identical.

## Who it is for

- **A game developer** who wants a simulated world under the game, and
  will build the rendering.
- **A modeller** who has outgrown a general agent framework and needs the
  agent count without giving up control of the arithmetic.
- **A researcher** who must reproduce a run exactly, months later, to
  defend a result.

You need Python. You need Rust only to extend the engine.

## What a unit does

A unit has needs, and the needs grow every tick. Food, rest, safety.

A unit has a place. It works a trade at a site: a woodcutter's hut, a
forge, a tavern. A site turns labour into goods, and it has room for a
fixed number of workers. Nothing creates jobs to fit the population, so a
settlement runs a surplus of work or a shortage of it. When a
unit cannot meet a need where it stands, it moves to where the need can be
met, it takes work that is open, or it goes without. Going without emits an
event, so the cost of a famine is in the log and not only in the outcome.

Not every unit acts every tick. The engine spreads the expensive work
across frames by region, so a million units run without a million decisions
in one frame.

You write the content in Python. It compiles to integer tables once, when
you build the world, and the engine reads only the tables after that.

```python
hunger = world.define_need(
    grows_by=2,                       # per tick, integers only
    satisfied_by=grain,
    when_unmet=Drain(health, by=1),   # a rule, not a sentence
)

world.define_trade(
    "woodcutter",
    at="hut", consumes={}, produces={"wood": 3}, per_tick=True,
)
```

Nothing scripts the outcome. Ten thousand soldiers march, their food need
crosses a threshold on the same tick, and ten thousand soldiers look for
food. No one wrote the famine. It fell out of the numbers.

## The same run, every time

Run a tick on one thread. Run it on twelve. The event logs match byte for
byte.

This is the constraint the engine is built around, and it is the only one
that cannot be added later. Simulated state holds no floating point number,
because float addition gives a different answer when the order changes.
Every random draw comes from a counter, keyed on the system, the frame, the
entity and the draw. Events are plain bytes with declared padding, so a
state hash means what it says. No result depends on which thread finished
first.

The promise is scoped, and the scope matters. **One binary gives one
answer at any thread count.** Across a different processor or a different
compiler, the engine does not promise a matching hash, and a project that
tells you otherwise is selling you something. Reproducing a study means
keeping the binary, not only the seed.

Two tests hold the line. One runs the same tick at 1, 2 and 12 threads and
compares the logs. The other hashes the whole world each frame against a
stored file.

A run you can reproduce is what separates a world you can study from a
world you can only watch.

## You write intent. Rust does the work.

You drive Cachette from Python. You never write the inner loop.

```python
north = world.areas.named("north")         # you define an area once
army  = world.units(faction="ashfall", kind="soldier").within(north)

army.march_to(harbour)                     # nothing ran until this line

for event in world.step():                 # events arrive in batches
    ...
```

That is one command for forty thousand soldiers. Python does not loop over
them and does not hold them. It builds a description of a set and hands it
to the engine.

The engine then chooses an algorithm for the whole set, which is the point.
Three kinds of work get a whole-set algorithm: movement toward a shared
destination, which becomes one flow field instead of forty thousand path
searches; spreading a quantity across terrain, which becomes one diffusion
pass; and recomputing what a faction can see. Everything else runs as a
parallel loop over the set, in a fixed order.

The world steps with the Python interpreter lock released. No Python runs
while the world runs.

## Ask the continent, not the tiles

The engine keeps the world at three levels. Level 0 holds every tile and
every unit. Level 1 summarises blocks of tiles at the scale of a city.
Level 2 summarises level 1 at the scale of a region.

Level 0 is the only truth. The upper levels are derived, and they are
exact:

- **A total** is the sum of its parts. The grain in a region equals the
  grain in its tiles, with no drift and no rounding.
- **An extreme** is the extreme of its parts. The driest tile in a
  continent is the driest tile in one of its cells.
- **An average** is stored as two totals and divided on read. Averaging
  the averages of unequal areas gives the wrong answer, so the engine
  never does it.

```python
world.regions().where(grain < 100).ids()      # answered at level 2
```

That query starts at level 2 and descends only where the answer is still in
doubt. A region that is entirely above the threshold is dismissed whole,
and so is a region entirely below it. A question that selects individuals
still visits individuals. The pyramid prunes the search. It does not
abolish it.

The same levels are how you draw the world. Ask for a tile buffer at the
level that matches the zoom, and you get one array copied out, sized to the
screen rather than to the world.

## What it is not

Cachette does not draw anything. It has no renderer, no input handling and
no assets. It simulates a world and hands you the numbers to
draw. The picture is yours to make.

It is not general. It buys its scale by knowing that the world is a hex
grid, that units are dense, and that arithmetic is exact. A simulation that
wants none of those is better served elsewhere.

It does not run across machines. The target is one large server.

## No numbers yet

This document gives no tick rate and no memory footprint, because neither
has been measured. The design derives its budgets from the data layout,
and deriving is not measuring. A derived figure that gets repeated becomes
a fact nobody checked, so this project does not print one. Numbers appear
here when they come off the target hardware.

## For contributors

The engine targets AWS Graviton servers. Development happens on x86-64 and
Apple Silicon, which have a different cache line size, so a local
performance measurement misleads.

```
just              # list the targets
just setup        # build the extension into a local environment
just check        # everything a commit must pass
```

The decision records under `docs/adrs/` hold the reasoning, and the
contribution guide explains the layout, the testing policy, and how to add
each kind of test.[^1]

## References

[^1]: Contribution guide. `CONTRIBUTING.md`
