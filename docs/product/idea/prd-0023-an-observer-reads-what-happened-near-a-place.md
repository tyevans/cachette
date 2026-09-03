---
id: 0023
title: An observer reads what happened near a place
status: Idea
created: 2026-09-03
---

# PRD-0023 — An observer reads what happened near a place

## Who this is for

A researcher who gives an agent a limited view of the world, and who must
decide what that agent could have noticed.

## What the person cannot do today

**A caller cannot ask what happened near an address.**

The engine records what happened. It hands that record back whole: every event
of a kind, from the first tick to the current one. A caller that wants the
events near one person at one tick reads the whole record and drops what does
not apply.

The engine already answers the same shape of question about state. A caller
names an address and a radius and reads what that window holds. It cannot name
an address and a span of ticks and read what happened there.

This has three costs.

**The filter runs in the wrong language.** The project holds that the control
plane sends one command and the core resolves it. A caller that narrows a
global log to a window is looping over the world in Python, which is the thing
the rule exists to prevent. It works for ten observers and it does not work for
a thousand.

**The caller pays for the whole run on every tick.** The record only grows. A
caller that reads it once a tick reads more each time, so the cost of watching
a run rises with the length of the run rather than with what happened in it.

**An observer's limits are not the engine's.** What one person could have
noticed is a claim about distance and about time. Today that claim is made
outside the engine, so nothing in the engine can check it, and two callers can
disagree about who saw what.

## What good looks like

- A caller names an address, a radius and a span of ticks, and reads the events
  that happened inside all three.
- The answer holds the same fields the whole record holds, so a caller that
  already reads events needs no second reader.
- A window that clips at the edge of the world reports the window it used.
- The cost falls with the size of the window, so a small window is cheap on a
  large world.
- Reading the same window twice returns the same events, and reading it changes
  nothing.

## What this does not do

It does not decide whether anybody noticed. Distance is not perception, and the
judgement of who saw what belongs to the caller.

It does not summarise. It returns the events, not a description of them.

## Which needs it sits beside

A faction that sees only what its own units observe needs the same question
answered for a faction rather than for an address.[^1] This record is the
narrower one and does not replace it.

## References

[^1]: PRD-0001, a faction sees only what it observes.
`accepted/prd-0001-a-faction-sees-only-what-it-observes.md`
