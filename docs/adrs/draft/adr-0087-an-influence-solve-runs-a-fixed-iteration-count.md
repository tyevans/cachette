# ADR-0087: An influence solve runs a fixed iteration count over the whole plane

## Context

An influence field spreads what a source puts into it. The spreading is a
relaxation: each pass replaces a cell with a weighted blend of itself and its
neighbours, scaled by how freely the ground between them carries influence.
The storage that the passes run over is a separate decision.[^1]

A relaxation solver is normally stopped by a convergence test. The solver
measures how much the last pass changed, compares that against a threshold,
and stops when it is small. It is the standard form, a contributor who has
written one before will reach for it, and the code cannot say why it must not
be here.

**This project cannot recover determinism once it is lost.** One binary must
give one answer at any thread count, and the rules that protect that are
invisible at review time.[^2] A convergence test is one of them: a solver that
stops on a residual invites a parallel reduction of that residual, and a
parallel reduction takes its order from the threads.[^3] [^4]

### The forces

**A solve reaches only as far as its passes.** One pass moves influence one
cell. A field that must span a province cannot be produced by a pass count
small enough to run every tick.

**The field therefore carries between solves.** A solve applies its passes to
the field the last solve left, so the reach grows over ticks rather than
inside one tick. That is also what produces the behaviour the project chose
for a faction with no source: the field falls rather than vanishing, and the
periphery loses its hold before the seat, because the periphery was the part
the interior was holding up.[^5]

**A plane that carries between solves is not a summary.** The accepted record
that owns the levels says that every fact lives at level 0, and that a value
which appears only at a level above it is a defect.[^6] This plane holds such
a value. It also makes no claim that a cell equals the combination of the
tiles it covers, because a cell of it reads its neighbours. The conflict is
real, it is named here rather than hidden, and an open row carries it to a
reviewer with three options and a recommendation.[^7]

**A convergence test is not the only thing that reads a thread count.** A
stencil that clips a neighbour read to the run one thread is filling produces
a plausible field at every cell and a different field at every thread count.
The run boundary is the thread count in disguise.

**A fixed pass count is not checkable by comparing two runs.** The project
learned this: a defect that repeats gives one answer on every thread and on
every run, so both determinism tests pass over it.[^8]

## Decision

### D1. A solve runs a constant number of passes, whatever the input

The solve loops a constant number of times and returns. It reads no clock, it
computes no residual, and it compares nothing against a threshold.

**No branch anywhere in the solve reads what the field holds.** A field at
rest runs the same passes as a field that is moving. A field with no source
runs the same passes as a field with one. A plane whose ground carries nothing
runs the same passes as open ground. The absence of a source is not a case: it
is the ordinary value.[^5]

**The constant is a reach, not a budget.** It is the number of cells that one
solve adds to the field. No measurement chose it, and no measurement may be
put in its place, because every cost figure in this project is derived and a
blocker holds the reason.[^9]

### D2. The pass count is observable, because a test must read it

The field reports the passes it has run. A test asserts that the count is the
constant times the number of solves, for a field at rest, a field that
saturates, a field with no source, and ground that carries nothing.

This is stated as a decision and not left to the implementation, because the
alternative was tried and does not work. Comparing two runs cannot see a solve
that stopped early, and the register records the case.[^8]

### D3. A pass writes disjoint runs, and it reads the whole of the other plane

Each thread fills one run of the output and reads the input wherever the
stencil points, including outside its own run. A cell is named by its index
and never by the thread that filled it.[^10] [^3]

**A neighbour read is never clipped to the run.** Clipping is the optimisation
that looks safe and is not: the run boundary follows the thread count, so the
field follows it too. The perturbed build clips, and the thread-count
assertion over the field is what fails.

### D4. The plane carries its own state between solves, and it is not a summary

The plane is not part of the level of detail pyramid. It does not claim that a
cell equals the combination of the tiles beneath it, no consumer may read it
as such, and the pyramid does not rebuild it.

**A consumer reads the level the plane is defined on and gets an answer about
that level.**[^11] The plane is derived in the sense that it can be rebuilt
from the sources by solving again from nothing. It is not derived in the sense
the pyramid means, because rebuilding it costs the ticks that produced it.

Until the open row is settled, this decision states the boundary and does not
claim that the record which owns level 0 permits it.[^7]

## Consequences

**A solve costs the same on every tick, and it costs it whether or not the
field is doing anything.** That is the price of the property. The cadence at
which solves are taken is a separate choice, and a backlog item holds it.[^12]

**The field lags the world.** A consumer reads what the last solve produced,
and a source raised now reaches a distant cell over several ticks. Nothing
here removes that lag and no consumer may assume it away.

**A saved world must save the plane, or reproduce the ticks that built it.**
The plane cannot be recomputed from the current sources alone.

**The determinism suite gains a test that is not a comparison of two runs.**
The pass count is asserted directly, because no comparison of runs can see it.

**A future contributor who wants the field to converge faster must add passes,
not a residual.** That is the trade this record refuses to leave open.

## References

[^1]: ADR-0060, an influence map is stored as a shared basis, not one plane per faction. `docs/adrs/draft/adr-0060-an-influence-map-is-stored-as-a-shared-basis.md`
[^2]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^3]: ADR-0004, iteration order is explicit, decisions D1 and D2. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^4]: Influence maps, section 6.5. `docs/research/reports/09-influence-maps.md`
[^5]: Decisions register, DEC-041. `docs/DECISIONS.md`
[^6]: ADR-0022, level 0 is the only truth, and every level above it is derived, decisions D1 and D2. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
[^7]: Decisions register, DEC-067. `docs/DECISIONS.md`
[^8]: Findings register, FND-160. `docs/FINDINGS.md`
[^9]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^10]: ADR-0009, parallel stages write disjoint outputs, because the memory model is weak. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
[^11]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D4. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
[^12]: Backlog item 0169. `docs/backlog/proposed/0169-choose-the-cadence-of-the-influence-solve.md`
