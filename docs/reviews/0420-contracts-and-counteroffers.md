# 0420 — Contractual trade between two factions, with counteroffers

This is the written outcome of backlog item 0420. It states the impact review
that was made before the work started, the decisions the work took, the
defects that were put back to prove each test, and every gate.

**The author of this work wrote this file.** Nothing here is a second reader's
verdict. The three decision records it produced carry the status `Draft`, and
only a reviewer moves a record past that.[^1]

## 1. What was asked for

The project owner asked for contractual trades between two players, with
counteroffers, and with a refusal that says no and asks for no more
counteroffers. The consumer is a game in which a player directs a
congregation, and a player is a person or a language model.

The project owner also stated one rule of that game in full. A player sends a
message to another player only while one of its own units stands in that
player's territory.

## 2. The impact review, record by record

This was made before any code was written.

**ADR-0001 D4, one binary gives one answer at any thread count.** A contract
is state that a later frame reads, so the plane enters the state hash. The
settlement pass runs on the calling thread and takes no thread count.
*Honoured.*

**ADR-0002 D1, no floating point in simulated state.** Every quantity, term,
deadline and closure here is an exact integer. The store the delivery writes is
a fixed-point value, and the transfer converts a whole number into it and back
without rounding. *Honoured.*

**ADR-0002 D2, arithmetic goes through the arithmetic module.** The transfer
uses the module for every add and subtract on a fixed-point quantity. The float
ban script passes. *Honoured.*

**ADR-0003 D1, every random draw is keyed.** **Nothing in this work draws.**
Every outcome follows from the terms, the tick, and what the carriers
delivered. The module says so in its own text, so a later contributor who adds
a draw has to remove that sentence. *Honoured, by absence.*

**ADR-0004 D1, D3 and D4, iteration order is explicit.** Three walks exist. The
plane is walked in pair order, which is an array index order. The carriers are
collected in unit slot order and then sorted on the settlement and the unit
identity before anything moves. The default pass walks the plane in pair order
and collects before it mutates. No hash order and no thread completion order is
read anywhere. *Honoured.*

**ADR-0006 D1, an event is plain data with declared padding.** The event type is
`repr(C)`, derives `Pod`, holds no boolean, and declares its two padding bytes.
The row type is `repr(C)` and holds no padding at all, and a constant assertion
fails the build when a field changes that. *Honoured.*

**ADR-0006 D2, events cross at the barrier.** The log is cleared at the top of
the step and read after it. No Python runs inside a step. *Honoured.*

**ADR-0040 D1, Python is a control plane.** A verb takes two faction numbers
and six values, and answers once. The read that a player uses to decide takes
one faction and answers every pair it is a party to, as columns, in one
crossing. Nothing asks the caller to loop. *Honoured.*

**ADR-0053 D2, a relation between factions is a plane.** The negotiation is one
row for each ordered pair of factions and it never follows the population.
*Honoured.*

**ADR-0062 D3 and D5, a pass that moves a quantity runs before the rates.** The
settlement pass runs directly after the ordinary delivery and before the rate
pass and the consumption pass. *Honoured.*

**ADR-0072 D5, conservation.** A delivery moves a quantity out of a carry and
into a store, so it credits the account that links the two. A transfer that
forgot it breaks the conservation check on the frame of the first delivery, and
the defect probe in section 6 shows that it does. *Honoured.*

**ADR-0073 D2, sort then admit.** The deliveries are ordered on a total key
before any store is written. *Honoured.*

**ADR-0085 D3, an identity crosses as one opaque value.** No trade verb takes an
entity identity. A faction number is not an entity, so nothing here resolves
one. *Not applicable.*

**ADR-0090 D1, a store that holds nothing until somebody writes.** The plane
follows it: a world that never traded allocates no row, and the golden state
hash did not move. *Honoured.*

**Blockers checked before starting.** BLK-007 governs every cost figure, so no
figure in the records is a measurement. BLK-050 holds the rules of the
downstream game, so every value a game would tune is a parameter the caller
supplies: the quantities, the kinds, the term and the closure duration. BLK-036
asks whether an upgrade changes hands with the ground, and it does not touch
this work.

**Findings checked before starting.** FND-362 says the presence question is a
relation between factions, and section 4 says why this work answers one pair
directly rather than waiting for the relation.

**Records created.** Three, and each states a claim a future contributor could
reasonably choose otherwise on.[^2] [^3] [^4] The registry rows were added
before the records were written.

## 3. Where the negotiation lives, and why

**The engine holds it.** The engine holds the resource kinds, the quantities,
the term, the deadline, the closure and the status. The control plane holds
every word.

The control plane was a real candidate for the whole exchange. A negotiation is
a conversation, it changes between frames, and one of the two players is a
language model that already keeps a transcript.

**Two facts refused it.** A contract binds future delivery, so it is engine
state whatever happens to the conversation. And the acceptance that turns a
conversation into a contract is the last act of the conversation. Splitting them
puts one fact in two places with nothing that fails when the copies disagree,
and that shape has a local instance in this project already.[^5]

**No cost argument favoured the control plane either.** The plane is one row for
each ordered pair of factions. A world at the target population and a world with
one unit hold the same plane, and it holds no row at all until somebody speaks.

The decisions register holds the outcome and the reasoning.[^6]

## 4. The presence gate

**A trade rides the same gate as a message.** An offer, a counteroffer, an
acceptance, a refusal and a terminal refusal each require a live unit of the
speaker standing on ground the listener holds.

**A bound contract needs no presence.** An obligation outlives the messenger.
Requiring presence for delivery would make a contract unenforceable the moment a
player pulled its units back.

The gate is answered from primary state. It walks the unit column and reads the
holder of the tile each unit stands on. It stores nothing, so it is not a second
copy of an answer that a derived relation also holds. A separate item builds the
whole-world presence relation, and when it lands this walk becomes a one-bit
read with no change of behaviour. The findings register holds that.[^7]

**The cost.** One column read for each live unit, once for each speech act, and
a speech act happens between frames. No frame pays for it.

## 5. The four questions the work had to answer

### A terminal refusal differs from a refusal

**A refusal ends one negotiation and closes nothing.** The pair is idle
afterwards, and either party may open a new one on the next call.

**A terminal refusal ends the negotiation and closes the direction the other
party would open, until a tick the refusing party names.** An offer made before
that tick is refused, and the refusal states the tick in its message.

**The closure is directional.** The refusing party shut its own door. It may
still open a negotiation toward the other party, because it promised no silence
of its own.

**Only the refusing party opens the direction early.** Nothing the other party
does shortens it. That is what makes it terminal rather than a delay.

**The duration comes from the caller and never from the engine.** A duration of
zero is refused, and a duration that saturates the clock is a permanent break.
Nothing but time and the refusing party's own verb ends a closure.

**A player can read all of it.** The row states the tick that opens the
direction, so a player that receives a no can tell at once whether asking again
is allowed. That is the whole reason the closure is engine state: a language
model that cannot tell a refusal from a closed door asks for ever, and the loop
costs a token budget rather than a frame, so nothing in the engine would ever
notice it.

### Goods do not move instantly

**A unit carries them.** No verb and no pass moves a quantity from the store of
one settlement into the store of another. A unit of the debtor that carries the
resource the contract names, and that stands on the tile of a settlement the
creditor holds, delivers what it carries against the debt.

A store-to-store transfer was the shorter answer and it was rejected. The engine
already moves a quantity by carrying it, and the record that opened the resource
sink says why a store may not simply rise.[^8] The downstream game is about
presence and territory, and a trade whose goods appear without anybody carrying
them makes the map decorative for the whole economy. The distance between two
players then becomes a real cost of trading with them, and it falls out of the
map rather than out of a rate the engine invents.

**A delivery never passes the debt.** The transfer takes the smallest of what
the unit carries, what the party owes, and what the store can hold. A quantity
the store cannot hold stays in the carry and arrives on a later tick.

### What happens when a party cannot deliver

**The contract fails at its deadline.** The deadline is checked after the
delivery of that tick, so a contract whose deadline is this tick gets this
tick's delivery and fails only when a debt survives it.

**Nothing is returned.** A quantity that already arrived stays in the store it
arrived at. Taking it back would need a transfer that no unit carried.

**The defaulting party loses the direction it would ask on again, for as long as
the contract ran.** The duration is the term of the contract itself, so no
balance figure decides it and no measurement can make it stale. When both
parties owe, both lose their direction. The closure is the same mechanism a
terminal refusal uses, rather than a second one.

**What a default does not cost.** There is no reputation, no fine and no
seizure. Whether the game wants a heavier cost is an open decision.[^9]

### The negotiation is visible to everybody, in the engine

**The engine holds no notion of who is asking, and it answers any ordered
pair.** A notion of the asker is an authentication model, and the engine process
serves every player at once.

**The read is shaped for privacy without enforcing it.** A caller asks for the
rows one faction is a party to, so a control plane that hands each player its
own view writes one call. Whether the game wants a negotiation private at all is
a rule nobody has stated, and a blocker holds it.[^10]

### What a player reads to decide

Three reads, and all of them cross once.

One answers a single ordered pair, with the status, whose turn it is, both sides
of the terms, what each party has delivered, the deadline, the term, the tick
that opens a closed direction, and how many times somebody spoke.

One answers every pair a faction is a party to, as columns, so a player never
asks about one pair at a time.

One answers what the last step said: one entry for each speech act and for each
settlement or default the step resolved.

A fourth read answers the gate directly, so a player can ask whether it may
speak before it tries.

## 6. The defects that were put back

Each defect below was written back into the engine, the extension was rebuilt,
and the test that covers the rule was run. A rule whose test stayed green would
mean the test measures the fixture rather than the engine.

| The rule | The defect | The test | Verdict |
|---|---|---|---|
| A delivery never passes the debt | The transfer took the smallest of the carry and the room, and dropped the debt from the comparison | `test_a_bound_contract_moves_the_goods` | **Caught** |
| A terminal refusal closes the direction the other party would open | The closure was written on the live row rather than on the reverse one | `test_a_terminal_refusal_closes_the_pair_and_says_when_it_opens` | **Caught** |
| A delivery credits the account that links the carries to the stores | The credit was replaced by an addition of zero | `test_a_bound_contract_moves_the_goods` | **Caught, after the test was repaired** |
| A terminal refusal closes the direction for the duration the caller names | The closure tick was written as zero, so the verb behaved as a plain refusal | `test_a_terminal_refusal_closes_the_pair_and_says_when_it_opens` | **Caught** |
| A closed direction opens at the tick it names, and not one tick later | The comparison that refuses an offer was widened from strict to inclusive | `test_a_closure_ends_at_the_step_it_named` | **Caught** |

**The third defect was missed on the first run, and that is the finding of this
section.** The test asserted that the store rose and that the contract recorded
the delivery, and both stayed true when the account was not credited. The
conservation check is the only thing that sees it, and the test did not call it.
The test now calls it, and the same defect then fails the test. **The rule that
a fixture proves nothing until the defect is put back held here, and it held on
the one rule that looked the least likely to need it.**[^14]

Each probe wrote the defect into the engine, rebuilt the extension, ran the
named test through the installed package, and restored the tree.

## 7. The gates

Every gate below ran in this worktree after the last change.

| The gate | The command | The result |
|---|---|---|
| Rust format | `cargo fmt --all -- --check` | Passed, no output |
| Rust lint | `cargo clippy --workspace --all-targets -- -D warnings` | Passed, `Finished dev profile` |
| Rust tests | `cargo test --workspace` | Passed, 0 failed across every test binary |
| Thread-count equivalence | `just determinism` | Passed, 14 tests |
| Golden state hash | `just determinism` | Passed, 2 tests. **The hash did not move**, because the plane holds no row until a world trades |
| Records | `just records` | Passed, 0 failures in each of the eight checks |
| Python lint and types | `just lint-python` | Passed, `All checks passed!` and `Success: no issues found in 23 source files` |
| Python tests | `just test-python` | Passed, 111 tests, 16 of them new |
| Documentation | `just docs` | Passed, `69 members with prose, 0 without`, and every summary reached the site |
| Documentation probe | `just docs-probe` | Passed, `both cases failed the job` |

**One lint suppression was added, with a reason at the site.** The verb that
opens a negotiation takes eight arguments, which is one above the threshold.
A structure that held the terms would be a second name for the row the function
writes, and the control plane would then state the terms twice: once to build
it and once to read the row back.

## 8. What was left undone

**The second product number was not used.** The allocation held product records
0034 and 0035. One record states the need, and splitting one need across two
records would have given a reader two gates to answer for one thing.

**The settlement pass opens no stage.** It follows the precedent of the delivery
pass beside it, which opens none either, so neither appears in a frame cost
report. This was found during the work rather than designed, the findings
register holds it, and a backlog item holds the repair for both at once.[^11]
[^12] Adding a stage for this pass alone would have left the older one behind
and made the table harder to read than a table that covers neither.

**No measurement was taken.** Every cost statement in the records and in this
file argues about which term a cost follows. None of them is a result. One
blocker governs every cost figure in this project.[^13]

**The golden state hash did not move**, because the plane holds no row until a
world trades and the golden fixture never trades. Nothing was regenerated.

**A load that the store cannot hold has no test.** The transfer clamps to the
room the store has, and the fixture never fills a store to its ceiling, so that
branch is covered by reading and not by a test.

**Two rules are enforced by one branch each and rest on the blockers.** A
contract toward a faction with no settlement runs to its deadline and fails, and
the engine answers a negotiation to any caller. Both are stated plainly so that
a change of game rule changes one branch.

## References

[^1]: ADR Registry, who reviews. `docs/adrs/REGISTRY.md`
[^2]: ADR-0126, a trade negotiation is engine state, and the words are not. `docs/adrs/draft/adr-0126-a-trade-negotiation-is-engine-state.md`
[^3]: ADR-0127, a terminal refusal closes an ordered pair until a named tick. `docs/adrs/draft/adr-0127-a-terminal-refusal-closes-a-pair-until-a-named-tick.md`
[^4]: ADR-0128, a contract moves a quantity only when a unit carries it onto the ground of the other party. `docs/adrs/draft/adr-0128-a-contract-moves-a-quantity-only-when-a-unit-carries-it.md`
[^5]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
[^6]: Decisions register, DEC-210. `docs/DECISIONS.md`
[^7]: Findings register, FND-432. `docs/FINDINGS.md`
[^8]: ADR-0062, production and upkeep are rates attached to a site, decision D2. `docs/adrs/accepted/adr-0062-production-and-upkeep-are-rates-attached-to-a-site.md`
[^9]: Decisions register, DEC-212. `docs/DECISIONS.md`
[^10]: Blockers register, BLK-121. `docs/BLOCKERS.md`
[^11]: Findings register, FND-431. `docs/FINDINGS.md`
[^12]: Backlog item 0421, put the two quantity passes into the stage table. `docs/backlog/proposed/0421-put-the-two-quantity-passes-into-the-stage-table.md`
[^13]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^14]: Testing Rules, section 2a. `.claude/rules/testing.md`
