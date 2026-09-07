---
id: 0520
title: Wrap the engine as an environment one learning stack can drive
status: proposed
created: 2026-09-06
implements: []
changes: []
creates: []
serves: [PRD-0056]
blocked-by: [BLK-007]
---

## Why

**This is the item that turns the parts into a harness.** A product record asks
for a learner that plays one faction against the built-in controllers.[^1] The
parts it needs are the fog-scoped readers, the flat observation and its schema,
the action table and the legality answer, the option weight setter, the reward,
and the batch step. Each has an item of its own. **None of them is a loop a
researcher can run.**

**The wrapper is the loop.** It seeds a world, sets the external control flag
for one faction, reads the observation, offers the legality answer, sends one
action integer, steps, and reads the reward. Every one of those calls belongs to
another item, and this item owns only the order between them and the shape a
learning stack expects.

**A runner of this shape already exists to copy.** The balance runner drives many
seeds through the engine and collects results, and it is the nearest thing in
the package to a training loop.[^2] [^3]

**The wrapper must call no full-information reader, and a check must catch an
adapter that does.** An accepted record already rules that a world-wide reader is
not a per-faction reader and that nobody may read it as one.[^4] A design
document says the same of the environment core.[^5] A survey lists the readers
that hand out the truth of the whole world, and says the work is to add
faction-scoped readers beside them rather than to delete them.[^3] **So the
check belongs to this item**, because this item is the first thing that could
call the wrong reader.

**One cheaper loop exists, and it is not this.** The option weights and the
reward with a crude wrapper close a learning loop on a small world today, with
no fog and no schema. That loop shows a learner the truth of the world, so it
breaks the first checkable statement of the product record.[^1] It is a
throwaway spike, and anybody who offers it must say so.[^3]

## What is missing before this can be refined

- **Which learning stack the wrapper answers to, and whether it names one at
  all.** A design document says one environment core serves every learning
  stack, which means the core is not a wrapper for a named library.[^5] The work
  must decide whether it ships the core alone, or the core and one adapter, and
  a core with no adapter is a capability that nothing invokes.[^6]

- **Where the check that keeps the wrapper off the full-information readers
  lives, and what it reads.** A check over the import list of one package is one
  shape. A check over the calls the wrapper makes is another, and it is the one
  that catches an adapter that reaches through a held world object. The work must
  say which, and a check that cannot see the second shape should say so rather
  than claim the rule.

- **What the wrapper does with a run that has ended.** A game end is recorded
  once and stops the controllers.[^7] A learning stack expects a signal that an
  episode ended and a way to start the next one. The work must say whether the
  next episode reseeds the world or builds a new one, and reseeding a world that
  holds state from the last run is the mistake to design against.

- **How many worlds one loop drives.** The batch step exists as an item of its
  own, and a wrapper written for one world does not become a wrapper for many by
  itself.[^8] The work must decide whether it takes the world count as a
  parameter from the start, or whether a second item widens it later.

- **What one decision costs end to end.** The loop crosses the boundary for the
  observation, the legality answer, the action and the step. Every figure for
  that is derived, and one blocker says every cost figure of this project stays
  derived until the target platform measures it.[^9] The work must express the
  cost parametrically and cite that blocker.

- **How a test drives the real loop.** A test that builds the wrapper and calls
  its parts proves that the parts work. It does not prove that a learning stack
  can drive it. The work must say what stands in for the learner in the test, and
  a test that only the wrapper's own author could run is not evidence.[^10]

## Done when

Filled in when the item moves to `refined/`.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: PRD-0056, a learner plays one faction against the controllers. `docs/product/accepted/prd-0056-a-learner-plays-one-faction-against-the-controllers.md`
[^2]: The balance runner of the Python package. `python/cachette/balance/`
[^3]: Research report 31, the state of the learner surface. `docs/research/reports/31-the-state-of-the-learner-surface.md`
[^4]: ADR-0059, fog storage grows with observed area, not with world area, decision D6. `docs/adrs/accepted/adr-0059-fog-storage-grows-with-observed-area.md`
[^5]: Design, one environment core serves every learning stack. `docs/superpowers/specs/2026-09-05-reinforcement-learning-interfaces-design.md`
[^6]: Recurring Defect Shapes, shape 3. `.agents/rules/recurring-defects.md`
[^7]: ADR-0148, a game end is recorded once and stops the controllers, decision D2. `docs/adrs/accepted/adr-0148-a-game-end-is-recorded-once-and-stops-the-controllers.md`
[^8]: Backlog item 0519, step a batch of worlds in one call, in index order. `docs/backlog/proposed/0519-step-a-batch-of-worlds-in-one-call.md`
[^9]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^10]: Testing Rules, section 5. `.agents/rules/testing.md`
