# ADR-0201: A policy attends over the entity tokens, and reads the ring stack by a shared kernel

## Context

The engine publishes an observation of fixed width. It holds an egocentric ring
stack, several sets of entity tokens, and a block of scalar positions. The
schema declares the width, and no caller states an offset of its own.[^schema]
A token position names no seat, so a reader must treat a set as a set.[^set]

The policy this project trains today reads each token set with one shared
encoder. It then reduces the result with a mean and a maximum. A pooled
reduction keeps the field and loses the joint structure of the set. A reader
cannot tell from a mean and a maximum whether the strongest rival is also the
nearest rival. The research report names a masked attention over the tokens as
the established answer to that loss.[^report]

One law governs the cost of any change to a policy here. The trainer is an
evolution strategy. The cosine between the step one generation takes and the
direction it looks for is near the square root of the pair count divided by the
trainable count.[^law] A larger policy is not slower to train. It is worse
aimed. Population is the only cure, and population costs episodes.

The trainable count of the present policy is not spread evenly. The readout and
the dense scalar layer hold most of it, and the shared weights over the ring
stack and over the token sets hold little.[^law] An attention head over the
token sets therefore costs a small part of the whole. The population a design
needs is arithmetic, and a reader computes it from the law before any run.

The ring stack carries a second property. Its sector axis wraps, so one kernel
over that axis states one rule for every direction. The report gives that
property as a reason to prefer a ring frame to a square crop.[^report] The
present policy already exploits it, and an attention over the ring cells would
give the property up in exchange for weights the ring mixing already holds.

Parts of the observation read zero in every frame. The ring stack cannot fill
the memory age channel or either strength channel, and it fills several further
channels in the near rings alone.[^ring] Most power ratios of a rival token
read zero, because the fog admits an estimate of the settlement ratio
alone.[^token] A reader that attends over a set whose discriminating channels
are absent spends population on nothing.

A policy runs in the Python control plane and holds floating point weights. A
reader will ask whether that breaks the arithmetic rule of this project. It
does not. The rule governs simulated state and an aggregate, and it admits a
floating point number outside them.[^float] The rule that does reach a policy
is the determinism claim, which requires one answer at any thread count.[^det]

## Decision

### D1. A policy attends over the entity tokens

A policy that attends runs its attention over the entity token sets, and over
nothing else. It joins the sets into one set. Each token carries a learned
identity of the set it came from, so the joined set separates a settlement from
a rival without a position that names one.

The attention is permutation-equivariant inside a set, and the reduction that
follows it is permutation-invariant. The reader therefore still meets the rule
that no position of the array names a seat.[^set]

### D2. A policy does not attend over the ring cells

The sector axis of the ring stack wraps. A shared kernel over that axis states
one rule for every direction, and a learned position relation between two cells
states the same rule once for each pair. The kernel is the cheaper statement of
the stronger prior, so it stays.

The ring axis does not wrap, and a learned mixing over the rings already lets a
policy choose its own near and far. Nothing an attention over the ring cells
buys is missing today.

A later record may reverse this. It must first show that the ring channels
carry a value in every ring, because a policy cannot attend to a channel that
reads zero.

### D3. The mask of an attention comes from the validity channel

An absent token reads zero in every channel, and its validity channel says so.
The attention masks a token by that channel and by nothing else.

**The policy never takes a count of present tokens from a caller.** A count
would be a second declaration of a fact the observation already holds, and
nothing would fail when the two disagreed.[^shape]

### D4. An attention head is admitted against a population budget

The population a design needs, to hold the alignment of the design it replaces,
is a property a reader can compute from the law before any run.[^law] A design
that raises that population must state the raise, and must state what buys it.

A design that lowers the trainable count while adding an attention head is
preferred to one that raises it. This project has never trained a policy whose
reward was dense, so the smallest design that tests the claim is worth more
than the largest design the machine can afford.

### D5. The action stays a deterministic argmax, and the policy draws nothing

The policy scores each row of the action table and returns the highest-scoring
legal row. The value that crosses into the engine is one action integer, and
never a floating point number.[^action] The policy holds no random state, and
it takes no draw.

A stochastic policy is not forbidden by this record. It is forbidden until a
record states the key of its draw, because thread-local random state destroys
determinism.[^det]

### D6. The rating over rotated seats settles a comparison of architectures

Two policies trained under two reward weightings hold scores that do not
compare. The rating tool seats the policies against each other and against the
built-in controller, over rotated seats, and reports an error bar for each
pairwise difference.[^league] That difference is the measurement that decides
whether an architecture stays.

A run that raises the training return and does not raise the rating has not
shown that the architecture reads better.

## Consequences

**The project gains one reader of the joint structure of a set.** A policy can
state a rule about the nearest strong rival. It could not state one before.

**The project gives up an attention over space until a later record.** A
long-range relation between two ring cells stays out of reach, and a policy
that needs one has no way to state it.

**A token set now has two readers of its shape.** The engine states the
validity channel and the policy masks by it. The two must agree, and only a
test that drives a policy over an observation with an absent token sees the
disagreement.

**An observation channel that reads zero now costs more than it did.** A pooled
reader wastes one shared weight on a dead channel. An attention reader also
wastes the weight that decides where to look. The work that fills a channel
therefore comes before the work that attends over it.

**The comparison costs a training run and a rating pass.** The rating cannot
order a pair whose difference does not clear its own error bars, so a result
that does not clear them ends the experiment rather than extending it.

**A floating point policy adds no float to the world.** The engine holds no
weight, the state hash reads no weight, and the boundary carries one integer.
The determinism claim is unaffected, and no test of it changes.

## References

[^schema]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decision D1. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
[^set]: ADR-0195, the observation of a faction is a fixed-width scale-free table, decision D4. `docs/adrs/draft/adr-0195-the-observation-of-a-faction-is-a-fixed-width-scale-free-table.md`
[^report]: Report 42, what a policy should be able to see, sections 6.4, 6.5 and 11. `docs/research/reports/42-what-a-policy-should-be-able-to-see.md`
[^law]: Findings register, FND-668. `docs/FINDINGS.md`
[^ring]: The ring stack block of the observation, the channels this block cannot fill. `crates/cachette-core/src/obs_ring_stack.rs`
[^token]: The entity token block of the observation, the channel list of a rival token. `crates/cachette-core/src/obs_token.rs`
[^float]: ADR-0002, state holds no floating point number, decision D4. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^det]: ADR-0001, one binary gives one answer at any thread count, decision D1. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^shape]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
[^action]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decision D4. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
[^league]: The policy rating tool. `scripts/policy_league.py`
