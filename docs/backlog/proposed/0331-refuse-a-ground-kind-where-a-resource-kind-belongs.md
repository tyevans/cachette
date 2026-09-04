---
id: 0331
title: Refuse a ground kind where a resource kind belongs
status: proposed
created: 2026-09-03
implements: []
changes: []
creates: []
serves: [PRD-0021]
blocked-by: [DEC-120]
---

## Why

**Two integer scales share their low numbers, and the guard cannot tell them
apart.** The resource kinds are food, wood and stone, numbered 0, 1 and 2. The
ground kinds are water, plain, forest, hill and mountain, numbered 0 to 4. The
tile report returns a ground kind. The gather verb takes a resource kind. A
caller who moves a number from the first to the second passes a legal number, is
not refused, and orders every soldier in the set to gather the wrong
resource.[^1]

**The wrong answer repeats exactly, so no test the project owns can see it.**
The two determinism tests compare a run against a run, and both runs hold the
same mistake. The testing rule names this shape, and this is the shape reaching
the public interface.[^2]

**The interface is inconsistent with itself.** An entity crosses to Python as
one opaque identity that Python cannot take apart or build. The project made
that decision for an identity and did not make it for a kind.

A decision register row holds the options and the recommendation.[^3] It must
close before this item is refined, because it decides what the work is.

## References

[^1]: Findings register, FND-342. `docs/FINDINGS.md`
[^2]: Testing Rules, section 2. `.claude/rules/testing.md`
[^3]: Decisions register, DEC-120. `docs/DECISIONS.md`
