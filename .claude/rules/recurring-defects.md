# Recurring Defect Shapes

This rule lists the mistake shapes to check for when you write, review, or
diagnose work in this project.

**Read the provenance before you trust a shape.** Cachette has a foundation
crate and almost no defect history. Two shapes below have local evidence from this project. Three
are imported from two sibling projects, where they were derived from an audit of
about 130 correction commits.[^1] [^2] An imported shape is a prior. It stays a
prior until someone records a local instance.

Add a local instance under each shape as you find one. A shape with local
evidence is worth more than a shape without it.

**The through-line:** most of these shapes are one fact stored in more than one
place, with nothing that fails when the copies disagree.

---

## 1. Redundant declaration sites with undocumented precedence

**Provenance: local evidence, and imported.**

One value is declared in two places. Both look authoritative. Only one reaches
anything. The other is read back correctly and changes nothing, so the failure
is silent.

Local instance. The registry records three record-number collisions during the
research phase, because agents chose their own numbers instead of taking one
from the registry.[^3] Three declaration sites for one number.

**Rule.** Declare a value once. When a second site must exist, add a check that
fails when the copies disagree. Do not add a comment that names the winner. A
comment that explains which copy loses is evidence that the second copy should
not exist.

This project has several places where the shape will recur: a constant in Rust
and the same constant in the Python control plane, an event layout and the
Python type that decodes it, and the registry against the record files.

## 2. Documents that rot when a sweep names specifics

**Provenance: local risk, and imported with strong evidence.**

A document names a count, a file list, or a measured figure. The next change to
the tree makes it false. Nothing fails.

Imported evidence. One reference project traced eleven correction commits whose
only purpose was to repair the specifics that a previous correction named. Five
sweeps in a survey of 823 commits needed a second sweep.[^2]

**Rule.** A decision record holds no count, no file table, and no measured
figure.[^4] Put them in the commit message.[^5]

**Rule.** A sweep is not done when the files look right. It is done when a
whole-tree search for the name comes back clean, and the search command is in
the commit body.

**Rule.** When two listings must agree, write a check that derives one from the
tree and compares. Do not sweep by hand.

## 3. Inert code that nothing invokes

**Provenance: imported.**

The project declares a capability, documents it, and never calls it. Its own
test passes, because the test constructs the mechanism and drives it directly.

Imported evidence. One reference project shipped nine inert capabilities in one
wave. It later wrote a record about a list of telemetry keys that nothing
emitted, after an earlier record had claimed the list described real
behaviour.[^1]

**Rule.** Ask who is obligated to invoke this: the user, or the engine. If the
engine, the test must start at the engine.[^6]

**Rule.** Do not declare a capability before something calls it.

## 4. Nondeterminism that the tests cannot see

**Provenance: local, from the project constraints. No instance yet.**

This shape has no reference evidence, because neither sibling project has
determinism as a constraint. It is listed because it is the shape this project
cannot survive.

Nondeterminism enters through paths that look harmless at review time:

- A hash map iteration order used to drive an update.
- A thread completion order or a work-stealing order used to order a result.
- A convergence test or a time budget that ends a solver.
- Undeclared padding in an event type, which puts uninitialised bytes into a
  state hash.
- Thread-local random state.
- A floating point sum over an aggregate, which is not associative.

Each of these passes a single-threaded test on one machine. The determinism
record states the rules that forbid them.[^7]

**Rule.** When you review a change, look for the ordering. Ask what fixes the
order of every parallel result. If the answer is a sort by a stable key, the
change is sound. If there is no answer, the change is a defect.

**Rule.** A determinism test must run at more than one thread count. A test that
runs once proves nothing.[^6]

## 5. A record that no longer describes the code

**Provenance: imported.**

The implementation moves. The record stays. A later contributor reads the record
and makes a decision from something false.

Imported evidence. Two reference records were superseded within one and two days
of creation. Both had recorded a module arrangement rather than a
constraint.[^8]

**Rule.** A record the code contradicts is worse than no record, because it
lies.[^9] When the implementation and the record disagree, one of them changes
before the work is done.

**Rule.** Record the constraint, not the arrangement.[^4]

## Quick checklist

Before you claim work is done, check each line.

1. Is any value declared twice? Does a check fail when the copies disagree?
2. Does any document name a count, a file list, or a figure that this change
   made false?
3. Does anything call the code you added? Does a test drive the real caller?
4. What fixes the order of every parallel result?
5. Does any record now disagree with the code?

## References

[^1]: The `eventsource-py` repository, `.claude/rules/recurring-defects.md`, derived from about 130 correction commits.
[^2]: The `redstring` repository, `.claude/rules/recurring-defects.md`.
[^3]: ADR Registry. `docs/adrs/REGISTRY.md`
[^4]: Decision Record Scope. `.claude/rules/adr-scope.md`
[^5]: Commit Message Rules. `.claude/rules/commits.md`
[^6]: Testing Rules. `.claude/rules/testing.md`
[^7]: ADR-0001, one binary gives one answer at any thread count. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^8]: Findings, the scope of a decision record. `docs/research/adr-scope-findings.md`
[^9]: Definition of Done. `.claude/rules/definition-of-done.md`
