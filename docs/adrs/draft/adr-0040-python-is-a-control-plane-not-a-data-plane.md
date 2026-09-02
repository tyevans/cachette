# ADR-0040: Python is a control plane, not a data plane

## Context

This project simulates a hex world. The core is Rust. The control plane is
Python. A designer writes Python to say what the world must do. The engine then
does it, and gives the same answer at any thread count.[^1]

Python exists here for one reason. A designer changes behaviour without a
compile. Nothing else about Python serves this project, and the language costs
the project an interpreter, a boundary and a second set of types.

The target scale sets the terms. The world holds far more tiles and units than
a script can visit, and the scale constants table holds the figures.[^2] A
crossing that carries one entity is therefore cheap once and ruinous at the
population.

Two ways to place the boundary exist. Python can hold the data and walk it, or
Python can hold the intent and send it. The project chose the second and wrote
the choice as prose. Nothing enforced it.

A finding records what the prose cost.[^3] A test had to produce one gather
event. The gather resolves against the ground a unit stands on, and the control
plane has no read that answers which ground holds a resource. The test
therefore put a unit on every open tile and ordered each one, and let the engine
find the ground. The author of that sweep recorded the rule against sweeping in
the same change. Discipline was not the missing part. The read was.

The same finding holds a second instance, through the type system. A verb
returned an identity column, and the next verb declared a sequence of integers.
A caller that passed the column straight on met a type error, and the obvious
repair was to build a list around the column. Every conversion of a mass-tier
column is a loop.

The need behind both instances is ordinary. A developer wants a unit to have a
reason to be somewhere, which means the developer must be able to name the
places that hold something worth taking.[^4]

The project owner closed the shape of the answer. A command takes a set. A
verb that takes a set has no per-entity form for a caller to repeat, so the rule
about looping stops being a rule a caller can break.[^5]

## Decision

### D1. The boundary carries an instruction and an answer, never the population

Python sends a description of the work. Rust performs the work over the columns
and returns the answer. Python receives what it asked for. Python does not
receive the world so that it can find the answer itself.

A read that crosses the boundary returns the answer to a question. It does not
return a buffer that Python must decode, and it does not return a handle that
Python must walk to reach a value.

The rule binds the direction of travel, not the volume. One column of results
is one answer and it crosses once. A thousand single-entity reads carry less
data and break this record.

### D2. The number of crossings does not grow with the number of entities

A command that acts on a set crosses once. Doubling the size of the set does
not double the number of calls.

This is the checkable form of D1. A reviewer counts the crossings a caller
needs, and asks whether that count is a function of the population. If it is,
the boundary is wrong, whatever the caller wrote.

A verb over a mass-tier shape therefore has no per-entity form.[^6] The tier of
a shape says which populations this rule binds, and a separate record states how
the interface enforces it.[^7]

### D3. A verb accepts what the verb before it returns

The engine hands identities back in one shape. Every verb that takes identities
accepts that shape.

A boundary that returns one type and demands another has instructed the caller
to convert, and a conversion of a mass-tier result is the loop this record
forbids. The instruction arrives as a type error, so it looks like a small
defect at the call site. It is not.

The type declaration of the interface is part of the interface. A stub that
narrows what a verb accepts breaks this decision as surely as the code would.

### D4. A question the control plane cannot ask is a defect of the boundary

When a caller sweeps the world to find something, the engine is missing a read.
The repair adds the read. The repair does not add a rule, a warning or a note in
a guide.

The read that gets added answers the question for a set. A read that answers the
question for one tile, or for one unit, moves the sweep to a larger population
and satisfies this record in form only. The tile population is the larger one,
so a per-tile read is the worse repair.[^3]

This decision binds the reviewer as much as the author. A sweep in a test or in
an example is evidence of a missing read, and the review names the read.

## The alternatives this rejects

**Python holds the data and the project relies on discipline.** The engine
would expose every column, and a rule would tell each caller not to walk them.
The project rejects this because the rule already existed and already lost. The
person who wrote the sweep had written the rule.[^3] A rule with no mechanism is
a wish.

**Python goes away, and the control plane moves into Rust.** This removes the
boundary and the whole class of defect. The project rejects it because a
designer would then need a compile for every change of behaviour, which is the
one thing Python is here to provide.

**A per-entity verb stays as a convenience for a small world.** A caller with
few units would loop, and a caller with many would not. The project rejects
this for the reason the tier record gives: a rule that depends on the size of
the world produces a script that works in development and fails at scale, and
the failure appears far from its cause.[^8]

**The escape hatch is closed completely.** No array of values would ever reach
Python. The project rejects this because a caller that cannot get an answer out
builds a worse way to get it out. An array of column values is an answer. It is
not the world.

## Consequences

**Every new question needs a new read, and the engine pays for it.** The
project cannot answer a question by handing Python the data and letting Python
work it out. Each question the control plane must ask becomes work in Rust.
This is the cost the record accepts, and it is the reason the selector exists as
a destination rather than a set of named reads.[^9]

**A missing read now has a name.** A caller that sweeps is reported as a
boundary defect. Nobody has to argue about discipline.

**The interface cannot grow a per-entity verb over a mass shape.** A
contributor who wants one must supersede this record.

**The type stubs are load-bearing.** D3 makes the declared types part of the
constraint, so a stub that disagrees with the code is a breach and not an
inconvenience.

**A column read that exists today is not a breach, and it is not a licence.**
The engine returns tile values and event fields as columns. Each is one
crossing and one answer.[^10] A future read that returns the world so that
Python can search it is a breach, whatever it costs.

**This record states no cost.** No measurement exists on the target platform,
so every cost figure in this project is derived rather than measured.[^11] The
argument here is about the shape of the traffic and not about its price.

**Nothing enforces this record today.** The rule is prose in the orientation
and in the package, and no check fails when a caller breaks it. Two records
carry the enforcement, and neither is built.[^7] [^9] Until one of them is, a
reader must treat this constraint as unenforced. A record that claims an
enforcement nothing performs is the defect this project keeps catching.[^12]

## References

[^1]: ADR-0001, one binary gives one answer at any thread count, decision D1. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^2]: Budgets and costs, the scale constants. `docs/reference/budgets.md`
[^3]: Findings register, FND-147. `docs/FINDINGS.md`
[^4]: PRD-0007, the world holds things worth taking. `docs/product/accepted/prd-0007-the-world-holds-things-worth-taking.md`
[^5]: Decisions register, DEC-063. `docs/DECISIONS.md`
[^6]: ADR-0054, an entity belongs to one of three tiers, declared at creation, decision D1. `docs/adrs/accepted/adr-0054-an-entity-belongs-to-one-of-three-tiers-declared-at-creation.md`
[^7]: ADR-0043, a declared tier enforces the no-loop rule, and the API refuses the loop. `docs/adrs/draft/adr-0043-a-declared-tier-enforces-the-no-loop-rule.md`
[^8]: ADR-0054, an entity belongs to one of three tiers, declared at creation, decision D2. `docs/adrs/accepted/adr-0054-an-entity-belongs-to-one-of-three-tiers-declared-at-creation.md`
[^9]: ADR-0051, a selector is a lazy expression tree that Rust evaluates. `docs/adrs/draft/adr-0051-a-selector-is-a-lazy-expression-tree.md`
[^10]: Decisions register, DEC-060. `docs/DECISIONS.md`
[^11]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^12]: Recurring Defect Shapes, shape 3. `.claude/rules/recurring-defects.md`
