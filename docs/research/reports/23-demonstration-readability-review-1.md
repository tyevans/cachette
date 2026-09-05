# Demonstration Readability Review 1

Research report 23. It asks what a watcher can read from a still picture of the
demonstration, at four zoom levels and at two ticks. It ranks the ten defects
that cost the most. Prepared 5 September 2026.

Cachette is a world simulation engine. The core is Rust. The control plane is
Python. The demonstration opens a window on a hex world, and a Rust module
paints the picture.[^1] A person who watches the window must tell one faction
from another, held ground from unheld ground, a unit and where it goes, and a
place a faction founded.[^2] [^3] [^4] This report checks each of those against
the picture the code paints today.

## 1. Method

One script builds the demonstration world with its default seed, an extent of
256 tiles a side and four factions.[^5] It steps the world to tick 300 and to
tick 1500. At each tick it draws four pictures: the whole world fitted to a 960
by 720 window, a region after two zoom presses from the fitted view, a city at
48 pixels a tile centred on the seat of the first founding, and a close-up at
64 pixels a tile in a 512 by 384 window. At tick 300 it also draws the city
with the panel and with the colour key.

Two zoom presses from the fitted view move the tile from 2.5 pixels to 3.0
pixels, and the picture still shows nine tenths of the world. The zoom step is
a factor of 1.1 for each press.[^6] The region pictures at that scale add
nothing that the fitted pictures do not show, so the script draws one more
region at 8 pixels a tile, centred on the same seat. The review reads that
picture as the region.

The demonstration world at these ticks has no upgrade near the seat, and the
control plane offers no reader for the upgrade table, so no picture here shows
a road or a terrace. The upgrade layer is not judged. Eight tiles hold a
luxury. None sits near the seat, so the script draws two more pictures centred
on the first luxury tile, at 8 and at 64 pixels a tile.

The review reads each picture with an image reader, and samples pixel values
from the PNG where the eye alone could not settle a question. Every pixel
region below is given as `x, y` from the top left, in the picture named.

The pictures are not committed. The script writes them under a build
directory, and the commit body names each file.

## 2. What the pictures show

### 2.1 The whole world, 2.5 pixels a tile

Pictures `t300-1-world.png` and `t1500-1-world.png`.

A watcher tells the four factions apart. Red, blue, green and yellow each hold
one region, and each region carries an edge line in its colour. The edge is
the strongest mark in the picture, and it works at this scale.

A watcher tells held ground from unheld ground by the edge alone. Inside the
edge, the tint over the ground is weak. The yellow holding, region `430, 90`
to `900, 640`, reads as slightly warmer plain and hill. The blue holding, region
`160, 100` to `400, 380`, reads as shallow water, because faction 1 is blue and
water is blue. A watcher who does not know the palette places the coast in the
wrong place on the west side.

A watcher cannot see a unit. At this scale a unit disc has a radius of one
pixel.[^7] A fed unit is one pixel of its faction colour over ground already
tinted in that colour, and it vanishes. At tick 300 the short units show as
white pixels, region `530, 20` to `660, 140`, because the shortage dot also has
a radius of one pixel and covers the whole disc.[^7] At tick 1500 nothing is
short, and the picture shows no unit at all. The world card says 246 people
are in the world, and the picture shows none of them.

A watcher cannot find a city. The founding mark is a square ring seven pixels
on a side.[^8] It sits at `645, 75` for faction 0, at `290, 288` for faction 1,
at `700, 247` for faction 2 and at `505, 214` for faction 3. Each is a speck the
size of a unit and the same shape as the tile grid. A watcher who does not know
where to look does not find it.

A watcher cannot see weather, wet ground or a luxury. The world at both ticks
has water in the air over every tile and wet ground in every cell, so the air
overlay and the wet shade cover the whole picture evenly.[^9] A layer that
covers everything carries no information, and it costs contrast. The whole
ground reads as pale and hazy. A luxury mark at this scale is one pink pixel
and is not distinguishable from the ground speckle.

The ground inside a holding reads as speckle. Each tile takes a brightness
step from its height and a second step from its food stock, and a plain tile
beside a forest tile changes hue.[^10] At 2.5 pixels a tile those steps make
noise, region `700, 180` to `780, 300` in the green holding.

The cards are legible. THE WORLD card mixes a world count and a window count.
Its row `short` reads 64 in the fitted view, 59 in the region and 2 in the
city, so it counts the window, but it sits under `people in world` with no
label that says so.[^11]

### 2.2 The region, 8 pixels a tile

Pictures `t300-2b-region8.png`, `t1500-2b-region8.png` and
`t300-luxury-region8.png`.

A watcher tells the factions apart, again by the edge line. The red holding
edge, region `230, 130` to `720, 700`, is clear. The interior tint is weaker
than the edge, and a sampled held hill tile reads `9e753b` against unheld hill
near `6e5a30` after the air overlay. The tint moves the hue a little toward red.
A watcher reads the edge, not the fill.

The one-pixel gap between tiles dominates. At 8 pixels a tile the gap takes
about a quarter of the area, and the picture reads as a brick wall.[^12] The
rows step by half a tile, so the bricks stagger, and the eye reads the stagger
before it reads the ground.

A watcher cannot see a fed unit. At tick 1500 the picture holds 55 units. A
unit disc has a radius of two pixels in the faction colour, over ground
tinted in the same colour.[^7] Region `280, 200` to `520, 480` of the tick 1500
picture holds units and shows none that a reader can point at. At tick 300 the
short units show as white dots with a one-pixel red rim, region `240, 300` to
`450, 440`. In the luxury region picture the yellow units of faction 3 on
yellow-tinted ground, region `280, 350` to `520, 500`, are specks.

A watcher finds the city only by the ring. The ring is 16 pixels on a side at
`472, 352` in the seat pictures and at `585, 545` in the luxury picture. The
seat tile under it looks like every tile beside it. No mark says which tile is
the site, and no mark says how many people are housed there.

The luxury mark at this scale is a two-pixel pink square at `480, 360` of the
luxury picture. It does not show.

The edge line does not tell a frontier from a coastline. The red edge runs
along every coast, region `470, 130` to `720, 700`, and along the border with
unheld ground alike. The module states this limit in its own text.[^13]

### 2.3 The city, 48 pixels a tile

Pictures `t300-3-city.png`, `t1500-3-city.png`, `t300-3-city-reference.png`
and `t300-3-city-panel.png`.

A watcher tells the ground kinds apart. Forest is dark green, plain is light
green, hill is ochre, and water is deep blue. Height and food make small
brightness steps inside a kind, and at this scale they read as relief rather
than as noise.

A watcher sees a unit, and tells its faction. A disc of radius 14 pixels in
red on green plain, at `193, 550` of the tick 300 picture, is clear. The
shortage dot inside it is clear. At tick 1500 seven red discs, region `60, 20`
to `520, 470`, are clear.

A watcher cannot tell where a unit is going. The disc has no heading, no
trail and no line to its target.[^7] The nearest-unit card says `chose deliver
1.00`, so the engine knows the intent, and the picture does not show it.

A watcher cannot tell held ground from unheld ground inside the holding. The
tint at weight 96 of 255 over hill gives brown, and unheld hill gives ochre.
Region `240, 0` to `720, 300` is held and reads as brown ground. The red edge
appears only along the coast, region `480, 0` to `940, 640`, because every
neighbour inland is also held. A watcher sees a red coast and no other sign of
ownership.

A watcher finds the city only because the camera is on it. The ring is 96
pixels on a side at `432, 312` to `528, 408`, and its line is three pixels
wide: red, dark, red. It reads as a selection cursor, not as a place. The seat
tile under it is a hill tile like its neighbours. No house, no store, no
people.

The cards cover the world. Three cards fill the top left, region `14, 14` to
`272, 300`, and cover about one tenth of the map. The hint bar at the foot
covers a unit at `120, 690` of the tick 300 picture. The colour key, when on,
adds a fourth card at bottom left, region `14, 550` to `200, 690`, and covers
the unit at `193, 550`.

The colour key names each faction and each ground kind with a swatch. It does
not name the shortage dot, the founding ring, the edge line, the luxury mark or
the over-capacity mark. A watcher who sees a white dot in a red disc has no row
to look it up in.

The panel picture is legible at 2121 pixels tall. Every number is labelled.
The panel names the tile under the crosshair and the region under the
crosshair, and the map draws no crosshair. A reader cannot tell which tile the
panel means.

### 2.4 The close-up, 64 pixels a tile

Pictures `t300-4-close.png`, `t1500-4-close.png` and `t300-luxury-close.png`.

The cards cover close to half the map, region `14, 14` to `272, 300` of a 512 by
384 window. In the tick 300 picture the CHARACTERS card covers the top of the
founding ring. Pixel samples at `250, 128` read card colour and not ring
colour.

A unit is clear. Its faction is clear. Its heading is absent, as at every
scale.

The luxury mark is a pink square 21 pixels on a side at `245, 180` of the
luxury picture. It is the only pink thing in the picture, and nothing names it.
The colour key has no row for it. A watcher reads it as a unit of a fifth
faction.

Nothing at this scale says what a tile is beyond its colour. A tile at 64
pixels has room for a glyph or a short word, and the picture uses none of it.
The panel knows the ground, the stock, the holder and the capacity of the
tile under the crosshair, and the map shows none of that on the tile.

## 3. Colours that collide

- Faction 1 blue `45a0e8` against water `123c5e`. A blue holding beside water
  reads as shallow water at the fitted and region scales.
- Faction 2 green `6ac46a` against plain `556b2a`. A green holding over plain
  reads as brighter plain. The terrace upgrade colour `52b86a` is a third green
  in the same range and would collide with both, when a terrace appears.
- Faction 3 yellow `e8c84a` against hill `6e5a30`. A yellow holding over hill
  reads as warmer hill. The road upgrade colour `c89a4a` is a fourth ochre in
  the same range.
- A unit disc against its own holder tint. Every faction colour collides with
  the ground its own faction holds, because the tint is the same colour at
  weight 96 and the disc is the same colour at full weight. At two pixels of
  radius the disc vanishes.
- The shortage dot `f2f0d8` against the air overlay `d8e8f8`. Both are pale,
  and at one pixel of radius the dot is the whole unit.
- The luxury mark `ff5ad2` against faction 5 pink `e88fc4`, when six factions
  play. In this run no faction has that colour.

## 4. The ten defects that cost the most

Each row names the picture, the pixel region, the cause the reviewer believes,
and one adjustment. The cause cites the painting function where the reviewer
found it.

1. **A fed unit is invisible below about 16 pixels a tile.** Picture
   `t1500-2b-region8.png`, region `280, 200` to `520, 480`, 55 units drawn and
   none visible. The disc radius is three tenths of the tile width, in the
   faction colour, over ground tinted in the same colour.[^7] Adjust: give every
   disc a one-pixel dark rim, and hold the radius at a floor of three pixels
   whatever the zoom.

2. **The weather layer covers every tile and so shows nothing.** Pictures
   `t300-1-world.png` and `t1500-1-world.png`, the whole ground. Every cell is
   wet and every tile has air over it, so the wet shade and the air overlay
   apply everywhere.[^9] Adjust: switch the air overlay off below a tile size
   of about 8 pixels, and scale the overlay from the mean air over the window
   rather than from zero, so only the tiles above the mean take it.

3. **Nothing shows where a unit is going.** Every picture with a unit, for
   example `t1500-3-city.png`, region `60, 20` to `520, 470`. The unit pass
   paints a disc and a condition dot and nothing else.[^7] Adjust: paint a
   short line from the disc centre toward the next tile the unit will step to,
   in the disc colour, from 24 pixels a tile upward.

4. **A city is a ring and nothing else.** Pictures `t300-3-city.png`, region
   `432, 312` to `528, 408`, and `t300-1-world.png`, `645, 75`. The founding
   mark is a square ring with a three-pixel line.[^8] The seat tile draws like
   its neighbours. Adjust: fill the seat tile with a solid block of the faction
   colour at full weight with a dark core, sized to a floor of nine pixels, and
   write the faction number beside it from 16 pixels a tile upward.

5. **The cards hide the world at every zoom.** Picture `t300-4-close.png`,
   region `14, 14` to `272, 300`, about half the map. The cards are anchored to
   the top left at a fixed pixel size, and the hint bar to the foot.[^11]
   Adjust: shorten THE WORLD, WHAT THEY CARRY and THE CHARACTERS to one card of
   four rows while the colour key is off, and move the hint into the card
   heading.

6. **The tile gap turns the region scale into a brick wall.** Picture
   `t300-2b-region8.png`, whole map. A one-pixel gap draws under every tile of
   3.5 pixels or wider.[^12] Adjust: draw the gap only from 16 pixels a tile
   upward, and let the colour change separate tiles below that.

7. **Faction 1 blue reads as water.** Picture `t300-1-world.png`, region
   `160, 100` to `400, 380`. The faction table holds blue `45a0e8` and water
   is `123c5e`.[^14] Adjust: change the second faction colour to an orange or a
   magenta that no ground kind uses.

8. **The world card counts the window without saying so.** Pictures
   `t300-1-world.png`, `t300-2b-region8.png` and `t300-3-city.png`, card
   THE WORLD, row `short`. The row reads the count of short units the drawing
   pass painted, under a row labelled `in world`.[^11] Adjust: label the row
   `short in window`.

9. **The colour key names only factions and ground.** Picture
   `t300-3-city-reference.png`, region `14, 550` to `200, 690`. The key lists
   the faction table and the kind table and no mark.[^15] Adjust: add one row
   each for the shortage dot, the founding ring, the edge line, the luxury
   mark and the over-capacity mark, with the mark drawn as its swatch.

10. **The founding ring has no size floor a watcher can find.** Picture
    `t300-1-world.png`, `645, 75`, a seven-pixel square. The side is twice the
    tile width with a floor of seven pixels.[^8] Adjust: raise the floor to 15
    pixels, and draw the ring as a filled disc with a dark rim below 8 pixels a
    tile, so it is the one round thing on a square grid.

Two more defects rank below these. The panel names a crosshair that the map
does not draw, so the tile rows have no visible referent. The zoom step of 1.1
means two presses from the fitted view change the picture very little, and a
watcher who wants a region presses fifteen times.[^6]

## 5. What a still picture cannot judge

The reviewer could not judge motion. Whether a watcher can follow one unit
across ticks, whether a disc that jumps one tile a tick reads as movement, and
whether the edge line flickers as tiles change hands, are questions for a
recording. The reviewer could not judge the cost cards, because the numbers
depend on the machine. The reviewer could not judge the upgrade layer, because
no upgrade stood in any picture and the control plane exposes no reader to
find one. The reviewer could not judge the over-capacity mark, because no
drawn tile was at capacity.

## References

[^1]: The painting module, module documentation. `crates/cachette-view/src/paint.rs`
[^2]: PRD-0002, a developer watches the world run. `docs/product/shipped/prd-0002-a-developer-watches-the-world-run.md`
[^3]: PRD-0003, a developer sees a world worth looking at. `docs/product/accepted/prd-0003-a-developer-sees-a-world-worth-looking-at.md`
[^4]: PRD-0005, a watcher can tell what is happening and why. `docs/product/shipped/prd-0005-a-watcher-can-tell-what-is-happening-and-why.md`
[^5]: The demonstration, `build_world` and `_write_picture`. `python/cachette/demo/app.py`
[^6]: The painting module, `ZOOM_STEP` and `Camera::zoomed`. `crates/cachette-view/src/paint.rs`
[^7]: The painting module, `draw` and `draw_soldiers`, the radius and the shortage dot. `crates/cachette-view/src/paint.rs`
[^8]: The painting module, `mark_foundings` and `FOUNDING_LEAST_SIDE`. `crates/cachette-view/src/paint.rs`
[^9]: The painting module, `draw`, the wet shade and the air overlay, and `World.weather_totals` in the stub. `crates/cachette-view/src/paint.rs`, `python/cachette/_core.pyi`
[^10]: The painting module, `tile_colour`, `HEIGHT_STEPS` and `FOOD_STEPS`. `crates/cachette-view/src/paint.rs`
[^11]: The glass module, `world_card`, `load_card`, `unit_card` and `hint`. `crates/cachette-view/src/glass.rs`
[^12]: The painting module, `gap_for` and `TILE_GAP`. `crates/cachette-view/src/paint.rs`
[^13]: The painting module, module documentation, the holder layer, and backlog item 0209. `crates/cachette-view/src/paint.rs`, `docs/backlog/proposed/0209-tell-a-frontier-from-the-edge-of-the-claimed-ground.md`
[^14]: The painting module, `FACTION_COLOURS` and `KIND_COLOURS`. `crates/cachette-view/src/paint.rs`
[^15]: The glass module, `colour_card`. `crates/cachette-view/src/glass.rs`
