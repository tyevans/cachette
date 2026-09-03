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
| 0094 | The caller owns the camera and the pixels, and one command fills them. **No code implements it yet, and a spike of the verb is the work that follows it.** It inverts the drawing boundary, so a reviewer should test D2 hardest: it argues that a caller-supplied output buffer is neither a read of the world nor a write to it, and both halves of that must hold or the record is an exception dressed as a rule. D4 is the one a reviewer can check today, because it is a dependency list. The record refuses a split that was asked for, in D3, and a reviewer who disagrees with the refusal should say so rather than accept it silently. |

| 0052 | A selector result may be a range, not only an enumerated set. **A review returned it.** D1, D4, D5 and D6 hold, and D6 is the corpus's best example of naming a parameter rather than inventing a value. D2 and D3 rest on the tiles being stored in blocks, and they are stored row by row, so the cost case is conditional on a reserved row that has no record. FND-217 records it. It moves when the two present-tense claims are corrected. **Nothing implements it.** |
| 0041 | A crate split enforces the boundary at compile time. **The code implements it and a gate proves it**, by reading the resolved dependency tree of the core crate rather than its manifest. It is the strongest record in this group: it converts the rule against a mid-step Python callback into a compile error, and determinism is what that rule protects. A reviewer should test D3 hardest, because the record is enforced by an absence and an absence is easy to fill by accident. |
| 0042 | The interpreter is released for the whole step. **The code implements it.** Its D3 states the property the release buys, that a frame is a function of what was fixed before it began, and it carries the durable half of a reserved row that this review retired. A reviewer should ask whether D3 is really bought by D1 and D2, or whether the exclusive hold on the world is doing the work. |
| 0044 | What copies and what does not is declared at the call site. **The code implements it, and seven files cite the number.** Every read across the boundary copies today and says so. A reviewer should test D3, which forbids a method ever changing from copying to borrowing, because that is the change a later contributor makes for speed. |
| 0046 | Every error is typed. **The code implements it and the Python tests drive it.** Its consequences name four declared exception types that nothing raises, which is the inert-capability shape stated against the project's own code. A reviewer should decide whether naming them is enough or whether the record should refuse them. |
| 0047 | Many worlds live in one interpreter. **The code implements it and a test proves two worlds diverge.** Its D2 names the one mutable process-wide counter the engine holds, rather than claiming there is none. A reviewer should test that naming: either the counter is outside the rule or the rule is wrong. |
| 0031 | Events live in type-segregated arenas of plain data. **The code implements it and cites the number.** One enumeration holding every event kind is the obvious implementation, and nothing in the code says why it was refused. It also absorbs the rejection of classic event sourcing, whose reserved row this review retired. |
| 0032 | The log holds a fact no solver reproduces, never derived state. **The code implements it**, in that no solver writes an event. The record states plainly that nothing enforces it and that a reviewer is the only check. A reviewer should test D1: the line between a fact and a derived value is the whole record, and a system author has to draw it correctly the first time. |
| 0040 | Python is a control plane, not a data plane. It is the rule that 0041, 0042, 0043, 0044 and 0051 rest on, and nothing enforces it. **The author of this record must not review it.** |
| 0043 | A declared tier refuses the loop. Today the rule is prose, and prose lost to a missing read once already.[^13] **The author of this record must not review it.** |
| 0093 | The window shows what changes, and the record of a moment goes to the inspection path. **The code implements it and cites it.** The panel it replaced grew until it could not fit the window, and every addition that made it grow was correct on its own, so a future contributor adding a row will not see what stops them. The record buys the right to refuse that row. A reviewer should test D2 hardest: it puts every consulted quantity behind one hold, and the record itself names that layer as the next thing that will grow too large. |
| 0092 | The agent tool surface grows one tool at a time, against a stated need. It records a policy that existed in one sentence, in a completed backlog item, where nothing a worker reads before starting carried it.[^20] The code that implements it exists: the surface grew this round against seven gaps another worker named, and no tool was built that nobody asked for. A reviewer should test D2 hardest, because it is the decision that has to hold when somebody meets a gap in a hurry. |
| 0091 | Movement takes its direction from a per-cell field, and never from a per-unit search over the neighbouring cells. **It runs ahead of the code.** No engine pass derives the field yet, and the item that builds it is refined and not started.[^17] A reviewer must read it as an argument and not as a description. The search it forbids is the obvious implementation, and the reason the field gives the same answer is invisible inside that loop. It also fixes a tie-break order, which is a determinism property. |
| 0081 | Housing. Two reviews returned it: the engine already holds the per-site count that its decision D3 asks the project to store.[^12] [^18] An open row holds the choice that follows.[^13] It reads against ADR-0074 D3 correctly, and that objection failed. **It runs ahead of the code.** No settlement holds a housing capacity, and the birth that its D3 names as a frequent caller does not exist either. |
| 0082 | Population growth. A review found its own decisions sound and returned it, because it rests on the free places of 0081.[^12] It moves when 0081 moves. |
| 0083 | The gate build profile. A review returned it: both decisions hold against the code, and three sentences do not.[^15] It moves when those three are corrected. |
| 0078 | Descent is a bounded record, and a relation is a bounded recursion. **A review found every decision sound against the code and gave it an accept.** The exactness argument is held by a compile-time assertion rather than by a comment, both figures it refuses to state are in the reference table with their derivations, and the tests drive the relation through the world rather than through the module. **The verdict is in and the status is not.** Accepting it moves the file, and all six of its path citations sit in the descent and character modules, which document work may not touch. DEC-092 holds the one gap the review found. |
| 0084 | The world reserves the unit columns at construction. The reservation is what a later contributor would trade away for a spawn that never refuses, and the code cannot say why it must not.[^14] A review returned it: all four decisions hold against the code, and two sentences do not.[^16] It moves when those two are corrected. |
| 0060 | The storage shape of an influence field. **A review found every decision sound against the code and gave it an accept.** D1's read-only clause is held by the compiler rather than by discipline, and the record states no figure anywhere, including the cell width that has already changed once. **The verdict is in and the status is not.** Accepting it moves the file, and five of its eleven path citations sit in the influence module, which document work may not touch. |
| 0087 | An influence solve runs a fixed iteration count. It also names the boundary it draws against ADR-0022 D1, and a reviewer must settle that boundary before it moves. A review found every decision sound against the code and returned it for that boundary alone, so it moves when DEC-067 closes and needs no edit of its own.[^18] |
| 0085 | The Python boundary. It states that an entity crosses as one opaque identity that the engine resolves. The code that implements it exists, and a test proves that the resolution refuses a stale identity. |
| 0065 | A group is a site membership, not a region. **A review returned it.** D1's central claim and D2 hold, and D2's check is wired into the world's invariant pass. Two statements are false: D3 says the two capacity bounds are one bound, and FND-193 and DEC-081 already record that they are two; and D1 says the register resolved the military case this way, when the register resolved it in the opposite direction. DEC-091 holds the choice that follows. It moves when both are corrected. |
| 0088 | A tile field is a generated base and a stored change. Three tile fields now sit outside the dense column record, and nothing states the rule that picks between the two shapes. The code that implements it exists, and a visit census proves that building a world stores nothing for the field. Two reviews found it sound. The first returned one bullet of D1, which said a build visits no tile of the field. The second read the correction against the code and gave it an accept.[^18] [^19] **The verdict is in and the status is not.** Accepting it moves the file, and one citation of the old path sits in a source comment. |
| 0090 | A tile upgrade is stored sparsely. A dense array over the tiles is what a future contributor would reach for, and the code shows the sparse map without showing why the dense one was refused. The implementation exists and every decision was checked against it. Two reviews found it sound. The first returned one sentence of D3, which claimed that every caller asking how many units a tile holds composes the upgrade. The author keyed it on enforcement instead, and the second review tested that key against ADR-0074 D2 and gave it an accept.[^18] [^19] **The verdict is in and the status is not.** Accepting it moves the file, and nineteen citations of the old path sit in source comments. DEC-083 holds the choice that would remove the sweep. |

## Reserved and not written

A reserved row is a number, not a promise. **Do not write one because the row
exists.** Apply the three-condition test first; most reserved rows should stay
reserved until the code that needs them exists.[^6]

| No. | Why it sits here |
|---|---|
| 0021 | Layout follows the access pattern. Write it with the descent columns, not before.[^7] |

| 0051 | The selector as a lazy expression tree. The owner decided that a set-valued command is how the control plane reaches a set, so this governs every verb written after it. Write it before the next verb.[^12] |
| 0052 | A selector result may be a range. It carries the cost argument that makes 0051 affordable at the target scale, so the two are reviewed together.[^12] |
| 0040 | Python is a control plane. The orientation states it and nothing enforces it. It is the rule 0043 and 0051 rest on.[^12] |
| 0043 | A declared tier refuses the loop. Today the rule is prose, and prose lost to a missing read once already.[^13] |
| 0021 | Written and at `Draft`, with the descent columns it points at. It needs a reviewer, and the author must not be the reviewer.[^7] |
| 0069 | Weather. Nothing needs it. |
| 0077 | The golden state hash. Write it when the first real golden file is committed, not before.[^9] |
| 0055 | The modifier pipeline. One source modifies a rate, so the test fails today.[^8] |

The two rows above were written before the tree they describe, because the
project owner decided on 1 September 2026 that a set-valued command is how the
control plane reaches a set and that the selector tree is the destination the
verbs are written toward.[^12] **The author of these two must not review them.**

ADR-0051 and ADR-0052 arrived here for the same reason and have both been
written. ADR-0051 is accepted and has left this index. ADR-0052 waits for review
in the table above.

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
[^17]: Backlog item 0185, steer a step by the option the unit chose. `docs/backlog/refined/0185-steer-a-step-by-the-option-the-unit-chose.md`
[^18]: Review 0199, the influence, tile field, upgrade and housing records. `docs/reviews/0199-the-influence-tile-field-upgrade-and-housing-records.md`
[^19]: Review 0204, the two corrected records. `docs/reviews/0204-the-two-corrected-records.md`
[^20]: Findings register, FND-202. `docs/FINDINGS.md`
