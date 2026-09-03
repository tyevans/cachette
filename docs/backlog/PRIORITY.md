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
| 0299 | Make the bridge rebuild cost less than a sixth of a frame. **It is the largest single stage in the engine that no item held**, at 31.4 milliseconds of a 177.9 millisecond frame. It takes no thread count, and ADR-0071 D2 is why rather than an oversight, so refining it must say whether it changes that record or works inside it. Sixty-three percent of it is the ordering pass. FND-301 holds the measurement. |
| 0300 | Cut every panel line to the width of the panel. A note writes from the left edge with no right bound and no cut, so the length of the text is the only thing keeping it inside the panel: one note ran twelve pixels past the edge. A founded row does not cut its value either, so it is a class and not an instance. |
| 0302 | Fail when a stage declaration disagrees with a measurement. Three stages declared that they take a thread count while an accepted record says the pass they wrap accepts none, and every cost table written in one night printed the wrong answer for all three. FND-301 holds it. It sits below 0299 because the stage it would have caught is the one 0299 is about, and above the layout items because a wrong declaration misdirects every plan made from the table. |
| 0290 | Ask for a huge page in the allocation. A frame costs 3.9 percent less when the kernel gives 2 MB pages, measured on the target platform, and the engine asks for none, so it never gets them where it runs. FND-278 holds the measurement. It sits below the stage above because the saving it names was measured against a frame more than three times the size of the one the engine runs now, so what it is worth has to be taken again, and above the layout items because the saving is measured rather than derived. |
| 0277 | Hold a thread back when the work will not pay for it. The influence solve is 5.9 times slower at twelve threads than at one on the demonstration world, and it does not cross over at any extent measured up to a million tiles. **It sits first because it is the only row here that is worth more than it costs at every extent below the target one, and because it is the reason the demonstration runs at the rate it does**: the frame is between two and three times faster with fewer threads and no engine change. It is a decision before it is a change, because any work-per-thread rule holds a constant the current code was written to avoid. FND-294 holds the measurement. |
| 0270 | Score the option set with integer vector instructions. The pass has a fixed option count, one shared weight profile and dense columns, and the invariant against floating point is what makes a vector result bit-identical to the scalar one. **The project pins a stable toolchain, so the portable vector library is not available**, and the item is limited to shaping the loop for the compiler rather than writing intrinsics. It sits below the layout items on purpose: a vector unit fed one scattered value at a time is a scalar unit with extra steps. |
| 0266 | Order the unit arena by cell. The unit cost is 2.11 times higher with the units scattered than packed at the target scale, and the same layout is why the unit passes reach only 1.88 at 12 threads and 1.85 at 16. **It was called the largest measured cost that no item held. Item 0291 was larger, and it is now complete.** It sits here because a decision that follows the lattice asks for the units of a cell together, and because a reorder that runs every frame could cost more than it saves, which 0289 has now made measurable. |
| 0237 | Declare what each stage reads and writes. **The timing half is done under 0289, so what is left here is the checking half alone**: two stages whose write sets intersect cannot run together, and today only a reviewer can see that. It moved down because the reason it sat this high was the measurement, and the measurement exists. |
| 0272 | Decide whether the choice interval can go. **This is the behavioural half of what 0238 was for**, and 0238 deliberately left it. The deciding work no longer follows the population, so the reason a unit acts on a reading as old as the interval has weakened. **The measurement it waited for now exists**: the choice costs 0.571 milliseconds of an 836 millisecond frame at the target scale, which is 0.07 percent, so the interval buys almost nothing. It sits last here because removing it also removes what makes a choice sticky, and ADR-0064 D2 says why stickiness is what makes the behaviour legible. |

| 0305 | Give a laden unit a reason to go home. **The delivery of a carried load is built and it never runs**: a unit delivers only while it stands on the tile of its own site, and no option steers it there, so 4000 ticks of the demonstration world delivered nothing of any kind. Until this exists the economy is still the rate the founding set, and every conclusion about a hungry faction is a conclusion about that rate. FND-317 holds the measurement. It sits here because it is the half of the resource loop that 0187 could not close. |
| 0102 | A settlement reads its own ground rule, not the passability rule. Blocked by 0071 and 0092, in that order. |

## Next

These close a gap a review found, or they unblock the items above.

| No. | Why it sits here |
|---|---|
| 0293 | The move to a dated nightly opened a door the compiler used to hold shut: the reassociating float methods now compile, and only the script's name check stands in it. The lint can name them and does not. FND-284 holds the measurement and DEC-107 must close first. It sits at the top of `Next` because it is the one thing the toolchain move made worse before anything makes it better, and because the fix is three lines and a check. |
| 0294 | The toolchain is declared in three places that now disagree in kind. The manifest's minimum version claim is true today only because nothing uses an unstable feature yet, and the moment something does it becomes false silently. |
| 0295 | Nothing enforces that the toolchain names a dated nightly rather than a floating channel. One deleted word turns the pin back into a moving compiler, and ADR-0097 D2 is prose that nothing checks. |
| 0296 | The Miri gate has only ever passed, so its reach is unmeasured, and the project's own rule calls a test with no proven failure mode decoration. The gate exists to catch a defect every other gate waves through, and nobody has shown that its fixture reaches the case. It sits below 0274 because a gate that may not fire is a smaller loss than a rule nothing enforces, and above 0273 because it can be finished without a decision closing first. |
| 0244 | The project orientation is two files and they have already diverged: one carries a repaired sentence and the other still says no measurement exists on the target platform. An agent that opens the second reads something false and nothing fails. FND-259 holds it. It sits beside 0242 because both are one fact in two places with nothing that compares them, and refining one should ask whether they are one script. |
| 0242 | Nothing fails when a document states a register in its own words, and that was tested rather than assumed: a stale sentence went back into a product record and all eight document checks passed. FND-258 holds the case and FND-223 holds the ninety documents it cost last time. It sits at the top of `Next` because the sweep it replaces has now been paid for three times. |
| 0307 | The Python package declares its public interface in two places and nothing compares them: nine exception docstrings in the type stub are the same words as the Rust strings, and one class docstring is an abridged copy that dropped the paragraph warning against exactly this. FND-321 holds the evidence and ADR-0107 D2 and D3 state which site owns the prose. It sits beside 0242 and 0244 because all three are one fact in two places with nothing that compares the copies, and the repair is a generator and a check rather than a sweep. It does not wait on a documentation build, and no documentation build should start before it, because a published reference built over two declaration sites publishes whichever one the tool reached. |
| 0063 | Assigning a unit to a position. **The pairing pass now exists and the demonstration seats sixteen units**, so the zero this row was moved for is closed. What is left is point four of the item: the property of the unit that limits which position it may take, which needs a column on the unit row and a record of its own. ADR-0099 D3 states the gap in its own text. It stays here because the remainder is smaller than the row it replaces and because a unit row change should wait for the arena reorder. |
| 0279 | The position table folds into the state hash and no golden scenario reaches the pass that writes it, so a change to who works where moves no golden file. It was measured, not assumed: the seating pass landed and every golden file matched unrecorded. **The promotion pass met the same gap and closed it in three lines**, by stating its parameter in the scenario the way that scenario already states its recovery periods, so this item now has a worked example to copy and FND-293 holds it. It sits above 0278 because a golden file that cannot move is a guard that has already stopped working, and beside 0179, which is the same gap in the build pass. |
| 0278 | Nothing says that a subsystem produced no instance, so an audit is the only way to find one. Three were found that way, and two were found the same way before them. It sits below 0063 and 0088 because those two remove two of the three zeros, and a guard is worth more once the thing it guards can be non-zero. |
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
| 0239 | A picture of a whole world walks every tile in it, and ADR-0094 D6 refuses that rather than serving it. Level 1 already holds the answer and the drawing cannot reach it. It sits below the spike of the render verb, which has to exist before anything draws through it. Item 0210 sat above it and is complete, so the cheap repair to every frame is already made and this one adds a second path. ADR-0022 D4 forbids the easy version, so this cannot be a quiet fallback. |
| 0241 | The demonstration says whether each founded ground carries its group, and nothing drives that report. It is the guard that makes a silent fixture speak, and it is itself unguarded, which is the shape the rules call inert. It sits here rather than in `Now` because 0240 already repaired the fixture; this hardens the thing that would catch the next one. FND-232 holds why the report exists. |
| 0271 | Nothing can count how many times one frame generates a ground, so the repair item 0210 made rests on two tests in two crates and on reading one call site. FND-261 records the limit. It sits here because a contributor can put the second generation back and no test fails. |
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
| 0181 | A kind of work maps onto the one commodity that exists, so the map carries no information. It waits on an economy that holds more than one. |
| 0065 | Letting the job decide what a unit weighs. Waits on 0063 and 0064. |
| 0169 | The influence solve runs on every tick for every faction, and the research says that is the wrong cadence at the target scale. It waits on a measurement. |
| 0124 | A fully recovered deposit still stores a take of zero. The recovery pass now reads that entry on every tick, so the depleted set grows and never shrinks. |
| 0125 | Show a watcher a deposit recovering. The engine recovers a deposit, and nothing shows it. |
| 0068 | A ruler and a succession. Item 0067 is complete, so the descent it reads exists. |
| 0262 | A unit cannot destroy an upgrade. The engine removes one instantly through a control-plane call, and the owner answered on 3 September 2026 that destruction takes work while the instant path stays. It sits beside 0180 because both add a verb the choice pass will need, and neither should be written before item 0238 rewrites that pass. |
| 0180 | Nothing makes a unit choose to build, so a world left to run improves no tile. **The faction rule it waited on is answered**, and item 0058 is complete, so nothing blocks it. It stays here rather than moving up because it adds an option to the option set that item 0238 is rewriting, and it should read that pass after it changes rather than before. |
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
| 0167 | A reverse index from a dwelling to its units. Take it when a measurement asks for it, and the one run on the target platform measured nothing that does. |
| 0200 | Admission admits more units onto a roaded tile than the position table believes it holds, and the fold that reports the largest capacity walks one of the two tables that state one. FND-193 records it. DEC-081 must close first, and no run reaches it until something makes a unit build. |
| 0201 | The record check drops every source file when it runs inside a worktree, so it reports records as uncited that source files cite. FND-194 records it. The same function also builds a corpus of every source file and never reads it, which FND-195 records and this item carries. It sits above 0198 because every worker runs in a worktree and reads the wrong count, and below 0200 because the check reports the note rather than failing on it. |
| 0205 | Accepting a record moves its file, and every citation of the old path then names nothing. Two records are at verdict Accept with a file move between them and binding, and the cost of a move scales with how well a record is cited. FND-197 records it, DEC-083 must close first, and 0198 is the same problem seen from the other side. |
| 0166 | The footnote baseline holds every document the new check would fail. It can only shrink, and it does not shrink by itself.[^10] |
| 0233 | The record of descent outlives every character and no reader delivers that, so a caller holds the descent identity of a dead ancestor and can ask nothing about it. DEC-092 must close first. It sits here because nothing exposes a character to the control plane, so the need is served in Rust alone. |
| 0234 | BLK-010 and ADR-0065 state opposite directions for one question, and the code follows the record. DEC-091 must close first. It sits here because only the workforce case is built, so the cost falls on the first person who builds a formation from the register. |
| 0221 | Source footnotes name the registry for six records that now have files. FND-214 found it. Nothing fails and nothing will, so it sits here, and it is worth a check rather than a sweep because the next reserved number a source file cites will do the same. |
| 0222 | The error hierarchy declares three exception types that nothing raises. ADR-0046 states the gap in its own consequences rather than claiming the capability, so a reader is not misled today. It waits on the selector for one of the three. |
| 0243 | The accepted records say in their own words that no measurement exists on the target platform, and the retcon window forbids repairing them. DEC-096 must close first, and a reviewer owns it. It sits here because the documents that guide work today are already repaired and these are read for their claims rather than for their cost clauses. |
| 0229 | The first measurement on the target platform found a frame cost at 4,096 tiles that disagrees with every larger extent, and four threads beat one thread on a machine with two. It sits last because it explains a figure rather than repairing behaviour, and because 4,096 tiles is the size most tests use, so anything it finds is paid by the suite and not by a player. |
| 0301 | The gate budget row describes a tree that no longer exists, and the cost report compares against it on every run, so every contributor reads a figure that means nothing many times a day. It sits above 0251 because 0251 is one candidate repair and this is the measurement that would say whether it is the right one. It needs a quiet machine and a tree nobody merges into, which is the part that has been hard to get. |
| 0251 | The Python test recipe now costs about as much as the whole gate budget, and 22 tests each start the agent server as a fresh subprocess. It sits here rather than higher because no per-module figure exists yet: the cost is known for the whole Python run and not for this module, and a shared fixture that hides a state leak between tests would be worse than the wait. Measure first, then decide. |

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
