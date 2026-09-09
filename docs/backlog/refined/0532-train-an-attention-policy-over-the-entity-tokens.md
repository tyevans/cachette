---
id: 0532
title: Train an attention policy over the entity tokens
status: refined
created: 2026-09-08
implements: [ADR-0201 D1, ADR-0201 D3, ADR-0201 D4, ADR-0201 D5, ADR-0201 D6, ADR-0201 D7]
changes: []
creates: [ADR-0201]
serves: [PRD-0056]
blocked-by: []
---

## Why

The project owner asked whether the project should train an attention-based
policy, at what size, and at what population.

The policy the project trains today reads each entity token set with one shared
encoder and reduces it with a mean and a maximum. That reduction keeps the
field of the set and loses its joint structure. A reader cannot tell from a
mean and a maximum whether the strongest rival is also the nearest rival. The
research report names a masked attention over the tokens as the established
answer.[^report]

This item answers the question with a run. It adds one policy kind, trains it
against the present policy on one reward row, and settles the comparison with
the rating tool.[^league]

## The arithmetic

Every figure below is derived, not measured, except the run costs, which come
from the last training run. BLK-007 governs every cost figure.[^costs]

### The law

The cosine between the step one generation takes and the direction it looks for
is near the square root of the pair count divided by the trainable count.[^law]
One generation holds two candidates for each pair.

The present structured policy trains 5,354 weights. At population 64 it holds
32 pairs, and the law gives 0.0773. To hold at least that alignment, a design
needs a population of at least its trainable count divided by 83.66.

### Where the trainable count sits today

| Part | Weights | Formula |
|---|---|---|
| Scalar layer | 1,684 | scalar width times the scalar positions and one bias |
| Ring tower | 246 | one sector kernel, one channel mixing and one ring band matrix |
| Token towers | 352 | one encoder for each set, sized by the channels of the set |
| Trunk | 732 | trunk width times the joined tower width and one bias |
| Readout | 2,340 | action rows times the trunk width and one bias |
| Total | 5,354 | |

**The readout and the scalar layer hold 4,024 of the 5,354.** The shared
weights over the ring stack and the token sets hold 598. An attention head over
the token sets therefore changes a small part of the whole, and the population
it costs is small.

The action row count of 180 is derived: it is the only integer that makes the
parts above sum to the trainable count the register records.[^law] The
implementer must read it from the engine schema and refuse a mismatch.

### The candidate designs

Each row is a change to the structured policy. The scalar layer, the ring tower
and the readout are unchanged unless the row says otherwise. `E` is the token
width, `C` is the channel count of a set, and the sets are the four the layout
declares.

| Row | Design | Added weights | Total | Population for 0.0773 |
|---|---|---|---|---|
| 0 | Control. The present structured policy | none | 5,354 | 64 |
| 1 | One attention pool for each set, beside its mean and maximum. Per set one query vector and one key matrix, `E + E*E` | 80, and the trunk grows by 192 | 5,626 | 68 |
| 2 | One joined set, one self-attention layer, `E` held at 4. Set identity `4*E`, attention `4*E*E`, mean and maximum over the joined set | 80, and the trunk shrinks by 288 | 5,146 | 62 |
| 3 | Row 2 with `E` at 8 | 992 against 352, and the trunk shrinks by 192 | 5,802 | 70 |
| 4 | Attention over the ring cells instead of the ring tower. Cell encoder, attention, and a relative bias factorised into one weight per sector offset and one per ordered ring pair | 672 against 246, and the trunk shrinks by 96 | 5,684 | 68 |
| 5 | Rows 3 and 4 together | as both | 6,132 | 74 |

**Row 2 is smaller than the control and better aligned than it.** The attention
pool replaces four pairs of pooled features with one pooled pair, so the trunk
loses more weights than the attention adds.

### What each row costs to run

The last run played population 128 at 8 seeds, which is 1,024 episodes for each
strategy for each generation. Four strategies ran at once, each with two shards
of eight workers, and a generation took about 417 seconds on a 64-core Graviton
instance at $0.7715 an hour. Sixteen workers for each strategy played 64
episodes each, so one episode costs about 6.5 worker-seconds.

A generation of one strategy therefore takes about `6.5 * population * seeds /
16` seconds of wall time while four strategies share the machine. The machine
costs about $0.000214 a second.

| Row | Population | Seconds a generation | Machine cost of 100 generations |
|---|---|---|---|
| 0 | 64 | 208 | $4.46 |
| 2 | 62 | 202 | $4.33 |
| 1 | 68 | 221 | $4.74 |
| 4 | 68 | 221 | $4.74 |
| 3 | 70 | 228 | $4.88 |
| 5 | 74 | 241 | $5.16 |

**The size and population trade is not the binding cost.** Every candidate
design sits within a sixth of the control on both population and money. The
binding cost is the wall time of a run and the quality of the reward.

One knob is worth naming. Cutting the scalar width by one saves 433 weights,
which is more than row 2 adds. The project can pay for an attention head out of
the scalar layer and hold the population fixed.

### Trading seeds for population

**This is the second lever, and it is worth more attention than the
architecture.** A generation plays one seed set, and every candidate of the
generation plays it.[^seeds] The pair is antithetic, so the difficulty of the
world is common to both members of a pair. What remains is the interaction
between a candidate and a world, and that falls as the seed count rises.

The register measures one point: scoring on a single world halves the
cosine.[^law] The rows below model the penalty as the square root of one plus
three over the seed count, which reproduces that point and gives no penalty in
the limit. **The middle rows are interpolation and are unverified.**

Put the model into the law and the seed count leaves the expression as one
term. At a fixed episode budget `E` and a trainable count `N`:

    effective alignment = sqrt( E / (2 * N * (s + 3)) )

**The whole trade is the product of `N` and `s + 3`, and the smaller product
wins.** Halving the trainable count divides that product by two. Halving the
seed count from 4 to 2 divides it by seven fifths. The parameter knob is
therefore worth about 1.4 times the seed knob for each halving, and it carries
none of the risk the seed knob carries.

At the last run's episode count, and at the trainable count of the control:

| Seeds | Population | Pairs | Alignment by the law | Modelled penalty | Effective |
|---|---|---|---|---|---|
| 8 | 128 | 64 | 0.1093 | 1.173 | 0.0932 |
| 4 | 256 | 128 | 0.1547 | 1.323 | 0.1169 |
| 2 | 512 | 256 | 0.2187 | 1.581 | 0.1383 |
| 1 | 1,024 | 512 | 0.3093 | 2.000 | 0.1547 |

The whole range from 8 seeds to 1 buys 1.66 times the alignment. The first step
from 8 to 4 buys 1.25 of that. The step from 4 to 2 buys a further 1.19, and
the step from 2 to 1 buys 1.12. **The trade is real and it is shallow.**

### The large policy, and what one night buys

A policy of about 100,000 weights needs population 1,200 to hold 0.0773. At 8
seeds that is 9,600 episodes a generation, about 65 minutes, and about 43 hours
for 40 generations. That is not affordable on one machine.

The cost model above reproduces both figures, and it reproduces the alternative
as well: population 1,024 at 2 seeds is 2,048 episodes, about 832 seconds a
generation, and about 9.2 hours for 40 generations. That is one night.

**Set that option against the one it hides.** One night holds about 2,048
episodes a generation. Two designs fit it, and the law gives them the same raw
alignment, because the product of the trainable count and the seed count is the
same in both.

| Option | Weights | Seeds | Population | Alignment by the law | Modelled penalty | Effective |
|---|---|---|---|---|---|---|
| A | 100,000 | 2 | 1,024 | 0.0716 | 1.581 | 0.0453 |
| B | 50,000 | 4 | 512 | 0.0716 | 1.323 | 0.0541 |

**Option B is better aimed by a fifth, for the same money and the same night.**
Halve the policy rather than halve the seed set, whenever both are open. Option
B still trains nine times the weights of the control.

One honest figure goes with this. The control, at its own budget, holds an
effective alignment near 0.0659. **Both large options are worse aimed than the
policy the project trains today**, by a third and by a fifth. That is the price
of the twenty-fold and ten-fold size, and it is the strongest argument for
building the smallest attention design first.

### The floor on seeds

**The floor is 4. Two seeds is not safe today, and it becomes safe against a
measurement rather than against an argument.**

The failure mode is not the variance of one candidate's fitness. Every
candidate of a generation plays the same seed set, so an easy world is easy for
all of them and the antithetic pair cancels most of it. The failure mode is
that the whole generation is then ranked on two worlds. **Rank shaping gives
the top candidates their full weight whatever the size of their true
advantage**, so a candidate that exploits one map feature moves the centre as
hard as a candidate that plays better everywhere. The seed set moves every
generation, so the bias is a fresh draw each time and 40 generations average
some of it away. Nothing measures how much.

The second risk is cheaper to state. The trainer refuses a generation whose
candidates all score the same. With two seeds one degenerate world can waste a
whole generation.

Two measurements would settle the floor, and both are cheap.

**The rank agreement.** Take the candidates of a finished generation. Replay
them on a large seed set. Compare the rank order that gives against the rank
order the small set gave. The rank correlation is the quantity the trade rests
on. **This needs no new training run**, only a replay of scored candidates.

**The seed axis of the alignment sweep.** The sweep that measured the law
already measures the cosine against a known direction, and it already measured
the single-world cost.[^law] Run it again with the seed count as the swept
axis. That replaces the model above with measured points and ends the
interpolation.

Run the seed change as its own comparison, whichever floor the measurement
gives. It is a second variable, and a run that moves the architecture and the
seed count together cannot say which one moved the rating.

### Where the arithmetic runs

**This item proposes no move to a graphics device, and a reader who wants one
should read the research before reopening it.**[^device] The simulation holds
nearly the whole training clock, so moving the policy alone buys almost
nothing, and the research found the processor ahead at every policy size it
tried, including networks far larger than any row of this item. Moving the
simulation is a rewrite and not a port. It would put a second implementation of
the engine beside the first, the two would have to agree byte for byte, and
only the golden state hash would say when they did not. No published batch
simulator states a bit-exact guarantee across thread counts, block counts or
devices, and the determinism claim outranks speed.[^det]

The float ban removes the obstacle a reader expects here, and that is the part
worth knowing. An integer atomic add gives the same total in any order, and the
counter-based keyed draw suits such a device well.[^device] The obstacle is the
second implementation and the absent guarantee, not the arithmetic.

## Impact review

**Governed by.** ADR-0201 D1 fixes what attends to what, and this item builds
row 2 and nothing else. ADR-0201 D2 forbids row 4 and row 5, so the item must
not build them. ADR-0201 D3 requires the mask to come from the validity
channel. ADR-0201 D4 requires the population figure before the run. ADR-0201 D5
makes the parameter count the first knob and forbids a lower seed count without
a measurement, so this item holds the seed count of the last run and proposes
the measurement instead. ADR-0201 D6 keeps the argmax and forbids a draw.
ADR-0201 D7 makes the rating the verdict.

ADR-0195 D4 requires a reader to treat a token set as a set, and an attention
over a set with a learned set identity meets it.[^set] ADR-0195 D8 requires an
estimate to carry a confidence position, and the rival token already holds
one.[^set] ADR-0154 D1 makes the schema the only declaration of the layout, so
the new kind states no width of its own.[^schema] ADR-0200 D3 rebuilds a
readout by verb identity, and this design does not change the readout, so a
stored file still loads.[^stored] ADR-0002 D4 admits the floating point
weights, because the value that crosses into the engine is one action
integer.[^float] ADR-0001 D1 requires one answer at any thread count, and the
policy holds no random state.[^det]

**Changes.** None. The structured kind stays, because it is the control of the
comparison and several stored policies read it.

**Creates.** ADR-0201, which is written and is in `draft/`.

**Blockers.** BLK-007 governs every cost figure this item states, and the run
costs above must be re-measured on the target platform rather than carried.
BLK-158 governs any strength value the item reads, and the item must not invent
one.[^strength]

**The gate this item waits on.** Most power ratios of a rival token read zero,
because the engine holds an estimate of the settlement ratio alone.[^token] The
rival set is the set the report's argument is about. An attention over it can
attend to the settlement ratio, the two relations, the war flag, the border
share, the settlement distance and the confidence, and to nothing else.

**Do not start this item until the rival power ratios carry a value.** ADR-0195
D8 already admits an estimate with a confidence position, so writing them is
engineering and not an open question. An audit of the observation found the wider
case: about 1,401 positions read zero forever while declaring a real value
form. Three ring channels are zero in every cell, and five more are written in
the near rings alone. **This worktree cannot read that audit**, so treat those
three figures as reported and check them before quoting them. The two ring
facts are read from the engine source and are verified.[^ring]

The ring channels are not a gate for this item, because row 2 does not touch
the ring stack. They are a gate for row 4, which is a second reason ADR-0201 D2
defers it.

**Precedent.** FND-668 holds the law and the measurement of scoring
noise.[^law] FND-679 records that a symmetric outcome weight made a draw the
rational play, and the loss weight is now a tenth of the win weight, so the
defect is closed.[^draw] FND-650 records that the conquest pair went flat after
five generations while the ground pair was still rising at sixty-seven, so
**this comparison runs on the ground row and not the conquest row**: a policy
that reads better cannot show it under a nearly ternary reward.

Reports 35, 40 and 41 recommend a policy kind this project has deleted. Do not
read an operating point from them. Their reasoning about a fixed random
projection is superseded by the law.[^law]

## Done when

- The learner package holds one new policy kind. It joins the token sets into
  one set, gives each token a learned set identity, runs one masked
  self-attention layer over the joined set, and pools the result with a mean
  and a maximum.
- The kind states no width, no channel count and no token count of its own. It
  reads every one from the schema the engine publishes.
- A test asserts the trainable count of the new kind against the layout the
  engine publishes, in the way the structured kind is already asserted, and the
  count is below the control's.
- A test permutes the tokens of one set and asserts that the chosen action does
  not change.
- A test builds an observation with an absent token, and asserts that the
  attention gives it no weight. **The test must be shown able to fail**: remove
  the mask and watch the assertion break.
- A fixture holds two rivals where the nearest is the weaker and the strongest
  is the further. A hand-set policy of the new kind separates the two orderings.
  **Put the pooled reader back and watch the separation go.** A fixture built
  from the demonstration world is not this fixture.[^fixture]
- The strategy table holds one row that pairs the new kind with the ground
  reward and the world the other rows play.
- The run reports the population the law requires for the trainable count it
  built, before the first generation.
- The rating tool plays the new policy, the control and the built-in
  controller, over rotated seats, and reports the pairwise difference with its
  error bar.
- The register holds the outcome, whichever way it went.

## What would decide it

**Recommend the kind** when the new policy rates above the control by more than
twice the standard error of the pairwise difference, and the attention weights
are not uniform on the held-out seeds. A uniform weight means the head learned
the mean pool, and the gain then came from the narrower trunk and not from the
attention.

**Drop the kind** when the difference does not clear twice its standard error,
or when the new policy rates below the control. The project then knows that
reading power is not the binding constraint at this reward, and the next move
is the reward or the observation.

**A third outcome needs naming.** Seven of the eight stored policies rate below
the built-in controller. A win over the control that still sits below the
controller keeps the kind and closes nothing. Say so in the outcome rather than
reading it as a success.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^report]: Report 42, what a policy should be able to see, sections 6.4, 6.5 and 11. `docs/research/reports/42-what-a-policy-should-be-able-to-see.md`
[^league]: The policy rating tool. `scripts/policy_league.py`
[^law]: Findings register, FND-668. `docs/FINDINGS.md`
[^costs]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^seeds]: The trainer, the seeds of one generation. `python/cachette/learn/train.py`
[^set]: ADR-0195, the observation of a faction is a fixed-width scale-free table, decisions D4 and D8. `docs/adrs/draft/adr-0195-the-observation-of-a-faction-is-a-fixed-width-scale-free-table.md`
[^schema]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decision D1. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
[^stored]: ADR-0200, a stored policy names each action row by verb and coordinates, decision D3. `docs/adrs/draft/adr-0200-a-stored-policy-names-each-action-row-by-verb-and-coordinates.md`
[^float]: ADR-0002, state holds no floating point number, decision D4. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^det]: ADR-0001, one binary gives one answer at any thread count, decision D1. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^strength]: Blockers register, BLK-158. `docs/BLOCKERS.md`
[^token]: The entity token block, the channel list of a rival token. `crates/cachette-core/src/obs_token.rs`
[^ring]: The ring stack block, the channels this block cannot fill. `crates/cachette-core/src/obs_ring_stack.rs`
[^draw]: Findings register, FND-679. `docs/FINDINGS.md`
[^device]: Report 38, where the training time goes, sections 3, 6 and 7. `docs/research/reports/38-where-the-training-time-goes.md`
[^fixture]: Testing Rules, a fixture supplies the input. `.agents/rules/testing.md`
