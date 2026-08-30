# Merge Notes for ADR-0001

This document records conflicts between the background reports, decisions the
project owner and the session lead made, and questions that remain open. Read
it before you merge any proposed decision block into the decision record.

The reports were written by agents that ran at the same time. An agent could
not see the work of another agent. Conflicts are expected.

Last updated after reports 10, 13 and 14 landed. Reports 15 and 16 are still
in progress.

## 1. Decision number ranges

Two early reports collided because they were told to continue from the end of
the decision record. Later agents received assigned ranges.

| Report | Range | Status |
|---|---|---|
| Entity economy and modifiers (12) | D51 to D59 | **Collides with report 11** |
| Resource and trade flow (11) | D51 to D60 | **Collides with report 12** |
| Crowd and movement (10) | proposes a new D51 | **Collides with both** |
| Character graph (14) | D70 to D89 | Assigned. No collision. |
| Needs and economy (15) | D90 to D109 | Assigned. No collision. |
| Individual agency (16) | D110 to D129 | Assigned. No collision. |

**The rewrite renumbers everything from D1. Keep a mapping table so each
report stays traceable.**

Open-question numbers also collide. Check every report before you accept one.

**Rule for future work:** always give an agent an assigned range.

## 2. Owner decisions

These are settled. Do not re-open them.

1. **Tile capacity is 8 units.** Stored as `u8`, with capacity as a
   data-driven parameter, not a type constraint. The array serves three uses:
   occupancy, capacity check, and the density field. A `u8` leaves 31 times
   headroom for transient overflow during sort-then-admit.
2. **A unit is an individual soldier, not a formation.** The owner wants
   individual units with individual experiences. Upkeep is therefore per unit
   at about 1 million entities. This is the expensive branch that report 12
   flagged as a 1 to 2 order of magnitude swing.
3. **Aggregation layers are how the project scales to larger groups.** A
   larger group is a coarser view, not a new entity type.

## 3. Lead decisions

1. **Derived state is not logged.** The event log holds commands and
   discontinuous facts. A derived field update is recomputed during replay.
   This resolves report 11's question about per-arc against bulk flux events:
   neither. The cost is that you cannot answer "why did this stock change"
   from the log alone without running the solver again.
2. **Positions are tile-discrete.** No sub-tile coordinate. From report 10.
3. **The no-per-entity-loop rule is a function of N, not a principle.** The
   character tier may expose per-entity Python access. The mass tiers may not.
   Report 14 sets the ceiling at 262,144 and enforces it by a declared tier on
   the class, not a runtime cardinality check.

## 4. Conflict: the commodity ceiling

| Report | Ceiling | Reason |
|---|---|---|
| Entity economy (12) | 64 | A presence mask is one `u64`. 64 `i64` values fill exactly 8 cache lines. |
| Resource and trade flow (11) | 16, hard limit 32 | Cache residency during the flow solve. |

The two limits bound different things. One bounds how many commodities can
**exist**. The other bounds how many can be **solved for**.

Probable resolution: 64 may exist, 16 take part in the transport solve, the
remainder stay local to a settlement. **Not yet decided.**

Report 16 adds a third limit: an individual carries only 4 to 8 commodities.
That is a third bound and it does not conflict.

## 5. Conflict: hex geometry cuts both ways

Both findings are correct. They concern different operations.

| Report | Finding |
|---|---|
| Field operator algebra (13) | Hex diffusion is **better**. Directional error at a 6-cell feature: hex 7-point 0.035%, best square 9-point 0.14%, square 5-point 4.2%. Hex beats the best square stencil by 4 times with 2 fewer taps, and takes no timestep penalty. |
| Crowd and movement (10) | Hex path metric is **worse**. A 6-connected lattice has 15.5% worst-case path error against 8.2% for an 8-connected square grid. Bounded at L0 by chunk size; systematic at L1. |

**Record both.** Diffusion likes hex. Distance does not. If this is not
written down it will return as a mystery defect.

## 6. Defects found in the superseded draft

Report 10 found two. Correct them; do not carry them forward.

1. **The D45 separation term is not achievable under D15.** It assumes
   per-tile occupancy, but D15 specifies a block-level bridge, so each of the
   7 neighbour reads becomes a search of up to 256 entries. The fix is a dense
   `u8` per-tile count at 16.8 MB, which also serves as the capacity check and
   the density field.
2. **D44 conflates cost anisotropy with metric error.** See section 5.

## 7. Resolved: the integer eikonal question

A citation check found **no published literature** on integer or fixed-point
eikonal solvers. Report 10 then derived that the solver works exactly in
integer arithmetic: the only non-linearity is `isqrt`, which is exact; the
update is monotone, so it terminates; sweep order is a compile-time constant.
The fixed-point form is **more** reproducible than the float form.

**Record the distinction. Absent literature is not impossibility.** The lead
relayed the first finding as though it implied the second. It did not.

Borgefors (1986) chamfer transforms remain the fallback that proved
unnecessary. Dial (1969) stays in the L0 path.

## 8. Legal constraint: the Nemesis patent

Report 14 records United States Patent 10,926,179 B2, held by Warner Bros.,
granted 2021, running to 2036. It covers nemesis characters, nemesis forts,
social vendettas, and followers.

The owner has asked for promotion of units into a named character tier, which
approaches this area. The general pattern is much older and broader — Dwarf
Fortress generated emergent historical figures long before the filing.

**This is a factual record, not legal advice and not an infringement
assessment.** Seek counsel if the feature grows toward the claimed
combination. Report 14 cites the patent as a claimed method, explicitly not
as a shipped implementation.

## 9. Accepted trade: the Alabama paradox

Largest-remainder apportionment, adopted for the conserving `transfer` verb,
is subject to the **Alabama paradox** — adding a unit to the total can reduce
an individual share. Balinski and Young (1982) prove no method avoids this
while satisfying quota.

This is a visible gameplay anomaly, not a defect to fix. Players may report it
as a bug. Record it as an accepted trade with its citation.

## 10. Needs owner confirmation

**A promoted soldier gets no parents.** Report 14 rejected invented ancestry
on arithmetic: it costs `2^d - 1` dead rows per promotion, which is 1.8 GB at
depth 4 over 500 years. A promoted soldier founds a new house and his kinship
to everyone is exactly zero.

**The consequence: he cannot inherit a title by blood.** Office by appointment
only. His children inherit from him normally.

This is thematically right for a risen commoner, but it is a rule the owner
should confirm rather than inherit by default.

## 11. Questions for the project owner

1. **Name three archetypes you expect to exist.** Unanswered since the first
   round. It decides about 2,000 lines of code and the zero-copy story.
2. **Confirm the grid extent and the world shape.**
3. **How many settlements?** Every storage figure in report 12 scales with it.
4. **What fraction of tiles carry an upgrade?** Report 12 estimated 2.7%.
5. **What is the commodity split?** See section 4.
6. **Target living character population.** Report 14 notes this is the real
   control on promotion, not the deeds threshold.
7. **Do formations exist, and what is the depth cap?** Report 14 open
   question 49.
8. **Confirm the promoted-soldier lineage rule.** See section 10.

Resolved since the last revision: upkeep granularity, tile capacity, and
whether formations are entities.

## 12. Soft citations to verify before publication

The project's citation verifiers have repeatedly found that game
implementation claims are community-wiki only, with no developer
documentation. Treat every such claim as documented behaviour, not
implementation.

- Victoria 2 formulas, Dwarf Fortress needs, The Sims decay rates, Anno tiers.
- Planetary Annihilation pathfinding — no paper exists. Cite the 2013
  developer video and press coverage.
- Crusader Kings succession internals.
- RimWorld `StatPart` ordering; Civilization VI modifiers.
- Anno and The Settlers agent counts; Offworld Trading Company algorithms.
- Leontief's *Structure of American Economy* is **1941** first edition. 1951
  is the revised second edition.

Verified and safe to cite: Hawkins and Simon (1949) in *Econometrica*, with
the equivalence to a non-negative inverse and to spectral radius below 1; the
Neumann truncation bound; Arrow, Chenery, Minhas and Solow (1961) on constant
elasticity of substitution; Tesfatsion (2006); Balinski and Young (1982).

### Corrections found by verification (report 16 pass)

- **Mike Lewis, not Mark Lewis.** The session lead wrote the wrong name in the
  agent brief. "Escaping the Grid" is about influence mapping, not utility.
  His utility chapter is a different one.
- **There is no Dill "needs-based AI" chapter.** Cite Zubek, *Game Programming
  Gems 8*, 2010, verified from the author's own draft.
- ***Assignment Problems*** is Burkard, Dell'Amico and Martello, SIAM 2009.
  **Not** Toth and Martello, who wrote *Knapsack Problems*.
- Versu is 2014, volume 6 number 2. Not 2013.
- Tarn Adams is an **editor** of *Procedural Generation in Game Design*, not a
  chapter author. The Dwarf Fortress chapter does not exist.
- **A usable primary source exists for The Sims.** Forbus and Wright, "Some
  notes on programming objects in The Sims", Northwestern University course
  notes, 2001, hosted on the author's institutional page and verified live.
  Course notes, not peer-reviewed, but author-hosted and citable. **No Will
  Wright GDC talk on smart objects exists — do not cite one.**
- RimWorld work priorities: no primary developer source. Community wiki only.

### The pattern is now conclusive

Eight subsystems across seven games are community-wiki only, with no developer
documentation: Victoria 2 formulas, Dwarf Fortress needs, The Sims decay
rates, Anno tiers, Planetary Annihilation pathfinding, Crusader Kings
succession, RimWorld work priorities, and the Nemesis system.

Every verification pass has found the same result.

**State this as a project finding.** The games that made these ideas familiar
have not documented their implementations. Citations must come from operations
research, numerical analysis, and academic simulation. Cite a game only for
observed behaviour, never for implementation.

## 13. Running per-tick budget

| Subsystem | Cost | Source |
|---|---|---|
| Movement | 1.9 to 3.8 wall-ms (12 cores) | Report 10 |
| Trade | 1.1 wall-ms | Report 11 |
| Economy | 0.4 to 0.6 wall-ms | Report 12 |
| Field layer, nine fields | 0.32 to 0.71 core-ms | Report 13 |
| Character tier, Rust | under 0.14 core-ms | Report 14 |
| Character tier, Python | 0.17 ms at 10k, 1.7 ms at 100k | Report 14 |

Report 13 recommends renaming the existing influence budget line rather than
adding one — the whole field layer fits inside it.

Nothing approaches the frame budget yet. **Every figure is derived, not
measured.** The research agenda flags benchmarking on the target platform as
blocking most conclusions.

## 14. Adjacent fields the project reinvented

The research agenda found three places where established literature already
holds results this project derived by hand. Reconcile before building.

- **Sparse volumetric structures in graphics.** Report 08 independently
  reinvented much of OpenVDB and NanoVDB.
- **Molecular dynamics neighbour handling.** Cell lists, Verlet lists, and
  periodic spatial reordering under a reproducibility requirement. This is the
  sorted-by-tile invariant, solved by people who also cannot tolerate
  non-determinism.
- **Incremental view maintenance in databases.** The dirty pyramid is a
  materialised view. The group-with-inverse rule that report 02 derived by
  hand is the standard maintenance condition.

---

# Addendum: findings after reports 15 to 18

All research threads are now closed. Nineteen background documents exist.
This addendum records what changed after the main notes were written.

## A. Number ranges, final

| Report | Range | Note |
|---|---|---|
| 11, 12 | D51 to D60 | **Collide with each other.** Renumber both. |
| 10 | proposes a new D51 | **Collides with both.** |
| 14 | D70 to D95 | Clean. Extended from D89 with permission. |
| 15 | D90 to D107 | **Overlaps report 14 at D90 to D95.** Check at merge. |
| 16 | D110 to D129 | Clean. |
| 17 | D130 to D149, D170 to D179 | Clean after renumbering. |
| 18 | D150 to D169 | Clean. Full range used. |

Report 17 first took D150, D152 and D154, which collided with report 18. The
collision was found by search and the agent renumbered to D170 to D179.
**Report 17 asked before taking numbers outside its range. That is the correct
behaviour and it is what caught the collision.**

## B. Report 16 invalidates the largest cost in report 15

Report 15 costs an individual decision at 400 ns and concludes that decisions
at 1 million entities cost 400 core-ms, four times the whole tick budget.

**Report 16 measures 4.1 ns, not 400 ns.** Report 15 assumed random gathers.
The gathers are sequential, because units are sorted by tile index and the
fields are L1 planes — eight `u8` planes is 512 KB, which stays in L2 with
about 15 times reuse for each cell.

**Applying the correction to report 15's own cohort decision line drops it
from 16.00 core-ms to under 0.05 core-ms.** That line is 92 percent of its
subsystem.

**Consequence: whether units make individual decisions is now a design choice,
not a budget one.** Report 16 recommends both tiers — individuals choose where
to go, cohorts choose what to buy — at 0.18 core-ms.

Report 15 must adopt report 16's formulation at merge.

## C. Report 18 read a stale copy of report 14

Report 18 computed against report 14's earlier 8-byte edges at mean degree 8,
giving 33.6 MB at the character ceiling and a net saving of −2.3 MB. It
therefore called the storage argument its weakest.

Report 14's revision has 20-byte effective edges with a reverse index, giving
**168 MB** at the ceiling. **The storage argument for vectors is materially
stronger than report 18 concluded. Re-run it at merge.**

Both reports independently corrected the session lead's claim that opinion
storage is quadratic. Sparse edges at a fixed mean degree grow linearly. The
out-degree cap is the mechanism, and it must be enforced, not assumed.

## D. A hard incompatibility between tile scale and crossing time

Report 17 worked the movement calibration through parametrically and found a
contradiction, not a preference.

| Tile scale | March rate | Result |
|---|---|---|
| 80 m | 24 km/day | Consistent. Dwell 2, crossing capacity 16, world about 330 km across. |
| 1 km | 24 km/day | Needs dwell 25. Holding a 12.5-second crossing then needs capacity 200 on a bridge tile, beyond `u8` headroom and visually absurd. |

**A continental tile scale and a 12.5-second crossing cannot both hold.**

The owner must choose one of three:
1. A regional world of roughly 330 km.
2. A slower crossing.
3. A shorter game day, which forces a re-bake of every per-tick rate,
   including needs decay.

**This constrains the world extent question rather than depending on it.** The
question is no longer how large a grid the engine can afford. It is what a
tile represents, and what that makes the world.

All movement constants in report 17 are parametric in tile scale, so they
resolve when the owner answers.

## E. Lead errors corrected by the reports

Recorded so the reasoning is not repeated.

- **Stagger key.** The lead specified staggering intent re-evaluation by
  entity id. That scatters the active fraction through a 16 MB array and costs
  0.5 to 0.7 core-ms instead of 0.17. **Stagger by a mix of the L1 cell index**
  so runs stay contiguous.
- **Vector layout.** The lead suggested struct-of-arrays. For characters the
  pass is a random graph gather, so struct-of-arrays touches 12 cache lines
  for each candidate and array-of-structs touches 1. **Array-of-structs for
  characters, struct-of-arrays for cohorts. A 12 times difference.**
- **State sharing saving.** The lead estimated 2 ms per tick. The real figure
  is 0.03 to 0.16 ms, because the needs pass already runs every ten ticks.
  Storage is 16 MB, not 6 MB.
- **Dwell-1 calibration.** The lead proposed a dwell-1 baseline to reach a
  12.5-second crossing. Report 17 found dwell-2 plus capacity-16 crossings
  reaches the same target while preserving the capacity-8 lock and the cavalry
  mechanic. **The lead's option list omitted raising capacity on crossing
  terrain only, which is the load-bearing part of the answer.**
- **Speed at a chokepoint.** The lead claimed speed does not help. The formula
  was right but the conclusion did not follow, because dwell is itself a
  function of speed. Correct statement: **speed and throughput are the same
  knob below one tile per tick, and independent above it.**
- **Hawkins-Simon as an insolvency test.** The lead proposed that an insolvent
  institution would fall out of a failed solve. It does not: the coefficient
  matrix is content, not state, so every faction shares one spectral radius.
  It is a **bake-time content validator**. Runtime insolvency is an explicit
  ledger comparison.
- **Spatial command.** The lead proposed a formation as a place rather than a
  membership list. Report 14 rejected it on five functional grounds. The
  structural one: **a region is not stable under movement, so a move order
  changes its own recipient set across frames.**
- **Mike Lewis, not Mark Lewis.** The lead put the wrong name in an agent
  brief.

## F. Defects found in existing rules

- **Progress accumulator overflow.** No clamp, so a unit whose speed exceeds
  the local step cost banks unspendable surplus and overflows `u16` in about
  341 ticks. The accumulator is simulated state, so an overflow enters the
  frame state hash and breaks the golden-file test and the thread-count
  equivalence test. A movement bug that presents as a determinism failure.
- **Integer decay bias.** `(x*k)>>16` sends positive values to exactly zero
  but sticks negative values at −1 forever, a permanent negative bias across
  every dimension. Fixed with a sign-symmetric ceiling decrement.
- **Opinion convergence.** Without an anchor term every entity converges to
  the same vector. This is proven, not merely a risk. The Friedkin-Johnsen
  immutable birth anchor is **not optional**.
- **Flat job field.** Unemployment needs no special case, but a job field with
  no gradient makes every entity a mover and triples movement cost. Needs a
  score floor.
- **`MAX_CAMP_TILES` inversion.** Defined against the world maximum capacity,
  it tightens rather than loosens when bridge capacity rises. Define against
  ordinary capacity.

## G. Questions for the owner, consolidated

Blocking, in rough order of how much they unblock:

1. **Tile scale, and therefore world extent.** See section D. Constrains the
   movement constants and the world's character.
2. **Name three archetypes you expect to exist.** Unanswered since the first
   round. Decides about 2,000 lines and the zero-copy story.
3. **Is 1 million the whole population, or 1 million soldiers plus
   civilians?** Figures in reports 15 and 16 depend on it.
4. **Target living character population.** Report 14 recommends 20,000 to
   50,000 and keeps 262,144 as a hard ceiling.
5. **Settlement count.** Every storage figure in report 12 scales with it.
6. **Commodity split.** 64 exist, 16 traded, remainder local — confirm.
7. **Tile upgrade fraction.** Report 12 estimated 2.7 percent.
8. **Do units make individual decisions?** Now affordable. See section B.

## H. Running per-tick budget, current

| Subsystem | Cost | Source |
|---|---|---|
| Movement | 1.9 to 3.8 wall-ms | Report 10 |
| Trade | 1.1 wall-ms | Report 11 |
| Economy | 0.25 to 0.35 wall-ms | Report 15 |
| Field layer, nine fields | 0.32 to 0.71 core-ms | Report 13 |
| Character tier, Rust | under 0.14 core-ms | Report 14 |
| Character tier, Python | 0.85 ms at 50,000 | Report 14 |
| Individual agency | 0.04 to 0.06 wall-ms | Report 16 |
| Group spatial | under 0.03 wall-ms | Report 17 |
| Vector model | 0.12 core-ms at the ceiling | Report 18 |

**Every figure is derived, not measured.** The research agenda flags
benchmarking on the target platform as blocking most conclusions.
