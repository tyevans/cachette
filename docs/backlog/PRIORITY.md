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
| 0187 | Nothing moves a carried load into a store, so gathering cannot feed anybody. It moved here from `Next` because 0186 is complete and a unit now carries something. It closes the chain from the ground to the store, and it is what makes a hungry unit able to feed itself. |
| 0216 | It replaces 0185 and 0186 at the top, because both are complete and neither reaches a watcher. The engine steers a step by the option and orders a gather, and the demonstration feeds every unit, so the option is always `roam` and no unit ever forages. FND-226 holds the measurement. The loop that 0185 and 0186 built is invisible until this is done. |

## Next

These close a gap a review found, or they unblock the items above.

| No. | Why it sits here |
|---|---|
| 0224 | The control plane names one entity of a mass shape in four places, and a test pays four crossings for each site because of it. FND-215 holds the measurement. It sits below 0223 because the review that found it returned ADR-0040 for an amendment, and the amendment names this item. |

| 0189 | The rules against inert work look for an absent caller, and this defect has one. DEC-074 holds the options and recommends this. |
| 0190 | The pyramid folds level 1 into a state hash and nothing calls the fold. It is a small instance of what 0189 is about, and it sits below 0189 because the rule should come before the sweep. |
| 0194 | The tile value pass writes a random walk over every tile on every tick, and no reader decides anything from it. Items 0183, 0184 and 0188 removed the last three. What is left is storage, a hash contribution and a public reader. Nothing blocks it. |
| 0211 | The agent tool surface can go stale against the engine and nothing fails. ADR-0092 names the failure mode and does not fix it. A worker who meets a wall cannot tell a gap in the engine from a gap in the surface. |
| 0212 | Two places count what a window holds, and the rule for a full tile is written in both. The engine now holds a census the viewer could call. Two workers changed this ground in one round, so read both before planning. |
| 0104 | A ruler decides nothing that reaches anybody. DEC-040 carries the writ in the influence field. |
| 0171 | Building a world still passes over every tile, twice, through the first pyramid level. PRD-0003 states it must not, so the record is still false of the code. Item 0112 removed the third pass and FND-162 records what it left. |
| 0113 | Admission enforces the capacity from a bridge count that no test compares against a scan. |
| 0080 | The settings struct prices every new parameter at twenty-five files. |
| 0130 | Three registers state a next number that the rows already hold, and it went stale and conflicted four times in one night. |
| 0239 | A picture of a whole world walks every tile in it, and ADR-0094 D6 refuses that rather than serving it. Level 1 already holds the answer and the drawing cannot reach it. It sits below 0210 because 0210 halves the cost of every frame and this one adds a second path, and below the spike of the render verb, which has to exist before anything draws through it. ADR-0022 D4 forbids the easy version, so this cannot be a quiet fallback. |
| 0210 | The drawing generates the ground of every visible tile twice on every frame, and one of the two answers is a value it already holds. At the far zoom that is most of the cost of a drawing, and a watcher feels it as a sluggish camera. It sits above 0209 because it repairs something the owner reported and 0209 adds a distinction nobody asked for. It needs a reader that the core crate does not have. |
| 0209 | The holding border draws the same picture for a frontier with another faction and for the edge of the claimed ground. Between 6 and 13 held tiles in every 100 border only unclaimed ground, so the two cases are distinguishable. FND-206 holds the counts and warns against judging this layer from a render. |
| 0206 | Superseded. Another worker closed every gap this item names, in the same round it was written. It stays open only until somebody confirms that and closes it against the item that did the work. Do not take it. |
| 0235 | A register number has two authorities and neither can see the other: the next-number line answers from merged history, and the dispatcher's ranges live in prompts. Four collisions in one session, every writer following the procedure correctly. FND-219 records it. It sits beside 0198 because both are the registry storing state where it cannot be read atomically, and refining one should decide whether they are one item. |
| 0198 | The record check reads any mention of a record number as a citation, so a record cannot name a number the registry retired. FND-192 records it. It moved up from `Later` on 2 September 2026, when a review of the reserved log and Python boundary rows retired nine more numbers. The cost it names grows with each one, and a record that must explain why a claim was dropped is the record that most needs to name the number. |
| 0153 | Python holds no way to read an event, so an agent gets bytes and a digest. Any decoder in Python is a second copy of the layout. |
| 0161 | The control plane cannot say where to act, so a caller sweeps. Four reserved rows hold the answer and none is written. DEC-063 names it the destination. |
| 0155 | Every test fixture builds its own world by hand, and the rule that forbids the easy route has no shared answer. |
| 0163 | An item can be finished and still read as open, and no check compares what merged against the item that asked for it. |
| 0179 | No golden scenario builds anything, so neither determinism test defends work that takes several ticks. FND-174 holds the experiment that proved it. |
| 0165 | A register states the rule for reading its rows and holds a comparison the rule forbids. The example FND-141 proved false is still alive across the tree. |
| 0149 | The cost section of PRD-0018 states the mechanism of ADR-0080. A review holds the record at `Shaped` until it states cost alone.[^11] |
| 0059 | Housing. A review rejected ADR-0081, so this cannot be taken until a record replaces it. It sits here, not above, because nobody can start it. |
| 0060 | Population growth. It waits on 0059, which is itself stopped. |

## Later

These are real and none of them blocks anything today.

| No. | Why it sits here |
|---|---|
| 0105 | Goods over a network. Three reserved records are unwritten, and nothing holds a surplus yet. |
| 0106 | Show a watcher what is moving. The display shape follows from 0105. |
| 0107 | Decide how a faction stores what it observes. Nothing hides anything yet. |
| 0108 | Let a unit observe the tiles around it. Follows 0107. |
| 0109 | Decide how the world holds a condition that moves. |
| 0110 | Advance a weather condition each tick. Follows 0109. |
| 0111 | Let the weather change a unit and show it. Follows 0110. |
| 0063 | Assigning a unit to a position. The structure it writes into now exists. |
| 0181 | A kind of work maps onto the one commodity that exists, so the map carries no information. It waits on an economy that holds more than one. |
| 0065 | Letting the job decide what a unit weighs. Waits on 0063 and 0064. |
| 0169 | The influence solve runs on every tick for every faction, and the research says that is the wrong cadence at the target scale. It waits on a measurement. |
| 0124 | A fully recovered deposit still stores a take of zero. The recovery pass now reads that entry on every tick, so the depleted set grows and never shrinks. |
| 0125 | Show a watcher a deposit recovering. The engine recovers a deposit, and nothing shows it. |
| 0068 | A ruler and a succession. Item 0067 is complete, so the descent it reads exists. |
| 0088 | Promotion into the character tier. |
| 0180 | Nothing makes a unit choose to build, so a world left to run improves no tile. It waits on the faction rule of BLK-034. |
| 0050 | Four product collisions still carried. |
| 0099 | The faction mask union has no engine caller. |
| 0226 | The relabel pass has no caller in the step, so a world left to run cannot answer a dynasty question about anybody born since it started. |
| 0225 | The record check reads no source file from a worktree, so its uncited note is a false signal there. |
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
| 0022 | Random behaviour through a keyed draw. Read against 0064 first. |
| 0021 | Audit the movement records for acceptance. |
| 0004 | Reconcile the public API examples. |
| 0005 | The cross-cutting records. |
| 0007 | The storage claims. |
| 0168 | The control plane cannot name a dwelling, so a household binding would be inert. Waits on 0161. |
| 0167 | A reverse index from a dwelling to its units. Take it when a measurement asks for it, and BLK-007 says none exists. |
| 0200 | Admission admits more units onto a roaded tile than the position table believes it holds, and the fold that reports the largest capacity walks one of the two tables that state one. FND-193 records it. DEC-081 must close first, and no run reaches it until something makes a unit build. |
| 0201 | The record check drops every source file when it runs inside a worktree, so it reports records as uncited that source files cite. FND-194 records it. The same function also builds a corpus of every source file and never reads it, which FND-195 records and this item carries. It sits above 0198 because every worker runs in a worktree and reads the wrong count, and below 0200 because the check reports the note rather than failing on it. |
| 0205 | Accepting a record moves its file, and every citation of the old path then names nothing. Two records are at verdict Accept with a file move between them and binding, and the cost of a move scales with how well a record is cited. FND-197 records it, DEC-083 must close first, and 0198 is the same problem seen from the other side. |
| 0166 | The footnote baseline holds every document the new check would fail. It can only shrink, and it does not shrink by itself.[^10] |
| 0233 | The record of descent outlives every character and no reader delivers that, so a caller holds the descent identity of a dead ancestor and can ask nothing about it. DEC-092 must close first. It sits here because nothing exposes a character to the control plane, so the need is served in Rust alone. |
| 0234 | BLK-010 and ADR-0065 state opposite directions for one question, and the code follows the record. DEC-091 must close first. It sits here because only the workforce case is built, so the cost falls on the first person who builds a formation from the register. |
| 0221 | Source footnotes name the registry for six records that now have files. FND-214 found it. Nothing fails and nothing will, so it sits here, and it is worth a check rather than a sweep because the next reserved number a source file cites will do the same. |
| 0222 | The error hierarchy declares three exception types that nothing raises. ADR-0046 states the gap in its own consequences rather than claiming the capability, so a reader is not misled today. It waits on the selector for one of the three. |

## References

[^1]: Documentation Rules. `.claude/rules/documentation.md`
[^2]: Backlog guide. `docs/backlog/README.md`
[^3]: The priority check script. `scripts/check_priority.py`
[^4]: Blockers register, BLK-018. `docs/BLOCKERS.md`
[^7]: Findings register, FND-080. `docs/FINDINGS.md`
[^8]: Findings register, FND-100. `docs/FINDINGS.md`
[^9]: PRD-0018, a depleted deposit comes back. `docs/product/shaped/prd-0018-a-depleted-deposit-comes-back.md`
[^10]: Findings register, FND-130. `docs/FINDINGS.md`
[^11]: Findings register, FND-134. `docs/FINDINGS.md`
