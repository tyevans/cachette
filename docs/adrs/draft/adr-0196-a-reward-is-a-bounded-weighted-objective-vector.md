# ADR-0196: A reward is a bounded weighted objective vector, and one generation scores one objective

## Context

The learner of this project trains one faction of one world. It needs one
number on every decision. The engine holds no reward, and that is a decision
rather than a gap. What a faction should be rewarded for is a rule of the
downstream game, so a product record refuses to state it.[^1] The reward
therefore lives in the control plane, where floating point is allowed and
where nothing the engine stores can read it.[^2]

The first reward held one weight for each field the engine publishes. That
reward trains one behaviour. A project that wants an aggressive policy beside
a trading one needs several weightings over one mechanism, and not several
mechanisms. An objective is a named quantity built from the signals of one
observation. The objective vector is the set of the objectives. A play style
is a weighting over that vector, and the scalar the learner receives is the
weighted combination. A research report works this out and states a
design.[^3]

Three things went wrong under the first reward. Each of the three is
measured.

**A reward of first differences carries no signal inside an episode.** The
optimiser of this project is an evolution strategy. It ranks a candidate by
the undiscounted sum of the rewards of its decisions. A sum of first
differences telescopes, so the sum over an episode is the weighted difference
between the last reading and the first. The findings register holds the
measurement and holds how closely the two agree.[^4]

**A term over a raw count means a different thing on two worlds.** Half of a
small world and half of a large world are the same play and a different
number, so a weight tuned on one misreads the other. The trainer takes a
fixed normalised step, so an input whose range spans many orders of magnitude
cannot be trained. A draft record already removes the raw count from what the
engine publishes.[^5] That record states no rule for the terms a reward
builds from a published value.

**A style that weights every objective trains what every other style
trains.** The first styles were code, and an unmentioned objective read as a
weight of nothing. A run then reported a generation under an objective its
author did not choose, and no register held the difference.

The engine owns the layout of the observation and states it in one
schema.[^6] A reward term therefore names its signal as data, and no field
name of the engine appears in the control plane. One blocker holds the rules
of the downstream game, so it governs every weight of every style.[^7] A
register holds one row for each weight, and every row is unset.[^8] **This
record states no weight.**

## Decision

**Every reward term is bounded by a declared kind. A play style states a
weight or a refusal for each objective of the vector. One generation scores
every one of its candidates under one objective.**

**This record extends the draft record that makes every published value scale
free.**[^5] That record governs what the engine publishes. It states no rule
for the quantity a reward builds from a published value, and a bounded input
does not give a bounded output. A reward can take the difference of two
shares, or multiply a compressed magnitude by any weight a researcher writes.
D1 and D2 below are the counterpart of that rule on the reward side. Every
decision of that record stands.

### D1. Every term maps a signal into a bounded interval through a declared kind, and no term reads a raw quantity

A term names the signal it reads and the kind that bounds it. Each kind maps
the signal into the closed interval from minus one to one. The kinds are a
share against a denominator the schema names, a signed relation between two
signals, a compressed magnitude against a structural cap, and a fixed-point
value for a signal the engine already publishes as a share.

**There is no raw kind.** A caller cannot write a term over a count, and
cannot write a term over a total whose bound is the integer range.

A share needs a denominator the engine publishes, and the reward invents
none. A caller with no denominator uses a compressed magnitude and accepts
the bias of that kind: the derivative of a compressed magnitude falls with
the quantity, so an early gain outweighs a late gain of the same size.[^9]

A reviewer finds a violation when a term reads a signal without a kind, when
a share names no denominator, when a denominator follows the episode or the
data seen so far, or when a term returns a value outside the interval.

### D2. The weighted combination divides by the sum of the absolute weights

The scalar of one decision is the weighted sum of the objective vector
divided by the sum of the absolute weights of the style. One decision's
reward therefore stays inside the range of one objective, whatever a style
weights and however many objectives it weights.

The trainer takes a step of a fixed size. An unnormalised sum would make the
step of a style with many weights larger than the step of a style with few,
so two styles would differ in the step as well as in the objective. A
researcher comparing two styles could not say which of the two differences
moved the policy.

A reviewer finds a violation when the reward of one decision leaves the range
of one objective, or when the divisor depends on the episode.

### D3. A style names every objective it refuses, and a weight of nothing is refused

A style holds a weight for each objective it rewards and a refusal for each
objective it does not reward. Every objective of the vector appears in one of
the two, and never in both. The reward refuses a style that leaves an
objective unmentioned.

A weight of nothing is refused as well. A weight of nothing and a refusal
mean the same thing to the arithmetic and different things to a reader, so
the reward admits one of the two spellings.

**This makes a style that weights everything unconstructible, and not merely
discouraged.** Such a style trains the same policy as every other style, so a
run that holds several of them measures one policy several times and reports
several answers.

One function declares this rule. The loader of the style table calls it
against the objectives the table declares, and the scoring calls it against
the objectives one world supplies. A second copy of the rule would be one
fact in two places, with nothing that fails when the copies disagree.

A reviewer finds a violation when a style loads while an objective of the
vector reads a default, or when a second site states the completeness rule.

### D4. A run holds one objective for every candidate of one generation, and one objective for a whole validation pass

An evolution strategy ranks the candidates of one generation against each
other. Two candidates scored under two objectives give a rank that says
nothing about either policy, and the update follows that rank.

A run may vary the objective between generations. A run may also vary it
between the episode positions of one generation, when every candidate of that
generation gets the same objective at the same position. The position is the
index of a seed inside the seed set of the generation, and one call steps a
batch of worlds in that index order.[^10] The objective is therefore a
function of the generation number and of the episode position. It
is a function of no candidate, of no shard boundary and of no worker process.

A validation pass and a holdout pass hold one objective for the whole run.
The run keeps the centre that scored highest on the validation seeds, and two
scores under two objectives cannot be compared.

The record that shards a generation states that the shard count changes the
spread of the work and nothing else, and that the combination joins the score
arrays in candidate order.[^11] That record fixes the order of the scores.
This decision fixes what makes them comparable, which that record does not
state. It also closes one form that record permits: an objective that follows
the candidate index would keep every rule of the sharding record, because a
worker knows the range of candidates it owns and would compute the same
objective for the same candidate at every shard count.

A reviewer finds a violation when a scorer is chosen by a candidate index,
when a scoring interface offers an argument for one candidate, or when the
run compares a validation score of one generation against a validation score
of another under a different objective.

### D5. The scoring names the optimiser that reads the reward, and it refuses a term form that optimiser discards

A shaping term of the difference form leaves the optimal policy unchanged
under a method that follows a discounted gradient. Under an undiscounted
evolution strategy the same term telescopes, and the findings register holds
that measurement.[^4] **The admissible form of a shaping term therefore
follows the optimiser, and this project cannot choose the form once.**

The scoring of a run names its optimiser. Under the evolution strategy it
refuses a weighted objective whose every term reads a change, and it names
the level form of the same signal in the refusal. Under a discounted gradient
method it admits the same objective, because that method discounts inside the
episode.

A reviewer finds a violation when the reward admits a term form without
naming the optimiser, or when any document states one shaping form for every
method.

## The alternatives this rejects

**A raw term with a weight tuned for one world.** This is the simplest
reward, and it is what the first reward did. The register that holds the
weights holds one row for each term and no row for a world. A per-world
weight would need a row for each world, and a stored policy would carry the
world it trained on in a place no schema declares. A researcher moving a
style to a larger world would read a plausible number and a different
behaviour.

**A weighted sum with no divisor.** The weights then read as the researcher
wrote them, and a report needs no explanation of the divisor. This is
rejected because the step size of the trainer is fixed, so the sum decides
how far one generation moves. A style is meant to change what the run
rewards, and not how far it steps.

**A default weight of nothing for an unmentioned objective.** A
configuration reader usually does this, and it costs the writer of a style
nothing. It is rejected because the failure is silent. The run completes, it
reports a number, and the number answers an objective nobody chose. A refusal
is a statement, and silence is not.

**A weight of nothing as the way to spell a refusal.** This needs no second
entry and no completeness rule. It is rejected because a reader cannot tell a
refusal from an omission, and those are the two cases D3 exists to separate.

**A per-candidate objective, to widen the search of one generation.** A
generation would explore several objectives at once, and a run would need
fewer generations to compare them. It is rejected because the rank of the
generation then carries no information about either policy, and the update
follows the noise. A run that wants to search over objectives spends
generations on it.

**The potential difference form of a shaping term, chosen once for the
project.** The form has a theorem behind it, and the theorem says the optimal
policy does not move.[^12] It is rejected because the theorem needs a
discount inside the episode, and the optimiser of this project applies none.

**A reward computed inside the engine.** The engine holds the quantities, so
a reward there would read them without a boundary crossing. It is rejected
because a reward inside the engine enters the state hash, so every golden
file moves when a researcher changes a weight.[^13] A weight would then be a
property of a deterministic build, and the register that holds the weights is
a register of unset rows under one blocker.[^7]

## Consequences

A researcher cannot write a term over a quantity the engine publishes without
a denominator, unless they accept the bias of a compressed magnitude. Asking
the engine for a denominator is engine work, and D1 turns a missing
denominator into that request.

Every style must be edited when the objective vector gains an objective.
Every style already written then fails to load until somebody states a weight
or a refusal for the new objective. That is the cost of D3, and it is the
intent of D3. A new objective is a question that every style must answer.

A run cannot search over objectives inside one generation. Searching over
objectives costs generations, and D4 makes that cost explicit.

A stored policy is meaningful only beside the optimiser that trained it. D5
makes the admissible term forms a function of the optimiser, so a style and
an optimiser travel together. A run that moves to a discounted gradient
method must revisit the measure of every term.

The reward holds no weight. One blocker holds the rules of the downstream
game, and the register that holds the weights carries one unset row for each
of them.[^7] [^8] The styles the package ships are therefore data. They state
a shape that D1 to D3 admit, and they state no constraint.

Two objectives are not separable while the engine publishes them as one
quantity. The findings register records that the population field and the
live unit count read the same value in every sample taken.[^14] A style that
rewards growth and a style that rewards force then differ in nothing the
observed data shows. D1 does not repair this. A bounded term over a
duplicated position is still a term over a duplicate, and the repair is
engine work.

## References

[^1]: PRD-0056, a learner plays one faction against the controllers, the first checkable statement. `docs/product/accepted/prd-0056-a-learner-plays-one-faction-against-the-controllers.md`
[^2]: ADR-0002, state holds no floating point number, decision D4. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^3]: Research report 42, what a policy should be able to see, sections 10.2 and 10.3. `docs/research/reports/42-what-a-policy-should-be-able-to-see.md`
[^4]: Findings register, FND-676. `docs/FINDINGS.md`
[^5]: ADR-0195, the observation of a faction is a fixed-width scale-free table in an egocentric frame, decision D2. `docs/adrs/draft/adr-0195-the-observation-of-a-faction-is-a-fixed-width-scale-free-table.md`
[^6]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables the engine owns, decision D1. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
[^7]: Blockers register, BLK-050. `docs/BLOCKERS.md`
[^8]: Reinforcement learning parameters register, the reward rows. `docs/reference/rl-costs.md`
[^9]: Research report 42, what a policy should be able to see, section 10.1. `docs/research/reports/42-what-a-policy-should-be-able-to-see.md`
[^10]: ADR-0155, a batch of worlds steps in one call, in index order, decision D1. `docs/adrs/accepted/adr-0155-a-batch-of-worlds-steps-in-one-call-in-index-order.md`
[^11]: ADR-0194, a generation is scored in shards and combined in candidate order, decisions D1 and D3. `docs/adrs/draft/adr-0194-a-generation-is-scored-in-shards.md`
[^12]: Ng, Harada and Russell, policy invariance under reward transformations, International Conference on Machine Learning, 1999.
[^13]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^14]: Findings register, FND-677. `docs/FINDINGS.md`
