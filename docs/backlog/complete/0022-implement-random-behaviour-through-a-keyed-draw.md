---
id: 0022
title: Implement random behaviour through a keyed draw
status: complete
created: 2026-08-30
---

A soldier chooses a neighbour tile to attempt each tick. The choice comes
from the counter-based generator, keyed on the system, the frame, the entity
and the draw index, as ADR-0003 requires.

Registry row 0064 says a unit chooses by scoring a small fixed option set. A
uniform draw over six neighbours is the degenerate case of that claim. Decide
at refinement whether this item implements row 0064 or defers it, and record
the answer.

Refine this at sprint 3 planning.

## Outcome

**Closed as already done. The work landed under other items.** An audit read the
code on 5 September 2026.

**The movement draw is keyed as the determinism record requires.** It takes the
system, the frame, the entity and the draw index, from a counter-based
generator. The comment at the call site states the key.[^1]

**No thread-local random state exists.** The generator is a pure function of the
seed and the key.[^2]

**The record that governs the scoring pass is accepted.** ADR-0064 states that a
unit chooses by scoring a small fixed option set, and the movement code cites
it. The scoring pass exists as its own module.[^3]

## References

[^1]: The movement draw and its key. `crates/cachette-core/src/world.rs`
[^2]: The counter-based generator and its system constants. `crates/cachette-core/src/rng.rs`
[^3]: ADR-0064, a unit chooses by scoring a small fixed option set. `docs/adrs/accepted/adr-0064-a-unit-chooses-by-scoring-a-small-fixed-option-set.md`
