# Demonstration Readability: Resources and Weather

Research report 24. It asks what a watcher can read about two subjects from a
still picture of the demonstration: the resources of the world, and the
weather. It ranks the ten defects that cost the most. Prepared 5 September
2026.

Cachette is a world simulation engine. The core is Rust. The control plane is
Python. The demonstration opens a window on a hex world of 256 tiles a side,
and a Rust module paints the picture.[^1] Two product records set the target.
One says that a watcher can see where the resources are, can see them being
taken, and can ask what a unit carries.[^2] The other says that a watcher can
see the weather on the map, and can tell it apart from the terrain beneath
it.[^3] This report checks the picture against both.

An earlier report reviewed the same window for factions, ground and units.[^4]
Two of its findings hold here and this report does not repeat them as new
work. The first is that the weather layers cover every tile at every zoom, so
they carry no information. The second is that a unit disc disappears below
about 16 pixels a tile. Section 6 refines the first against a measurement.

## 1. Method

One script builds the demonstration world with its default seed and four
factions, and steps it 300 ticks at four threads.[^5] It then finds the
faction whose founding place is nearest the middle of the map, takes a tile
that faction holds, and raises one storm there at the strength ceiling.[^6]
The storm raises 16 384 drops of water over one level 1 cell. The script steps
1, 20 and 100 more ticks, and writes pictures at each of the three stops.

Each stop draws three zooms of the same place: the region, which fits the
whole world to a 960 by 720 window at 2.5 pixels a tile; the city at 12 pixels
a tile; and the close-up at 64 pixels a tile, which shows about 15 tiles
across. The script also writes one picture with the colour key on, one with
the weather panel and the tile panel on, and a picture of the world at tick
300 before the storm, for a direct comparison.

The script then finds the tile with the most gather events on the last step,
and draws it at the three zooms with the tile panel. It draws a luxury tile
and a settlement in the same way. A second script reads the air and the ground
water of every level 1 cell at each stop, counts the pixels that the storm
changed, and samples tile colours. A third script samples the generated stock
of 1500 random tiles. A fourth script maps each named tile to its pixel
rectangle under each camera, so that every region below is measured and not
estimated.

The pictures are not committed. The scripts write them under a build
directory, and the commit body names each file. Every pixel region below is
given as `x, y` from the top left of the named picture.

## 2. Weather, by zoom

### 2.1 The region, 2.5 pixels a tile

Pictures `storm_p0_region.png`, `storm_p1_region_bare.png`,
`storm_p20_region.png` and `storm_p100_region.png`.

A watcher sees that a storm happened. The first two pictures are one tick
apart and differ over the whole northwest of the map. The second is pale and
hazy where the first is saturated.

A watcher cannot see where the storm fell. The storm fell on one cell, and
that cell occupies `240, 2` to `356, 78`. The change covers 248 576 pixels of
the 691 200 in the picture, which is about 27 times the area of the cell. The
water spread to six cells in one tick, and the field carries it further each
tick. A watcher reads a change of palette over a third of the map, not a place
where something happened.

The footprint is a set of axis-aligned rectangles. The overlay answers each
tile from the cell that covers it, so every tile of one cell takes one weight,
and the boundary between two cells is a straight line.[^7] The world is a
sheared rhombus, so those straight lines cut across the coast and the terrain
at an angle that means nothing. A seam of this kind runs across the top left of
`storm_p1_region_bare.png`.

A watcher cannot see the storm fade. It does not fade; it goes out. The air
over the storm cell is 2525 drops one tick after the storm and 62 drops twenty
ticks after it. The overlay saturates at 4096 drops and covers a tile at a
weight of 150 of 255 there, so 2525 drops give a weight of 92 and 62 drops give
a weight of 2.[^8] The picture at plus 20 and the picture at plus 100 are the
same picture, and neither differs from the picture before the storm.

A watcher cannot see that the ground is wet. Every cell of this world is wet at
every tick of the run, before the storm and after it. The wet mark is 64 drops,
and the ground water of the driest cell is 124 drops before the storm and 251
drops a hundred ticks after it.[^9] A layer that is on everywhere is a constant.

### 2.2 The city, 12 pixels a tile

Pictures `storm_p1_city.png`, `storm_p20_city.png`, `storm_p100_city.png` and
`storm_p20_city_panel.png`.

A watcher sees a storm as a wash over the whole window. The storm cell covers
`184, 164` to `756, 544` at this zoom, which is most of the picture. The
picture at plus 1 is pale over that whole area and saturated outside it, so a
watcher sees one straight boundary and no shape.

A watcher cannot read the strength of the storm from the picture. The panel
states 4872 drops in the air and 785 drops on the ground of the pointed cell,
and the map at that moment shows nothing at all.

### 2.3 The close-up, 64 pixels a tile

Pictures `storm_p0_close.png`, `storm_p1_close.png`, `storm_p20_close.png` and
`storm_p1_close_panel.png`.

A watcher cannot see the storm. The whole window lies inside one level 1 cell,
because a cell is 32 tiles a side and the window shows about 15. Every tile
therefore takes the same overlay weight. The tile at the middle of the window
reads `687837` before the storm and `90a07b` one tick after it. That is a large
change and it is the same change on every tile, so the picture has no edge, no
centre and no gradient.

A watcher who holds only the second picture cannot tell weather from a second
palette. Nothing in the frame is unwashed, so there is no reference. This is
the statement of the product record that the picture fails: the record asks
that a watcher see the condition and tell it apart from the terrain beneath
it.[^3]

The panel states what the map does not. It names the pointed tile, the air over
its cell, the water on its ground and whether the ground is wet.

## 3. Resources, by zoom

### 3.1 The region, 2.5 pixels a tile

Pictures `deposit_region.png`, `luxury_region.png` and `city_region.png`.

A watcher cannot see where a deposit is, or what kind it is, or how rich it is.
The ground colour carries one resource, which is food, and it carries it as a
brightness step.[^10] A deposit of wood or of stone changes no pixel. The
deposit tile of this run sits at `600, 246`, one pixel, and it holds six of six
stone.

A watcher cannot see a luxury. The luxury tile sits at `470, 156`, and the mark
is one pink pixel among the ground speckle.

A watcher cannot see a settlement. Four settlements stand, and the map draws no
mark for any of them. The only mark near a seat is the founding ring, which
records the founding and not the settlement.[^11] The panel states the store of
each site, so the store is legible and the place is not.

### 3.2 The city, 12 pixels a tile

Pictures `deposit_city.png` and `luxury_city.png`.

A watcher cannot see a deposit or how rich it is, for the reason above.

A watcher can find a luxury only when told where to look. The mark is a pink
square of one third of a tile, so it is four pixels across at this zoom, at
`472, 354` to `486, 364`. The picture holds 65 unit discs of a similar size in
the same area, and the mark reads as one more of them.

### 3.3 The close-up, 64 pixels a tile

Pictures `deposit_close.png`, `deposit_close_panel.png`, `luxury_close.png`
and `city_close.png`.

A watcher cannot see a deposit. The deposit tile occupies `432, 328` to
`526, 390` and draws as an ochre hill like every hill beside it. The panel says
that it holds six of six stone and none of one wood. Two units stand on it and
took wood from it on the last step, and no pixel says so.

A watcher cannot see that a tile is being worked. The picture shows two green
discs. A disc that gathered on the last step and a disc that walked past draw
alike. The drawing pass reads what each unit carries and counts it for a card,
and it paints no mark on the unit.[^12]

A watcher sees a luxury and cannot tell which one. The mark is a solid pink
square of 21 pixels by 21 pixels at `469, 349` to `489, 369`. This world holds eight luxury tiles and
eight different kinds, and the table admits 64 kinds. All 64 draw in one
colour.[^13]

A watcher cannot see a settlement. The seat tile occupies `432, 328` to
`526, 390` and draws as a forest tile inside a founding ring. The economy panel
states a store of 200.00 for that site and 925.00 for another. The map states
neither.

## 4. Colours that collide

- **The wet shade against the food shade.** The wet layer takes 34 from every
  channel. The food layer adds up to 34 to every channel.[^10] [^14] The two
  are the same size and opposite in sign, so a wet tile with the most food
  draws as the same colour as a dry tile with none, except where a channel
  clamps. Every cell of this world is wet at every tick, so the food layer is
  shifted down by its own full range at all times.
- **The wet shade against forest and against water.** Forest is `1d4a2b` and
  darkens to `002809`, so the red channel clamps at zero and the hue moves. The
  wet layer also darkens open water, which no ground rule makes wet.
- **The air tint against the holder tint.** The holder tint covers a tile at a
  weight of 96 of 255. The air overlay at the storm covers it at a weight of
  92 of 255, in a pale blue-white.[^8] [^15] The faction contribution falls to
  about a quarter of the final pixel, so a stormed holding loses its colour.
  The pale northwest of `storm_p1_region_bare.png` shows this.
- **The luxury mark against a unit disc.** The mark is a square of one third of
  the tile at the tile centre. A unit disc has a radius of three tenths of the
  tile width at the same centre, so it is wider than the mark. The unit pass
  runs after the tile pass, so a unit standing on a luxury covers the mark
  completely.[^12] No picture in this run shows it, because no unit stood on a
  luxury tile. The cause is the order of the two passes and the geometry, not
  an observation.
- **The luxury pink against the sixth faction colour.** The mark is `ff5ad2`
  and the sixth faction is a near pink. This run has four factions, so no
  picture shows the collision.

## 5. What the product records ask for and the picture does not give

The resource record asks that a watcher see where the resources are, see them
being taken, and see that two units cannot take one unit of resource.[^2] The
picture gives none of the three. It carries food as a brightness step and
carries wood and stone not at all. It marks no gather. It marks no unit that
carries a load, so the exclusion rule has nothing a watcher can check.

The weather record asks that a watcher see the condition on the map and tell it
apart from the terrain beneath it.[^3] The picture gives a wash whose edges are
the cell lattice and whose extent is far larger than the storm. At the close-up
it gives a wash with no edge at all.

## 6. A refinement of the earlier report

The earlier report says that the air overlay and the wet shade together cover
the whole picture and cost contrast.[^4] The measurement splits the two. Before
any storm, the air over a cell runs from 54 to 156 drops, which is an overlay
weight of 1 to 5 of 255. The air layer is therefore near absent at rest, and it
appears only for a few ticks after a storm. The constant pale wash that the
earlier report names comes from the wet shade, which takes 34 from every
channel of every tile of the world at every tick. The adjustment the earlier
report proposes for the air layer is sound and does not reach the constant.

## 7. The ten defects that cost the most

Each row names the picture, the region, the cause the reviewer believes, and
one adjustment. The cause cites the painting function where the reviewer found
it.

1. **A storm covers the close-up window with one flat wash.** Picture
   `storm_p1_close.png`, the whole window, against `storm_p0_close.png`. The
   overlay reads one air value for the cell that covers each tile, and a cell
   is wider than the close-up window.[^7] [^8] Adjust: interpolate the air
   between the four nearest cell centres, so a close-up shows a gradient rather
   than a step.

2. **The storm leaves the picture in 20 ticks and leaves no trace.** Pictures
   `storm_p20_close.png` and `storm_p100_region.png`, the whole window. The air
   falls from 2525 to 62 drops, and the ground water that the storm added is
   drawn only as a binary wet mark.[^9] [^14] Adjust: draw the ground water as
   a ramp in place of the binary shade, so the place a storm fell stays legible
   after the air clears.

3. **The wet shade cancels the food shade exactly.** Every picture, the whole
   ground. The wet layer takes 34 from each channel and the full food ramp adds
   34 to each channel.[^10] [^14] Adjust: give the wet layer a blue channel
   gain rather than a brightness cut, so the two layers move different axes.

4. **A storm erases the faction that owns the ground.** Picture
   `storm_p1_region_bare.png`, region `240, 0` to `600, 300`, against
   `storm_p0_region.png`. The air overlay covers a tile at a weight near the
   holder weight, in a colour no faction uses.[^8] [^15] Adjust: apply the air
   overlay to the ground colour before the holder mix, so the holder tint keeps
   its share of the pixel.

5. **Nothing says a deposit is there, or what kind, or how rich.** Picture
   `deposit_close.png`, region `432, 328` to `526, 390`, a tile of six of six
   stone drawn as plain hill. The tile colour takes food alone.[^10] Adjust:
   paint one small pip in a corner of the tile for each resource the ground
   gave, coloured by kind and sized by the amount, from 16 pixels a tile
   upward.

6. **Nothing says a tile is being worked or a unit is carrying.** Pictures
   `deposit_close.png` and `deposit_close_panel.png`, the two discs centred at
   `480, 360` and `479, 488`. The unit pass reads the load of each unit, counts
   it for a card and paints no mark.[^12] Adjust: draw a rim on a disc that
   holds a load, in the colour of the kind it holds, and outline a tile that a
   gather event named on the last step.

7. **One colour stands for every kind of luxury.** Pictures
   `luxury_close.png`, region `469, 349` to `489, 369`, and `luxury_city.png`, region
   `472, 354` to `486, 364`. The mark is one constant, and the table admits 64
   kinds.[^13] Adjust: derive the mark hue from the kind ordinal, and add a row
   to the colour key for each kind that the window holds.

8. **A settlement has no mark of its own.** Pictures `city_close.png`, region
   `432, 328` to `526, 390`, and `city_region.png`, `504, 216`. The drawing
   passes paint ground, holder, upgrade, luxury, air and units, and none of
   them paints a site. The only mark near a seat is the founding ring, which
   records a past event.[^11] Adjust: paint the seat tile with a solid block in
   the faction colour and a dark core, and scale a bar beside it with the store
   the economy panel already reads.

9. **The food ramp and the height ramp share one brightness.** Picture
   `deposit_close.png`, the whole ground. The two are added into one shade, and
   the height range is 56 steps against 34 for food.[^10] The food ramp also
   saturates at eight units while the ground generates up to twelve, and 83 in
   100 tiles carry no food at all. Adjust: put food on the saturation of the
   tile colour and leave height on the brightness.

10. **The storm footprint is a set of rectangles on a sheared map.** Picture
    `storm_p1_region_bare.png`, the seam across region `200, 55` to `430, 95`.
    Each cell takes one weight, so the boundary between two cells is a straight
    line that follows nothing in the world.[^7] Adjust: as for defect 1, and
    stop drawing the overlay below a weight of about 8 of 255, so a cell at
    rest adds no edge.

Two defects rank below these. The wet shade darkens open water, which no rule
makes wet. The colour key names factions and ground kinds only, so a watcher
who sees a pink square or a pale wash has no row to look it up in; the earlier
report already asks for the missing rows.[^4]

One control plane defect blocks a watcher outside the window. The Python site
reader takes a settlement identity, and the demonstration has no verb that
gives one for a world the seeding built, so every slot index from 0 to 3 is
refused. A watcher who wants a store must open the economy panel.[^16]

## 8. What a still picture cannot judge

The reviewer could not judge motion. Whether a wash that arrives in one tick
reads as weather arriving, whether a watcher follows the edge of a storm across
cells, and whether a deposit that drains reads as draining, are questions for a
recording. The reviewer could not judge a storm on a dry world, because this
world is wet in every cell from early in the run, and the dry note of the
weather panel therefore never appears. The reviewer could not judge a crowded
gather, because the whole world logged two gather events on the step that was
drawn. The reviewer could not judge the collision between the luxury mark and a
unit disc, because no unit stood on a luxury tile in this run.

## References

[^1]: The painting module, module documentation, `draw` and `draw_soldiers`. `crates/cachette-view/src/paint.rs`
[^2]: PRD-0007, the world holds things worth taking. `docs/product/accepted/prd-0007-the-world-holds-things-worth-taking.md`
[^3]: PRD-0004, the world has weather that a watcher can read. `docs/product/accepted/prd-0004-the-world-has-weather-that-a-watcher-can-read.md`
[^4]: Research report 23, demonstration readability review 1. `docs/research/reports/23-demonstration-readability-review-1.md`
[^5]: The demonstration, `build_world` and `_write_picture`. `python/cachette/demo/app.py`
[^6]: The weather verbs and the ceilings, `inflict_weather` and `weather_strength_ceiling`. `python/cachette/_core.pyi`
[^7]: ADR-0140, weather is a field over the level 1 cell lattice, decision D1. `docs/adrs/draft/adr-0140-weather-is-a-field-over-the-level-1-cell-lattice.md`
[^8]: The painting module, `air_weight`, `AIR_AT_FULL_SHADE` and `AIR_WEIGHT_CEILING`. `crates/cachette-view/src/paint.rs`
[^9]: The weather module, `WET_MARK`. `crates/cachette-core/src/weather.rs`
[^10]: The painting module, `tile_colour`, `HEIGHT_STEPS`, `FOOD_STEPS` and `FOOD_AT_FULL_SHADE`. `crates/cachette-view/src/paint.rs`
[^11]: The painting module, `mark_foundings`. `crates/cachette-view/src/paint.rs`
[^12]: The painting module, `draw_soldiers`, the carry read and the disc radius. `crates/cachette-view/src/paint.rs`
[^13]: The painting module, `mark_luxury` and `LUXURY_MARK`, and the luxury ceiling. `crates/cachette-view/src/paint.rs`, `python/cachette/_core.pyi`
[^14]: The painting module, `darkened` and `WET_SHADE`. `crates/cachette-view/src/paint.rs`
[^15]: The painting module, `AIR_COLOUR` and `HOLDER_WEIGHT`. `crates/cachette-view/src/paint.rs`
[^16]: The site reader, `site_economy`, and the economy panel. `python/cachette/_core.pyi`, `crates/cachette-view/src/panel/economy.rs`
