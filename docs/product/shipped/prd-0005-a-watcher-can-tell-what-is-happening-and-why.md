---
id: 0005
title: A watcher can tell what is happening and why
status: Shipped
created: 2026-08-30
---

# PRD-0005 — A watcher can tell what is happening and why

## Who this is for

A developer who builds a strategy game on this engine, and who now watches a
window of coloured cells move.

The window exists. It shows a world, and the world moves. This record is for
the moment after that, when the developer asks a question about what they
see and the window gives no answer.

The other two audiences do not need this. A modeller reads numbers through
the control plane. A researcher compares hashes between runs. Only the game
developer must read the picture itself.

## What the person cannot do today

A developer cannot name anything they see.

The window shows coloured cells. The developer cannot say which tick the
world has reached. They cannot say where in the world they are looking,
because they scrolled and the world has no landmarks. They cannot say which
faction a colour belongs to. They cannot say how many units the world holds,
or how many of them the window shows.

The developer also cannot tell a slow engine from a slow drawing. The loop
steps and then draws, so one number covers both. When the picture stutters,
the developer has no way to say which half is at fault. The report that
answers this prints when the window closes, so the developer must stop
watching in order to read it.

Two failures follow, and both are silent. A defect that shows a plausible
picture stays invisible, because nothing states the numbers the picture
should agree with. A cost that grows with the world rather than with the
window stays invisible for the same reason.

## What good looks like

Each statement below can be checked.

- The window states the tick without the developer stopping the run.
- The window states where the person is looking, as a tile address, and how
  much of the world the window covers.
- The window names every colour it draws. A developer can point at a unit and
  say which faction it belongs to.
- The window states how many units the world holds, and how many of them it
  is showing. The two numbers are labelled, so a reader cannot mistake one
  for the other.
- The window states the cost of the step and the cost of the drawing as two
  separate numbers, while the run continues.
- Every number the window states is a number a reader can check against the
  picture, or against the report the run prints when it ends.
- A number the viewer cannot know is absent. The window never estimates.
- The panel does not hide the world. A developer can read the numbers and
  watch the units at the same time.
- **One command writes every number the window does not show, and the window
  names that command.** A developer who wants the whole record runs it and
  reads an image, without a display and without stopping the run.

### The window shows the moment, and a command writes the record

**This section is an amendment.** The record was written when the window held
one panel and the panel held everything. The window now shows what changes
moment to moment, drawn over the map, and one command writes the rest.

The statements above still hold. Four of them are met on demand rather than
continuously: the window names its colours, states where the person is
looking, and states the two cost numbers while a developer holds one key, and
it states the count of units in the window on the same key. The window naming
its colours on request is the window naming its colours. Nothing was dropped.

**The reason is the demonstration.** The panel grew past the height of the
window and cut, so the numbers below the cut were not reachable at all. The
project owner chose a heads-up display over a scrollable panel, and ruled that
the demonstration should look its best and that this record should be amended
where the product outgrows it.[^5] A product record is expected to change as
the product grows.

**What this costs a developer.** A number behind a key is a keystroke away
rather than in view. That is a real loss for diagnosis and a real gain for
watching, and the owner chose watching. A developer who diagnoses reads the
image the command writes, which holds every section at a height that never
cuts.

## What this does not do

- It does not explain a single unit's choice. A unit that moves at random has
  no reason to give. Behaviour that means something is a separate need.
- It does not let the developer command anything. The developer watches.
- It does not select a unit, inspect a tile, or answer a question the
  developer asks. It states one fixed set of numbers. A query is a separate
  need.
- It does not chart a number over time. It states the present value and the
  worst value so far.
- It does not replace the report that the run prints when it ends. That
  report states its conditions at length, and a panel has no room for that.
- It does not measure the target platform. A figure it shows describes the
  machine the developer is sitting at.
- It does not present the engine to a player. It is an instrument, not a user
  interface.

## What it costs at the target scale

The panel must cost nothing that the picture does not already cost.

Two of the numbers are the risk, and both are counts of units. A count of the
units in the window and a count of the units of each faction both look free.
Either becomes a pass over every unit in the world when it is written the
obvious way, and the world holds one million units at the target scale. A
pass like that costs more than the drawing it labels, every frame, and
nothing fails when it does.

Three properties follow. A solution must have all three.

- The cost of the panel grows with what the window shows, never with the size
  of the world or with the number of units in it.
- The panel adds no field to the engine. A number the panel needs is the
  viewer's to work out from what the engine already exposes.
- The panel adds no pass over the world. A count it states is a count that
  the drawing already produced, or a value the engine can give at once.

The third property is what bounds the first two. It also decides the content:
a number that would need its own pass is not shown at all, rather than shown
at a cost nobody sees.

No cost figure appears here, because nobody has measured one on the target
platform.[^1] The shape of the growth is the requirement. The figure is not.

## Which blockers govern this

- **No measurement exists on the target platform.**[^1] Every cost statement
  above states a shape, not a number. A frame rate, a draw count and a panel
  cost are all measurements, so this record states none of them.

  The panel itself displays measured figures. Those describe the machine the
  developer runs, which is a development machine. They do not answer this
  blocker, and the panel says so on its face.

No other blocker governs this record. The questions that settled the world
shape and the faction ceiling are answered, so this record states neither
parametrically.[^2]

## References

[^1]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^2]: Blockers register, BLK-013 and BLK-014. `docs/BLOCKERS.md`
[^5]: Decisions register, DEC-084. `docs/DECISIONS.md`
