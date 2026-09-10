---
id: 0539
title: Restart a reclaimed training run on a new instance
status: proposed
created: 2026-09-10
implements: []
changes: []
creates: []
serves: []
blocked-by: []
---

## Why

**A reclaimed training run now keeps its weights and cannot use them.** The
launcher rents a spot instance, trains the learner on it, and follows the run
from this machine. A spot instance can be taken back at any time. The follower
copies the resume point, the best centre and the copy of every generation of
every strategy to this machine at every poll, so a reclaim costs the work of
one poll.[^1]

That closes the loss of the weights. It does not close the loss of the run. A
person who wants the search to continue must rent a new instance, and the new
instance starts every strategy from nothing. The resume points sit in the run
directory of the reclaimed run and nothing sends them anywhere.

**The trainer already reads a resume point.** It takes a flag that continues
each strategy from the centre of its last generation, and the launcher uses
that flag when a strategy dies and the machine survives.[^2] The missing part
is the copy in the other direction: this machine sends the resume points to a
new instance, and the launcher runs the trainer with that flag on the first
attempt rather than on the second.

One run was reclaimed two hours into a cap of six hours. It had reached the
best held-out figure this project has measured, and the whole of that work was
lost. The fetch stops that happening again. This item stops the search
restarting from nothing after it.

## The architectural impact review

This item is not refined. Refining it is the work. The review must answer four
questions.

**Which records govern it.** No decision record governs the launcher. The
review must confirm that, and it must say whether a restart that carries a
policy across two engine builds needs one.

**Whether a resume point is portable across an engine build.** The trainer
refuses a checkpoint written under another feature transform, and it names the
transform in the file.[^2] A new instance may build a different engine. The
review must say what a restart does when the two disagree: refuse, or start
from nothing and say so.

**What a restart does to the controller baseline.** A run measures the
built-in controller over the held-out seeds to set the bar the policy must
beat, and it keeps that measurement in a cache keyed on the engine build, the
world, the seeds and the objective.[^1] A restart must carry the cache, or it
pays for the bar again.

**What a restart does to the generation count and to the cost estimate.** The
plan the launcher prints before it spends money states the generations a run
delivers inside the cap. A restart that continues an earlier search delivers a
different number, and a figure that describes no run is worse than none.

## What the work delivers

A way to take a run directory whose instance is gone, and continue the search
on a new instance.

The work must state the interface. One option is a flag on the launcher that
names the directory of the reclaimed run. Another is a mode that reads the
newest run directory. The review chooses.

The work must send four things to the new instance: the resume point of every
strategy, the best centre of every strategy, the controller baseline cache,
and the arguments the reclaimed run used. The launcher already sends a wheel
cache and a baseline cache, so the mechanism exists.

The run directory also holds the copy of every generation. The trainer resumes
from the resume point alone, so a restart need not send those copies.[^2]

The work must prove itself without renting a machine. A probe drives the fetch
of the launcher against a stand-in for the copy tool, and a restart probe can
be built the same way.[^3]

## What this item does not do

It does not make a run survive a reclaim by itself. Nothing restarts the run
without a person, and the launcher buys a one-time spot request for that
reason. This item shortens what a person does after a reclaim. It does not
remove the person.

It changes no engine code and no trainer code. The trainer already resumes.

## References

[^1]: The training launcher. `scripts/graviton-train.sh`
[^2]: The trainer, the checkpoint of a run. `python/cachette/learn/train.py`
[^3]: The fetch probe of the training launcher. `scripts/train-fetch-probe.sh`
