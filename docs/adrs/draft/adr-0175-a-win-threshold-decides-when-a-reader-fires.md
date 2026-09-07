# ADR-0175: A win threshold decides when a reader fires and never what the simulation does

## Context

The engine ends a game when a reader fires. Each reader compares a running
quantity of a faction against a value: the renown of its best character against
a target, the tick against a limit, a standing victory claim against zero.[^1]

The project owner wants a search over those values, so that games end inside a
horizon and each win path takes a comparable share. A search needs to try many
candidates.

**Every such value was a compile-time constant.** A candidate therefore cost a
rebuild of the Rust core, and a search of any width was not affordable.

There is a second problem underneath the first. A search that plays one run for
each candidate pays for the whole simulation each time. Most of the values do not
change the simulation at all: they decide when the run stops being watched. If
the engine can be asked to watch nothing, one run yields a trajectory that scores
any set of those values by reading it afterward. The search then costs one run
for each candidate of the values that do change the simulation, and nothing for
the rest.

That saving is only real when it is true. It rests on a property the engine must
hold and a test must check.

## Decision

### D1. Each win value is a value the world holds, and its constant is the default

Each value that a game end reader compares is a value the world holds and a
caller sets through one setter. The constant that held it stays, and it names the
default. A world that nobody configures behaves as it did before the value became
settable.

Each such value is state that the step reads, so each enters the state hash.[^2]
No such value is a floating point number: each is a whole number or a raw
fixed-point value at the project scale.[^3]

A value that already lives in a table a caller can write stays in that table. A
setter for it writes one column of the row and does not declare the value a
second time.[^4]

A reviewer finds a violation when a reader compares a constant rather than a
value the world holds, when a setter creates a second declaration site for a
value a table already holds, or when a win value does not reach the state hash.

### D2. A value is a threshold or a rate, and the difference is stated

A threshold decides when a reader fires and changes nothing else the simulation
does. A rate changes what the simulation does.

The engine publishes which each value is, in the manifest that a caller reads to
find the setters. A caller that scores a recorded run may vary a threshold
freely. A caller that varies a rate must play a new run.

This distinction is a claim about the code, not a label. A value marked as a
threshold that any pass reads is a defect, not a mislabelling.

A reviewer finds a violation when a pass other than a game end reader reads a
value the manifest calls a threshold.

### D3. Turning the readers off changes nothing but the record

A caller may turn the game end readers off. While they are off no reader records
a game end and the world runs to the tick limit.

**A run with the readers off holds the same event log as a run with the readers
on that never fires.** The readers decide when the step stops watching. They
change nothing else.

This is what makes the recording mode worth having: one run to the limit yields
the trajectory that scores any threshold vector as post-processing.

A test holds this. It plays one run with the readers on that never fires and one
with the readers off, and compares the event logs byte for byte. A second test
proves the comparison can fail, by lowering the tick limit so that the watching
run ends and the logs then differ.[^5]

A reviewer finds a violation when the reader switch reaches any pass other than
the game end readers.

## The alternatives this rejects

**Read the values from a file at world construction.** Rejected because a search
sets a value on a world it already built, and because a file is a second
declaration site with no check that the copies agree.[^4]

**Rebuild for each candidate.** Rejected on cost. The project owner asked for a
search, and a search whose inner loop is a compiler is not one.

**Score every candidate from one run, whatever the value.** Rejected because it
is false for a rate. A renown share that a felled unit gives is written into the
renown column, which a later frame reads, so a different share is a different
run. The threshold-and-rate split exists to say which values this shortcut is
sound for.

**Let the readers run and ignore the record.** Rejected because the record is not
inert. After a game end the controllers emit nothing, so the run diverges from
the moment a reader fires.[^1] A recorded run must therefore be one in which
nothing fired.

**Keep the reader switch out of the state hash, so that the two runs hash the
same.** Rejected because the switch is a stored value the step reads, and the
rule about the hash admits no exception for convenience.[^2] The equivalence this
record claims is about the event log, and the test states it there.

## Consequences

**A caller can now end a game early or never.** A world configured with a
threshold far below its running value ends on the first tick. A world with the
readers off never ends. Neither is a defect, and a caller that wants the shipped
game must leave the values alone.

**The manifest is a second statement of every default.** The engine is the first.
A check reads the manifest, reads each default back from a world nobody
configured, and fails when the two disagree. Without that check the manifest
would drift and mislead a search in silence.[^4]

**A setter must be reachable for reading back.** A value a caller can set and
cannot read is a value a search cannot verify it applied. Each setter has a
matching reader on the boundary.

**The threshold-and-rate labels bind future work.** A pass that starts reading a
value labelled a threshold breaks the search that this record makes possible, and
nothing in the compiler notices. The label is the constraint, and a reviewer
enforces it.

## References

[^1]: ADR-0148, a game end is recorded once and stops the controllers, decisions D1 and D4. `docs/adrs/accepted/adr-0148-a-game-end-is-recorded-once-and-stops-the-controllers.md`
[^2]: ADR-0164, every stored value the step reads enters the state hash, decision D1. `docs/adrs/draft/adr-0164-every-stored-value-the-step-reads-enters-the-state-hash.md`
[^3]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^4]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
[^5]: Testing rules, section 1. `.agents/rules/testing.md`
