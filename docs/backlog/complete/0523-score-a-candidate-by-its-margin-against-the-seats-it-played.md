---
id: 0523
title: Score a candidate by its margin against the seats it played, and rotate the seats
status: complete
created: 2026-09-07
implements: [ADR-0004 D1]
changes: []
creates: []
serves: [PRD-0056]
blocked-by: [BLK-050]
---

## Why

**A sixteen-generation training run on the target platform improved nothing,
and the diagnosis is measured.** Three readings say why.[^1]

**The objective took seven values.** Every population spread of that run was an
integer multiple of about 673, within two percent, on sixteen generations of
sixteen. The outcome term of the reward is two thousand for a win and minus two
thousand for a loss, and the shaped term of the conquest strategy pays a few
tens over a whole episode. A candidate's score is therefore its win count over
the seeds of the generation, and with six seeds that quantity takes seven
values.

**The centre never moved.** The validation score took exactly two values over
sixteen generations, one step apart, and the fitted trend was negative.

**The seat decides who wins.** With the same built-in controller in all three
seats of twenty-four held-out worlds, seat 0 won 9, seat 1 won 4 and seat 2 won
11. That is start-position luck with opponent quality held constant.

**A multi-seat harness already existed and nothing called it.** The module
holds the plan, the seated game, the vector and the runner, and it is tested.
No trainer imported it. That is the inert capability shape the recurring defect
rule names third, and closing it is most of this work.[^2]

## The architectural impact review

**The records that govern this work.** ADR-0004 D1 governs iteration order: no
result may be ordered by thread completion or by work-stealing order. PRD-0056
states the need, that a learner plays one faction against the controllers.[^3]
PRD-0001 governs the fog rule, and the seated game already honours it through
the readers that answer for one faction.[^4]

**The records this work changes.** None. The scoring of a candidate is a
property of the learner-side search. It enters no world, no event and no state
hash, so no record binds it.

**The records this work creates.** None. The choice of margin formula is a
decision, and it is recorded in the decisions register rather than in a record,
because a future contributor changing it costs nothing that a record would
buy.[^5]

**The blockers that hold it.** BLK-050 holds the rules of the downstream game,
which includes what a win is worth against a tile. The work therefore changes
no weight. It changes how two candidates are compared, which is a property of
the search and not of the game.

**Has this been settled before?** The findings register held nothing on this
before the work. It now holds what the measurement said.[^1]

## What the work does

**A candidate is scored by its margin against the other learner seats of its
own world.** The score is its own return minus the mean return of the other
learner seats of that world. Two candidates in one game share the map, the
weather and the opponents, so the difference between their returns holds almost
none of the variance that either return holds alone.

**The seat turns by one position at each seed index.** A pair that held one
seat for a whole generation would carry that seat's advantage into a ranking
that compares every pair. Both halves of a pair still share a seat and a seed,
so the antithetic difference is unchanged.

**The run keeps an absolute yardstick.** A relative score is zero on average by
construction, so it cannot tell a population that improved from one that got
worse together. The trainer plays the built-in controller in the learner's own
seat on the validation seeds, once, and reports every validation score against
that number on every generation.

## What it does not do

It changes no reward weight. It does not shrink the outcome term. The project
owner chose the relative score over that, deliberately.

## References

[^1]: Findings register, FND-630. `docs/FINDINGS.md`
[^2]: Recurring Defect Shapes, section 3. `.agents/rules/recurring-defects.md`
[^3]: PRD-0056, a learner plays one faction against the controllers. `docs/product/accepted/prd-0056-a-learner-plays-one-faction-against-the-controllers.md`
[^4]: PRD-0001, a faction sees only what it observes. `docs/product/accepted/prd-0001-a-faction-sees-only-what-it-observes.md`
[^5]: Decisions register, DEC-281. `docs/DECISIONS.md`
