---
id: 0071
title: Derive tile passability from tile capacity
status: proposed
created: 2026-08-31
serves: []
---

The terrain module states that passability is the capacity being zero and
that nothing else states it. The module then states it again. The kind's
passability test matches on the water kind by name, and it does not read the
capacity.

Every engine caller reads the passability test. No caller reads the capacity
to decide whether a unit may stand. The two agree today, because water is the
one kind with a capacity of zero.

A kind added with a capacity of zero and a name that is not water would be
passable and would admit nobody. Nothing fails, and no test compares the two.

The fix is small: return the capacity being greater than zero. That removes
the second site rather than reconciling it, which is what the recurring defect
rule asks for.[^1] The findings register holds the instance.[^2]

Refine this against the terrain record, which is still a draft.

## References

[^1]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
[^2]: Findings register, FND-060. `docs/FINDINGS.md`
