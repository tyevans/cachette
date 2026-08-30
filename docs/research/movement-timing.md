# Movement timing: a crossing time needs the terrain multiplier

This note records a timing check of the movement calibration. It states what
the project believed, what the check found, and one question that the check
opened.

## Context

Cachette moves a unit one hex tile at a time. A unit accumulates progress each
tick. When the progress reaches the step cost of the next tile, the unit steps
and spends that progress.

Two quantities set the time to cross a chokepoint. **Dwell** is the number of
ticks a unit stays on a tile. **Capacity** is the number of units a tile
holds. The engine runs at 10 ticks for each second.

The throughput law is `throughput = capacity / dwell`. The dwell comes from
the tile beyond the chokepoint, not from the chokepoint itself. The time for a
formation of `strength` units to pass is therefore
`strength * dwell / capacity` ticks.[^1]

The dwell is `ceil(step_cost_of_exit / speed)`. The step cost carries a
terrain multiplier. A mountain tile costs more than ordinary ground.

## What the project believed

The project believed that a dwell of 2 ticks and a capacity of 8 units gives a
crossing of 12.5 seconds for a formation of 1,000 units.

## What the check found

That belief is wrong. The arithmetic that produced it used the ordinary ground
step cost for a mountain exit tile. It omitted the terrain multiplier.

With the multiplier applied, a dwell-2 baseline and a capacity of 8 give a
mountain crossing of about **50 seconds**, not 12.5 seconds.

The combination that meets the target is a **dwell-2 baseline with a
capacity-16 crossing**. The check measured **12.9 seconds** for that
combination.

## An unresolved difference of 4 ticks

The closed-form law gives 125 ticks for 1,000 units at dwell 2 and capacity
16, which is 12.5 seconds.[^1] The check measured 12.9 seconds, which is 129
ticks.

The difference is 4 ticks. The likely cause is that the closed-form law counts
the steady state only. It omits the ticks that the leading rank spends to
enter the chokepoint and the ticks that the last rank spends to clear it.

**This note does not verify that cause.** The difference is small and it does
not change the choice of capacity 16. A later check must resolve it before any
record states a crossing time as an exact figure.

## The mountain multiplier is not recorded

No record states the terrain multiplier for a mountain tile. The 50-second
result implies a multiplier of 2 against ordinary ground, at capacity 8. That
value is inferred from the result. It is not a decision that the project has
made.

## What this note could not do

The check could not compute the per-frame cost of the movement pass. No
measurement exists on the target platform. A blocker already records that
gap.[^2] Every cost figure in this project is derived, not measured.

## What follows

1. A crossing time is a function of three quantities, not two. Capacity,
   dwell, and the terrain multiplier all enter it. An arithmetic check that
   omits one of the three gives a confident wrong answer.
2. The dwell-2 baseline with a capacity-16 crossing stands. The recalibration
   that a research report recommends is unchanged by this check.[^3]
3. A 50-second mountain crossing is the accepted outcome. The register records
   the owner's acceptance.[^4]
4. The terrain multiplier needs a recorded value, or a recorded rule that
   supplies one.

## The open question

The check found that nobody has decided **whether a tile crossing time is
content-configurable for each terrain type, or fixed by the engine**.

The two options differ in where the multiplier lives. A content-configurable
multiplier sits in the terrain table that content authors write. A fixed
multiplier sits in engine code.

This note leans toward content-configurable, because the terrain capacity
table is already content and the two values describe the same tile. The
question is genuinely open. The decisions register holds it.[^5]

## References

[^1]: Report 17, Group Spatial Dynamics, sections 13.2 and 15.4. `docs/research/reports/17-group-spatial-dynamics.md`
[^2]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^3]: Report 17, Group Spatial Dynamics, section 15.4. `docs/research/reports/17-group-spatial-dynamics.md`
[^4]: Decisions register, DEC-008. `docs/DECISIONS.md`
[^5]: Decisions register, DEC-017. `docs/DECISIONS.md`
