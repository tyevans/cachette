---
id: 0019
title: An agent can ask the running engine what it holds
status: Shaped
created: 2026-09-02
---

# PRD-0019 — An agent can ask the running engine what it holds

## Who this is for

An agent that works on this repository. It reads the source, writes code
against it, and reviews what another agent wrote.

This is a new audience for this directory. Every other record here serves a
person who builds something on the engine. This one serves the worker who
builds the engine. The two want different things: a builder wants the engine to
behave, and a worker wants the engine to answer.

A person who contributes to this repository reads this next. The need is the
same, and the answer serves both. The agent is named first because an agent
cannot open a window, cannot watch a picture, and cannot try something and see.
It can only read and ask.

## What the person cannot do today

**An agent cannot ask the engine a question. It can only read the source and
work out the answer.**

An agent that must know what a world holds has three routes today, and each one
fails differently.

It reads the Rust source and reasons about it. This gives a claim about what
the code should do, never a fact about what a world did. A reader of that claim
cannot tell the two apart, because they are written in the same words.

It writes a throwaway test, runs it, reads the output and deletes it. This
gives a fact, and it costs a build. The fact then goes into a comment, where
nothing checks it again. One test in this repository carries a comment. The
comment records where a resource sits in a generated world. It was measured
that way, because nothing else could ask. The comment has no defence against
the day the answer changes.

It runs the picture the watcher tools draw. An agent cannot see a picture.

The cost is not the time. The cost is that a claim nobody could check went into
the tree looking exactly like a claim somebody had. This has already happened
here: an agent designed a change to what a watcher sees. It went looking for the
detail the change would need. The engine held that detail, and nothing could
reach it.

## What good looks like

Each statement below can be checked.

- An agent builds a world, runs it, and reads what it holds, without writing
  code and without building anything.
- Every quantity an agent reads is one the engine computed. Nothing in the
  answer was worked out on the way to the agent.
- An agent can check an answer that is derived against the parts it came from.
  A total over a set of tiles equals the sum of those tiles, read one at a
  time, exactly.
- An answer states which of the two it is. An agent can tell a quantity it can
  verify from a quantity it must take on trust.
- An agent that asks the same question of two runs of the same world gets the
  same answer, whatever thread count either run used.
- An answer never grows with the size of the world. A question about a large
  world costs what the same question costs about a small one, or it names the
  bound it read within.
- An agent that meets a question with no answer learns that, and the gap is
  written down where the next agent finds it.

## What this does not do

This does not serve a person who builds a game on the engine. That person gets
the engine's own interface. Nothing here is a promise about that interface.
A tool that exists for an agent is not a feature of the product.

This does not put a picture in front of anybody. Watching the world run is a
different need and a different record holds it.

This does not answer every question. The answers grow one at a time, against a
question somebody asked. An agent that meets a wall today is the expected case,
not a defect.

This does not let an agent drive a large run. It is a way to ask a small world
a question, not a way to operate a simulation.

This does not replace a test. An answer read once is a fact about one moment. A
claim that must stay true needs something that fails when it stops being true.

## What it costs at the target scale

The target is 16.7 million tiles and one million units.

**The read cost must not follow the world.** Three shapes of question satisfy
that, and this record admits only those three.

A question about one thing: one address, one unit, one site. The engine already
answers each of these in a fixed number of steps. The cost is therefore the
same in the largest world as in the smallest.

A question about a summary the engine already maintains. The engine derives its
region summaries as it runs, so reading one is reading a value that exists.

A question about a bounded region the asker named. The cost follows what the
asker named and never the world. A question that could name the whole world in
one call is a pass over the world with no name on it. This record does not
admit one.

A question of any other shape is a question this need does not cover. The
answer is then a new capability of the engine, and that is a different record.

**The memory cost is the world, and it is the asker's choice.** A world lives
until the asking ends. An agent that builds a world at the target scale pays
for it until it stops asking.

**No figure here is measured.** Every cost statement above is a shape, not a
number. A blocker governs that.[^1]

## Which blockers govern this

- **No measurement exists on the target platform.**[^1] Every cost statement
  above states a shape and no number. The development machines are not the
  target, so a figure taken here would mislead.

No other blocker governs this record. The need is about what can be asked, and
every value it depends on is one the engine already holds.

## References

[^1]: Blockers register, BLK-007. `docs/BLOCKERS.md`
