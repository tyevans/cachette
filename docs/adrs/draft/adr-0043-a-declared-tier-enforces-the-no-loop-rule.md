# ADR-0043: A declared tier enforces the no-loop rule, and the API refuses the loop

## Context

The control plane is Python. Python sends an instruction and receives an
answer. Python does not walk the population.[^1]

That rule needs two things to be real. It needs to say which populations it
binds, and it needs something that stops a caller who breaks it.

The first part exists. Every entity shape declares one of three tiers at the
type. A mass tier holds more entities than a caller can walk. A character tier
holds a bounded population that a caller may walk. A singleton tier holds one
thing.[^2] The soldier shape declares the mass tier, and the declaration carries
the reason: a soldier is one of a million, so no caller walks the population.

The second part does not exist. The tier is a constant in the core crate. The
core crate checks it once, when it builds the storage of a shape. Nothing at
the Python boundary reads the tier, and no Python type knows that a soldier
belongs to the mass tier.

A finding records the result.[^3] A test spawned a unit on every open tile of a
small world and ordered each one to gather. The author of that test had recorded
the rule against sweeping in the same change. The same finding names this
reserved record by number, and states that nothing implements it, so a reader
who meets the rule and not the gap concludes the project enforces something it
does not.

The project owner closed the shape of the repair. A verb takes a set, so there
is no per-unit form for a caller to repeat.[^4] That removes the easiest loop.
It does not remove the loop a caller writes around a result.

This record states what the enforcement is. It does not build it.

## Decision

### D1. The tier of a shape decides the shape of its control plane interface

The tier is the single declaration site, and the interface derives from it.[^2]

A mass-tier shape gets a set-valued interface. Every verb over it takes a set
and returns a set. No verb over it names one entity, and no read over it walks
the population.

A character-tier shape gets both forms. A caller may name one character, and a
caller may walk the population.

A singleton shape gets the singular form only.

The tier is not restated at the boundary. A second declaration that says which
Python calls exist would be a second copy of one fact, and nothing would fail
when the copies disagreed.[^5] The boundary derives its surface from the tier
the shape already declares.

### D2. The refusal happens where the loop is written, and it names the correct method

A caller who writes a loop over a mass-tier set meets an error at that line.
The error is not a warning, and it is not slow behaviour that a profiler finds
later.

The four operations that Python uses to start a loop each raise. Truth testing,
length, iteration and indexing all refuse. Each message names the method that
answers the question the caller was really asking: a count, an emptiness test, a
column read, or a verb applied to the whole set.[^6]

A refusal that does not name the alternative recreates the failure this record
exists to prevent. The finding shows that a caller who is told no, and is not
told what to do instead, writes the sweep.[^3]

### D3. The refusal is a property of the type, never a check on a count

Nothing counts the population and then decides whether to refuse.

A check on the count makes one script work against a small world and fail
against a large one. The author develops against a few entities and ships
against many. The failure then appears far from its cause, and it appears only
at scale.[^7]

A refusal that comes from the declared tier fails on the first attempt, in
development, and no world size makes it succeed.

### D4. The escape hatch returns columns, and it never returns entities

A caller can get values out. The values come out as one column for each field,
in one crossing.

A caller who then writes a Python loop loops over arrays of numbers. That
caller has left the set, so the engine no longer knows which entities the caller
holds, and a verb cannot take the result back as a set.

The engine offers no operation that yields entities one at a time from a
mass-tier set. It offers no per-element callback, no mapping operation and no
chunked iterator. Each of those is a loop with a different name.

An identity that does cross stays whole, so a caller cannot assemble one from a
number it chose.[^8]

### D5. This constraint is unenforced until the interface refuses, and the project says so

No Python type in this project refuses a loop today. The tier reaches no code
outside the core crate.

Until the refusal exists, every document that describes this boundary states
the rule as a rule and not as an enforcement. A record that claims a capability
nothing invokes is the defect shape this project keeps meeting, and it costs
more than silence, because a reader stops looking.[^9]

This decision is what a reviewer checks first. It fails the moment a document
says the interface refuses the loop while nothing refuses it.

## The alternatives this rejects

**A run-time check on the population size.** The engine would count the
entities and refuse a call over a large set. The tier record rejects this
already, and D3 states the reason.[^7]

**Documentation alone.** The orientation states the rule, and the Python
package repeats it in its own text. Both statements are correct, and the finding
shows a sweep written by the person who wrote one of them.[^3] The project
rejects this because it has already been tried and measured against a real
caller.

**A lint over the calling code.** A lint would search a script for a loop over a
selector. The project rejects this for two reasons. A lint sees the shape it
knows, so a caller who converts to a list first passes it. A lint also runs in
this repository, and a caller who uses this engine runs their own tooling.

**Refusing to return values at all.** The interface would answer questions and
never hand out an array. The project rejects this because a caller who cannot
get an answer builds a worse route to it. D4 keeps the hatch and makes it
columnar.

## Consequences

**A shape cannot change tier to make a script work.** Widening a tier keeps
every script that already runs. Narrowing one does not, so a shape declares the
stricter tier its population admits.[^2]

**The error messages are part of the interface.** D2 makes the text of a
refusal a thing a test asserts, because a refusal that names no alternative
fails this record.

**A verb over a mass shape must have a whole-set algorithm, or must admit that
it does not.** The set form is what permits a cheaper algorithm. The owner
recorded that the spawn verb is set-valued at the boundary and is still a loop
inside the engine, so the principle is satisfied in form and not yet in
substance.[^4] This record does not close that gap.

**The interface needs a set type before it can refuse anything.** There is
nothing to raise from today, because no selector type exists. The record that
states what that type is comes with this one.[^10]

**A character-tier caller may still write a slow script.** The record does not
protect a caller from a bounded population. It protects the world from an
unbounded one.

**This record states no cost.** One blocker holds every cost figure this record
would state, and it says which figures are measured.[^11]

## References

[^1]: ADR-0040, Python is a control plane, not a data plane, decision D2. `docs/adrs/draft/adr-0040-python-is-a-control-plane-not-a-data-plane.md`
[^2]: ADR-0054, an entity belongs to one of three tiers, declared at creation, decision D1. `docs/adrs/accepted/adr-0054-an-entity-belongs-to-one-of-three-tiers-declared-at-creation.md`
[^3]: Findings register, FND-147. `docs/FINDINGS.md`
[^4]: Decisions register, DEC-063. `docs/DECISIONS.md`
[^5]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
[^6]: Selector engine and verbs, section 1.4. `docs/research/reports/04-selector-engine-and-verbs.md`
[^7]: ADR-0054, an entity belongs to one of three tiers, declared at creation, decision D2. `docs/adrs/accepted/adr-0054-an-entity-belongs-to-one-of-three-tiers-declared-at-creation.md`
[^8]: ADR-0085, an entity crosses to Python as one opaque identity that the engine resolves, decision D2. `docs/adrs/accepted/adr-0085-an-entity-crosses-to-python-as-one-opaque-identity.md`
[^9]: Recurring Defect Shapes, shape 3. `.claude/rules/recurring-defects.md`
[^10]: ADR-0051, a selector is a lazy expression tree that Rust evaluates. `docs/adrs/accepted/adr-0051-a-selector-is-a-lazy-expression-tree.md`
[^11]: Blockers register, BLK-007. `docs/BLOCKERS.md`
