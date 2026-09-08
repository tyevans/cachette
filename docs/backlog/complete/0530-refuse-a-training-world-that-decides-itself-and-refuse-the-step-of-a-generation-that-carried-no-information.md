---
id: 0530
title: Refuse a training world that decides itself, and refuse the step of a generation that carried no information
status: complete
created: 2026-09-08
implements: []
changes: []
creates: []
serves: [PRD-0056]
blocked-by: []
---

## Why

**Some seeds build a world that decides the game before a policy can matter.**
The project owner saw it while watching the game: a spawn with a great deal of
water destroys one or more clans at once. Two live training runs both reached
one generation, both drew one seed, and both recorded a mean and a best that
were exactly the loss value, with a spread of zero.

The seed filter was the only guard, and it did not guard this. It built a
world, seeded it, and kept the seed when the seeding did not raise. **The
engine refuses a world only when it seats nobody**, so a world that seats one
faction of three passed the filter and ended on the first tick by domination.

A generation whose candidates all score the same number is worse than a wasted
generation. The rank of an equal set comes from the order of a stable sort, so
the set ranks by candidate index, and the update takes a step of the full
learning rate along a direction that the perturbation noise alone chose.
Nothing in the log said that the step carried no information. A finding holds
the measurement.[^1]

## Impact review

**Governed by.** No decision record states how a run chooses its seeds or what
it does with a generation that scored flat. ADR-0192 D1 states that a sharded
generation and a single-process generation give the same weights for the same
seed, and the refusal of a step sits above the combination, so both paths take
it. PRD-0056 states that a learner plays one faction against the built-in
controllers, and a world that seats one faction is not that game.[^2] [^3]

**How the work honours them.** The filter walks upward from the start as it
did, and its answer is a pure function of the founding report, so the same
start and the same count give the same seeds. The refusal of the step reads
the spread of the score that the update ranks, which both the sharded path and
the single-process path produce. No simulated value and no aggregate gains a
float, because the score, the spread and the weights were already learner-side
floats that never enter a world.

**Changes.** None.

**Creates.** None. The three tests of the scope rule do not all hold.[^4] A
future contributor could choose otherwise, so the first holds. The second does
not: both rules are one predicate each, and reversing either costs nothing
structural. The third does not either, because each predicate carries its
reasoning where a reader of the code meets it, and the finding carries the
measurement. The counter-test asks whether the decision governs determinism,
and it does not: the filter stays a pure function of the report, and the
refusal removes a step rather than reordering anything.

**Blockers.** BLK-050 governs what a reward weight is worth, and this work
states no weight.

## Done when

- The seed filter refuses a world that seats fewer factions than the run asked
  for, and a world with a seat that reaches no food.
- A test builds the pool that a run draws and shows that the seed the owner
  found is gone, and a test-only switch puts the old filter back and shows the
  seed return.
- The trainer takes no step from a generation whose candidates all scored the
  same number, and it names that generation where a reader of the log and a
  reader of the run report both see it.
- A test drives the trainer over a world that decides itself and shows the
  centre standing still, and a test-only switch puts the old update back and
  shows the centre move.

## Outcome

**Both are built, and the filter alone would not have been enough.** A sweep
played nine policies on each pool seed that ended early, and eleven of three
hundred seeds gave nine equal returns. The founding report names nine of the
eleven. The other two seat every faction and feed every seat, and the rival
factions collapse on their own near tick 200. The refusal of the step is the
only guard that covers those.

**Every figure a training run stored before this change came from a different
pool.** The filter refuses about nine percent of the raw seeds above the base
the runs use, and the validation set moved as well, so no earlier score can be
compared with a later one. The stored policy index says so.[^5]

**The filter also removes some worlds where a policy did matter.** A world
that seats two factions of three is a different game rather than a harder one,
and the run asks for three. That is the reason to refuse it, and the cost is
real.

## References

[^1]: Findings register, FND-666. `docs/FINDINGS.md`
[^2]: ADR-0192, a generation is scored in shards and combined in candidate order. `docs/adrs/draft/adr-0192-a-generation-is-scored-in-shards.md`
[^3]: PRD-0056, a learner plays one faction against the controllers. `docs/product/accepted/prd-0056-a-learner-plays-one-faction-against-the-controllers.md`
[^4]: Decision Record Scope, section 1. `.agents/rules/adr-scope.md`
[^5]: Stored policies index. `checkpoints/README.md`
