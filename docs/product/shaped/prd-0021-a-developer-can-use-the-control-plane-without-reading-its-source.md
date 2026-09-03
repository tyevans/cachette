---
id: 0021
title: A developer can use the control plane without reading its source
status: Shaped
created: 2026-09-03
---

# PRD-0021 — A developer can use the control plane without reading its source

## Who this is for

A developer who builds a strategy game on this engine, and who has never read
the source of this project.

Python is the only part of the engine this person writes. The core is Rust, and
a rule keeps the game logic out of it. The developer builds a world, gives one
command to a set of units, steps the world, and reads what came back. Every one
of those actions crosses the Python package. That package is therefore the
whole product, as far as this person can see.

The other two audiences of this directory do not need this first. A modeller
and a researcher both reach the engine through the same package, so both gain
from the answer. Neither states the need as sharply. A modeller wants a large
agent count and asks about cost. A researcher wants a run to repeat and asks
about the hash. Only the game developer must learn the whole interface before
doing anything at all.

An agent that works on this repository is a fourth audience, and another
record holds it.[^1] That record asks the running engine to answer a question.
This record asks the package to explain itself before anything runs. Neither
answer replaces the other.

## What the person cannot do today

**A developer cannot learn the control plane without reading its source.**

The package explains itself, but only in places a newcomer does not reach.

One place is the docstring of each module. A reader gets a docstring after the
import works, and the import works after the reader has already found the
package, built it and installed it. A docstring also answers about one module.
Nothing tells the reader which module to open first.

Another place is the orientation document of this repository.[^2] It holds one
worked example. The example builds a world, spawns a set of units, gives one
order, steps the world and reads the result. It runs today. Nothing runs it, so
nothing fails on the day it stops running.

A third place is the compiled module. Its methods carry documentation, and a
contributor wrote that documentation in the Rust source. A developer who wants
to know which error a call raises reads a Rust file to find out.

Three costs follow, and each of them blocks work.

**The developer cannot find the boundary.** One rule shapes every program on
this engine: Python builds a selector and sends one command, and Python does
not walk a population. The package does not refuse the walk, and its own
docstring says so. A developer who never reads that docstring writes the walk,
gets a program that works on a small world, and meets the rule at the scale
where the program must not fail.

**The developer cannot tell what exists from what is planned.** The package
holds a compiled module, a window that shows a world, and a tool server that
serves this repository. Each surface is at a different stage. Nothing states
which surface a program may depend on.

**The developer cannot answer a question without the source.** What does a
returned column hold. In what unit is a value. Which error does a call raise,
and what does the error mean. Each answer exists, and each answer sits in a
file the developer must first learn to find.

## What good looks like

Each statement below can be checked.

- A reader who has never seen the package builds a world and runs one tick
  from the documentation alone, without reading the source.
- A reader who has never seen the package installs the package from the
  documentation alone. The instruction names each thing the reader must
  already have.
- A reader reaches the documentation without cloning this repository.
- Every example a reader can copy is executed by something that fails when the
  example stops working. No person has to notice by reading.
- Every public name of the package appears in the documentation. Something
  derives the list of public names from the package itself and fails when the
  two disagree.
- For every call the documentation describes, the reader learns what the call
  returns and which error it raises, without opening a Rust file.
- The documentation states the rule that governs every program on this engine:
  Python sends one command over a set, and Python does not walk a population.
  It states plainly whether anything enforces the rule.
- The documentation separates the surface a program may depend on from the
  surface that exists for this repository. A reader can tell which is which.
- The documentation states what the package cannot do yet. A reader who wants
  a missing thing learns that it is missing, and does not conclude that the
  reader failed to find it.

## What this does not do

This does not document the Rust core. A person who changes the engine is a
different audience with different questions, and that person already has the
source in front of them. This need is about the person who never opens it.

This does not document the decision records or the research reports. Those
explain why the engine is the way it is. A developer who builds a game on the
engine needs to know what the engine does. Both bodies of work have their own
homes and their own rules, and pulling them into this need would give it no
bound.

This does not document the surface that serves this repository. The tool
server exists for an agent that works on this project, and another record holds
that audience.[^1]

This does not choose how the documentation is made, where a reader finds it,
or what shape it takes. Those are architectural choices and a decision record
holds them. This record states only the need.

This does not promise that the interface stops changing. Documentation
describes what exists at a moment. A stable interface is a separate promise,
and this record does not make it.

This does not teach a person how to design a strategy game. It explains the
engine, and it stops there.

This does not replace a test. An example that runs proves that the example
runs. It proves nothing about whether the engine is correct.

## What it costs at the target scale

**The target scale of this record is not the world.** The engine targets 16.7
million tiles and one million units. No statement in this record depends on
that target. A document costs the same to write for a large world as for a
small one, and a reader reads it once.

**The cost is maintenance, and prose does not fail.** Every sentence of
documentation is a copy of a fact whose original lives in the code. When the
code moves, the copy states something false, and nothing reports it. The
reader who then acts on the copy pays the cost, and the reader cannot tell a
current sentence from a stale one, because both are prose.

This project has measured that cost on itself twice, and both cases are
recorded. In the first, one sentence about the state of the project spread
through the tree in each writer's own words. The fact changed in one day, and
the sweep was not affordable, so most of the copies stayed.[^3] In the second,
a register held a list, and the register claimed that a check kept the list
current. The check read a different list. The claim was convincing enough that
nobody looked, and the list was wrong.[^4]

Three statements bound the cost, and this record admits only documentation
that satisfies them.

A statement that something executes costs nothing to keep current, because it
fails on the day it becomes false. Prefer that statement.

A statement that nothing executes and nothing derives costs the reader's trust
the first time it is wrong, and it costs that trust for every other statement
in the document at the same time.

The cost follows the size of the interface, and never the size of the world. An
interface that grows costs more to document. A world that grows costs nothing.

**No figure here is measured.** Every cost statement above states a shape and
no number. A blocker governs the figures in this project.[^5]

## Which blockers govern this

**One blocker governs every cost figure.**[^5] It states which figures the
project measured and which it derived. This record states no figure, so no
value here waits on it. A performance claim in the documentation does wait on
it, and a writer who wants to make one cites the blocker rather than inventing
a number.

**One blocker governs a rule the documentation would describe.**[^6] It asks
whether an upgrade changes hands when the ground under it changes hands. The
documentation must not answer while the question is open. A writer who reaches
that behaviour states that the project has not decided it, and cites the
blocker.

No other open blocker governs this record.

## References

[^1]: Product requirement record 0019, an agent can ask the running engine what it holds. `docs/product/shaped/prd-0019-an-agent-can-ask-the-running-engine-what-it-holds.md`
[^2]: Project orientation, the Python example. `README.md`
[^3]: Findings register, FND-223. `docs/FINDINGS.md`
[^4]: Findings register, FND-242. `docs/FINDINGS.md`
[^5]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^6]: Blockers register, BLK-034. `docs/BLOCKERS.md`
