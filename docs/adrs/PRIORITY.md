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
sand, and every draft below is already implemented in the crate.

**Every draft and every reserved row appears exactly once.** A check enforces
it, so a written record cannot wait for review unseen.[^4]

The second table below is a **selection, not a set.** The registry holds about
forty proposed rows, and the scope rule says most of them should never be
written. A check that demanded all of them here would argue against that
rule.

## Waiting for review

These are written. A reviewer reads the record against the code, decision by
decision, and accepts or returns it.[^5]

| No. | Why it sits here |
|---|---|
| 0068 | Terrain. Every other subsystem now reads it, and it is still a draft. |
| 0022 | Level 0 is the only truth. The pyramid, the holding and the rates all cite it. |
| 0023 | An aggregate combines exactly. Cited by every summary field. |
| 0024 | Extensive or intensive. Cited by the level 1 fields. |
| 0053 | A faction is a bit in a mask. Written and implemented today. |
| 0054 | Three entity tiers. The ruler and the family depend on it. |
| 0062 | Production and upkeep as rates. Written and implemented today. |
| 0075 | The founding reads a bounded sample. Written and implemented today. |
| 0071 | The bridge rebuild orders on one thread. |
| 0072 | Allocated by the resource work. |
| 0073 | Allocated by the resource work. |
| 0008 | The primary target is aarch64. Old, and it governs every cost claim. |
| 0009 | Parallel stages write disjoint outputs. |
| 0011 | Every value type is a newtype. |

## Reserved and not written

A reserved row is a number, not a promise. **Do not write one because the row
exists.** Apply the three-condition test first; most reserved rows should stay
reserved until the code that needs them exists.[^6]

| No. | Why it sits here |
|---|---|
| 0064 | A unit chooses by scoring an option set. The next item written will write it. |
| 0063 | A need is a rate with a threshold. Wanted by consumption. |
| 0021 | Layout follows the access pattern. Write it with the descent columns, not before.[^7] |
| 0069 | Weather. Nothing needs it. |
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
[^8]: ADR-0062, decision D7, a draft record. `docs/adrs/draft/adr-0062-production-and-upkeep-are-rates-attached-to-a-site.md`
