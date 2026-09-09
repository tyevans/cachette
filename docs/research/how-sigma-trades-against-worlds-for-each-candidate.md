# How sigma trades against the worlds each candidate plays

This note measures two knobs of the evolution strategy against each other. The
first knob is sigma. It is the fraction of the centre that one candidate moves.
The second knob is the number of worlds a candidate plays before the search
ranks it. The project set sigma from one measurement of the first knob, and
nothing had measured what the second knob buys back.

The measurement gives one recommendation, and it states the recommendation as a
pair, because the two knobs do not decide separately.

## 1. Context

Cachette trains a policy with an evolution strategy. The search draws a
perturbation of unit length, scales it, and adds it to the centre in both
directions. It plays each candidate on a set of worlds. It ranks the candidates
by score, and it steps the centre along the ranked sum.[^1]

Sigma is that scale. It is a fraction of the length of the centre and never an
absolute length. One kind of policy holds its centre at unit length, and the
structured kind does not, so sigma for the structured kind is a fraction of the
length the centre reached.[^1]

The trainer holds sigma at 1.5.[^2] A random direction in several thousand
dimensions is near orthogonal to the centre, so a perturbation of 1.5 centre
lengths puts a candidate 56 degrees from the centre. The two halves of one
antithetic pair then sit 113 degrees apart. An audit of the training path names
that arc and says a finite difference over it does not estimate a local
direction.[^3]

The project recorded the value of 1.5, and the reason for it is sound. A
perturbation that is too small gives every candidate the same choices, the
scores tie, and the ranking carries no information. A commit records the
measurement that raised sigma: on 480 real decisions, a perturbation of 0.25
changed 0.008 of the choices of a trained policy, and one of 1.5 changed about
a third of them.[^4]

That measurement used a linear policy of 5133 weights held at unit length. The
kind the trainer runs today is the structured kind, and that kind keeps the
length its centre reached. This note therefore measures the locality again for
the kind that runs.[^4]

**That measurement fixes the spread and gives up the locality, and only the
spread was ever measured.** A ranking needs the spread between the candidates
to exceed the noise on each candidate's score. More worlds for each candidate
lower that noise. A small sigma may therefore be affordable at a higher world
count. This note measures that trade.

## 2. What was measured

The measurement holds one centre fixed and varies sigma over 0.1, 0.25, 0.5 and
1.5. The centre is the published structured policy of the aggressive play style
at generation 7.[^5] The score is the aggressive play style of the style table,
which is the style that centre was trained under.[^6] The world is the world
the strategy table states: 128 columns by 128 rows, three factions, a decision
every ten ticks, and a tick limit of 6000.[^7]

Each sigma draws 8 candidates as 4 antithetic pairs. **The four unit directions
are the same at every sigma**, so the comparison between two sigmas changes the
length of a perturbation and nothing else. Every candidate plays the same 8
worlds, which are the first 8 viable seeds from zero. The candidates come from
the search itself, so the perturbation is the perturbation a generation
builds.[^1]

The policy reads the feature normalizer that the current tree derives for this
world.[^8] The stored file states none, because it predates the normalizer, so
the centre is a trained centre read through a transform it was not trained
through. That is what a resumed run does today with such a file.[^9] The
measurement is therefore of the current tree and not of the tree the paid run
used.

The centre scored a mean of 100.63 over the 8 worlds. It won 6 of them.

### Why the world was not shortened

The plan for this measurement was to lower the tick limit and keep the design.
A probe rejected that. Every one of the 8 episodes of the centre ended by
domination between tick 1271 and tick 3311, so the tick limit of 6000 never
binds. A limit of 1000 truncates instead, and the engine then names a winner
from held ground.

A probe at that limit played one antithetic pair at sigma 1.5 over 4 worlds.
One half scored between minus 80.80 and minus 79.25. The other scored between
minus 99.51 and minus 99.10. The style pays minus 100 for a loss, so the
outcome term is the same on every world. The spread of one candidate across the
worlds is under 1.6 points, against 80 points at the full limit.

**A shorter episode removes the variation in the outcome, and the outcome term
is most of the score.**

The reduction went into the world count rather than the episode length. The
design uses 8 worlds for each candidate and splits them into two disjoint sets
of 4 for the agreement test. Four worlds is what the trainer configures for one
generation, so the split answers for a real generation.[^2]

## 3. The locality of each sigma

The table gives the share of decisions where a candidate takes an action row
that the centre did not take. The sample is 506 real decisions of the centre
over two of the eight worlds. The mean is over the 8 candidates, and the range
is over the same 8.

| sigma | mean share changed | lowest candidate | highest candidate |
|---|---|---|---|
| 0.1 | 0.0035 | 0.0000 | 0.0079 |
| 0.25 | 0.0072 | 0.0000 | 0.0336 |
| 0.5 | 0.1117 | 0.0040 | 0.3992 |
| 1.5 | 0.4116 | 0.0079 | 0.9980 |

**This reproduces the recorded figure.** The commit that raised sigma reports
0.008 changed at 0.25 and about a third changed at 1.5. This measurement gives
0.0072 at 0.25 and 0.4116 at 1.5. The project's own calibration tool, run on
the same file over six held-out worlds and eight fresh directions, gives 0.000
at 0.25 and 0.367 at 1.5.[^10] All three instruments say the same thing. A
perturbation of 0.25 changes under 1 percent of the choices, and a perturbation
of 1.5 changes between a third and four tenths of them.

Two things beside the reproduction are worth stating. The candidates of one
generation differ widely in locality: at sigma 1.5 one candidate changes 0.008
of the choices and another changes 0.998. The centre is also close to a
constant policy. The calibration tool reports that it takes one action row in
461 of 480 decisions, with about 14 rows legal at each one.[^10] A perturbation
changes nothing until it unseats that row.

## 4. The signal and the noise

The signal is the spread of the true means of the candidates. The estimate
splits the scores of one sigma into a candidate part, a world part and a
residual part, and corrects the candidate part for the residual. The table
gives the estimated standard deviation of the candidate part.

The noise column that governs a ranking is the residual one. **Every candidate
plays the same worlds, so the world part shifts every candidate by the same
amount and cannot change an ordering.** The table gives both the raw
within-candidate standard deviation, which the world part inflates, and the
residual standard deviation, which is what a ranking has to fight.

| sigma | mean of the population | spread of the means | signal, standard deviation |
|---|---|---|---|
| 0.1 | 98.19 | 47.89 | 6.55 |
| 0.25 | 103.53 | 71.76 | 19.57 |
| 0.5 | 77.03 | 102.50 | 37.30 |
| 1.5 | 51.03 | 118.01 | 31.95 |

| sigma | within, standard deviation | residual, standard deviation | noise at 8 worlds, shared | noise at 8 worlds, raw | signal over noise, shared | signal over noise, raw |
|---|---|---|---|---|---|---|
| 0.1 | 94.74 | 38.70 | 13.68 | 33.49 | 0.48 | 0.20 |
| 0.25 | 90.18 | 47.86 | 16.92 | 31.88 | 1.16 | 0.61 |
| 0.5 | 88.06 | 58.49 | 20.68 | 31.13 | 1.80 | 1.20 |
| 1.5 | 80.63 | 71.95 | 25.44 | 28.51 | 1.26 | 1.12 |

Two readings matter. The signal rises with sigma up to 0.5 and then falls, and
the noise rises with sigma throughout. So the ratio has a maximum, and the
maximum is at 0.5 and not at 1.5.

**A large sigma also makes the population worse.** The centre scored 100.63.
The population mean is 98.19 at sigma 0.1 and 51.03 at sigma 1.5. The
population won 47 of 64 episodes at sigma 0.1 and 38 of 64 at sigma 1.5. The
best single candidate is 125.36 at sigma 0.1, 125.20 at 0.25, 122.69 at 0.5 and
107.62 at 1.5. **Sigma 1.5 does not find better candidates than a local sigma
finds. It finds worse ones.**

## 5. The worlds each candidate needs, and whether a ranking repeats

| sigma | worlds needed, shared | worlds needed, raw | agreement of two 4-world rankings | agreement the decomposition predicts |
|---|---|---|---|---|
| 0.1 | 35 | 210 | 0.476 | 0.103 |
| 0.25 | 6 | 22 | -0.095 | 0.401 |
| 0.5 | 3 | 6 | 0.905 | 0.619 |
| 1.5 | 6 | 7 | 0.857 | 0.441 |

The agreement column is the rank correlation between the candidate ordering on
worlds 0 to 3 and the ordering on worlds 4 to 7. **A rank correlation over 8
candidates has a standard error of 0.378 when the true value is zero.** The
0.905 at sigma 0.5 and the 0.857 at sigma 1.5 are therefore distinguishable
from zero, at 2.4 and 2.3 standard errors. The 0.476 at sigma 0.1 is not. The
difference between 0.905 and 0.857 is not distinguishable at all, so this
instrument alone does not choose between sigma 0.5 and sigma 1.5.

The minus 0.095 at sigma 0.25 needs an explanation, because the decomposition
predicts 0.401 there. Two of the eight candidates scored the same number to six
decimal places, and a third scored within 0.1 of them. Their ordering is
therefore arbitrary, and a rank correlation is harsh on an arbitrary ordering.
The same three candidates hold the same three scores at sigma 0.5. So the
figure is not a contradiction of the decomposition. It is a second reading of
the same fact: a local sigma produces candidates that no world set can separate.

### The tie guard does not catch this

The search refuses to move when the spread of a generation is exactly zero, and
it says so in the log.[^1] **That guard fires only when the whole generation
ties.** Every generation measured here has a non-zero spread, so the guard
passes each of them.

At sigma 0.25 and at sigma 0.5, two of the eight candidates scored the same
number exactly. The guard passed both generations, and the sort then ordered
those two by candidate index. That ordering reaches the step. At sigma 0.1 five
of the eight candidates scored within 2 points of the centre, and the smallest
gap between two of them is 0.09. **A partial tie is the common case and the
guard does not see it.**

## 6. The derivation of the worlds needed

The model is one line. The score of candidate `c` on world `w` is
`mu + a_c + b_w + e_cw`. The term `a_c` is what the policy is worth, `b_w` is
what the world is worth, and `e_cw` is the interaction of the two.

Every candidate of a generation plays the same worlds, so `b_w` is common to all
of them and cannot change an ordering. The noise on the mean score of one
candidate over `W` shared worlds is therefore the standard deviation of `e`
divided by the square root of `W`.

The signal exceeds the noise when `W` is above the square of the residual
standard deviation divided by the square of the signal standard deviation. For
sigma 0.5 that is 58.49 squared over 37.30 squared, which is 2.46, so 3 worlds.
For sigma 1.5 it is 71.95 squared over 31.95 squared, which is 5.07, so 6
worlds. For sigma 0.25 it is 5.98, so 6 worlds. For sigma 0.1 it is 34.9, so 35
worlds.

The raw column repeats the derivation with the raw within-candidate standard
deviation in place of the residual one. That column answers for a search that
gave each candidate its own worlds. It is 6 worlds at sigma 0.5 and 210 worlds
at sigma 0.1.

**The shared world set is worth a factor of six at sigma 0.1 and a factor of
1.2 at sigma 1.5.** The ratio of the two columns is 6.0, 3.7, 2.0 and 1.2 as
sigma rises. A large perturbation moves the score in a way that depends on the
world, so sharing the worlds cancels less of the noise.

The signal estimate comes from 8 candidates, so it carries at least 27 percent
relative error. It comes from a difference of two mean squares, so the true
error is larger. The worlds-needed figure is a ratio of two such quantities.
Treat it as an order of magnitude and not as an exact count. The gap between 3
and 35 survives the error, and the gap between 3 and 6 does not.

## 7. The answer to the question, and the recommendation

The question was whether more worlds for each candidate buy back the ranking
spread that a local sigma gives up. **The answer is no for a very local sigma
and yes for a moderate one, and neither case needs more worlds than the run
already plays.**

Sigma 0.1 would need about 35 worlds for each candidate. That is nearly nine
times what the run plays. Five of the eight candidates at that sigma score
within 2 points of the centre, so most of that population is a set the world
count cannot separate. Buying the world count is not worth it there.

Sigma 0.5 needs 3 worlds. It has the largest signal of the four, the largest
signal-to-noise ratio of the four, the highest measured rank agreement of the
four, and a population whose best candidate beats the best candidate at sigma
1.5. It changes 0.11 of the choices, so it is a real perturbation and not a
tie.

**The recommendation is the pair (sigma 0.5, four worlds for each candidate).**
Sigma falls by a factor of three. The world count stays where the trainer
configures it.[^2] Four worlds is above the derived need of 3, and the margin
covers the error on the derivation.

The incumbent pair is (sigma 1.5, four worlds). **That pair is under-sampled by
its own arithmetic**: sigma 1.5 needs 6 worlds and gets 4. So the incumbent has
two defects rather than one. It ranks candidates that are not neighbours of the
centre, and it ranks them on too few worlds.

A run that will not change its world count should still lower sigma, because
sigma 0.5 needs half the worlds that sigma 1.5 needs. A run that can afford 6
worlds should still lower sigma, because the signal at 0.5 is larger at every
world count.

## 8. What this did not reach

Each item below is a measurement and not an argument, so the next agent can
take it directly.

| What is missing | Why it matters |
|---|---|
| A second centre, and a second play style | Every figure here is around one centre under one style. The shape of the curve may be a property of this centre, which is close to a constant policy |
| A centre trained through the feature normalizer | The centre here is a trained centre read through a transform it was not trained through. A run started today produces a different object |
| The measurement at a zero centre | A fresh run starts at a centre of length zero, where sigma falls back to a length of one. The first generations of a run are that case, and this note says nothing about them |
| A rank agreement over more than 8 candidates | The standard error of 0.378 makes the difference between sigma 0.5 and sigma 1.5 indistinguishable on that instrument alone. The recommendation rests on the signal-to-noise ratio instead |
| Whether the step the search takes improves the centre | This note measures whether a ranking repeats. It does not measure whether the ranking points anywhere useful. A ranking can be perfectly reproducible and still rank a quantity that does not predict play |
| The locality over all 8 worlds | The share of changed choices comes from 2 of the 8 worlds, so a candidate that looks local here may not be local elsewhere |

**One figure of this note is unverified.** The claim that the recommended pair
trains better than the incumbent pair is not measured. This note measures the
inputs of a ranking, and a training run measures the outcome. Nothing here says
how a run at sigma 0.5 ends.

## 9. What this cost

The main run played 256 full episodes for the four sigmas, 8 for the centre, and
2 for the locality sample. It took 3036 seconds of wall clock with 6 engine
workers. The probes that chose the design took a further 220 seconds, and the
calibration cross-check took 40 seconds. Three derivations of the feature
normalizer played 12 truncated episodes each.

The whole measurement is therefore about 55 minutes of wall clock on a
development machine with 16 cores. Another agent was working on the same machine
throughout, so the rate varied between 83 and 280 world ticks each second. The
measurement did not run on the target platform, so no figure here is a cost
figure.[^11]

## References

[^1]: The search module, the perturbation scale and the tie guard. `python/cachette/learn/search.py`
[^2]: The training configuration, sigma and the seeds of a generation. `python/cachette/learn/config.py`
[^3]: Report on what is wrong with training and evaluation, items 6 and 13. `docs/research/what-is-wrong-with-training-and-evaluation.md`
[^4]: Commit 35bbe42a, hold the policy centre at unit length. `git show 35bbe42a`
[^5]: Stored policies index, the aggressive policy at generation 7. `checkpoints/README.md`
[^6]: The play style table, the aggressive style. `python/cachette/learn/play_styles.toml`
[^7]: The strategy table of the trainer, and the world it states. `python/cachette/learn/__main__.py`
[^8]: The reference sample of the feature normalizer. `python/cachette/learn/normalize.py`
[^9]: The trainer, how a resume reads a checkpoint that states no normalizer. `python/cachette/learn/train.py`
[^10]: The sigma calibration tool. `scripts/calibrate_sigma.py`
[^11]: Blockers register, BLK-007. `docs/BLOCKERS.md`
