# ADR-0052: A selector result may be a range, not only an enumerated set

## Context

A selector describes a set. Building one reads nothing, and the whole
description crosses the boundary once.[^1] The engine then evaluates it.

Evaluation produces a result, and the result must be held somewhere. The
obvious form is a list of the members. It is simple, it is easy to test, and
every caller already understands it.

It is also the form that undoes the selector. A list costs one index for every
member, so the cost of holding an answer follows the size of the answer. The
target scale sets how bad that is, and the scale constants table holds the
figures.[^2]

The predicates a caller will write make this concrete. A caller asks for
passable ground, for ground nobody holds, or for ground of one kind. Each of
those matches a large part of the world. Under a list, the engine builds an
index for every match, holds it, and does so again for every intermediate node
of the expression. The description was free and the answer was not.

The world already has a structure that a result can use. Tiles are stored in
blocks at the aggregation block size, and the summary pyramid divides the world
into the same blocks.[^3] [^4] A descent through the pyramid already produces
its answer block by block, and a cost model decides when to descend and when to
fall back to a flat pass.[^5]

The project has taken this shape once already, in a smaller place. A set of
factions is one mask, and no field of the world is indexed by a faction.[^6]

The research examined the candidates and reached a recommendation with a stated
caveat: no single structure is best for all data, and the claim that a
purpose-built form beats a general library here is a design argument and not a
measurement.[^7] [^8]

## Decision

### D1. The result of a selector is a set representation, and enumeration is one form of it

The engine defines the result of an evaluation as a value that answers two
questions. It says whether a given member is in the set, and it yields the
members in order.

A list of indices satisfies that definition. So does a bitmask. So does a range.
So does a statement that every member of a block belongs. The engine chooses
among them, and no part of the interface requires one.

A caller cannot depend on the internal form, and no verb signature names it.

### D2. The representation states that a whole block matches, in one entry

The world is stored in blocks, and the summary pyramid uses the same blocks.[^3]
[^4] The result representation carries a form that says every member of a named
block belongs to the set.

**This is the decision that makes the selector affordable.** A predicate that a
whole block satisfies costs one entry, and the entry does not grow when the
block holds more members. The cost of the answer then follows the boundary of
the answer rather than its area.

The case is common rather than exotic. A descent prunes a summary cell in or
out as a whole, so a level of the pyramid that answers a predicate outright
produces exactly this form.[^5] A predicate on a faction or on a terrain kind
produces it over any block that is uniform in that field.

The gain is not universal, and this record does not claim that it is. A result
of many isolated members that fall in different blocks costs at least what a
list costs. The representation is chosen so that the common shapes are cheap,
not so that every shape is.

### D3. The representation follows the storage layout, and iteration follows memory order

The blocks of the result are the blocks of the storage. The engine does not map
one space onto another.

A verb that consumes the result therefore walks the columns in the order they
sit in memory, and reads a run of values rather than gathering scattered ones.
A list of indices loses that property, because nothing in a list keeps the
members in the order the columns hold them.[^9]

A representation whose divisions were arbitrary would need a conversion step
between the descent that produced the answer and the verb that consumes it. The
project refuses that step.

### D4. An enumerated array exists at the boundary only, and never as the working form

A caller may ask for the members as a flat array, and the engine produces one.
That array is a crossing and an answer, and it obeys the boundary rule.[^10]

The engine never uses that array to hold an intermediate result, and it never
produces one to pass a set from one internal stage to the next. An intermediate
enumeration would pay the cost this record exists to avoid, and it would pay it
where no caller asked for it.

### D5. Iteration over a result yields one fixed order, whatever the representation

Two representations of one set yield the members in the same order.

The order comes from a stable key over the addresses or the identities. It never
comes from the order a thread finished in, and it never comes from the order the
engine happened to fill a container in.[^11]

This makes the representation an implementation choice rather than a
determinism risk. The engine may change the form it picks for a set without
changing any answer, and a test that compares two runs cannot see the
difference.

### D6. The record names the representation as a parameter, and states no measured value

Which concrete form the engine uses, and where each form gives way to the next,
are open. The research recommends a two-level form built for this storage
rather than a general compressed bitmap library, and it records that the
recommendation is a design argument.[^8]

One blocker holds every cost figure this decision rests on, and it says which
figures are measured.[^12] No measurement separates the forms below. This record
therefore states the property the representation must have, and it states no
threshold, no density and no size. A decision register entry carries the open choice with its options
and the recommendation.[^13]

## The alternatives this rejects

**A sorted list of indices, always.** It is the simplest form and it makes
every test easy to read. The project rejects it as the only form, because it
makes the cost of an answer follow the number of members. A predicate that most
of the world satisfies then costs the world, at every node of the expression,
and the laziness of the selector buys nothing.

**A dense bitmask over the whole key space, always.** Set algebra becomes
branch-free word work, which is the fastest option when the answer is dense. The
project rejects it as the only form, because the engine must scan the empty
words of a sparse answer, and because one mask for every intermediate node of a
large expression is a cost the caller did not ask for.[^7]

**A general compressed bitmap library on the evaluation path.** A mature library
would arrive tested and would need no work. The project rejects it for the
evaluation path, because its division of the key space is arbitrary and this
project's key space is not: the storage blocks and the pyramid blocks already
provide the division, and D3 depends on that agreement. A general library
remains the right answer for a cold, sparse side table, where the key space
really is arbitrary.[^8]

**Making the representation part of the interface.** A caller could choose the
form and tune it. The project rejects this because it would fix an
implementation detail into every script, and because it would give a caller a
lever over a decision the caller cannot measure.

## Consequences

**The selector is affordable at the target scale, on the strength of an
argument and not a measurement.** D2 is the whole cost case for the expression
tree, and it rests on the structure of the storage rather than on a benchmark.
A benchmark on the target platform can still overturn the choice of form. It
cannot overturn the requirement that a whole block cost one entry.[^12]

**A scattered answer is not made cheap.** A predicate that matches a few members
in each of many blocks costs about what a list costs. The record states this
rather than hiding it, because a reader who believes otherwise will design a
predicate vocabulary that cannot deliver.

**Every set operation must handle every form.** Union, intersection and negation
work across a whole-block entry and a partial one. This is more code than a list
needs, and it is where the defects will be.

**Negation is expensive and needs a bound.** The complement of a small set is
close to the whole world. The engine bounds a negation by the domain the
selector already restricts, and the interface has no free-standing complement.

**A test asserts the form as well as the answer.** A test that only compares
members passes when the engine enumerates everything, so it measures nothing
about this record. A test asserts that a predicate satisfied by a whole block
produces one entry for that block. Put the enumerating implementation back and
watch such a test stay green, because that is the only proof the test reaches
the case.[^14]

**The result representation is one fact, declared once.** The descent writes its
answer in the shape the verbs consume. Nothing converts, so nothing can hold a
second copy that disagrees.[^15]

**Nothing implements this record.** No selector exists, and no result
representation exists. This record binds the work that builds them.

## References

[^1]: ADR-0051, a selector is a lazy expression tree that Rust evaluates, decision D1. `docs/adrs/accepted/adr-0051-a-selector-is-a-lazy-expression-tree.md`
[^2]: Budgets and costs, the scale constants. `docs/reference/budgets.md`
[^3]: ADR-0016, tiles are stored in block-tiled order at the aggregation block size, a reserved number with no record. `docs/adrs/REGISTRY.md`
[^4]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D2. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
[^5]: ADR-0028, descent has a cost model and a flat fallback, a reserved number with no record. `docs/adrs/REGISTRY.md`
[^6]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D3. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
[^7]: Selector engine and verbs, section 3.1. `docs/research/reports/04-selector-engine-and-verbs.md`
[^8]: Selector engine and verbs, section 3.2. `docs/research/reports/04-selector-engine-and-verbs.md`
[^9]: ADR-0012, tiles are dense columns and units are a generational arena, decision D2. `docs/adrs/accepted/adr-0012-tiles-are-dense-columns-and-units-are-a-generational-arena.md`
[^10]: ADR-0040, Python is a control plane, not a data plane, decision D1. `docs/adrs/draft/adr-0040-python-is-a-control-plane-not-a-data-plane.md`
[^11]: ADR-0004, iteration order is explicit, decision D4. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^12]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^13]: Decisions register, DEC-069. `docs/DECISIONS.md`
[^14]: Testing rules, section 2a. `.claude/rules/testing.md`
[^15]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
