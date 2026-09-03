# ADR-0041: A crate split enforces the boundary at compile time

## Context

The simulation is Rust. The control plane is Python.[^1] The engine must give
one answer at any thread count, byte for byte, and determinism is the one
property this project cannot recover once it is lost.[^2]

Python code inside a simulation step destroys that property. The interpreter
reorders nothing on its own, but a callback allocates, takes and releases the
interpreter, can raise, and can run arbitrary code that reads a clock or a
thread identity. The defect does not appear at the call site. It appears as a
state hash that differs a thousand frames later, and finding it means bisecting
the whole frame.

The rule against it is easy to write down and impossible to keep by hand. It is
one rule that a reviewer must apply to every change, forever, and one miss is
enough. This project has already recorded what a rule with no mechanism is
worth: the person who wrote the rule against sweeping the world from the
control plane wrote a sweep in the same change.[^3]

The storage code is the second force. It uses unsafe Rust by necessity, and
Miri is the only tool that finds an aliasing or a provenance defect in it. Miri
cannot run the interpreter, so it cannot run any crate that links the
interpreter binding.

## Decision

### D1. The simulation crate has no dependency on the interpreter binding

The simulation lives in a crate that does not depend on the interpreter binding
library, at all, in any dependency kind. The bindings live in a second crate
that depends on the first.

The dependency runs one way and never the other.

### D2. The split is what makes a mid-step callback impossible, not a rule

No type in the simulation crate can name a Python object, and no function in it
can take an interpreter token, because neither type exists in its dependency
graph.

A contributor who tries to call Python from a system does not get a review
comment. They get a compile error at the line they wrote it.

This is the whole point of the decision. A convention costs a reviewer's
attention on every change and fails silently when the attention lapses. A
missing dependency costs nothing and fails loudly on the first attempt.

### D3. A check proves the split rather than asserting it

The build reads the resolved dependency tree of the simulation crate and fails
when the interpreter binding appears in it. It reads the resolved tree and not
the manifest, so a binding pulled in through another dependency fails the same
way a direct one does.[^4]

A check that read the manifest would pass on the case that is hardest to see.

### D4. A verb is Rust, and the control plane cannot supply one

A user cannot register behaviour written in Python that the engine calls during
a step. Such behaviour would have to run inside the step, which D2 makes
impossible.

A new verb is Rust. The engine holds the verbs behind a trait rather than a
fixed match, so a verb may arrive from another crate, and that crate is a Rust
crate.

## The alternatives this rejects

**One crate, with a rule.** The simulation and the bindings live together and a
rule forbids calling Python from a system. The project rejects this because the
rule is the thing that fails, and because it gives up Miri over the storage
code, which no test replaces.

**One crate, with a lint.** A lint would look for the interpreter types inside
the simulation modules. The project rejects it because a lint sees the shape it
knows: a type alias, a generic parameter or a trait object routes around it,
and the failure is again silent.

**A feature flag that removes the binding from the simulation crate.** The
crate would depend on the interpreter binding behind a flag, and the flag would
be off for the simulation build. The project rejects this because feature
unification turns one dependent that enables the flag into a build where the
simulation links the interpreter, and nothing reports it.

**Splitting later, when the boundary hurts.** The project rejects this because
the retrofit is an audit of every function signature in the simulation, and
because the defect the split prevents has already been paid for by then.

## Consequences

**Every type both sides need is declared in the simulation crate and wrapped in
the bindings crate.** The wrapper is real, ongoing work, and it is the price of
the boundary. A field added to a shape that Python reads is two changes.

**Miri runs over the storage code.** This is the benefit that no test gives,
and it exists only because the simulation crate links no interpreter.

**The engine cannot ever call Python during a step.** Any feature that wants a
per-entity callback inside a frame is out of scope while this record stands. A
contributor who wants one must supersede this record, not work around it.

**A determinism hazard moved from review to the compiler.** The reviewer no
longer has to look for it, which means the reviewer will stop looking for it.
That is correct here and it would be wrong for any rule the compiler does not
hold.

**The dependency check is a gate and must stay one.** D3 is the part of this
record that can decay: the split is enforced by an absence, and an absence is
easy to fill by accident. If the check is removed, this record states something
nothing enforces.

## References

[^1]: ADR-0040, Python is a control plane, not a data plane, decision D1. `docs/adrs/draft/adr-0040-python-is-a-control-plane-not-a-data-plane.md`
[^2]: ADR-0001, one binary gives one answer at any thread count, decisions D1 and D3. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^3]: Findings register, FND-147. `docs/FINDINGS.md`
[^4]: The crate split check. `scripts/check-crate-split.sh`
