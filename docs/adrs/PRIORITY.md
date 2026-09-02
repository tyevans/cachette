# Decision Record Priority (Index)

This document is an **index**. It states which records need attention next, and
why each sits where it does.

**It holds no title and no status.** The registry holds those, and it is the
only place that holds them. A second copy here would go stale, and a stale
status is the failure this project has already recorded twice.[^1] [^2]

Rule 4 of the documentation rule exempts a table that lists files as data.[^3]

## The two queues

A record is either **written and waiting for review**, or **reserved and not
written**. They are different work and they have different owners.

**Review is the bottleneck.** An author may set `Draft`. Only a reviewer may
set `Accepted`.[^1] A draft binds nothing, so work built on one is built on
sand.

**A draft below is not always implemented.** Most describe code that exists.
A record that runs ahead of the code says so in its own row, because a
reviewer must know whether the claim was tested or only argued.[^5]

**Every draft and every reserved row appears exactly once.** A check enforces
it, so a written record cannot wait for review unseen.[^4] When no record is
waiting, the first table is empty and says so.

The second table below is a **selection, not a set.** The registry holds about
forty proposed rows, and the scope rule says most of them should never be
written. A check that demanded all of them here would argue against that
rule.

## Waiting for review

A record here binds nothing until a reviewer moves it.

| No. | What it claims |
|---|---|
| 0081 | Housing. A review returned it: the engine already holds the per-site count that its decision D3 asks the project to store.[^12] An open row holds the choice that follows.[^13] It reads against ADR-0074 D3 correctly, and that objection failed. |
| 0082 | Population growth. A review found its own decisions sound and returned it, because it rests on the free places of 0081.[^12] It moves when 0081 moves. |
| 0083 | The gate build profile. A review returned it: both decisions hold against the code, and three sentences do not.[^15] It moves when those three are corrected. |
| 0084 | The world reserves the unit columns at construction. The reservation is what a later contributor would trade away for a spawn that never refuses, and the code cannot say why it must not.[^14] A review returned it: all four decisions hold against the code, and two sentences do not.[^16] It moves when those two are corrected. |
| 0090 | A tile upgrade is stored sparsely. A dense array over the tiles is what a future contributor would reach for, and the code shows the sparse map without showing why the dense one was refused. The implementation exists and every decision was checked against it. |

## Reserved and not written

A reserved row is a number, not a promise. **Do not write one because the row
exists.** Apply the three-condition test first; most reserved rows should stay
reserved until the code that needs them exists.[^6]

| No. | Why it sits here |
|---|---|
| 0051 | The selector as a lazy expression tree. The owner decided that a set-valued command is how the control plane reaches a set, so this governs every verb written after it. Write it before the next verb.[^12] |
| 0052 | A selector result may be a range. It carries the cost argument that makes 0051 affordable at the target scale, so the two are reviewed together.[^12] |
| 0040 | Python is a control plane. The orientation states it and nothing enforces it. It is the rule 0043 and 0051 rest on.[^12] |
| 0043 | A declared tier refuses the loop. Today the rule is prose, and prose lost to a missing read once already.[^13] |
| 0021 | Layout follows the access pattern. Write it with the descent columns, not before.[^7] |
| 0069 | Weather. Nothing needs it. |
| 0077 | The golden state hash. Write it when the first real golden file is committed, not before.[^9] |
| 0055 | The modifier pipeline. One source modifies a rate, so the test fails today.[^8] |

The four rows above moved here on 1 September 2026, when the project owner
decided that a set-valued command is how the control plane reaches a set and
that the selector tree is the destination the verbs are written toward.[^12]
They are written before the tree is built, because they govern the verbs
rather than describe them. **The author of these four must not review them.**

Every other reserved row in the registry is left alone on purpose. The scope
rule asks whether a constraint exists, not whether the code exists, and most
reserved rows name a topic rather than a constraint.[^6] A record may be
accepted before its code, and the registry says how: the acceptance states
plainly that nothing implements it yet.[^1]

## References

[^1]: ADR Registry, who reviews. `docs/adrs/REGISTRY.md`
[^2]: Findings register, FND-055. `docs/FINDINGS.md`
[^3]: Documentation Rules. `.claude/rules/documentation.md`
[^4]: The priority check script. `scripts/check_priority.py`
[^5]: Definition of Done, section 3. `.claude/rules/definition-of-done.md`
[^6]: Decision Record Scope, section 1. `.claude/rules/adr-scope.md`
[^7]: Decisions register, DEC-032. `docs/DECISIONS.md`
[^8]: ADR-0062, decision D7. `docs/adrs/accepted/adr-0062-production-and-upkeep-are-rates-attached-to-a-site.md`
[^9]: Decisions register, DEC-014. `docs/DECISIONS.md`
[^10]: Backlog item 0123. `docs/backlog/refined/0123-recover-a-depleted-deposit-without-a-pass-over-the-world.md`
[^11]: Backlog item 0123. `docs/backlog/complete/0123-recover-a-depleted-deposit-without-a-pass-over-the-world.md`
[^12]: Review 0143, the housing, growth, founding and recovery records. `docs/reviews/0143-the-housing-growth-founding-and-recovery-records.md`
[^13]: Decisions register, DEC-057. `docs/DECISIONS.md`
[^14]: Decisions register, DEC-059. `docs/DECISIONS.md`
[^15]: Review 0164, the gate build profile record. `docs/reviews/0164-the-gate-build-profile-record.md`
[^16]: Review 0175, the unit reservation record. `docs/reviews/0175-the-unit-reservation-record.md`
