# 42. What a Policy Should Be Able to See

## Purpose and scope

This report states a complete observation and signal design for a
reinforcement learning policy that plays a deterministic hex-world strategy
simulation. It is a clean-sheet design. It does not describe what the engine
publishes today.

The report gives every signal group and the width of each group. It gives the
integer arithmetic that produces each value, and the bound on each value. It
also gives the spatial encoding and the rival encoding. It then gives the
normalisation rule, the reward design, three policy architectures, the
validation plan, the rejected alternatives, and the cost.

The total observation width is 2976 slots. Each slot is one signed 32-bit
integer in Q16.16 fixed point. One observation occupies 11904 bytes.

## 1. The domain this design serves

A reader needs the domain before the design. This section states it.

The world is a rhombus of hex tiles. The target scale is 16.7 million tiles
and one million units. Training runs today on worlds as small as 24 by 24
tiles. Each tile carries terrain, height, food, water, a value, and resources.
Some ground admits a unit. Water admits a unit only when the unit holds a
crossing capability.

A faction holds ground, units, settlements and stores. The seated faction
count varies from 2 to 16. A unit has a type, an individual address, and a set
of verbs: move, fight, gather, build and settle. A faction holds from zero to
hundreds of thousands of units.

A settlement holds population, stores of goods, and upgrades built on the
ground around it. An upgrade changes hands when the ground changes hands. A
reach rule limits how far a faction acts from its settlements. Reach grows
with the count of finished upgrades, not with population.

Trade runs through a public board. A faction posts a row that offers a good
and a quantity, and asks a good and a quantity. A contract binds and moves the
goods. Diplomacy is a relation value between each ordered pair of factions. A
character carries renown, and the highest live renown of a faction matters.

Fog of war governs knowledge. A faction sees what its units and holdings see
now. It remembers what it has ever seen. It knows nothing else. A typical mid-
game faction has explored under a fifth of the map.

Weather moves over the world. Storms, water and fire act on tiles and on
units. Four conditions end the game with a winner. They are domination, a
wonder that stands on held ground, renown, and the largest holding at a tick
limit.

## 2. The physics the design must respect

Five properties constrain every choice in this report. The project
instructions state the first four.[^1]

1. No floating point appears in simulated or aggregated state. The
   fixed-point scale is Q16.16. Every published value comes from integer
   arithmetic.
2. Every pass that builds the observation has an explicit and stable
   iteration order. The design never reads a hash order or a thread
   completion order.
3. The engine computes the observation. The Python control plane never loops
   over entities. It asks the engine for one array.
4. Cost follows what the faction has observed. A faction that has explored 3
   percent of a 16.7 million tile world does not pay for the other 97
   percent.
5. Evolution strategies train the policy today, with antithetic pairs, rank
   shaping, and a fixed-size normalised step.[^2] [^3] A gradient method may
   replace it. The design assumes neither.

## 3. The three failures the design removes

The design exists to prevent three specific failures.

The first failure is a width that follows the world. A field indexed by
faction multiplies by the faction count. A field indexed by lattice position
multiplies by the map size. A policy trained on one shape cannot run on
another. This design produces one fixed width for every map size and every
faction count.

The second failure is a raw count with an unbounded range. A held-tile count
of 300 means one thing on a 24 by 24 world. It means another thing on a 512 by
512 world. A fixed-point running total may be declared with the whole integer
range as its bound. Such an input spans twelve orders of magnitude. A fixed-
size step cannot serve such an input. This design publishes no raw count.
Section 8 states the rule.

The third failure is the absence of competitive information. A faction that
reads only its own economy and a terrain lattice cannot tell whether it leads.
It also cannot tell who leads, or by how much. It also cannot read quantities
that drive its own success, such as its count of finished upgrades. Sections 6
and 7 address this.

## 4. The three value kinds and their integer arithmetic

Every slot in the observation is one of three kinds. Each kind has a fixed
range and a named integer function.

A share lies in the closed interval from 0 to 1, held as 0 to 65536. A signed
relation lies in the closed interval from minus 1 to 1, held as minus 65536 to
65536. A compressed magnitude lies in the closed interval from 0 to 1. It
carries a sign when the underlying quantity is signed.

```rust
const ONE: i32 = 1 << 16;          // Q16.16 unit
const CAP_BITS: u32 = 40;          // structural cap on a compressed magnitude

/// A share. The denominator is named in the layout table for every slot.
fn share(n: i64, d: i64) -> i32 {
    let d = d.max(1);
    (((n.max(0) as i128) << 16) / d as i128).clamp(0, ONE as i128) as i32
}

/// A signed relation between two magnitudes.
fn relation(a: i64, b: i64) -> i32 {
    let scale = (a.abs() + b.abs()).max(1);
    ((((a - b) as i128) << 16) / scale as i128)
        .clamp(-(ONE as i128), ONE as i128) as i32
}

/// Base-two logarithm of x, in Q16.16, for x >= 1. Sixteen exact iterations.
fn ilog2_q16(x: u64) -> u32 {
    debug_assert!(x >= 1);
    let e = 63 - x.leading_zeros();
    let mut m = (x as u128) << (63 - e);   // m in [2^63, 2^64)
    let mut out = e << 16;
    let mut bit = 1u32 << 15;
    while bit != 0 {
        m = (m * m) >> 63;
        if m >= 1u128 << 64 {
            m >>= 1;
            out |= bit;
        }
        bit >>= 1;
    }
    out
}

/// A compressed magnitude. Monotone, sign preserving, and bounded.
fn magnitude(v: i64) -> i32 {
    let a = ilog2_q16(1 + v.unsigned_abs());
    let m = ((a as i64) / CAP_BITS as i64).min(ONE as i64) as i32;
    if v < 0 { -m } else { m }
}

/// An integer triangle wave, for a cyclic phase. Range minus ONE to ONE.
fn triangle(p: i64, period: i64) -> i32 {
    let period = period.max(1);
    let u = (p.rem_euclid(period) * 4 * ONE as i64) / period;   // [0, 4*ONE)
    let f = if u < 2 * ONE as i64 { u } else { 4 * ONE as i64 - u };
    (f - ONE as i64) as i32
}
```

The compressed magnitude is an integer form of a symmetric logarithm. A world
model that trains across domains of different scale uses the same device on
its inputs. Its authors report that a static compression removes the need for
a learned per-dimension scale.[^4] The cap of 40 bits is a structural
constant, not a budget. It admits any quantity below 1.1 times 10 to the
twelfth. The largest quantity in this design is a Q16.16 store total summed
over one million units, and it fits.

Every division truncates toward zero. Truncation is the documented rounding
rule for the whole design. No slot depends on a rounding mode.

## 5. Spatial awareness

### 5.1 The encoding

The design samples space with an egocentric multi-resolution ring stack. The
frame has a centre, a set of rings at geometric radii, and a set of angular
sectors in each ring. Resolution falls with distance from the centre.

The centre is the integer centroid of the tiles the faction holds. The engine
sums the axial coordinates of held tiles as i64 values. It divides each sum by
the held count, with truncation. When the faction holds no tile, the centre is
the centroid of its units. When it holds neither, the centre is the world
centre.

The ring index comes from the hex distance from the centre. Hex distance on an
axial grid is the standard cube distance.[^5] Let `d` be that distance. The
ring index is `min(7, bit_length(d))`, where `bit_length(0)` is 0. The table
gives the distance band and the cell count of each ring.

| Ring | Hex distance | Cells |
|------|--------------|-------|
| 0 | 0 | 1 |
| 1 | 1 | 6 |
| 2 | 2 to 3 | 12 |
| 3 | 4 to 7 | 12 |
| 4 | 8 to 15 | 12 |
| 5 | 16 to 31 | 12 |
| 6 | 32 to 63 | 12 |
| 7 | 64 and beyond | 12 |

The sector count varies with the ring, because a near ring holds few tiles. A
hex ring at distance `d` holds `6d` tiles. Ring 0 holds one tile and has no
direction, so it holds one cell. Ring 1 holds six tiles, so it holds six
cells. Rings 2 to 7 hold twelve cells each. The stack therefore holds 79
cells, as the table above states.

The sector frame is anchored to the world axes, not to a rotating frame.
Sector 0 points along the positive `q` axial direction. The frame is
translation invariant and is not rotation invariant. This is the correct
choice, because the action space names the six hex directions in the world
frame. A rotating frame would make the meaning of a sector drift between
decisions.

### 5.2 How the engine computes a sector without trigonometry

The engine converts the axial delta to cube coordinates `(x, y, z)`. It sets
`x = dq`, then `z = dr`, then `y = -x - z`. The three coordinates sum to zero,
so exactly one has a sign opposite to the other two, or one is zero. Six sign
patterns partition the plane into six sextants. The engine selects the sextant
by comparing signs, which needs no division.

Inside a sextant, let `a` and `b` be the magnitudes of the two coordinates
that share a sign. The engine splits the sextant in two by testing `2 * a >= a
+ b`. The result is one of twelve sectors, from integer comparisons only. For
a ring with six cells the engine drops the sub-split.

### 5.3 The channels

Each cell holds 25 channels. The engine accumulates i64 sums over the tiles
that fall in the cell. For a far ring it sums over the summary cells that fall
in the cell instead. The engine then divides each sum by the cell count, with
truncation.

| Channel | Signal | Kind | Denominator |
|---------|--------|------|-------------|
| 1 | share of the cell area that lies inside the world | share | nominal cell area |
| 2 | share of cell tiles the faction has ever observed | share | in-world cell tiles |
| 3 | share of cell tiles the faction sees now | share | in-world cell tiles |
| 4 | mean ticks since the faction last saw a tile | magnitude | cap |
| 5 | share of observed tiles that admit a unit without a crossing | share | observed cell tiles |
| 6 | share of observed tiles that are water | share | observed cell tiles |
| 7 | mean height | share | world height range |
| 8 | mean absolute height difference from the cell mean | share | world height range |
| 9 | mean tile food | share | maximum tile food |
| 10 | mean tile water | share | maximum tile water |
| 11 | mean tile value | share | maximum tile value |
| 12 | share of observed tiles carrying a resource | share | observed cell tiles |
| 13 | share of observed tiles the faction holds | share | observed cell tiles |
| 14 | share of observed tiles a rival holds | share | observed cell tiles |
| 15 | share of observed passable tiles nobody holds | share | observed passable tiles |
| 16 | own units per observed tile | magnitude | cap |
| 17 | remembered rival units per observed tile | magnitude | cap |
| 18 | own settlement count | magnitude | cap |
| 19 | remembered rival settlement count | magnitude | cap |
| 20 | own finished upgrade count | magnitude | cap |
| 21 | remembered rival finished upgrade count | magnitude | cap |
| 22 | share of cell tiles inside own reach | share | in-world cell tiles |
| 23 | share of observed tiles under a weather hazard | share | observed cell tiles |
| 24 | own military strength in the cell | magnitude | cap |
| 25 | remembered rival military strength in the cell | magnitude | cap |

Channel 1 is the channel that lets one layout serve every world size. On a 24
by 24 world, rings 5 to 7 lie outside the world, and channel 1 reads 0 there.
On a 4096 by 4096 world the same rings read a positive share. The policy reads
the same shape and learns that channel 1 gates the rest.

Channel 4 is the channel that separates knowledge from memory. A remembered
tile from 500 ticks ago is not current knowledge, and the policy must be able
to tell the difference.

The stack holds 25 times 79 slots, which is 1975 slots.

### 5.4 How the engine builds the stack, and what it costs

Rings 0 to 3 cover hex distance below 8. That region holds 169 tiles. The
engine reads level 0 tiles there, masked by the faction's current fog state.

Rings 4 to 7 read the summary level of the detail pyramid. The pyramid
summarises blocks of tiles at city scale, and it holds exact integer
accumulators. The engine iterates the faction's remembered summary cells in
ascending cell index. For each cell it computes the ring and the sector from
the cell centre, then accumulates.

The shared pyramid holds the truth, not what a faction saw. The design
therefore places one requirement on the engine: each faction keeps a
remembered summary record per observed summary cell. That record holds only
the fog-dependent channels, which are channels 2, 3, 4, 13, 14, 17, 19, 21 and
25. The engine reads the other sixteen channels from the shared pyramid and
masks them with the faction's observed bitset. Terrain, height, food, water
and value do not change with the observer.

The build cost is the count of observed summary cells plus 169. Take a 16.7
million tile world with a 256-tile summary block and 3 percent explored. That
gives about 1960 summary cells. The pass performs about 2100 accumulate steps,
each a small number of i64 additions. The cost follows observed area, and it
does not follow world area.

The pass parallelises by sharding the summary cell index range. Each shard
accumulates into its own i64 array. The engine combines the shards in
ascending shard index. Integer addition is associative, so the combination is
exact and the order is stated.

## 6. Competitive awareness

### 6.1 The problem

A faction must read its rivals. A block indexed by seat makes the width follow
the faction count, and it teaches the policy a seat number. The design uses
two mechanisms instead, and neither depends on the faction count.

### 6.2 Twelve power quantities

The design names twelve quantities that measure the standing of a faction. The
faction estimates each one for each rival from what it has observed.

1. Held tile count.
2. Settlement count.
3. Population total.
4. Unit count.
5. Military strength.
6. Finished upgrade count.
7. Highest live renown.
8. Wonder progress.
9. Total store value.
10. Held tiles gained over the window.
11. Reach area.
12. Trade volume over the window.

### 6.3 Seven order statistics per quantity

For each quantity the engine publishes seven values. Let `v_self` be the own
value. Let `v_i` be the estimate for rival `i`. Let `T` be the sum of the
quantity over all seated factions, including the self.

| Statistic | Kind | Formula |
|-----------|------|---------|
| own share of the total | share | `share(v_self, T)` |
| own rank as a share | share | `share(count(v_i < v_self), rival_count)` |
| own gap to the strongest rival | signed relation | `relation(v_self, max(v_i))` |
| own gap to the median rival | signed relation | `relation(v_self, median(v_i))` |
| the leader share of the total | share | `share(max over all, T)` |
| concentration | share | `sum(share(v_k, T)^2) >> 16` |
| observation confidence | share | observed part over estimated part |

The concentration statistic is a sum of squared shares. It tells the policy
whether one faction runs away or the field is level. Two factions at equal
strength give 0.5. Sixteen equal factions give 0.0625. One dominant faction
approaches 1.

The observation confidence statistic tells the policy how much of an estimate
it actually saw. Without it a policy treats an inferred quantity and a seen
quantity as the same fact. It then cannot learn to scout.

The block holds 7 times 12 slots, which is 84 slots. The width does not depend
on the faction count. The engine iterates seated factions in ascending seat
index; the statistics are order independent, and the order is stated for
determinism.

### 6.4 Per-rival tokens

Order statistics lose the joint structure. A policy cannot tell from them
whether the strongest rival is also the nearest rival. The design therefore
also publishes six rival tokens, each of 24 channels.

The engine selects the six rivals by descending threat. Threat is the
remembered rival military strength inside the faction's reach, plus the
remembered strength adjacent to its border. It breaks a tie by descending held
tile estimate, then by descending population estimate, then by ascending seat
index. The last tie-break exists only to fix the byte order.

A token holds 24 channels. The table gives them.

| Channels | Signal | Kind |
|----------|--------|------|
| 1 | validity flag | share |
| 12 | relative ratio against the own value, one per power quantity | signed relation |
| 1 | relation the faction holds to the rival | signed relation |
| 1 | relation the rival holds to the faction | signed relation |
| 1 | war flag | share |
| 1 | shared border length over the own border length | share |
| 1 | distance to the nearest known rival settlement | share |
| 1 | trend of that distance over the window | signed relation |
| 1 | trade volume with the rival over the own volume | share |
| 1 | trend of the rival power share | signed relation |
| 1 | observation confidence | share |
| 1 | distance between the rival unit type mix and the own mix | share |
| 1 | relation the rival holds to the current leader | signed relation |

A token carries no seat number. The design requires the policy to consume the
token set through a permutation-invariant encoder. Section 10 names the
encoder. With such an encoder the sort order does not change the output. The
sort exists only to make the published bytes deterministic.

Six is the recommended count. A faction that ranks below sixth on threat does
not drive a decision, so six covers a game of 16 seated factions. The order
statistics in section 6.3 cover the whole field, so nothing is lost.

### 6.5 What each mechanism loses

Order statistics lose identity, joint structure, and position. They keep the
whole field at constant width and constant cost. A quantile vector loses less
shape than a mean and a maximum. It still loses identity, and it needs a sort
over factions. A set encoder over all seated factions keeps everything, and
the mask handles the variable token count. It makes the policy spend capacity
on weak rivals.

The recommendation is the pair. Order statistics over all rivals give the
field. A masked token set over the six strongest gives the detail. Strong
agents in large strategy domains use that combination. One agent for a real-
time strategy game encodes each entity as a token and attends over the
set.[^6] Another encodes each unit and pools over the set with a maximum.[^7]
A maximum is a permutation-invariant reduction.

## 7. Signals the domain has and a naive design omits

This section names the signals a strong player needs that a simple self-
economy observation does not carry. Each entry states why a decision needs it.

Finished upgrade count and reach headroom. Reach grows with finished upgrades,
and reach limits every action at distance. A policy that cannot see its
upgrade count cannot learn that building an upgrade unlocks a move.

Distance from each settlement to the reach boundary. This is the quantity that
says whether a settlement can still project force outward.

Frontier quality by direction. A faction expands into unclaimed reachable
ground. A scalar count of unclaimed tiles does not say where. Section 9 gives
the per-sector form.

The gap to the leader on each of the four victory tracks. A faction that leads
on ground and trails on renown must play differently from a faction in the
reverse position. Without the per-track gap the policy cannot choose a track.

Estimated ticks to completion, for the self and for the leader, on each track.
A gap of equal size means different things at tick 100 and at tick 9000.

Coalition structure. The engine computes the mean pairwise relation among the
rivals, with the self excluded. That value tells the faction whether the other
factions are aligned against it. A faction that reads only its own relations
cannot see it.

Trade prices, depth and price trend. The board is public, so no fog applies. A
faction that cannot read a price cannot learn to trade.

Staleness of memory. Section 5.3 states the channel. A policy needs to know
that its map is old.

Observation confidence per rival quantity. Section 6.3 states it. It is the
signal that makes scouting learnable.

Hazard exposure and its trend on own ground. Fire burns tiles and units. The
policy therefore needs to see a hazard before it arrives.

Food coverage in ticks, not in units. A store of 1000 food means nothing until
the policy divides by consumption. The engine performs that division, because
it is a fixed integer operation and the policy should not spend capacity on
it.

Contested border length, and where the border is contested. A border share is
a different decision from a border length.

Momentum. The first difference of every share over a fixed window. A level and
a rate drive different decisions. A faction at 20 percent of the world and
rising plays differently from one at 20 percent and falling.

Affordance. Whether an action is legal and affordable right now. Without it
the policy learns the rules from the reward signal. That is the slowest way to
learn a rule the engine already holds.

Unit type mix, own and rival. A counter-composition decision needs both.

Centroid offset from the world centre, and centroid movement over the window.
This tells the policy whether it sits in a corner and whether it is being
pushed.

## 8. The normalisation rule

### 8.1 The rule

Every published value is a share, a signed relation, or a compressed
magnitude. The design publishes no raw count and no unbounded fixed-point
total.

A share divides by a denominator that both the engine and the reader can name.
The layout table names the denominator for every share slot. A signed relation
divides a difference by the sum of the two magnitudes, which bounds it without
a chosen denominator. A compressed magnitude maps the quantity through the
integer base-two logarithm of section 4 against a fixed 40-bit cap.

The design uses the denominators in the list below, and no others. Each is a
structural property of the world, or a total the engine already maintains.

- The world tile count.
- The world passable tile count.
- The world height range.
- The world hex radius.
- The maximum tile food, water and value.
- The tick limit.
- The seated faction count, and the rival count.
- The sum of a power quantity over all seated factions.
- The own total for the quantity in question.
- The own border length.
- The observed tile count, and the observed passable tile count.
- The nominal cell area of a ring-sector cell.

### 8.2 Why a running normaliser is rejected

A learned running mean and variance, or an adaptive output normaliser, gives
the same range control.[^8] [^9] Two properties make it wrong here. It holds
float state, which the physics forbids. It also makes the mapping depend on
the data seen so far. The same world state then produces different inputs in
two runs, and the determinism test fails. The static compressed magnitude
gives bounded range with no state and no float.

A per-episode minimum and maximum normalisation has the same defect for the
same reason. The denominator depends on the episode.

### 8.3 What normalisation loses, and how the design gives it back

A share hides absolute scale. Two factions each hold half of a world. The
first world is 24 by 24 and the second is 512 by 512. Both read the same
share, and the second faction can field a hundred times the army. The loss is
real. It matters whenever the correct action depends on the absolute quantity,
not on the relative one.

The design gives the scale back in three ways.

First, it publishes a compressed magnitude beside the share. It does this for
the eight quantities where absolute scale changes the correct action. Those
quantities are held tiles, units, military strength, population, settlements,
finished upgrades, reach radius, and the stock of each good class.

Second, it publishes three explicit world-scale scalars. They are the
compressed magnitude of the world tile count, the compressed magnitude of the
world passable tile count, and the seated faction count as a share of 16. A
policy that holds both a share and the scale can recover the absolute
quantity. A two-layer network can perform that multiplication.

Third, it publishes the tick limit and the tick share. The policy then reads
its position in the episode, not an absolute tick count.

## 9. The full layout

The observation holds twelve blocks. The table gives the width of each. Every
slot is an i32 in Q16.16.

| Block | Contents | Slots |
|-------|----------|-------|
| A | Self state, non-spatial | 96 |
| B | Clock and phase | 8 |
| C | Victory race | 24 |
| D | Rival order statistics | 84 |
| E | Egocentric ring stack | 1975 |
| F | Frontier and pressure by sector | 32 |
| G | Trade board summary | 40 |
| H | Diplomacy summary | 16 |
| I | Entity tokens | 624 |
| J | Weather and hazard | 12 |
| K | Affordance | 24 |
| L | Objective weights and reserve | 41 |
| | Total | 2976 |

The width formula is `1001 + C * N`. Here `C` is the channel count of the ring
stack, and `N` is its cell count. With `C` equal to 25 and `N` equal to 79 the
total is 2976. Both are design knobs, and changing either changes the width,
so a change to either invalidates a trained policy.

The total is 2976 slots, which is 11904 bytes, which is 186 cache lines of 64
bytes on the target platform.

### 9.1 Block A: self state, 96 slots

| Slots | Signal | Kind | Denominator |
|-------|--------|------|-------------|
| 1 | held tile count | magnitude | cap |
| 1 | held tiles over observed passable tiles | share | observed passable tiles |
| 1 | held tiles over world passable tiles | share | world passable tiles |
| 1 | unit count | magnitude | cap |
| 1 | units per held tile | share | held tiles times 8 |
| 1 | settlement count | magnitude | cap |
| 1 | population total | magnitude | cap |
| 1 | population per settlement | magnitude | cap |
| 1 | finished upgrade count | magnitude | cap |
| 1 | upgrades under construction | magnitude | cap |
| 1 | finished upgrades per settlement | magnitude | cap |
| 1 | reach radius in tiles | magnitude | cap |
| 1 | reach radius over the world hex radius | share | world hex radius |
| 1 | reach area over world passable area | share | world passable tiles |
| 1 | held tiles inside reach over held tiles | share | held tiles |
| 8 | stock of each good class | magnitude | cap |
| 8 | stock share of each good class | share | own total stock |
| 8 | net flow of each good class | signed relation | gross throughput |
| 8 | production rate of each good class | magnitude | cap |
| 8 | consumption rate of each good class | magnitude | cap |
| 1 | total store value | magnitude | cap |
| 1 | total military strength | magnitude | cap |
| 1 | military strength per unit | magnitude | cap |
| 8 | unit share of each type class | share | own unit count |
| 1 | idle unit share | share | own unit count |
| 1 | share of units inside reach | share | own unit count |
| 1 | share of units adjacent to a rival unit | share | own unit count |
| 1 | food coverage in ticks | magnitude | cap |
| 1 | population change over the window | signed relation | population |
| 1 | held tile change over the window | signed relation | held tiles |
| 1 | unit count change over the window | signed relation | unit count |
| 1 | military strength change over the window | signed relation | strength |
| 1 | store value change over the window | signed relation | store value |
| 1 | upgrade completions over the window | magnitude | cap |
| 1 | tiles taken from rivals over the window | magnitude | cap |
| 1 | tiles lost to rivals over the window | magnitude | cap |
| 1 | settlements lost over the window | magnitude | cap |
| 1 | highest live renown | magnitude | cap |
| 1 | highest live renown share | share | total live renown |
| 1 | live character count | magnitude | cap |
| 1 | observed tiles over world tiles | share | world tiles |
| 1 | observed passable over observed tiles | share | observed tiles |
| 1 | newly observed tiles over the window | magnitude | cap |
| 1 | mean staleness of remembered tiles | magnitude | cap |
| 1 | share of held tiles under a hazard | share | held tiles |
| 1 | share of settlements under a hazard | share | settlements |
| 1 | world tile count | magnitude | cap |
| 1 | world passable tile count | magnitude | cap |
| 1 | seated faction count over 16 | share | 16 |
| 1 | centroid offset from the world centre | share | world hex radius |
| 1 | centroid movement over the window | share | world hex radius |
| 1 | own border length | magnitude | cap |
| 1 | contested border share | share | own border length |
| 1 | own wonder progress | share | wonder requirement |

The slot counts sum to 96.

The goods taxonomy holds eight classes, and the unit type taxonomy holds eight
classes. The engine maps every good and every unit type to a class. Both
taxonomies are fixed, so a new good or a new unit type does not change the
width.

### 9.2 Block B: clock and phase, 8 slots

| Slots | Signal | Kind | Denominator |
|-------|--------|------|-------------|
| 1 | tick over the tick limit | share | tick limit |
| 1 | remaining ticks | magnitude | cap |
| 1 | decisions taken | magnitude | cap |
| 1 | window length in ticks | magnitude | cap |
| 2 | weather phase, as a triangle wave and a quarter-shifted triangle wave | signed relation | period |
| 2 | long world cycle phase, in the same form | signed relation | period |

The triangle pair replaces a sine and cosine pair. A single phase value
carries a discontinuity at the wrap point. A network reads that discontinuity
as a large change in the world. A pair of quadrature waves removes it, and the
integer triangle of section 4 is exact.

### 9.3 Block C: victory race, 24 slots

The block holds four tracks: domination, wonder, renown, and held ground at
the tick limit. Each track holds six slots.

| Slots | Signal | Kind | Denominator |
|-------|--------|------|-------------|
| 1 | own progress on the track | share | condition requirement |
| 1 | leader progress on the track | share | condition requirement |
| 1 | own gap to the leader | signed relation | sum of magnitudes |
| 1 | own rank on the track | share | rival count |
| 1 | own estimated ticks to completion | magnitude | cap |
| 1 | leader estimated ticks to completion | magnitude | cap |

The engine derives an estimated tick count from the current rate over the
window. When the rate is zero or negative, the engine publishes 0. A reader
takes 0 as unreachable at the current rate.

### 9.4 Block D: rival order statistics, 84 slots

Section 6.3 states the block. It holds twelve quantities and seven statistics
for each.

### 9.5 Block E: the ring stack, 1975 slots

Section 5 states the block. It holds 25 channels and 79 cells.

The published array holds 79 distinct cells with no duplication. A policy that
needs a rectangular tensor reshapes to 8 rings by 12 sectors. It replicates
the single ring 0 cell twelve times, and it replicates each ring 1 cell twice.
The reshape happens in the policy, not in the published array.

### 9.6 Block F: frontier and pressure by sector, 32 slots

| Slots | Signal | Kind | Denominator |
|-------|--------|------|-------------|
| 12 | expansion score by sector | share | largest sector score |
| 12 | threat pressure by sector | share | largest sector pressure |
| 1 | frontier length over the own border length | share | own border length |
| 1 | mean food of frontier-adjacent unclaimed land | share | maximum tile food |
| 1 | mean value of frontier-adjacent unclaimed land | share | maximum tile value |
| 1 | unclaimed reachable tile count | magnitude | cap |
| 1 | best site score | share | structural maximum score |
| 1 | available site count | magnitude | cap |
| 1 | reach headroom in tiles | magnitude | cap |
| 1 | share of the frontier that borders a rival | share | frontier length |

The engine computes a site score for each unclaimed observed passable tile
inside or adjacent to reach. It sums the scores by sector as i64 values. It
then divides each sector sum by the largest sector sum, so the largest sector
reads 1.

The engine computes threat pressure the same way, from remembered rival
military strength weighted by the inverse of the hex distance.

The sector axis of this block matches the sector axis of the ring stack. A
policy can therefore relate a direction in one block to the same direction in
the other.

### 9.7 Block G: trade board summary, 40 slots

The board is public, so no fog applies to it. The engine publishes five slots
for each of the eight good classes.

| Slots | Signal | Kind | Denominator |
|-------|--------|------|-------------|
| 1 | best price to buy | magnitude | cap |
| 1 | best price to sell | magnitude | cap |
| 1 | spread between the two | signed relation | sum of magnitudes |
| 1 | depth offered on the board | magnitude | cap |
| 1 | price change over the window | signed relation | price |

A price is a ratio of two integer quantities. The engine holds it as a Q16.16
value produced by the share function against a fixed numeraire quantity. It
then compresses that value with the magnitude function.

The engine iterates board rows in ascending row identity. The cost is the row
count. The row count does not follow the world size.

### 9.8 Block H: diplomacy summary, 16 slots

| Slots | Signal | Kind |
|-------|--------|------|
| 1 | mean relation the faction holds to its rivals | signed relation |
| 1 | minimum relation the faction holds | signed relation |
| 1 | maximum relation the faction holds | signed relation |
| 1 | mean relation the rivals hold to the faction | signed relation |
| 1 | minimum relation the rivals hold to the faction | signed relation |
| 1 | mean asymmetry between the two directions | signed relation |
| 1 | share of rivals at war with the faction | share |
| 1 | share of rivals at peace with the faction | share |
| 1 | share of rivals above an alliance threshold | share |
| 1 | mean pairwise relation among the rivals, self excluded | signed relation |
| 1 | share of rival pairs at war with each other | share |
| 1 | relation the faction holds to the leader | signed relation |
| 1 | relation the leader holds to the faction | signed relation |
| 1 | mean relation change over the window | signed relation |
| 1 | share of rivals whose relation to the faction fell | share |
| 1 | share of rivals holding a live contract with the faction | share |

No slot in this block names a seat.

### 9.9 Block I: entity tokens, 624 slots

The block holds four token sets. Each set has a fixed token count and a
validity channel. A missing token holds zero in every channel, including the
validity channel.

| Set | Tokens | Channels | Slots |
|-----|--------|----------|-------|
| Own settlements | 8 | 24 | 192 |
| Rivals | 6 | 24 | 144 |
| Threat clusters | 8 | 20 | 160 |
| Candidate sites | 8 | 16 | 128 |
| | | | 624 |

Section 6.4 gives the rival token channels. The three tables below give the
other three sets.

A settlement token holds 24 channels.

| Channels | Signal | Kind |
|----------|--------|------|
| 1 | validity flag | share |
| 1 | ring index | share |
| 2 | sector, as a triangle pair | signed relation |
| 1 | hex distance from the centre | magnitude |
| 1 | population over the own total population | share |
| 1 | population | magnitude |
| 1 | store value over the own total store value | share |
| 1 | finished upgrade count | magnitude |
| 1 | upgrades under construction | magnitude |
| 1 | garrison strength over the own total strength | share |
| 1 | garrison strength | magnitude |
| 1 | food coverage in ticks | magnitude |
| 1 | hazard flag | share |
| 1 | distance to the nearest known rival settlement | share |
| 1 | distance to the reach boundary | share |
| 1 | rival strength within 8 tiles | magnitude |
| 1 | own strength within 8 tiles | magnitude |
| 1 | unclaimed passable share within 8 tiles | share |
| 1 | tiles lost near it over the window | magnitude |
| 1 | population change over the window | signed relation |
| 1 | siege indicator | share |
| 1 | wonder progress at the settlement | share |
| 1 | settlement age in ticks | magnitude |

A threat cluster token holds 20 channels.

| Channels | Signal | Kind |
|----------|--------|------|
| 1 | validity flag | share |
| 1 | ring index | share |
| 2 | sector, as a triangle pair | signed relation |
| 1 | hex distance from the centre | magnitude |
| 1 | strength | magnitude |
| 1 | strength over total remembered rival strength | share |
| 1 | unit count | magnitude |
| 1 | share of the largest unit type class | share |
| 1 | distance to the nearest own settlement | share |
| 1 | closing rate over the window | signed relation |
| 1 | sighting staleness | magnitude |
| 1 | flag for inside own reach | share |
| 1 | flag for inside the owner reach | share |
| 1 | owner power share | share |
| 1 | owner relation to the faction | signed relation |
| 1 | hazard share on the cluster | share |
| 1 | passable share around the cluster | share |
| 1 | own strength within 8 tiles | magnitude |
| 1 | local strength balance | signed relation |

A candidate site token holds 16 channels.

| Channels | Signal | Kind |
|----------|--------|------|
| 1 | validity flag | share |
| 1 | ring index | share |
| 2 | sector, as a triangle pair | signed relation |
| 1 | hex distance from the centre | magnitude |
| 1 | mean food within 2 tiles | share |
| 1 | mean water within 2 tiles | share |
| 1 | mean value within 2 tiles | share |
| 1 | resource share within 2 tiles | share |
| 1 | passable share within 2 tiles | share |
| 1 | flag for inside reach | share |
| 1 | distance to the nearest own settlement | share |
| 1 | distance to the nearest known rival settlement | share |
| 1 | rival strength within 8 tiles | magnitude |
| 1 | hazard share within 2 tiles | share |
| 1 | site score | share |

The engine selects each token set with a bounded partial selection. It keeps a
fixed-size binary heap of eight entries and scans the candidate list once.

For settlements the candidate list is the faction settlement list, iterated in
ascending settlement identity. For threat clusters the candidate list is the
faction's remembered summary cells, not the unit list. A faction with hundreds
of thousands of rival units in view must not scan those units. The summary
level already holds the strength sum for each cell. For candidate sites the
candidate list is the summary cells inside or adjacent to reach.

### 9.10 Block J: weather and hazard, 12 slots

| Slots | Signal | Kind | Denominator |
|-------|--------|------|-------------|
| 1 | observed area under a storm | share | observed tiles |
| 1 | observed area under a water hazard | share | observed tiles |
| 1 | observed area under fire | share | observed tiles |
| 1 | held ground under a storm | share | held tiles |
| 1 | held ground under a water hazard | share | held tiles |
| 1 | held ground under fire | share | held tiles |
| 3 | change in each held-ground share over the window | signed relation | the share |
| 1 | units lost to a hazard over the window | magnitude | cap |
| 1 | tiles burnt over the window | magnitude | cap |
| 1 | hazard concentration across the twelve sectors | share | sum of squared shares |

### 9.11 Block K: affordance, 24 slots

| Slots | Signal | Kind | Denominator |
|-------|--------|------|-------------|
| 1 | flag for whether the faction may found a settlement now | share | 1 |
| 1 | settle cost coverage | share | required stores |
| 1 | flag for whether the best sector admits a settlement | share | 1 |
| 8 | affordability of each upgrade class | share | required stores |
| 8 | legality flag for each upgrade class | share | 1 |
| 1 | share of units free to take an order | share | own unit count |
| 1 | share of stores committed to live contracts | share | own total stock |
| 1 | headroom for new board rows | share | row limit |
| 1 | count of acceptable contracts | magnitude | cap |
| 1 | reach headroom for a new settlement | share | reach radius |

A flag is a share that holds 0 or 65536. The design uses no other
representation for a boolean. No slot holds a Rust `bool`, because an event
type and an observation type must be plain data with declared padding.

### 9.12 Block L: objective weights and reserve, 41 slots

The first twelve slots hold the objective weight vector of section 10. The
remaining 29 slots hold zero.

The reserve exists so that adding a signal does not change the width. A width
change invalidates every trained policy and every stored checkpoint. The
engine asserts that every reserved slot reads zero until a layout revision
claims it. The reserve size of 29 slots is a judgement, not a measurement.

## 10. The reward side

### 10.1 Objectives and shaping terms

The observation and the reward read the same quantities. That is deliberate. A
quantity the policy cannot see is a poor reward term. The policy cannot
attribute a change in it to an action.

The four victory conditions make the only sound terminal objectives. Each
gives a clean, sparse signal, and each is what the game actually rewards. The
design publishes the terminal reward as the win margin, not as a binary win. A
margin carries a gradient and a binary win does not.

The shares in blocks A, C and D make the good shaping terms. Each is already
scale free, each is bounded, and each is monotone in the direction a strong
player wants.

Two categories make poor reward terms. A compressed magnitude makes a poor
term. Its derivative falls with the quantity, so an early gain outweighs a
late gain of the same size. A signal indexed by position makes a poor term.
The reward then follows where the faction sits, not how well it plays.

### 10.2 The objective vector

The design publishes a twelve-element objective vector `v`. Each element is a
signed relation, so each lies between minus 1 and 1.

1. Change in the territory share.
2. Change in the population share.
3. Change in the military strength share.
4. Change in the finished upgrade share.
5. Change in the renown share.
6. Change in the store value share.
7. Change in the observed area share.
8. Rival territory removed, as a share of rival territory.
9. Own settlements lost, negated, as a share of own settlements.
10. Net trade surplus as a share of the own throughput.
11. Change in the mean diplomacy relation.
12. The terminal win margin, which is zero until the episode ends.

The reward at a decision is the integer dot product `w . v`, held in Q16.16
and divided by the sum of the absolute weights. The result is bounded, which
matters for a fixed-size step.

### 10.3 Training several play styles from one system

Vary `w` to train a style. An aggressive style weights terms 3 and 8. A
defensive expansionist weights terms 1, 4 and 9. A trade-led style weights
terms 6 and 10. A diplomatic style weights terms 11 and 5. A wealth-hoarding
style weights term 6 alone.

Append `w` to the observation, in the first twelve slots of block L. One
policy then serves every style, and the style becomes an input rather than a
separate set of weights. This is the goal-conditioned form of a value
function. The multi-objective literature uses it to recover a whole preference
front from one network.[^10] [^11] [^12] Sample `w` from a fixed integer
simplex grid during training. The policy then sees the whole front.

### 10.4 A property of shaping that evolution strategies change

Potential-based shaping adds `gamma * Phi(s') - Phi(s)` to the reward. It
leaves the optimal policy unchanged for any potential function.[^13] That
guarantee is the reason to prefer it under a gradient method.

Under evolution strategies the guarantee has a consequence that is easy to
miss. Evolution strategies optimise the sum of the reward over the whole
episode, with no discount inside the episode. A potential-based term then
telescopes: the sum over the episode equals `Phi(end) - Phi(start)`. The
shaping therefore contributes one number per episode and gives no intermediate
signal at all.

The design draws two conclusions. Under evolution strategies, use the level of
a share integrated over time. Do not use its first difference when a dense
signal is wanted. Accept that this biases the optimum, and state the bias.
When the trainer moves to a gradient method, switch the same terms to the
potential-based form and recover the guarantee. The objective vector supports
both, because element `k` is a first difference and the engine also publishes
the level it differences.

## 11. Policy architecture

### 11.1 The starter design

Flatten all 2976 slots. Feed one multilayer perceptron of 2976 to 256 to 128,
then the action heads. The parameter count is about 800 thousand.

This works with evolution strategies, and it is the right first
implementation. Its cost is sample efficiency. It shares no weight across the
ring stack. It must learn each of the 79 cells separately, so it learns a
direction seventy-nine times.

### 11.2 The intermediate design

Three towers over one shared trunk.

The spatial tower reshapes the ring stack to 8 rings by 12 sectors by 25
channels, as section 9.5 states. It runs two convolutions with a 3 by 3
kernel, from 25 to 32 to 32 channels. It uses circular padding on the sector
axis and replicate padding on the ring axis. It then pools twice: a global
mean and maximum, and a per-ring mean. The output is about 128 features from
about 16 thousand parameters. A sector wraps and a square edge does not.
Circular padding on the sector axis is therefore the reason to prefer a ring
frame to a square crop.

The set tower runs a shared per-token network over each token set, then pools
with a mean and a maximum. The set literature proves that this form is
general. Any permutation-invariant function decomposes into a per-element map,
then a sum, then an outer map.[^14] A 24 to 64 to 64 per-token network with
two pools costs about 6 thousand parameters for each set.

The scalar tower runs one layer over the 1001 non-spatial slots to 128
features.

The trunk concatenates the three and runs one 128-unit layer to the heads. The
total is about 200 thousand parameters, which is a quarter of the starter
design and carries more structure. This is the recommended design.

### 11.3 The state-of-the-art design

Treat every element as a token in one set. Each ring-sector cell becomes a
token with a learned ring embedding and a learned sector embedding. Each
settlement, rival, threat cluster and site becomes a token. The token count is
79 plus 6 plus 8 plus 8 plus 8, which is 109. One further token carries the
scalar blocks.

Run three layers of masked multi-head attention over the 110 tokens. Pool the
result with a learned query.[^15] [^16] Add a recurrent core over decisions,
so the policy carries memory that fog denies it. Both leading agents in large
strategy domains use this shape. One attends over entity tokens and scatters
the result back to a spatial map.[^6] The other pools over unit embeddings and
attends over targets.[^7]

The cost per layer is the square of the token count times the width. That is
110 squared times 128, or about 1.5 million multiply-accumulate operations.
Three layers plus the feed-forward blocks give about 20 million operations per
decision. That is small.

The trade-off is the parameter count. Three attention layers at width 128 run
to about 600 thousand parameters, and a recurrent core adds more. Evolution
strategies scale poorly in the parameter count. The estimator variance grows
with the dimension.[^2] This design needs a gradient method. Do not adopt it
until the trainer moves.

One further option belongs here. Feed the ring index and the sector as
sinusoidal position features rather than as a learned embedding. A random or
fixed Fourier feature map lets a network learn a high-frequency function of a
low-dimensional coordinate.[^17] A plain network does not learn one. The
integer triangle wave of section 4 gives a first harmonic. A three-harmonic
integer version costs six slots per token and is worth a test.

## 12. How to validate the design

### 12.1 The tests

Each test below states what it proves and how it fails.

Width invariance. Build the observation on a 24 by 24 world with 2 factions.
Build it again on a 512 by 512 world with 16 factions. Assert both are 2976
slots. Assert every block starts at the same offset in both.

Bounds. Run a property test over generated worlds. Assert every slot lies
between minus 65536 and 65536. Include a faction with 10 to the twelfth in
stores and a faction with zero of everything. Assert no slot saturates the i32
and no division divides by zero.

Determinism. Build the observation at 1 thread, at 2 threads and at 12 threads
from the same state. Compare the arrays byte for byte. Build twice in one
thread and compare.

Fog containment. Build the observation. Mutate a region the faction has never
observed. Build again. Assert the arrays are byte identical. This is the test
for physics property 4.

Seat invariance. Relabel the seats of every rival by a permutation. Assert
block D is byte identical. Assert the rival token set is identical as a set.

Cost bound. Instrument the build to count the tiles and summary cells it
touches. Build on a 24 by 24 world and on a 512 by 512 world with the same
observed area. Assert the counts match. Assert the count is a function of the
observed area and of nothing else.

Scale invariance of meaning. Build a world. Build a second world that scales
each axis by 4 and scales every quantity in proportion. Assert every share
slot differs by less than a stated tolerance. This is the test that catches a
regression to a raw count.

No constant slot. Sample a corpus of states from a scripted policy. Assert no
slot holds the same value in every state. A constant slot is either inert or
mis-wired, and a policy wastes capacity on it.

Level agreement. Take a cell that the faction has observed in full. Assert
that the ring cell built from the summary level equals the same cell built
from level 0 tiles. The summary level is a derived projection, and a
projection that disagrees with the truth is a defect.

### 12.2 The tests that prove the tests can fail

A test that compares a run against itself always passes and proves nothing.
Each test above needs a paired negative test behind a test-only switch.

For determinism, replace the shard combination order with the thread
completion order. Assert the determinism test then fails.

For fog containment, disable the observed mask on the summary read. Assert the
fog test then fails.

For seat invariance, replace the token sort key with the seat index. Assert
the seat test then fails.

For bounds, remove the clamp from `share`. Assert the bounds test then fails.

For the cost bound, replace the remembered summary iteration with a full world
scan. Assert the cost test then fails.

### 12.3 The fixture must reach the extreme

A defect appears at an extreme of a distribution. A fixture built by copying a
demonstration world supplies no extreme. An assertion then never receives the
input that would fail it. The test then measures the fixture.

The fixture corpus must include each case in the list below.

- A faction with zero settlements.
- A faction with one held tile.
- A faction that holds 90 percent of the world.
- A game with 16 seated factions.
- A game with 2 seated factions.
- A faction that has observed 1 percent of the world.
- A faction that has observed all of the world.
- A faction with a store of 10 to the twelfth.
- A world where every ring beyond ring 3 lies outside the world.

Put the defect back and watch the test stay green. That is the only proof that
a fixture reaches a case.

### 12.4 Measuring whether a signal predicts winning

Do not spend training time on a signal before pricing it. Price it with
episodes, which are cheap, rather than with training runs, which are not.

Collect a corpus of episodes from a scripted policy and from a random policy.
For each episode, record the observation at a fixed set of episode fractions.
Record the outcome as the win flag and the final territory share.

For each slot, compute the rank correlation between the slot and the outcome
at each fraction. Use an integer rank correlation on the slot ranks. Rank the
slots by the absolute correlation. Report the strongest twenty and the weakest
twenty.

Then fit one linear probe from the whole observation to the outcome, and
record its accuracy. Refit with one block removed at a time. The accuracy drop
prices the block. A block whose removal costs nothing is a candidate for the
reserve.

Also bucket each slot into sixteen bins. Compute the mutual information with
the win flag, in integer arithmetic on the counts. Mutual information catches
a non-monotone signal that a rank correlation misses. Concentration in block D
is likely to be such a signal, because both extremes matter.

## 13. Alternatives rejected

Full-map raster at native resolution. The width follows the world tile count,
and the build cost follows the world area. It fails the fixed-width
requirement and the cost bound. Rejected.

Downsampled whole-map raster at a fixed resolution. A leading real-time
strategy environment publishes two views. It gives a fixed-size minimap of the
whole map, and a local screen view.[^18] The width is fixed, which is correct.
Two problems remain. The resolution per tile falls with the world size. One
settlement fills a pixel at 24 by 24 and covers no pixel at 4096 by 4096. The
build cost also follows the world area. Rejected on both counts, and partly
adopted. The outer rings of the ring stack do the same job at a cost bounded
by observed area.

Fixed local crop only. One large multi-agent environment gives each agent a
small square crop centred on itself.[^19] The width is fixed, and the cost is
constant. The agent reads nothing beyond the crop, which prevents a strategic
decision at map scale. Rejected as the whole answer, and adopted as the inner
rings.

The detail pyramid published directly. The pyramid holds one derived summary
level. Reading it whole gives a fixed channel count, and a cell count that
follows the world. Rejected for the same reason as the full raster, and
adopted as the source for the outer rings.

Cartesian egocentric mipmap crops. Publish three or four square crops centred
on the faction, each at half the resolution of the last. This is the closest
rejected alternative, and a future reader should reconsider it first. It is
simpler to compute than a ring frame, and it needs no sector arithmetic. Two
reasons decide against it. The ring and sector frame aligns with the six hex
directions, and therefore with the action space. The sector axis also wraps. A
circular convolution over a wrapping axis shares weight correctly, and a
square edge cannot. A hex world with hex actions therefore fits a hex-aligned
polar frame. The log-polar frame also has a long record in robotic vision for
the same reason. It gives high resolution at the centre and wide coverage at
bounded cost.[^20]

Per-faction indexed slots. Give each seat its own block. The width follows the
faction count, and the policy learns a seat number, so it fails on a different
seating. Rejected. This is the failure the design exists to remove.

Rivals sorted into fixed slots without a set encoder. Sort rivals by strength
and place them in fixed slots. The width is fixed, which is correct. The
policy learns slot-conditional behaviour, and its output jumps discontinuously
when two rivals swap rank. Rejected in favour of order statistics plus a
permutation-invariant encoder. A masked attention over entity tokens is the
established solution to exactly this problem.[^21]

A learned running input normaliser. Section 8.2 states the rejection. It holds
float state and it makes the mapping depend on history.

Per-episode minimum and maximum scaling. The denominator depends on the
episode, so one world state maps to different inputs in two runs. Rejected.

An input for the world size and the faction count, used as the whole
mechanism. Publishing the scale helps, and the design publishes it. It does
not by itself make the width fixed. Adopted as a signal, rejected as the
mechanism.

The full trade board as rows. The width follows the row count. Rejected in
favour of the per-good-class summary of block G.

A symbolic or textual observation read by a language model. It fits neither
the fixed-size step nor the cost bound, and it needs float state. Rejected.

Hand-scripted strategic features with no spatial input. Cheap and easy to
train. It caps the policy at the quality of the features. It cannot learn a
spatial idea that the author did not have. Rejected.

A raw entity list of every unit. One agent for a large team game encodes every
unit and pools over them.[^7] [^22] It works at a few hundred units. At one
million units the cost is prohibitive, and the token count is unbounded.
Rejected in favour of summary-level threat clusters.

Reading the shared summary level without a fog mask. Cheaper, and it would
remove the per-faction remembered record of section 5.4. It gives the faction
knowledge it has not observed. That breaks the fog rule, and it makes every
trained policy invalid against a human. Rejected.

## 14. What it costs

### 14.1 Memory

One observation holds 2976 i32 slots, which is 11904 bytes. Sixteen factions
hold 190464 bytes in total. The observation array is negligible against every
other structure in the engine.

The per-faction remembered summary record is the real cost. It holds nine fog-
dependent channels per observed summary cell. At eight bytes per i64
accumulator that is 72 bytes per cell, rounded to 80 bytes for alignment.

A 16.7 million tile world with a 256-tile summary block holds about 65 200
summary cells. A faction that has explored all of it pays about 5.2 megabytes.
Sixteen such factions pay about 84 megabytes. A faction that has explored 3
percent pays about 157 kilobytes.

The record must be a sorted array keyed by summary cell index, not a dense
array over the world. A dense array makes the cost follow the world area and
breaks physics property 4. A sorted array with a binary search makes the cost
follow the observed area. The ascending index order also gives the
deterministic iteration the design needs.

### 14.2 Work per decision

The ring stack costs 169 tile reads for rings 0 to 3. It costs one further
read per observed summary cell, for rings 4 to 7. At 3 percent explored on the
target world that is about 2100 reads, each a small number of i64 additions.
Call it 20 thousand integer operations.

Block A, block B, block C and block J read faction aggregates the engine
already maintains. The cost is constant.

Block D costs the seated faction count times twelve quantities, plus one
partial sort for the median. Call it 200 operations at 16 factions.

Block G costs the board row count. Block H costs the square of the faction
count for the pairwise mean, which is 256 operations at 16 factions.

Block I costs one pass over the faction settlement list with an eight-entry
heap. It costs a second pass over the observed summary cells, for the threat
clusters and the sites. The settlement pass is the settlement count, which
reaches thousands. The cell passes reuse the pass that builds the ring stack,
so they are free.

Block F costs one pass over the summary cells inside or adjacent to reach.
That is a subset of the ring stack pass and reuses it.

The total is about 25 thousand integer operations per faction per decision. At
16 factions and one decision every 10 ticks, the observation costs about 40
thousand integer operations per tick. Measure this on the target platform
before quoting a time. The development machines have a different cache line
size, so a local measurement misleads on false sharing.

### 14.3 How the cost scales

The width is constant in the world size and constant in the faction count.
That is the design goal, and every block meets it.

The build cost is linear in the observed summary cell count. That count is
linear in the observed area and independent of the world area. The inner rings
cost a fixed 169 tile reads at every world size.

The faction count enters only block D and block H. Block D is linear in the
count, and block H is quadratic in it. The count is bounded by 16, so both are
constant in practice.

The persistent memory is linear in the observed area per faction, and linear
in the faction count.

Nothing in the design is superlinear in the world size, and nothing is linear
in the world size.
## References

[^1]: Cachette project instructions, the hard invariants and the design principles. `CLAUDE.md`
[^2]: Salimans, Ho, Chen, Sidor, Sutskever, "Evolution Strategies as a Scalable Alternative to Reinforcement Learning", 2017. https://arxiv.org/abs/1703.03864
[^3]: Wierstra, Schaul, Glasmachers, Sun, Peters, Schmidhuber, "Natural Evolution Strategies", Journal of Machine Learning Research 15, 2014, pages 949 to 980. https://jmlr.org/papers/v15/wierstra14a.html
[^4]: Hafner, Pasukonis, Ba, Lillicrap, "Mastering Diverse Domains through World Models", 2023, the symlog input transformation. https://arxiv.org/abs/2301.04104
[^5]: Patel, "Hexagonal Grids", cube and axial coordinates and the distance formula. https://www.redblobgames.com/grids/hexagons/
[^6]: Vinyals and others, "Grandmaster level in StarCraft II using multi-agent reinforcement learning", Nature 575, 2019, pages 350 to 354. https://www.nature.com/articles/s41586-019-1724-z
[^7]: Berner and others, "Dota 2 with Large Scale Deep Reinforcement Learning", 2019. https://arxiv.org/abs/1912.06680
[^8]: van Hasselt, Guez, Hessel, Mnih, Silver, "Learning values across many orders of magnitude", 2016. https://arxiv.org/abs/1602.07714
[^9]: Hessel, Soyer, Espeholt, Czarnecki, Schmitt, van Hasselt, "Multi-task Deep Reinforcement Learning with PopArt", 2018. https://arxiv.org/abs/1809.04474
[^10]: Schaul, Horgan, Gregor, Silver, "Universal Value Function Approximators", International Conference on Machine Learning, 2015. https://proceedings.mlr.press/v37/schaul15.html
[^11]: Yang, Sun, Narasimhan, "A Generalized Algorithm for Multi-Objective Reinforcement Learning and Policy Adaptation", 2019. https://arxiv.org/abs/1908.08342
[^12]: Roijers, Vamplew, Whiteson, Dazeley, "A Survey of Multi-Objective Sequential Decision-Making", Journal of Artificial Intelligence Research 48, 2013, pages 67 to 113. https://www.jair.org/index.php/jair/article/view/10836
[^13]: Ng, Harada, Russell, "Policy invariance under reward transformations: theory and application to reward shaping", International Conference on Machine Learning, 1999. https://people.eecs.berkeley.edu/~pabbeel/cs287-fa09/readings/NgHaradaRussell-shaping-ICML1999.pdf
[^14]: Zaheer, Kottur, Ravanbakhsh, Poczos, Salakhutdinov, Smola, "Deep Sets", 2017. https://arxiv.org/abs/1703.06114
[^15]: Vaswani and others, "Attention Is All You Need", 2017. https://arxiv.org/abs/1706.03762
[^16]: Lee, Lee, Kim, Kosiorek, Choi, Teh, "Set Transformer: A Framework for Attention-based Permutation-Invariant Neural Networks", 2019. https://arxiv.org/abs/1810.00825
[^17]: Tancik and others, "Fourier Features Let Networks Learn High Frequency Functions in Low Dimensional Domains", 2020. https://arxiv.org/abs/2006.10739
[^18]: Vinyals and others, "StarCraft II: A New Challenge for Reinforcement Learning", 2017, the minimap and screen feature layers. https://arxiv.org/abs/1708.04782
[^19]: Suarez, Du, Isola, Mordatch, "Neural MMO: A Massively Multiagent Game Environment for Training and Evaluating Intelligent Agents", 2019. https://arxiv.org/abs/1903.00784
[^20]: Traver, Bernardino, "A review of log-polar imaging for visual perception in robotics", Robotics and Autonomous Systems 58, 2010, pages 378 to 398. https://www.sciencedirect.com/science/article/abs/pii/S0921889009001687
[^21]: Baker, Kanitscheider, Markov, Wu, Powell, McGrew, Mordatch, "Emergent Tool Use From Multi-Agent Autocurricula", 2020, the entity-centric observation with masked attention. https://arxiv.org/abs/1909.07528
[^22]: Ye and others, "Mastering Complex Control in MOBA Games with Deep Reinforcement Learning", 2019. https://arxiv.org/abs/1912.09729
