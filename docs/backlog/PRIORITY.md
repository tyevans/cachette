# Backlog Priority (Index)

This document is an **index**. It states the order in which the open backlog
items should be taken, and why each sits where it does.

**It holds no title, no status and no summary.** Those live in the item file,
and a second copy of them here would go stale the moment an item moved. This
document holds one thing the item files cannot hold: the order between them.

Rule 4 of the documentation rule exempts a table that lists files as data.[^1]

## How to use this

Take the highest item you can start. An item in `proposed/` must be refined
first, and refining it is the work.[^2]

**A number here is not a promise.** Order changes when the project learns. Move
a row and say why in the commit body.

**Every open item appears exactly once.** A check enforces it, so an item
cannot be forgotten by being left off this list.[^3]

## Now

These answer a need the project owner has stated, or they repair something the
engine gets wrong today.

| No. | Why it sits here |
|---|---|
| 0084 | Two values name a tile's owner, and the event log publishes the wrong one to Python.[^6] |
| 0102 | A settlement reads its own ground rule, not the passability rule. Blocked by 0071 and 0092, in that order. |

## Next

These close a gap a review found, or they unblock the items above.

| No. | Why it sits here |
|---|---|
| 0120 | The engine gains state the window never shows. Refining it is the work.[^8] |
| 0059 | Housing. The population cannot be bounded by anything until a place has capacity. ADR-0081 holds the decisions and waits for review. |
| 0060 | Population growth from the store and the housing. It waits on 0059, and DEC-044 decides whether a grown population can survive.[^9] |
| 0103 | A household has no home. DEC-039 answers it: derive the household from the dwelling slot. |
| 0104 | A ruler decides nothing that reaches anybody. DEC-040 carries the writ in the influence field. |
| 0112 | Building a world passes over every tile. PRD-0003 states it must not, so the record is false of the code. |
| 0113 | Admission enforces the capacity from a bridge count that no test compares against a scan. |
| 0080 | The settings struct prices every new parameter at twenty-five files. |
| 0130 | Three registers state a next number that the rows already hold, and it went stale and conflicted four times in one night. |
| 0101 | A behavioural claim about terrain is defended by a test of constants.[^7] |

## Later

These are real and none of them blocks anything today.

| No. | Why it sits here |
|---|---|
| 0062 | Ranked positions at a site. Wanted by job assignment. |
| 0105 | Goods over a network. Three reserved records are unwritten, and nothing holds a surplus yet. |
| 0106 | Show a watcher what is moving. The display shape follows from 0105. |
| 0107 | Decide how a faction stores what it observes. Nothing hides anything yet. |
| 0108 | Let a unit observe the tiles around it. Follows 0107. |
| 0109 | Decide how the world holds a condition that moves. |
| 0110 | Advance a weather condition each tick. Follows 0109. |
| 0111 | Let the weather change a unit and show it. Follows 0110. |
| 0063 | Assigning a unit to a position. Waits on 0062. |
| 0065 | Letting the job decide what a unit weighs. Waits on 0063 and 0064. |
| 0058 | Improvements. Waits on a site that produces. |
| 0123 | A deposit never comes back. Waits on 0057, because nothing drains the world until a shortage bites.[^9] |
| 0124 | A fully recovered deposit still stores a take of zero. Follows 0123. |
| 0125 | Show a watcher a deposit recovering. Follows 0123. |
| 0067 | Descent. Waits on the character tier being used for something. |
| 0068 | A ruler and a succession. Waits on 0067. |
| 0088 | Promotion into the character tier. |
| 0097 | The layout record. Write it with the descent columns, not before.[^5] |
| 0050 | Four product collisions still carried. |
| 0099 | The faction mask union has no engine caller. |
| 0072 | The panel fit check has no production caller. |
| 0077 | The batched structural path, once its record exists. |
| 0053 | Superseded in substance by the completed resource work. Close it or restate it. |
| 0036 | How a watcher reads a count of the whole world. |
| 0043 | How a level 1 cell is repaired. |
| 0041 | Read the ground once for each target. |
| 0046 | Read the ground of a new world in parallel. |
| 0039 | A rejected unit is not stuck. Waits on units having plans. |
| 0040 | Record where an out-of-frame change gets its barrier. |
| 0034 | Measure the generated terrain against a stored one. |
| 0011 | Record the terrain multiplier. |
| 0022 | Random behaviour through a keyed draw. Read against 0064 first. |
| 0021 | Audit the movement records for acceptance. |
| 0004 | Reconcile the public API examples. |
| 0005 | The cross-cutting records. |
| 0007 | The storage claims. |
| 0009 | The log claims. |
| 0010 | The Python boundary claims. |

## References

[^1]: Documentation Rules. `.claude/rules/documentation.md`
[^2]: Backlog guide. `docs/backlog/README.md`
[^3]: The priority check script. `scripts/check_priority.py`
[^4]: Blockers register, BLK-018. `docs/BLOCKERS.md`
[^5]: Decisions register, DEC-032. `docs/DECISIONS.md`
[^6]: Findings register, FND-079. `docs/FINDINGS.md`
[^7]: Findings register, FND-080. `docs/FINDINGS.md`
[^8]: Findings register, FND-100. `docs/FINDINGS.md`
[^9]: PRD-0018, a depleted deposit comes back. `docs/product/shaped/prd-0018-a-depleted-deposit-comes-back.md`
