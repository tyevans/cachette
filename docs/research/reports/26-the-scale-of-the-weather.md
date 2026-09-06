# The Scale of the Weather

Research report 26. It measures the weather field of the demonstration world
and says why the weather reads as a haze over the whole map instead of as a
storm that a watcher can follow. Prepared 5 September 2026.

Cachette is a world simulation engine. The core is Rust. The control plane is
Python. The engine holds weather as two quantities of water over a lattice of
level 1 cells: the water in the air above a cell, and the water on the ground
of that cell. Both are whole numbers of drops. A record fixes the lattice.[^1]
A second record fixes the transfer rule, which moves water and never scales
it.[^2] A third bounds the divine power that raises a storm.[^3] A fourth makes
wet ground yield more to a gatherer.[^4]

The project owner read the demonstration on 5 September 2026 and said that the
weather is "kinda foggy in areas". The owner wants to follow a pattern across
the map over many ticks, and wants one part of the map to hold a storm while
another part holds none.

**Every figure below is a development-machine measurement.** The machine is
`ty001-ubuntu`, an x86-64 Linux host with a 12th Gen Intel Core i7-1260P and 16
logical processors. The engine targets a different platform, and one blocker
governs every cost figure in this project.[^5] No figure below is a cost. Each
is a quantity of water, a count of cells or a share of terrain, so the machine
changes none of them. The machine is named because the rule asks for it.

Three scratch scripts produced the figures. Each builds the demonstration world
with its default seed and four factions, seeds it, and steps it at four
threads. The scripts are not committed, and the commit body holds each one.

## 1. How large a cell is, and how many there are

The block edge of the level 1 lattice is 32 tiles, so one cell covers 1024
tiles. The demonstration world is 256 tiles by 256 tiles, which is 65 536
tiles. The lattice is therefore 8 cells by 8 cells, and the whole world holds
64 cells.

**Sixty-four cells is the whole resolution of the weather.** The close-up
camera of the demonstration shows about 15 tiles across, so the whole close-up
window lies inside one cell and every tile in it takes one value.[^6]

At the target scale of 16.7 million tiles the same block edge gives about
16 300 cells. The demonstration is small, and the lattice is small because the
world is small.

## 2. What the world holds when nobody raises a storm

The first script steps the world 1000 ticks and raises no storm.

| Reading | Tick 300 | Tick 1000 |
|---|---|---|
| Water in the air, whole world | 5002 | 4325 |
| Water on the ground, whole world | 18 523 | 19 173 |
| Water counted as evaporated | 152 603 | 570 678 |
| Water ever raised | 176 128 | 594 176 |
| Cells whose ground is wet | 64 of 64 | 64 of 64 |

The distribution of the ground water over the 64 cells, in drops:

| Reading | Tick 300 | Tick 1000 |
|---|---|---|
| Lowest cell | 124 | 124 |
| Lower quartile | 241 | 247 |
| Median | 277 | 285 |
| Upper quartile | 333 | 344 |
| Highest cell | 465 | 496 |

The distribution of the air over the 64 cells, in drops:

| Reading | Tick 300 | Tick 1000 |
|---|---|---|
| Lowest cell | 54 | 44 |
| Median | 70 | 62 |
| Highest cell | 156 | 214 |

**The wet mark is 64 drops, and every cell is above it at every tick of a 1000
tick run.**[^7] The driest cell holds 124 drops, which is 1.9 times the mark.
The median cell holds more than 4 times the mark. A second script counted the
cells above other marks at tick 400: 63 of 64 cells hold at least 128 drops, 32
hold at least 256, 18 hold at least 300, 4 hold at least 400, and none holds
512.

**The wetness of a cell therefore carries no information.** It is on
everywhere, so a reader that asks whether a cell is wet reads a constant, and
the rule that gives a gatherer more on wet ground gives every gatherer more
everywhere.[^4]

**The ground water itself does carry information.** It runs from 124 to 465
drops, which is a spread of nearly four to one over a still map. The value that
hides that spread is the mark, not the field.

## 3. Where the standing water comes from

**Nothing outside the engine puts it there.** The solve lifts it. Each cell
takes one keyed draw each frame. The draw is taken below the tile count of the
cell multiplied by a fixed period, and the cell lifts a fixed quantity when the
draw falls below the number of tiles of that cell that hold open water.[^8] So
the odds that a cell lifts follow the water share of that cell, and a cell with
no water never lifts.

Measured over 100 ticks from tick 300: the world raises 547.8 drops a tick on
average, from 2.14 lifts a tick, with a highest tick of 6 lifts. The quantity
one lift raises is 256 drops.

The ground rests where the arrivals and the drying balance. The settle pass
takes one thirty-second of the ground water out of every cell in every solve,
so the resting ground of the whole world is near 32 times the arrival rate.
The measured ground total of 18 500 to 19 200 drops over 64 cells agrees with
that: it is 290 to 300 drops for each cell, against a wet mark of 64.

**So a calm world saturates itself.** The sea lifts water, the spread carries
it everywhere within a few ticks, the fall puts it on the ground, and the
drying is slow enough that every cell settles far above the wet mark.

## 4. What one storm does

The second script steps the world 300 ticks, finds the held tile nearest the
middle of the map, and raises one storm there at the strength ceiling. The
storm put 16 384 drops into the air over one cell, which is cell row 2 and cell
column 4. The world held 5002 drops in the air over all 64 cells before it.

The air over the whole world, and the highest cell, at each stop:

| Tick after the storm | Air, whole world | Highest cell | Where the highest cell is |
|---|---|---|---|
| Before | 5002 | 156 | row 7, column 7 |
| 1 | 19 659 | 1288 | row 2, column 4 |
| 2 | 18 591 | 672 | row 2, column 4 |
| 3 | 16 757 | 502 | row 0, column 5 |
| 5 | 13 100 | 330 | row 0, column 6 |
| 10 | 8786 | 196 | row 6, column 0 |
| 20 | 4879 | 140 | row 7, column 7 |
| 100 | 4917 | 152 | row 6, column 0 |

The air of every cell one tick after the storm, in drops, by cell row:

```text
  53    51    73   218   541   737   541   231
  51    70   195   572   996   997   572   217
  55   126   439   989  1288   979   433   133
  68   197   594  1004  1001   578   190    72
  97   243   482   615   468   221    87    60
 129   208   288   254   169   102    71    72
 146   162   159   133   103    86    82    92
 132   132   123   110    96    78    86   112
```

The same, five ticks after the storm:

```text
 130   156   199   247   293   323   330   326
 150   189   218   265   300   314   307   290
 172   196   237   271   288   287   266   238
 174   195   238   254   261   245   198   186
 187   216   231   232   219   198   155   128
 202   225   225   204   180   153   129   107
 211   209   205   171   147   126   104    98
 188   182   169   151   128    99    90    88
```

**The maximum does not travel. It flattens in place.** The highest cell is the
storm cell for two ticks. It then stops being the storm cell, and where it goes
is not a direction: at tick 3 it is in the top row, at tick 10 it is in the
bottom left, and at tick 20 and tick 100 it is back at the resting maximum of
the calm world. The field between those ticks is a dome that spreads and sinks.

**One tick spreads the storm over a third of the map.** The picture at plus one
tick shows raised air over about 20 of the 64 cells, and the raise reaches the
opposite corner within a few ticks. The storm cell falls to 52 per cent of its
first value in one tick and to 39 per cent in two.

**The storm is gone in 20 ticks.** The air of the whole world is 4879 drops at
plus 20, against 5002 before the storm. Nothing distinguishes the two.

The ground records the storm a little longer. The highest cell runs from 465
drops before the storm to 615 at plus 5 and 631 at plus 20, and it is back to
450 at plus 100. The lowest cell moves from 124 to 177 and back to 116. Every
cell is above the wet mark at every one of those stops, so the binary wet
reader answers the same at all of them.

## 5. The terrain a model could read

The third script reads the level 1 summary of every cell. The share of each
cell that is open water runs from 0 to 100 per cent, and 12 of the 64 cells
hold no water at all while 2 are almost entirely water. The mean height of a cell runs
from 11 to 83 per cent of the height range.

**The terrain has strong contrast and the water field has none.** Any rule that
derives a quantity per cell from the height and the water share of that cell
has a varied input to work from. The flatness measured in section 2 comes from
the transport and from the mark, not from the world.

## 6. The three candidate causes, ranked

**First, the pass has no direction, so nothing travels.** The spread is a
symmetric gather: a cell hands each neighbour the same share whatever lies
around it.[^2] Section 4 measures the consequence directly. A maximum injected
at one cell stays at that cell for two ticks, halves each tick, and is
indistinguishable from the calm world by tick 20. A watcher cannot trace a
pattern across the map, because no pattern moves across the map. **No balance
value fixes this.** A share is a scalar, and no choice of scalar makes a
symmetric kernel carry a maximum in a direction. This needs a decision record.

**Second, the world holds far more water than the wet mark asks for.** Section
2 measures 64 of 64 cells wet at every tick of a 1000 tick run, with the driest
cell at 1.9 times the mark. **A balance value fixes this**, and no record is
needed for it. Raising the mark toward the median, or lowering what one lift
raises, separates the map at once, because the ground water already varies
nearly fourfold from cell to cell. That fix alone would give the owner one
half of what was asked: distinct parts of the map, wet and dry. It would not
give the other half, because those parts would stand still.

**Third, the lattice is coarse.** One cell is 1024 tiles, the demonstration
world holds 64 cells, and the close-up window lies inside one cell. This is
real and it is third, because a finer lattice over a field that is uniformly
wet reads exactly the same as a coarse one. The measurement does not support
changing the lattice record, and this report recommends leaving it as it
stands.[^1] Section 4 also shows that the coarseness is not what smears the
storm: the storm covers a third of the lattice in one tick, and that is the
transfer share against a small lattice rather than the size of one cell.

**The interpolation that an earlier report proposes is a drawing matter, not a
fix.** That report asks the viewer to interpolate the air between the four
nearest cell centres, so that a close-up shows a gradient rather than a
step.[^6] The proposal is sound for what it is. It changes no simulated value,
so it cannot make a storm travel and it cannot make one part of the map wet
while another is dry. It smooths the picture of a field that has nothing in it.

## 7. What the measurements say about a larger model

The project owner asked for a flow field of wind, heat and water rather than a
wind term added to diffusion. Three measurements bear on whether that is
buildable on this lattice.

**The inputs exist and are derived.** Section 5 shows that the height and the
water share of each cell already vary strongly, and the level 1 summary already
carries both as exact integer totals. A heat value per cell is a function of
those two, so it needs no storage of its own and it cannot drift from the
world.

**The source is already stochastic, so a steady wind still gives a moving
picture.** Section 3 measures 2.14 lifts a tick over 64 cells, each of a fixed
quantity, each decided by a keyed draw. Parcels of water therefore enter at
scattered places and scattered times. A wind field that is steady in time still
carries those parcels across the map, so a watcher sees a pattern that moves
even before anything makes the wind itself vary.

**The transport share must fall, whatever else changes.** Section 4 shows one
storm reaching a third of the lattice in one tick under the present share and
pass count. A directional transport at that rate would carry a front off the
map before a watcher saw it. The share and the pass count are values, and they
belong in the balance register rather than in a record.

## References

[^1]: ADR-0140, weather is a field over the level 1 cell lattice, decisions D1 and D3. `docs/adrs/draft/adr-0140-weather-is-a-field-over-the-level-1-cell-lattice.md`
[^2]: ADR-0141, a weather pass moves water and never scales it, decisions D1 and D3. `docs/adrs/draft/adr-0141-a-weather-pass-moves-water-and-never-scales-it.md`
[^3]: ADR-0142, a god inflicts weather only on ground its own faction holds, decisions D1 and D2. `docs/adrs/draft/adr-0142-a-god-inflicts-weather-only-on-ground-it-holds.md`
[^4]: ADR-0143, wet ground yields more to a gatherer, decisions D1 and D2. `docs/adrs/draft/adr-0143-wet-ground-yields-more-to-a-gatherer.md`
[^5]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^6]: Research report 24, demonstration readability, resources and weather, sections 2.3 and 7. `docs/research/reports/24-demonstration-readability-resources-and-weather.md`
[^7]: The weather module, `WET_MARK`. `crates/cachette-core/src/weather.rs`
[^8]: The weather module, `cell_lifts`, `LIFT_DROPS` and `LIFT_PERIOD`. `crates/cachette-core/src/weather.rs`
