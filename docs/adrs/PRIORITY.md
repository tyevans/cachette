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
| 0064 | A unit chooses by scoring a small fixed option set. Backlog item 0064 wrote it, and the code it describes exists. |
| 0074 | A tile occupancy count is stored densely, and a spawn refuses a full tile. **Nothing implements it yet.** Backlog item 0095 is the work. |

## Reserved and not written

A reserved row is a number, not a promise. **Do not write one because the row
exists.** Apply the three-condition test first; most reserved rows should stay
reserved until the code that needs them exists.[^6]

| No. | Why it sits here |
|---|---|
| 0021 | Layout follows the access pattern. Write it with the descent columns, not before.[^7] |
| 0069 | Weather. Nothing needs it. |
| 0077 | The golden state hash. Write it when the first real golden file is committed, not before.[^9] |
| 0055 | The modifier pipeline. One source modifies a rate, so the test fails today.[^8] |

Every other reserved row in the registry is left alone on purpose. A record for
a subsystem nobody has built is the failure the scope rule opens with.[^6]

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
