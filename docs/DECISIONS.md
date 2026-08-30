# Open Decisions (Register)

This document is a **register**. It lists choices that are open, the options,
and a recommendation.

A decision needs **judgement**. The options are known and work can continue
under a stated assumption. Compare `BLOCKERS.md`, which lists work that is
stopped for want of information.

Numbers are permanent. Never reuse one. A closed decision keeps its row, with
the outcome recorded.

When a decision closes, and it corrected something the project believed,
record the correction in `FINDINGS.md` as well.

## Open

### DEC-001 — The commodity split

Two reports set different ceilings, and they bound different things.

| Report | Ceiling | Reason |
|---|---|---|
| Entity economy | 64 | A presence mask is one `u64`. 64 `i64` values fill exactly 8 cache lines. |
| Trade and flow | 16, hard limit 32 | Cache residency during the flow solve. |
| Individual agency | 4 to 8 | What one individual can carry. |

**Recommendation:** 64 may exist, 16 take part in the transport solve, the
remainder stay local to a settlement, and an individual carries 8. The three
limits are compatible because they bound existence, participation and carriage
separately.

**Assumption in the meantime:** the recommendation above.

### DEC-002 — Do units make individual decisions?

The needs report concluded that units do not decide, because a decision cost
400 nanoseconds and one million decisions would take four times the tick
budget.

The agency report measured 4.1 nanoseconds. The gathers are sequential, not
random, because units are sorted by tile index and the fields are level-1
planes that stay in cache.

**This is now a design choice, not a budget one.**

**Recommendation:** both tiers. Individuals choose where to go; cohorts choose
what to buy. Cost is 0.18 core-ms. The project owner has asked for individual
experiences, and this delivers them.

### DEC-003 — Do dead characters keep relation edges?

Retaining them costs 531 MB at 100,000 living characters and 1.39 GB at the
ceiling. Dropping them loses the ability to reason about a dead person's
former ties.

**Recommendation:** drop them. The character report notes this is how
expensive the question is to answer wrongly.

### DEC-004 — One fog layer or two

The fog report specifies explored and visible as separate layers, and asks
whether both are needed.

**Recommendation:** unresolved. It depends on whether the game shows explored
terrain differently from currently visible terrain.

### DEC-005 — Does the military influence plane need terrain conductance?

With conductance the solve costs 150 microseconds. Without it, 12
microseconds. The difference is whether influence flows around mountains or
through them.

**Recommendation:** include it. Twelve times a small number is still a small
number, and influence that ignores terrain will look wrong.

### DEC-006 — Simulated or procedural weather

Procedural weather is a deterministic function of position, tick and seed:
zero storage, no update cost, perfectly reproducible, but no feedback.
Simulated weather supports orographic rain shadow and fire-driven weather at
real cost.

**Recommendation:** procedural base with simulated perturbation, if weather is
built at all. It is not yet in scope.

### DEC-007 — Retained or transient event log

The log is currently transient. Retention costs 3.2 MB per frame, which is
11.5 GB per minute. Retention would buy rollback, time travel and audit.

**Recommendation:** stay transient. Events are already serialisable and the
apply step is pure, so retention remains additive.

### DEC-008 — Is a 50-second mountain crossing acceptable?

The approved calibration puts an ordinary crossing at 12.5 seconds and a
mountain crossing at 50 seconds. The project owner rejected 50 seconds as the
ordinary case. The recalibration relocates it to mountains.

**Recommendation:** accept. A mountain pass should be a serious obstacle.

### DEC-012 — Does a product record cite a decision record?

**Decided: no.** Recorded here because the reasoning is easy to lose.

A product record states a need. A decision record answers to a constraint. A
product direction changes more often than a constraint does, so a citation
from a decision record to a product record would place changing material
inside a historical document, which the scope rule forbids.

The join runs the other way and through one place only: a refined backlog
item names both the record that governs it and the product record it serves.
A check enforces that a product record contains no decision record citation.

**Revisit if.** The backlog stops being the only route from a need to the
work, or a reader cannot answer "which need does this record serve" and needs
to.

### DEC-013 — Is a tile crossing time content-configurable, or fixed by the engine?

A crossing time depends on the terrain multiplier that scales the step cost of
a tile. No record states where that multiplier lives.

**Option A. Content-configurable for each terrain type.** The multiplier sits
in the terrain table beside the terrain capacity. A content author tunes a
crossing without an engine change.

**Option B. Fixed by the engine.** The multiplier sits in engine code. The
engine can then bound the dwell range at compile time.

**Recommendation:** content-configurable. The terrain capacity table is
already content, and the capacity and the multiplier describe the same tile.
Splitting them across content and code would put one crossing's two levers in
two places. Option B buys a compile-time bound that a validated range in
content also buys.

**Assumption in the meantime:** content-configurable.

**Related.** The mountain multiplier has no recorded value. The accepted
50-second mountain crossing implies a multiplier of 2 against ordinary
ground.[^DEC1] Whichever option wins, that value needs recording.

[^DEC1]: See DEC-008 in this document, and the movement timing note, `docs/research/movement-timing.md`.

### DEC-013 — Which toolchain version does the project pin?

**Open.** The pin is currently the version the development machine had. That
is not a reason.

The record scope rule forbids a version in a record body, so this belongs
here and not in a record. State the property the project needs from the
toolchain, then pin the lowest version that provides it.

**Recommendation.** Decide the property first. The float ban already depends
on toolchain behaviour, because the reassociating methods do not resolve on
the current pin, and a later toolchain may make them resolvable and therefore
bannable by lint rather than by script.

### DEC-014 — Which hash does the golden state test use?

**Open.** The scaffolding chose FNV-1a. Nothing has ratified it.

This choice is load-bearing for determinism. The golden file is written by
the hash, so changing the hash invalidates every stored hash. It is cheap to
change now and expensive later, which is the shape of a decision that earns a
record once it is settled.

**Recommendation.** Confirm FNV-1a or replace it before the first golden file
is committed for real content. State the requirement the hash must meet:
exact, order-sensitive, and stable across the platforms the project builds
on.

### DEC-015 — The Python mutation gate is off

**Decided, and reversible.** The gate was removed rather than left failing,
which the definition of done requires. The Python package only re-exports the
compiled module, so no mutant is covered and the tool exits non-zero.

Turn it on when the Python package holds logic of its own. The testing policy
says how.

### DEC-016 — Type checking uses mypy, not pyright

**Decided.** Chosen to avoid a second language runtime in continuous
integration. Recorded because it was made in passing and no record holds it.

## Decisions to apply at merge

These are mechanical. They do not need judgement, but they must not be
forgotten.

### DEC-009 — Renumber the colliding decision ranges

Reports 10, 11 and 12 all claim D51. Report 15 overlaps report 14 at D90 to
D95. Every decision number becomes local to its record, so the collision
disappears when the records are written.

### DEC-010 — The needs report must adopt the agency report's decision cost

The needs report's cohort decision line is 16.00 core-ms and is 92 percent of
its subsystem. Corrected, it is under 0.05 core-ms. See DEC-002.

### DEC-011 — Re-run the vector storage argument

The vector report computed against a stale copy of the character report. It
used 8-byte edges at mean degree 8, giving 33.6 MB at the ceiling. The real
figure is 168 MB. The storage argument for vectors is stronger than the report
concluded, and it called that argument its weakest.

## Closed

None yet.
