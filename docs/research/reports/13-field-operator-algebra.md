# Field Operator Algebra

Research report 13 for the foundational architecture decision record.

## 0. Context

Cachette is a world simulation engine. The core is Rust. The control plane
is Python. The engine simulates a hex world at three levels of detail.
Level 0 holds 16,777,216 tiles in a 4,096 by 4,096 grid. Level 1 holds
65,536 cells in a 256 by 256 grid. One level 1 cell summarises 256 tiles.
Level 2 holds 256 blocks. The target scale is 16,777,216 tiles, one million
units and up to 1,024 factions.[^1]

The engine bans floating point from simulated and aggregated state. It
fixes one fixed-point scale, Q16.16. It requires exact associativity from
every aggregate. It requires bit-exact results across thread counts.[^1]

The project owner proposes one abstraction for most world change. The
proposal is that state lives in grids and in masks, and that a change is
the application of an operator to a field. A hut adds quantity to a cell.
A cart carries quantity from one cell to another. A road changes how
easily quantity moves. A mask selects where an operator applies.

The engineering lead reads this as the advection-diffusion-reaction
equation on a hex lattice:

```
d(phi)/dt = div(D grad phi) - div(v phi) + S
```

`D` is a conductance. `v` is a transport velocity. `S` is a source or a
sink. The three terms give spreading, directed transport, and generation.

This report tests that reading. It does not re-derive the results of the
subsystem reports. It cites them and tests the framework against them.

### 0.1 The eight findings

**Finding 1. The unification holds as a vocabulary and as a shared kernel
layer. It does not hold as a runtime engine.** Nine candidate mechanics
express correctly as advection, diffusion and reaction on the level 1 hex
lattice. The shared parts that are hard to get right are few and specific:
the flux pair, the rounding policy, the limiter and the mask convention.
Those parts belong in one module. The composition of operators belongs in
Rust source, monomorphised at compile time, not in a runtime operator
graph. Section 8 gives the break-even arithmetic.

**Finding 2. Finite volume on the 6 hex edges is the correct discrete
form.** It makes conservation exact by construction, in integer
arithmetic, for any flux function and for any rounding rule. The
conservation proof does not depend on the accuracy of the flux. That
property is the single strongest reason to choose finite volume over
finite difference, and it is worth more here than in continuous
mathematics, because the project cannot use floating point.

**Finding 3. The hex lattice is genuinely more isotropic than the square
lattice, and the margin is large.** The 7-point hex Laplacian carries a
directional error of `k^4 / 2880`, where `k` is the wavenumber in radians
for each cell. The 5-point square Laplacian carries `k^2 / 24`. At a
feature size of 6 cells the hex error is 0.035 percent and the square
error is 4.2 percent, a factor of 120. The 9-point isotropic square
stencil closes most of the gap, but it uses 9 taps and two weight classes
against 7 taps and one weight, and it is still 4 times less isotropic.

**Finding 4. Positivity binds before stability, and it fixes one
constant.** An explicit hex diffusion is stable up to a per-neighbour
weight of 2/9. It stays non-negative only up to 1/6. Choose 1/6. A
diffusion that respects that bound needs no flux limiter. An advection
always needs one, because the velocity is arbitrary.

**Finding 5. Truncation causes a real and measurable drift, and one
mechanism removes it.** A truncating integer flux loses the remainder on
every edge on every tick. For a conserved field the loss is not a loss of
mass, because the flux pair still balances, but it is a systematic
retardation: gradients below one quantum never equalise. Store the
remainder for each edge in a persistent `i16` and add it back on the next
tick. This is exact error feedback. It costs 384 KiB for each conserved
field at level 1 and it removes the long-run bias completely.

**Finding 6. The algebra has three composition laws, not one.** A rate
composes by integer addition and forms a group. A constraint composes by
minimum or maximum and forms a semilattice, so a cap must never be written
as a negative source. A mask composes by bitwise AND and OR and also forms
a semilattice. The transport operators compose by matrix product and do
not commute. Confusing the first law with the second is the most likely
error in an implementation, and the type system can prevent it.

**Finding 7. Level 0 is out of reach for every field operator.** One
Jacobi iteration over a `u8` field at level 0 costs 4.7 core-ms. The whole
per-tick budget for influence is 1 to 3 core-ms, and the record allows two
or three full-map passes for each tick in total.[^2] Nine fields at level
1, updated on their own cadences, cost 0.71 core-ms in total, and about
0.32 core-ms with the dirty-block restriction. The whole field layer fits
inside the existing influence-map budget line.

**Finding 8. Instant equalisation is not a field operation.** Two shipped
games converged on the same answer. Factorio replaced a per-tile fluid
model with connected segments that equalise at once.[^3] Dwarf Fortress
moves water under pressure by tracing a path through full tiles, which is
not a local stencil.[^4] A designer who asks for "the price is the same
along the whole road within one tick" is asking for a connected-component
solve, not a diffusion. Section 7.4 states the test that catches this.

---

## 1. Terms

**Field.** One value for each cell at one level of detail.

**Extensive field.** A field whose value is a quantity held in the cell.
Wood, population and pollution mass are extensive. An extensive field sums
when the pyramid restricts it to a coarser level. Transport conserves it.

**Intensive field.** A field whose value is a density, a potential, a rate
or a ratio. Price, temperature, morale and threat are intensive. An
intensive field averages when the pyramid restricts it. Transport does not
conserve it.

**Edge field.** One value for each directed edge between two neighbouring
cells. A hex cell has 6 edges. Each edge is shared by two cells, so the
grid stores 3 edge classes for each cell.

**Mask.** One bit for each cell. The project already uses masks for fog of
war, for faction ownership and for dirty tracking.[^5]

**Conductance.** A per-cell coefficient that states how freely a quantity
crosses that cell. A road raises it. A mountain lowers it.

**Flux.** The signed quantity that crosses one edge in one tick.

**Flux pair.** The two writes that one flux causes. The engine subtracts
the flux from the source cell and adds the same integer to the target
cell.

**Operator.** A function that reads one or more fields and writes one
field.

**Stencil.** A kernel that reads a cell and its 6 neighbours.

**Jacobi sweep.** A stencil pass that reads one plane and writes a second
plane. The result does not depend on the visit order.

**Diffusion number.** The dimensionless group `D dt / h^2`. `D` is the
conductance, `dt` is the timestep and `h` is the centre-to-centre spacing.

**Courant number.** The dimensionless group `|v| dt / h`.

**Quantum.** The smallest representable change in a field. For a `Fix32`
field in Q16.16 the quantum is 1/65536 of one game unit.

---

## 2. The claim, tested

### 2.1 The nine candidate mechanics

Test each claimed member against the three terms of the equation.

| Mechanic | Diffusion | Advection | Reaction | Conserved | Verdict |
|---|---|---|---|---|---|
| Influence and threat | yes, with conductance | no | source and decay | no | fits |
| Economic presence | yes, separable | no | source and decay | no | fits |
| Pollution | yes | wind, optional | source and decay | yes | fits |
| Morale and rumour | yes | no | source and decay | no | fits |
| Price equalisation | yes, as a potential | no | source and sink | no | fits, with a caution |
| Resource transport, aggregate | small | yes, on roads | production and upkeep | yes | fits |
| Migration pressure | yes | no | source | no | fits |
| Supply range | yes, with conductance | no | source at a depot | no | fits |
| Scent and heat | yes | no | source and decay | no | fits |

**Nine of nine fit.** The framework is not forced for any of them. That
result is stronger than the lead expected, and section 7 states why it
does not extend as far as it appears to.

### 2.2 The three terms are not equally used

Count the operator use across the nine mechanics.

| Term | Mechanics that use it | Share |
|---|---|---|
| Reaction, that is source and sink | 9 | 100% |
| Diffusion | 9 | 100% |
| Advection | 2 | 22% |

**Advection is the rare term, and it is also the expensive one.** Section
9 shows that a limited advection pass costs 279 microseconds against 74
microseconds for a diffusion iteration on the same field type. It is also
the only term that needs a flux limiter and an edge velocity store.

This split matters for the engineering decision. A framework that supports
diffusion and reaction only is much smaller than a framework that also
supports advection. Section 8 uses that fact.

### 2.3 The caution on price

Price is an intensive field. A price does not move from one cell to
another. What moves is a good, and the price follows from the local
balance of the good against the local demand.

Diffusing a price directly is a category error. It produces a plausible
picture and a wrong answer, because it lets a cell give away price without
giving away goods. **Diffuse the goods. Derive the price.** If a mechanic
needs a price gradient sooner than the goods can move, model the
expectation as a separate potential field and state in the design that it
is an expectation, not a balance.

This is the first place where the abstraction is forced. It is forced in a
recoverable way, because the correct model is also a field model.

---

## 3. The discrete formulation

### 3.1 The five candidates

**Finite difference.** Write the Laplacian as a weighted sum over the cell
and its 6 neighbours. Cheap and simple. It computes a rate of change for
each cell independently. **Conservation is not structural.** In exact
arithmetic the outflow of one cell equals the inflow of its neighbour, and
the total is preserved. In integer arithmetic the two are computed
separately and rounded separately, so they do not match, and the total
drifts. The drift is small for each edge and it accumulates for every tick
of a long game.

**Finite volume.** Treat each cell as a volume. Compute one flux for each
edge. Apply the flux as a signed pair: subtract it from one cell and add
the same integer to the other. **Conservation is exact by construction,
for any flux function and any rounding rule**, because integer addition is
exactly associative and commutative and because the same integer is
subtracted and added.[^6] The cost is one extra concept, the edge, and one
extra store for the edge residual.

**Discrete exterior calculus.** Represent the field as a 0-form on cells,
the flux as a 1-form on edges, and the divergence as the discrete
codifferential. The framework gives exact discrete versions of the
Stokes theorem and of the vector identities.[^7] For a scalar transport
problem it produces the same arithmetic as finite volume, and it adds a
vocabulary that the team must learn. **It is the right framework for a
future problem with a vector field, a curl or a circulation.** No mechanic
in section 2 needs one.

**Lattice gas cellular automaton.** Move discrete particles along the 6
lattice directions and collide them at the nodes. Every quantity is an
integer by construction, so conservation is exact and no rounding exists.
The FHP model on the hex lattice recovers the Navier-Stokes equation in
the continuum limit, and the earlier HPP model on the square lattice does
not, because the square lattice lacks the necessary symmetry.[^8] **The
defect is statistical noise.** A lattice gas reproduces a smooth field
only after averaging over many nodes and many steps. The project needs a
smooth field from a small grid at every tick, and it cannot average away
noise that it never wanted.

**Lattice Boltzmann.** Store a real-valued distribution for each of the 6
or 7 lattice velocities and relax it toward equilibrium. It removes the
noise of the lattice gas. It stores 7 values for each cell instead of 1,
so a level 1 field costs 7 times more. Its relaxation parameter is a
division, which is awkward in integer arithmetic. It solves a fluid
problem that the project does not have.

### 3.2 The recommendation

**Adopt finite volume with fluxes across the 6 hex edges.**

The reason is specific to the project and it is not the usual reason.
In floating-point science code, finite volume is chosen for shock capture
and for physical fidelity. Here it is chosen because **the conservation
proof survives integer truncation**. The engine cannot use floats. Every
flux is rounded. Finite volume is the only candidate in which rounding
degrades accuracy without degrading conservation.

State the theorem plainly, because it is the load-bearing claim of this
report.

> **Theorem.** Let `F(e)` be any integer-valued function of the two cells
> that share edge `e`. Let the update visit each edge exactly once and
> perform `phi[a] -= F(e)` and `phi[b] += F(e)`. Then the sum of `phi`
> over every cell is unchanged, for every `F`, for every visit order, and
> for every worker count.
>
> **Proof.** Each application adds `-F(e) + F(e) = 0` to the total.
> Integer addition is associative and commutative, so the total does not
> depend on the order in which the applications combine. The total is
> therefore invariant.

The proof needs no accuracy assumption. It is the reason to pay for the
edge concept.

Two obligations follow, and both are cheap to test.

1. **Visit each edge exactly once.** Store 3 edge classes for each cell
   and derive the other 3 from the neighbour. A test that counts edge
   visits catches a violation at once.
2. **Never write one half of a pair without the other.** Make the flux
   applicator the only function that writes a conserved field. A
   conservation test that sums the field before and after each tick and
   asserts equality catches a violation on the first tick.

### 3.3 Hex against square, quantified

The lead asked for a measurement rather than an assertion. Derive it.

A stencil's directional error appears in its Fourier symbol. Write the
wavevector as `k` at angle `theta`. The exact Laplacian has the symbol
`-k^2`, which does not depend on `theta`. A discrete stencil has extra
terms. The terms that depend on `theta` are the anisotropy.

**Square, 5 points, unit weights.** The symbol is
`2 cos(kx) + 2 cos(ky) - 4`. Expanding gives
`-k^2 + k^4 (3/4 + cos(4 theta)/4) / 12`. The part that depends on
`theta` is `k^4 cos(4 theta) / 48`. Relative to the leading `k^2` term,
the peak-to-peak directional variation is `k^2 / 24`.

**Square, 9 points, weights 4 on the edges and 1 on the diagonals, divided
by 6.** The fourth-order term becomes exactly `k^4 / 2`, which does not
depend on `theta`. The stencil is isotropic at fourth order. The
sixth-order term is not isotropic. Its peak-to-peak variation, relative to
the leading term, is `k^4 / 720`.

**Hex, 7 points, unit weights, scaled by 2/3.** The sum of
`cos(k . e_i)` over the 6 unit directions is `3 - (3/2) k^2 +
(3/32) k^4 - ...`. The first term that depends on `theta` appears at sixth
order, because 6-fold symmetry cancels every angular harmonic below
`cos(6 theta)`. The peak-to-peak variation, relative to the leading term,
is `k^4 / 2880`.

| Stencil | Taps | Weight classes | Relative directional error | at k = 0.5 | at k = 1.0 |
|---|---|---|---|---|---|
| Square, 5 points | 5 | 1 | `k^2 / 24` | 1.0% | 4.2% |
| Square, 9 points, isotropic | 9 | 2 | `k^4 / 720` | 0.0087% | 0.14% |
| **Hex, 7 points** | **7** | **1** | **`k^4 / 2880`** | **0.0022%** | **0.035%** |

A wavenumber of `k = 1.0` radians for each cell is a feature about 6 cells
across. A wavenumber of `k = 0.5` is a feature about 12 cells across. Both
are inside the operating range of an influence field at level 1, where
report 09 sets the halving length at 4 cells.[^9]

**Three conclusions follow.**

1. **The hex lattice is 120 times more isotropic than the naive square
   lattice at the working wavenumber.** The claim is real and the margin
   is large. The reason is symmetry: 6-fold symmetry cancels the
   fourth-rank anisotropy that 4-fold symmetry cannot.
2. **The hex 7-point stencil beats the tuned 9-point square stencil by a
   factor of 4, with 2 fewer taps and 1 weight class instead of 2.** The
   comparison that matters is against the best square stencil, not against
   the naive one, and hex still wins.
3. **The advantage is a consequence of the grid the project already
   chose.** The project chose hex for game reasons.[^10] This report
   records that the choice also buys an accuracy result, and that the
   result is not the reason to keep it.

### 3.4 Where hex costs more

State the other side.

A hex cell has 6 neighbours and a square cell has 4. A stencil pass reads
7 values instead of 5, so the arithmetic is 40 percent more expensive for
the same cell count. The isotropy is worth that cost only if the field is
smooth and directional accuracy matters. For a mask operation, a scale or
a clamp, the neighbour count is irrelevant and hex costs nothing extra.

The odd-r offset index means that the 6 neighbour offsets depend on the
parity of the row.[^10] A stencil kernel must therefore hold two offset
tables and select by row parity. Select once for each row, not once for
each cell. The selection is then 256 branches for each pass at level 1,
not 65,536.

---

## 4. Determinism and integer arithmetic

This section states the hard constraints. Every rule here is testable.

### 4.1 Exact conservation through flux pairs

Section 3.2 proves the property. Three implementation rules make it hold.

**Rule 1. One applicator.** Give a conserved field a type that only the
flux applicator can write. The applicator takes an edge index and an
integer flux, and performs both writes. Nothing else may write the field
except a declared source or sink, which section 4.6 handles separately.

**Rule 2. Disjoint outputs.** Partition the grid by level 2 block. Give
each worker a set of blocks. An edge inside a block is written by one
worker. An edge on a block boundary is written by the owner of the
lower-numbered block, and the other worker reads only. This removes every
atomic operation. The weak ARM memory model makes a relaxed atomic emit a
real barrier, so removing atomics is worth the extra partition
logic.[^11]

**Rule 3. Fixed order.** Process blocks in ascending level 2 block number
and edges in ascending edge class inside a block. The order does not
affect the total, because the total is invariant. It does affect the flux
limiter of section 4.4, which is why the order must still be fixed.

### 4.2 The stability and positivity bounds

An explicit scheme has a step limit. Derive it for the hex lattice.

Write one Jacobi diffusion step as
`phi'[c] = phi[c] + alpha * sum over the 6 neighbours of (phi[n] - phi[c])`.

**The stability bound.** The amplification factor is
`1 + alpha * (sum of cos(k . e_i) - 6)`. The sum of the cosines has a
minimum of `-3`, which occurs at the corner of the first Brillouin zone.
The largest value of `6 - sum` is therefore 9. Stability needs
`|1 - 9 alpha| <= 1`, so **`alpha <= 2/9`**.

**The positivity bound.** The new value is a weighted average of 7 old
values with weights `alpha` on each neighbour and `1 - 6 alpha` on the
cell. Every weight must be non-negative, so **`alpha <= 1/6`**.

**Positivity binds first.** Choose `alpha <= 1/6`.

Compare the square lattice. There the two bounds coincide at `alpha <= 1/4`.
Convert both to the diffusion number:

| Lattice | Laplacian scale | Stability | Positivity | Diffusion number limit |
|---|---|---|---|---|
| Square, 5 points | `1/h^2` | `alpha <= 1/4` | `alpha <= 1/4` | `D dt / h^2 <= 1/4` |
| Hex, 7 points | `(2/3)/h^2` | `alpha <= 2/9` | `alpha <= 1/6` | `D dt / h^2 <= 1/4` |

**The two lattices admit the same diffusion number.** Hex gives no
timestep penalty. The difference is which constraint binds.

**The consequence for the engine is large. A diffusion that obeys
`alpha <= 1/6` cannot produce a negative value, so it needs no flux
limiter.** Eight of the nine mechanics in section 2 use diffusion without
advection. Eight of nine therefore need no limiter at all.

**The advection bound.** For an upwind flux form on hex, at most 3 of the
6 outward normals carry outflow for any velocity direction, and the
projections of a unit vector on 3 adjacent hex normals sum to at most 2.
Working through the cell area and the edge length gives
**`|v| dt / h <= 3/4`**, where `h` is the centre-to-centre spacing. The
square upwind limit is `|v| dt / h <= 1 / sqrt(2)`, which is about 0.71.
Hex is again slightly more permissive.

Express the bound as a design rule rather than as a runtime check. At 10 Hz
and a level 1 cell of 16 tiles, a velocity of 3/4 of a cell for each tick
is 12 tiles for each tick, which is far above any transport speed the game
needs. **The bound is never approached in practice. Do not add a runtime
CFL test. Add a debug assertion instead.**

### 4.3 The fixed-point scale

The project fixes Q16.16 for positions and stats.[^1] Report 09 chose a
`u8` in Q0.8 against a compile-time reference for influence, and justified
the exception on the grounds that no consumer reads an absolute
magnitude.[^9] This report needs a rule that covers both.

**The rule is conservation.**

| Field kind | Scale | Type at L1 | Bytes for each cell | Reason |
|---|---|---|---|---|
| Conserved and extensive | Q16.16 | `Fix32` | 4 | The total is a game quantity. It must be exact and it must be comparable against an entity inventory. |
| Potential and intensive, compared only | Q0.8 against a constant | `u8` | 1 | No consumer reads a magnitude. 4 lanes of throughput and 4 times less memory. |
| Intensive with a real unit, such as a price | Q16.16 | `Fix32` | 4 | A price enters a transaction, so its magnitude is read. |

**Justify Q16.16 for a conserved field.** A conserved field must agree
with the entity-level inventory that it summarises. A caravan carries an
integer count of goods. If the field stored whole goods only, then every
flux below one good would truncate to zero, and a slow trickle along a
road would never move. Q16.16 puts the truncation floor at 1/65536 of one
good. A flux of one good for each 65,536 ticks is 1.8 hours of game time
at 10 Hz, which is below any design tolerance. **The 16 fractional bits
are not precision for the designer. They are headroom against
truncation.**

The range of a `Fix32` in Q16.16 is plus or minus 32,768 game units for
each cell.[^1] One level 1 cell covers 256 tiles. A stockpile above 32,768
units of one good in 256 tiles is a design decision the owner must avoid,
or the field must widen to `Fix64`. Record this as a bound, not as a
problem: widening one field from `Fix32` to `Fix64` doubles that field
from 256 KiB to 512 KiB at level 1 and changes nothing else.

### 4.4 Flux limiting

A diffusion under `alpha <= 1/6` needs no limiter. An advection does,
because the velocity is set by game logic and a cell may be asked to send
out more than it holds.

**The limiter, in three passes.**

```
pass 1   for each edge e:  F[e] = unlimited_flux(e)          // may overdraw
pass 2   for each cell c:  out[c] = sum of F[e] leaving c
                           if out[c] > phi[c]:
                               scale[c] = phi[c]             // numerator
                           else:
                               scale[c] = out[c]             // no scaling
pass 3   for each edge e:  G[e] = (F[e] * scale[a]) / out[a]  // i64 intermediate
                           phi[a] -= G[e];  phi[b] += G[e]
```

Three properties make this deterministic and exact.

1. **The scaling is a single integer multiply and divide in `i64`.** The
   numerator is at most `2^31 * 2^31`, which fits in `i64` with room. Use
   a truncating divide.
2. **The truncation leaves a remainder in the source cell.** Because the
   scaled fluxes sum to at most `phi[a]`, the cell never goes negative.
   The unsent remainder stays where it is and is offered again next tick.
   This is correct behaviour, not an error.
3. **The result does not depend on the worker count.** Pass 2 is a
   per-cell reduction over 3 to 6 edges in a fixed direction order. Pass 3
   reads only pass 2's output.

The cost is 3 passes instead of 1. Section 9 prices it.

**Do not use a slope limiter or a total-variation-diminishing scheme.**
Those exist to control oscillation near a shock in a high-order
scheme.[^12] The engine uses a first-order upwind flux, which does not
oscillate. It is diffusive, which is acceptable, because the transported
quantity is already an aggregate over 256 tiles.

### 4.5 Rounding policy and drift

This is the part that a careless implementation gets wrong, and the error
is slow and hard to find.

**The problem.** A flux is a product of a coefficient and a difference,
scaled by a right shift. The shift truncates. For a non-negative field
with a signed flux, an arithmetic right shift rounds toward negative
infinity, so it biases flux in one spatial direction. Truncation toward
zero biases every flux downward in magnitude. Either way, a gradient
smaller than one quantum produces a flux of zero, and it never equalises.
The field freezes into a permanent staircase.

Three mechanisms remove the bias. Evaluate all three.

**Mechanism A. Deterministic dithering.** Add a per-edge offset derived
from a counter-based hash of the edge index and the tick, then shift. The
offset is unbiased over many ticks. It is deterministic, because the
project already keys every draw on a counter.[^1] **Reject it.** It
injects visible noise into a field whose whole purpose is smoothness, and
it makes a field that should be steady change every tick, which defeats
the dirty-block restriction.

**Mechanism B. Larger quantum headroom only.** Rely on Q16.16 to put the
staircase below any visible threshold. **Accept it for potential fields.**
An influence field is compared, not read, and report 09 already derives
its support radius from the quantisation floor.[^9] The staircase is the
support boundary and it is the intended behaviour.

**Mechanism C. Residual carry.** Store the truncated remainder for each
edge in a persistent `i16`. On the next tick, add the stored remainder to
the numerator before the shift, then store the new remainder. This is
exact error feedback. Over `n` ticks the accumulated flux differs from the
exact flux by less than one quantum, for every edge, with no bias.
**Accept it for conserved fields.**

| Field kind | Mechanism | Extra store at L1 | Long-run bias |
|---|---|---|---|
| Potential, compared only | B, headroom | none | present, and intended as the support bound |
| Conserved and extensive | C, residual carry | 384 KiB for each field | none |

The residual store is 3 edge classes for each cell, 65,536 cells, `i16`,
which is 393,216 bytes. **Pay it for a conserved field. Do not pay it for
a potential field.**

One further rule applies to every kernel. **Use a truncating right shift
by a compile-time constant. Never divide by a runtime variable.** A shift
is exact and identical on every target. A division by a variable invites a
compiler to choose a reciprocal path, and the project's lint boundary
exists for exactly this reason.[^1] Report 09 states the same
condition.[^9]

### 4.6 Sources and sinks

A source adds quantity. A sink removes it. Neither is a flux, so neither
conserves.

Three rules keep a source deterministic.

**Rule 1. A source is a scatter with a fixed key order.** Sort the sources
by cell index, then by a stable secondary key such as the entity
identifier. Accumulate into the field in that order. The project already
requires one very good parallel radix sort, so the sort is not new
work.[^2]

**Rule 2. A sink must not drive a field negative.** Clamp the withdrawal
at the stored value, and report the shortfall to the caller. The engine's
partial-failure rule already returns summaries rather than failing a whole
command.[^2]

**Rule 3. Widen the accumulator.** A source scatter accumulates many
contributions into one cell. Accumulate in `i64` and store back as
`Fix32`. The record's accumulator-width rule requires this.[^1]

### 4.7 Fixed iteration counts

The engine must never stop on a convergence test.

A convergence test reads a residual. A residual is a reduction over the
whole grid. A parallel reduction of a residual has a result that depends
on the combining order unless the reduction is exactly associative and
ordered. Integer reduction is exactly associative, so an ordered integer
reduction is in fact safe. **The hazard is not the arithmetic. The hazard
is that a variable iteration count makes the wall time variable and makes
the tick budget unpredictable, and that any later change to the reduction
becomes a determinism bug.**

**Fix every iteration count as a compile-time constant.** Report 09
reaches the same conclusion for the same reason and fixes 8 iterations for
its military plane.[^9] This report extends the rule to every operator.

State the accuracy consequence honestly. A fixed iteration count does not
reach a steady state. It reaches a partial relaxation whose error decays
over ticks, because the next tick continues from the previous result. The
field is therefore lagging, not wrong. Section 6.3 shows that no consumer
needs a true steady state.

---

## 5. The operator algebra

### 5.1 The types

The algebra needs four types. Two are fields, one is a mask and one lives
on edges.

```rust
/// A quantity held in a cell. Sums under restriction. Transport conserves it.
struct Extensive<const L: u8> { cells: Plane<Fix32> }

/// A density, a potential or a ratio. Averages under restriction.
struct Intensive<const L: u8, T> { cells: Plane<T> }   // T is u8 or Fix32

/// One bit for each cell.
struct Mask<const L: u8> { bits: BitPlane }

/// One value for each of the 3 stored edge classes for each cell.
struct EdgeField<const L: u8, T> { edges: [Plane<T>; 3] }
```

The level `L` is a const generic. That makes a mixed-level operator a
compile error and it costs nothing at run time.

### 5.2 The operators

| Operator | Signature | Kernel | Conserves | Linear |
|---|---|---|---|---|
| `source` | `(Field, Scatter) -> Field` | scatter, then reduce | no, adds | yes if the source does not read the field |
| `sink` | `(Field, Scatter) -> Field` | scatter, then reduce | no, removes | yes, under the same condition |
| `decay` | `(Field, k) -> Field` | map | no | yes |
| `diffuse` | `(Extensive, Coeff, alpha, n) -> Extensive` | stencil, n times | yes | yes |
| `advect` | `(Extensive, EdgeField) -> Extensive` | stencil, 3 passes | yes | yes in the field, no in the velocity |
| `scale` | `(Field, k) -> Field` | map | no unless k = 1 | yes |
| `clamp` | `(Field, lo, hi) -> Field` | map | no | **no** |
| `gate` | `(Field, Mask) -> Field` | map | no | yes, it is a projection |
| `combine` | `(Field, Field, op) -> Field` | map | depends on op | yes for add |
| `gradient` | `(Field) -> EdgeField` | stencil | not applicable | yes |
| `restrict` | `(Field@L) -> Field@L+1` | reduce | yes for extensive | yes |
| `prolong` | `(Field@L+1) -> Field@L` | map | yes for extensive | yes |

### 5.3 The type rules

Six rules. A compiler can check all six.

**Rule 1. Only an extensive field may be advected or diffused as a
quantity.** An intensive field may be diffused only as a potential, and
the design must then state that the result is a potential and not a
balance. Section 2.3 gives the price example.

**Rule 2. `restrict` uses a different combine for each kind.** An
extensive field restricts by sum. An intensive field restricts by a
weighted mean, stored as a sum and a count so that the combine stays a
group.[^2] The field registry already generates the combine for each
field, so this rule is a registry entry, not new machinery.[^2]

**Rule 3. `prolong` on an extensive field must divide the parent value by
the child count and hand the remainder to the lowest-indexed child.**
Otherwise the prolongation creates quantity. An intensive field prolongs
by copying.

**Rule 4. Two fields combine only at the same level and the same kind.**
Adding an extensive field to an intensive field is a category error.

**Rule 5. `clamp` breaks conservation. Never apply it to a conserved
field.** Use the flux limiter instead, which bounds the field from below
without destroying quantity. A clamp on a conserved field silently deletes
or invents quantity, and the conservation test then fails, which is the
correct outcome.

**Rule 6. A conserved field has exactly one writer per operator class.**
The flux applicator writes transport. The source scatter writes
generation. Nothing else writes it.

### 5.4 Composition, and what commutes

This table is the practical core of the section. Each entry is a statement
about the order of two operators applied to the same field.

| Pair | Commutes | Note |
|---|---|---|
| `source(a)` then `source(b)` | **yes** | Integer addition. This is the group law. |
| `sink(a)` then `sink(b)` | **yes** | Only while neither clamps. A clamped sink does not commute. |
| `scale(k)` then `diffuse` | **yes**, if `k` is a spatial constant | If `k` is a field, they do not commute. |
| `scale(k)` then `advect` | **yes**, if `k` is a spatial constant | Same condition. |
| `diffuse` then `advect` | **no** | This is operator splitting. The error is first order in `dt`. |
| `diffuse(n)` then `diffuse(m)` | **yes in exact arithmetic, no in integers** | Rounding makes `n + m` separate steps differ from one combined step. |
| `source` then `diffuse` | **no** | Differs by one diffusion of the source term. |
| `gate(m)` then `diffuse` | **no** | Gating first zeroes a source. Gating last clips the spread. Both are useful and they are different. |
| `clamp` then anything | **no** | Clamp is idempotent and monotone. It is not linear. |
| `gate(m1)` then `gate(m2)` | **yes** | Bitwise AND is commutative and associative. |
| `restrict` then `scale(k)` | **yes** | Scaling and summing commute for a constant `k`. |
| `restrict` then `diffuse` | **no** | Coarse diffusion is not fine diffusion followed by restriction. |

**The three laws.** Read the table again by composition law rather than by
pair.

1. **The group law.** `source`, `sink` and `combine(add)` compose by
   integer addition. They commute, they associate, and they have an
   identity and an inverse. **A rate is a member of this class.**
2. **The semilattice law.** `clamp`, `gate` and `combine(min)` and
   `combine(max)` compose by an idempotent, commutative and associative
   operation with no inverse. **A constraint and a set are members of this
   class.**
3. **The matrix law.** `diffuse`, `advect`, `restrict` and `prolong`
   compose by matrix product. They associate. They do not commute.

**The most likely implementation error is to treat a constraint as a
rate.** A designer says "the granary holds at most 500 units". A
programmer writes a sink that removes the excess. The sink is a rate, so
it composes by addition, and two granary caps then subtract twice. The
correct form is `clamp`, which composes by minimum and is idempotent.
**Name the three laws in the code. Give each class its own trait.**

### 5.5 Where a field is a rate and where it is not

The project's existing taxonomy distinguishes a rate, a constraint and a
set. Map the field vocabulary onto it.

| Project term | Field term | Composition | Aggregates by |
|---|---|---|---|
| Rate | The source term `S`, and any per-tick delta | addition, group | sum |
| Constraint | `clamp`, the flux limiter, a capacity | minimum or maximum, semilattice | minimum or maximum, with an extremum count[^2] |
| Set | `Mask`, and the gate operator | AND and OR, semilattice | popcount for each bit[^2] |

**A field is a rate when it holds a per-tick change.** Production for each
tick is a rate. Upkeep for each tick is a rate. These sum, and the pyramid
summarises them by sum.

**A field is not a rate when it holds a stock.** Stored wood is a stock,
not a rate. A stock also sums under the pyramid, so it is also a group,
but it does not compose with another stock by addition in the way a rate
does. Two production buildings in one cell give a combined rate. Two
stockpiles in one cell give a combined stock only if the good is fungible.

**A field is neither a rate nor a stock when it holds a potential.**
Threat, price and morale are potentials. A potential is intensive. It does
not sum under the pyramid and it does not compose by addition across
sources in general. Report 09's influence field is a special case: it
composes by saturating addition, which is exactly associative and
commutative, so it does behave as a monoid, but it has no inverse above
the saturation point.[^9] Report 09 declares it under case (b) of the
aggregation rule for that reason.[^2]

**State the rule for a designer.** Ask what the sum over a region means.
If the sum is meaningful, the field is extensive and it aggregates by sum.
If only the average is meaningful, the field is intensive and it
aggregates by a sum and a count. If neither is meaningful, the quantity is
not a field.

---

## 6. The linear-algebra view

### 6.1 One application is a sparse matrix-vector product

One diffusion step is `phi' = A phi`, where `A` is a 65,536 by 65,536
matrix with 7 non-zero entries in each row: the cell and its 6 neighbours.
On the square lattice the same operator has 5. The matrix is never stored.
The stencil kernel is the matrix.

Three properties follow.

1. **`A` is symmetric when the conductance is symmetric on each edge.**
   Define the edge conductance as a function of the two cells that is
   symmetric in its arguments, such as the minimum or the harmonic mean.
   Then `A` is symmetric, and the operator conserves the total exactly
   when written in flux form.
2. **`A` is a stochastic matrix when `alpha <= 1/6`.** Every row is
   non-negative and sums to 1. This is the positivity result of section
   4.2 restated. A stochastic matrix cannot increase the maximum or
   decrease the minimum, so the field cannot overshoot.
3. **Iterating `n` times is the power method applied to `A`.** The
   dominant eigenvector of `A` is the uniform field, with eigenvalue 1.
   Without a decay term, repeated diffusion drives every field to a
   constant. **Every potential field needs a decay term.** With decay `d`,
   the operator is `d A` and the dominant eigenvalue is `d`, so the field
   settles at a finite profile.

### 6.2 Steady state as a linear solve

The steady state satisfies `(I - d A) phi = S`. That is a sparse linear
system with 65,536 unknowns and about 458,752 non-zero entries.

Three solvers exist.

**Direct factorisation.** Reject. A sparse Cholesky factor of a
two-dimensional grid Laplacian has fill-in of order `N log N`, so the
factor is tens of megabytes, and it must be refactorised whenever the
conductance changes. The conductance changes whenever a road is built.

**Krylov methods, such as conjugate gradient.** Convergence takes about
`sqrt(kappa)` iterations, where `kappa` is the condition number. A strong
decay term makes the system well conditioned, so 10 to 30 iterations
suffice. Each iteration costs one matrix-vector product, which is one
Jacobi sweep. **Reject it anyway, for two reasons.** First, the method
needs a step length `alpha = r'r / p'Ap`, which is a division of two
reductions. In integer arithmetic that division truncates, and the
optimality property that makes conjugate gradient converge quickly no
longer holds. Second, each iteration needs two global reductions, which
are two barriers inside a kernel that currently has none.

**Multigrid.** Restrict the problem to a coarser grid, solve there, and
prolong the correction back. Accept it. The project already holds a three
level pyramid with a restriction path and a dirty bitset.[^2] Report 09
already uses a two-grid cycle for its military plane and measures the
result at 150 microseconds against 590 microseconds for 32 plain Jacobi
iterations.[^9] **Multigrid is the correct accelerator and the project
already owns its machinery.**

### 6.3 Does any consumer need a true steady state?

Enumerate the consumers and check.

| Consumer | Reads | Needs a steady state |
|---|---|---|
| Threat assessment | a comparison of two regions | no |
| Territory and borders | an argument of the maximum | no |
| Path cost modifier | a gradient sign | no |
| Target selection | an argument of the maximum | no |
| Economic gradient | a gradient sign | no |
| Migration and settlement | a ranking of candidate cells | no |
| Contested detection | a difference of two stored values | no |
| Resource transport | the field itself, as a stock | it is a stock, so it has no steady state |
| Price | a magnitude | **it would prefer one** |
| Python observation | an array | no |

**No consumer needs a true steady state.** Nine of ten compare, rank or
take a gradient, and a partially relaxed field preserves the order of two
well-separated cells. Price is the one consumer that reads a magnitude,
and section 2.3 already places price on the boundary of the framework.

**Therefore: iterate. Do not solve.** Use a fixed count of Jacobi sweeps,
seeded by a two-grid cycle where the range demands it. The field carries
its state across ticks, so the relaxation continues and the error decays
over ticks rather than inside one tick.

The break-even, stated as a rule: **solve only when a consumer reads an
absolute magnitude and the source changes faster than the relaxation
converges.** No mechanic in section 2 meets both conditions.

### 6.4 Where the model stops being linear

The linear view is an approximation and the report must say where it
fails. Five mechanisms break linearity.

**Saturation.** Report 09 stores influence as a `u8` with saturating
addition at 255.[^9] Saturation is not linear. Two sources whose separate
fields each reach 200 do not sum to 400. The consequence is concrete:
**report 09's method 5, the closed-form falloff summed over sources, is
valid only below saturation.** It is a superposition argument, and
superposition needs linearity. State the condition where the method is
used.

**Capacity caps.** A granary holds 500. A cap is a clamp, which is
monotone but not linear.

**State-dependent sources.** A woodcutter's hut produces only if the tile
still holds trees and the hut holds workers. The source term is then a
function of the field, so the equation becomes `d(phi)/dt = ... + S(phi)`.
That is a semi-linear reaction-diffusion equation, not a linear one. It is
still tractable, because `S` is evaluated pointwise before the transport
step, and operator splitting handles it.

**The flux limiter.** Section 4.4's limiter scales fluxes by a factor that
depends on the field. It is nonlinear by construction. It applies only
where a cell would overdraw, so the operator is piecewise linear with a
field-dependent switch.

**Field-dependent masks.** A mechanic such as "fire spreads only where
fuel exceeds a threshold" makes the mask a function of the field. That is
a switching system, and its behaviour can be discontinuous in the initial
condition. **This is the most dangerous of the five**, because a threshold
crossing amplifies a one-quantum difference into a visible difference. It
does not break determinism, because the arithmetic is exact, but it does
break the intuition that a small change gives a small effect.

**The honest summary.** The model is linear in the transport terms and
semi-linear overall. Superposition holds only below every saturation and
every threshold. Do not precompute a Green's function and reuse it, except
where a measurement shows the field stays in its linear range.

---

## 7. Where the abstraction fails

This section is the one the owner should read first.

### 7.1 The membership test

**A mechanic belongs inside the field framework only if all five answers
are yes.**

**Question 1. Fungibility.** Are two units of this quantity
interchangeable? Ten wood is ten wood. Hero Bors is not two units of five
Bors. A thing with a name, a history or an inventory is not a field.

**Question 2. Locality.** Does the change depend only on a cell and its 6
neighbours? A treaty between two factions on opposite sides of the map is
not local. Water that equalises through a full pipe within one tick is not
local.

**Question 3. Divisibility.** Is a fractional value meaningful at the tick
scale? Half a unit of wood in a cell is a reasonable intermediate. Half a
cart is not a thing, and a model that produces one has a bug that the
player will see.

**Question 4. Additive aggregation.** Is the sum over a region meaningful,
or is the average meaningful? If neither is, the quantity is not a field.
The sum of unit types in a region is meaningless.

**Question 5. Density.** Is the quantity present in enough cells to pay
for a full pass? A field costs the same whether 10 cells or 65,536 cells
hold a value. Section 7.5 gives the break-even count.

### 7.2 What falls outside

| Mechanic | Fails which question | Correct home |
|---|---|---|
| A named unit or hero | 1, fungibility | The entity store |
| A specific building | 1, fungibility | The entity store |
| A treaty or an alliance | 1 and 2 | The diplomacy relation plane[^5] |
| A duel between two heroes | 1, 2 and 3 | A per-entity system |
| A construction order | 1 and 4 | The command queue |
| A technology tree | 2 and 4 | Per-faction state |
| A contract or a trade agreement | 1 and 2 | A per-faction record |
| An individual inventory | 1 | The entity store |
| Ownership transfer | 1 and 4 | An event |
| A quest or a scripted trigger | every question | The control plane |
| A siege at one settlement | 1 and 5 | A per-settlement system |
| A caravan of 200 on the map | 5, density | The entity store |

### 7.3 Combat is not advection

Say this plainly, because the surface similarity is misleading.

Advection moves a conserved quantity along a velocity field. The total
does not change. Combat destroys quantity on both sides, at a rate that
depends on both sides, with thresholds for rout and morale collapse.

The correct field-like model for army-scale attrition is a **pointwise
reaction term** of the Lanchester kind: each side loses at a rate
proportional to the opposing strength in the same cell. That is the `S`
term of the equation, evaluated cell by cell, with no transport at all.
It is a legitimate use of the framework, and it is confined to one term.

Two limits apply, and both are hard.

**Limit 1. A reaction term cannot express a named participant.** A duel
between two heroes has two participants with names, histories and
equipment. The outcome is not a rate. It is an event on two entities.

**Limit 2. A reaction term averages away the thing the player watches.**
A field says that faction A lost 12.4 percent of its strength in this
cell. A player wants to know which regiment broke. **Run the field model
where nobody is watching and the entity model where somebody is.** The
project already tiers factions this way for fog and for
influence.[^5][^9] Extend the same tiering to combat: an abstracted
reaction term for a passive faction, and per-entity resolution for a
rendered one.

**State the reconciliation cost.** Two models of the same mechanic must
agree at the boundary. When a rendered faction meets a passive faction,
one side resolves per entity and the other resolves as a field. The engine
must convert between them. That conversion is not free and it is not in
this report's scope. Record it as an open question.

### 7.4 The non-locality trap, with evidence

Two shipped games converged on the same answer, and the convergence is
evidence rather than opinion.

**Factorio.** The original fluid system equalised pressure between
adjacent pipe segments each tick. It was slow, and its behaviour depended
on update order in ways that players could not predict. The rewrite
merged connected pipes, underground pipes and tanks into one **segment**
with a shared volume, and made the flow rate a function of how full the
segment is.[^3] The rewrite replaced a per-cell local field with a
connected-component aggregate.

**Dwarf Fortress.** Water occupies a tile at one of 7 levels. Under
pressure, water traces a path through already-full tiles and appears at
the far end, without generating flow along the way.[^4] That is an
explicitly non-local operator, added because the local model did not
produce the behaviour the design wanted.

**The lesson, stated as a rule.** A local stencil propagates information
at one cell for each iteration. If a design requires a quantity to
equalise across a connected region within one tick, no fixed number of
stencil iterations delivers it. The correct structure is a
connected-component solve on the graph of connected cells, followed by one
scatter of the equalised value.

**The test for the owner.** Ask the designer: "how many ticks may this
take to settle across the whole road network?" If the answer is "it must
be instant", the mechanic is a graph solve, not a field. If the answer is
a number of seconds, the mechanic is a field and the iteration count
follows from the answer.

The project already owns the graph. The portal graph over 32 by 32 pathing
chunks holds about 100,000 nodes.[^2] A connected-component pass over it
is cheap, and a road network is a subgraph of it.

### 7.5 The density break-even

A field pass costs the same for an empty cell as for a full one. Quantify
the point where a field becomes cheaper than a list of entities.

One diffusion iteration over a `u8` field at level 1 costs 18 microseconds
for 65,536 cells, which is 0.27 nanoseconds for each cell. A per-unit
steering update costs 20 to 40 nanoseconds.[^2] Take 30.

| Operator cost | Field cost at L1 | Entities that cost the same |
|---|---|---|
| 1 map pass | 4.1 us | 140 |
| 1 diffusion iteration | 18 us | 600 |
| 8 diffusion iterations | 144 us | **4,800** |
| 1 limited advection pass | 279 us | 9,300 |

At level 0 multiply every entity figure by 256.

| Operator cost | Field cost at L0 | Entities that cost the same |
|---|---|---|
| 1 map pass | 1.05 ms | 35,000 |
| 8 diffusion iterations | 37 ms | **1,230,000** |

**Read the second table as a prohibition.** Eight diffusion iterations at
level 0 cost 37 core-ms, which is comparable to the whole movement and
steering line in the record's budget, and the record's entire per-tick
total is 90 to 360 core-ms.[^2] The break-even against entities is 1.23
million, which exceeds the target unit count. **No field operator runs at
level 0. There is no case in which it wins.** This confirms report 09's
conclusion by an independent argument.[^9]

**Read the first table as a design rule.** A quantity carried by fewer
than about 5,000 agents at level 1 is cheaper as a list of agents. A
caravan system with 200 caravans is 24 times below the break-even. Report
11 designs resource and trade flow, and this is the number it needs.

### 7.6 The hybrid, and where it costs

Three mechanics sit on the boundary and need both models.

**Resource transport.** The caravan is an entity, because it has a route,
a cargo and an owner. The aggregate flow along a corridor is a field. The
two must agree. The clean split is: **the entity moves, and the field
records what the entities are doing in aggregate**, so the field is a
derived projection and never a source of truth. That matches the project's
existing rule that level 0 is the only source of truth.[^1]

**Crowd movement.** The flow field is a field. The unit is an entity. The
record already splits it this way: the flow tile supplies the direction
and the per-unit blend supplies the local behaviour.[^2] No new
reconciliation is needed.

**Population.** A settlement's population is a number on an entity. The
migration pressure that decides where population moves is a field. The
field is an input to an entity decision, not a store of population. This
split is clean.

**The rule that makes each hybrid safe: one side owns the truth and the
other side is derived.** A hybrid in which both sides store the same
quantity will drift, and the drift will be invisible until a player
notices that the numbers disagree.

---

## 8. Over-unification risk

### 8.1 The two things that could be built

Separate them, because they have different costs and different value.

**Option 1. A shared kernel layer.** One module holding the flux
applicator, the residual carry store, the limiter, the mask convention,
the field type distinction, the rounding policy and the level dispatch.
Operators are Rust functions. A mechanic composes them in source code,
monomorphised at compile time.

**Option 2. A runtime operator graph.** A data structure describing a
sequence of operators, built at run time, possibly from Python, and
interpreted or compiled by a scheduler each tick.

### 8.2 The cost of each

Estimate against the specific kernels that reports 09 and 06 already
describe.

| Item | Lines of Rust | Test burden |
|---|---|---|
| One specific potential-field kernel, as report 09 describes | 200 to 400 | determinism, one golden file |
| Five such kernels, written independently | 1,000 to 2,000 | five golden files |
| **Option 1, the shared kernel layer** | **700 to 1,000** | conservation, positivity, determinism, residual carry, limiter, and one golden file for each field |
| **Option 2, the runtime graph, on top of option 1** | **+1,500 to 2,500** | plus a scheduler, plus a validation pass, plus an error path for every type rule |

### 8.3 The break-even

**The break-even is not on line count. It is on the number of mechanics
that need exact integer conservation.**

A potential field needs no flux pair, no residual carry and no limiter.
Report 09's influence kernel is 200 to 400 lines and it is complete and
correct. Five potential fields are five such kernels, and the shared parts
between them are small.

A conserved field needs all of it. Each conserved field written
independently must re-derive the flux pair, the edge ownership rule, the
limiter and the residual carry. Each of those is easy to get subtly wrong
and hard to detect, because the failure is a slow drift.

| Conserved fields | Verdict on option 1 |
|---|---|
| 0 | Not justified. Write specific kernels. |
| 1 | Marginal. The one kernel carries the machinery itself. |
| **2 or more** | **Justified.** The shared machinery is written once and tested once. |

The project has at least two: resource transport and pollution. Population
is a likely third. **Option 1 is justified.**

For option 2, the break-even is different and much higher.

| Distinct field mechanics | Verdict on option 2 |
|---|---|
| Under 8 | Not justified. |
| 8 to 10 | Marginal. |
| Above 10, with mechanics added by designers rather than programmers | Justified. |

Section 2 lists nine candidates and two of them are on the boundary. The
project therefore sits just below the break-even.

### 8.4 Three arguments against the runtime graph

**Argument 1. It hides the decisions that must be explicit.** Section 5.4
shows that gating before a diffusion and gating after it give different
and both-useful results, and that repeated diffusion does not compose in
integer arithmetic. A runtime graph makes those choices look like
interchangeable node orderings. They are not. In Rust source the order is
visible on the page and a reviewer sees it.

**Argument 2. It defeats vectorisation.** A monomorphised composition
fuses into one loop with the operands in registers. An interpreted graph
writes each intermediate to memory and reads it back. At level 1 a `Fix32`
field is 256 KiB, so a three-operator chain moves 1.5 MB instead of
512 KiB. That is a factor of 3 in traffic for no gain.

**Argument 3. The project's own precedent points the other way.** The
record already generates the accessor, the combine, the summary slot and
the predicate from one field registry at compile time.[^2] A field
operator registry is the same pattern applied to the same problem.
**Extend the field registry. Do not build a second, dynamic mechanism
beside it.**

### 8.5 The recommendation

**Adopt option 1. Reject option 2 for version 1.**

Express a mechanic as a Rust function that composes typed operators. Let
the type system enforce the six rules of section 5.3. Let the field
registry generate the restriction combine, the summary slot and the
Python view for each field, as it already does for a tile field.

**Record the condition that would reverse the decision.** Adopt option 2
if, and only if, a designer needs to add a new field mechanic without a
Rust change, and the count of such mechanics passes ten. That is the
extensibility-ladder question, and the record already stages
extensibility rather than building it first.[^2]

### 8.6 The honest counter-argument

State the case against this report's own recommendation.

A shared kernel layer is a dependency. Every field mechanic then waits on
it, and a defect in it breaks every mechanic at once. Five independent
kernels fail independently. For a project with no code yet, a shared layer
built before the first mechanic is a design written against imagined
requirements.

**The mitigation is ordering.** Build report 09's influence kernel first,
as a specific kernel with no shared layer. Build the second field the same
way. **Extract the shared layer from two working kernels, not before
them.** The record's staged plan already works this way, and it builds the
flat path before the descent for the same reason.[^2]

This ordering costs one refactor and it removes the risk of designing the
abstraction against a guess.

---

## 9. Fit to the engine

### 9.1 Every operator in the kernel vocabulary

The engine's kernel vocabulary is map, gather, scatter, reduce, scan,
sort, stencil and local join.[^13] Express each operator in it.

| Operator | Kernels used | Parallel shape | Atomics needed |
|---|---|---|---|
| `source` | sort, then scatter, then reduce | disjoint by block after the sort | none |
| `sink` | sort, then scatter, then reduce | same | none |
| `decay` | map | fully disjoint | none |
| `diffuse` | stencil, `n` times | disjoint by block, halo read only | none |
| `advect`, pass 1 | stencil | disjoint by block | none |
| `advect`, pass 2 | reduce, per cell over 6 edges | disjoint by cell | none |
| `advect`, pass 3 | scatter through the flux applicator | disjoint by block, boundary edges owned by the lower block | none |
| `scale`, `clamp`, `gate` | map | fully disjoint | none |
| `combine` | map, two inputs | fully disjoint | none |
| `gradient` | stencil, writing an edge field | disjoint by block | none |
| `restrict` | reduce over 256 children | disjoint by parent | none |
| `prolong` | map with a shared parent read | disjoint by child | none |

**No operator needs an atomic.** Every write is disjoint after the block
partition. That satisfies the target platform's preference for disjoint
outputs over atomics.[^11]

**Two operators need a sort.** `source` and `sink` scatter from a list of
producers into cells. The engine already builds one very good parallel
radix sort and uses it for the spatial unit index.[^2] Sort the producers
by cell index once for each tick and reuse the result for every source
operator in that tick.

**No operator needs a scan.** No operator needs a local join beyond the
one that the source sort already performs.

### 9.2 The cost model

Calibrate against report 09, which measures 37,000 vector operations at 18
microseconds for one hex Jacobi iteration on a `u8` field of 65,536
cells.[^9] That is 2.06 billion vector operations for each second on a
Graviton core at about 2 GHz.[^14] Use **2.0 billion vector operations for
each second**.

A 128-bit NEON register holds 16 `u8` lanes, 8 `u16` lanes or 4 `i32`
lanes.[^11] Level 1 holds 65,536 cells.

| Field type | Vector groups at L1 |
|---|---|
| `u8` | 4,096 |
| `u16` | 8,192 |
| `Fix32` | 16,384 |

### 9.3 Operator cost at each level

| Operator | Ops for each group | `u8` at L1 | `Fix32` at L1 | `u8` at L0 | at L2 |
|---|---|---|---|---|---|
| `map`, that is scale, decay or clamp | 2 | 4.1 us | 16.4 us | 1.05 ms | 0.02 us |
| `gate` by a mask | 3 | 6.1 us | 24.6 us | 1.57 ms | 0.02 us |
| `diffuse`, one iteration | 9 | **18.4 us** | 73.7 us | **4.72 ms** | 0.07 us |
| `advect`, unlimited, one pass | 17 | — | 139 us | — | 0.54 us |
| `advect`, limited, three passes | 34 | — | **279 us** | — | 1.09 us |
| `restrict` to the next level | 2 | 4.1 us | 16.4 us | 1.05 ms | — |
| `prolong` from the next level | 2 | 4.1 us | 16.4 us | 1.05 ms | — |
| Separable recursion, 6 passes | 6 | 12 us | 50 us | 3.07 ms | 0.05 us |
| `source` scatter, 1,000 sources | 4 for each source | 2.0 us | 2.0 us | 2.0 us | 2.0 us |

**Level 2 is free.** Every operator at 256 cells costs under 2
microseconds. A field whose decay length exceeds 1,024 tiles belongs at
level 2 and costs nothing.[^9]

**Level 0 is unaffordable.** One diffusion iteration is 4.72 core-ms. The
record's whole per-tick budget is 90 to 360 core-ms and it allows two or
three full-map passes for each tick.[^2] A single field with 8 iterations
consumes 37 core-ms, which is 10 to 40 percent of the entire tick for one
field. **Place no field operator at level 0.**

### 9.4 Which fields live where

| Field | Kind | Level | Reason |
|---|---|---|---|
| Military presence | potential | L1 | decay length in the hundreds of tiles[^9] |
| Economic presence | potential | L1 | same |
| Pollution | conserved | L1 | spreads over tens of cells |
| Morale and rumour | potential | L1 | follows settlements, which are L1 features |
| Supply range | potential | L1 | a depot reaches tens of cells |
| Scent and heat | potential | L1 | the shortest decay length of the set |
| Price potential | intensive | L1 | markets are L1 features |
| Resource transport, aggregate | conserved | L1 | roads are L1 corridors |
| Migration pressure | potential | **L2** | acts over thousands of tiles, changes over hundreds of ticks |

**Every field is at level 1 or level 2. None is at level 0.** The rule
that decides it is report 09's sampling rule: use level 1 when the decay
length is at least 64 tiles and level 2 when it is at least 1,024
tiles.[^9] A quantity with a decay length of a few tiles is a local query,
and the batched neighbour sweep over the sorted unit index answers it at a
cost the record already budgets.[^2]

### 9.5 The per-tick budget

Take the nine fields of section 2. Give each a cadence. Amortise.

| # | Field | Type | Operators for one update | Cadence, ticks | Cost for one update | Core-us for each tick |
|---|---|---|---|---|---|---|
| 1 | Military presence, all tiers | `u8` | restrict, solve at L2, prolong, 8 diffuse | amortised | 150 us | 408 |
| 2 | Economic presence, tier R | `u8` | separable recursion | amortised | 12 us | 12 |
| 3 | Shared support work | mixed | all-faction sum, dominant pair, complement | mixed | — | 110 |
| 4 | Pollution | `Fix32` | source, 2 diffuse, decay | 4 | 180 us | 45 |
| 5 | Morale and rumour | `u8` | source, 4 diffuse, decay | 8 | 80 us | 10 |
| 6 | Supply range | `u8` | source, 6 diffuse with conductance | 16 | 112 us | 7 |
| 7 | Resource transport | `Fix32` | source, limited advect, sink | 4 | 312 us | 78 |
| 8 | Price potential | `Fix32` | 3 diffuse | 8 | 222 us | 28 |
| 9 | Migration pressure | `u8` at L2 | composite, gradient | 64 | 1 us | 0 |
| 10 | Scent and heat | `u8` | source, 3 diffuse | 8 | 58 us | 7 |
| | **Total** | | | | | **705** |

Rows 1 to 3 reproduce report 09's schedule and total 530 core-us, which is
its published figure of 0.53 core-ms.[^9] Rows 4 to 10 add 175 core-us.

**The whole field layer costs 0.71 core-ms for each tick.**

Apply the dirty-block restriction that report 09 defines. It reduces the
diffusion lines by about a factor of 3. It does not reduce the advection
line, because a transport field is dirty wherever a caravan moves.
**The restricted total is about 0.32 core-ms.**

### 9.6 Against the record's budget table

The record holds one line: influence maps at level 1, 8 maps, 1 to 3
core-ms and 0.1 to 0.3 wall-ms.[^2]

| Case | Core-ms | Wall-ms at 2 workers |
|---|---|---|
| Nine fields, no restriction | 0.71 | 0.36 |
| Nine fields, with the dirty-block restriction | 0.32 | 0.16 |
| The record's existing line | 1 to 3 | 0.1 to 0.3 |

**Nine fields fit inside the budget line that the record wrote for eight
influence maps.** The core-ms figure is well inside it. The unrestricted
wall figure is slightly above the upper bound, and the restricted figure
is inside it.

**Recommend renaming the line rather than adding one.** Change "Influence
maps at L1" to "Field operators at L1" and keep the 1 to 3 core-ms range.
Adding a second line would double-count the influence work, which is
already inside this total.

### 9.7 Memory

| Structure | Size at L1 |
|---|---|
| One `u8` field, dense | 64 KiB |
| One `Fix32` field, dense | 256 KiB |
| Edge residual store, for each conserved field | 384 KiB |
| Edge velocity store, for each advected field | 384 KiB |
| Worker scratch, 8 workers, 2 planes each[^9] | 1,024 KiB |

| Field | Store | Size |
|---|---|---|
| Pollution | field plus residual | 640 KiB |
| Resource transport | field plus residual plus velocity | 1,024 KiB |
| Price potential | field | 256 KiB |
| Morale, supply, scent | 3 `u8` fields | 192 KiB |
| Migration pressure at L2 | 256 B | 0.25 KiB |
| **Subtotal, the seven new fields** | | **2.06 MiB** |
| Influence, as report 09 states, at 1,024 factions[^9] | | 2.90 MiB |
| **Total field layer** | | **4.96 MiB** |

The figure is small against the record's tile grid at 160 MiB and its fog
of war at 50.7 MB.[^2][^5] **Memory is not the constraint on this design.
Per-tick time is.**

### 9.8 The frame loop

Phases 1 to 4 read the world and write only events. Phases 5 to 8 write
the world.[^2]

**Run every field operator at the end of phase 8, after fog and after
influence.** Every operator reads unit positions that phase 6 settled and
level 1 summaries that phase 7 produced. Every operator writes only
fields.

A selector or a controller that reads a field runs in phases 1 to 4 and
therefore reads the value that phase 8 produced on an earlier tick. The
lag is the cadence, not one tick. Report 09 shows every influence consumer
tolerates that lag.[^9] The same argument covers the other fields, because
their cadences are longer.

Give the module two types. `FieldRead` exposes the point query and the
plane view. `FieldWrite` exposes the operators. Phase 8 is the only phase
that receives `FieldWrite`. This mirrors the split that report 09 defines
for influence.[^9]

**One ordering constraint is new.** A conserved field must run its source
operators before its transport operator within one tick, because a source
that arrives after the transport would sit still for a whole cadence
window. Fix the order in the compiled schedule, not at run time.[^2]

---

## 10. The mask connection

### 10.1 The problem

A mask gates an operator. A branch for each cell defeats vectorisation on
the target, because a mispredicted branch costs more than the arithmetic
it protects. The design must gate without a branch for each cell.

### 10.2 Three techniques, ordered by preference

**Technique 1, best. Fold the mask into the coefficient.** A diffusion
already multiplies by a per-cell conductance. Set the conductance to zero
where the mask is clear. A zero coefficient produces a zero flux, so the
operator is gated exactly, and the cost is zero, because the multiply
already happens.

This covers every operator that has a coefficient: `diffuse`, `advect`,
`scale` and `decay`. It is the technique to reach for first.

State the one condition. The masked cell must still be a legal cell. A
zero conductance stops flow into and out of the cell, and the cell keeps
whatever it already holds. If the design wants the quantity destroyed
rather than trapped, add an explicit sink. **Do not confuse "no flow" with
"no quantity".**

**Technique 2. Expand the bit to a lane mask and combine bitwise.** For an
operator with no coefficient, such as a source term, convert the mask bits
to a byte mask and apply a bitwise AND.

```
load    one u8 from the bitplane          // 8 cells
dup     it across 8 lanes
and     with the constant [1,2,4,8,16,32,64,128]
cmeq    against zero, then invert         // 0x00 or 0xFF for each lane
and     with the operand
```

That is 4 NEON instructions for 8 lanes, so about 0.5 instructions for
each cell. At level 1 it adds about 2 microseconds to a pass. There is no
branch.

**Technique 3. Dispatch at the block, never at the cell.** The engine
already stores a plane as 256 level 2 blocks with an `Empty` leaf and a
`Dense` leaf.[^9] Store the mask the same way and test the leaf form once
for each block.

| Block mask state | Action | Cost |
|---|---|---|
| All bits clear | Skip the block entirely | 0 |
| All bits set | Run the ungated kernel | no gate cost |
| Mixed | Run the gated kernel with technique 2 | the technique 2 cost |

**That is 256 branches for each plane pass, not 65,536.** It is the
correct granularity, it reuses the container that fog and influence
already define, and it makes a sparse mask cheaper than a dense one rather
than the same price.

### 10.3 How each mask kind parameterises an operator

| Mask kind | Source | Parameterises | Technique |
|---|---|---|---|
| Terrain class | the tile terrain field, restricted to L1 | the conductance of `diffuse` | 1 |
| Road bitplane | the tile flags, restricted to L1 | the conductance, raised on a road | 1 |
| Faction ownership | the L1 faction mask[^5] | which sources contribute | 2 |
| Diplomacy relation | the relation plane rows[^5] | which factions a threat sum subtracts | 2, on the plane, not on the grid |
| Fog of war | the faction visible set[^5] | which cells a Python view exposes | 3 |
| Dirty block | the L1 and L2 dirty bitsets[^2] | which blocks an operator visits | 3 |
| Water and land | a terrain-derived bitplane | the conductance, zero across a coast | 1 |

**Five of seven use technique 1 or technique 3, and both are free.** Only
the two faction masks need the lane expansion, and one of those applies to
a 1,536-byte relation plane rather than to the grid.[^5]

### 10.4 The rule to record

**Prefer a coefficient of zero to a gate. Prefer a block skip to a lane
mask. Use a lane mask only for an operator with no coefficient, and only
inside a mixed block.**

---

## 11. Prior art

### 11.1 Numerical methods

**Finite volume methods.** The standard reference treats conservation laws
on unstructured meshes, and states the property this report depends on:
a flux computed once for each face and applied with opposite signs to the
two neighbouring cells conserves the integral exactly, independently of
the accuracy of the flux.[^6] The project's contribution is to notice that
this property survives integer truncation, which matters far more here
than in floating-point science code.

**The Courant-Friedrichs-Lewy condition.** The original 1928 paper
establishes that an explicit scheme for a hyperbolic equation is stable
only if the numerical domain of dependence contains the physical one.[^15]
Section 4.2 derives the hex form of the bound.

**Flux limiting and monotonicity.** Total-variation-diminishing schemes
and slope limiters exist to prevent spurious oscillation in high-order
schemes near a discontinuity.[^12] Section 4.4 rejects them for this
project, because the engine uses a first-order upwind flux which does not
oscillate. The engine's limiter is a positivity limiter, which is a
different and simpler device.

**Discrete exterior calculus.** The framework places scalars on cells,
fluxes on edges and circulations on faces, and gives exact discrete
analogues of the Stokes theorem.[^7] It is the correct framework if the
project later needs a vector field with a curl, such as wind with
vorticity. It is heavier than the problem in section 2.

**Reaction-diffusion systems.** The founding paper shows that a diffusing
system with a nonlinear reaction term produces stable spatial patterns
from a uniform start.[^16] This is the direct ancestor of the `S(phi)`
term in section 6.4, and it is a warning: a semi-linear system produces
structure that no designer asked for. Test any state-dependent source term
for pattern formation before shipping it.

### 11.2 Lattice methods

**Lattice gas cellular automata.** The HPP model of 1973 moved particles
on a square lattice and failed to reproduce the Navier-Stokes equation,
because the square lattice does not satisfy the isotropy condition.[^17]
The FHP model of 1986 moved to a hexagonal lattice and succeeded, for
exactly that reason.[^8] **This is the strongest independent evidence for
section 3.3's result.** The advantage of 6-fold symmetry over 4-fold
symmetry is not a game-development observation. It is a published result
in statistical physics, and it decided the design of a whole method.

**Lattice Boltzmann.** The method replaces the noisy particle occupation
of a lattice gas with a real-valued distribution over the lattice
velocities.[^18] It removes the noise at the cost of 7 stored values for
each cell and a relaxation parameter that is a division. Section 3.1
rejects it on cost, not on correctness.

**Conservative cellular automata.** The Margolus neighbourhood partitions
the lattice into non-overlapping blocks and updates each block by a
permutation, which conserves particle count exactly by
construction.[^19] It is the cellular-automaton analogue of the flux pair,
and it reaches the same conclusion from a different direction: **make
conservation structural, not arithmetical.**

### 11.3 Graphics and animation

**Stable fluids.** The method makes an explicit fluid solver
unconditionally stable by tracing characteristics backward and
interpolating, which is semi-Lagrangian advection.[^20] It removes the
Courant limit entirely. **Reject it for this project, for a specific
reason:** semi-Lagrangian advection interpolates at a point that is not a
grid node, so it does not conserve. It is designed for a visual result,
where a small loss of mass is invisible. A resource-transport field that
silently loses goods is a defect the player will find.

**Continuum crowds.** The method treats a crowd as a density field driven
by a potential, and computes a flow field once for a whole group rather
than a path for each agent.[^21] The record already takes one idea from
it, that unit density raises the local path cost.[^2] It is the closest
prior art to the project's own crowd design.

### 11.4 Game artificial intelligence

**Influence maps.** The idea originates in computer Go, where a stone's
influence spreads over the board and decays with distance, and the
resulting field decides territory.[^22] The technique reached mainstream
game development as a general spatial-reasoning tool for threat, safety
and desirability.[^23] Report 09 is the project's treatment.[^9]

**Artificial potential fields.** The original robotics formulation gives
an agent a goal that attracts and obstacles that repel, and moves the
agent down the gradient.[^24] The known defect is the local minimum, where
an agent stops in a bowl formed by several repulsors. **This defect
applies to any project mechanic that follows a field gradient.** The
project's mitigation exists already: the portal graph supplies the global
plan and the field supplies only the local behaviour.[^2]

### 11.5 Shipped simulation games

**SimCity.** The original design layered several tile fields over one map:
land value, pollution, crime, population density and traffic. Each field
was updated on its own cadence, and each read others as inputs. The
published source of the later Micropolis release shows the pattern
directly.[^25] **This is the closest existing example of the owner's
proposal, and it worked.** It also shows the constraint: the map was small,
in the low tens of thousands of tiles, and the fields were updated every
few frames rather than every frame.

**Factorio.** The original fluid system equalised pressure between
adjacent pipe segments, which is a local diffusion. It was replaced. The
replacement merges connected pipes into a segment with one shared volume
and one fill level, and makes the extraction rate a function of that
level.[^3] **The lesson is section 7.4's:** a design that wants
equalisation within one tick needs a connected-component aggregate, not a
stencil. Factorio also documents the earlier local design and the reasons
it was unsatisfactory.[^26]

**Dwarf Fortress.** Water occupies a tile at one of 7 discrete levels,
which is an integer field with a very coarse quantum. Flow between
adjacent tiles is a local rule. Movement under pressure traces a path
through already-full tiles and emerges at the far end without generating
flow along the way.[^4] **The design contains both models at once:** a
local field for ordinary flow and a non-local path trace for pressure.
That is the same split this report recommends in section 7.4, reached by a
different route and under different constraints.

### 11.6 Scientific and agent-based modelling

**Cellular-automaton urban growth.** The SLEUTH model grows an urban area
by applying a small set of local transition rules to a raster, calibrated
against historical imagery.[^27] It is the direct ancestor of a
field-based settlement or migration mechanic, and it establishes that a
few local rules on a coarse raster reproduce large-scale spatial pattern.

**Agent-based modelling frameworks with field layers.** The main
general-purpose frameworks all provide a raster layer beside the agent
population, and let agents read and write it.[^28] **The architecture that
these frameworks converged on is exactly the hybrid of section 7.6:**
discrete agents with identity, plus continuous field layers, with the
agents reading the fields and depositing into them. That convergence is
evidence that the hybrid is the right shape, and that neither model alone
is sufficient.

### 11.7 What the survey establishes

Three conclusions.

1. **The unification is not novel and it is not speculative.** Finite
   volume, reaction-diffusion, lattice methods and layered tile simulation
   are all mature, and they all use the same three terms.
2. **The hex advantage is a published physics result, not a preference.**
   The FHP model exists because the square lattice failed.[^8][^17]
3. **Every large system that tried a pure field model added a non-local
   escape.** Factorio added segments. Dwarf Fortress added pressure
   tracing. Agent-based frameworks kept the agents. **Plan the escape
   before it is needed.**

---

## 12. Proposed decision text

The text below is a new decision for the draft record, plus two
amendments. Do not apply it to the record. The record is a draft under
review.

---

> #### D52. Field mechanics use finite volume on the hex edges, with a shared kernel layer and no runtime operator graph
>
> **The unification holds.** Influence, economic presence, pollution,
> morale, price potential, aggregate resource transport, migration
> pressure, supply range and scent are one framework with different
> coefficients. The framework is the advection-diffusion-reaction equation
> on the level 1 hex lattice. Nine mechanics fit without forcing. Two of
> the nine, price and resource transport, need the caveats stated below.
>
> **Adopt finite volume. Reject finite difference.** Compute one integer
> flux for each hex edge. Apply it as a signed pair: subtract it from one
> cell and add the same integer to the other. **Conservation is then exact
> for any flux function, any rounding rule, any visit order and any worker
> count**, because integer addition is exactly associative and
> commutative. Finite difference does not have this property, because it
> rounds the outflow and the inflow separately. This is the decisive
> reason, and it exists because D4 bans floating point.
>
> **Reject discrete exterior calculus, lattice gas and lattice Boltzmann
> for version 1.** Discrete exterior calculus produces the same arithmetic
> as finite volume for a scalar problem and adds a vocabulary; adopt it
> only if a mechanic needs a vector field with a curl. A lattice gas is
> exactly conservative but statistically noisy, and the engine needs a
> smooth field from a small grid. Lattice Boltzmann stores 7 values for
> each cell and needs a division.
>
> **The hex lattice is quantitatively more isotropic, and this is a
> measurement.** The relative directional error of a stencil, as a
> peak-to-peak fraction of the leading term, at wavenumber `k` radians for
> each cell:
>
> | Stencil | Taps | Weight classes | Error | at k = 1.0 |
> |---|---|---|---|---|
> | Square, 5 points | 5 | 1 | `k^2 / 24` | 4.2% |
> | Square, 9 points, isotropic | 9 | 2 | `k^4 / 720` | 0.14% |
> | Hex, 7 points | 7 | 1 | `k^4 / 2880` | **0.035%** |
>
> Six-fold symmetry cancels the fourth-rank anisotropy that four-fold
> symmetry cannot. The hex 7-point stencil beats the best square stencil
> by a factor of 4 with two fewer taps and one weight class. The FHP
> lattice gas moved from the square lattice to the hexagonal lattice for
> exactly this reason.
>
> **Positivity binds before stability, and it fixes one constant.** For a
> Jacobi diffusion with weight `alpha` on each of the 6 neighbours,
> stability needs `alpha <= 2/9` and non-negativity needs
> `alpha <= 1/6`. **Set `alpha <= 1/6`.** A diffusion that obeys this
> bound cannot produce a negative value and therefore needs no flux
> limiter. In diffusion-number terms both hex and square admit
> `D dt / h^2 <= 1/4`, so hex costs no timestep. The hex advection limit
> is `|v| dt / h <= 3/4`, which no game velocity approaches; make it a
> debug assertion, not a runtime test.
>
> **Two fixed-point scales, and conservation decides which.**
>
> | Field kind | Scale | Type at L1 | Bytes |
> |---|---|---|---|
> | Conserved and extensive | Q16.16 | `Fix32` | 4 |
> | Potential, compared only | Q0.8 against a constant | `u8` | 1 |
> | Intensive with a read magnitude, such as a price | Q16.16 | `Fix32` | 4 |
>
> A conserved field must agree with an entity inventory, so it uses the
> project scale of D4. The 16 fractional bits are headroom against
> truncation, not precision for the designer: they put the truncation
> floor at one good for each 65,536 ticks. A potential field keeps the
> `u8` that D51 already justifies, because no consumer reads its
> magnitude.
>
> **The rounding policy has two cases.** A truncating flux loses the
> remainder on every edge on every tick. The loss is not a loss of mass,
> because the pair still balances, but it freezes a gradient below one
> quantum into a permanent staircase.
>
> | Field kind | Mechanism | Extra store at L1 | Long-run bias |
> |---|---|---|---|
> | Potential | rely on Q16.16 or Q0.8 headroom | none | present, and intended as the support bound |
> | Conserved | **residual carry**: store the remainder for each edge in a persistent `i16` and add it back next tick | 384 KiB for each field | **none** |
>
> Reject deterministic dithering. It is deterministic, but it injects
> noise into a field whose purpose is smoothness and it dirties a block
> that should be clean. **Use a truncating right shift by a compile-time
> constant. Never divide by a runtime variable**, as D51 also requires.
>
> **The flux limiter runs in three passes and applies to advection only.**
> Pass 1 computes the unlimited flux for each edge. Pass 2 reduces, for
> each cell, the total outflow, and forms an integer scale factor. Pass 3
> applies the scaled fluxes as pairs, using an `i64` intermediate and a
> truncating divide. The unsent remainder stays in the source cell and is
> offered again next tick. Reject slope limiters and total-variation-
> diminishing schemes; the engine uses a first-order upwind flux, which
> does not oscillate.
>
> **The algebra has three composition laws. Give each its own trait.**
>
> | Law | Members | Composition | Project term |
> |---|---|---|---|
> | Group | `source`, `sink`, `combine(add)` | integer addition; commutes, associates, has an inverse | **rate** |
> | Semilattice | `clamp`, `gate`, `combine(min)`, `combine(max)` | idempotent, commutative, associative, no inverse | **constraint** and **set** |
> | Matrix | `diffuse`, `advect`, `restrict`, `prolong` | matrix product; associates, does not commute | — |
>
> **The most likely defect is to write a constraint as a rate.** A
> capacity cap written as a sink composes by addition, so two caps
> subtract twice. A cap is a `clamp`, which is idempotent. The traits
> prevent this at compile time.
>
> **Four operator orderings do not commute and are easy to get wrong.**
> `gate` before `diffuse` zeroes a source; `gate` after `diffuse` clips
> the spread; both are useful and they differ. `diffuse` then `advect`
> differs from the reverse by a first-order splitting error. `source` then
> `diffuse` differs from the reverse by one diffusion of the source.
> **Two `diffuse` calls of `n` and `m` iterations do not equal one call of
> `n + m` iterations in integer arithmetic**, because each call rounds.
>
> **Six type rules, all checkable by the compiler.** Only an extensive
> field may be advected or diffused as a quantity. `restrict` sums an
> extensive field and takes a sum-and-count mean of an intensive one.
> `prolong` on an extensive field divides and hands the remainder to the
> lowest-indexed child. Two fields combine only at the same level and the
> same kind. **`clamp` breaks conservation, so it must never touch a
> conserved field**; use the flux limiter. A conserved field has exactly
> one writer for each operator class.
>
> **Iterate. Do not solve.** One operator application is a sparse
> matrix-vector product with 7 non-zero entries in each row. Iterating is
> the power method. **No consumer needs a true steady state**: nine of ten
> consumers compare, rank or take a gradient, and a partially relaxed
> field preserves the order of two well-separated cells. **Reject Krylov
> methods**, because the step length is a division of two reductions,
> which truncates in integer arithmetic and destroys the optimality that
> makes them converge quickly, and because each iteration adds two global
> barriers. **Accept multigrid**: D51 already uses a two-grid cycle and
> measures 150 microseconds against 590 for 32 plain Jacobi iterations,
> reusing the D17 pyramid. **Fix every iteration count as a compile-time
> constant**, as D51 requires.
>
> **State the limits of linearity.** Saturation, capacity caps,
> state-dependent sources, the flux limiter and field-dependent masks all
> break linearity. The model is semi-linear at best. **Superposition holds
> only below every saturation and every threshold**, so D51's closed-form
> falloff scatter is valid only in that range; state the condition where
> the method is used. A state-dependent source makes the system a
> reaction-diffusion system, which forms spatial patterns nobody asked
> for; test for that before shipping one.
>
> **The membership test. A mechanic belongs inside the framework only if
> all five answers are yes.**
>
> 1. **Fungible.** Are two units interchangeable? A named thing is not.
> 2. **Local.** Does the change depend only on a cell and its 6
>    neighbours?
> 3. **Divisible.** Is a fractional value meaningful at the tick scale?
> 4. **Additive.** Is the sum or the mean over a region meaningful?
> 5. **Dense.** Is the quantity in enough cells to pay for a full pass?
>
> **These stay outside:** a named unit, a specific building, a treaty, a
> duel between heroes, a construction order, a technology tree, a
> contract, an individual inventory, an ownership transfer, a quest, a
> siege, and any population of fewer than about 5,000 agents at level 1.
>
> **Combat between individuals is not advection.** Advection moves a
> conserved quantity and changes no total. Combat destroys quantity on
> both sides at a rate that depends on both, with thresholds. Army-scale
> attrition is a legitimate **pointwise reaction term** of the Lanchester
> kind, with no transport. It cannot express a named participant and it
> averages away what a player watches. **Run the field model where nobody
> is watching and the entity model where somebody is**, using the same
> three faction tiers that D48 and D51 define. The conversion between the
> two at a tier boundary is an open question.
>
> **Instant equalisation is not a field operation.** A local stencil
> propagates one cell for each iteration, so no fixed iteration count
> equalises a connected region within one tick. Two shipped games reached
> the same conclusion: Factorio replaced per-tile fluid pressure with
> connected segments holding one shared level, and Dwarf Fortress moves
> water under pressure by tracing a path through already-full tiles.
> **The test to apply to a designer's request:** ask how many ticks the
> quantity may take to settle across the network. "Instant" means a
> connected-component solve on the portal graph of D44, not a diffusion.
>
> **The density break-even, as arithmetic.** One diffusion iteration at
> level 1 costs 18 microseconds for 65,536 cells, which is 0.27
> nanoseconds for each cell. A per-unit update costs about 30 nanoseconds
> (D45).
>
> | Operator | Field cost at L1 | Entities that cost the same |
> |---|---|---|
> | 1 map pass | 4.1 us | 140 |
> | 1 diffusion iteration | 18 us | 600 |
> | 8 diffusion iterations | 144 us | **4,800** |
> | 8 diffusion iterations **at L0** | **37 ms** | **1,230,000** |
>
> **No field operator runs at level 0.** Eight iterations there cost 37
> core-ms against a whole-tick budget of 90 to 360 core-ms, and the
> break-even against entities exceeds the target unit count. This confirms
> D51's level 0 prohibition by an independent argument.
>
> **Build a shared kernel layer. Do not build a runtime operator graph.**
> The shared layer holds the flux applicator, the residual carry, the
> limiter, the mask convention, the field type distinction, the rounding
> policy and the level dispatch: about 700 to 1,000 lines. **It is
> justified at two or more conserved fields**, and the project has at
> least two, resource transport and pollution. A runtime graph adds 1,500
> to 2,500 lines, **hides the four non-commuting orderings listed above**,
> writes every intermediate to memory instead of fusing into one loop, and
> duplicates the compile-time generation that D18 already performs.
> Its break-even is above ten designer-authored field mechanics, and the
> project has nine, two of them marginal.
>
> **Express a mechanic as a Rust function that composes typed operators,
> monomorphised at compile time. Extend the D18 field registry to generate
> the restriction combine, the summary slot and the Python view for each
> field.** Revisit only if a designer must add a field mechanic without a
> Rust change and the count passes ten. That is the D43 extensibility
> ladder, and the answer is the same: stage it, do not build it first.
>
> **Extract the shared layer from two working kernels. Do not write it
> first.** Build D51's influence kernel as a specific kernel. Build the
> second field the same way. Then extract. This costs one refactor and it
> removes the risk of designing the abstraction against a guess, in the
> same way that D17 requires the flat scan path before the pyramid
> descent.
>
> **Gate an operator without a branch for each cell, by three techniques
> in this order.**
>
> | Rank | Technique | Cost | Use for |
> |---|---|---|---|
> | 1 | **Set the coefficient to zero** where the mask is clear | **zero**; the multiply already happens | `diffuse`, `advect`, `scale`, `decay` |
> | 2 | Expand the bit to a lane mask and combine bitwise: dup, AND with the bit constants, compare against zero, AND with the operand | 4 NEON instructions for 8 lanes, about 2 us for each L1 pass | `source`, and any operator with no coefficient |
> | 3 | **Dispatch at the level 2 block**: skip an all-clear block, run the ungated kernel on an all-set block, gate only a mixed block | 256 branches for each plane pass, not 65,536 | every operator; reuses the D51 container |
>
> A zero coefficient stops flow into and out of a cell and leaves whatever
> the cell already holds. **Do not confuse "no flow" with "no quantity".**
> If the design destroys the quantity, add an explicit sink.
>
> **Every operator maps onto the existing kernel vocabulary and needs no
> atomic.** `source` and `sink` are a sort then a scatter then a reduce,
> and they reuse the D50 radix sort. `diffuse`, `advect` and `gradient`
> are stencils. `scale`, `clamp`, `gate` and `combine` are maps.
> `restrict` is a reduce and `prolong` is a map. No operator needs a scan.
> Partition by level 2 block, give each worker a disjoint set, and let the
> owner of the lower-numbered block write a boundary edge. This satisfies
> D1's preference for disjoint outputs over atomics.
>
> **Determinism.** Visit each edge exactly once and store 3 edge classes
> for each cell. Process blocks in ascending level 2 block number and
> edges in ascending edge class. Sort every source list by cell index and
> then by a stable secondary key. Accumulate a source scatter in `i64` and
> store back as `Fix32`, as D4's accumulator-width rule requires. Add two
> tests: one that counts edge visits and fails on any count other than
> one, and one that sums a conserved field before and after each tick and
> fails on any difference.
>
> **Every field lives at level 1 or level 2.** Use level 1 when the decay
> length is at least 64 tiles and level 2 when it is at least 1,024 tiles,
> which is D51's sampling rule. Migration pressure is the only level 2
> field, at 256 bytes. Level 2 is free: every operator there costs under 2
> microseconds.
>
> **Run every field operator at the end of phase 8 of D28, after fog and
> after influence.** Give the module a `FieldRead` type and a `FieldWrite`
> type, and give `FieldWrite` to phase 8 only. **Within one tick, run the
> source operators of a conserved field before its transport operator**;
> fix the order in the D27 compiled schedule, not at run time. A consumer
> in phases 1 to 4 reads the value that phase 8 produced on an earlier
> tick; the lag is the cadence, not one tick.
>
> **Cost, at nine fields on their own cadences.**
>
> | Case | Core-ms for each tick | Wall-ms at 2 workers |
> |---|---|---|
> | Nine fields, no restriction | 0.71 | 0.36 |
> | Nine fields, with the dirty-block restriction | 0.32 | 0.16 |
>
> Of the 0.71, D51's influence schedule is 0.53 and the seven other fields
> add 0.18. **Rename the budget line "Influence maps at L1" to "Field
> operators at L1" and keep its 1 to 3 core-ms range. Do not add a second
> line**, because that would double-count the influence work.
>
> **Memory: 4.96 MiB for the whole field layer at 1,024 factions**, of
> which 2.90 MiB is D51's influence total and 2.06 MiB is the seven other
> fields. A conserved field costs 256 KiB for the field plus 384 KiB for
> the edge residual store; an advected field costs a further 384 KiB for
> the edge velocity. Memory is not the constraint on this design.
>
> **Revisit trigger.** Adopt a runtime operator graph only when a
> designer must add a field mechanic without a Rust change and the count
> of such mechanics passes ten. Adopt discrete exterior calculus only when
> a mechanic needs a vector field with a curl. Move a field to level 0
> never; the arithmetic above forbids it in every case.

---

> #### Amendment to the per-tick cost budget table
>
> Replace the line "Influence maps at L1 | 8 maps | 1-3 | 0.1-0.3" with
> the following line.
>
> | Work | Scale | Core-ms | Wall-ms (12c) |
> |---|---|---|---|
> | Field operators at L1 | 9 fields | 1-3 | 0.1-0.3 |
>
> The measured schedule is 0.71 core-ms without the dirty-block
> restriction and 0.32 core-ms with it. Nine fields therefore fit inside
> the range the record wrote for eight influence maps. Do not add a second
> line for the non-influence fields; the influence work is already inside
> this total.

---

> #### Amendment to the memory budget table
>
> Add these lines after the influence lines that D51 introduces.
>
> | Structure | Size |
> |---|---|
> | Conserved field at L1, `Fix32` | 256 KiB each |
> | Edge residual store, for each conserved field | 384 KiB each |
> | Edge velocity store, for each advected field | 384 KiB each |
> | Field layer total at 1,024 factions, 9 fields | 4.96 MiB |
>
> The edge stores are the only structures this decision adds that the
> record does not already hold. Each is 3 edge classes over 65,536 level 1
> cells at `i16`. The residual store removes the long-run truncation bias
> in a conserved field and it has no substitute.

---

## 13. Open questions from this report

1. **How do a field model and an entity model reconcile at a tier
   boundary?** Section 7.3 says to run abstracted combat for a passive
   faction and per-entity combat for a rendered one. When the two meet,
   the engine must convert between a strength field and a unit list.
   **The measurement that decides the design:** run the same engagement
   under both models 1,000 times and compare the outcome distributions.
   The conversion is acceptable if the means agree within 5 percent.

2. **Does the price mechanic need a true steady state?** Section 6.3
   places price as the only consumer that reads a magnitude, and section
   2.3 says to diffuse the goods and derive the price. **The measurement:**
   compare a derived price against a solved equilibrium price over 1,000
   ticks and count the ticks on which a trade decision differs. Accept the
   derived form if the rate is under 1 percent.

3. **What is the real decay length of each new field?** Every cost in
   section 9.5 assumes a cadence, and every cadence follows from a decay
   length and a source speed. A designer must set them. Until then, use
   report 09's default of a halving length of 4 level 1 cells.

4. **Is the residual carry store worth 384 KiB for each conserved field?**
   Section 4.5 argues that truncation freezes a sub-quantum gradient.
   **The measurement:** run a transport field for 100,000 ticks with and
   without the carry, and compare the final distribution against an `i64`
   reference. Drop the carry if the difference stays under one quantum for
   each cell.

5. **Do reports 10, 11 and 12 agree?** They were not written when this
   report was written, so this report could not reconcile with them. The
   three claims this report makes that most affect them are: the density
   break-even of about 5,000 agents at level 1 (report 10 and report 11);
   the rule that a caravan is an entity and the corridor flow is a derived
   field (report 11); and the rule that a capacity cap is a constraint and
   not a negative rate (report 12). **Check each against those reports
   before the record adopts this decision.**

6. **Should pollution advect with wind?** Section 2.1 marks wind advection
   as optional. Advection costs 279 microseconds against 74 for a
   diffusion iteration, and it is the only term that needs the limiter and
   the velocity store. **Decide it against a design requirement, not
   against a cost.** If pollution need only spread, diffusion is enough.

7. **Does any mechanic need a vector field with a curl?** Section 3.1
   defers discrete exterior calculus on the grounds that no mechanic
   does. Wind with vorticity, ocean currents and a circulating trade route
   would each change the answer.

---

## References

[^1]: Cachette project instructions, sections "Hard invariants" and "Design principles". `CLAUDE.md`
[^2]: ADR-0001, Foundational Architecture, decisions D1, D4, D5, D9, D16, D17, D18, D25, D27, D28, D29, D43, D44, D45, D50, the byte budget tables and the per-tick cost budget. `docs/adrs/draft/adr-0001-foundational-architecture.md`
[^3]: Wube Software, 2024. "Friday Facts #416 — Fluids 2.0". Factorio development blog. https://www.factorio.com/blog/post/fff-416
[^4]: Dwarf Fortress Wiki, "DF2014:Pressure" and "DF2014:Flow". https://dwarffortresswiki.org/index.php/DF2014:Pressure
[^5]: Research report 08, Fog of War Representation, sections 6.1, 6.3, 6.4 and 8. `docs/adrs/background/adr-0001/08-fog-of-war-representation.md`
[^6]: LeVeque, R. J., 2002. *Finite Volume Methods for Hyperbolic Problems*. Cambridge University Press, chapters 4 and 19.
[^7]: Desbrun, M., Hirani, A. N., Leok, M. and Marsden, J. E., 2005. "Discrete Exterior Calculus". arXiv preprint math/0508341. https://arxiv.org/abs/math/0508341
[^8]: Frisch, U., Hasslacher, B. and Pomeau, Y., 1986. "Lattice-Gas Automata for the Navier-Stokes Equation". *Physical Review Letters*, 56(14), pp. 1505-1508. https://doi.org/10.1103/PhysRevLett.56.1505
[^9]: Research report 09, Influence Maps, sections 4.1, 5.1, 6.1, 6.3, 6.5, 7.1, 7.3 and 12. `docs/adrs/background/adr-0001/09-influence-maps.md`
[^10]: Research report 02, Hex Grid and LOD Pyramid, sections 1.2, 1.3 and 2.3. `docs/adrs/background/adr-0001/02-hex-grid-and-lod-pyramid.md`
[^11]: Research report 07, Target Platform and Value Types. `docs/adrs/background/adr-0001/07-target-platform-and-value-types.md`
[^12]: Sweby, P. K., 1984. "High Resolution Schemes Using Flux Limiters for Hyperbolic Conservation Laws". *SIAM Journal on Numerical Analysis*, 21(5), pp. 995-1011.
[^13]: Research report 06, Algorithms and Scheduling, sections 8.3, 8.4, 8.5, 8.6, 9.1 and 10. `docs/adrs/background/adr-0001/06-algorithms-and-scheduling.md`
[^14]: Arm Ltd. *Arm Neoverse N1 Software Optimization Guide*, instruction throughput and memory system tables. https://developer.arm.com/documentation/swog309707/latest
[^15]: Courant, R., Friedrichs, K. and Lewy, H., 1928. "Über die partiellen Differenzengleichungen der mathematischen Physik". *Mathematische Annalen*, 100(1), pp. 32-74.
[^16]: Turing, A. M., 1952. "The Chemical Basis of Morphogenesis". *Philosophical Transactions of the Royal Society of London B*, 237(641), pp. 37-72.
[^17]: Hardy, J., Pomeau, Y. and de Pazzis, O., 1973. "Time Evolution of a Two-Dimensional Model System. I. Invariant States and Time Correlation Functions". *Journal of Mathematical Physics*, 14(12), pp. 1746-1759.
[^18]: Chen, S. and Doolen, G. D., 1998. "Lattice Boltzmann Method for Fluid Flows". *Annual Review of Fluid Mechanics*, 30, pp. 329-364.
[^19]: Toffoli, T. and Margolus, N., 1987. *Cellular Automata Machines: A New Environment for Modeling*. MIT Press, chapter 12.
[^20]: Stam, J., 1999. "Stable Fluids". *Proceedings of SIGGRAPH 1999*, pp. 121-128.
[^21]: Treuille, A., Cooper, S. and Popović, Z., 2006. "Continuum Crowds". *ACM Transactions on Graphics (Proceedings of SIGGRAPH 2006)*, 25(3), pp. 1160-1168.
[^22]: Zobrist, A. L., 1969. "A Model of Visual Organization for the Game of Go". *Proceedings of the AFIPS Spring Joint Computer Conference*, pp. 103-112.
[^23]: Tozour, P., 2001. "Influence Mapping". In *Game Programming Gems 2*, edited by M. DeLoura, Charles River Media, pp. 287-297.
[^24]: Khatib, O., 1986. "Real-Time Obstacle Avoidance for Manipulators and Mobile Robots". *International Journal of Robotics Research*, 5(1), pp. 90-98.
[^25]: Electronic Arts and Don Hopkins, 2008. *Micropolis*, the released source of the original SimCity. Simulation layer sources. https://github.com/SimHacker/micropolis
[^26]: Wube Software, 2018. "Friday Facts #260 — New fluid system". Factorio development blog. https://factorio.com/blog/post/fff-260
[^27]: Clarke, K. C., Hoppen, S. and Gaydos, L., 1997. "A Self-Modifying Cellular Automaton Model of Historical Urbanization in the San Francisco Bay Area". *Environment and Planning B: Planning and Design*, 24(2), pp. 247-261.
[^28]: Wilensky, U., 1999. *NetLogo*. Center for Connected Learning and Computer-Based Modeling, Northwestern University; and Luke, S., Cioffi-Revilla, C., Panait, L., Sullivan, K. and Balan, G., 2005. "MASON: A Multiagent Simulation Environment". *Simulation*, 81(7), pp. 517-527.
