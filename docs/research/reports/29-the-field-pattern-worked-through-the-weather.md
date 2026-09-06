# The Field Pattern, Worked Through The Weather

Research report 29. It designs a replacement weather model, states the data
layout and the instruction-level design that the target platform rewards,
states the boundary rule that each field over this lattice needs, and separates
what is weather from what is a pattern. Prepared 6 September 2026.

Cachette is a world simulation engine. The core is Rust. The control plane is
Python. The engine simulates a hex world of up to 16,777,216 tiles. Simulated
state holds no floating point number, and every aggregate combines exactly in
any order.[^1] The primary target is a 64-bit Arm server.[^2] The engine holds
one property it cannot recover once lost, which is determinism.[^3]

**This author ran nothing.** No build, no test, no benchmark and no probe. A
second agent held the weather module while this report was written. Every
claim about the code comes from reading the tree on the `integration` branch
at commit `be89835`. Every cost figure here is derived, and each one names the
measurement it derives from. One blocker governs every cost figure in this
project.[^4] A second blocker governs what the weather is worth.[^5]

## 0. The conclusion

**Rewrite the water half of the weather model. Keep the air half. Do the data
layout as a separate change that comes first and that a golden hash can
prove.**

The repair history is not a list of bugs in a good model. It is a list of
terms that do not share a quantity.[^6] Evaporation follows heat and
condensation follows cold, and no repair to either one creates the quantity
they are both missing. That quantity is the saturation deficit, which is the
water the air can hold at its temperature less the water it holds. One
function then reads it and moves water in one direction or the other. That
change subsumes the fixed saturation ceiling and the separate fall rule, and
it costs about six integer operations for each cell.

The air half did not fail in the same way. The insolation, the season, the
wind, the drag and the turn each took one repair and then settled. Their last
three commits tune rather than replace. Keep them.

**The layout is a separate argument, and it is the more general one.** The
lattice is a rhombus in raw axial coordinates, so a cell index is the row
times the width plus the column.[^7] The six neighbours are therefore six
constant index offsets, and four of them come from the two adjacent rows. A
pass over such a lattice needs three row streams and no gather at all. No pass
in the engine uses this. Every one of them turns an index back into an
address, adds an axial offset, checks the bounds and multiplies back to an
index, six times for each cell. That chain measures about 9 nanoseconds for
each neighbour visit on a development machine, which is about thirty
cycles.[^8]

**The honest size of each prize.** The model change buys correctness and costs
nothing. The layout change buys about a factor of twenty-five on a field pass,
which does not change what is possible and does change what is affordable. It
is the difference between weather at one cell for each 1024 tiles and weather
at one cell for each tile. Resolution is what lets a field couple to a
tile-level system without averaging, and coupling is where the depth of this
engine comes from. So the layout buys depth, but only through resolution, and
only after the model is right.

**One defect comes before both.** The lattice is exactly the size of the world,
so air leaves one edge and nothing arrives at the other. Every mass must be
born inside the frame and the border is permanently starved of upwind. **A
field that advects needs a margin of simulated cells that nobody reads**, and
the margin width derives from the pass count. That is a property of any
advecting field and not of the weather. It costs 3 percent of the cells at the
target tile pitch and four times the work at the demonstration coarse pitch,
because a margin is a perimeter cost and the coarse lattice is eight cells
across.

**Be sceptical of the order.** The measured frame at the target scale is 836
milliseconds, and one serial pass holds 61.5 percent of it.[^9] A weather
stage that is twenty-five times cheaper changes nothing a watcher sees while
that pass stands. If the project may make one change, make the model change.

## 1. What the history measured, and what it did not

Twelve commits carry the current model.[^6] Each one repairs a structural
defect and each one holds a measurement. The defects are these.

- A symmetric spread kernel could not carry a maximum, so a storm flattened
  where it was raised.
- A season marched along one axis and wrapped, so it jumped the whole map
  every lap.
- A triangular season profile had a constant slope, so the pressure gradient
  reversed across one column and put a seam down the map.
- The pressure divisor shifted a reference value right before it shifted it
  left, so it returned one at every pitch and the value the documentation
  described reached nothing.
- The heat term read the mean height of a whole cell, and a water tile stands
  below the water mark by construction, so the deeper the sea the hotter it
  read.
- The lift, the wet mark, the saturation and the storm quantity are absolute
  counts of drops that do not scale with the tiles a cell covers.
- The saturation is one global constant that no temperature reads.

**Five of the seven are one shape.** A value was declared in one place and
meant something different in another, or a value was written down where it
should have been derived. That is recurring defect shape 1.[^10] The remedy is
not a better value. The remedy is a derivation.

**What the history does not show is churn in the air terms.** The sun position
with a smooth fall replaced the marching band once and has not moved. The wind
gained drag, then a turn, then a softer pressure divisor, and each change
measured better than the one before. Those terms are converging. The water
terms are not: the saturation ceiling was added to bound a runaway, the
rain-out was added to end a storm, the back-lift was added to give inland
cells a source, and each addition was a patch on the previous one.

## 2. Testing the diagnosis

The brief states a diagnosis and asks for a test rather than agreement. The
test is a reading of the code.

**The first half of the diagnosis holds.** The lift raises water in proportion
to the heat of a cell, from open water only.[^11] The fall takes a share of
the air that rises with the coldness of the cell, with the cooling the air met
on the way, and with the climb.[^11] Those are two quantities that point in
opposite directions, and no third quantity relates them. The saturation is a
constant of 2048 drops that no temperature reads.[^11]

**The second half needs one correction.** The brief says the poles are a sink
that destroys everything arriving. The destroying sink is not the
condensation. It is the drying pass. Every cell loses a share of its ground
water plus one whole drop on every tick, and that water leaves the field into
a running total rather than returning to the air.[^11] The return path from
the ground to the air is proportional to the heat of the cell, so at a cold
latitude the return path is zero while the drying path is not. **The asymmetry
is between the two directions of the same transfer, not between evaporation
and condensation.** That is a sharper statement, and it points at the same
remedy.

**One clause of the diagnosis does not hold.** The saturation deficit does not
subsume the wet mark. The wet mark is not a weather quantity. Three other
systems read it: the gather resolve, the production moisture term and the
upgrade wear pass.[^12] It is a game threshold on the ground water, it belongs
in the balance register, and it survives the rewrite unchanged in meaning. It
must change in value, because it is a stock and a stock scales with the pitch.

## 3. The model, term by term

The field holds five planes for each cell and one static record.

| Plane | Type | What it holds | In or out |
|---|---|---|---|
| `warmth` | `i16` | The temperature of the cell | In |
| `vapour` | `i32` | The water the air holds, in drops | In |
| `cloud` | `i32` | The condensed water the air carries | In |
| `ground` | `i32` | The water on the ground | In |
| `wind` | two `i16` | The flow, in the two lattice axes | In |
| `inside` | `u8` | The world edge mask | In |
| `CellGround` | static | The tiles, the open tiles, the land height and the water depth | In |

### 3.1 The saturation curve

**The capacity of the air is a function of its temperature, and the function
doubles at a fixed step.** Real capacity roughly doubles for each ten degrees.
A doubling is a shift, so the curve is exact in integers and needs no table
lookup in the pass.

Write the temperature as an exponent and a fraction. The exponent is the
temperature shifted right by the doubling width. The fraction is the
temperature masked to that width. The capacity is the base quantity, scaled
linearly inside one doubling, then shifted left by the exponent. That is one
shift, one mask, one multiply, one add and one variable shift. The Arm
baseline holds a vector shift whose amount is a lane, so the last step
vectorises.[^13]

**A 256-entry table stays in the module as the specification.** A test walks
the table and compares each entry against the arithmetic. The table is what a
reviewer reads. The arithmetic is what the machine runs. The test is what
fails when the two disagree, which is what recurring defect shape 1
demands.[^10]

**This buys** a saturation that varies over the map and over the year, and a
capacity that a cold cell holds less of than a warm one. It costs one plane
read that the field already makes.

### 3.2 The phase step: one function, two directions

**The deficit is the capacity less the vapour.** One function reads it once
for each cell.

- When the deficit is negative, the excess vapour condenses into cloud water.
  Condensation releases heat, which raises the temperature of the cell by a
  bounded share of what condensed.
- When the deficit is positive, the surface gives water to the air. The
  quantity is a share of the deficit, bounded by what the surface holds. Open
  water is an unbounded surface. Ground water is a bounded one.

**Both directions are exact integer moves.** What one side loses, the other
gains, and the account balances at every moment.[^14]

**This buys** the behaviour the brief names. Cold dry air over a cold sea takes
water off it, because the capacity is low and the vapour is lower still. Warm
saturated air over a warm sea takes nothing. A parcel that rises and cools
condenses, without any term that names a coast or a ridge.

**It replaces** the lift, the fall, the back-lift, the fixed saturation ceiling
and the rain-out. Five terms become one function.

**It does not replace** the drying of the ground, which is a real sink and
which the deficit already drives correctly: ground water returns to the air
when the air is dry, and stays when the air is wet. The unconditional drop
that leaves the field goes.

### 3.3 The rain

**Cloud water falls, and vapour does not.** That is why the two are separate
planes. A cloud that condenses in one cell and rains in the same tick cannot
travel, and a travelling storm is the thing the owner asked to see. Cloud
water advects with the wind and loses a fixed share to the ground on each
tick. The share is a share, so it does not scale with the pitch. The threshold
below which cloud does not rain is a stock, so it does.

**This buys** travelling storms and overcast skies, and it is the one term this
report adds that the current model does not hold in any form.

### 3.4 The wind

**Keep the current design.** The pressure gradient accelerates the wind by a
bounded step, drag bleeds it, a turn deflects it, and a ceiling bounds
it.[^15] Three repairs converged on it and the last one measured a rain
shadow, a travelling storm and a circulation.[^6]

**Make one change.** The turn must carry the sign of the latitude. Today the
deflection is a fixed share of a sixth of a turn in one direction, which is
the only turn the lattice holds exactly. A sixth-turn in the opposite sense is
equally exact. Choosing the sense by the row relative to the equator gives two
hemispheres that circulate opposite ways, and it gives banded prevailing winds
without any term that names a band. It costs one sign.

**Pressure is not a plane.** With one layer, pressure is a monotone function of
temperature, so storing it would be one fact in two places.[^10]

### 3.5 The insolation and the season

**Keep the current design.** A cell reads how far it sits from where the sun
stands, the sun swings on a sine of the tick, and the fall follows a curve
whose slope is zero at both ends.[^6] The seam defect is fixed and the tilt
was measured against two alternatives.

**Restate it in two parts.** The latitude term is fixed geography. The
declination is the swing. Writing them apart makes the latitude available to
the turn of the wind, which needs the same quantity, and removes a second
declaration of the equator.

### 3.6 Land and sea

**Keep the current design.** The temperature of a cell moves a share of the
way toward what the world asks, and the share is divided by a lag that grows
with the water depth.[^11] That is what makes a coast interesting, because the
land tracks the season while the sea does not, and the temperature difference
across a coast changes sign as the year turns. It was measured.[^6]

### 3.7 Advection and mixing

**Advection is a donor-cell flux to the six neighbours, and it stays that
way.** A cell gives each neighbour a share that rises with the part of its own
wind pointing that way. Both ends compute the same integer from the same
settled planes, so the transfer is exact.[^14]

**Mixing is a symmetric share, and it is not optional.** Pure advection with
no mixing tears the field into threads one cell wide. That was measured
directly: raising the mixing share four-fold moved the number of turning cells
from 771 to 70 and widened the widest storm from 592 cells to 3636.[^6]

**The travel distance for each tick is the pass count.** Exact conservation on
one integer plane forces a flux to the six adjacent cells. A donor step of
more than one cell would need a distance that varies for each cell, and a
varying distance is a gather. So the pass count stays a derived function of
the pitch, and this report does not claim to remove it.

### 3.8 What the model leaves out

- **A third dimension.** A layered field multiplies the whole stage by the
  layer count. The orographic term buys the behaviour a watcher recognises for
  one neighbour read, and that was the reasoning when the term was added.[^6]
  This is the largest thing the report rejects.
- **Ocean currents.** They would double the wind machinery for a feature no
  watcher can name at this scale.
- **Storms as objects.** A storm is what a convergence looks like. Tracking one
  as an object is a second declaration site for a thing the field holds.
- **Relative humidity as stored state.** A ratio is not conserved and cannot be
  moved exactly. Store the absolute vapour and derive the ratio for a display.
- **Snow and hail as stored state.** One bit derived from the temperature at
  read time gives a watcher the same picture for no plane.
- **Any convergence test.** The determinism rules forbid it.[^16]

## 4. The layout

### 4.1 The lattice already has the property, and nothing uses it

The world is a rhombus. A cell index is the row times the width plus the
column, and there is no offset step.[^7] The six neighbour offsets are the
constant set of minus the width, minus the width plus one, minus one, plus
one, plus the width minus one, and plus the width.

**So two neighbours lie in the row itself and four lie in the two adjacent
rows.** The brief states the split the other way round, and the code says
otherwise. The useful fact is not the split. It is that the four cross-row
neighbours come from only two row streams. **Three row streams give all six
neighbours.** From the row above, take lane `j` and lane `j+1`. From the row
below, take lane `j-1` and lane `j`. From the row itself, take lane `j-1` and
lane `j+1`.

A vector of lanes therefore needs three aligned loads and four shifted views
of them. The Arm baseline holds an instruction that concatenates two vectors
and extracts a shifted window, which is one instruction for each shifted
view.[^13] **No gather appears anywhere in the pass.**

### 4.2 What the passes do today

Every field pass in the engine spells the neighbour step the same way. It
turns the index back into an address with a reciprocal multiply, adds an axial
offset, checks four bounds, multiplies back to an index, and takes a dependent
load. It does this six times for each cell, and it carries two optional values
that each cost a branch.

The engine has already paid twice to make this chain cheaper. The reverse
index map stores a reciprocal so that it multiplies rather than divides,
because a measurement found the division to be the largest single part of
turning a tile into a cell.[^17] The approach field materialises a six-entry
neighbour table for each tile of a block, because the passes ask the same
question hundreds of times.[^18] **Both are workarounds for a chain the
lattice does not require.**

### 4.3 Planes, alignment and the row stride

**One aligned plane for each field, not a struct for each cell.** The weather
module already does this. The pyramid does not: its cell summary is a
seven-field record of 56 bytes, all of them 64 bits wide, and a pass that
reads one field pulls all seven.[^18] The climate field is the same shape at
40 bytes.[^19]

**Choose the narrowest element that holds the range, and widen only the
totals.** Today the water planes are 64 bits wide for a quantity whose ceiling
is 2048 and whose largest single injection is 16,384. Thirty-two bits hold it
with a factor of a hundred thousand to spare. The wind is two 32-bit values
for a quantity whose ceiling is 48. Sixteen bits hold it. **The narrower
element is a double win**: it halves the memory traffic and it doubles the
lane count of every vector instruction.

**The running totals stay 64 bits wide.** The raised total, the evaporated
total and the plane sums are aggregates over the whole world, and the hard
invariant on accumulator width is binding.[^20] The per-cell plane is 32 bits
and the accumulator is 64. That boundary must be stated where the totals are
declared, because it is exactly the place a later contributor would narrow one
to match the other.

**Pad the row stride so that every row starts on a 64-byte boundary.** The
target cache line is 64 bytes and that is a property of the platform the
project chose, not a budget.[^2] For a 32-bit plane the stride is a multiple
of 16 cells, and for a 16-bit plane a multiple of 32. **The padding costs at
most 15 cells for each row in the wide case**, so under 16 divided by the
width. At the target width of 4096 that is under 0.4 percent. At the
demonstration width of 256 it is under 6 percent, of a plane that measures
kilobytes.

### 4.4 The edges, which is where such schemes break

Two edges fail differently and both need an answer.

**The row edge is the one that breaks silently.** A shifted load of minus one
at column zero reads the last cell of the previous row. Nothing detects it and
the answer is plausible. **The answer is a halo column on each side.** The
stride is the width plus two, rounded up to the alignment, and the halo column
holds a cell that no thread writes as a real cell.

**The world edge is the one that changes the answer.** Today a neighbour
outside the world is skipped, and a skipped neighbour means the cell neither
sends that way nor takes that way. A halo that holds zero water gives the take
correctly and gives the send wrongly, because the sending cell would still
subtract.

**The answer is a mask plane, not a branch.** One byte for each cell, all ones
inside the world and zero outside. Every flux term multiplies by the mask of
the far end. A cell then sends nothing across the world edge and takes
nothing, which is exactly today's rule, and the inner loop holds no branch at
all. The mask also makes a stale halo harmless, because a masked flux is zero
whatever the halo holds.

**The halo fill costs the perimeter.** Two rows of the stride plus two columns
of the height. At the target that is about 24,600 cells against 16.7 million,
which is under 0.15 percent.

### 4.5 The boundary, which is a property of the field and not of the layout

Section 4.4 answers where a load may read. This section answers a different
question: **what the field means outside the world.** The two are separate. The
halo is a layout device and it holds whatever the boundary rule puts there.

**The owner reports the defect that makes this urgent.** The lattice is exactly
the size of the world. Air leaves one edge and nothing arrives at the other, so
every mass must be born inside the frame and the border is permanently starved
of upwind. A separate agent builds the remedy against the current field. This
report states only the general form.

**Four boundary rules exist, and the meaning of the field chooses between
them.**

- **A margin.** Cells outside the read area are simulated and never read. The
  interior then receives inflow that it did not make.
- **A wrap.** One edge takes the other edge as its neighbour.
- **A clamped edge.** The outside holds a fixed value that no pass writes.
- **An absorbing edge.** The outside holds a sentinel that means "not part of
  the field", and a pass takes nothing from it.

**The rule is short. A field that advects wants a margin. A field that relaxes
from sources inside the world wants a clamped or an absorbing edge. A field
that reduces wants no boundary at all.**

The reason is that a margin invents content, and inventing content is right
only when the content would genuinely exist. Air exists beyond the map. A
faction's influence does not, and a margin would manufacture influence from
nothing.

| System | What it does | The boundary it wants | Why |
|---|---|---|---|
| The weather field | advects | a margin | Air beyond the map is real, and the interior needs upwind it did not make |
| The influence solve | relaxes from sources | clamped at zero | No faction stands outside the world, so an outside value would be invented |
| The seeded and return fields | a distance relaxation | absorbing, by the existing sentinel | There is no ground out there to walk on. Already correct |
| The approach field | a distance relaxation inside a block | absorbing, by the existing sentinel | Already correct |
| The exit field | one sweep of neighbours | absorbing | Already correct |
| The holding map | a distance test to cities | none | It is not a stencil |
| The observation field | a shadowcast | absorbing, and an off-world block does not block sight | Already correct |
| The pyramid summary and the climate field | a reduction | none | Neither reads a neighbour |
| The terrain and the resource field | generated on demand | defined everywhere | A margin cell has real ground for free |

**So the weather is the only system in the tree that wants a margin, and it is
the only one that is currently wrong.** Three systems already hold the right
rule and hold it with a sentinel. That is another convergence, and it is
further evidence that the boundary follows from the meaning rather than from
taste.

**The margin gets its ground for nothing.** The terrain is a pure function of
the seed and the address, and it allocates nothing.[^23] So a cell outside the
world has a real height, a real moisture and a real kind, without any store and
without any generation step the field does not already run. **A margin cell is
therefore not invented content. It is content the world already defines and
does not display.** That is what makes the margin the honest answer here and
not a fudge.

**Reject the wrap, and say why, because it is the cheapest option.** A wrap
costs no extra cell at all. It is wrong here for two reasons. The terrain is
not periodic, so a wind crossing the seam would carry the temperature and the
humidity of a coast that has no relation to the one it arrives at. The season
term is not periodic in the row axis either. **A wrap would put a seam down the
map, which is the exact defect the season profile already produced once.**[^6]
A wrap is the right answer for a world that is a cylinder, and this world is a
rhombus by an accepted record.[^7] If the world ever becomes a cylinder, the
wrap becomes correct and the margin becomes waste, and that is a decision for a
record rather than for a field.

**The margin width derives from the pitch, like every other constant.** A
transport pass carries water one cell, so information crosses the margin in as
many passes as the margin is wide. **Set the margin equal to the transport pass
count.** The interior is then unaware of the outer boundary for a whole tick,
which is exactly as long as it needs to be, because the boundary is refilled
every tick anyway. That is a derivation and not a written-down number, which is
what section 6.2 demands of every constant.

**The proposed layout makes a margin cheaper than it is today, and it pays for
a good part of itself.** Three reasons.

**A margin is the halo, widened.** Section 4.4 already adds one halo column and
one halo row, and it already rounds the stride up to the cache line. A margin
of `M` cells is the same mechanism with `M` in place of one. The stride
arithmetic does not change shape, and the six neighbour offsets do not change
at all, because they are stated against the stride. **A margin on today's
layout is a new concept. A margin on the proposed layout is a parameter.**

**The margin removes the mask from the inner loop.** Section 4.4 needs a mask
plane because the world edge sits where the passes read. With a margin, the
read area has no edge inside it, and the boundary rule runs once for each tick
over the outer ring. So the inner loop drops one byte of state for each cell,
one load and one multiply for each of six flux terms. **That is about seven
operations for each cell for each stencil pass**, against a margin that costs 3
percent of the cells at the target tile pitch. The saving is larger than the
cost there, and smaller than the cost on a small lattice.

**The boundary fill costs the ring, not the area.** The outer ring is
`2M(W + H + 2M)` cells. At the target tile pitch with a margin of 32 that is
about 528,000 cells against 17.3 million, which is 3 percent, and it runs once
for each tick rather than once for each pass.

**Say the case where it does not pay.** On a lattice smaller than about ten
times the margin, the margin costs more than the mask saves, because the
multiplier grows as the square. Section 5.3 gives the arithmetic. The
demonstration at the coarse pitch is that case, and it costs four times the
work.

### 4.6 Threads, and the false sharing the development machines hide

**Chunk by whole rows, and assign the row bands by the row index.** The band
index is a stable key, so the answer does not depend on the thread count.[^3]
Today the chunk length is the cell count divided by the thread count, which
splits rows at arbitrary offsets.

Row bands with 64-byte-aligned rows mean that two threads never write one
cache line on the target. **A development machine cannot check this.** Apple
Silicon uses a 128-byte line, so a 64-byte-aligned row shares a line with its
neighbour there and does not on the target. A local test would report a
problem the target does not have. That is the reverse of the usual warning and
it is worth stating in that direction.

### 4.7 Temporal tiling, and why the pass count stops mattering

At the tile pitch the field runs 35 stencil passes for each tick over 16.7
million cells. A pass that streams a plane from memory and writes another
moves about 8 bytes for each cell, so 35 passes move about 4.7 gigabytes for
each tick. That is a bandwidth wall.

**Three rows of a 4096-wide 32-bit plane are 48 kilobytes.** That fits the
level 2 cache of one Graviton3 core with room for every plane at once. So a
thread that takes a band of rows and runs all 35 passes on that band, rather
than one pass over the whole plane, sees main memory about once for each plane
for each tick rather than 35 times.

**A pass barrier is the obstacle, and skewing removes it.** Pass `p+1` at row
`r` needs pass `p` at rows `r-1` to `r+1`. A band therefore runs pass `p` over
a range that narrows by one row on each side for each pass, and neighbouring
bands exchange the overlap once. **The decomposition is by band index, which
is a stable key, and the arithmetic is exact integer, so the answer does not
depend on the band count.** That is the property a float model does not have,
and it is why this technique is available here and is not available to a float
simulation that must reproduce its own results.

### 4.8 The vector width is free, and one record over-constrains it

**Integer arithmetic vectorises exactly.** There is no reassociation, so two
lane counts give one answer. **Determinism does not constrain the lane count,
and that is unusual.** A float model must fix the lane count and the reduction
shape, because changing either changes the sum. This project banned floating
point in simulated state for a different reason, and this is a consequence it
gets for nothing.[^1]

An accepted record states that vector code compiles for the baseline of the
target and that the engine holds no run-time dispatch. Its reasoning is that a
dispatch would mean two implementations of one calculation, and that two
implementations in a project that hashes its state each frame is two
answers.[^2] **That reasoning is a floating point reasoning.** For exact
integer arithmetic, two lane counts are one answer, and a test can prove it in
the same way the thread-count test proves the thread count does not matter:
run one tick at two lane counts in one process and compare the event log byte
for byte.

**This report does not ask to overturn the record.** Two implementations can
still differ from a coding mistake, so the record's caution is not worthless.
It asks for a record that carves out the exact-integer case and states the
test that pays for it. Graviton3 holds a 256-bit vector length beside the
128-bit baseline, so the carve-out is worth about a factor of two on a
compute-bound pass.

### 4.9 What a development machine misleads about

The project already records that a local measurement proves nothing.[^2] Four
specific traps apply to this work.

1. **The cache line.** 128 bytes on Apple Silicon against 64 on the target.
   The padding decision and every false-sharing test.
2. **The gather.** x86-64 holds a gather instruction and the Arm baseline does
   not. **A gather-shaped pass is worse on the target than it is locally, and
   every measurement this project holds of the weather is an x86-64
   measurement.** That is the trap that matters most here, because the current
   code is gather-shaped.
3. **The bandwidth for each core.** An Apple core pulls far more than one vCPU
   of a 16-vCPU cloud instance. A bandwidth-bound pass looks cheap locally.
4. **The vector width.** 256 or 512 bits locally against 128 in the target
   baseline. A local autovectorisation win at 16 lanes is 4 lanes on the
   target. Autovectorisation is also silent when it fails, so the assembly must
   be checked, and local assembly says nothing about target assembly.[^21]

## 5. What each pass costs

### 5.1 The measurement this section derives from

One probe measured the weather stage on a development machine: an x86-64 host,
a 256 by 256 world, 200 ticks, one thread, as the nanoseconds the stage took
for each tick.[^8]

| Pitch | Cells | Transport passes | Stencil cell-passes | Measured ns for each tick |
|---|---|---|---|---|
| 32 tiles a side | 64 | 4 | 448 | 29,022 |
| 16 tiles a side | 256 | 8 | 2,816 | 193,596 |
| 8 tiles a side | 1,024 | 16 | 19,456 | 1,116,622 |
| 1 tile a side | 65,536 | 32 | 2,293,760 | 120,809,461 |

A stencil cell-pass is one cell visited once by one pass that reads
neighbours. The count is the cells times the transport passes plus two wind
passes plus one carry pass.

**The cost for each stencil cell-pass is about 53 to 65 nanoseconds across a
range of 1024 in the cell count.** Divided by six neighbours that is about 9 to
11 nanoseconds for each neighbour visit, or about thirty cycles. **The
consistency across four pitches is the evidence that the cost is the neighbour
chain and not the cache.** A cache effect would not hold flat from 64 cells to
65,536.

### 5.2 The redesigned pass, term by term

Per cell, per tick, the design runs six passes. Three read neighbours.

| Pass | Neighbours | Bytes read | Bytes written | Lane operations |
|---|---|---|---|---|
| Insolate and lag the temperature | no | 14 | 2 | 14 |
| Wind from the pressure gradient | yes | 8 | 4 | 26 |
| Advect the temperature | yes | 6 | 2 | 30 |
| Advect the vapour and the cloud | yes | 12 | 8 | 44 |
| Phase, from the saturation deficit | no | 22 | 14 | 40 |
| Rain and dry | no | 8 | 8 | 12 |

**The state is 31 bytes for each cell**, against 72 bytes today. That is five
planes, two scratch planes, one mask and the static ground record. It is
smaller despite holding one more plane, because the elements are narrower.

At the target scale the tile-pitch field is 537 megabytes. At the level 1 pitch
it is 524 kilobytes, which fits the level 2 cache of one core.

### 5.3 What the margin costs, in the arithmetic rather than in a footnote

**A margin is a constant multiplier on every pass and on every byte of state.**
A lattice of `W` by `H` cells with a margin of `M` on each side holds
`(W + 2M)` by `(H + 2M)` cells. The multiplier is therefore
`(1 + 2M/W) × (1 + 2M/H)`.

Section 4.5 sets `M` equal to the transport pass count, which is itself derived
from the pitch. So the multiplier is known at every pitch and at every world
size.

| World | Pitch | Lattice | Passes, so margin | Multiplier |
|---|---|---|---|---|
| Demonstration, 256 tiles | 32 tiles a side | 8 by 8 | 4 | **4.00** |
| Demonstration, 256 tiles | 1 tile a side | 256 by 256 | 32 | **1.56** |
| Target, 16.7 million tiles | 32 tiles a side | 128 by 128 | 4 | **1.13** |
| Target, 16.7 million tiles | 1 tile a side | 4096 by 4096 | 32 | **1.03** |

**The margin is expensive on a small lattice and nearly free on a large one.**
At the target tile pitch it costs 3 percent. At the demonstration coarse pitch
it costs four times the work, because the lattice is eight cells across and the
margin is four cells on each side.

**That last row is the finding, not the cost.** A lattice whose margin is as
wide as the lattice is a lattice that is entirely border. **The owner reports
that the weather looks wrong at exactly the configuration where the arithmetic
says every cell is a border cell.** The margin does not repair that
configuration. It prices it, and the price says the demonstration should run at
a finer pitch rather than pay four times for eight cells.

**The margin therefore strengthens the case for the layout work rather than
weakening it.** The multiplier falls as the lattice grows, and the layout is
what makes a large lattice affordable. A coarse lattice pays the most for its
boundary and gets the least from it.

### 5.4 The demonstration scale

The demonstration world is 256 by 256 tiles.

| Pitch | Cells | Measured today | Derived, redesigned | Factor |
|---|---|---|---|---|
| Level 1, 32 tiles | 64 | 29,022 ns | under 5,000 ns | about 6 |
| Per tile | 65,536 | 120,809,461 ns | 4 to 8 ms | about 20 |

At 64 cells the whole field is under two kilobytes and the tick cost is the
call rather than the work. Neither figure matters in a picture that runs for
2.8 seconds.

**The tile-pitch figure does matter.** The demonstration picture costs 19.1
seconds at the tile pitch today against 2.8 seconds at the default.[^12] At 4
to 8 milliseconds for each tick the tile pitch would cost about 3.5 seconds.
**That would make tile-pitch weather usable in the demonstration**, which is
the one place a watcher can judge the model at all.

### 5.5 The target scale

The target is 16,777,216 tiles. The measured frame at that scale is 836
milliseconds at 12 threads.[^9]

**At the level 1 pitch** the lattice holds 16,384 cells and runs 7 stencil
passes.

- Today, derived by transferring the per-cell figure: 6.4 milliseconds for
  each tick at one thread. That is 0.77 percent of the frame.
- Redesigned: under 0.5 milliseconds.

**The coarse pitch is affordable either way, so cost is not the question
there. Correctness is.**

**At the tile pitch** the lattice holds 16,777,216 cells and runs 35 stencil
passes.

- Today, derived: 32.9 seconds for each tick at one thread, or about 2.74
  seconds at 12 threads with perfect scaling. **That is 3.3 frames of weather
  for each frame.** Unaffordable by a factor of about thirty.
- Redesigned, bandwidth bound with no temporal tiling: 4.7 gigabytes for each
  tick, which is about 117 milliseconds at an assumed 40 gigabytes each
  second.
- Redesigned, with temporal tiling: main memory sees each plane about once for
  each tick, so 738 megabytes and about 18 milliseconds. The pass then becomes
  compute bound.
- Redesigned, issue-rate bound: about 1400 lane operations for each cell for
  each tick, which is 23.5 billion lane operations. At four lanes and two
  vector instructions for each cycle across 12 cores at 2.6 gigahertz that is
  about 94 milliseconds. At the 256-bit vector length, about 47 milliseconds.

**So tile-pitch weather lands between about 50 and 120 milliseconds for each
tick on the target, against a derived 2.74 seconds today. The factor is about
twenty-five.**

**Every figure in this section is derived and three assumptions carry it.**
The transfer of an x86-64 per-cell figure to an Arm target is not valid and is
used only because it is the only measurement that exists. Perfect thread
scaling is assumed. The memory bandwidth of a 16-vCPU Graviton3 slice is
assumed at 40 gigabytes each second and has not been measured. A benchmark on
the target settles all three, and the register that would hold it holds no
weather row today.[^9]

### 5.6 Is the correct model cheaper or dearer?

**Dearer, at equal layout, and by about a quarter.** The saturation curve adds
about six operations. The latent heat adds about four. The separate cloud
plane adds one advected plane, which is four bytes of state and about fourteen
lane operations. Against that, five terms collapse into one function, which
removes about ten operations.

**Say this plainly: the model change costs more per cell than the model it
replaces.** The layout change pays for it about twenty times over. The two are
independent, and the honest accounting keeps them apart rather than netting
them off.

## 6. What generalises, and what does not

This section is the one that outlives the weather.

### 6.1 Weather only

- The saturation curve, and the doubling step it uses.
- The deficit as the one quantity that both directions of a transfer read.
- The separation of vapour from cloud, which exists so that a storm travels.
- The latitude sign on the turning term.
- The thermal lag of deep water.

### 6.2 Every field over this lattice

- **One aligned plane for each field, not a struct for each cell.** A pass
  reads and writes contiguous memory, and a pass that reads one field pulls
  one field.
- **The narrowest element that holds the range, with 64-bit accumulators for
  the totals.** The width is a bandwidth decision and a lane-count decision at
  the same time.
- **Neighbours as three row streams and four shifted views, never as a
  gather.** The lattice gives this for nothing and no pass takes it.
- **A halo column and a halo row, so no inner loop holds a bounds check.**
- **A mask plane rather than a branch for the world edge.** It also makes the
  refactor provably behaviour-preserving.
- **A boundary rule chosen from what the field means, not from the layout.** A
  field that advects wants a margin of simulated cells nobody reads. A field
  that relaxes from sources inside the world wants a clamped or absorbing edge.
  A field that reduces wants no boundary. **The margin width derives from the
  pass count, and the margin replaces the mask in the inner loop.**
- **Row-aligned thread bands, assigned by row index.** A stable key, and no
  false sharing on the target line.
- **Integer lanes chosen freely, because the answer cannot depend on the
  width.** Proven by a test that runs one tick at two widths and compares.
- **A fixed pass count with no convergence test.** Already the rule.[^16]
- **Every constant derived from the lattice pitch, and each one declared with
  its class.** A constant is a stock, a share, a distance, a gradient or a
  time. A stock scales with the tiles a cell covers. A share does not scale. A
  distance in cells scales inversely with the cell side. A gradient across a
  cell scales with the cell side. A time does not scale. **Declare the class
  beside the value, and assert at build time that the reference pitch
  reproduces the tuned figure.** Not doing this is the source of four of the
  seven defects in the history.
- **Temporal tiling over row bands for any solver of more than a few passes.**

### 6.3 What does not generalise

- Temporal tiling needs many passes over one plane. A single-sweep field gains
  nothing from it.
- The halo needs a dense plane. A field that stores only what it observed
  cannot have one.
- The shifted-load neighbour step needs a dense row-major plane over the whole
  world. A block-local index is contiguous within a block and is not a world
  row, so the offsets differ.
- **A margin does not generalise, and only one system wants one.** A margin
  invents content, and inventing content is right only where the content would
  genuinely exist. Air beyond the map is real. A faction's influence is not.
  **The margin is the one part of this design that is a weather decision
  wearing the clothes of a pattern**, and section 4.5 is the place that says
  which system wants which boundary instead of giving one answer.

## 7. The other field systems

A survey of the tree reports the following.[^22] Four systems already hold
planes, a fixed pass count and a six-neighbour relaxation, and they differ
only in element width and in how they spell the neighbour step.

| System | Planes | Neighbour step | Halo or alignment | Fixed passes | Element |
|---|---|---|---|---|---|
| The influence solve | yes, four | gather | no | yes, eight | `u16`, `u8` |
| The weather field | yes, eight | gather | no | yes, derived from the pitch | `i64`, `i32` |
| The pyramid summary | no, one 56-byte record | not applicable | no | not applicable | seven `i64` |
| The exit field | yes | gather into the record above | no | one sweep | `u8` |
| The seeded and return fields | yes, three | gather | value sentinel | yes, eight | `u8` |
| The approach field | yes, five | a materialised neighbour table | value sentinel | yes, derived from the block edge | `u8` |
| The holding map | yes, seven columns | not a stencil | no | not applicable | mixed |
| The observation field | no, a per-block tagged union | a shadowcast | not applicable | radius bounded | `u32`, `u64` |
| The resource field | no, generated on demand | none | not applicable | not applicable | `u32` |
| The terrain | no, generated on demand | none | not applicable | not applicable | `Fix32` |
| The climate field | no, one 40-byte record | none | no | tick count | five `i64` |

**The thesis is supported by convergence rather than by assertion.** Nobody
wrote a field rule, and four systems arrived at planes and a fixed pass count
anyway. Two of them then paid separately to work around the neighbour chain:
the reverse index map stores a reciprocal because a division measured as the
largest cost of one conversion, and the approach field caches a six-entry
neighbour table because its passes ask the same question hundreds of
times.[^17] [^18] **Two independent workarounds for one missing property is
better evidence than any argument in this report.**

**Three systems are outliers and each for a different reason.**

- **The pyramid summary and the climate field are records where planes would
  do.** Neither has a stencil, so the change is a pure layout change with no
  model question attached. The pyramid rebuild is 9.3 percent of the target
  frame, so the pyramid is the one with a measured stake.[^9]
- **The observation field is compressed by decision**, because fog storage must
  grow with the observed area and not with the world area. It cannot hold a
  dense halo plane. **The learner work will have to reconcile this**, because a
  learner observation is a dense tensor and nothing in the tree materialises
  one. That is a real conflict and it should be named in a record before the
  learner work assumes either shape.
- **The terrain and the resource field store nothing at all** and generate on
  demand. That is load-bearing for this report in the wrong direction: every
  field pass that needs the ground re-runs the generator for each cell for each
  pass. The approach field already hoists it once for each entry rather than
  once for each pass. **A shared terrain plane is a prerequisite for any pass
  becoming aligned loads**, and the weather module already has one in
  `CellGround`, folded once at construction. That pattern should spread before
  the layout work does.

**Two systems should not be forced into the pattern.** The holding map decides
ground by a distance test to each city rather than by a local stencil, and its
cost follows the candidates times the cities rather than the cells times six.
The observation field is a shadowcast with data-dependent branches. Reshaping
either one to fit a stencil would be the rewrite-for-its-own-sake failure the
brief warns against.

## 8. Does this buy depth, or only speed?

The brief asks this directly and asks for scepticism about the answer.

### 8.1 The budget, in coupling terms

At the target scale, one tile-pitch stencil pass over 16.7 million cells costs
about 2.7 milliseconds at 12 threads, derived. If the field layer may take 100
milliseconds of a frame, that is about **35 tile-pitch stencil passes for each
tick**, however they are divided.

**The margin spends part of that budget on cells nobody reads.** Section 5.3
gives the multiplier. At the target tile pitch it is 1.03, and at the target
coarse pitch it is 1.13. Apply it to every field that advects. A field that
relaxes from sources pays nothing, because it wants a clamped edge and not a
margin.

- Tile-pitch weather wants 35 passes, and the margin makes them cost 36.
  **The whole field budget buys exactly one deep field at tile resolution, and
  the margin does not change that answer.**
- A six-pass tile-pitch field costs about 16 milliseconds, or 17 with a margin,
  so the budget buys about **six such systems**.
- At the level 1 pitch a cell covers 1024 tiles, so the same budget buys about
  **thirty field systems** without a margin, or about **twenty-six** if every
  one of them advects and pays 13 percent.

**So the margin costs about one field system in thirty at the coarse pitch and
nothing that matters at the tile pitch.** It does not change the shape of the
answer, and the honest reason is that a margin is a perimeter cost on an area
budget. It only bites when the area is small.

**So the choice is not fast against slow. It is one field at tile resolution
against many fields at block resolution.**

### 8.2 Where the depth actually comes from

The depth this engine already has came from systems meeting each other, not
from any one system being deep. The climate shapes the ground. The moisture
shapes the regrowth. The wear couples the weather to the economy. The holding
couples armies to territory. Each of those is a coupling and none of them is a
deep model.

**Two systems couple through a shared resolution.** When they differ, they
couple through an average, and an average destroys the local structure that
made the coupling worth having. A rain shadow at the level 1 pitch is 32 tiles
wide. It cannot distinguish two farms, so it cannot make a farmer choose. At
one cell for each tile it can.

**So the layout buys depth, and it buys it through resolution alone.** That is
a real answer, and it is narrower than the premise.

### 8.3 Three reasons to doubt it

**First, a constant factor changes what is affordable and not what is
possible.** Twenty-five times cheaper is a large number and it is still a
constant. Nothing in this report changes the order of the cost, which stays
the cells times the passes.

**Second, the finickiness was never a cost problem.** The model is finicky
because its terms do not share a quantity. The saturation deficit fixes that,
and it would fix it on today's layout, at today's cost, at today's pitch. **If
the project may make one change, make that one.** A report that leads with the
layout would be flattering the machine at the expense of the model.

**Third, the field layer is not the bottleneck.** The measured frame is 836
milliseconds and 61.5 percent of it is one serial candidate pass inside the
holding spread.[^9] The influence solve, which is a real field solve at the
target scale, is 1.6 percent of the frame. **A field layer that is twenty-five
times cheaper moves a number that nobody is currently waiting on.** The layout
work earns its place by making a new resolution possible, not by removing a
cost anybody feels.

## 9. The migration path

### 9.1 What can be replaced in place, and proven

**Step one is a pure layout change and the golden hash proves it.** The
planes, the halo, the mask, the row-aligned bands and the shifted-load
neighbour step all preserve the arithmetic exactly. The mask reproduces
today's rule that an edge cell neither sends nor takes. **The state hash must
not change**, and if it does, the refactor is wrong and the test says so before
anything else moves.

This step touches no record, no register row and no test expectation. It
should land alone.

**Step two is the same change for the influence solve, the seeded field and
the approach field**, which have the same shape and the same neighbour chain.
Each is again hash-preserving.

### 9.2 What must go together

**The saturation deficit cannot land in halves.** It replaces the lift, the
fall, the back-lift, the fixed ceiling and the rain-out in one change, because
each of the five exists to compensate for the others. Landing two of the five
gives a field with neither the old balance nor the new one.

That change carries the following with it.

- **Four decision records.** Three are accepted and one is a draft. The
  accepted records state that water enters the air where it is hot and falls
  where the air cools, that water rides the wind and every transfer is an exact
  integer move, and that the wind is carried state. The transfer rule and the
  wind survive. The hot-and-cold rule is the one the deficit replaces, and it
  needs a superseding record rather than an edit.[^11] [^14] [^15]
- **About twenty rows of the balance register**, each naming a weather
  constant, its provisional default and its declaration site.[^12]
- **The golden state hash and every weather test expectation.** The tests
  number 42 today.
- **Every constant that is a stock**, which must gain a pitch derivation and a
  build-time assertion at the reference pitch.

### 9.3 What would break outside the module

- The viewer reads the saturation constant for its cloud shade, and reads the
  air, ground, wind and warmth planes for four overlays.
- The weather panel names a temperature, a wind speed and a heading.
- The gather resolve, the production moisture term and the upgrade wear pass
  read the wet mark.
- The Python control plane surfaces the same readings.

**The wet mark keeps its meaning and changes its value**, so those three
readers need no code change and do need a re-tuned constant. The saturation
constant stops being a constant, so the viewer must read a function of the
temperature of the cell it is painting. That is the one interface break.

### 9.4 The order

1. The margin, which a separate agent builds now. It changes the state hash and
   it is the defect the owner can see, so it goes first whatever else happens.
2. The layout refactor, hash-preserving, weather only. The margin becomes a
   parameter of the stride rather than a concept of its own.
3. The pitch classification of every constant, with build-time assertions.
   Hash-preserving at the reference pitch, and it fixes four defects at every
   other pitch. The margin width joins the classification as a distance.
4. The saturation deficit, with the four records and the balance rows.
5. The latitude sign on the turning term.
6. The layout refactor for the influence solve and the pyramid fields. Neither
   takes a margin, because neither advects.
7. A benchmark on the target platform, and the first weather rows in the
   target cost register.

**Steps two and three are worth doing whatever happens to the model.** Step
four is worth doing whatever happens to the layout. **Step one is worth doing
before either**, because a starved border makes every judgement about the model
unreliable, and every measurement in this report was taken on a field that had
one.

## 10. Is the rewrite worth it?

**Yes for the water terms. No for the air terms. And the layout is not a
rewrite at all.**

The argument for rewriting the water half is not that the current code is
untidy. It is that seven repairs found seven structural defects and the
seventh was still finding them. Five of the seven have one shape, and the
remedy for that shape is a derivation rather than a value. The water terms
have no quantity in common, and repair cannot create one. **A further repair
would produce an eighth defect of the same kind, and the history predicts it.**

The argument against rewriting the air half is the same history read the other
way. The season took one replacement and settled. The wind took three changes,
each measured better than the last, and the most recent one reports a rain
shadow, a travelling storm and a circulation. **Those terms are converging and
a rewrite would discard measurements that cost a long session to obtain.**

The layout is not a rewrite because it changes no answer. It is a refactor
with a proof, and the proof is a hash the project already computes every
frame.

**The margin is neither a rewrite nor a refactor. It is a missing term.** A
field that advects and holds no inflow at its edge is not a repaired model or
an unrepaired one. It is a model whose boundary condition was never stated. So
the margin does not belong on either side of this question, and it should land
before the question is asked again.

**What would make this report wrong.** A benchmark on the target that shows
the neighbour chain costing far less on Arm than the derived figure, which
would shrink the factor of twenty-five. A measurement of the memory bandwidth
of the target slice below about 15 gigabytes each second, which would make the
tile-pitch design bandwidth bound whatever the layout. Or a decision that the
weather is worth less than the coarse pitch already gives, which the open
blocker has not yet made.[^5]

## References

[^1]: ADR-0002, simulated and aggregated state holds no floating point number. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^2]: ADR-0008, the primary target is `aarch64-unknown-linux-gnu`, decisions D1, D2 and D3, and its consequences. `docs/adrs/accepted/adr-0008-the-primary-target-is-aarch64.md`
[^3]: ADR-0001, one binary gives one answer at any thread count. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^4]: Blockers register, BLK-007, most cost figures are still derived on the target platform. `docs/BLOCKERS.md`
[^5]: Blockers register, BLK-130, nobody has said what weather should be worth. `docs/BLOCKERS.md`
[^6]: The twelve weather commits and their bodies, from `c99e5bc` to `e13bc63` on the `integration` branch. Read with `git log main..integration -- crates/cachette-core/src/weather.rs`
[^7]: ADR-0017, the world is a rhombus, so a tile index is raw axial, decisions D1 and D2. `docs/adrs/accepted/adr-0017-the-world-is-a-rhombus-so-a-tile-index-is-raw-axial.md`
[^8]: Balance register, the lattice pitch row, which holds the four measured stage figures and the machine that produced them. `docs/reference/balance.md`
[^9]: Target platform costs, every stage of a frame at 16,777,216 tiles and 12 threads. `docs/reference/graviton-costs.md`
[^10]: Recurring Defect Shapes, shape 1, redundant declaration sites with undocumented precedence. `.agents/rules/recurring-defects.md`
[^11]: ADR-0162, water enters the air where it is hot, and it falls where the air cools. `docs/adrs/accepted/adr-0162-water-enters-the-air-where-it-is-hot-and-falls-where-the-air-cools.md`
[^12]: Balance register, the weather section. `docs/reference/balance.md`
[^13]: Arm Architecture Reference Manual for A-profile, the Advanced SIMD vector extract and vector shift instructions. https://developer.arm.com/documentation/ddi0487/latest
[^14]: ADR-0161, water rides the wind, and every transfer is an exact integer move. `docs/adrs/accepted/adr-0161-water-rides-the-wind-and-every-transfer-is-an-exact-integer-move.md`
[^15]: ADR-0160, the wind is carried state, and the pressure gradient accelerates it. `docs/adrs/accepted/adr-0160-the-wind-is-carried-state-and-the-pressure-gradient-accelerates-it.md`
[^16]: ADR-0087, an influence solve runs a fixed iteration count over the whole plane, decision D1. `docs/adrs/draft/adr-0087-an-influence-solve-runs-a-fixed-iteration-count.md`
[^17]: Findings register, FND-282, the division that turns a tile index into an address. `docs/FINDINGS.md`
[^18]: The pyramid module, the cell summary and the approach field neighbour table. `crates/cachette-core/src/pyramid.rs`
[^19]: The climate module, the cell climate record. `crates/cachette-core/src/climate.rs`
[^20]: Project orientation, hard invariant 9, widen pyramid accumulators at level 1. `CLAUDE.md`
[^21]: Report 01, ECS and memory layout, on 64-byte column alignment and on silent autovectorisation failure. `docs/research/reports/01-ecs-and-memory-layout.md`
[^22]: A read-only survey of every field system in the core crate, made for this report on 6 September 2026. `crates/cachette-core/src/`
[^23]: ADR-0068, terrain is generated from the seed and is never stored as a map. `docs/adrs/accepted/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
