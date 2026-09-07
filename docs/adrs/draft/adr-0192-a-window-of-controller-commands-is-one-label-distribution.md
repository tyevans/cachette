# ADR-0192: A window of controller commands is one label distribution over the action table

## Context

A learner plays one faction of a world while the built-in controller plays the
rest.[^1] The learner acts by one integer over a bounded table of actions, and
the engine declares that table in a schema.[^2] [^3] The verb set of the table
is the set the controller's own choice enumeration holds, plus a no-op row, so
a controller choice and a learner action reach one encoding.[^4] [^5]

**A trainer that scores an episode by its return gets one number for a whole
game.** The project fits a policy by an evolution strategy, and that strategy
spends a large number of episodes to estimate one direction. The same episodes
already hold what the controller chose at every decision. Reading those choices
turns each episode into many labelled samples, and a supervised fit over them
answers a question no return can answer: whether the observation array holds
what the controller reads.

**The two sides do not act on the same clock, and that is the whole
difficulty.** The controller plans its commands once for each tick, and it
emits one for each entry of its draw order that fires. The learner takes one
decision every several ticks, and the tick count between two decisions is a
parameter of the environment. A window between two learner decisions therefore
holds many controller commands and one learner action.

**No reduction of that window is lossless.** The learner cannot send the set
the controller sent, because the action table takes one row for one decision.
Any rule that turns the window into one label throws something away, and the
rule decides what. The rule also decides what an accuracy figure means, so
every diagnostic taken from such a dataset is read against it. A reader who
finds two figures under two rules cannot compare them.

The window also holds two facts that a naive rule would lose. A window may hold
no command at all, and a window may hold a choice that the action table cannot
express. The two look the same in a log that carries only an action integer,
because the table refuses a choice by giving nothing and the no-op row is
zero.[^5]

## Decision

**A window of controller commands is one label distribution over the action
table, and a fit against it minimises the cross entropy of that distribution.**

### D1. Every command of a window carries the observation of that window

The observation of a sample is the array the learner would read at the start of
the window, taken through the reader that answers for one faction and at the
point in the tick the learner reads it.[^6] Every command the controller emits
inside the window carries that observation.

A recorder must not read the observation at any other point of the tick, and it
must not read a reader that answers from the truth of the whole world. A
dataset built from an observation the learner never sees teaches a policy a
game nobody plays, and no accuracy figure from it says so.

A reviewer finds a violation when a recorder reads the observation after the
window rather than before it, when it reads a reader outside the faction-scoped
set, or when it records a window whose length is not the decision interval of
the environment it claims to model.

### D2. The label of a window is the share each action row holds of its commands

The label is one number for each row of the action table. The number is the
count of the commands of the window that encode to that row, divided by the
count of the commands of the window. The numbers of one window sum to one.

**One window contributes one unit of loss, whatever number of commands it
holds.** A rule that gave one unit to each command would weight a busy tick
above a quiet one, and the busy tick is the one whose commands are least
separable.

A window that holds no command at all carries the no-op row, because doing
nothing is what the controller did and the no-op is the row that says so.

A reviewer finds a violation when a label row does not sum to one, when a
window contributes more than one unit of loss, or when an empty window carries
anything but the no-op.

### D3. A choice the table cannot express is counted and never labelled

The engine states, for each command, whether the action table could express the
choice. A recorder reads that statement before it reads the action integer.

A command the table cannot express contributes no label. A window whose
commands the table could not express at all contributes no sample, and the
recorder counts it. **A report of a dataset states the share of the commands
that the table could not express.** A silent drop would put the no-op label on
a window the controller acted in, because the refused encoding and the no-op
row are one integer.[^7]

A reviewer finds a violation when a recorder reads an action integer without
reading whether it is an encoding, when a refused command reaches a label, or
when a report of a dataset states no such share.

### D4. A figure taken from such a dataset is read against a constant answer

A report of an accuracy on this dataset states, beside it, what one constant
answer scores on the same set, and how many different action rows one window
holds on average.

**A label that spreads over many rows bounds every accuracy, and the bound is a
property of the data and not of the policy.** A reader who takes an accuracy
without the constant answer cannot tell a policy that read the observation from
one that named the most common row.

A reviewer finds a violation when a report gives an accuracy with no constant
answer beside it, or when it gives one accuracy figure without saying whether
it counts commands or windows.

### D5. The recorder writes nothing to the world

A recorder reads the world between two ticks, at the frame barrier, and it
writes nothing. The engine is therefore free to give the same dataset for the
same seeds.[^8]

A reviewer finds a violation when a reader passed to the environment calls a
verb, when it changes a weight, or when the dataset of one seed set differs
between two runs of one binary.

## Alternatives rejected

**Take the first command of the window.** The engine sorts the commands of one
tick by the faction and then by the draw index, and the draw index is a slot in
the plan rather than a preference.[^9] The first command of a window is
therefore always the command of the lowest draw index that fired, which is an
evaluation. Every verb that sits at a high draw index would never appear in a
label at all. The loss is systematic and it falls on the verbs that decide a
game.

**Take the command the controller ranked highest.** The controller states no
rank. It draws whether each command fires, and the draw indexes order the
commands so that they apply in an order the data fixes. To read a draw index as
a preference would invent a rank the engine does not hold, and a later reader
would take the invented rank for a property of the controller.[^10]

**Emit one sample for each command, each weighted one.** This is the same
dataset as this record states, under a different weighting. It gives a busy
window more weight than a quiet one. A window is one decision of the learner,
so a busy window is not a more important decision; it is a window the learner
can answer less well. The rule would put the most weight where the label is
least separable.

**Change what one learner decision means, so that a window holds one tick.**
This makes the mapping exact, and it changes the environment the trainer
measures. Every figure of the current training runs would then describe a
different game, and the sample cost of one episode would rise by the decision
interval. The mismatch is a property of the two clocks, and moving one clock to
remove it costs more than reading the window as a distribution.

**Give the learner a set-valued action, so that it can send what the controller
sends.** The action table takes one integer for one decision, and that shape
carries two accepted records.[^2] [^3] A set-valued action is a change to the
learner interface and not to a dataset rule, and it belongs in a record of its
own if the project wants it.

## Consequences

**An accuracy on this dataset has a ceiling below one, and the ceiling is not a
defect.** A window that holds several different rows cannot be answered exactly
by one row. A report must therefore state the spread of the labels beside the
accuracy, and a reader must not read a figure below one as a failure of the
policy alone.

**The project can now separate two questions it could not separate.** A fit
that stays near the constant answer says the observation array does not hold
what the controller reads. A fit well above it says the array is sufficient and
the optimiser was the problem. Neither answer was available from a return.

**A policy fitted this way plays the mode of a mixture.** It is not the
controller, and a report must not call it one. The controller acts several
times for each of the policy's single actions, so the policy is at best the
most common of the things the controller did.

**A change to the decision interval changes every label.** The interval fixes
the width of a window, so a dataset recorded under one interval states nothing
about another. A stored dataset or a fitted policy therefore names the interval
it was recorded under.

**The engine must state whether it could encode each choice.** That statement
is now a column of the command log, and a reader that ignores it reads a
refused choice as a no-op.

## References

[^1]: PRD-0056, a learner plays one faction against the controllers. `docs/product/accepted/prd-0056-a-learner-plays-one-faction-against-the-controllers.md`
[^2]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decisions D2 and D4. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
[^3]: ADR-0176, an action integer is a mixed radix over the argument positions each verb declares, decision D1. `docs/adrs/accepted/adr-0176-an-action-integer-is-a-mixed-radix-over-the-positions-a-verb-declares.md`
[^4]: ADR-0176, an action integer is a mixed radix over the argument positions each verb declares, decision D4. `docs/adrs/accepted/adr-0176-an-action-integer-is-a-mixed-radix-over-the-positions-a-verb-declares.md`
[^5]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decision D6. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
[^6]: PRD-0001, a faction sees only what it observes. `docs/product/accepted/prd-0001-a-faction-sees-only-what-it-observes.md`
[^7]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
[^8]: ADR-0001, one binary gives one answer at any thread count, decision D1. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^9]: ADR-0004, iteration order is explicit, decision D4. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^10]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decision D4. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
