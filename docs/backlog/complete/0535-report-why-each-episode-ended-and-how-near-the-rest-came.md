---
id: 0535
title: Report why each episode of a generation ended, and how near the losing ones came to a win
status: complete
created: 2026-09-09
implements: [ADR-0148 D1, ADR-0154 D6]
changes: []
creates: []
serves: [PRD-0001]
blocked-by: []
---

## Why

**A run reports the share of episodes a seat won, and nothing else about the
end of a game.** A policy that reaches most of a wonder in every game and a
policy that reaches almost none of one both report a win share of zero. The
two states call for opposite decisions: run for longer, or change the reward.

**A win is a threshold event and the reward is continuous.** A win share is
therefore expected to stand still and then jump. An instrument that only
reports the share says nothing until the jump has already happened.

**The project has measured this by hand once.** A finding took 24 games,
counted the path each one ended on, and overturned two things the register
held about the balance.[^1] That count is the figure a run should print every
generation, and no run printed it.

## The architectural impact review

**The records that govern this work.** ADR-0148 D1 governs the end record and
the running value of each faction on each path. ADR-0154 D6 governs the
observation as a schema-declared bounded table, which is where every progress
share comes from. PRD-0001 governs what a faction may read, and this work
reads nothing outside it.

**The records this work changes.** None. It adds an instrument and changes no
rule of a game and no term of a reward.

**The records this work creates.** None. The three tests for a record fail.
The engine already publishes every quantity, so a future contributor cannot
reasonably choose otherwise, the arrangement is cheap to change, and the
reasoning fits in the module that holds it.

**The blockers that hold it.** None.

## What the work found

**The engine already published everything.** The end record of a world names
the path a reader fired on. The observation of a faction carries one progress
share for each of the four win paths, and one for the faction that leads each
path. The record of an episode already read every one-position signal of the
observation and stored it. Nothing had to be added to the simulation.

**One thing was missing, and it was a list.** No reader gave the control
plane the names of the win paths. A report that names a path a generation
never reached needs the whole list, so a report either held a copy of the
list or dropped the path. The binding now answers the list from the reader
that turns a stored number into a path, so the engine states it once.

**Two of the four shares do not end at one.** The wonder share and the renown
share are fractions of a threshold. The domination share reads the seats a
faction holds over the seats the reader asks for, and the other clause of
that reader stays unpublished because a share of it would state the unit
count of a rival the faction never observed. The territory share reads the
held ground over the passable world, and that reader holds no threshold at
all. A reader of the instrument must know which path it is looking at, and
the module states it.

**The rule for reading a path from a world was declared three times.** Two
measurement scripts each held their own copy, and each held its own name for
an episode with no end record. The learner package now holds one reader and
one name, and both scripts call it. The name they report changes with that,
because the two copies had to agree on one string.

## What the work delivers

A generation prints two further lines: the share of its episodes that ended
on each path or held no end record, and the median, ninth decile and highest
value the seat reached along each path. The report file holds the same
figures for the leading faction as well, and the record of each episode names
its ending.

The instrument costs one pass over the episodes of a generation. It reads no
world and steps no tick, because the record of an episode read every figure
when the episode ended.

## References

[^1]: Findings register, FND-710. ``docs/FINDINGS.md``
