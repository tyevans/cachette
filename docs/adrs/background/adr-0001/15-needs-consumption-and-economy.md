# Needs, Consumption and the Input-Output Economy

Research report 15 for ADR-0001. This report covers **consumption, needs,
wages, exchange and institutional solvency**. It covers the chain that links
them: a person decays, a person consumes, a position pays money, and a
governing body that cannot pay fails.

## Scope

This report decides eight things.

1. Which population carries a need, and at what granularity the engine
   simulates it.
2. Which needs exist, at what decay rate, and in what fixed-point scale.
3. How resource quality changes need satisfaction.
4. Whether money is a commodity or a separate quantity.
5. How a wage flows from an institution to a population without a
   per-individual transaction.
6. How a cohort converts money into food.
7. Whether Leontief input-output analysis is the correct model, and whether
   a failed solve is a usable insolvency test.
8. How a governing body fails, in what order, and what drives loyalty.

This report does **not** decide production rates, upkeep kernels, the
modifier pipeline, or where a stockpile lives. Another report owns those.[^1]
It does not decide transport, arc capacity or the flow solver. Another
report owns those.[^2] It does not decide character identity, offices or
succession. Another report owns those.[^3] This report states the interface
to each and does not redesign it.

## Context that this report assumes

Cachette is a world simulation engine. The core is Rust. The control plane
is Python. The world holds 16.7 million hex tiles and up to one million
units. The deployment target is an AWS Graviton server. The tick rate is
10 Hz.[^4]

Six rules from the foundational architecture record govern every
recommendation below.[^4]

- **No floating point in simulated or aggregated state.** The fixed-point
  scale is Q16.16. `Fix32` is `i32`. `Fix64` is `i64`. `Accum` is always
  `i64`.
- **Determinism is bit-exact for one binary at any thread count.** Iterate
  in a stated order. Use a stable sort key. Never use thread completion
  order. Never use hash iteration order.
- **The frame loop splits reads from writes.** The read phase writes only
  events. The write phase reads only events and columns.
- **Never stop on a convergence test.** Fix every iteration count as a
  compile-time constant.[^5]
- **Per-unit work has a hard ceiling of about 400 nanoseconds** of core
  time at one million units, shared between movement, combat and planning.
- **An intensive quantity aggregates as a pair.** Store the sum of value
  times weight, and the sum of weight. Divide at read.[^6]

Two facts from the neighbouring reports are load bearing here.

- The `transfer` verb conserves exactly. It sums the demand, computes one
  scale, writes the floor, then distributes the remainder by the
  largest-remainder method in a canonical order. Consumption caps route
  through it in priority bands.[^1]
- Prices move by one clamped tatonnement step for each tick. The engine
  never runs a converged price solve. The price band is one quarter to
  seven quarters of a base price.[^2]

---

## 1. Terms used in this report

| Term | Meaning in this report |
|---|---|
| **Individual** | One simulated person. The engine does not give an individual a row. |
| **Cohort** | A row that holds a headcount and one shared state vector. The unit of consumption and of decision. |
| **Stratum** | The class label of a cohort. It selects the need weights and the wage rate. |
| **Pool** | A settlement stockpile. It holds one `i64` stock for each commodity.[^1] |
| **Institution** | An entity with a treasury, an obligation list and a set of offices. A governing body is an institution. |
| **Need** | A `Fix32` scalar in Q16.16. 65,536 means satisfied. 0 means unmet. |
| **Technical coefficient** | The quantity of commodity `i` that one unit of commodity `j` needs as an input. |
| **Gross output** | The total production that an economy needs to meet a final demand after every intermediate use. |
| **Arrears** | Money that an institution owes and has not paid. |

---

## 2. Executive summary

**Nine findings.**

1. **The Leontief framing holds as a planning and pricing model. It fails
   as an insolvency test.** Input-output analysis correctly describes the
   intermediate demand chain, and the truncated Neumann series is the right
   integer solver for it. But the Hawkins-Simon condition is a property of
   the recipe data, not of an institution's finances. Every faction that
   shares a recipe table shares the condition. A failed solve therefore
   means the content is broken, not that a governing body is bankrupt.
   Section 3.4 states this in full.
2. **Make the Hawkins-Simon check a bake-time content validator.** It is the
   correct test for "this recipe set describes an impossible economy". Run
   it once when the engine bakes the type tables. Fail the build.
3. **Make runtime insolvency an explicit ledger comparison.** Reserves plus
   income against obligations. It is a subtraction, it is exact in integer
   arithmetic, and it is the mechanic the design actually wants.
4. **Fix the Neumann series at 32 terms and cap the column sum of the
   coefficient matrix at 0.6875.** At that cap the relative truncation error
   is 1.4 times 10 to the power minus 5, which is 0.89 of one Q16.16 step.
   Section 3.3 gives the table for other caps.
5. **The three-tier split holds at one million individuals.** The owner has
   confirmed that a unit is an individual soldier, so needs are per
   individual. Needs decay over one million individuals costs 3.6
   core-milliseconds at 3 needs, 4.8 at 4 needs and 7.2 at 6 needs, with a
   bandwidth floor of 0.6 to 1.2 milliseconds. At a period of 10 ticks that
   is **0.36 to 0.72 core-milliseconds per tick. It is affordable.** It is
   not, however, the cheapest kernel in the engine, and the report says so.
6. **Individual decision-making is unaffordable by two orders of
   magnitude.** A realistic decision costs 400 nanoseconds. At one million
   individuals that is 400 core-milliseconds for one pass, which exceeds the
   whole tick budget of 90 to 360 core-milliseconds. **The affordable ceiling
   is 100,000 deciders on a period of 10 ticks.**
7. **The decision aggregate already exists. Do not invent a second one.**
   The military aggregate is the **formation**, which another report already
   specifies at 10,000 rows with a `formation` column on the unit and a
   compressed sparse row reverse index.[^3] The civilian aggregate is the
   settlement stratum, at 40,000 rows. Together that is 50,000 deciders, which
   is half the ceiling. **Individuals carry needs. Aggregates make choices.**
8. **A wage is a map over cohorts, not a transaction over individuals.**
   Money is commodity slot 0 in the existing pool array, which makes the
   conservation test cover money for free. It stays out of the transport
   solve.
9. **Loyalty is a per-entity scalar. Legitimacy and unrest are a field.**
   Loyalty belongs to an identity and moves with it. Unrest belongs to a
   place and spreads to a neighbour. The engine already owns a diffusion
   solver for the second.[^5]

**Two further findings, added after the owner confirmed the individual
soldier.**

10. **Quantised per-entity state does not degrade the effective-stat hit
    rate at all.** A veterancy tier enters the pipeline as a **post-stage
    multiplier**, after the table lookup. It never enters the configuration
    key, so the distinct-configuration count `K` does not change. The tier
    count is therefore free with respect to sharing. What limits it is
    visible stepping, and the report gives that arithmetic. **Recommend 16
    veterancy tiers.**
11. **Spend the fourth post-stage multiplier slot on supply, not on
    morale.** Terrain, health tier and veterancy take three of the four
    slots that the schema allows. Morale and stance are already multiplier
    **categories** at stage 3, so putting morale in a post-stage slot spends
    a scarce slot on a value the pipeline can already carry. Supply, which
    is need satisfaction, has no other route into a combat stat. **Without
    that slot the whole consumption chain has no mechanical consequence.**

**The cost.** The whole subsystem adds **30.4 to 31.6 core-milliseconds per
economy tick**, which is **3.0 to 3.2 core-milliseconds and 0.40 to 0.55
wall-milliseconds** at a period of 10 ticks. Section 12 gives the table.
That is about 1 to 3 percent of the mean tick. The individual-needs branch
is the expensive branch, and it fits.

---

## 3. The input-output framing, tested

### 3.1 What Leontief analysis states

Wassily Leontief published the input-output model in 1936.[^7] He gave the
full treatment in 1941 and revised it in the second edition of 1951.[^8] The model divides an economy into `n` activities. Activity
`j` needs `a_ij` units of commodity `i` to make one unit of commodity `j`.
The numbers `a_ij` form the **technical coefficient matrix** `A`.

Let `x` be the vector of gross outputs and `d` the vector of final demand.
Final demand is consumption that no other activity uses as an input. Then:

```
x = A x + d
```

The term `A x` is intermediate demand. The solution is:

```
x = (I - A)^-1 d
```

**This describes the project's economy correctly.** A recipe in the
neighbouring report is exactly one column of `A`: a short list of pairs of a
commodity and a signed rate.[^1] A negative rate is an input. A positive
rate is an output. Dividing every input rate in a recipe by that recipe's
output rate gives the column of `A`. The mapping needs no new data.

The model gives the engine three things that a per-entity kernel cannot.

- **Gross output targets.** Given a final demand, `x` states how much of
  every commodity the faction must produce. That is a planning signal for an
  artificial player and for a build-order heuristic.
- **Shadow prices.** The dual system is `p = A' p + v`, where `v` is the
  labour or land cost of each activity and `A'` is the transpose. Its
  solution gives a cost of production for every commodity that is consistent
  with the whole chain. That is a far better base price than a hand-written
  constant.
- **A structural check.** Section 3.2.

### 3.2 The Hawkins-Simon condition

Hawkins and Simon proved the condition in 1949.[^9] Take a non-negative
matrix `A`. The system `(I - A) x = d` then has a non-negative solution `x`
for every non-negative `d` **if and only if every leading principal minor of
`(I - A)` is positive**.

For a non-negative `A` this condition is equivalent to two other statements
that are easier to test.[^10]

- `(I - A)^-1` exists and every entry of it is non-negative.
- The spectral radius of `A` is less than 1.

The economic meaning is direct. If the condition fails, some subset of
activities consumes more of its own output, through the chain, than it
produces. Such an economy cannot deliver any positive final demand. It is
not merely poor. It is impossible.

**A sufficient condition that an integer engine can check in one pass.**
Assume every column sum of `A` is less than 1. The spectral radius is then
less than 1, so the Hawkins-Simon condition holds. A column sum is the total
input that one unit of output needs. A column sum below 1 therefore means
"an activity consumes less than it makes". That is a rule a content designer
understands.

The check is a reduce over the recipe table. It costs microseconds. Run it
when the engine bakes the type tables.

### 3.3 The Neumann series in fixed point

Do not invert a matrix in integer arithmetic. Gaussian elimination on
integers either overflows or truncates, and the truncation is not bounded in
a useful way.

Use the Neumann series instead.[^11] When the spectral radius of `A` is
below 1:

```
(I - A)^-1 = I + A + A^2 + A^3 + ...
```

so:

```
x = d + A d + A^2 d + ...
```

Evaluate it by repeated multiply and accumulate. Each step is one sparse
matrix-vector product. No division appears anywhere.

```rust
// A is Q16.16 in an i32 array, 64 x 64, column major.
// d and x are i64 quantities.
let mut term = d.clone();      // A^0 d
let mut x    = d.clone();
for _ in 0..NEUMANN_TERMS {    // NEUMANN_TERMS is a compile-time constant
    term = spmv_q16_16(&a, &term);   // term = A * term, i64 accumulate, >> 16
    for i in 0..N { x[i] = x[i].saturating_add(term[i]); }
}
```

**The iteration count.** Let `s` be the largest column sum of `A`. Then the
1-norm of `A` is `s`, so the norm of `A^k` is at most `s^k`. Truncating
after `N` terms leaves a relative error of at most `s^(N+1) / (1 - s)`.

Q16.16 resolves 1 part in 65,536, which is 1.526 times 10 to the power minus
5. The table gives the smallest `N` that reaches that error, and the error
that a fixed `N` of 32 gives.

| Largest column sum `s` | Terms needed | Error at 32 terms | Error at 16 terms |
|---|---|---|---|
| 0.50 | 16 | 2.3e-10 | 1.5e-5 |
| 0.60 | 23 | 1.2e-7 | 4.2e-4 |
| **0.6875** | **32** | **1.4e-5** | 5.5e-3 |
| 0.70 | 34 | 2.6e-5 | 7.8e-3 |
| 0.75 | 43 | 3.0e-4 | 3.0e-2 |
| 0.80 | 56 | 3.2e-3 | 1.1e-1 |
| 0.90 | 127 | 3.1e-1 | 1.7e0 |
| 0.95 | 274 | 3.7e0 | 8.4e0 |

**Recommendation: fix 32 terms and cap the largest column sum at 0.6875.**
The cap is 45,056 in Q16.16, which is `11/16`. Check the cap at bake time
with the same reduce as section 3.2. The truncation error is then 0.89 of
one Q16.16 step, which is below the representable resolution.

The cap is not a restriction that hurts. A column sum of 0.6875 means an
activity turns 0.6875 units of input into 1 unit of output, which is a
profit margin of 45 percent. A strategy game does not want a thinner margin,
because a thinner margin makes every chain fragile.

**The rounding bias.** Each product `a_ij * term_j` shifts right by 16.
A right shift on a negative integer rounds towards negative infinity, so
repeated shifts bias the result downwards. All quantities here are
non-negative, so the bias is a systematic **undercount** of gross output, of
at most one Q16.16 step per term per row. Over 32 terms and 64 rows the
worst case is 32 steps, or 0.0005 of one unit. That is acceptable, because
the output is a plan and not a conserved stock. **Do not use this result as
a stock.** Section 7 of the neighbouring report holds the conservation
rule.[^1]

### 3.4 What the framing gets wrong

The session lead proposed that institutional insolvency needs no
hand-written rule, because an economy that cannot sustain itself has no
non-negative solution, so the failed solve **is** the failure.

**Reject that proposal.** Four objections, in order of weight.

**Objection 1. `A` is content, not state.** The technical coefficient matrix
comes from the recipe table, and the recipe table is immutable baked data.
Two factions that share a recipe table share the same `A` and the same
spectral radius. A test on `A` therefore cannot distinguish a rich faction
from a bankrupt one. It tests the game's data, once, for everybody.

**Objection 2. Insolvency is a flow condition, not a structural one.** A
governing body fails when its income stays below its obligations. It fails
when that shortfall runs long enough to drain the reserve. That is a
statement about `d`, about prices, and about a treasury balance. It is not a statement about `A`. An economy
with a perfectly productive `A` becomes insolvent when a war raises the
wage bill. An economy with a defective `A` was never playable.

**Objection 3. The Neumann series does not fail visibly.** In integer
arithmetic with saturating addition, a divergent series does not raise an
error. It saturates at the `i64` maximum. Detecting that means comparing a
magnitude against a bound, which is the explicit rule the proposal wanted to
avoid. The rule reappears, only less clearly.

**Objection 4. It gives the player no readable signal.** "The linear system
has no non-negative solution" cannot appear in a user interface. "You owe
1,200 gold in wages and hold 300" can.

**What to keep from the proposal.** One piece is sound and valuable. Compute
the gross output `x` that the current final demand needs, then compare it
against the faction's **productive capacity** vector. If any component of
`x` exceeds capacity, the faction cannot meet its own demand from its own
production, however much money it holds. Call that **structural
insolvency**, and make it a distinct state from financial insolvency. It is
a comparison of two vectors of length 64. It costs nothing. It is
readable: "you need 400 iron per tick and you can make 250."

So the model survives and the mechanic changes. **The solve is a planning
tool and a diagnostic. The ledger is the mechanic.**

### 3.5 Substitution: what Leontief cannot express

Leontief assumes **fixed proportions**. Two units of iron and one of coal
make one of steel, always. If iron is dear and copper is cheap, a real agent
substitutes. A Leontief agent cannot.

Three alternatives exist. The report assesses each at 64 commodities.

**Constant elasticity of substitution production.** Arrow, Chenery, Minhas
and Solow introduced this family in 1961.[^12] The output is a power mean of
the inputs, with an exponent that sets how easily one input replaces
another. Leontief is the limiting case where the elasticity is zero.

**Reject it for the tick loop.** The function needs a fractional power. A
fractional power in fixed point needs a logarithm and an exponential, and
each of those is a table lookup plus an interpolation. A fixed-point
logarithm with the accuracy that Q16.16 permits costs about 15 to 30
nanoseconds on the target: one leading-zero count, one table gather that
misses the level-1 cache, and one linear interpolation. A recipe averages
2.2 inputs, and each input needs two of them. Over 40,000 cohorts and 50,000
producers that is 4 to 8 core-milliseconds of transcendental work. The
player cannot see the result. The cost-optimal input mix also needs a division by a price, and
a fixed-point division truncates, which introduces the drift that the
project bans.

**Linear programming.** Model the faction's economy as a linear program:
maximise the value of final demand subject to the input balance and the
capacity limits. This expresses substitution exactly, because the simplex
method chooses among alternative activities.

**Reject it at the tick.** A revised simplex solve over 64 commodities and a
few hundred activities takes a data-dependent number of pivots. A
data-dependent count breaks the fixed frame budget. Exact rational
arithmetic makes it deterministic and makes the numerators grow without a
bound. Integer programming is worse: the neighbouring report already
records that integer multi-commodity flow is NP-hard for two
commodities.[^2]

**Recommended: discrete substitution at a slow cadence.** Do not make the
production function smooth. Give an activity **two to four alternative
recipes**, and choose among them by an argument of the maximum over the
delivered cost at current prices. The choice is a small integer search over
a short list. It runs at the cohort level and at the producer level, on the
market cadence and not every tick. Its cost is one price gather and three
comparisons.

This is the correct point on the spectrum for four reasons.

1. It keeps every arithmetic operation an integer add, multiply or shift.
2. It gives a designer explicit control. A designer writes "a bakery may use
   wheat or rye" instead of tuning an elasticity constant.
3. The switch is a discrete, visible, loggable event. An elasticity is a
   silent gradient.
4. It costs 3 comparisons, not 2 transcendentals.

**State the loss honestly.** Discrete substitution is a step function where
the real behaviour is a curve. An economy with two recipes flips the whole
population between them when the price crosses one point. Damp that with the
existing price band and with a hysteresis margin: switch only when the
alternative is cheaper by at least 1/16 of the current cost. That margin is
a shift, not a division.

---

## 4. The three tiers, costed

The session lead proposed three layers with different affordability. This
section verifies each with arithmetic. The figures assume 40 GB/s of
bandwidth and 3.5 GHz, which the record identifies as the development
machine, and gives core-milliseconds as the reliable column.[^4]

### 4.1 Tier 1: needs decay is a map kernel

Needs decay reads a need scalar, subtracts a rate with saturation, and
writes it back. Each need is an `i32` in Q16.16. The kernel has no branch,
no gather and no dependence between elements. It vectorises to NEON without
help.

| Population | Needs | Traffic | Bandwidth floor | Core-ms |
|---|---|---|---|---|
| 1,000,000 individuals | 3 | 24.0 MB | 0.60 ms | 3.6 |
| 1,000,000 individuals | 4 | 32.0 MB | 0.80 ms | 4.8 |
| 1,000,000 individuals | 6 | 48.0 MB | 1.20 ms | 7.2 |
| 40,000 cohorts | 4 | 1.3 MB | 0.03 ms | 0.19 |

**The lead's claim needs a correction.** Needs decay at one million
individuals is affordable, at 4.8 core-milliseconds for 4 needs. It is not
the cheapest kernel in the engine. It costs about the same as the threshold
predicate pass, which the neighbouring report measures at 1 to 2
core-milliseconds, and rather more than the influence maps at 0.53
core-milliseconds.[^1][^13] The 0.80 millisecond bandwidth floor is one
quarter of the cost of a full tile pass, and the record allows only two or
three full passes per tick.[^4]

At 40,000 cohorts the same work costs 0.19 core-milliseconds. That is a
factor of 25. **The saving is real, but it is not the reason to use
cohorts.** Tier 3 is the reason.

### 4.2 Tier 2: consumption is a segmented reduce and one transfer

Consumption has the same shape as upkeep. Each consumer contributes a
negative rate to a pool. The neighbouring report already specifies the
kernel: sort consumers by pool identifier, run a segmented reduce, fix the
span boundaries in ascending task order, then resolve caps through
`transfer` in priority bands.[^1]

**Reuse that kernel exactly. Do not write a second one.** A cohort is a
consumer with a recipe. Its recipe slots carry a priority byte, exactly like
a unit's upkeep slots. The only new thing is that a cohort's rate scales
with its headcount.

| Item | Scale | Core-ms |
|---|---|---|
| Cohort rate evaluation | 40,000 cohorts x 4 slots = 160,000 | 0.08-0.16 |
| Segmented reduce into pools | 160,000 deltas | 0.15-0.30 |
| Resolve through `transfer` | shares the existing pool-commodity pass | 0 additional |

**The lead's claim holds.** Pooled consumption is cheap and it needs no new
machinery. Total: **0.23 to 0.46 core-milliseconds.**

### 4.3 Tier 3: decisions do not fit at one million

A decision is a search. The cheapest useful form is "gather the price of `k`
options, weight each by an unmet need, and take the argument of the
maximum". At `k` of 4 to 8 that is 4 to 8 random gathers, which are cache
misses, plus a short comparison chain. A cache miss on the target costs
about 80 to 100 nanoseconds. A realistic figure is **100 nanoseconds for a
trivial decision and 400 nanoseconds for a useful one**. The 400 nanosecond
figure matches the record's own per-unit ceiling.[^4]

| Deciders | At 100 ns | At 400 ns | At 400 ns, period 10 |
|---|---|---|---|
| 1,000,000 | 100 core-ms | 400 core-ms | 40 core-ms per tick |
| 200,000 | 20 core-ms | 80 core-ms | 8 core-ms per tick |
| **100,000** | 10 core-ms | 40 core-ms | **4 core-ms per tick** |
| **40,000** | 4 core-ms | 16 core-ms | **1.6 core-ms per tick** |
| 20,000 | 2 core-ms | 8 core-ms | 0.8 core-ms per tick |

The record's whole tick budget is 90 to 360 core-milliseconds.[^4] A
decision pass over one million individuals costs 400 core-milliseconds by
itself. **The lead's claim holds and it is not close.** It is over budget by
a factor of about 4, before any other system runs.

**The affordability ceiling.** Allow the economy 4 core-milliseconds per
tick for decisions, which is about 2 percent of the mean tick. At a period
of 10 ticks and 400 nanoseconds each, that permits **100,000 deciders**.
Take half of that as the working figure to leave room for a richer decision.

---

## 5. The cohort model

### 5.1 What a cohort is

A cohort is one row that stands for many individuals of the same kind in the
same place. The row holds a headcount and one shared state vector. Every
individual in the cohort is identical, so the engine simulates the vector
once and scales the result by the headcount.

The design comes from the population-unit model that Paradox Interactive
uses in the Victoria series. A population unit there holds a size, a
profession, a culture, a religion, a wealth level and a set of need tiers.
The market prices goods, and each population unit buys against its needs in
tier order.[^14][^15]

### 5.2 Two need-bearing populations, not one

**This is the central structural recommendation.**

The project already has one million units. A unit is a military or working
formation with a position. The neighbouring report already gives every unit
an upkeep recipe and runs it through the production kernel.[^1] That
subsystem already works and it is already costed.

Population is a different thing. Population lives in a settlement. It does
not move each tick. It is far more numerous than the unit count and far less
individually interesting.

| Population | Rows | Needs handled by | Makes decisions |
|---|---|---|---|
| Units | 1,000,000 | The existing upkeep recipe kernel | No |
| Cohorts | 40,000 | This report's needs kernel | Yes |
| Characters | a few thousand | Per-entity, see the character report[^3] | Yes |

**A unit does not decide.** Its upkeep is a fixed recipe. Its owner decides
for it. This removes the whole tier-3 problem from the one million row
population at a stroke, and it is also the correct simulation: a soldier
does not choose their rations.

### 5.3 The recommended cohort count and shape

**Recommendation: 8 strata in each of 5,000 settlements, so 40,000 cohorts.**

```rust
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct CohortRow {
    pool:      u32,   // PoolId. The settlement. Also the sort key.
    headcount: u32,   // Individuals represented. Whole people.
    stratum:   u8,    // Selects need weights, wage rate and recipe.
    culture:   u8,
    _pad:      [u8; 2],
    wealth:    i64,   // Money held, in the money commodity's unit
    need:      [i32; 4],   // Fix32, Q16.16
    quality:   [i32; 4],   // Fix32, Q16.16. See section 7.
    loyalty:   i32,   // Fix32, Q16.16
    unrest:    i32,   // Fix32, Q16.16
}
```

The row is 56 bytes. At 40,000 rows the whole population state is **2.24
MB**. Store it as struct-of-arrays, not as this struct, so that the needs
kernel reads only the need columns. The struct above states the fields, not
the layout.

**Why 8 strata.** Eight is the smallest count that expresses the
distinctions the mechanics need: who pays tax, who receives a wage, who
revolts, and who fills an office. A plausible set is farmer, labourer,
artisan, merchant, soldier, clerk, noble and clergy. Eight also packs into a
`u8` with room, and 8 cohorts per settlement means a settlement's whole
population fits in one cache-line group beside its pool.

**Why 5,000 settlements.** That is the pool count the neighbouring report
already assumes.[^1] The cohort array indexes as `pool * 8 + stratum`, which
makes the pool sort free: **the array is already sorted by pool by
construction**. The segmented reduce of section 4.2 therefore needs no sort
at all. This is the strongest argument for the fixed 8, and it is the reason
to prefer a fixed stratum count over a variable one.

**Mean headcount.** It equals the total population divided by 40,000. The
total population is an open question, because it is a design choice and not
an engine constraint. At 40 million people a cohort holds 1,000. At 10
million it holds 250. The engine cost does not change either way, because
the headcount is one `u32` field.

### 5.4 What a cohort loses against individual agents

State this honestly. A cohort is a lossy model and the losses are specific.

| Lost property | Consequence | Mitigation |
|---|---|---|
| Variance inside the cohort | Everyone starves at once, or nobody does | Split the shortfall: reduce the headcount rather than driving the need to zero. See section 6.4. |
| Individual identity | No named person in the population | Characters are separate entities and keep identity.[^3] |
| Individual movement | Migration is a headcount transfer, not a path | Use `transfer` between two cohorts of the same stratum. Exactly conserving. |
| Fine-grained substitution | A whole cohort switches recipe at once | The hysteresis margin of section 3.5 damps the flip. |
| Emergent inequality inside a stratum | Wealth is one number for 1,000 people | Add strata, not variance. Eight strata already give the inequality that a strategy game reads. |

The most important loss is the first. A pure cohort model has a cliff: at
99 percent of the food the cohort is fine, at 101 percent of the demand it
starves entirely. Section 6.4 removes that cliff without adding rows.

---

## 6. Needs and decay

### 6.1 Which needs

**Recommendation: four needs.** The owner named food, water and sleep. Add
shelter, because it is the need that a settlement's buildings satisfy and it
therefore gives construction a demand-side meaning.

| Index | Need | Satisfied by | Notes |
|---|---|---|---|
| 0 | Sustenance | Food commodities | The primary consumption driver |
| 1 | Water | Water, or a tile property | May be free on a river tile |
| 2 | Rest | Time, not a commodity | Decays with work, recovers when idle |
| 3 | Shelter | A building capacity in the settlement | A stock, not a flow |

Four is the right count for three reasons. It fits one 16-byte NEON vector
of `i32`, so the decay kernel is one load, one saturating subtract and one
store. It gives each need a distinct mechanical consequence, which is the
test of whether a need earns a slot. And the fourth need attaches
construction to consumption, which closes the loop that the design wants.

Reject 6 or more. Two extra needs cost 50 percent more traffic in the
hottest kernel of this subsystem, and the design has no distinct consequence
for them.

### 6.2 The scale and the decay rate

A need is an `i32` in Q16.16. **65,536 means fully satisfied. 0 means fully
unmet.** The range is 0 to 65,536, so the value never leaves the low 17
bits, and every intermediate product fits in an `i64` with wide margins.

Let `TICKS_PER_DAY` be the number of ticks in one game day. At 10 Hz and a
one-minute day, that is 600. The decay rate for a need that empties in `D`
days is:

```
rate = 65536 / (D * TICKS_PER_DAY)     // computed at bake time, not per tick
```

| Need | Days to empty | Rate at 600 ticks per day |
|---|---|---|
| Sustenance | 3 | 36 per tick |
| Water | 1 | 109 per tick |
| Rest | 1 | 109 per tick |
| Shelter | no decay | it tracks a capacity ratio |

**Compute every rate at bake time.** Store it as an `i32`. The tick kernel
then contains no division. A division inside a hot kernel is both slow and
a rounding hazard.

If the needs kernel runs on a period of 10 ticks, multiply each rate by 10
at bake time. The rate stays exact, because it is a stored constant and not
a repeated accumulation.

### 6.3 Saturating arithmetic

```rust
// The decay kernel. One map over the need columns.
for i in span {
    need[i] = (need[i] - rate).max(0);          // saturating at the floor
}

// The satisfaction kernel, after `transfer` has paid the cohort.
for i in span {
    let gain = ((paid[i] as i64 * quality[i] as i64) >> 16) as i32;
    need[i] = (need[i] as i64 + gain as i64).min(65536) as i32;
}
```

Use `max(0)` and `min(65536)`, not a wrapping subtract. A wrapping subtract
turns a small shortfall into a full satisfaction, which is the classic
underflow bug in this exact kernel. Write the clamp explicitly.

**The clamp on a need is safe.** The record and the field algebra report both
warn that a clamp breaks conservation and must never touch a conserved
field.[^5] A need is **not** a conserved field. It is an intensive state
variable of a cohort. Nothing flows into it and nothing flows out. The
commodity that satisfies it **is** conserved, and that commodity moves only
through `transfer`, which conserves exactly. The clamp applies to the need,
after the conserving transfer, and never to the commodity.

### 6.4 Unmet need: degradation, then a threshold crossing

**Reuse the dense-bitset plus sparse-scan pattern from the neighbouring
report. Do not invent another.**[^1] The pattern is: a dense branchless map
writes a predicate bit into a bitset plane, a barrier follows, then a sparse
ascending scan runs the handler for each set bit.

Two stages, and this is what removes the cohort cliff.

**Stage 1: degradation, every economy tick, dense.** While a need is below a
threshold, a **deficit accumulator** rises. The accumulator is an `i32` in
Q16.16 and it falls again when the need recovers. It drives continuous
effects: output falls, loyalty falls, unrest rises. There is no event and no
branch.

```rust
let deficit = (THRESHOLD - need[i]).max(0);         // 0 when satisfied
accum[i] = (accum[i] + deficit).min(ACCUM_MAX);     // saturating
out_scale[i] = 65536 - ((accum[i] * PENALTY) >> 16); // multiplies production
```

**Stage 2: the crossing, when the accumulator passes its bound.** Now the
cohort loses headcount, or a unit deserts, or a settlement revolts. The
predicate goes into a bitset plane. One plane per class.

| Class | Predicate | Handler | Plane |
|---|---|---|---|
| Starvation | sustenance deficit accumulator at maximum | Reduce the headcount by a fraction. Emit a `Starved` event. | 0 |
| Thirst | water accumulator at maximum | Same shape, faster rate | 1 |
| Desertion | a unit's supply accumulator at maximum | Reduce the unit's strength. Emit `Deserted`. | 2 |
| Revolt | cohort unrest above its bound | Emit `Revolted`. A handler outside this subsystem responds. | 3 |
| Vacancy | a character's loyalty below its bound | Emit `OfficeVacated`. Succession responds.[^3] | 4 |

At 40,000 cohorts a plane is 5 KB. Five planes cost 25 KB. That is nothing.
The unit planes for desertion are sized to the unit capacity and cost 128
KiB each, exactly as the neighbouring report states.[^1]

**The ordering rule is the same rule, unchanged: the handler runs in
ascending row index order, and nothing else.**[^1] The plane's contents are
identical at any thread count, because a bitwise OR into disjoint words is
exactly commutative and each span owns whole 64-bit words.

**Why the accumulator removes the cliff.** A cohort at 60 percent food does
not starve. Its deficit accumulator rises slowly, its output falls
proportionally, and its loyalty drops. Only sustained deprivation crosses
the threshold. The population then falls by a fraction, which is a partial
outcome from a whole-cohort model. This gives the graded behaviour that
individual agents would give, at no extra rows.

---

## 7. Quality as an intensive quantity

### 7.1 The pair form is correct

The project rule states that an intensive quantity aggregates as a pair: the
sum of value times weight, and the sum of weight.[^6] The field algebra
report restates it as rule 2 of its type system: an intensive field restricts
by a weighted mean, stored as a sum and a count, so that the combine stays a
group.[^5]

**Confirmed. Quality is intensive and the pair form is exactly right.**

Quality is a property of a quantity of a commodity, not of a place. Mixing
100 units of quality 0.5 grain with 300 units of quality 1.5 grain gives 400
units of quality 1.25. That is a weighted mean, and the weight is the
quantity.

```rust
// Per pool, per commodity. Both are i64. `stock` IS the weight.
struct QualityPair {
    weighted: i64,   // sum over deliveries of amount * quality, Q16.16 in the low bits
    weight:   i64,   // the stock. Already stored. Do not duplicate it.
}
// The quality at read:
let quality = if pair.weight > 0 { pair.weighted / pair.weight } else { DEFAULT };
```

Three properties make this the correct choice.

1. **It is a group.** Adding a delivery adds to both terms. Removing a
   consumption subtracts from both. There is no rescan and no recompute
   path, so it satisfies the record's aggregation rule as case (a).[^4]
2. **The weight is already stored.** The pool stock is the weight. Quality
   therefore costs one extra `i64` per pool-commodity cell, not two. At
   5,000 pools and 64 commodities that is **2.56 MB**, which doubles the
   stock array and no more.
3. **The division happens once at read, not in the aggregate.** The
   truncation of that division does not accumulate, because the stored pair
   is exact. This is the whole reason the rule exists.

**One correction to naive use.** Do the multiply before the divide. Compute
`(amount * weighted) / weight`, not `amount * (weighted / weight)`. The
second form truncates the quality first and then scales the error by the
amount. The first form truncates once. All three values are `i64`, and the
product needs at most 62 bits for a stock below 2 to the power 30 and a
quality below 2 to the power 18.

### 7.2 How quality changes satisfaction

**Recommendation: quality scales the amount for a physical need, and the
surplus above 1.0 feeds a separate contentment channel.**

This answers the question directly: better food satisfies **more**, up to the
point where the need is full, and beyond that point it satisfies
**differently**.

```
gain      = (amount * quality) >> 16          // quality 65536 is neutral
need     += min(gain, 65536 - need)           // the physical need saturates
surplus   = gain - min(gain, 65536 - need)    // what the body cannot use
contentment += (surplus * LUXURY_GAIN) >> 16  // feeds loyalty, section 11
```

Four reasons for this split.

1. **It matches physical reality.** A hungry person fed excellent bread is
   no less hungry than one fed twice as much plain bread. Both are fed.
2. **It gives luxury a mechanic.** Without the split, high-quality goods have
   no use once a population is fed, so a luxury economy has no demand curve.
   With it, a satisfied population still wants better goods, and that demand
   is what drives trade after subsistence.
3. **It keeps one arithmetic path.** Both channels are the same multiply and
   shift. There is no second formula to keep consistent.
4. **It is monotone.** More quality never reduces satisfaction, so a designer
   cannot build a trap.

**The quality range.** Store quality as a `Fix32` in the band 16,384 to
262,144, which is 0.25 to 4.0. Enforce the band at bake time and at every
production site. A quality of 0 would let a delivery satisfy nothing while
still consuming stock, which is a silent value leak. A quality above 4 makes
one luxury delivery satisfy a whole settlement.

---

## 8. Money, wages and taxation

### 8.1 Money is commodity slot 0

**Recommendation: money is a commodity in the existing pool array. Reserve
slot 0 for it. Exclude it from the transport solve.**

The alternative is a separate `i64` treasury column per pool and per
institution. Compare the two.

| Property | Money as commodity 0 | Money as a separate column |
|---|---|---|
| Conservation test | The existing pool conservation test covers money for free | Needs a second test that someone will forget |
| `transfer` verb | Works unchanged. A payment is a transfer. | Needs a second implementation |
| Storage | 5,000 pools x 8 bytes = 40 KB, already allocated | 40 KB, newly allocated |
| Presence mask | Bit 0 of the existing `u64` mask | A separate flag |
| Transport | Must be excluded, which is one line | Excluded by construction |
| Price | Money has no price. It is the numeraire. | Same |

**The single reason that decides it: conservation.** The neighbouring report
requires an exact conservation test over every pool, every in-flight
quantity and every spill record.[^1] Money must satisfy the same test.
Putting money in the same array means the same test covers it, with the same
code, at the same time. A separate treasury is a second thing to check and
therefore a second thing to get wrong.

**Two special rules for slot 0.**

1. **Money does not enter the transport solve.** The transport solve carries
   16 commodities, and money is not one of them.[^2] Money moves by an
   explicit `transfer` between two named pools, which represents a payment
   or a shipment of specie. This is correct, not a limitation: a payment is
   an instruction, not a flow across a field.
2. **Money has no price and no quality.** Bit 0 of the quality array is
   unused. Money is the numeraire against which every other price is
   quoted.

### 8.2 A wage is a map, not a transaction

**A wage never touches an individual.** The flow is:

```
Institution treasury  ->  cohort wealth      (a wage, by stratum)
Institution treasury  ->  character wealth   (an office salary, per entity)
```

The cohort wage is one map kernel over 40,000 rows.

```rust
// Phase 1: compute the obligation. A map. No writes to the treasury.
// Runs in ascending cohort index order. The order is the array order.
for c in cohorts {
    let rate = wage_rate[institution_of[c]][stratum[c]];   // Fix32, per head
    obligation[c] = (headcount[c] as i64 * rate as i64) >> 16;
}

// Phase 2: reduce obligations per institution. A segmented reduce.
// The cohort array is sorted by pool, and a pool has one institution,
// so the array is already segmented. No sort.

// Phase 3: pay. One `transfer` call per institution, over its cohorts,
// in ascending priority band and then ascending cohort index.
```

**Phase 3 is the important one.** It is the same `transfer` verb, with the
same largest-remainder split, that the neighbouring report uses for
upkeep.[^1] If the treasury cannot pay every wage, the bands decide who is
paid first, and the split inside the failing band is exact. The sum of the
payments equals the treasury withdrawal exactly, in integers. **No money is
created and none is lost.**

The cost at 40,000 cohorts is one multiply and one shift per row, then a
segmented reduce over 40,000 elements. That is **under 0.05
core-milliseconds**. There are no individual transactions at any point.

**A character salary is different and that is correct.** A character is one
entity with an identity, so a per-entity payment is affordable at a few
thousand rows. A character's salary is one row in the same obligation list,
at a higher priority band than a cohort wage. The office that pays it is an
entity with a holder column and an anchor column, and another report owns
that structure.[^3] **That report's character row holds no wealth column
today.** A salary needs one. Open question OQ68 records it.

### 8.3 Taxation is the same kernel with the sign reversed

Tax flows the other way, from a cohort to an institution. Use the same three
phases.

Two tax bases are cheap and one is not.

| Base | Formula | Cost | Verdict |
|---|---|---|---|
| **Poll tax** | `headcount * rate` | one multiply | Accept |
| **Wealth tax** | `wealth * rate >> 16` | one multiply | Accept |
| **Consumption tax** | a fraction of the value bought | free; the exchange pass already computes the value | **Prefer.** It couples tax to activity. |
| Income tax on a flow | needs an income accumulator per cohort | one extra `i64` column and one extra pass | Accept only if the design needs it |

**Recommended: a consumption tax as the primary base.** The exchange pass of
section 9 already computes the money value of every purchase per cohort. The
tax is a fraction of that number, taken during the same pass. It costs one
multiply and one shift, and it adds a term to a reduce that already runs.

The tax must be a `transfer`, not a subtract-then-add. A subtract from the
cohort followed by an add to the treasury is two operations that can
disagree if one saturates. A `transfer` moves the quantity in one operation
and conserves by construction.

**Rounding bias in tax.** `(wealth * rate) >> 16` rounds down, so every tax
collection favours the taxpayer by up to one unit. Over 40,000 cohorts and
a period of 100 ticks that is at most 40,000 units of money per 100 ticks
that the state does not collect. That is a systematic bias, not a random
one, and it is worth stating. It does **not** leak value, because the
untaken remainder stays with the cohort. It only shifts the balance
slightly. If the design needs it exact, apply the largest-remainder method
across the cohorts of one institution: take `floor` from each, then take one
extra unit from the `r` cohorts with the largest remainder, in ascending
cohort index order on a tie. That makes the collection exactly the intended
total.

---

## 9. Exchange and price formation

### 9.1 Do not build a market. Build an allocation.

An order book, a double auction, or a per-agent bid loop are all wrong here.
Each needs a per-agent iteration, and section 4.3 shows that per-agent
iteration is the thing the engine cannot afford.

**Recommendation: exchange is one allocation per settlement per commodity,
executed by the existing `transfer` verb, at the existing settlement
price.** There is no order book and there is no clearing loop.

The price already exists. The neighbouring report specifies one clamped
tatonnement step per tick, per settlement cell, per commodity, clamped to a
band of one quarter to seven quarters of a base price.[^2] Build on it
without changing it. This report adds only two things: the demand that feeds
the excess term, and the base price that the band centres on.

**The base price comes from the dual Leontief system.** Section 3.1 gives it
as `p = A' p + v`, solved by the same 32-term Neumann series on the
transpose. That gives every commodity a cost of production consistent with
the whole chain, at the cost of one more solve. It is a far better centre
for the band than a hand-typed constant, and it moves when technology
changes.

### 9.2 The exchange pass

The pass runs per settlement, over the 8 cohorts of that settlement and the
commodities they need. It is a fixed, bounded amount of work per settlement.

```
Step 1  For each cohort, for each needed commodity:
          desired  = need_gap * consumption_per_head * headcount   // i64
          budget   = wealth * SPEND_FRACTION >> 16
          affordable = budget / price                              // i64 divide
          demand[cohort][commodity] = min(desired, affordable)

Step 2  Sum demand over the 8 cohorts, per commodity.  (a reduce of 8)

Step 3  transfer(stock, demands, priority_bands)
          -> goods move from the pool to the cohorts, exactly

Step 4  For each cohort, for each commodity:
          cost      = received * price >> 16
          wealth   -= cost                       // a transfer, not a subtract
          treasury += cost                       // the same transfer
          tax       = cost * tax_rate >> 16      // section 8.3

Step 5  Accumulate excess = total_demand - total_supply for the price step.
```

**Step 1 has the only division in the pass.** `budget / price` truncates
downwards, so a cohort buys at most what it can afford and never more. The
bias is towards under-buying by less than one unit per cohort per commodity.
That direction is safe: it can never drive wealth negative.

**Step 3 is the whole market.** The `transfer` verb sums the demand,
computes one scale, writes the floor, and distributes the remainder by
largest remainder in ascending cohort index order. The goods that leave the
pool equal the goods that reach the cohorts, exactly. Priority bands express
"the soldiers eat first" without a second mechanism.

**Step 4 must be one transfer.** Money leaving a cohort and money entering
the treasury are the same operation. Write it as a transfer over the money
commodity so that the conservation test covers it.

### 9.3 The cost

| Step | Scale | Core-ms |
|---|---|---|
| Demand and budget, with one divide | 40,000 cohorts x 4 commodities = 160,000 | 0.30-0.50 |
| Reduce over 8 cohorts per settlement | 5,000 x 4 | 0.02 |
| `transfer` allocation | 5,000 settlements x 4 commodities | 0.10-0.30 |
| Payment and tax | 160,000 cells | 0.08-0.16 |
| Price step, existing kernel | already counted in the trade budget[^2] | 0 |
| **Total** | | **0.50-0.98** |

**Under 1 core-millisecond for the whole exchange.** The divide in step 1
dominates. A 64-bit integer divide costs about 20 to 40 cycles on the
target, and 160,000 of them is about 1 to 2 milliseconds of latency if they
serialise. They do not serialise, because the loop is independent across
rows and the divider pipelines. If measurement shows otherwise, replace the
divide with a reciprocal that the price step computes once per settlement
per commodity, at 5,000 times 4 divides instead of 160,000. That is a 40-fold
reduction and it costs one `i64` column.

---

## 10. Institutional solvency

### 10.1 The institution row

```rust
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct InstitutionRow {
    faction:      u16,
    parent:       u16,      // The institution above. 0xFFFF for none.
    seat_pool:    u32,      // Where the treasury sits
    treasury:     i64,      // Money. Also pool[seat_pool][0]. See below.
    reserve_min:  i64,      // Below this, the institution stops discretionary spend
    arrears:      i64,      // Money owed and unpaid. Cumulative.
    income_ema:   i64,      // A smoothed income, for a readable projection
    legitimacy:   i32,      // Fix32
    _pad:         [u8; 4],
}
```

**The treasury is not a second store of money.** It is a named view of the
money commodity in the institution's seat pool. Keeping it as a separate
number would create the second conservation problem that section 8.1
rejects. Store the institution's money **only** in the pool array, and let
`treasury` be an accessor.

At a few hundred institutions the whole array is under 32 KB.

### 10.2 Solvency is a comparison, not a solve

```
obligations = sum of wage obligations
            + sum of unit upkeep in money
            + sum of construction commitments due this period
            + sum of debt service

available   = treasury + projected income this period

solvent            when available >= obligations
strained           when available >= obligations but treasury < reserve_min
financially insolvent  when available <  obligations
structurally insolvent when any component of the Leontief gross output x
                       exceeds the faction's productive capacity  (section 3.4)
```

Every term is an `i64`. The comparison is exact. There is no tolerance,
because there is no rounding.

**Financial insolvency and structural insolvency are different states and
the design should show both.** A faction can hold gold and still be unable
to make enough iron. A faction can make plenty of iron and still be unable
to pay for it. The first is a production problem and the second is a
treasury problem, and a player needs to know which one they have.

### 10.3 The failure cascade, and its ordering rule

**The ordering rule, stated once:**

> The cascade runs in ascending institution index order. Within one
> institution it runs in ascending priority band, then ascending obligation
> index. No stage reads a value that a later stage writes in the same
> economy tick.

Six stages. Each stage is a separate pass with a barrier between it and the
next, so that no stage depends on how the previous stage was scheduled.

| Stage | Work | Kernel | Order |
|---|---|---|---|
| 1 | Sum obligations per institution | segmented reduce | ascending obligation index |
| 2 | Pay in ascending priority band. `transfer` splits the failing band. | map plus transfer | band, then obligation index |
| 3 | Add the unpaid amount to `arrears`. Emit one `Unpaid` event per unpaid obligation. | map | ascending obligation index |
| 4 | Reduce loyalty for every unpaid party by a function of the arrears. | map | ascending row index |
| 5 | Phase A: write the crossing predicates into the bitset planes. | dense map | span order, disjoint words |
| 6 | Phase B: scan the planes and run the handlers. | sparse ascending scan | ascending row index |

Stage 2 uses the existing `transfer` verb with no change. The bands are the
mechanic: pay the army before the clerks, or pay the clerks and watch the
army desert. This is a designer's table, not code.

Stage 6 raises three event classes, and this subsystem owns none of the
handlers.

- `OfficeVacated`. A character's loyalty fell below its bound. **The
  succession machinery replaces the holder.** This report does not duplicate
  it.[^3]
- `Revolted`. A cohort's unrest passed its bound. A handler outside this
  subsystem responds.
- `Defaulted`. The institution's arrears passed a bound relative to its
  income. The design decides what that means.

**A note on the frequency of replacement.** The owner states that losing a
position is rare. The model gives that for free: loyalty is a slow
accumulator over many economy ticks, and the vacancy threshold sits far from
the resting value. Rarity is a constant, not a special case.

---

## 11. Loyalty and legitimacy

### 11.1 Loyalty is a per-entity scalar

**Recommendation: loyalty is one `Fix32` column on the cohort row and on the
character row. It is not a field.**

The project owns a diffusion solver and a field operator algebra, so the
question is real.[^5] The answer is that loyalty fails the membership test
for a field.

| Test | Loyalty | A field |
|---|---|---|
| Does the quantity belong to a place? | No. It belongs to a person or a group. | Yes |
| Does it move to a neighbour by transport? | No. It is not transferred. | Yes |
| Would a spatial average be meaningful? | No. Averaging a noble's loyalty with a farmer's is a category error. | Yes |
| Does it move with its owner? | Yes. A cohort that migrates keeps its loyalty. | No |

The fourth row settles it. A field cannot follow an entity. Diffusing
loyalty over space would let the loyalty of an unrelated neighbouring
stratum leak into a cohort's own value, which is exactly wrong.

The update is one map over 45,000 rows, which is negligible.

```rust
// One map. Runs in ascending row index order. Every term is Fix32.
let mut d: i64 = 0;
d += (wage_paid_ratio - 65536) * W_WAGE      >> 16;   // negative when underpaid
d += (need_satisfaction - THRESHOLD) * W_NEED >> 16;
d += (contentment) * W_LUXURY                 >> 16;   // section 7.2
d -= (unrest_here) * W_UNREST                 >> 16;   // the field, sampled
d += (legitimacy_of_institution - 65536) * W_LEGIT >> 16;
d -= (arrears_ratio) * W_ARREARS              >> 16;
loyalty = (loyalty + (d * DAMPING >> 16)).clamp(0, 65536);
```

**The damping term matters.** Without it, loyalty oscillates when the inputs
oscillate, and the vacancy threshold then fires on noise. Set `DAMPING` to
1/16 or less, so that loyalty needs at least 16 economy ticks to cross a
quarter of its range.

### 11.2 Unrest is a field

Unrest **does** pass the membership test. It belongs to a place, it spreads
to a neighbour, a spatial average is meaningful, and it does not follow an
entity. It is a potential, not a conserved quantity, so the field algebra's
rule 1 applies: diffuse it as a potential and state that the result is a
potential and not a balance.[^5]

Run it on the L1 cell grid, not on tiles. There are 65,536 L1 cells, and the
engine already runs a relaxation over that grid with a fixed 8 iterations.
Its whole influence schedule costs 0.53 core-milliseconds, of which a
separable economic plane costs 12 microseconds and a seeded Jacobi military
plane costs 150 microseconds.[^13] **Unrest is one more plane in that
existing solver.** Its marginal cost is between those two figures, so budget
0.07 core-milliseconds. It needs a decay term, because a potential without
decay relaxes to a constant.[^5]

The two couple in both directions and both directions are cheap.

- Low cohort loyalty is a **source** into the unrest plane. A scatter of
  40,000 values into 65,536 cells, in ascending cohort index order.
- High unrest is an **input** to the loyalty update, sampled by a gather at
  the cohort's own cell. One gather per cohort.

**This is the correct split and it gives the design what it wants.** Loyalty
answers "will this person stay". Unrest answers "is this province about to
rise". A revolt needs both: a region of high unrest and a set of disloyal
cohorts inside it.

### 11.3 Legitimacy

Legitimacy is a per-institution scalar and it is cheap, because there are a
few hundred institutions. It rises when the institution pays its
obligations, when needs are met across its territory, and when it wins. It
falls with arrears. It enters the loyalty update as one term. It needs no
field and no special machinery.

---

## 12. Cadence and cost

### 12.1 The cadence table

Not every part of this subsystem runs at the same rate. The record requires
a period and a phase offset for every system.[^4]

| Work | Period | Phase | Why |
|---|---|---|---|
| Needs decay | 10 | 0 | Rates are scaled by 10 at bake time. Nothing observes a need between economy ticks. |
| Cohort consumption reduce | 10 | 0 | Shares the existing production reduce.[^1] |
| Exchange and payment | 10 | 1 | Must run after transport delivers.[^2] |
| Price tatonnement step | 1 | — | Already specified. Do not change it.[^2] |
| Cohort decisions | 10 | staggered by faction | The expensive tier. Section 12.2. |
| Degradation accumulators | 10 | 1 | Reads the exchange result |
| Threshold phase A and B | 10 | 2 | The last stage of the economy tick |
| Wages and taxation | 100 | 0 | One game month. A wage is not a per-second event. |
| Solvency ledger and cascade | 100 | 1 | Must run after wages |
| Loyalty update | 100 | 2 | Must run after the cascade |
| Unrest diffusion plane | 10 | shares the influence solver[^13] | A field needs frequent relaxation |
| Leontief plan and dual prices | 40 | staggered by faction | Structural. It changes only when technology or capacity changes. |
| Hawkins-Simon check | bake time | — | It tests content, not state. |

**The interleave rule.** Within one economy tick the order is fixed and it
extends the neighbouring report's three-part ordering contract.[^1]

```
5c   production and upkeep rates, segmented reduce      (report 12)
5c'  cohort needs decay and consumption rates           (this report)
--   the transport solve                                (report 11)
5d   resolve caps through transfer                      (report 12)
5d'  exchange, payment, taxation                        (this report)
5d'' cohort decisions, on the stagger                   (this report)
5e   degradation accumulators                           (this report)
5e'  threshold phase A, then a barrier, then phase B    (report 12 pattern)
```

Every step has a stated total order and no step reads a value that another
step writes in the same parallel region.

### 12.2 Stagger the decisions

Cohort decisions cost 16 core-milliseconds in one pass. Do not take that as
a spike on every tenth tick. **Run one eighth of the factions on each tick
with a phase offset**, exactly as the neighbouring report staggers the
economy.[^1] The cost then flattens to 1.6 to 2.0 core-milliseconds on every
tick with no change to the total and no new machinery.

### 12.3 The cost table

The figures assume 40,000 cohorts, 5,000 settlements, 4 needs, 64
commodities, a few hundred institutions and 8 factions. They give
core-milliseconds for one economy tick, then the amortised figure at a
period of 10.

| Work | Scale | Core-ms per economy tick |
|---|---|---|
| Needs decay | 40,000 x 4 | 0.19 |
| Cohort consumption rates and reduce | 160,000 deltas | 0.23-0.46 |
| Exchange, payment and taxation | 160,000 cells | 0.50-0.98 |
| **Cohort decisions** | 40,000 at 400 ns | **16.00** |
| Degradation accumulators | 40,000 x 4 | 0.10 |
| Threshold phase A | 40,000 cohorts plus 1M unit planes | shares report 12's pass |
| Threshold phase B, sparse | at 1 percent crossing | 0.10 (serial) |
| Wages, taxation, solvency, at period 100 | 40,000 rows plus 300 institutions | 0.05 amortised |
| Loyalty update, at period 100 | 45,000 rows | 0.01 amortised |
| Unrest plane, in the influence solver | 65,536 cells x 8 sweeps | 0.07 |
| Leontief plan and dual, at period 40 | 8 factions x 64 x 64 x 32 x 2 | 0.15 amortised |
| Hawkins-Simon check | bake time only | 0 |
| **Total per economy tick** | | **17.40-18.06** |
| **Amortised at period 10** | | **1.74-1.81** |
| **Wall-ms at 12 cores** | | **0.25-0.35** |

**The decision pass is 92 percent of the cost.** Every other part of this
subsystem together costs under 1.5 core-milliseconds. That is the single
number to attack if the budget breaks, and section 4.3 gives the lever: the
cost is linear in the cohort count, so halving the strata halves it.

The Leontief solve is the cheapest thing in the table at 0.15
core-milliseconds amortised. The lead's concern about matrix cost was
misplaced, because the matrix is 64 by 64 and not 40,000 by 40,000. A dense
64 by 64 matrix of `i32` is 16 KB, which sits in the level-1 cache for the
whole solve.

### 12.4 The row to add to the running budget

| Work | Scale | Core-ms | Wall-ms (12c) |
|---|---|---|---|
| Needs, consumption, exchange, wages, solvency | 40,000 cohorts, period 10, staggered | 1.7-1.9 | 0.25-0.35 |

The running budget table currently holds movement at 1.9 to 3.8
wall-milliseconds, trade at 1.1 wall-milliseconds, economy at 0.4 to 0.6
wall-milliseconds, the whole field layer at 0.32 to 0.71 core-milliseconds,
and the character tier at under 0.14 core-milliseconds in Rust.[^16] With
this row the economic stack of trade, economy, and needs costs about 1.8
wall-milliseconds. The total tick is 12 to 46 wall-milliseconds.[^4] **The
economic stack is about 6 percent of the mean tick.**

**Every figure in that table is derived and not measured.**[^16] Treat the
figures in this report the same way. The core-millisecond column is the
reliable one, and the target platform must confirm both.

### 12.5 Storage

| Item | Size |
|---|---|
| Cohort rows, 40,000 x 56 bytes | 2.24 MB |
| Quality pairs, 5,000 pools x 64 commodities x 8 bytes | 2.56 MB |
| Institution rows, 300 x 64 bytes | 19 KB |
| Technical coefficient matrix, 64 x 64 `i32`, one per faction | 16 KB each |
| Gross output and price vectors, 8 factions x 64 x 8 bytes | 4 KB |
| Threshold planes, 5 classes over 40,000 cohorts | 25 KB |
| Unrest plane, 65,536 cells x 4 bytes | 256 KB |
| **Total** | **about 5.2 MB** |

Compare that against 21.0 MB of fog of war for each faction.[^4] This
subsystem is small.

---

## 13. Conservation and the invariants to test

Three tests protect this subsystem. Each is an exact integer equality, not
a tolerance.

**Test 1: money is conserved.** Sum the money commodity over every pool,
every cohort's wealth, and every in-flight payment. The total changes only
by an explicit mint or burn event. Assert the equality on every economy
tick in a debug build, and on a sampled cadence in a release build.

**Test 2: goods are conserved.** The neighbouring report already states
it: the change in total stock equals total production minus total
consumption minus total spill.[^1] Cohort consumption is one more term in
the same sum. Do not add a second test.

**Test 3: headcount is conserved.** The sum of every cohort's headcount
changes only by a birth event, a death event, a starvation handler or a
migration transfer. A migration is a `transfer`, so it conserves by
construction. This test catches the most likely bug in the whole
subsystem, which is a headcount scaling that rounds a person out of
existence.

**Why integer arithmetic gives all three.** Every quantity is an `i64`.
Every movement of a quantity is a `transfer`, which computes a floor for
each receiver and then distributes the exact remainder by the
largest-remainder method in a canonical order.[^4] The sum of the payments
equals the amount withdrawn, exactly, by construction of that method.[^17]
There is no rounding error, so there is nothing to accumulate over a
million ticks. A floating-point implementation of the same allocation
loses or gains a fraction on every operation, and at 10 Hz that is 36,000
opportunities to drift in one hour.

**Where rounding could still bias the result.** Three places. Each is
stated with its direction, because a stated bias is a design choice and an
unstated one is a bug.

| Place | Direction | Size | Mitigation |
|---|---|---|---|
| The Neumann shift, section 3.3 | Undercounts gross output | at most 32 Q16.16 steps | None needed. It is a plan, not a stock. |
| The exchange divide, section 9.2 | Cohorts under-buy | under 1 unit per cohort per commodity | None needed. It cannot drive wealth negative. |
| The tax multiply, section 8.3 | The state under-collects | up to 1 unit per cohort per period | Apply largest remainder across the institution's cohorts if exactness matters. |

None of the three leaks value out of the world. Each moves value between
two parties by less than one unit. **A leak would be a different and more
serious matter, and there is none, because no quantity is ever discarded:
overflow becomes a `Spilled` event and never a silent drop.**[^1]

---

## 14. Prior art

This section separates what a published or official source states from
what only a community source states. The merge notes record that unsourced
claims about game internals have already caused a problem in this
project.[^16] Every claim below is marked.

### 14.1 Input-output analysis

**Verified.** Leontief published the model in 1936 and the full treatment
in 1941, revised in 1951.[^7][^8] **Cite the 1941 first edition, not 1951,
when the claim is about priority.** The merge notes record that this
distinction has been got wrong before.[^16] Hawkins and Simon proved the productiveness condition in
1949.[^9] The equivalence between the Hawkins-Simon condition,
non-negative invertibility and a spectral radius below 1 is standard in
the non-negative matrix literature.[^10] The Neumann series and its
truncation bound are standard results in matrix analysis.[^11]

**The engine is not the first to use it, but the fit is unusually good
here.** The project's recipe table is already a coefficient matrix and its
commodity ceiling of 64 is already the matrix dimension.[^1]

### 14.2 Constant elasticity of substitution

**Verified.** Arrow, Chenery, Minhas and Solow introduced the family in
1961.[^12] Section 3.5 rejects it for the tick loop and states why.

### 14.3 Population units in the Victoria series

**Partly verified.** Paradox Interactive documents the population-unit
model in official developer diaries for both titles. A population unit
holds a size, a profession or occupation, a culture, a religion and a
wealth level, and it buys goods against a set of need tiers.[^14][^15]

**Mark as community-sourced:** the exact number of population units in a
running game, the exact need tier boundaries, and the exact market
clearing formula. These appear on community wikis and in forum posts. The
official diaries describe the model qualitatively. **Do not cite a
specific population-unit count as a fact.**

**What the project takes from it.** The idea that a row stands for many
people, that the row buys against tiered needs from a settlement market,
and that unmet needs drive a discontent scalar. The project's cohort is
the same idea with a fixed count of 8 per settlement, which the Victoria
model does not have.

### 14.4 Needs in colony simulators

**Mark as community-sourced.** The needs systems of Dwarf Fortress and
RimWorld are documented on community wikis rather than in a published
specification. Both games give a character a set of decaying need or mood
scalars, and both apply a penalty when a need is unmet. **The engine takes
the shape and not any number.** Do not cite a decay rate from either.

The general design lesson is available without a citation: both games
model needs at the individual level, and both run at hundreds to a few
thousand agents rather than at one million. **That is direct evidence for
the cohort recommendation**, not against it.

### 14.5 Utility-based action selection

**Mark as partly unverifiable.** The Sims uses a motive-decay model with
utility-scored actions advertised by objects. The design is widely
described but a precise published specification of the decay rates is not
available. The engine takes only the shape: score each option by the
weighted unmet need it satisfies, then take the argument of the maximum.
That is the decision kernel of section 4.3.

### 14.6 Production chains in city and colony builders

**Mark as community-sourced.** The Anno series and comparable titles use
fixed-proportion production chains with population tiers whose needs
unlock further chains. The **structure** matches Leontief exactly: fixed
proportions, no substitution, a chain of intermediate goods. The engine
takes the confirmation that fixed proportions are playable and expected in
this genre, which is the main practical answer to section 3.5's objection.
Do not cite specific ratios.

### 14.7 Agent-based computational economics

**Verified.** Tesfatsion's survey defines the field and states its central
method: a market outcome emerges from interacting agents rather than from
an equilibrium condition imposed from outside.[^18]

**The engine sits deliberately outside this tradition, and the report
should say so.** Agent-based computational economics buys emergence by
paying for per-agent decisions. Section 4.3 shows that this project cannot
pay that price at one million rows. The cohort is the compromise: 40,000
decision-makers is enough for a market to show a pattern, and it is 25
times cheaper than one million.

### 14.8 Exact allocation

**Verified.** The largest-remainder method that the `transfer` verb uses is
Hamilton's apportionment method. Balinski and Young give its properties
and its failure modes.[^17] The property this project needs is the only
one it guarantees without qualification: **the parts sum to the whole,
exactly.**

**The method carries a known anomaly and the project has accepted it.**
Under the Alabama paradox, adding one unit to the total can reduce an
individual share. Balinski and Young prove that no method avoids the
anomaly while it also satisfies quota. The merge notes record this as an
accepted trade rather than a defect to fix.[^16] It becomes visible in this
subsystem when a settlement's stock rises by one unit and one cohort
receives less food than before. **Expect a player to report it as a bug.**
Nothing in this report changes that trade, and no cheaper method removes
it.

---

## 15. Ready-to-apply ADR decision block

Copy this section into the decision record. The numbers D90 to D109 and
OQ60 to OQ69 are reserved for this report and do not collide with any other
report's range.

### Part J — Needs, consumption and the economy

#### D90. Two need-bearing populations. A unit does not decide

The engine holds two populations that carry a need, and they use different
machinery.

| Population | Rows | Needs handled by | Decides |
|---|---|---|---|
| Units | 1,000,000 | The existing upkeep recipe kernel | No |
| Cohorts | 40,000 | The needs kernel of D92 | Yes |
| Characters | a few thousand | Per-entity | Yes |

A unit's upkeep is a fixed recipe and its owner decides for it. This removes
per-agent decision-making from the one-million-row population entirely,
which is the only way the budget closes. It is also correct simulation: a
soldier does not choose their rations.

**The owner has confirmed that a unit is an individual soldier and not a
formation, so upkeep is per unit at about one million entities.**[^16] That
confirmation does not change this decision. It fixes the row count that the
existing upkeep kernel must carry, and that kernel is already costed at that
count. What this decision removes is the **search**, not the upkeep.

Source: report 15, sections 4.3 and 5.2.

#### D91. The cohort is the unit of consumption and of decision. 8 strata, 40,000 rows

A cohort is one row that holds a headcount and one shared state vector. Its
index is `pool * 8 + stratum`, so the array is **sorted by pool by
construction**. The segmented reduce that consumption needs therefore costs
no sort.

The row holds the pool, the headcount, the stratum, the culture, the wealth,
4 need scalars, 4 quality scalars, loyalty and unrest. It is 56 bytes.
40,000 rows cost 2.24 MB. Store it as struct-of-arrays.

The ceiling is set by cost, not by taste. A decision costs about 400
nanoseconds. At a period of 10 ticks and a budget of 4 core-milliseconds
per tick, the engine affords 100,000 deciders. 40,000 leaves room for a
richer decision.

Source: report 15, sections 4.3, 5.3 and 12.3.

#### D92. Four needs. Q16.16. Saturating arithmetic. Rates are baked constants

The four needs are sustenance, water, rest and shelter. Each is an `i32` in
Q16.16 with a range of 0 to 65,536. Four `i32` values fill one 16-byte NEON
vector, so the decay kernel is one load, one saturating subtract and one
store.

Compute every decay rate at bake time as `65536 / (days * ticks_per_day)`,
scaled by the kernel period. **The tick kernel contains no division.**

Use `max(0)` on decay and `min(65536)` on satisfaction. Never a wrapping
subtract. A wrapping subtract turns a small shortfall into full
satisfaction, which is the classic bug in this exact kernel.

The clamp is safe here because a need is not a conserved field. It is
intensive state on a cohort. The commodity that satisfies it is conserved
and moves only through `transfer`.

Source: report 15, sections 6.1 to 6.3.

#### D93. Unmet need degrades continuously, then crosses a threshold. Reuse the existing pattern

Two stages. Do not invent a third pattern.

**Stage 1, dense and branchless.** A deficit accumulator rises while a need
is below its threshold and falls when the need recovers. It scales output,
lowers loyalty and raises unrest continuously. There is no event.

**Stage 2, the crossing.** When the accumulator reaches its bound, write a
predicate bit into a dense bitset plane, take a barrier, then scan the plane
in ascending word order and run the handler for each set bit. This is the
existing threshold pattern, unchanged. **The handler runs in ascending row
index order and nothing else.**

Five planes: starvation, thirst, desertion, revolt, vacancy. At 40,000
cohorts a plane is 5 KB.

The accumulator is what removes the cohort cliff. A cohort at 60 percent
food does not starve; it degrades. Only sustained deprivation crosses. The
handler then reduces the headcount by a fraction, which gives a partial
outcome from a whole-cohort model.

Source: report 15, section 6.4. Pattern source: report 12, section 8.

#### D94. Quality is an intensive pair. It scales the amount, and its surplus feeds contentment

Store quality per pool per commodity as the pair `(sum of amount times
quality, stock)`. The stock is already the weight, so quality costs one
extra `i64` per cell, which is 2.56 MB at 5,000 pools and 64 commodities.
The pair is a group, so a delta update is exact and no rescan is needed.

Read the quality by dividing at read. **Multiply before dividing.** Compute
`(amount * weighted) / weight`, never `amount * (weighted / weight)`.

Satisfaction:

```
gain         = (amount * quality) >> 16
need        += min(gain, 65536 - need)
surplus      = gain - min(gain, 65536 - need)
contentment += (surplus * LUXURY_GAIN) >> 16
```

Better food satisfies **more**, until the need is full. Beyond that point it
satisfies **differently**, through contentment, which drives loyalty. This
is what gives a fed population a demand for luxury, and therefore what gives
trade a purpose after subsistence.

Clamp the stored quality to the band 16,384 to 262,144, which is 0.25 to
4.0. Enforce it at bake time and at every production site.

Source: report 15, section 7.

#### D95. Money is commodity slot 0. It is excluded from the transport solve

Money lives in the existing per-pool commodity array at index 0. It is an
`i64`, like every other stock.

The reason is conservation, and it is decisive. The existing conservation
test sums every pool, every in-flight quantity and every spill record. If
money is in that array, the same test covers money, with the same code, at
the same time. A separate treasury column is a second thing to check and
therefore a second thing to get wrong.

Two special rules. Money does not enter the transport solve, because a
payment is an instruction and not a flow across a field; money moves by an
explicit `transfer` between two named pools. Money has no price and no
quality, because money is the numeraire.

An institution's `treasury` is an accessor onto its seat pool's slot 0. It
is not a second store of money.

Source: report 15, sections 8.1 and 10.1.

#### D96. A wage is a map over cohorts, then one `transfer`. There is no per-individual transaction

Three phases, each a separate pass.

1. **Compute the obligation.** A map over 40,000 cohorts:
   `obligation = (headcount * wage_rate[institution][stratum]) >> 16`.
2. **Reduce per institution.** A segmented reduce. The cohort array is
   already sorted by pool and a pool has one institution, so no sort runs.
3. **Pay.** One `transfer` call per institution, over its cohorts, in
   ascending priority band and then ascending cohort index.

If the treasury cannot pay everything, the bands decide who is paid and the
largest-remainder split inside the failing band is exact. The sum of the
payments equals the withdrawal exactly. **No money is created and none is
lost.**

The cost is under 0.05 core-milliseconds at 40,000 cohorts. A character's
salary is a separate row in the same obligation list at a higher band, and
a per-entity payment is affordable at a few thousand characters.

Source: report 15, section 8.2.

#### D97. Taxation is the same kernel with the sign reversed. Prefer a consumption tax

Tax uses the same three phases as D96 and the same `transfer` verb. Never
write a subtract on one side and an add on the other; that is two operations
that can disagree.

Prefer a **consumption tax**: a fraction of the money value of each
purchase, taken during the exchange pass that already computes that value.
It costs one multiply and one shift, and it couples tax revenue to economic
activity, which is the behaviour the design wants.

Accept a poll tax and a wealth tax as alternative bases. Both are one
multiply.

**A stated rounding bias.** `(base * rate) >> 16` rounds down, so the state
under-collects by up to one unit per cohort per period. It is a systematic
bias and not a leak: the untaken remainder stays with the cohort. If the
design needs the collection exact, apply the largest-remainder method across
the institution's cohorts.

Source: report 15, section 8.3.

#### D98. Exchange is one allocation per settlement, not a market. There is no order book

Reject an order book, a double auction and any per-agent bid loop. Each
needs per-agent iteration, which D90 and D91 exist to avoid.

The pass, per settlement, over its 8 cohorts:

```
1  desired    = need_gap * per_head * headcount
   affordable = (wealth * SPEND_FRACTION >> 16) / price
   demand     = min(desired, affordable)
2  reduce demand over the 8 cohorts
3  transfer(stock, demands, priority_bands)      // this IS the market
4  cost = received * price >> 16;  one transfer of money the other way
5  accumulate excess = demand - supply for the price step
```

Step 3 is the whole market and it is the existing `transfer` verb. Goods
leaving the pool equal goods reaching the cohorts, exactly. Priority bands
express "the soldiers eat first" with no second mechanism.

Step 4 must be **one** transfer over the money commodity, so that the
conservation test covers it.

Prices are unchanged: one clamped tatonnement step per tick, in a band of
one quarter to seven quarters of a base price. This decision adds only the
demand term that feeds the excess, and the base price of D99.

Total cost: **0.50 to 0.98 core-milliseconds.** If the 160,000 divides in
step 1 measure badly, precompute a reciprocal once per settlement per
commodity, which is 20,000 divides instead of 160,000.

Source: report 15, section 9. Price source: report 11, D58.

#### D99. The Leontief plan is a Neumann series with a fixed count of 32 terms

The recipe table already is a technical coefficient matrix. Divide each
input rate of a recipe by that recipe's output rate to get a column of `A`.
No new data is needed.

Solve `x = A x + d` by the truncated Neumann series, never by inversion:

```
term = d;  x = d;
for _ in 0..32 { term = A * term; x += term; }      // 32 is a constant
```

Solve the dual `p = A' p + v` with the same 32 terms on the transpose. The
result is a cost of production for every commodity, and it is the base price
that D98's band centres on. It is far better than a hand-typed constant and
it moves when technology changes.

**Cap the largest column sum of `A` at 0.6875, which is 45,056 in Q16.16.**
Check the cap at bake time. The relative truncation error is then 1.4 times
10 to the power minus 5, which is 0.89 of one Q16.16 step. The cap means an
activity turns 0.6875 units of input into 1 unit of output, a margin of 45
percent, which is what a strategy game wants anyway.

| Largest column sum | Terms needed for one Q16.16 step |
|---|---|
| 0.50 | 16 |
| 0.60 | 23 |
| **0.6875** | **32** |
| 0.75 | 43 |
| 0.90 | 127 |

The matrix is 64 by 64 `i32`, which is 16 KB and stays in the level-1 cache
for the whole solve. Two solves for 8 factions at a period of 40 ticks cost
**0.15 core-milliseconds amortised**. It is the cheapest item in this
subsystem.

The right shift biases the result downwards by at most 32 Q16.16 steps.
That is acceptable, because `x` is a plan and not a stock. **Never use the
result of this solve as a stock.**

Source: report 15, sections 3.1 and 3.3.

#### D100. The Hawkins-Simon condition is a bake-time content check

For a non-negative `A`, the system `(I - A) x = d` has a non-negative
solution for every non-negative `d` if and only if every leading principal
minor of `(I - A)` is positive. For a non-negative `A` this is equivalent to
a spectral radius below 1.

A sufficient condition that one pass can check: **every column sum of `A` is
below 1**. Its meaning is plain to a designer: an activity must consume less
than it makes.

Run the check when the engine bakes the type tables. Fail the build. The
same reduce also enforces the 0.6875 cap of D99. The cost is microseconds.

Source: report 15, section 3.2.

#### D101. Runtime insolvency is an explicit ledger comparison, not a failed solve

**Reject the proposal that a failed input-output solve is the insolvency
mechanic.** Four reasons.

1. `A` comes from the immutable recipe table, so it is content, not state.
   Every faction that shares a recipe table shares the same spectral radius.
   A test on `A` cannot distinguish a rich faction from a bankrupt one.
2. Insolvency is a flow condition about income, obligations and a treasury
   balance. It is not a structural property of `A`.
3. In integer arithmetic with saturating addition, a divergent series does
   not raise an error. It saturates. Detecting that means an explicit
   magnitude comparison, which is the rule the proposal wanted to avoid.
4. "The linear system has no non-negative solution" cannot appear in a user
   interface. "You owe 1,200 gold and hold 300" can.

The engine holds **two** distinct insolvency states and shows both.

```
obligations = wages + unit upkeep in money + construction due + debt service
available   = treasury + projected income

financially insolvent   when available < obligations
structurally insolvent  when any component of the Leontief gross output x
                        exceeds the faction's productive capacity
```

Every term is an `i64`, so the comparison is exact and needs no tolerance.
The structural test is a comparison of two vectors of length 64 and costs
nothing. It is the one valuable piece of the original proposal, and it is
readable: "you need 400 iron per tick and you can make 250."

Source: report 15, sections 3.4 and 10.2.

#### D102. The insolvency cascade runs in a fixed six-stage order

> The cascade runs in ascending institution index order. Within one
> institution it runs in ascending priority band, then ascending obligation
> index. No stage reads a value that a later stage writes in the same
> economy tick.

| Stage | Work | Kernel |
|---|---|---|
| 1 | Sum obligations per institution | segmented reduce |
| 2 | Pay in ascending band. `transfer` splits the failing band. | map plus transfer |
| 3 | Add the unpaid amount to arrears. Emit one `Unpaid` event each. | map |
| 4 | Reduce loyalty for each unpaid party as a function of arrears | map |
| 5 | Write the crossing predicates into the bitset planes | dense map |
| 6 | Scan the planes and run the handlers, ascending index | sparse scan |

Take a barrier between each stage, so that no stage depends on how the
previous stage was scheduled.

Stage 6 raises three event classes and this subsystem owns no handler for
any of them. `OfficeVacated` sets the office's holder column to the vacant
value and hands the office to the succession machinery, which selects a
replacement by filter, then sort, then allocate.[^3] `Revolted` goes to a
handler outside this subsystem. `Defaulted` goes to the design.

The owner states that losing a position should be rare. The model gives
that without a special case: loyalty is a slow accumulator over many economy
ticks, and the vacancy threshold sits far from the resting value. Rarity is
a constant.

Source: report 15, section 10.3. Succession source: report 14.

#### D103. Loyalty is a per-entity scalar. Unrest is a field

Loyalty is one `Fix32` column on the cohort row and on the character row.
It is **not** a field. The character tier does not hold that column today,
so adding it is a prerequisite of this decision.[^3] It fails the field membership test on the decisive
row: a field cannot follow an entity, and a cohort that migrates keeps its
loyalty. Diffusing loyalty would let an unrelated neighbouring stratum's
loyalty leak into a cohort's own value.

The update is one map over 45,000 rows with six weighted terms: the paid
wage ratio, need satisfaction, contentment, the sampled unrest, the
institution's legitimacy, and arrears. Damp it by 1/16 or less, so that
loyalty needs at least 16 economy ticks to cross a quarter of its range.
Without the damping, loyalty oscillates and the vacancy threshold fires on
noise.

Unrest **does** pass the field test: it belongs to a place, it spreads to a
neighbour, and it does not follow an entity. Run it as **one more plane in
the existing influence-map solver** at L1, with a fixed 8 sweeps and a decay
term. Its marginal cost is about 0.07 core-milliseconds.

The two couple both ways and both are cheap. Low cohort loyalty is a source
into the unrest plane, by a scatter in ascending cohort index order. High
unrest is a term in the loyalty update, by one gather per cohort.

This split gives the design what it needs. Loyalty answers "will this person
stay". Unrest answers "is this province about to rise". A revolt needs both.

Source: report 15, section 11.

#### D104. Reject CES and linear programming. Substitution is a discrete recipe switch

Leontief assumes fixed proportions and no substitution. Three ways to add
substitution exist and two are rejected.

**Reject constant-elasticity-of-substitution production.** It needs a
fractional power, which in fixed point is a logarithm and an exponential,
each a table lookup plus interpolation. At 2.2 inputs per recipe over 40,000
cohorts and 50,000 producers that is 4 to 8 core-milliseconds of pure
transcendental work for a result the player cannot see. The cost-optimal
input mix also needs a division by a price, which truncates.

**Reject linear programming at the tick.** A simplex solve takes a
data-dependent pivot count, which breaks a fixed frame budget. Exact
rational arithmetic makes the numerators grow without a bound. Integer
programming is worse.

**Adopt discrete substitution at a slow cadence.** Give an activity 2 to 4
alternative recipes and choose by an argument of the maximum over the
delivered cost at current prices. Every operation stays an integer add,
multiply or shift. A designer writes "a bakery may use wheat or rye" rather
than tuning an elasticity. The switch is a discrete, visible, loggable
event.

**Damp the flip.** A step function makes a whole population switch at one
price point. Switch only when the alternative is cheaper by at least 1/16 of
the current cost. That margin is a shift, not a division.

Source: report 15, section 3.5.

#### D105. The cadence table for the needs and economy subsystem

| Work | Period | Phase |
|---|---|---|
| Needs decay | 10 | 0 |
| Cohort consumption reduce | 10 | 0 |
| Exchange, payment, taxation | 10 | 1 |
| Cohort decisions | 10 | staggered by faction |
| Degradation accumulators | 10 | 1 |
| Threshold phase A and phase B | 10 | 2 |
| Price tatonnement step | 1 | unchanged |
| Wages | 100 | 0 |
| Solvency ledger and cascade | 100 | 1 |
| Loyalty update | 100 | 2 |
| Unrest diffusion plane | 10 | in the influence solver |
| Leontief plan and dual prices | 40 | staggered by faction |
| Hawkins-Simon check | bake time | — |

The order within one economy tick extends the existing three-part contract:

```
production and upkeep rates, segmented reduce
cohort needs decay and consumption rates
--- the transport solve ---
resolve caps through transfer
exchange, payment, taxation
cohort decisions, on the stagger
degradation accumulators
threshold phase A, barrier, phase B
```

**Stagger the decisions by faction.** One eighth of the factions on each
tick flattens a 16 core-millisecond spike to 1.6 to 2.0 core-milliseconds
per tick, with no change to the total and no new machinery.

Source: report 15, section 12.

#### D106. The per-tick budget for needs, consumption and the economy

| Work | Scale | Core-ms per economy tick |
|---|---|---|
| Needs decay | 40,000 x 4 | 0.19 |
| Consumption rates and reduce | 160,000 deltas | 0.23-0.46 |
| Exchange, payment, taxation | 160,000 cells | 0.50-0.98 |
| **Cohort decisions** | 40,000 at 400 ns | **16.00** |
| Degradation accumulators | 40,000 x 4 | 0.10 |
| Threshold phase B, sparse | 1 percent crossing | 0.10 |
| Wages, tax, solvency, at period 100 | amortised | 0.05 |
| Loyalty, at period 100 | amortised | 0.01 |
| Unrest plane | 65,536 cells x 8 | 0.07 |
| Leontief plan and dual, at period 40 | amortised | 0.15 |
| **Total per economy tick** | | **17.4-18.1** |
| **Amortised at period 10** | | **1.7-1.8** |

Add this row to the per-tick cost budget table:

| Work | Scale | Core-ms | Wall-ms (12c) |
|---|---|---|---|
| Needs, consumption, exchange, wages, solvency | 40,000 cohorts, period 10, staggered | 1.7-1.9 | 0.25-0.35 |

**The decision pass is 92 percent of the cost.** Everything else together is
under 1.5 core-milliseconds. The cost is linear in the cohort count, so
halving the strata halves it. Storage for the whole subsystem is 5.2 MB.

Source: report 15, section 12.

#### D107. Three exact conservation tests protect this subsystem

Each is an integer equality, not a tolerance.

1. **Money.** Sum the money commodity over every pool, every cohort's
   wealth and every in-flight payment. The total changes only by an explicit
   mint or burn event.
2. **Goods.** The existing test already covers this. Cohort consumption is
   one more term in the same sum. Do not add a second test.
3. **Headcount.** The sum of every cohort's headcount changes only by a
   birth, a death, a starvation handler or a migration `transfer`. This
   test catches the most likely bug in the subsystem, which is a headcount
   scaling that rounds a person out of existence.

Run all three on every economy tick in a debug build, and on a sampled
cadence in a release build.

**Three rounding biases exist, and each is stated deliberately.** The
Neumann shift undercounts gross output by at most 32 Q16.16 steps, which is
harmless because the result is a plan. The exchange divide makes cohorts
under-buy by under 1 unit, which is safe because it cannot drive wealth
negative. The tax multiply makes the state under-collect by up to 1 unit per
cohort per period, which moves value between two parties and does not leak
it.

**None of the three leaks value out of the world.** No quantity is ever
discarded: overflow becomes a `Spilled` event, never a silent drop.

Source: report 15, section 13.

---

## 16. Open questions

**OQ60. What is the total population, and is it separate from the one
million units?** This report assumes it is separate. The engine cost does
not depend on the answer, because a headcount is one `u32` field. The answer
sets the mean cohort size, which sets whether losing 1 percent of a cohort
is 10 people or 10,000.

**OQ61. Are 8 strata the right number, and which 8?** The fixed count of 8
is what makes the cohort array sorted by pool by construction, so the
segmented reduce needs no sort. Changing 8 to a variable count costs a sort
of 40,000 rows on every structural change. **Prefer to change which 8 rather
than how many.**

**OQ62. How many ticks are in one game day?** Every decay rate in D92 is
`65536 / (days * ticks_per_day)`. This report assumes 600. The rates are
baked constants, so the answer costs nothing to change, but it must be
answered before the content is written.

**OQ63. Does the design want a visible price, or only visible scarcity?** A
visible price needs the dual Leontief solve of D99 and the tatonnement step.
Visible scarcity alone needs neither. The saving is 0.15 core-milliseconds,
which is small, so the answer is a design question and not a budget
question.

**OQ64. Is there one currency, or one per faction?** One currency makes
money one commodity slot. One per faction makes money `faction_count`
slots, which at 8 factions consumes 8 of the 64 commodity slots and
complicates the conservation test. **This report assumes one currency.**

**OQ65. How many institutions exist?** This report assumes a few hundred.
The storage is trivial at that count. At tens of thousands the cascade of
D102 becomes a per-institution pass rather than a small loop, and the
serial stages would need review.

**OQ66. Can a cohort migrate, and does migration need a decision?**
Migration as a headcount `transfer` between two cohorts of the same stratum
is exactly conserving and cheap. Migration as a **decision** adds a search
over candidate destinations to the tier-3 budget, which is the expensive
tier. If migration is a decision, state its option count.

**OQ67. May the technical coefficient matrix differ per faction?** It costs
16 KB per faction, so the storage is free. But a shared matrix makes the
Hawkins-Simon check a single bake-time test, and a per-faction matrix makes
it a per-faction runtime test. **A per-faction matrix is the mechanism that
would make structural insolvency dynamic**, and it is the one way to
recover part of the lead's original proposal.

**OQ68. Two columns must be added to the character tier.** The character
report specifies a character row and an office row, and **neither holds a
wealth field or a loyalty field**.[^3] This report needs both: a salary
needs somewhere to land, and D103 puts loyalty on the character. Two `i64`
and `i32` columns on the character arena are the smallest change. Confirm
with the owner of that tier before either report is merged.

**OQ69. What is the cohort decision, exactly?** The budget of 400
nanoseconds per decision assumes 4 to 8 gathers and a short comparison
chain. A richer decision costs more and the cohort count must then fall.
Fix the option count before the cohort count.

---

## References

[^1]: Research report 12, Entity Economy and Modifiers. Sections 1, 5, 7, 8, 9 and 11, and decisions D51 to D59. `docs/adrs/background/adr-0001/12-entity-economy-and-modifiers.md`
[^2]: Research report 11, Resource and Trade Flow. Sections 3.5, 3.6 and 6.4, and decisions D54, D58 and D60. `docs/adrs/background/adr-0001/11-resource-and-trade-flow.md`
[^3]: Research report 14, Character Graph and Inheritance. Sections 5 and 6, and decisions D77 and D79. `docs/adrs/background/adr-0001/14-character-graph-and-inheritance.md`
[^4]: ADR-0001, Foundational Architecture. Decisions D4, D5, D16, D27, D29, D42, the byte budget section and the per-tick cost budget section. `docs/adrs/draft/adr-0001-foundational-architecture.md`
[^5]: Research report 13, Field Operator Algebra. Sections 4.7, 5.3 and 6.2 to 6.4. `docs/adrs/background/adr-0001/13-field-operator-algebra.md`
[^6]: ADR-0001, Foundational Architecture. Decision D16, the corrected aggregation rule and its statistics table. `docs/adrs/draft/adr-0001-foundational-architecture.md`
[^7]: Leontief, W. W., 1936. Quantitative Input and Output Relations in the Economic System of the United States. The Review of Economics and Statistics, 18(3), pp. 105-125. https://doi.org/10.2307/1927837
[^8]: Leontief, W. W., 1941. The Structure of American Economy, 1919-1929: An Empirical Application of Equilibrium Analysis. Cambridge, Massachusetts: Harvard University Press. The revised and enlarged second edition, covering 1919-1939, appeared in 1951 from Oxford University Press.
[^9]: Hawkins, D. and Simon, H. A., 1949. Note: Some Conditions of Macroeconomic Stability. Econometrica, 17(3/4), pp. 245-248. https://doi.org/10.2307/1905526
[^10]: Berman, A. and Plemmons, R. J., 1994. Nonnegative Matrices in the Mathematical Sciences. Classics in Applied Mathematics 9. Philadelphia: SIAM. Chapter 6 covers M-matrices, the Hawkins-Simon condition and the equivalence with a spectral radius below 1. https://doi.org/10.1137/1.9781611971262
[^11]: Horn, R. A. and Johnson, C. R., 2013. Matrix Analysis. 2nd ed. Cambridge: Cambridge University Press. Section 5.6 gives the Neumann series, its convergence condition and the truncation bound.
[^12]: Arrow, K. J., Chenery, H. B., Minhas, B. S. and Solow, R. M., 1961. Capital-Labor Substitution and Economic Efficiency. The Review of Economics and Statistics, 43(3), pp. 225-250. https://doi.org/10.2307/1927286
[^13]: Research report 09, Influence Maps. Section 7.5, the cost of the whole schedule. `docs/adrs/background/adr-0001/09-influence-maps.md`
[^14]: Paradox Interactive. Victoria II. Official developer diary series, 2010 to 2012, published on the Paradox Interactive forums. The source of the population-unit model that this report adapts. **The exact diary numbers and their URLs were not verified when this report was written. Verify before publication.**
[^15]: Paradox Interactive. Victoria 3. Official developer diary series, 2021 onward, published on the Paradox Interactive forums and as Steam news items. The source of the pop and market model that this report adapts. **The exact diary numbers and their URLs were not verified when this report was written. Verify before publication.**
[^16]: Merge Notes for ADR-0001. Section 2, owner decisions; section 9, the accepted Alabama paradox trade; and the running per-tick budget table. `docs/adrs/background/adr-0001/MERGE-NOTES.md`
[^17]: Balinski, M. L. and Young, H. P., 1982. Fair Representation: Meeting the Ideal of One Man, One Vote. New Haven: Yale University Press. The largest-remainder method appears there as Hamilton's method of apportionment.
[^18]: Tesfatsion, L., 2006. Agent-Based Computational Economics: A Constructive Approach to Economic Theory. In: L. Tesfatsion and K. L. Judd, eds. Handbook of Computational Economics, Volume 2: Agent-Based Computational Economics. Amsterdam: North-Holland, chapter 16, pp. 831-880. https://doi.org/10.1016/S1574-0021(05)02016-2
