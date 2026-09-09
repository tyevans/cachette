# ADR-0202: A run selects on the win share, and every published figure names its seed set

## Context

The control plane trains a policy with an evolution strategy. The search ranks
the candidates of a generation by a shaped return, which is a weighted sum over
the quantities the engine publishes about the seat, plus a terminal term for the
outcome.[^1] The shaped return is dense. A win is one bit at the end of an
episode, so a search that ranked wins alone would rank ties for most of a run.

**The shaped return trains well and it does not measure play.** A policy can
raise it without learning to play. The register holds the instance: four
published policies are each a fixed preference order over the action rows, the
engine's legality answer supplies what looks like situational play, and every
figure of the run that produced them said the policies were strong.[^2] The
project owner played them and reported one unit wandering.

A run draws three seed sets and they are disjoint. The training pool feeds the
generations. The validation set chooses which centre the run keeps. The held-out
set influences nothing.

**A figure taken on the seeds that chose the centre is a maximum over the passes
of the run.** A run takes many validation passes and publishes the highest, so
the figure carries an optimistic bias that nothing estimates. A figure taken on
the held-out set carries no such bias. The two are different quantities, and a
reader who compares one against the other compares nothing.

A run on a rented machine ends when a wall clock cap ends it. A measurement that
runs only after the training loop returns therefore may never run.

The engine names a winner at the tick limit. Its territory reader compares held
ground there and records the outcome, so every episode ends won or lost and no
episode ends drawn by a timeout.[^3] The win share is therefore a real quantity
over any pass, and not a mostly-zero one.

## Decision

### D1. The win share selects, and the shaped return breaks a tie

A run keeps the centre whose validation pass reached the highest share of
episodes that ended in a win. Two passes with the same win share are ordered by
the mean shaped return.

**The shaped return stays exactly as it is as the training signal.** This
decision governs selection and reporting alone. Nothing about what the search
ranks changes.

### D2. A pass reports both figures, in one value

The pass that plays a seed set gives back the win share and the mean shaped
return together. It never gives back one number that a caller must interpret.

A caller that takes one of the two names which one it took. A caller cannot
silently read the shaped mean where it meant the win share.

### D3. A stored figure names the seed set that produced it

Every figure a run writes to a weight file, to a manifest, to a report or to a
progress line carries a name that says which seed set produced it and whether
that set chose the centre.

A figure from the validation set is a selection figure. A figure from the
held-out set is a measurement. No name may be read as the other.

### D4. The held-out pass runs at an interval

A run plays the held-out set at a stated interval of generations, in the way it
plays the validation set. A run that a wall clock cap ends therefore leaves an
honest figure behind at the last interval it reached.

The pass measures the centre the run would publish, and it chooses nothing.

### D5. A run reports two behaviour instruments, and fails on neither

Every pass over a fixed seed set reports the share of decisions on which the
policy emitted its most common action, and the share of its episodes in which
the highest-scoring row over the **unmasked** action rows changed.

The second reading takes the argmax before the legality mask. A fixed preference
order emits many different actions once the mask removes the rows it cannot
take, so a reading taken after the mask cannot separate a preference from a
policy.

**Both are instruments.** No run fails on either. A threshold on a behaviour
reading would be a second selection rule, and D1 states the only one.

### D6. A resumed run refuses a centre from another feature transform

A run reads a stored centre back only when the file states the feature transform
the run holds. A file that states none is refused for a resume, and it still
loads for play.

Every weight of a centre scores a standardized feature. A centre read under
another standardization means something else at every position, and no other
entry of the file separates the two. That is one fact held in two places with
nothing that fails when they disagree.[^4]

## Consequences

**A run cannot publish a policy on a shaped return alone.** A style whose
weighting rewards something other than winning still trains on that weighting,
and the centre it keeps is the one that won most. A researcher who wants the
old behaviour must change this record.

**A tie is common early in a run.** A small validation set gives a coarse win
share, so many passes tie and the shaped return decides. The set must be large
enough that the win share separates two centres, and the size of it is a
measurement and not a decision.

**Measurement takes a larger share of a run.** The held-out pass at an interval
buys the honest figure, and it buys it with episodes that train nothing. A
caller who cannot afford the pass turns the interval off and gets a run with no
honest figure, which is the state this record exists to prevent.

**Every reader of a stored figure must be updated when a name changes.** D3
makes the name carry the meaning, so a name is an interface. The names live in
the weight file, in the run report and in the generation line the progress
readers parse.

**A resume across a transform boundary now fails rather than proceeding.** A
contributor who holds an old checkpoint must start a fresh run.

## References

[^1]: ADR-0196, a reward is a bounded weighted objective vector. `docs/adrs/draft/adr-0196-a-reward-is-a-bounded-weighted-objective-vector.md`
[^2]: Findings register, FND-707. `docs/FINDINGS.md`
[^3]: What is wrong with training and evaluation, section 14. `docs/research/what-is-wrong-with-training-and-evaluation.md`
[^4]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
