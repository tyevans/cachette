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
| 0102 | A settlement reads its own ground rule, not the passability rule. Blocked by 0071 and 0092, in that order. |

## Next

These close a gap a review found, or they unblock the items above.

| No. | Why it sits here |
|---|---|
| 0104 | A ruler decides nothing that reaches anybody. DEC-040 carries the writ in the influence field. |
| 0171 | Building a world still passes over every tile, twice, through the first pyramid level. PRD-0003 states it must not, so the record is still false of the code. Item 0112 removed the third pass and FND-162 records what it left. |
| 0113 | Admission enforces the capacity from a bridge count that no test compares against a scan. |
| 0080 | The settings struct prices every new parameter at twenty-five files. |
| 0130 | Three registers state a next number that the rows already hold, and it went stale and conflicted four times in one night. |
| 0153 | Python holds no way to read an event, so an agent gets bytes and a digest. Any decoder in Python is a second copy of the layout. |
| 0161 | The control plane cannot say where to act, so a caller sweeps. Four reserved rows hold the answer and none is written. DEC-063 names it the destination. |
| 0155 | Every test fixture builds its own world by hand, and the rule that forbids the easy route has no shared answer. |
| 0163 | An item can be finished and still read as open, and no check compares what merged against the item that asked for it. |
| 0165 | A register states the rule for reading its rows and holds a comparison the rule forbids. The example FND-141 proved false is still alive across the tree. |
| 0149 | The cost section of PRD-0018 states the mechanism of ADR-0080. A review holds the record at `Shaped` until it states cost alone.[^11] |
| 0059 | Housing. A review rejected ADR-0081, so this cannot be taken until a record replaces it. It sits here, not above, because nobody can start it. |
| 0060 | Population growth. It waits on 0059, which is itself stopped. |

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
| 0169 | The influence solve runs on every tick for every faction, and the research says that is the wrong cadence at the target scale. It waits on a measurement. |
| 0124 | A fully recovered deposit still stores a take of zero. The recovery pass now reads that entry on every tick, so the depleted set grows and never shrinks. |
| 0125 | Show a watcher a deposit recovering. The engine recovers a deposit, and nothing shows it. |
| 0133 | The panel is longer than the window and cuts. A watcher cannot reach the rows below the notice. |
| 0068 | A ruler and a succession. Item 0067 is complete, so the descent it reads exists. |
| 0088 | Promotion into the character tier. |
| 0097 | The layout record. Write it with the descent columns, not before.[^5] |
| 0050 | Four product collisions still carried. |
| 0099 | The faction mask union has no engine caller. |
| 0135 | The deposit amount reader has no caller, and the record now rests on the step order instead. |
| 0072 | The panel fit check has no production caller. |
| 0077 | The batched structural path, once its record exists. |
| 0145 | The faction count states what zero means at six sites in one module. Item 0080 may absorb it. |
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
| 0168 | The control plane cannot name a dwelling, so a household binding would be inert. Waits on 0161. |
| 0167 | A reverse index from a dwelling to its units. Take it when a measurement asks for it, and BLK-007 says none exists. |
| 0166 | The footnote baseline holds every document the new check would fail. It can only shrink, and it does not shrink by itself.[^10] |

## References

[^1]: Documentation Rules. `.claude/rules/documentation.md`
[^2]: Backlog guide. `docs/backlog/README.md`
[^3]: The priority check script. `scripts/check_priority.py`
[^4]: Blockers register, BLK-018. `docs/BLOCKERS.md`
[^5]: Decisions register, DEC-032. `docs/DECISIONS.md`
[^7]: Findings register, FND-080. `docs/FINDINGS.md`
[^8]: Findings register, FND-100. `docs/FINDINGS.md`
[^9]: PRD-0018, a depleted deposit comes back. `docs/product/shaped/prd-0018-a-depleted-deposit-comes-back.md`
[^10]: Findings register, FND-130. `docs/FINDINGS.md`
[^11]: Findings register, FND-134. `docs/FINDINGS.md`
