# Vector Entity Representation and Derived Opinion

Research report 18 for the foundational architecture decision record.

## 0. Context

Cachette is a world simulation engine. The core is Rust. The control plane
is Python. The engine simulates a hex world at three levels of detail.
Level 0 holds 16,777,216 tiles in a 4,096 by 4,096 grid. Level 1 holds
65,536 cells. One level 1 cell summarises 256 tiles. Level 2 holds 256
blocks. The target scale is 16,777,216 tiles and one million units.[^1]

The engine runs a fixed timestep at 10 Hz. The frame loop splits into a
read phase and a write phase. Python attaches before the first phase and
after the last phase. Python never runs while the simulation runs.[^1]

Four rules bind every recommendation in this report.

- **No floating point in simulated or aggregated state.** The fixed-point
  scale is Q16.16 for stats and modifiers. `Fix32` is `i32`. `Accum` is
  `i64`.[^1]
- **Every random draw is a keyed hash of the tuple (system, frame, entity,
  draw).** No thread-local generator is allowed.[^1]
- **Every parallel result is ordered by a stable key.** Thread completion
  order, work-stealing order and hash iteration order are forbidden.[^1]
- **The target is AWS Graviton.** The vector unit is NEON at 128 bits. The
  cache line is 64 bytes. A Neoverse V1 core holds a 1 MB private level 2
  cache.[^1]

This report evaluates one proposal from the project owner.

> Each individual is an n-dimensional vector that changes over time. A
> relation is a decaying vector modification applied across a k-nearest-
> neighbour search of the target.

### 0.1 The eight findings

**Finding 1. Adopt the vector, for characters and cohorts only.** The model
is sound at about 302,000 entities. It is not sound at one million units. A
unit does not decide, so a unit holds no opinion. Section 7 gives the
argument and section 8 gives the one thing a unit still needs.

**Finding 2. The storage argument is wrong, and it is the weakest argument
for the proposal.** The vector adds about 18.7 MB at the character ceiling.
Cutting the relation-edge degree saves about 21.0 MB. The two figures
cancel. Section 3 gives the arithmetic and corrects a related claim: sparse
relation edges grow **linearly** with the character count at a fixed mean
degree, not with the square of it.

**Finding 3. The real argument is coverage, and it is very strong.** An
edge model answers the question "what does A think of B" for the 2.1
million stored pairs. A derived model answers it for all 34.4 billion
unordered pairs. The dense equivalent costs 34.4 GB. **The vector buys 34.4
GB of coverage for 18.7 MB.** State the case this way and it holds.

**Finding 4. Never build a global nearest-neighbour index, and the reason
is not cost.** A rebuild is affordable if you choose the cheapest structure:
locality-sensitive hashing over 262,144 points costs about 86
core-milliseconds for each rebuild, which is 2.7 core-milliseconds for each
tick when amortised over 32 ticks. An exact search costs 141 core-seconds
and a hierarchical graph rebuild costs 2.7 core-seconds. The correct reason
to refuse a global index is that it answers a question the design does not
ask. Section 5 gives the table and the replacement.

**Finding 5. Twelve named dimensions, and `i16` in Q8.8.** Distance
concentration begins near 10 to 15 dimensions for independent data. Real
personality axes correlate, so the effective dimension is lower than the
stored dimension. Twelve is the recommendation. Manhattan distance is exact
in `i32`, needs no square root, and preserves contrast better than
Euclidean distance in this range. Section 6 gives the evidence and the
dimension set.

**Finding 6. The layout recommendation reverses the usual advice.** The
character access pattern is a random gather over graph neighbours, so
struct-of-arrays costs 12 cache lines for each candidate and
array-of-structs costs 1. **Use array-of-structs for characters and
struct-of-arrays for cohorts.** Pack the current vector and the anchor
vector into one 64-byte line. Section 9 gives the 12-times figure.

**Finding 7. The effective-stat conflict is real, and one schema rule
removes it.** A continuous per-entity value in the modifier pipeline
collapses the configuration hit rate from 99.5 percent to zero and raises
the pipeline cost from 0.05 core-milliseconds to about 480
core-milliseconds. That is a factor of about 9,600 and it exceeds the whole
frame budget by an order of magnitude. **No vector dimension enters the
modifier pipeline.** Section 10 states the rule and the one exception the
budget could afford.

**Finding 8. Without one extra term the population converges to a single
point, and the convergence is proven, not suspected.** Averaging models
reach consensus on any connected graph. Adding cultural drift to the
best-known cultural model destroys its diversity and drives a finite system
to a single culture. The fix is one immutable anchor vector for each entity
and a pull toward it in every update. Section 2 gives the literature and
the proof sketch.

---

## 1. Terms

**Trait vector.** An ordered tuple of `n` fixed-point numbers held by one
entity. Each position is a named, ordered quantity. Position `d` means the
same thing for every entity.

**Anchor vector.** A second trait vector for the same entity. The engine
writes it once, at birth, and never changes it. It represents the innate
disposition that social influence cannot erase.

**Manhattan distance.** The sum of the absolute differences of the paired
coordinates of two vectors. The symbol is `L1`. The formula is
`sum over d of abs(a[d] - b[d])`.

**Derived opinion.** A value computed from the distance between two trait
vectors at the moment it is needed. The engine stores no value for the
pair.

**Relation edge.** A stored record that names two entities and holds a
value for the ordered pair. The engine stores the value.

**Candidate set.** The list of entities that may influence one entity in
one pass. This report takes candidate sets from existing structure, not
from a similarity search.

**Approximate nearest-neighbour index.** A data structure that answers "the
`k` closest points to this query" faster than a full scan, at the cost of a
small error rate. Hierarchical navigable small-world graphs, inverted file
indexes and locality-sensitive hashing are the three main families.

**Bounded confidence.** A rule in opinion-dynamics models. Two agents
influence each other only when their opinions differ by less than a
threshold.

**Homophily.** The observed tendency of similar people to form ties. A
model shows homophily when it makes interaction probability depend on
similarity.

**Cohort.** One row that stands for many identical individuals of one
stratum in one settlement. The engine holds 40,000 cohorts, which is 8
strata in each of 5,000 settlements.[^2]

**Character.** A named individual with an identity that persists after
death. The character tier is capped at 262,144 entities and exposes a
per-entity object model to Python.[^3]

**Post-stage multiplier.** A per-entity factor applied after the shared
modifier pipeline produces an effective stat. The engine caps the count of
these at 4.[^4]

---

## 2. The literature, and the convergence pathology

### 2.1 The five models that matter

The owner's proposal is a known model class. The class is opinion dynamics
and cultural dissemination in agent-based simulation. Five models define
it.

**DeGroot averaging (1974).** Each agent holds one opinion. At each step
each agent replaces its opinion with a weighted average of the opinions of
its neighbours.[^5] The result is a theorem, not a tendency. If the
influence graph is strongly connected and aperiodic, every agent converges
to the same value. The model **cannot** produce lasting disagreement.

**Axelrod cultural dissemination (1997).** Each agent holds a vector of `F`
features. Each feature takes one of `q` discrete traits. Two neighbouring
agents interact with a probability equal to their share of matching
features. On interaction, one agent copies one differing feature from the
other.[^6] This is the closest published model to the owner's proposal. It
has three properties the project wants: the vector is the whole state, the
interaction probability comes from similarity, and homophily is a
consequence of the rule rather than a written rule.

**Bounded confidence, Deffuant and colleagues (2000).** Two agents chosen at
random converge toward each other only if their opinions differ by less
than a threshold.[^7] Below the threshold the pair moves together by a fixed
fraction of the gap.

**Bounded confidence, Hegselmann and Krause (2002).** The same threshold
rule, but every agent updates at once, and each agent averages over all
agents inside its confidence range.[^8] The two papers agree on the
qualitative result. A large threshold gives consensus. A small threshold
gives a set of stable clusters. The cluster count falls roughly as the
inverse of the threshold.

**Friedkin and Johnsen social influence (1990).** Each agent keeps a fixed
susceptibility weight. At each step the agent mixes the neighbour average
with **its own initial opinion**, in a fixed ratio.[^9] The retained initial
opinion is the whole point of the model.

Homophily itself is the best-supported empirical claim in this area.
McPherson, Smith-Lovin and Cook survey the evidence and show that similarity
predicts tie formation across race, age, religion, education, occupation and
attitude.[^10] Castellano, Fortunato and Loreto give the standard physics
review of the whole family.[^11] Flache and six colleagues give the standard
review from the social-simulation side, and it is the best single entry
point.[^12]

### 2.2 What each model predicts, including the failures

| Model | Long-run outcome | Stable diversity | Failure mode |
|---|---|---|---|
| DeGroot | Consensus, always | **No** | Every agent reaches one identical value |
| Axelrod, no noise | Several frozen cultural regions | Yes, in a finite system | Diversity is a finite-size artefact |
| Axelrod, with noise | One culture | **No** | Any drift rate destroys the regions |
| Deffuant | Clusters set by the threshold | Yes, but brittle | Threshold tuning decides everything |
| Hegselmann and Krause | Clusters set by the threshold | Yes, but brittle | Same |
| Friedkin and Johnsen | A unique non-consensual fixed point | **Yes, provably** | None of this kind |

**The Axelrod row is the important one, and it is the direct answer to the
owner's test.** Axelrod's model is famous because it produces global
polarisation from local convergence. Klemm, Eguíluz, Toral and San Miguel
showed that this is not robust. In one paper they show that the
multicultural state is not the stable state of the model in the large-system
limit, and that the transition between one culture and many depends on the
trait count.[^13] In a second paper they add **cultural drift**, which is a
small random change to one feature of one agent at a low rate. Any non-zero
drift rate drives a finite system to a single global culture.[^14] The
polarisation is an artefact of a finite system with exactly zero noise.

**State the consequence plainly. If the project builds the owner's proposal
as written, every character holds the same vector after enough simulated
time.** The engine adds noise everywhere, because births perturb vectors and
events push them. That noise is exactly the drift that Klemm and colleagues
show is fatal.

### 2.3 The three mechanisms that prevent collapse

Three published mechanisms restore lasting diversity. All three are cheap.
Take all three.

**Mechanism 1. The anchor term, from Friedkin and Johnsen.** Give each
entity an immutable anchor vector, written at birth. Every influence update
mixes the neighbour pull with a pull back toward the anchor, in a fixed
ratio. The Friedkin-Johnsen fixed point is unique when every agent keeps a
positive weight on its own anchor, and the fixed point is a weighted mixture
of the **initial** vectors of the whole population.[^9] A population of
distinct anchors therefore has a distinct, non-consensual, stable end
state. **This is a proof, not an observation, and it costs one extra vector
for each entity.**

**Mechanism 2. Structural sparsity, from Centola and colleagues.** Centola,
González-Avella, Eguíluz and San Miguel added homophilous rewiring to the
Axelrod model. An agent drops a tie to a dissimilar neighbour and forms a
tie to a similar one. Cultural groups then survive cultural drift, because
the network splits into components that stop exchanging.[^15] The project
gets this mechanism free. Section 5 takes candidate sets from the kinship
and vassalage graph and from the settlement grid. Neither is a complete
graph. Distant houses never exchange, so they never converge.

**Mechanism 3. Population turnover.** A character has a mean lifespan of 60
simulated years.[^3] Over 500 simulated years there are about 8.3
generations. Each birth draws a new vector from the two parents plus a keyed
perturbation. Turnover injects variance for as long as the simulation runs.

Two further mechanisms exist and this report does **not** recommend them for
version 1. Mäs, Flache and Helbing show that an individualisation term, in
which an agent moves away from over-similar others, sustains clustering.[^16]
Macy, Kitts, Flache and Benard show that negative influence, in which
dissimilar agents push each other further apart, produces two opposed
camps.[^17] Both add a sign flip inside the update and both are one extra
compare. Record them as the levers to pull if mechanisms 1 to 3 still give
too little variance in testing.

### 2.4 The test the owner asked for

The owner asked whether all characters converge to identical vectors after
200 simulated years. The answer has two parts.

**Without the anchor term: yes, and it is proven.** The influence rule is a
weighted average, so it is a DeGroot process on the candidate graph. On any
connected component the process converges to one value.[^5] Drift makes it
worse, not better.[^14]

**With the anchor term: no, and that is also proven.** Each agent retains a
fixed weight on an immutable anchor, so the iteration is a contraction
toward a unique fixed point that is not the consensus point.[^9]

**Therefore the anchor term is not optional and it is not a tuning
parameter. It is the mechanism that makes the proposal work at all.**

---

## 3. Storage, measured against the character graph

### 3.1 What the character report actually specifies

The character report specifies relation edges of 8 bytes each, holding a
target identifier, a kind, a strength and a start time. It assumes a mean of
8 non-kinship edges for each living character and stores a symmetric edge at
both endpoints. That is 128 bytes for each living character. A dead
character drops its relation edges.[^3]

**Correct one claim before using it.** A sparse edge set at a **fixed** mean
degree grows **linearly** with the living character count. It does not grow
with the square of it. Quadratic growth needs the mean degree to grow with
the population, which no design in this project asks for. The table shows
both readings.

| Living characters | Mean degree 8 | Mean degree 24 | Dense matrix, 1 byte a pair |
|---|---|---|---|
| 10,000 | 1.3 MB | 3.8 MB | 50 MB |
| 100,000 | 12.8 MB | 38.4 MB | 5.0 GB |
| 262,144 | **33.6 MB** | **100.7 MB** | **34.4 GB** |

A figure above 100 MB at 262,144 characters therefore needs a mean degree of
about 24, not 8. Both are defensible design targets. The report's own figure
is 33.6 MB.

### 3.2 What the vector costs

Take 12 dimensions of `i16`, which is 24 bytes, plus an equal anchor vector,
which is 48 bytes. Round the character record to 64 bytes so it occupies one
whole cache line. Section 9 gives the reason.

| Item | Rule | 100,000 living | 262,144 living |
|---|---|---|---|
| Character vector plus anchor | 64 bytes each | 6.4 MB | 16.8 MB |
| Cohort vector plus anchor | 48 bytes each, 40,000 rows | 1.9 MB | 1.9 MB |
| **Total added** | | **8.3 MB** | **18.7 MB** |

### 3.3 The net, and why the storage argument fails

The vector does not remove the relation edges. Section 4 shows why. It
removes the **generic** part of them, so the mean degree falls. Assume the
degree falls from 8 to 3, because only specific and asymmetric facts still
need an edge.

| Item | 262,144 living |
|---|---|
| Relation edges saved, degree 8 to degree 3 | −21.0 MB |
| Trait vectors added | +18.7 MB |
| **Net** | **−2.3 MB** |

**The storage saving is about 2.3 MB, which is 0.9 percent of the tile side
alone.** The character arena is already small against the 268 MB tile side
and the 21.0 MB of fog of war for each faction.[^3][^1]

**Reject the storage argument. It is not a reason to adopt the proposal.**

### 3.4 The argument that does hold

An edge model can answer "what does A think of B" only for the pairs it
stores. At degree 8 that is 2.1 million ordered pairs out of 68.7 billion,
which is 0.003 percent of the possible pairs. Every other pair returns a
default.

A derived model answers every pair. There are 34.4 billion unordered pairs
at the character ceiling. Storing one byte for each pair costs 34.4 GB and
is impossible on any target.

**The vector buys complete pairwise coverage for 18.7 MB against a 34.4 GB
dense alternative.** Every stranger has an immediate, consistent, ordered
reaction to every other stranger, with no bookkeeping. That is the argument
to record.

Three secondary benefits follow and each is real.

- **No decay pass.** An edge model must walk 4.2 million edge endpoints to
  decay them. At about 2 nanoseconds for each endpoint that is 8.4
  core-milliseconds for each decay pass. A derived value has no decay pass.
- **No pruning policy.** An edge model must decide when a weak edge is
  deleted, and deletion order must be deterministic. A derived value has no
  such policy.
- **No death question.** An edge model must decide whether an edge survives
  the death of one endpoint. A derived value between a living character and
  a dead one is still well defined, provided the dead character keeps a
  vector.

---

## 4. What a derived value cannot represent

This section states the loss. The loss is real and it is the reason the
answer is "complement", not "replace".

### 4.1 Four properties a distance function cannot have

**Property 1. Asymmetry.** Manhattan distance is symmetric by definition.
`d(A, B)` equals `d(B, A)`. A grudge is not symmetric. Character A may hate
character B while character B has never heard of character A. No function of
the two vectors alone can express that, because the function receives the
same two arguments in both directions.

**Property 2. Specificity.** A derived value names no cause. "A dislikes B
because B killed A's brother in year 214" is a fact about one ordered pair
and one event. The vector holds no room for it, because a vector position
means the same thing for every entity.

**Property 3. Discontinuity.** A betrayal is a step change in one
relationship and no change in any other. Moving A's vector to express it
changes A's relation to **everyone**, which is wrong. This is the sharpest
failure and it is easy to demonstrate.

**Property 4. Commitment.** A promise, a debt, a marriage contract and an
oath are records with a counterparty and a term. They are not quantities.

### 4.2 The recommendation is a hybrid

**Keep the relation edges. Change what they are for.**

| Fact | Where it lives |
|---|---|
| "These two get on, in general" | Derived from vector distance. No storage. |
| "These two share a faith and a temperament" | Derived. No storage. |
| "A holds a grudge against B for a named deed" | A relation edge, asymmetric, with a cause identifier. |
| "A owes B a debt of 300" | A relation edge, or the ownership index. |
| "A swore an oath to B in year 214" | A relation edge with a start tick. |
| "A is the second cousin of B" | The kinship recursion, which is already exact.[^3] |

The edge becomes rarer, larger and more meaningful. Widen the edge record
from 8 bytes to 12 bytes to carry a cause identifier, and cut the mean
degree from 8 to about 3. The storage table of section 3.3 uses the 8-byte
figure and is therefore conservative by about 50 percent on the saving.

**The engine combines the two.** The opinion of A about B is the derived
baseline plus the sum of the stored overrides on the ordered pair `(A, B)`.
Both terms are integers, so the sum is exact and order-independent.

---

## 5. The k-nearest-neighbour problem

### 5.1 The cost of a global index over mutating points

Approximate nearest-neighbour structures assume a mostly static point set.
Hierarchical navigable small-world graphs build a layered proximity graph
whose edges encode the point positions, so a moved point invalidates its
edges.[^18] Inverted file indexes assign points to centroids, so a moved
point may change its cell. Locality-sensitive hashing assigns points to
buckets by a set of random projections, so a moved point may change its
bucket.[^19]

Every trait vector changes on every influence pass. The table gives the
rebuild cost at the character ceiling of 262,144 points at 12 dimensions.
All figures assume 3.5 GHz and an instructions-per-cycle rate of 2.5, which
matches the assumption the neighbouring reports use.[^20] One Manhattan
distance at 12 dimensions costs about 36 integer operations in a scalar
loop.

| Method | Work for one rebuild | Core-time | Fits a 40 ms tick |
|---|---|---|---|
| Exact all-pairs | 3.44e10 pairs x 36 ops | **141 core-s** | No, by 3,500 times |
| Hierarchical graph, M=16, ef=200 | 2.62e5 x 1,000 distances x 36 ops | **2.7 core-s** | No, by 67 times |
| Inverted file, 512 centroids, 10 rounds | 2.62e5 x 512 x 36 x 10 ops | **5.5 core-s** | No, by 137 times |
| Locality-sensitive hashing, 8 tables x 16 bits | 2.62e5 x 128 projections x 12 ops, plus a 2.1 M key sort | **86 core-ms** | Marginal |

**The honest conclusion is not "impossible".** Three of the four methods die
by two or more orders of magnitude. Locality-sensitive hashing survives. At
86 core-milliseconds for a rebuild, staggered over 32 ticks, it costs 2.7
core-milliseconds for each tick. That is about the same as the whole
movement subsystem.[^21] It is affordable and it is legal under the
determinism rules, because the sign of an integer dot product with a fixed
projection matrix is exact and reproducible.

**Refuse it anyway, and refuse it on the grounds of value, not cost.** A
global index answers "who in the world is most similar to me". No mechanic
in the design asks that question. Spending 2.7 core-milliseconds and a whole
index structure to answer an unasked question is the wrong trade. Section
5.4 gives the two behaviours that this refusal removes and gives a cheaper
answer for each.

### 5.2 The lead's resolution, verified

The proposed resolution is:

> Never do a global k-nearest-neighbour search. Take candidates from
> structure. Take weights from vector similarity.

**The resolution is correct.** It is also the more realistic model. The
literature supports the claim directly. Homophily operates on the ties a
person actually has, not on a global ranking of strangers.[^10] Centola and
colleagues show that a model in which agents interact only along existing
ties, and rewire them by similarity, is the version that keeps stable
cultural groups under drift.[^15] The structural version is therefore both
cheaper and better behaved than the global version.

The engine has two candidate sources and both are already sorted.

**Characters: the compressed sparse row graph.** The character report stores
the child list as a compressed sparse row structure and stores relation
edges at both endpoints.[^3] A character's candidate set is the union of its
parents, its children, its siblings, its liege, its vassals, its court and
its relation edges. Assume a mean of 16 candidates.

**Cohorts: the settlement index.** The cohort array indexes as
`pool * 8 + stratum`, so the 8 strata of one settlement are contiguous by
construction.[^2] A cohort's candidate set is the other 7 strata in its own
settlement, plus the same stratum in the 6 neighbouring settlements. That is
13 candidates, and 7 of them are in the same cache line group.

### 5.3 The cost of the structural form

**The character pass.** The pass is dominated by the gather, not by the
arithmetic. The character record array is 16.8 MB at the ceiling. That
exceeds the 1 MB private level 2 cache and competes for the shared level 3
cache. Take 40 cycles for a gather that misses level 2, and 1 cache line for
each candidate under the array-of-structs layout of section 9.

```
262,144 characters x 16 candidates x 40 cycles = 1.68e8 cycles
1.68e8 cycles / 3.5 GHz                        = 48 core-ms per pass
```

Order the character array by the father-tree Euler entry label, which the
character report already computes.[^3] A lineage then occupies a contiguous
run, so kinship candidates hit level 1 or level 2. Assume this halves the
miss count.

| Living characters | Cost for one full pass | Staggered over 256 ticks |
|---|---|---|
| 100,000 | 11 core-ms | 0.043 core-ms for each tick |
| 262,144 | 30 core-ms | **0.12 core-ms for each tick** |

The character report budgets the whole character tier in Rust at under 0.14
core-milliseconds for each tick.[^3] This pass roughly doubles that line, to
under 0.3 core-milliseconds. The running budget shows movement at 1.9 to 3.8
wall-milliseconds and a tick of 12 to 46 wall-milliseconds.[^21] The pass
fits with a wide margin.

**The cohort pass.** The cohort arrays are 80 KB for each dimension at
`i16`, so 12 dimensions are 960 KB and stay in the level 2 cache. Candidates
are contiguous. The pass vectorises 8 lanes wide.

```
40,000 cohorts x 13 candidates x 12 dimensions / 8 lanes x 2 ops
  = 1.56e6 operations = 0.18 core-ms per pass
```

At the economy period of 10 ticks that is **0.018 core-milliseconds for each
tick**.[^2] The cost is negligible.

**Total: about 0.14 core-milliseconds for each tick at the character
ceiling, and about 0.06 at 100,000 living characters.**

### 5.4 What the refusal loses

Two behaviours disappear. Each has a cheaper replacement.

**Loss 1. No alliance between distant strangers who happen to agree.** Two
lords on opposite sides of the map who share a temperament never find each
other, because no edge connects them. **Replacement: the diplomacy graph is
the candidate set for that question, and it is small.** A few hundred
institutions and a few thousand titled characters give a candidate set that
a full scan handles at no measurable cost.

**Loss 2. No spontaneous cross-map ideology.** A heresy or a political
movement cannot form among strangers who never meet. This is a genre-
relevant behaviour and the loss is real. **Replacement: make it a field, not
a search.** Seed one level 1 plane from every character above a threshold on
one dimension, diffuse it with the existing solver, and let characters read
the plane at their own cell. The field layer already runs nine planes for
0.32 to 0.71 core-milliseconds, and a separable economic plane costs 12
microseconds while a seeded Jacobi military plane costs 150
microseconds.[^22][^2] One ideology plane therefore costs between those two
figures. **This is the correct machinery and it is 20 times cheaper than the
locality-sensitive hashing index it replaces.**

**Loss 3, which is not a loss.** Marriage matching by compatibility does not
need a global search. The eligible set for a given character is already
small, because the kinship recursion runs for a few thousand pairs each
year.[^3] Score the eligible set by distance and take the best.

---

## 6. Dimensionality, type and dimension set

### 6.1 The real limit

Distance concentration is the phenomenon in which, as the dimension grows,
the distance to the nearest point and the distance to the farthest point
become nearly equal. Beyer, Goldstein, Ramakrishnan and Shaft prove the
general condition and show that under broad assumptions the ratio of the
farthest distance to the nearest distance converges to 1.[^23] Their
experiments show the effect appearing between 10 and 15 dimensions for
independent, identically distributed data.

Aggarwal, Hinneburg and Keim study which metric survives best. They show
that the `L1` norm preserves contrast better than the `L2` norm as the
dimension grows, and that fractional norms preserve it better still.[^24]
**This is an independent argument for Manhattan distance, and it agrees with
the arithmetic argument.**

Two facts raise the practical limit above the 10 to 15 figure.

- **The concentration proof assumes independent dimensions.** Personality
  and value axes correlate strongly. A population generated from correlated
  axes occupies a lower-dimensional surface, so the effective dimension is
  below the stored dimension.
- **The query is not a nearest-neighbour query.** Section 5 removes the
  global search. The remaining use is a **weight** over a candidate set of
  13 to 16. Ranking 16 candidates needs far less contrast than finding the
  single nearest of 262,144.

**Recommendation: `n = 12`.** Below 8 the model cannot carry both
personality and values. Above 16 the contrast falls without adding
behaviour. Twelve also gives a clean layout, which section 9 uses.

### 6.2 The per-dimension type

**Recommendation: `i16` in Q8.8, clamped to the range −16,384 to +16,383.**

| Property | Value | Reason |
|---|---|---|
| Width | 16 bits | 12 dimensions fit 24 bytes |
| Scale | Q8.8, so 256 steps for each unit | An `i8` gives too coarse a decay step |
| Clamp | −16,384 to +16,383, that is −64.0 to +63.996 | 12 x 32,767 = 393,204, which fits `i32` |
| Distance accumulator | `i32` | No overflow, no widening to `i64` |

Q8.8 differs from the Q16.16 scale that the record mandates for stats and
modifiers.[^1] **This is deliberate and it must be recorded, because a
silent scale mismatch is a defect class.** The reason for the narrower scale
is that the vector never enters the stat pipeline, which section 10
establishes. A conversion to Q16.16 is one left shift by 8 and it is exact.

An `i8` is tempting at 12 bytes for each vector. Reject it. Section 8.3
shows that an `i8` with a decay rate of 1/64 cannot represent the decrement
for any value below 64, so the decay stalls across most of the range.

### 6.3 Named dimensions, not learned embeddings

**Recommendation: 12 named dimensions, fixed at bake time.**

The argument for named dimensions has four parts and the fourth is decisive.

1. **Content authoring.** A designer must be able to write "this event
   raises ambition by 2.0". An opaque dimension makes that impossible.
2. **Debugging.** A tester must be able to read a vector and say why a
   character acted. An embedding gives twelve unlabelled numbers.
3. **Determinism and reproducibility.** A learned embedding comes from a
   floating-point training process. Baking it and quantising it is possible,
   but every content change then needs a retraining step, and the retraining
   must itself be reproducible to keep replays valid. That is a large new
   determinism surface for no gameplay gain.
4. **The reinforcement-learning audience prefers named dimensions too.** A
   named observation space is stable across engine versions. A learned
   embedding changes meaning whenever it is retrained, which invalidates
   every trained policy.

### 6.4 The proposed dimension set

Twelve dimensions in two groups of six. Group 1 is disposition, which is how
an entity acts. Group 2 is value, which is what an entity believes is right.
Each dimension is signed and each pole is named, so a designer never has to
ask which direction is positive.

| Index | Name | Negative pole | Positive pole |
|---|---|---|---|
| 0 | ambition | content | ambitious |
| 1 | wrath | calm | wrathful |
| 2 | boldness | cautious | reckless |
| 3 | honesty | deceitful | honest |
| 4 | compassion | cruel | merciful |
| 5 | greed | generous | avaricious |
| 6 | piety | worldly | devout |
| 7 | tradition | reforming | traditional |
| 8 | hierarchy | levelling | hierarchical |
| 9 | openness | insular | open to outsiders |
| 10 | martial | mercantile | martial |
| 11 | austerity | indulgent | austere |

**Do not add culture or faith as dimensions.** Both are already `u16`
categorical columns on the character row.[^3] The numeric distance between
culture 3 and culture 7 has no meaning, so putting a categorical identifier
into a metric space is a category error. Section 11 makes this a rule.

The set is not an arbitrary list. Indexes 0 to 5 map onto the
personality-trait vocabulary that character-driven strategy games already
use, and indexes 6 to 11 map onto the ideology axes that grand-strategy
games already use. **This report cites no game as an implementation source.
Every game claim in this project's citation record has proved to be
community documentation with no developer source.**[^21]

---

## 7. Scope: which tiers hold a vector

### 7.1 The rule

**Recommendation: characters and cohorts hold a vector. Units do not.**

The argument comes from the neighbouring reports and it is already settled
there.

**A unit does not decide.** The needs report states it directly: a unit's
upkeep is a fixed recipe, and its owner decides for it.[^2] An entity that
takes no decision has no use for a disposition.

**A unit's action selection uses different inputs.** The agency report gives
the mass-tier decision as an argmax over 4 to 8 level 1 field values,
weighted by unmet need and an occupation weight profile.[^25] Neither input
is a personality. Adding a personality term would add 12 gathers to a loop
that the report costs at 4.1 nanoseconds for each individual, which is the
whole reason the mass tier is affordable.

### 7.2 The cost if units did hold vectors

State the figure so the rejection is quantitative, not stylistic.

| Item | One million units |
|---|---|
| Storage, 48 bytes for each unit | 48 MB |
| Influence pass, 16 spatial candidates, cache-friendly at 8 cycles each | 37 core-ms for each full pass |
| Staggered over 32 ticks | **1.2 core-ms for each tick** |

The storage is affordable. The pass is not obviously affordable: 1.2
core-milliseconds for each tick is three times the whole entity-economy
subsystem, which the economy report costs at 0.4 to 0.6
wall-milliseconds.[^21] **The engine would pay three times its economy
budget for a quantity that no mass-tier mechanic reads.**

**Reject the vector for units.**

### 7.3 The one thing a unit still needs

A unit may need a morale or desertion threshold that depends on
disposition. Give it that without a per-unit vector.

**A unit reads two vectors that it does not own.** The first is the cohort
vector of the stratum and settlement it was recruited from. The second is
the vector of the character who commands its formation. Both are one gather
from a small array. The cost is 2 gathers rather than 12, and the storage is
zero.

**This is also the better model.** A soldier's willingness to fight depends
on where he came from and who leads him. It does not depend on a private
personality that nothing else in the simulation can observe.

---

## 8. Fixed point, determinism and decay

### 8.1 Manhattan distance is exact

Confirmed. The computation has three steps and none of them rounds.

```
diff  = (a[d] as i32) - (b[d] as i32)   // exact, no overflow at i16 inputs
term  = diff.abs()                       // exact
total = total + term                     // exact in i32, sum bounded by 393,204
```

There is no square root, no division and no reciprocal. The result is
identical on every core, in every thread count and on every target triple.

Euclidean distance would need either a square root or a squared accumulator.
The squared accumulator at 12 dimensions reaches 12 x 32,767 squared, which
is 1.29e10, so it needs `i64`. An integer square root is exact and
monotone.[^21] Euclidean distance is therefore legal but strictly worse
here: it needs a wider accumulator, it costs more instructions, and it
preserves less contrast.[^24]

### 8.2 Additive updates are order-independent, with one condition

Integer addition is commutative and associative, so a sum of deltas is
identical for any collection order. **Saturating addition is not
associative.** Clamping between two additions gives a different answer from
clamping after both.

**The rule: accumulate every delta for one pass in an `i32` accumulator,
apply the total once, then clamp once.**

```
// phase 1: read only. Accumulate. No clamp.
let mut acc: i32 = 0;
for each candidate c in candidate_set(e):       // ascending candidate id
    let w = weight_from_distance(v[e], v[c]);   // i32, non-negative
    acc += ((v[c][d] - v[e][d]) * w) >> 16;     // i32, exact
acc += ((anchor[e][d] - v[e][d]) * W_ANCHOR) >> 16;

// phase 2: write only. Apply once, clamp once.
v[e][d] = clamp(v[e][d] as i32 + acc, -16384, 16383) as i16;
```

The accumulator cannot overflow. Sixteen candidates, each contributing at
most 32,767, gives at most 524,272, which is far inside `i32`.

This mirrors the resolve rule that the economy report gives for capped
quantities, where a clamp after a reduce is wrong and the clamp position
must be a fixed schema property.[^4]

### 8.3 Decay: the error behaviour, which contains a defect

The owner's proposal says the relation modification decays. This is the
place where integer arithmetic bites, and the naive form has a sign-
dependent defect.

**The naive form.** Multiply by a fraction and shift right.

```
x = (x * 64512) >> 16;   // 64512/65536 = 0.984375, a decay of 1/64
```

In Rust the `>>` operator on a signed integer is an arithmetic shift, which
rounds toward negative infinity. The two signs behave differently.

| Start value | Behaviour under `(x * 64512) >> 16` |
|---|---|
| `x = +1` | `(64512) >> 16 = 0`. Reaches exactly zero. |
| `x = −1` | `(−64512) >> 16 = −1`. **Sticks at −1 forever.** |

**Both stated outcomes in the brief occur, and which one occurs depends on
the sign.** A positive value drifts to exactly zero. A negative value sticks
at a floor of −1 and never leaves it.

**Does it matter? Yes, for two reasons.** First, it introduces a permanent
systematic bias toward the negative pole of every dimension, which
accumulates over a 500-year run. Second, the state hash then differs between
a run that visited a negative value and one that did not, in a way that
looks like a determinism defect during debugging but is not one.

**The fix.** Use an explicit decrement with a ceiling, so every step moves
at least one unit toward zero and the value reaches exactly zero.

```rust
// Decays toward zero. Exact. Sign-symmetric. Terminates.
fn decay(x: i32, num: i32, den: i32) -> i32 {
    if x == 0 { return 0; }
    let mag  = x.unsigned_abs() as i64;
    let step = ((mag * num as i64) + den as i64 - 1) / den as i64;  // ceiling
    let step = step.max(1) as i32;                                   // never zero
    if x > 0 { (x - step).max(0) } else { (x + step).min(0) }
}
```

**The error behaviour, stated exactly.** The decay is geometric until the
computed step falls to 1, then it is linear. With a rate of 1/64 and a start
value of 16,384 the geometric part takes about 352 steps to reach 64, and
the linear part takes 64 more. **The value reaches exactly zero after about
416 steps and stays there.** There is no floor, no drift and no residual.

This also answers the `i8` question of section 6.2. With an `i8` clamped to
±127, a rate of 1/64 gives a computed step of 1 for every value below 64.
Half the range therefore decays linearly at one unit for each step, which is
a visible staircase. Q8.8 in `i16` removes it.

### 8.4 The determinism checklist

| Requirement | How this design satisfies it |
|---|---|
| No floating point | Every value is `i16` or `i32`. No division except the decay, which is integer. |
| Ordered iteration | The pass runs over entities in ascending index order. Candidates run in ascending candidate identifier order. |
| Stable keys | The entity index is the key. The character array is ordered by the Euler entry label, which is itself deterministic. |
| No thread-completion order | The accumulator for one entity is private to that entity. No cross-entity reduction occurs. |
| No hash iteration order | No hash map appears in the pass. Candidate sets come from compressed sparse row arrays and from the settlement index. |
| Read and write phase split | Phase 1 reads vectors and writes accumulators. Phase 2 reads accumulators and writes vectors. No entity reads an updated value in the same pass. |
| `bytemuck::Pod` | The record is `repr(C)`, all fields are fixed-width integers, and the padding is declared. |

**One further rule.** The perturbation applied at birth must be a keyed hash
of the tuple (system, frame, entity, draw), as the record requires.[^1] Do
not draw it from a sequence.

---

## 9. Memory layout on Graviton

### 9.1 The usual advice does not apply here

The usual advice is struct-of-arrays. An n-dimensional vector in
struct-of-arrays is `n` separate arrays, and a pass that touches one
dimension over all entities is then a perfect sequential scan. That advice
is correct for a dimension-wise map, such as the decay pass.

**It is wrong for this workload, and the factor is about 12.** The character
influence pass is a **random gather over graph neighbours**. Under
struct-of-arrays, reading all 12 dimensions of one random candidate touches
12 separate arrays at 12 separate addresses, which is 12 cache lines. Under
array-of-structs it touches 1.

| Layout | Lines for one random candidate | Cost at 262,144 x 16 candidates |
|---|---|---|
| Struct-of-arrays, 12 arrays | 12 | 359 core-ms for each pass |
| Array-of-structs, 64-byte record | 1 | **30 core-ms for each pass** |

### 9.2 The recommendation

**Characters: array-of-structs. One 64-byte record for each character.**

```rust
#[repr(C, align(64))]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct TraitRecord {
    current: [i16; 12],   // 24 bytes, Q8.8
    anchor:  [i16; 12],   // 24 bytes, Q8.8, immutable after birth
    scratch: [i16; 8],    // 16 bytes, declared padding and future use
}                         // 64 bytes exactly, one cache line
```

One record is one cache line. One gather fetches the current vector and the
anchor vector together, so the anchor term costs no extra miss. The record
never straddles a line, so no gather costs two misses.

**Cohorts: struct-of-arrays. Twelve `i16` arrays of 40,000 entries.**

The cohort access pattern is the opposite. Candidates are contiguous by
construction, because the array indexes as `pool * 8 + stratum`.[^2] The pass
is a dimension-wise sweep over a small, level-2-resident array. Each array is
80 KB and all twelve are 960 KB.

### 9.3 The NEON argument, which applies to cohorts

NEON registers are 128 bits, so one register holds 8 `i16` lanes. The
instruction set has a signed absolute-difference instruction and a signed
absolute-difference-and-accumulate-long instruction, which widens to `i32`
as it accumulates. Under struct-of-arrays a whole Manhattan distance for 8
entity pairs is therefore 12 instructions, which is **1.5 instructions for
each pair distance**.

This is the argument the usual advice rests on, and it is correct. It
applies to the cohort pass, where the candidates are contiguous. It does not
rescue the character pass, because there the gather dominates and no
instruction count can hide 12 cache misses.

**Record both layouts and record why they differ. A single rule for both
tiers is wrong.**

---

## 10. The conflict with the effective-stat table

This is the finding that limits the proposal, and it is the section to read
before any other.

### 10.1 What the economy report established

The economy report memoises effective stats on the key
`(unit_type, faction, upgrade_set)` and reports a hit rate of 99.5 to 99.98
percent across the design cases. It names two failure modes and both apply
to a per-entity vector.[^4]

- **Failure 1.** Per-entity state enters the pipeline as a stage input, so
  it selects **which** modifiers apply. This destroys sharing.
- **Failure 2.** Per-entity state enters as an unbounded continuous factor,
  so every entity has a distinct result.

The report's fix is a schema rule: a per-entity input may only enter as a
post-stage multiplier drawn from a small fixed table, and the count of such
multipliers is capped at 4.

### 10.2 The cost of breaking the rule

Let one continuous dimension enter the configuration key. Then the distinct
configuration count `K` rises from about 4,800 toward the entity count `N`,
because a Q8.8 dimension has 32,768 distinct values inside its clamp.

| Case | `K` | Hit rate | Pipeline cost |
|---|---|---|---|
| As designed | 4,800 | 99.5% | 0.05 core-ms, amortised |
| One continuous dimension in the key | 1,000,000 | **0%** | 1e6 x 32 fields x 15 ns = **480 core-ms** |

**That is a factor of about 9,600, and 480 core-milliseconds exceeds the
whole 12 to 46 wall-millisecond tick by an order of magnitude even at 12
cores.**[^21] It is not a slow path. It is a design that does not run.

**The verdict: a continuous per-entity vector is exactly the state that
defeats the memoisation, and the economy report's rule already forbids it.**

### 10.3 The resolution, and the slot arithmetic

**Recommendation: no vector dimension enters the modifier pipeline at all.**

The agency report already made this ruling for a neighbouring quantity. It
states that morale does not belong in the stat pipeline, because it gates
behaviour rather than scaling a rate, and so belongs in the threshold-
crossing machinery instead.[^25] **Every vector dimension is the same kind
of quantity.** Ambition decides whether a character rebels. It does not
scale his attack.

The slot budget confirms this is the only affordable answer. The agency
report resolves six candidates into the 4-slot cap by two merges, and the
result leaves one spare slot.[^25]

| Slot | Occupant | Source |
|---|---|---|
| 1 | condition tier, which merges health and fatigue | Agency report |
| 2 | proficiency tier, which merges veterancy and skill | Agency report |
| 3 | terrain factor | Economy report |
| 4 | **spare** | |

**At most one vector dimension could ever reach the stat pipeline, and only
by quantising into 8 bands and spending the last free slot.** Version 1
should spend zero. Record slot 4 as reserved and make the owner spend it
deliberately if a mechanic ever needs it.

### 10.4 Where the vector does its work instead

The vector must be able to change behaviour, or it is decoration. It does
so in four places and none of them touches the stat pipeline.

| Consumer | How the vector enters | Cost |
|---|---|---|
| Threshold crossings | A per-entity threshold offset, quantised into 8 bands, compared once for each tick | Shares the existing phase-A pass[^4] |
| Action selection | One extra weight term in the argmax, for characters only | Inside the 0.12 core-ms of section 5.3 |
| Derived opinion | A distance, computed on demand | Section 5.3 |
| Field seeding | A character above a dimension threshold seeds one level 1 plane | 12 to 150 microseconds[^22] |

**The first row is the important one.** A threshold offset is a compare, not
a multiply. It changes **whether** something happens, not **how much**. That
is exactly the kind of coupling the stat table does not care about.

---

## 11. The membership test for the vector

The field report gives a five-question test for whether a mechanic belongs
inside the field framework.[^22] This section gives the same kind of test
for the vector. The parallel is deliberate: a reader who knows one test
already knows how to use the other.

**An attribute belongs in the trait vector only if all four answers are
yes.**

**Question 1. Is it ordered?** Is "more of this" a meaningful statement?
Ambition is ordered. Culture identity is not, because culture 7 is not more
than culture 3.

**Question 2. Is it symmetric under distance?** If two entities hold the
same value, must they react to each other in the same way? Piety passes. A
grudge fails, because A may hate B while B has never met A.

**Question 3. Is the midpoint a real state?** Is the value halfway between
two values a state an entity could actually be in? Greed passes. "Holds the
duchy of Anvil" against "holds the duchy of Brand" fails, because there is
no halfway duchy.

**Question 4. Is it sourceless?** Can the entity hold this value without
remembering why? Wrath passes. "Owes 300 to character 7712" fails, because
the value is meaningless without the counterparty.

### 11.1 What falls outside, and where it goes

| Attribute | Fails which question | Correct home |
|---|---|---|
| Which title a character holds | 1 and 3 | The character row, `primary_title` |
| Whose child a character is | 1, 3 and 4 | The genealogy graph |
| Alive or dead | 1 and 3 | The `flags` byte and `death_tick` |
| Culture, faith | 1 | The categorical columns already on the row |
| A grudge against one named person | 2 and 4 | A relation edge with a cause identifier |
| A debt of a stated amount to one person | 2, 3 and 4 | A relation edge, or the ownership index |
| An oath sworn in a stated year | 2 and 4 | A relation edge with a start tick |
| Kinship to another character | 2 is arguable, 4 fails | The exact kinship recursion[^3] |
| Position on the map | 3 passes, 1 and 2 fail | The tile index column |
| Inventory contents | 1 and 4 | The inventory slots |
| Ambition, wrath, piety, greed | none fail | **The trait vector** |
| Loyalty to a specific institution | 2 and 4 | The `loyalty` scalar column[^2] |

**Note the last row, because it is easy to get wrong.** Loyalty is directed
at a named institution, so it fails question 4. It stays a scalar column on
the cohort row and on the character row, exactly as the needs report
specifies.[^2] The vector may **influence** loyalty as an input term. It
does not replace it.

---

## 12. The note for the reinforcement-learning audience

### 12.1 What the representation gives

An n-dimensional per-entity state vector is directly what a policy network
consumes. Three properties matter to that audience and this design has all
three.

**A fixed-width observation.** The vector is `n` values of a known type in a
known order, identical for every entity. No padding, no variable-length
sequence and no graph encoder is needed to read it.

**A stable observation space.** Named dimensions do not change meaning
between engine versions. A policy trained against version 1 still reads
version 2 correctly, provided the dimension list only grows at the end.
Section 6.3 gives this as the fourth argument for named dimensions.

**Both the state and the disposition.** The record holds the current vector
and the anchor vector. A policy therefore sees both who an entity has become
and who it started as. That difference is often the informative feature.

### 12.2 Zero-copy to NumPy

**The cohort tier is zero-copy. The character tier is not, and the copy is
cheap.**

**Cohorts, struct-of-arrays.** Each dimension is one contiguous `i16` array
of 40,000 entries. Expose the whole set as one `(12, 40000)` `int16` array
through the buffer protocol. That is a view with no copy and no allocation.
NumPy reads it directly. A policy that wants shape `(40000, 12)` takes the
transpose, which is also a view, though a strided one.

**Characters, array-of-structs.** The 64-byte record interleaves the current
vector and the anchor vector. Expose it as a structured `int16` array of
shape `(N, 32)`, which is a zero-copy view of the raw records. Slicing
columns 0 to 11 gives the current vectors as a strided view with no copy.
Any operation that needs a contiguous buffer pays one copy.

```
262,144 characters x 24 bytes = 6.29 MB
6.29 MB / 40 GB/s             = 0.16 milliseconds
```

**The copy costs about 0.16 milliseconds and it happens in Python, outside
the simulation step.** That is acceptable. It is far below the character
report's own Python budget of 1.7 milliseconds at 100,000 characters.[^3]

**One rule for that audience.** Do not convert to floating point in Rust.
The values are `i16` in Q8.8. Divide by 256.0 on the Python side. Python is
a control plane and is not simulated state, so a float there breaks
nothing.[^1]

---

## 13. Cost and storage summary

### 13.1 Cost for each tick

| Work | Scale | Core-ms for each tick |
|---|---|---|
| Character influence pass, staggered over 256 ticks | 262,144 x 16 candidates | 0.12 |
| Character influence pass, at 100,000 living | 100,000 x 16 candidates | 0.043 |
| Cohort influence pass, at economy period 10 | 40,000 x 13 candidates | 0.018 |
| Decay pass, folded into the influence pass | no extra traffic | 0 |
| Derived opinion queries, on demand | a few thousand each year | under 0.001 |
| One ideology field plane | 65,536 cells | 0.012 to 0.150 |
| **Total at the character ceiling** | | **0.15 to 0.29** |
| **Total at 100,000 living characters** | | **0.07 to 0.21** |

Compare that against the running budget. Movement costs 1.9 to 3.8
wall-milliseconds, trade costs 1.1 wall-milliseconds, the economy costs 0.4
to 0.6 wall-milliseconds, the whole field layer costs 0.32 to 0.71
core-milliseconds and the character tier in Rust costs under 0.14
core-milliseconds. The tick is 12 to 46 wall-milliseconds.[^21]

**This subsystem is about 0.6 percent of the mean tick at the character
ceiling.**

**Every figure in this report is derived and not measured.** The running
budget carries the same caveat and the research agenda flags benchmarking on
the target platform as blocking most conclusions.[^21]

### 13.2 Storage

| Item | Rule | 262,144 living |
|---|---|---|
| Character trait records | 64 bytes each | 16.8 MB |
| Cohort vectors and anchors | 48 bytes each, 40,000 rows | 1.9 MB |
| Relation edges saved, degree 8 to degree 3 | −40 bytes each | −21.0 MB |
| Ideology field plane | 65,536 cells x 4 bytes | 0.26 MB |
| **Net** | | **−2.0 MB** |

### 13.3 The kernel vocabulary

Every step is in the engine's kernel vocabulary. Nothing here needs a new
primitive.

| Step | Kernel |
|---|---|
| Read the candidate list from compressed sparse row | gather |
| Read the candidate vectors | gather |
| Compute distance and weight | map |
| Accumulate the delta for one entity | reduce, over a bounded local set |
| Write the updated vector | map |
| Seed the ideology plane from characters | scatter |
| Diffuse the ideology plane | stencil |
| Read the plane at an entity's cell | gather |
| Order the character array by Euler label | sort, at rebuild time only |

The cohort candidate set is a **local join** in the strict sense: both sides
are sorted by the same settlement key, so the join is a merge over
contiguous runs and needs no search.

---

## 14. Ready-to-apply decision block

**Status: recommended for adoption, with two hard limits.** The proposal is
adopted for the character tier and the cohort tier. It is **rejected for the
unit tier**. It is **forbidden from the modifier pipeline**. It **does not
replace** the relation edges; it narrows them.

### Part L — The entity trait vector

**D150. Adopt a per-entity trait vector for the character tier and the
cohort tier only.** The vector is the entity's disposition and values. About
302,000 entities hold one: up to 262,144 characters and 40,000 cohorts.

**D151. The vector has 12 named dimensions, fixed at bake time.** The
dimensions are ambition, wrath, boldness, honesty, compassion, greed, piety,
tradition, hierarchy, openness, martial and austerity, in that index order.
A dimension may be appended but never reordered and never removed, because a
reorder invalidates every replay and every trained policy. Learned
embeddings are rejected: a designer must be able to author "this event
raises ambition".

**D152. A dimension is `i16` in Q8.8, clamped to the range −16,384 to
+16,383.** This scale is deliberately narrower than the Q16.16 scale used
for stats and modifiers. The conversion to Q16.16 is one exact left shift by
8. A Manhattan distance over 12 dimensions is at most 393,204 and therefore
fits `i32` with no widening.

**D153. Each entity holds a second, immutable anchor vector, written once at
birth.** Every influence update includes a pull toward the anchor.

**D154. Distance is Manhattan, computed in `i32`.** Euclidean distance and
square roots are forbidden in this subsystem. Manhattan distance is exact,
needs no widening to `i64`, costs one vector instruction for each dimension
on the target, and preserves more contrast at 12 dimensions than Euclidean
distance does.

**D155. The character tier stores trait records as array-of-structs, in one
64-byte record for each character.** The record holds the current vector,
the anchor vector and 16 declared bytes of scratch. One record is one cache
line and never straddles two. **The cohort tier stores vectors as
struct-of-arrays, in 12 `i16` arrays of 40,000 entries.** The two layouts
differ because the access patterns differ: the character pass is a random
gather over graph neighbours and the cohort pass is a contiguous sweep.

**D156. The engine never builds a global nearest-neighbour index over trait
vectors.** Hierarchical navigable small-world graphs, inverted file indexes
and locality-sensitive hashing are all rejected. The reason is value, not
cost: no mechanic asks "who in the world is most similar to me".

**D157. Candidates come from structure. Weights come from similarity.** A
character's candidate set is its graph neighbourhood: parents, children,
siblings, liege, vassals, court and relation edges, taken from the existing
compressed sparse row arrays. A cohort's candidate set is the other 7 strata
of its own settlement plus the same stratum in the 6 neighbouring
settlements, taken from the existing settlement index. Similarity weights
the influence among those candidates and nothing else.

**D158. Order the character trait array by the father-tree Euler entry
label.** A lineage then occupies a contiguous run and kinship gathers hit the
level 1 or level 2 cache. Rebuild the ordering when the Euler labels are
rebuilt, not more often.

**D159. An influence pass accumulates every delta in an `i32` accumulator,
applies the total once, and clamps once.** Never clamp between two
additions. Saturating addition is not associative, so an intermediate clamp
makes the result depend on the collection order.

**D160. A decaying value decays by an explicit ceiling decrement toward
zero, never by an arithmetic shift.** An arithmetic shift right rounds toward
negative infinity, so a negative value sticks at −1 forever while a positive
value reaches exactly zero. The ceiling decrement is sign-symmetric, reaches
exactly zero in a bounded number of steps, and leaves no residual.

**D161. Units hold no trait vector.** A unit does not decide; its upkeep is a
fixed recipe and its owner decides for it. A per-unit vector would cost 48 MB
and about 1.2 core-milliseconds for each tick, which is three times the whole
entity-economy budget, for a quantity no mass-tier mechanic reads.

**D162. A unit that needs a disposition reads two vectors it does not own:
the cohort it was recruited from, and the character who commands its
formation.** That is 2 gathers and zero storage.

**D163. No trait dimension enters the effective-stat modifier pipeline.** Not
as a stage input, and not as a continuous post-stage multiplier. A continuous
per-entity value in the configuration key collapses the memoisation hit rate
from 99.5 percent to zero and raises the pipeline cost from 0.05
core-milliseconds to about 480 core-milliseconds, which exceeds the whole
tick budget by an order of magnitude.

**D164. Post-stage multiplier slot 4 is reserved.** Slots 1 to 3 hold the
condition tier, the proficiency tier and the terrain factor. Slot 4 is the
only slot a quantised trait dimension could ever occupy, and only as an
8-band index. **Version 1 spends zero slots.** Spending slot 4 is an explicit
owner decision, not an implementation choice.

**D165. A trait dimension changes behaviour through thresholds, action
selection, derived opinion and field seeding.** It gates whether something
happens. It never scales how much.

**D166. Opinion between two entities is derived by default and stored by
exception.** The derived baseline is a function of Manhattan distance. Stored
relation edges remain, for specific, asymmetric, event-caused facts only: a
grudge, a debt, an oath, a betrayal. Widen the edge record from 8 bytes to 12
to carry a cause identifier, and cut the mean degree from 8 to about 3. The
opinion of A about B is the derived baseline plus the sum of the stored
overrides on the ordered pair. Both terms are integers, so the sum is exact
and order-independent.

**D167. The vector membership test.** An attribute belongs in the trait
vector only if all four answers are yes. Is it ordered? Is it symmetric under
distance? Is its midpoint a real state? Is it sourceless? Identity, structure
and named counterparties fail at least one question and belong in the entity
store, the graph or an event.

**D168. Categorical identifiers are never vector dimensions.** Culture and
faith stay as `u16` columns. The numeric distance between two category
identifiers has no meaning.

**D169. A birth draws the child's vector from the two parents plus a keyed
perturbation.** The perturbation is a keyed hash of the tuple (system, frame,
entity, draw), as the record requires. The child's anchor vector equals its
birth vector and never changes again. A dead character keeps its current
vector and drops its anchor.

### Rules recorded but not numbered

- Cross-map ideological spread is a **field**, not a search. Seed one level 1
  plane from characters above a dimension threshold and diffuse it with the
  existing solver.
- Expose the cohort vectors to Python as a zero-copy `(12, 40000)` `int16`
  view. Expose the character records as a zero-copy `(N, 32)` `int16` view.
  Do not convert to floating point in Rust.
- The influence pass obeys the read and write phase split. Phase 1 reads
  vectors and writes accumulators. Phase 2 reads accumulators and writes
  vectors. No entity reads an updated vector inside the same pass.

---

## 15. Open questions

**OQ90. Confirm the 12 dimension names and their poles.** The list in this
report is a proposal. It is content, not engineering, and the owner should
set it before any event is authored. The engineering result does not change
if the names change.

**OQ91. What is the anchor pull weight?** The weight decides how far social
influence can move an entity from its innate disposition. A weight of zero
gives eventual consensus, which section 2 proves is a failure. A weight of
one gives a population that never changes. The recommendation is a starting
value near 1/8, tuned by measuring the spread of the population after 200
simulated years.

**OQ92. Does the anchor itself change over a lifetime?** A childhood
influence period, in which the anchor is still soft, is a plausible mechanic.
It costs one extra write. The literature gives no guidance, because the
Friedkin-Johnsen anchor is fixed by construction.

**OQ93. What is the mean relation-edge degree after the narrowing?** This
report assumes it falls from 8 to about 3. If it stays near 8, the net
storage change becomes positive rather than negative, and the storage
argument for the vector disappears entirely.

**OQ94. Does any mechanic need post-stage multiplier slot 4?** Leave it
empty in version 1. If a mechanic needs it, name the mechanic and the
dimension, and confirm the 8-band quantisation is acceptable in play.

**OQ95. Do cohort vectors and character vectors influence each other, and in
which direction?** A ruler shaping the values of his subjects is a real
mechanic. So is a population shaping a ruler. The engine can do both. Each
direction is one extra scatter or gather. The design has not chosen.

**OQ96. What is the character influence cadence in simulated time?** This
report assumes one full pass staggered over 256 ticks, which is 25.6 seconds
of wall time at 10 Hz. The correct figure depends on the tick-to-year ratio,
which the record has not fixed.

**OQ97. Do dead characters keep their trait vectors?** This report
recommends keeping the current vector and dropping the anchor, at 24 bytes
for each dead character. At the 500-year figure of 8.3 dead for each living,
that is 52 MB at the character ceiling. If it is not needed for biography or
for heir generation, drop both and save it all.

**OQ98. Does the ideology field need one plane for each dimension?** One
plane for each of 12 dimensions costs 3.1 MB and 12 times the solver cost.
One plane for a single named ideology is far cheaper. Start with one plane.

**OQ99. Measure the character influence pass on Graviton.** The 30
core-millisecond figure for a full pass rests on an assumed 40-cycle cost for
a gather that misses the level 2 cache, and on an assumed halving from the
Euler ordering. Both need measurement. Every other figure in this report is
smaller and less sensitive.

---

## 16. What this report changes in the neighbouring reports

State this plainly so the merge is mechanical.

**The character graph report.** Its relation edges survive, with a narrowed
purpose and a widened record. Its statement that a dead character drops its
relation edges still holds. Its Euler labels gain a second use, as the sort
key for the trait array. Its Rust budget line roughly doubles, from under
0.14 to under 0.30 core-milliseconds for each tick. Its 8-edge mean degree
assumption should fall to about 3.

**The needs and economy report.** Its cohort row gains a 12-dimension vector
and a 12-dimension anchor, held in separate struct-of-arrays columns rather
than inside the 56-byte row. Its loyalty scalar is **not** replaced; the
vector becomes one more input term in the loyalty update. Its unrest field
is unaffected.

**The entity economy report.** Nothing changes. Its 4-slot cap holds, its
memoisation key is unchanged, and this report adds a decision that protects
both.

**The individual agency report.** Its mass-tier decision loop is unchanged,
because units hold no vector. Its ruling that morale gates behaviour rather
than scaling a rate is extended to every trait dimension. Its resolution of
the 4-slot budget is adopted unchanged.

**The field operator algebra report.** It gains one consumer: an ideology
plane seeded from character vectors. Its five-question membership test gains
a companion four-question test for the vector.

---

## References

[^1]: ADR-0001, Foundational Architecture, and the project invariants. `docs/adrs/REGISTRY.md`
[^2]: Research report 15, Needs, Consumption and the Input-Output Economy. `docs/research/reports/15-needs-consumption-and-economy.md`
[^3]: Research report 14, Character Graph, Offices and Inheritance. `docs/research/reports/14-character-graph-and-inheritance.md`
[^4]: Research report 12, Entity Economy and Modifiers. `docs/research/reports/12-entity-economy-and-modifiers.md`
[^5]: DeGroot, M.H. "Reaching a Consensus". Journal of the American Statistical Association, volume 69, number 345, pages 118 to 121, 1974. https://doi.org/10.1080/01621459.1974.10480137
[^6]: Axelrod, R. "The Dissemination of Culture: A Model with Local Convergence and Global Polarization". Journal of Conflict Resolution, volume 41, number 2, pages 203 to 226, 1997. https://doi.org/10.1177/0022002797041002001
[^7]: Deffuant, G., Neau, D., Amblard, F., Weisbuch, G. "Mixing Beliefs among Interacting Agents". Advances in Complex Systems, volume 3, numbers 1 to 4, pages 87 to 98, 2000. https://doi.org/10.1142/S0219525900000078
[^8]: Hegselmann, R., Krause, U. "Opinion Dynamics and Bounded Confidence: Models, Analysis and Simulation". Journal of Artificial Societies and Social Simulation, volume 5, number 3, article 2, 2002. https://www.jasss.org/5/3/2.html
[^9]: Friedkin, N.E., Johnsen, E.C. "Social Influence and Opinions". Journal of Mathematical Sociology, volume 15, numbers 3 to 4, pages 193 to 205, 1990. https://doi.org/10.1080/0022250X.1990.9990069
[^10]: McPherson, M., Smith-Lovin, L., Cook, J.M. "Birds of a Feather: Homophily in Social Networks". Annual Review of Sociology, volume 27, pages 415 to 444, 2001. https://doi.org/10.1146/annurev.soc.27.1.415
[^11]: Castellano, C., Fortunato, S., Loreto, V. "Statistical Physics of Social Dynamics". Reviews of Modern Physics, volume 81, number 2, pages 591 to 646, 2009. https://doi.org/10.1103/RevModPhys.81.591
[^12]: Flache, A., Mäs, M., Feliciani, T., Chattoe-Brown, E., Deffuant, G., Huet, S., Lorenz, J. "Models of Social Influence: Towards the Next Frontiers". Journal of Artificial Societies and Social Simulation, volume 20, number 4, article 2, 2017. https://doi.org/10.18564/jasss.3521
[^13]: Klemm, K., Eguíluz, V.M., Toral, R., San Miguel, M. "Nonequilibrium Transitions in Complex Networks: A Model of Social Interaction". Physical Review E, volume 67, number 2, article 026120, 2003. https://doi.org/10.1103/PhysRevE.67.026120
[^14]: Klemm, K., Eguíluz, V.M., Toral, R., San Miguel, M. "Global Culture: A Noise-Induced Transition in Finite Systems". Physical Review E, volume 67, number 4, article 045101(R), 2003. https://doi.org/10.1103/PhysRevE.67.045101
[^15]: Centola, D., González-Avella, J.C., Eguíluz, V.M., San Miguel, M. "Homophily, Cultural Drift, and the Co-Evolution of Cultural Groups". Journal of Conflict Resolution, volume 51, number 6, pages 905 to 929, 2007. https://doi.org/10.1177/0022002707307632
[^16]: Mäs, M., Flache, A., Helbing, D. "Individualization as Driving Force of Clustering Phenomena in Humans". PLoS Computational Biology, volume 6, number 10, article e1000959, 2010. https://doi.org/10.1371/journal.pcbi.1000959
[^17]: Macy, M.W., Kitts, J.A., Flache, A., Benard, S. "Polarization in Dynamic Networks: A Hopfield Model of Emergent Structure". In Dynamic Social Network Modeling and Analysis, National Academies Press, Washington DC, pages 162 to 173, 2003. https://nap.nationalacademies.org/read/10735/chapter/11
[^18]: Malkov, Y.A., Yashunin, D.A. "Efficient and Robust Approximate Nearest Neighbor Search Using Hierarchical Navigable Small World Graphs". IEEE Transactions on Pattern Analysis and Machine Intelligence, volume 42, number 4, pages 824 to 836, 2020. https://doi.org/10.1109/TPAMI.2018.2889473
[^19]: Indyk, P., Motwani, R. "Approximate Nearest Neighbors: Towards Removing the Curse of Dimensionality". Proceedings of the 30th Annual ACM Symposium on Theory of Computing, pages 604 to 613, 1998. https://doi.org/10.1145/276698.276876
[^20]: Research report 16, Individual Agency and Occupations, section 3.4, for the clock and instructions-per-cycle assumptions. `docs/research/reports/16-individual-agency-and-occupations.md`
[^21]: Merge Notes for ADR-0001, sections 12 and 13, for the running per-tick budget and the game-citation finding. `docs/research/reports/MERGE-NOTES.md`
[^22]: Research report 13, Field Operator Algebra. `docs/research/reports/13-field-operator-algebra.md`
[^23]: Beyer, K., Goldstein, J., Ramakrishnan, R., Shaft, U. "When Is 'Nearest Neighbor' Meaningful?". Proceedings of the 7th International Conference on Database Theory, Lecture Notes in Computer Science volume 1540, pages 217 to 235, 1999. https://doi.org/10.1007/3-540-49257-7_15
[^24]: Aggarwal, C.C., Hinneburg, A., Keim, D.A. "On the Surprising Behavior of Distance Metrics in High Dimensional Space". Proceedings of the 8th International Conference on Database Theory, Lecture Notes in Computer Science volume 1973, pages 420 to 434, 2001. https://doi.org/10.1007/3-540-44503-X_27
[^25]: Research report 16, Individual Agency and Occupations, sections 2 and 3. `docs/research/reports/16-individual-agency-and-occupations.md`
