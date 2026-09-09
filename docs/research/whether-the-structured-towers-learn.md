# Whether the structured towers learn

The structured policy of this project reads the observation through three
towers, joins them in a trunk, and scores each row of the action table in a
readout. The readout starts at zero and every tower starts at a random draw.
The reason the project prefers this shape over a flat linear policy is that
fewer trainable weights give a better aligned step, and the alignment of one
generation is near the square root of the pair count divided by the trainable
count.[^1]

That reason fails if the towers do not move. The policy is then a readout over
a fixed random projection of the observation. Its effective trainable count is
the readout's.

This note measures four things over real training runs: whether the towers
move, where the step goes, whether the zero readout start delays the towers,
and whether tower movement changes what a run reaches. It closes two rows that
an earlier audit of the training path left open.[^2]

The short answer: the towers move, the step is spread over the weights exactly
in proportion to their count, the zero start delays nothing, and freezing the
towers costs nothing that twelve held-out worlds can measure.

## 1. The split of the trainable weights, and a correction

The policy reports its own weight count, and the run measured it on the current
tree.

| block | weights | share | root mean square per weight at the start |
|---|---|---|---|
| scalar tower | 1,684 | 31.5 % | 0.0479 |
| ring tower | 246 | 4.6 % | 0.3495 |
| token towers | 352 | 6.6 % | 0.1998 |
| trunk | 732 | 13.7 % | 0.1361 |
| readout | 2,340 | 43.7 % | 0.0000 |
| total | 5,354 | 100 % | |

**The readout holds 2,340 weights and not 4,024.** A refined backlog item
states that the readout and the scalar layer together hold 4,024 of the
5,354, and that statement is correct.[^3] A reading that gives the 4,024 to
the readout alone is not. The distinction matters, because the scalar tower is
a dense trainable layer over the unstructured positions of the observation and
the readout is not a tower at all. The two geometric towers, which are the
part of the design that shares weight over the shape of the world, hold 598
weights between them. That is 11.2 percent of the policy.

Read the right-hand column with care. The initial draw of the ring tower is
more than seven times as large per weight as the initial draw of the scalar
tower. Each layer is scaled by its own fan-in.

## 2. The world this measured, and why the conclusion transfers

The configured world is 128 columns square with a tick limit of 6,000 and a
decision interval of 10 ticks.[^4] One episode of it costs about half a minute
on sixteen cores, and a generation is the population times the worlds each
candidate plays. Five runs of ten generations each were not affordable at that
size, so the world was reduced.

| field | configured | measured here |
|---|---|---|
| extent, both sides | 128 | 64 |
| tick limit | 6,000 | 2,500 |
| decision interval | 10 | 10 |
| horizon, decisions per episode | 600 | 250 |
| factions | 3 | 3 |
| population | 16 by default | 12 |
| worlds each candidate plays | 4 by default | 3 |
| generations | 12 by default | 10 |
| sigma | 1.5 | 1.5 |
| learning rate | 0.3 | 0.3 |

The perturbation radius is the value the tree configures, which is 1.5. A
finding recommends 0.5 and the tree has not yet moved.[^5] The learning rate is
the configured 0.3.

**The shortened world still spreads the candidate scores.** An earlier
measurement found that a tick limit of 1,000 forces the same outcome on every
world and collapses the spread, so the check matters. Over the fifty
generations of the five runs below, the spread between the highest and the
lowest candidate of a generation ran from 84 to 1,757 with a median of 804. It
fell under 200 points in five generations and reached 600 or more in
forty-one. Every one of the fifty generations carried information, so the
search never met a tie. The reduced world therefore ranks candidates, and that
is the only thing a generation of an evolution strategy needs of it.

The five runs cost 3 hours and 34 minutes of run time between them, on a
machine of sixteen cores. Two of them ran at the same time, so the elapsed
time was shorter than the sum.

**Why the conclusion transfers.** Sections 3 and 4 rest on the arithmetic of
the search and not on the world. The search draws each perturbation from an
isotropic normal in the whole flat vector and normalises it to unit length, so
the share of the step that reaches a block is a property of the block's weight
count. The measurement confirms the arithmetic to three decimal places, and no
change of extent or tick limit can move it. Section 5 rests on the arithmetic
of the policy, which is also independent of the world. **Section 6 is the one
that depends on the world**, and its conclusion is stated as a null result
against a stated error, not as a measured gain.

## 3. Question 1. The towers move, and the ring tower moves least

The table gives the length of the step that each block took in one generation,
divided by the length that block held at the start of that generation. The run
is the one the tree configures.

| generation | scalar tower | ring tower | token towers | trunk | readout |
|---|---|---|---|---|---|
| 0 | 0.693 | 0.094 | 0.162 | 0.231 | from zero |
| 1 | 0.577 | 0.096 | 0.170 | 0.240 | 1.058 |
| 2 | 0.526 | 0.097 | 0.162 | 0.245 | 0.757 |
| 3 | 0.482 | 0.098 | 0.181 | 0.253 | 0.628 |
| 4 | 0.445 | 0.105 | 0.184 | 0.249 | 0.570 |
| 5 | 0.430 | 0.112 | 0.176 | 0.264 | 0.508 |
| 6 | 0.406 | 0.114 | 0.194 | 0.260 | 0.475 |
| 7 | 0.387 | 0.114 | 0.199 | 0.268 | 0.450 |
| 8 | 0.378 | 0.124 | 0.205 | 0.270 | 0.424 |
| 9 | 0.361 | 0.125 | 0.213 | 0.276 | 0.412 |

The cumulative figures follow. The displacement column is the length of the
whole path the block travelled, taken as the square root of the sum of the
squared steps. Two steps in a space of this dimension are near orthogonal, so
the sum approximates the displacement rather than bounding it.

| block | length at the start | length at the end | ratio | displacement | displacement over the start |
|---|---|---|---|---|---|
| scalar tower | 1.965 | 5.504 | 2.80 | 5.080 | 2.58 |
| ring tower | 5.482 | 5.753 | 1.05 | 1.941 | 0.35 |
| token towers | 3.748 | 4.322 | 1.15 | 2.318 | 0.62 |
| trunk | 3.682 | 4.656 | 1.26 | 3.316 | 0.90 |
| readout | 0.000 | 6.013 | from zero | 6.034 | from zero |

**Read the ratio and the displacement together.** The norm of the ring tower
grows by five percent over ten generations, and the tower nonetheless travels
35 percent of its own initial length. The displacement is near orthogonal to
the initial vector, so it adds in quadrature and barely moves the norm. A
report that gave the ratio alone would call the ring tower frozen, and it is
not.

**The towers move. The geometric towers move least, and the reason is the
initial scale.** The next section gives it.

## 4. Question 2. The readout does not dominate the step. Every weight moves the same distance

| generation | scalar tower | ring tower | token towers | trunk | readout | sum of the squares |
|---|---|---|---|---|---|---|
| 0 | 0.579 | 0.220 | 0.258 | 0.362 | 0.647 | 1.000 |
| 1 | 0.564 | 0.216 | 0.261 | 0.366 | 0.658 | 1.000 |
| 2 | 0.575 | 0.211 | 0.241 | 0.366 | 0.657 | 1.000 |
| 3 | 0.562 | 0.207 | 0.262 | 0.371 | 0.660 | 1.000 |
| 4 | 0.556 | 0.214 | 0.257 | 0.356 | 0.672 | 1.000 |
| 5 | 0.564 | 0.222 | 0.238 | 0.369 | 0.663 | 1.000 |
| 6 | 0.558 | 0.214 | 0.257 | 0.362 | 0.668 | 1.000 |
| 7 | 0.552 | 0.207 | 0.258 | 0.367 | 0.672 | 1.000 |
| 8 | 0.557 | 0.218 | 0.257 | 0.365 | 0.665 | 1.000 |
| 9 | 0.548 | 0.212 | 0.261 | 0.368 | 0.672 | 1.000 |

Each cell is the length of that block's step divided by the length of the whole
step. The readout takes about two thirds of the length. That is not dominance.
The blocks are orthogonal, so the shares combine in quadrature, and the
squared shares are what add to one.

| block | mean share of the squared step length | share of the weights |
|---|---|---|
| scalar tower | 0.3154 | 0.3145 |
| ring tower | 0.0458 | 0.0459 |
| token towers | 0.0651 | 0.0657 |
| trunk | 0.1334 | 0.1367 |
| readout | 0.4403 | 0.4371 |

**The two columns agree to within three parts in a thousand at every row.** The
step of a generation is spread over the weights exactly in proportion to their
count, and the scores of the population change nothing about that split. The
root mean square displacement per weight was 0.0385 to 0.0391 in every block of
every generation, so it is one number within five percent.

The mechanism is in the search, and it is simple. A perturbation is an
isotropic normal draw over the whole flat vector, scaled to unit length. The
update is a rank-weighted sum of those draws. The search then takes the unit
direction of that sum and scales it by the learning rate. **Nothing in
that path asks which block a coordinate belongs to.** A block therefore
receives a step length proportional to the square root of its weight count,
whatever the episodes said.

This has two consequences that the design does not currently account for.

**A block whose initial draw is large per weight barely changes, and a block
whose initial draw is small per weight changes completely.** The step per
weight is one number. The initial scale per weight runs from 0.048 in the
scalar tower to 0.350 in the ring tower, a factor of 7.3. So the scalar tower
travels 2.58 of its own length in ten generations and the ring tower travels
0.35 of its own. The design chose the initial scales from fan-in, which is the
right rule for a gradient method, and this search is not one.

**Movement is not evidence of learning.** Every figure in section 3 would look
the same if the tower coordinates of the perturbation had no effect on any
score at all. Section 5 measures whether they do.

## 5. Question 3. The zero start delays nothing, because the delay it predicts does not exist

The policy's own documentation states that a zero readout scores every row the
same, so the first generation moves the readout alone and the towers begin to
move once the readout is not zero. **The first half of that claim is true of
the centre. The second half is false, in two separate ways.**

The first way is section 4. The step moves every block from generation zero
onward, in proportion to the weights, and the readout is not consulted.

The second way needs the candidates rather than the centre. A candidate carries
a perturbation in every block at once. Over 374 real decisions from four
worlds, with the centre at its zero readout, each of the six perturbation
directions of the first generation was applied three ways: whole, readout part
only, and tower part only.

| pair | whole perturbation | readout part alone | tower part alone | whole against readout part |
|---|---|---|---|---|
| 0 | 0.310 | 0.791 | 0.000 | 0.660 |
| 1 | 0.906 | 0.906 | 0.000 | 0.481 |
| 2 | 0.882 | 1.000 | 0.000 | 0.380 |
| 3 | 1.000 | 1.000 | 0.000 | 0.294 |
| 4 | 0.821 | 0.874 | 0.000 | 0.511 |
| 5 | 1.000 | 0.906 | 0.000 | 0.647 |

Each cell is the share of the 374 decisions whose chosen action differs from
the named comparison. The tower column is exactly zero at every pair, as the
arithmetic requires: with the readout at zero, no tower weight can move any
score. The last column is the one that answers the question. **Once the readout
of a candidate is perturbed, the tower part of the same perturbation moves
between 29 and 66 percent of that candidate's choices.** So the ranking of the
very first generation already sees what the towers do, and there is no
generation in which the towers are invisible to the search.

A paired run confirms it. One run starts the readout at zero and the other
starts it at a small random draw of length 0.658, against a readout that
reaches 6.0 after ten generations. Everything else is identical, including the
search seed and the seed pool, so the two runs draw the same perturbations.

| block | displacement over the start, zero readout start | displacement over the start, small random readout start |
|---|---|---|
| scalar tower | 2.58 | 2.60 |
| ring tower | 0.35 | 0.35 |
| token towers | 0.62 | 0.63 |
| trunk | 0.90 | 0.92 |

**The tower movement of the two runs is the same to two decimal places.** The
zero readout start does not delay the towers by any amount this measurement can
detect. Section 6 gives the scores the two runs reached.

## 6. Question 4. Freezing the towers costs nothing that twelve worlds can measure

Four runs shared one world, one seed pool of 24, one held-out set of 12 seeds,
one feature normalizer and one search seed. They differ only in what the search
may move.

- **full.** The run the tree configures. Every block trainable, readout at zero.
- **warm.** The same, with the small random readout start of section 5.
- **readout only.** Every perturbation confined to the readout block. The
  towers and the trunk keep the draw they started from.
- **geometry frozen.** Every perturbation confined to the readout, the scalar
  tower and the trunk. The ring tower and the token towers keep their draw.

The held-out figures follow. Each cell is the mean shaped return of the centre
over the twelve held-out worlds, and the win share on the same worlds. The
built-in controller in the same seat on the same worlds scored 913.9 and won
0.33.

| generation | full | warm | readout only | geometry frozen |
|---|---|---|---|---|
| 1 | 591.6 / 0.17 | 133.5 / 0.00 | 156.5 / 0.00 | 501.6 / 0.17 |
| 3 | 100.1 / 0.00 | 162.3 / 0.00 | 258.4 / 0.08 | 319.8 / 0.08 |
| 5 | 246.0 / 0.08 | 278.5 / 0.08 | 272.5 / 0.08 | 679.1 / 0.25 |
| 7 | 97.8 / 0.00 | 465.6 / 0.17 | 1162.0 / 0.50 | 679.1 / 0.25 |
| 9 | 155.4 / 0.00 | 459.4 / 0.17 | 521.5 / 0.17 | 679.1 / 0.25 |

**Read the win share with its error.** Twelve worlds give a proportion near
0.17 a standard error of 0.108, and a proportion near 0.5 an error of 0.144. No
column separates from any other column at that error, and no column separates
from the controller. **The table states a null result and it must not be read
as a ranking.**

The null result is the finding. The run that trains every weight is not ahead
of the run that trains 43.7 percent of them at any pass, and it is behind at
three of the five. The run that freezes only the two geometric towers reached
the highest figure that any run held at more than one pass. **If the towers
carried something, ten generations of this world did not find it.**

Two facts weaken the readout-only column, and the report states both.

**Its readout moves further per generation than the full run's readout does.**
The search takes a unit direction, so a step confined to one block moves that
block the whole learning rate rather than the 0.647 of it that the block takes
in the full run. The readout-only run therefore travelled 2.869 per generation
against 1.893, and it finished at a readout length of 9.198 against 6.013. A
fifth run removes most of that difference. It scales the step by the square
root of the readout's share of the weights, so the readout receives the same
share of the step that it receives in the full run. Its figures are in section
7.

**Ten generations is short and three worlds a candidate is thin.** A finding
measured that the configured radius needs six worlds for a candidate to lift
the signal over the noise of one episode, and this ran three.[^5] So the search
of every one of the four runs spent much of each step on scoring noise, and a
difference that appears later than generation ten is out of reach here.

## 7. The matched readout-only run

The fifth run confines the perturbation to the readout. It scales the step by
the readout's share of the step length, so the readout receives the travel the
full run gives it. It is the controlled form of the readout-only column.

| generation | full | matched readout only |
|---|---|---|
| 1 | 591.6 / 0.17 | 236.4 / 0.08 |
| 3 | 100.1 / 0.00 | 282.4 / 0.00 |
| 5 | 246.0 / 0.08 | 139.5 / 0.00 |
| 7 | 97.8 / 0.00 | 438.2 / 0.08 |
| 9 | 155.4 / 0.00 | 383.2 / 0.08 |
| mean of the five passes | 238.2 / 0.050 | 295.9 / 0.050 |

The step scale is the square root of 0.4371, which is 0.6611. The readout of
the matched run moved 1.699 per generation against 1.893 in the full run, and
it finished at a length of 5.336 against 6.013. The remaining difference is in
the other direction from the first frozen run: the matched run holds a shorter
centre, because its towers never grow, and the step is a fraction of the length
of the centre. **So the matched run gives its readout slightly less travel than
the full run gives it, and it still matches the full run.**

The win shares are equal over the five passes. The mean shaped return favours
the matched run by 58 points, which is well inside the error of twelve worlds.
**Training 2,340 weights reached what training 5,354 weights reached, at the
same displacement of the shared block.** The extra 3,014 weights bought nothing
this measurement can find, and the alignment law says they cost a factor of
1.51 on the cosine of every step.

## 8. What this could not measure

- **Whether the towers help at the configured world.** Every figure comes from
  a world of a quarter the area and a tick limit of 2,500. Sections 3, 4 and 5
  do not depend on the world. Section 6 does, and it is unverified at extent
  128.
- **Whether the towers help over a long run.** Ten generations at three worlds
  a candidate is the budget this measurement had. A published run of this
  project takes tens of generations.
- **Whether the towers help at the configured population.** The population here
  is 12 against a configured 16 and a paid run of 64. Alignment rises with the
  population, so a larger run may find a tower direction this one could not.
- **Which tower carries what.** The geometry-frozen run freezes the ring tower
  and the token towers together. Nothing here separates the two.
- **Whether a different initial scale would change the answer.** Section 4 says
  the initial scale per weight decides how far a block travels relative to
  itself, and no run varied it. That is the cheapest follow-up this note names
  and it was not run.
- **The reward the measurement used.** One weighting drove every run: the one
  that rewards ground and settlements at a level. A weighting that rewards
  almost nothing but the win was not measured, and a finding records that the
  two behave differently over the same policy.[^6]

## 9. Recommendation

**Stop calling 5,354 the trainable count of a search that behaves as though it
were smaller, and fix the initial scale rather than the readout start.**

Three separate actions follow, in the order of their evidence.

1. **Correct the two claims that this measurement falsified.** The policy's
   documentation says the towers start to move once the readout is not zero.
   They start to move in generation zero, in proportion to their weight count,
   and the ranking of generation zero already sees what they do. A refined
   backlog item's 4,024 belongs to the readout and the scalar layer together,
   not to the readout.
2. **Set the initial scale of each layer against the step the search gives it,
   not against the layer's fan-in.** This is the finding with the strongest
   evidence and the lowest cost. The search moves every weight the same
   distance, so a layer drawn at 0.35 per weight is seven times harder for the
   search to revise than a layer drawn at 0.048, and nothing chose that ratio.
   The two geometric towers hold the largest initial scale and 11.2 percent of
   the weights, which is the worst pair of the five blocks.
3. **Do not freeze a tower on this evidence, and do not defend the shape on the
   trainable count either.** The module states that no layer of this policy is
   frozen, because a layer the trainer never moves states a rule the run cannot
   revise. That is a design position and this measurement does not overturn it:
   twelve held-out worlds cannot separate the runs, so the honest reading is
   that freezing costs nothing measurable, not that it gains something. But the
   alignment argument for the structured shape must be restated. The square
   root law over 5,354 weights gives a cosine of 0.0335 at six pairs, and over
   the readout alone it gives 0.0506. **A run that trains everything pays for
   5,354 weights of dilution and this measurement found no return on 3,014 of
   them.** Either find the return, at a larger population or a longer run, or
   shrink the policy.

The cheapest next measurement is the one this note skipped: hold the shape and
the search fixed, vary only the initial scale of the ring tower and the token
towers, and see whether the ten-generation held-out figure moves. It costs what
one run of section 6 cost.

## References

[^1]: Findings register, FND-668. `docs/FINDINGS.md`
[^2]: Research, what is wrong with training and evaluation, the rows the report could not close. `docs/research/what-is-wrong-with-training-and-evaluation.md`
[^3]: Backlog, train an attention policy over the entity tokens. `docs/backlog/refined/0532-train-an-attention-policy-over-the-entity-tokens.md`
[^4]: The learner launcher, the world constants. `python/cachette/learn/__main__.py`
[^5]: Findings register, FND-711. `docs/FINDINGS.md`
[^6]: Findings register, FND-650. `docs/FINDINGS.md`
