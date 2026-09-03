# Review 0223: The tier record

## What was reviewed

| Item | Value |
|---|---|
| `docs/adrs/draft/adr-0043-a-declared-tier-enforces-the-no-loop-rule.md` | `Draft` at review, and `Draft` after it |
| Commit | `79dcd9b`, the head of the review branch |
| Code read | the tier module, the character arena, the soldier arena, the settlement arena, the bindings, the type stubs, and the public interface tests |

The reviewer did not write this record. The reviewer wrote review 0223 of the
control plane record immediately before this one, and so came to this record
already holding the objection that section 2 repeats. Section 4 states what the
reviewer did about that.

**The reviewer compiled nothing.** Other workers hold the machine.

## Verdict

**Accept with amendment.** The five decisions are right, and D3 in particular is
the decision this project would most regret not having. Two statements about the
code are wrong, and both are the kind this project punishes: they claim more of
the artefact than the artefact does.

Section 5 gives the exact text for both. The status stays `Draft`.

**Four objections were attempted against the decisions themselves and all four
failed.** They are in section 3.

## 1. The first wrong statement: nothing checks the tier of a mass shape

The context says:

> The tier is a constant in the core crate. The core crate checks it once, when
> it builds the storage of a shape.

**That is true of one shape out of three, and it is false of both shapes this
record is about.**

The tier module gives each tier a population ceiling. The character tier states
one. The mass tier states none, deliberately, because the whole point of the
mass tier is that no ceiling bounds it. The only code that reads a declared tier
is the character arena, which reads its own ceiling to size itself at
construction.

The soldier arena and the settlement arena contain no mention of the tier at
all. Their declarations live in the tier module and nothing reads them. A search
of both modules for the trait, the constant or the word returns nothing.

So for the two mass shapes the tier is a declaration that reaches no code
anywhere, in either crate. The record's second paragraph says the tier reaches
no code outside the core crate. It reaches none inside it either.

**This matters because the record rests on it.** The context argues that the
first part of the rule exists and the second does not. The declaration exists.
The check does not, for these shapes, and a reader who takes the sentence at
face value believes there is a mechanism to extend rather than one to build.

## 2. The second wrong statement: the interface does not merely fail to refuse, it offers

D5 says no Python type refuses a loop today, and that every document must state
the rule as a rule until the refusal exists. That is honest and it is correct.

It is also not the whole of what is true. **The interface already breaks D1 in
four places, and no refusal would have caught three of them, because the engine
offers them.**

D1 says a mass-tier shape gets a set-valued interface, that no verb over it
names one entity, and that no read over it walks the population. Both the
soldier shape and the settlement shape declare the mass tier. Against that, the
bindings answer the tile of one soldier, the positions of one site and the
preference of one site, and the agent server wraps the first as a tool that
reads one unit.

The type stub goes further than permitting it. The documentation of the gather
event columns instructs a reader to take a value from the unit column and hand it
back to the per-unit read.

A test already pays four crossings for each site, because the set-valued command
carries one target for the whole set and the test needs a different target for
each. The measurement is in the register, with the evidence.[^1]

**D5 as written tells a reader the surface is clean and unguarded.** It is
neither. The distance between "nothing stops a violation" and "the engine ships
four" is the distance this record exists to close.

## 3. The objections that failed

**Does the character ceiling break D3, which forbids a check on a count?** This
is the strongest objection available and it fails. D3 forbids a check that reads
the current population and decides whether to refuse a call, because such a
check passes in development and fails at scale. The character arena instead
reads a constant of its own type at construction and sizes itself to it. It
consults no population, it runs once when the world is built, and it gives the
same answer on every machine and every world. That is the load-time guard, not
the runtime cardinality check, and the two differ exactly where D3 says they do.

**Is D1 implementable, or does it ask for something the binding layer cannot
do?** D1 says the boundary derives its surface from the tier rather than
restating it. The objection fails. Which methods a type exposes is decided when
the bindings are compiled, and the tier is a compile-time constant of a sealed
trait, so a generic wrapper or a macro over the shape can decide the surface
from it. The decision constrains how the enforcement is built rather than
promising that it is built, and D5 says the latter plainly.

**Does the spawn verb break D4 by handing out entities?** The objection fails.
D4 forbids an operation that yields entities one at a time. The spawn returns
one column in one crossing, and a column is the escape hatch D4 keeps.

**Does the holder column of the site positions break D4, since it carries whole
identities?** The objection fails for the same reason, and the identity record
governs what crosses inside it.[^2] A column of identities is one answer. It is
not the population handed out one at a time.

## 4. What the reviewer did about arriving with the objection already formed

The reviewer found the per-entity reads while reviewing the control plane record
and so was not a fresh reader of this one. A reviewer that already holds one
objection tends to find that objection and stop.

The reviewer therefore read this record's decisions first and looked for reasons
to reject each one on its own terms, before returning to the known objection.
Section 3 is the result, and it is longer than section 2. The two wrong
statements in sections 1 and 2 were then checked against the source rather than
against the earlier review: section 1 is a new finding that the earlier review
did not have, and it was found by asking which code reads a declared tier.

## 5. The amendment

The reviewer did not edit the record. The text below is a proposal for the
author.

**First**, replace the context sentence:

> The tier is a constant in the core crate. The core crate checks it once, when
> it builds the storage of a shape.

with:

> The tier is a constant in the core crate. One shape reads its own tier: the
> character arena sizes itself to the ceiling the character tier states. The
> mass tier states no ceiling, so nothing reads the tier of a mass shape, in
> either crate. For the two shapes this record is about, the declaration is a
> statement that reaches no code at all.

**Second**, add to D5, after the sentence "The tier reaches no code outside the
core crate":

> **The interface does not merely fail to refuse. It offers.** Three reads name
> one entity of a mass-tier shape: the tile of one soldier, the positions of one
> site and the preference of one site. The agent server wraps the first as a
> tool. The type stub tells a reader to hand a unit identity back to the
> per-unit read. A refusal would not have caught any of these, because the
> engine documents and answers them. A caller that wants a different value for
> each member of a set has no command form and sends one command for each
> member. The finding holds the evidence and an item holds the repair.

The references for the finding and the item go in the reference list as
footnotes.

## 6. Why the record is worth accepting once amended

D3 is the decision that pays for the record. A refusal that reads the population
is the design a contributor reaches for, it passes every test written against a
small world, and it fails in production far from its cause. Nothing in the code
says why that design was refused, and the tier constant on its own does not say
it, because for a mass shape nothing reads the constant.

That is a constraint a future contributor could reasonably choose against,
expensive to reverse once scripts exist, and invisible in the artefact. It is
the case the scope rule describes.

## References

[^1]: Findings register, FND-215. `docs/FINDINGS.md`
[^2]: ADR-0085, an entity crosses to Python as one opaque identity that the engine resolves. `docs/adrs/accepted/adr-0085-an-entity-crosses-to-python-as-one-opaque-identity.md`
