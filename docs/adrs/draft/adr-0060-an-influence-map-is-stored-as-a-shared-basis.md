# ADR-0060: An influence map is stored as a shared basis, not one plane per faction

## Context

An influence field carries the reach of a faction across the world. A source
raises the field at one place, a solve spreads it, and a consumer reads the
cell it stands in. It is how a decision made by one actor reaches every unit
without any unit following a link to that actor.[^1]

The obvious storage is a plane of cells for each concern, for each faction.
That shape multiplies the world by the faction count, and it multiplies it
again by the number of concerns anybody thinks of. The project holds the same
shape elsewhere and rejects it: a faction is a bit in a mask rather than an
index that multiplies the tile columns.[^2]

**Most of an influence system does not depend on the faction.** How freely the
ground carries influence is a property of the ground. The value of a place is
a property of the place. What depends on the faction is what that faction
itself puts into the world, and the research that supports this record found
that the per-faction part is a small share of the whole.[^3]

### The forces

**A plane for each faction is the storage that grows.** The world grows with
its extent. A per-faction plane grows with the extent and with the faction
count together, and the faction count is the term the project has least
control over.

**A shared plane read by every faction is read while it is written.** A value
that every faction reads must not change while one faction is being solved, or
the answer depends on which faction went first. Sharing therefore constrains
the frame position of the update as well as the storage.

**A narrow cell is not free.** It is smaller and it fits more lanes in a
vector register. It also holds fewer distinct values, and an iterated solve
narrows into it on every pass, so what a narrow cell costs is the tail of the
field rather than its head. The project measured this and recorded the
correction.[^4]

**The project already has one fixed-point scale, and a second scale is a
liability.** The scale exists so that a position and a stat share one
representation through many arithmetic stages.[^5] A value that enters no such
pipeline gains nothing from it and pays its width.

**A transposed store was considered and rejected.** In that shape a cell holds
one value for each faction, side by side. It answers "what does every faction
hold here" in one read. It loses on typical size against separate planes, and
it moves the whole cell of a cache line to change one faction's value. The
research states the comparison.[^3]

## Decision

### D1. A value that does not depend on the faction is stored once

The field holds one plane for each faction and one conductance plane that
every faction shares. Nothing that is a property of the ground is stored a
second time under a faction's name.

**The shared plane is read-only for the whole of a solve.** No faction writes
it, and the rule that fills it runs at a moment when nothing is solving. That
is what makes the sharing safe under a weak memory model without an
atomic.[^6]

**The rule that fills the shared plane is not this record.** The conductance
of a cell follows the ground it covers, and the project has already decided
that the influence plane carries terrain conductance so that influence flows
around ground which resists it.[^7] Which ground resists, and by how much, is
a content table and an open choice.[^8] The solve reads a conductance of any
value, so replacing the rule does not touch the solve.

### D2. A cell is a narrow unsigned integer against a fixed reference, not the project-wide scale

A cell holds an unsigned integer. Its ceiling means one reference unit of
influence, so the cell is an unsigned fixed-point fraction against a fixed
reference. It is not the project-wide fixed-point scale, and the reason is
that an influence cell enters no pipeline: one kernel produces it and a
comparison consumes it.

**The width is set by the solve, not by the reader.** Every consumer compares
two influence values and no consumer reads an absolute magnitude, so a reader
needs few distinct values. The solve needs more, because it narrows into the
cell on every pass and the narrowing truncates. A width chosen for the reader
alone destroys the far field, and the register records the measurement that
found it.[^4]

**Reject a per-plane exponent.** Scaling a plane by its own maximum makes the
stored values depend on the history of that maximum, which is a determinism
hazard for no gain.

### D3. The combine at a cell is saturating addition at the ceiling

Two contributions at one cell combine by saturating unsigned addition. That
operation is exactly associative and commutative, and its identity is zero, so
a fold over a set of contributions gives one answer whatever the order.[^9]

It has no inverse above the saturation point, so a cell cannot be repaired by
removing a contribution.[^10] The repair path is the solve itself, which runs
on a schedule rather than on demand.

### D4. The write half of a pass holds one plane, and every faction reuses it

A relaxation pass reads one plane and writes another. The plane it writes is
one scratch buffer that the field owns and reuses for every faction. A pass
relaxes one plane into the scratch and copies the scratch back, then takes the
next faction. The storage of the write half therefore does not grow with the
faction count.

**The copy is the price, and it is the trade this decision makes.** A private
second plane for each faction would remove the copy and would make the write
half as large as the field itself. The write half would then be a share of the
whole system that no consumer ever reads. Nothing has priced the copy against
that storage, because no run separates the two, so the decision takes the
smaller storage and says which figure would reopen it.[^11]

**A second ground for the trade now exists, and this decision did not weigh
it.** The paragraph above weighs the copy against the storage. A measurement
since taken weighs a third quantity that neither side of that sentence names:
the cost of opening a parallel section. One scratch plane means a pass relaxes
one faction at a time, so a solve opens a thread scope for each faction in each
pass. Giving each faction its own plane would let one scope serve them all.
The register holds the figures and the machine they came from.[^12]

**The reopening condition this decision states is not the one that was met.**
It asks for a run that separates the copy from the storage, and no run has. The
spawn cost is a different quantity, and it was measured on a development
machine rather than on the target platform, so it does not close the question
either. This paragraph records that the ground has widened. It does not change
the decision, and the decision stands as written until a run on the target
platform prices the spawn against the storage.[^11]

**A plane of one faction is read only by that faction.** That is what makes
one scratch enough: relaxing one plane before another changes neither of them,
so the order over the factions cannot change the result. The order is fixed
anyway, in ascending faction identifier, because a reader should not have to
prove that to know what a pass returns.[^13]

## Consequences

**The storage of the system grows with the extent, and with the faction count
only in the part that a faction owns.** Adding a shared concern costs one
plane, whatever the faction count.

**A consumer cannot ask what every faction holds at one cell in one read.**
The transposed store answered that in one read and this shape does not. The
research proposes a shared summary that answers the useful part of that
question, and nothing here builds it.[^3]

**The shared plane fixes where the update may run.** No faction may write the
shared plane during a solve, so the rule that fills it runs outside the solve.
Today the ground fixes it once, and nothing writes it after a world is built.

**A cell holds a bounded number of distinct values, and a source cannot be
recovered from the field.** Saturation is a valid combine and it is not a
group. A consumer that needed to remove one contributor's share would have to
solve again without it.

**The narrow cell does not carry an absolute magnitude between worlds.** The
reference unit is a constant of the build. Two worlds that chose different
reference units hold values that must not be compared.

## References

[^1]: Decisions register, DEC-040. `docs/DECISIONS.md`
[^2]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D3. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
[^3]: Influence maps, sections 5.2, 5.4 and 5.7. `docs/research/reports/09-influence-maps.md`
[^4]: Findings register, FND-159. `docs/FINDINGS.md`
[^5]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^6]: ADR-0009, parallel stages write disjoint outputs, because the memory model is weak. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
[^7]: Decisions register, DEC-005. `docs/DECISIONS.md`
[^8]: Decisions register, DEC-017. `docs/DECISIONS.md`
[^9]: ADR-0023, an aggregate combines exactly, in any order, decisions D1 and D2. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
[^10]: ADR-0023, an aggregate combines exactly, in any order, decision D4. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
[^11]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^12]: Findings register, FND-300. `docs/FINDINGS.md`
[^13]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
