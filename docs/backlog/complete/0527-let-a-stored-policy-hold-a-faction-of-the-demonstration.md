---
id: 0527
title: Let a stored policy hold a faction of the demonstration
status: complete
created: 2026-09-07
implements: [ADR-0154 D2, ADR-0154 D3, ADR-0176 D1, ADR-0040 D1, ADR-0002 D1]
changes: []
creates: []
serves: []
blocked-by: []
---

## Why

**A trained policy could be read as a number and never watched.** A training
run reports a score and writes a weight file, and a report names the verbs a
policy chose. None of those says whether the policy does something sensible or
wins by an accident of the scoring. A person judges that by watching.

The demonstration already gives one faction to a person at the keyboard. It
reads the action table of the engine, marks it against the legality mask, and
sends one action integer through one verb.[^1] The learner reads the same
observation and the same mask, and its policy returns the same action
integer.[^2] The seam was therefore complete, and nothing joined the two ends.

## Impact review

**Governed by.** ADR-0154 D2 fixes the observation as one flat integer array
whose length follows the world parameters.[^3] ADR-0154 D3 binds the reader to
the sight rule, so a policy reads what a person reads. ADR-0176 D1 fixes an
action as one integer over a bounded table, so the policy and the person cross
the boundary the same way.[^4] ADR-0040 D1 says the control plane never loops
over entities, and this work sends one integer for one faction.[^5] ADR-0002 D1
forbids a floating point value in simulated state, and the weights stay on the
control plane.[^6]

**Changes.** None. No record changed and no record was superseded.

**Creates.** None. The cadence rule is visible in the refusal the code raises,
and a reader of the code sees why, so it does not meet the test for a
record.[^7]

**Blockers.** None. The work states no cost figure.

**Precedent.** Recurring defect shape 1 says that one value declared in two
places with no check is the shape this project meets first.[^8] The cadence of a
checkpoint is declared in the weight file and in the manifest beside it, so the
reader takes both and refuses a checkpoint that states two different values.

## What the work decided

**The cadence follows the checkpoint, and a checkpoint that names none is
refused.** A learner acts once and then lets the world run a fixed number of
ticks. A policy played at another rhythm plays a game it never learned, and
nothing raises. The pilot therefore counts its own ticks against the interval
the checkpoint names. It holds no constant, and it refuses rather than guessing.

**A policy takes no turn.** The turn of a person freezes the clock while they
read. A policy needs no reading time, and a turn would tie every policy of one
world to one cadence. The two mechanisms are therefore separate, and several
policies at several cadences play one world beside a person.

**A checkpoint of another world stops the run.** The observation length counts
the cells of a lattice, so two worlds of different extents can hold the same
length and mean something else at every position. The reader checks the extent
and the faction count beside the two lengths and the two schema versions, and it
names every entry that disagreed. Nothing falls back to the built-in controller,
because a faction that reverted in silence would be watched as if it were the
policy.

## Acceptance

- A watcher names a weight file on the command line, and a faction plays it.
- A watcher names the option again, and a second faction plays a second file.
- The title block names the file that holds each faction.
- A checkpoint trained against another world stops the run with a message that
  names the entry that disagreed.
- A test drives the frame loop of the demonstration and shows the faction
  acting. Breaking the call in the loop turns that test red.

## What this does not do

It does not let a policy play a demonstration to the end of a game. The
observation reader refuses once three seated policies have acted for about a
hundred ticks, and that condition is engine side.[^9] A separate item holds
it.[^10]

## References

[^1]: The player seat of the demonstration. `python/cachette/demo/player.py`
[^2]: The learner environment. `python/cachette/learn/env.py`
[^3]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
[^4]: ADR-0176, an action integer is a mixed radix over the argument positions each verb declares. `docs/adrs/accepted/adr-0176-an-action-integer-is-a-mixed-radix-over-the-positions-a-verb-declares.md`
[^5]: ADR-0040, Python is a control plane, not a data plane. `docs/adrs/draft/adr-0040-python-is-a-control-plane-not-a-data-plane.md`
[^6]: ADR-0002, state holds no floating point number. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^7]: Decision Record Scope, section 1. `.agents/rules/adr-scope.md`
[^8]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
[^9]: Findings register, FND-644. `docs/FINDINGS.md`
[^10]: Backlog item 0528, let a stored policy play a demonstration to the end of a game. `docs/backlog/complete/0528-let-a-stored-policy-play-a-demonstration-to-the-end-of-a-game.md`
