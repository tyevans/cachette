---
id: 0222
title: Raise the error types the hierarchy declares, or stop declaring them
status: proposed
created: 2026-09-02
implements: []
changes: []
creates: []
serves: []
blocked-by: []
---

## Why

**The bindings declare exception types that nothing raises.** The error
hierarchy holds a root and seven leaves. Four leaves have a raise site. Three
do not: a selector error, a determinism error, and a panic that reached the
boundary.

A caller reading the exported names concludes that the engine reports a
determinism defect and converts a panic. It does neither. Nothing installs a
panic hook, nothing catches an unwind at the boundary, and no determinism check
reports through the interface.

The type list passes its own test, because the test asserts that each type sits
under the root.[^1] That test proves the hierarchy and proves nothing about
whether the engine produces any of it. This is the inert capability shape, and
the project has a rule against declaring one.[^2]

The record for typed errors states the gap in its own consequences rather than
claiming the capability.[^3] Stating it is not closing it.

## What is missing before this is refined

- The impact review.
- Which of the three should get a raise site now and which should be removed. A
  selector error has no selector to fail, so removing it may be right until the
  selector exists.
- Whether the panic conversion is a gap or is already provided by the binding
  macros, and what the release profile must be for it to work.
- What test drives the real caller for each one, rather than constructing the
  exception and asserting on it.

## Done when

Stated when the item is refined.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: The public interface tests. `tests/test_public_api.py`
[^2]: Recurring Defect Shapes, shape 3. `.claude/rules/recurring-defects.md`
[^3]: ADR-0046, every error is typed, the consequences. `docs/adrs/draft/adr-0046-every-error-is-typed.md`
