---
id: 0224
title: Answer and command a set of mass-tier entities
status: proposed
created: 2026-09-02
implements: []
changes: []
creates: []
serves: []
blocked-by: []
---

## Why

**The boundary offers three reads that each name one entity of a mass-tier
shape, and a set-valued command that cannot carry a value for each member.**

The soldier shape and the settlement shape both declare the mass tier, and the
soldier's declaration carries the reason: a soldier is one of a million, so no
caller walks the population.

Against that, the engine answers the tile of one soldier, the positions of one
site and the preference of one site. A caller that wants any of those for a set
calls once for each member. The agent server exposes the first as a tool that
reads one unit, and the type stub for the gather event columns tells a reader to
take a value from the unit column and hand it back to the per-unit read.

**A test already pays this four times for each site.** The thread count test for
the positions sets a preference one site at a time, because the command takes
one target for the whole set and the test wants a different target for each
site. It then reads the positions of each site twice. Eight sites cost
twenty-four crossings, and the count grows with the sites.

That test is not badly written. It is the only way to express what it needs
through the surface that exists, which is what a missing capability looks like
from the caller's side. FND-215 holds the measurement, and FND-147 holds the
precedent: a rule that forbids a shape and offers no alternative loses to the
absence of the read every time.[^1]

Two capabilities are missing, and they are different work.

**A set-valued read.** One call answers for a set and returns one column for
each field, in the order of the set.

**A command that carries one value for each member.** The preference command
takes a set and one target. A caller that wants a different target for each
member has no form to write, so it sends one command for each member.

## What is missing before this is refined

- The impact review. ADR-0040 and ADR-0043 govern it, and the review of ADR-0040
  returned that record for an amendment that names these gaps.[^2]
- Whether the set-valued read belongs on the world or waits for the selector,
  which is the destination the owner named for reaching a set.
- Whether a per-member value is a parallel column argument or a different verb.
  A parallel column has to state what happens when the lengths disagree.
- Whether the per-entity reads are removed or kept. Keeping one is a second
  declaration site for one question, and the tier record forbids the interface
  naming one entity of a mass shape at all.
- What the agent server's one-unit tool becomes.
- Whether the stub's instruction to hand a unit identity back to the per-unit
  read is removed in the same change.

## Done when

Stated when the item is refined.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Findings register, FND-215 and FND-147. `docs/FINDINGS.md`
[^2]: Review 0223, the control plane record. `docs/reviews/0223-the-control-plane-record.md`
