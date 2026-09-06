# Demonstration Readability: Upgrades and Unit Positioning

Research report 25. It asks what a watcher can read from a still picture of the
demonstration about two subjects: the upgrades that units build, and where a
unit stands. It ranks the ten defects that cost the most. Prepared 5 September
2026.

Cachette is a world simulation engine. The core is Rust. The control plane is
Python. The demonstration opens a window on a hex world of 256 tiles a side,
and a Rust module paints the picture.[^1] Three product records set the target.
One says that a watcher can see who holds a tile and where one holding meets
another.[^2] One says that a watcher can see an improvement on the map, can
tell it from terrain, and can see the work in progress.[^3] One says that the
window names every colour it draws.[^4] This report checks the picture against
all three.

Two earlier reports reviewed the same window.[^5] [^6] Their findings are not
repeated here as new work. Six of them hold and this report builds on them: a
unit disc disappears below about 16 pixels a tile and carries no rim; nothing
shows where a unit is going; the wet shade cancels the food shade; a settlement
has no mark beyond the founding ring; the faction 1 blue reads as water; and
the cards cover the map. Two new upgrade kinds landed after those reports, so
the upgrade layer is judged here for the first time.

## 1. Method

One script builds the demonstration world with its default seed, an extent of
256 tiles a side and four factions, and seeds it with the engine's own seeding
call.[^7] It steps to tick 300 and later to tick 1500, at four threads. At each
named place it draws three zooms into a 960 by 720 window: the region at 3
pixels a tile, the city at 12 pixels a tile, and the close-up at 64 pixels a
tile, which shows about 15 tiles across. Some places are drawn again with the
tile panel and the event panel, and three with the colour key.

The engine's own controllers build. No verb of the script ordered a build, and
the 90 sites near the seat of faction 0 at tick 300 are the world's own.

Three cases do not occur in the run, so the script makes them.

- No unit of the run stands on ground that nobody holds. The script puts one
  unit of faction 1 on an unheld tile.
- No tile holds more units than it admits. The script puts ten more units of
  faction 1 on a tile of faction 3 that already stood at its capacity of eight.
- A cohort sent toward a neighbour's seat did not arrive in 60 ticks, and no
  unit of the sender stood on the neighbour's ground.[^8] The script puts four
  units of faction 1 on a tile deep inside the holding of faction 3.

The fight needed two changes. Every unit of the run carries unit type 0, whose
attack is zero, so two cohorts of the run stand together and nothing happens.
The script spawns eight units of faction 1 and eight of faction 3 on
neighbouring tiles, gives both the one table row that can fight, and sets the
pair to war in both directions. A contest then resolves.

A second script decodes each written PNG, maps each named tile to its pixel
rectangle from the camera the picture used, and samples the colour at the
middle of the tile. Every colour and every region below is measured in the
named picture, not estimated. Every pixel region is given as `x, y` from the
top left.

The pictures are not committed. The script writes them under a build directory,
and the commit body names each file.

## 2. Upgrades

### 2.1 The close-up, 64 pixels a tile

Pictures `t300-upgrades-close.png`, `t300-upgrades-panel-close.png`,
`t300-upgrades-reference-close.png` and `t1500-upgrades-close.png`.

A watcher cannot tell a build site from bare ground. The tint of a site runs
from a floor of 56 of 255 at the start of the work to a ceiling of 200 at the
end.[^9] A store two work units into its 48 draws at weight 62. The tile at
`480, 360` of `t300-upgrades-close.png` is such a store, and it measures
`6a4d33`. The bare held forest beside it measures `664e34`. The two differ by 4
in one channel and by 1 in the other two. The mark is not weak. It is absent.

The four kinds separate unevenly at the same early progress. Over held forest
the road at 1 of 8 measures `82633a`, the terrace at 1 of 24 measures `606740`,
and the wonder at 8 of 240 measures `846f4d`. The road and the wonder read as
lighter ground, the terrace as greener ground, and the store as the same
ground. None of the four reads as a thing that somebody made.

The kind with the most work is the palest for the longest. A wonder needed 240
work units when this report ran, so it sat under weight 62 for the first 5 of
its work and the whole of that time it looked like bare ground.

The project owner raised the wonder work on 5 September 2026, and the register
holds the new value.[^21] The raise makes this defect worse and not better,
because a larger work value keeps the site pale for more ticks. The colours
above are a reading of one run and this report does not repeat them.

A watcher cannot tell a finished site from an unfinished one, except at the two
ends. A finished road measures `b28946` against `664e34` for the bare tile
beside it, which is clear. Nothing between the floor and the ceiling is
readable as a fraction. The weight is the only carrier of progress, and a
watcher cannot read a fraction from one flat colour without a reference beside
it.

A finished road hides the ground it stands on. Region `600, 0` to `960, 720` of
`t1500-upgrades-close.png` is 163 finished roads and reads as one flat ochre
field. The ground under the field is forest, and the colour key of the same
picture says the window holds no hill. A watcher reads hill.

The tile panel says nothing about the upgrade. The panel names the address, the
ground, the three stocks, the capacity, the holder and the units.[^10] It has
no row for the upgrade, although the engine answers the kind, the progress and
the completion at one address.[^11] In `t300-upgrades-panel-close.png` the
panel describes a tile that carries a store at 2 of 48 and says `ground forest`
and nothing else about it. In `t1500-upgrades-panel-close.png` the panel
describes a tile that carries a finished road and reports `it admits 16`. The
capacity is the only trace of the road anywhere in the window, and a watcher
must know that forest admits 8 to read it.

The colour key names no upgrade. The card in `t300-upgrades-reference-close.png`
holds four faction rows and five ground rows.[^12] A watcher who sees an ochre
tile has no row to look it up in, and the four new colours are the four that
most need one, because each of them is near a ground colour.

### 2.2 The city, 12 pixels a tile

Pictures `t300-upgrades-city.png` and `t1500-upgrades-city.png`.

A watcher sees terraces and nothing else. The terrace green stands out as
patches at region `290, 370` to `430, 450` of `t300-upgrades-city.png`, because
its colour is far from the ochre of the ground under it. Roads, wonders and
stores at this tick are invisible.

The whole holding changes colour between the two ticks, and that is the only
sign that 163 roads were built. In `t1500-upgrades-city.png` the holding of
faction 0 is one ochre wash from `0, 0` to `860, 620`. The wash is the roads.
It cannot be told from the holder tint of faction 0 over hill, and it cannot be
told from hill.

A watcher cannot tell where one built tile ends and the next begins. Two
neighbouring finished roads are the same colour, so the only separator is the
one-pixel gap that every pair of tiles has.

### 2.3 The region, 3 pixels a tile

Pictures `t300-upgrades-region.png` and `t1500-upgrades-region.png`.

Nothing about an upgrade survives at this scale. The road field of faction 0
appears as a slightly warmer patch inside the red edge line, region `370, 300`
to `620, 450` of `t1500-upgrades-region.png`, and it is not distinguishable
from the hill ground of the same holding.

### 2.4 What the run showed about progress

A site that nobody works on draws the same as a site under work. A window of 21
tiles a side round the seat of faction 0 held 45 stores and wonders at tick 300.
All 45 held the same progress at tick 1500, and no store and no wonder in the
window ever finished. The picture carries the progress and never carries the
rate, so a watcher reads an abandoned site and a busy site alike.

## 3. Unit positioning

### 3.1 A unit on its own ground, on foreign ground, and on unheld ground

Pictures `t300-lone-close.png`, `trespass-close.png` and `trespass-city.png`.

Nothing on a unit says whose ground it stands on. The disc carries the faction
colour and one condition dot.[^13] A watcher must compare the colour of the
disc against the tint of the tile under it, and the tint is 96 of 255 of the
same table.[^14] The comparison is hardest for exactly the unit that matters:
a unit standing on its own faction's ground is a disc of one colour on a wash
of that colour.

At the close-up the three cases separate. In `t300-lone-close.png` the disc at
`608, 488` stands on grey mountain with no tint and no edge line, and the disc
at `480, 360` stands on a tile with a blue tint and a blue outline. A watcher
can tell held ground from unheld ground here, because the ground is grey and
the tint shows on it.

A unit on foreign ground is a state the world does not hold for a whole frame.
Four units of faction 1 put on a tile that faction 3 held took the tile in one
step: the tile reported holder 3 before the step and holder 1 after it. The
engine's own guest reader agreed and reported no guest.[^15] The picture
therefore cannot show a trespass, because presence is a claim. In
`trespass-close.png` the result is a one-tile blue island at `480, 360` inside a
yellow holding, with six yellow-outlined neighbours around it. That reads well.
In `trespass-city.png` the same island is one blue speck at `480, 357` in a
yellow field, and a watcher does not find it.

### 3.2 The border

Pictures `t300-border-close.png`, `t300-border-city.png` and
`t300-border-region.png`.

The border works at the close-up, and it works for one reason: the two holdings
carry outlines of different colours, drawn at 230 of 255.[^14] In
`t300-border-close.png` the tiles of faction 1 left of `480, 360` are teal with
a blue outline, and the tiles of faction 3 right of `510, 360` are olive with a
gold outline. A watcher reads the line between them.

The two edges together take two tiles of width. At the city zoom that is 24
pixels and it still reads, region `330, 0` to `500, 360` of
`t300-border-city.png`. At the region zoom it is 6 pixels and the two chains
merge into one line of mixed colour.

The edge does not say which side is inside. Every tile of a holding that meets
anything different takes the outline, and unheld ground counts as
different.[^16] A holding one tile wide is therefore all outline, and a
scattered holding reads as a mesh rather than as a region. The module states
this limit for the frontier against the coastline, and the same cause gives
this one.

The border of faction 1 reads as a coast at every zoom. A forest tile that
faction 1 holds measures `26646c` in `t300-border-close.png`. That is the tint
of the blue faction over dark green forest, and it is the colour of shallow
water. No faction holds water in this run; the tint alone does it. The earlier
report reported the blue against open water.[^5] This is the same colour under
a different cause: the tint turns a whole holding into sea.

### 3.3 A crowd

Pictures `t300-crowd-close.png`, `t300-crowd-city.png`,
`t300-crowd-panel-close.png`, `t300-overfull-close.png` and
`t300-overfull-panel-close.png`.

**Eight units on one tile draw one disc.** Every unit of a tile takes the
centre of that tile, so the discs of a crowd land on each other exactly.[^13]
The tile at `480, 360` of `t300-crowd-close.png` holds eight units and shows
one disc of the ordinary size. There is no count, no ring, no growth and no
spread. The picture of eight is the picture of one.

A tile standing exactly at its capacity carries no mark. The drawing counts it
and reports the count to the panel, and it draws a mark only when the count
passes the capacity.[^17] The tile above admits eight and holds eight.

**A tile that two factions share shows one of them.** The over-full tile at
`480, 360` of `t300-overfull-close.png` holds twelve units: eight of faction 3
and four of faction 1. The picture draws one blue disc. The units arrive in the
order the bridge holds, and the last disc covers every earlier one, so the
colour of a shared tile is the colour of whichever unit the structure happens to
put last. Nothing in the picture says the tile is shared.

The over-capacity outline works. The same tile carries a red outline that is
clear at 64 pixels and at 12 pixels. It is the one mark in this subject that
does its job.

The tile panel is the only place a crowd can be read. For the same tile it says
`it admits 8`, `units here 12`, and it lists six units with their factions:
`f3 fed`, `f3 fed`, `f1 fed`, `f1 fed`.[^10] A watcher who has not opened the
panel sees one blue disc.

### 3.4 A fight

Pictures `fight-t0-close.png`, `fight-t0-panel-close.png`,
`fight-t1-close.png`, `fight-t1-panel-close.png` and `fight-t8-close.png`.

Nothing marks a fight. In `fight-t0-close.png` one blue disc at `480, 360` and
one yellow disc at `544, 360` stand on neighbouring tiles across a border. The
two factions are at war and the contest pass will resolve between them on the
next step. The picture is the picture of two units standing still.

**The fight is over in one tick, and the frame after it is empty.** In
`fight-t1-close.png`, at the same place one step later, both discs are gone.
Sixteen units stood there and none remains. The ground carries no mark, the
holding did not move, and `fight-t8-close.png` eight ticks later is the same
picture. A watcher cannot tell who was fighting, and cannot tell who lost,
because the picture at the only moment that holds the answer shows nothing at
all.

**The panel that exists to say what happened says nothing happened.** In
`fight-t1-panel-close.png` the heading WHAT HAPPENED carries the line
`nothing happened this tick`, in the same tick in which the contest killed
units on the tile that the tile panel names. The panel reads four of the
engine's logs: the units a shortage ended, the sites rationed, the sites short
of upkeep, and the units promoted.[^18] The fallen log is a fifth, the engine
writes it, and Python already reads it.[^19] One measured contest between two
groups of eight wrote seven rows to it in one step, three of one faction and
four of the other.

Nothing says two factions are at war. The relation is a signed number for each
ordered pair, and no layer of the picture reads it. Two cohorts at peace and
two cohorts at war are the same picture.

## 4. Colours and marks that collide

Every pair below is measured in a named picture or computed from the two
tables the module holds.[^9] [^14] [^20]

- A store early in its work against the ground under it. `6a4d33` against
  `664e34` on held forest. The two are one colour.
- A store over held forest against bare hill. The store tile computes to
  `6c5033` and the hill colour is `6e5a30`. A build site is the colour of
  another ground kind.
- The road ochre `c89a4a` against the hill ochre `6e5a30` and against the
  faction 3 yellow `e8c84a`. Three ochres in one range, and a finished road
  field cannot be told from hill ground or from a yellow holding.
- The terrace green `52b86a` against the faction 2 green `6ac46a` and the
  plain green `556b2a`. A terrace inside the holding of faction 2 has no colour
  of its own left.
- The wonder gold `f0e0a0` against the shortage dot `f2f0d8` and against the
  air overlay `d8e8f8`. All three are pale washes.
- The faction 1 blue over forest, `26646c`, against water. A holding reads as
  sea.
- A disc against the tint of its own faction's ground. The two are the same
  colour at two weights.
- Two factions on one tile. The colours do not blend; the later one wins
  outright.

## 5. The ten defects that cost the most

Each row names the picture, the region, the cause the reviewer believes, and
one adjustment.

1. **A build site is the same colour as the ground it stands on.** Picture
   `t300-upgrades-close.png`, `480, 360`, a store at 2 of 48 measuring `6a4d33`
   against `664e34` beside it. The weight runs from a floor of 56 of 255, and a
   weight that low over a tinted tile changes nothing a person can see.[^9]
   Adjust: draw a site as a glyph rather than a wash. Put a small shape in the
   middle of the tile, one shape for each kind, in the kind colour at full
   weight with a dark rim, and keep the wash for the finished state only.

2. **The tile panel says nothing about the upgrade.** Pictures
   `t300-upgrades-panel-close.png` and `t1500-upgrades-panel-close.png`, the
   panel at `660, 14` to `915, 140`. The panel builds its rows from the ground,
   the stocks, the capacity, the holder and the units, and never asks for the
   upgrade, although one call answers all three of its fields.[^10] [^11]
   Adjust: add two rows, `building` naming the kind, and `work done` giving the
   progress against the work the kind asks for.

3. **A fight leaves the picture with no mark, and the event panel says nothing
   happened.** Pictures `fight-t1-close.png`, the whole window, and
   `fight-t1-panel-close.png`, the heading at `678, 14`. Sixteen units fell on
   the two tiles at `480, 360` and `544, 360`, and the panel reads four logs
   that do not include the fallen log.[^18] [^19] Adjust: read the fallen log
   into a fifth section of the panel, and outline every tile named in that log
   for the frame, in one colour that no faction uses.

4. **A crowd of eight draws as one unit.** Picture `t300-crowd-close.png`,
   `480, 360`, eight units and one disc. Every unit of a tile takes the centre
   of the tile.[^13] Adjust: draw the count as a badge on the disc from 24
   pixels a tile upward, and below that grow the disc radius with the count so
   that a full tile is visibly fuller than a single unit.

5. **A tile that two factions share shows one faction.** Picture
   `t300-overfull-close.png`, `480, 360`, twelve units of two factions and one
   blue disc. The pass paints in the order the bridge gives and the last disc
   covers the rest.[^13] Adjust: when a tile's run holds more than one faction,
   split the tile into wedges, one for each faction present, and draw the disc
   from the largest.

6. **A watcher cannot tell an abandoned site from one under work.** Pictures
   `t300-upgrades-close.png` and `t1500-upgrades-close.png`, the wonder at
   `512, 424`, whose progress did not move in 1200 ticks. The weight carries
   the progress and nothing carries the rate.[^9] Adjust: draw a site whose
   progress rose on the last step with a bright rim, and leave a site that
   nobody worked on plain.

7. **The colour key names no upgrade.** Picture
   `t300-upgrades-reference-close.png`, the card at `14, 550` to `200, 690`.
   The key builds rows from the faction table and the ground table
   only.[^12] Adjust: add one row for each of the four upgrade kinds, drawn as
   the mark the map draws, and state the count of each in the window. The
   earlier report asked for the mark rows; these four are new since it.[^5]

8. **A holding of the blue faction reads as sea at every zoom.** Pictures
   `t300-border-close.png`, `480, 360`, measuring `26646c`, and
   `t300-border-city.png`, region `0, 300` to `470, 700`. The tint of the blue
   faction over dark green forest lands on the colour of shallow water.[^14]
   Adjust: as the earlier report asks, change the second faction colour; and
   pick the replacement against every ground colour mixed at the holder weight,
   not against the ground colours alone.[^5]

9. **A tile at its capacity carries no mark.** Picture
   `t300-crowd-close.png`, `480, 360`, eight units on a tile that admits eight.
   The drawing marks a tile only when the count passes the capacity.[^17]
   Adjust: draw the outline in a second colour when the count equals the
   capacity, so a watcher sees a tile that is full before it sees one that is
   over-full.

10. **A one-tile holding inside another vanishes above the close-up.** Picture
    `trespass-city.png`, `480, 357`, against `trespass-close.png`,
    `480, 360`. The mark of a holding is a tint and a one-pixel outline, and
    both scale with the tile.[^14] [^16] Adjust: hold the outline at two pixels
    below 16 pixels a tile, and draw the outline of a tile whose holder differs
    from all six of its neighbours at a floor of five pixels a side, so a
    single foreign tile is always findable.

Two defects rank below these. The edge line does not say which side of it is
inside a holding, which the module already states as a limit.[^16] The
demonstration cannot show a fight without help, because every unit of the run
carries the unit type whose attack is zero, so a reader who opens the
demonstration and waits will never see a contest.

## 6. What this review could not judge

The reviewer could not judge motion. Whether a build site that deepens over
ticks reads as work, whether a crowd that gathers reads as gathering, and
whether the one frame of a fight is enough at a real frame rate, are questions
for a recording.

The reviewer could not judge a finished store or a finished wonder, because
neither kind finished anywhere in a 1500-tick run.

The reviewer could not judge a natural fight, a natural trespass, a natural
over-full tile or a natural unit on unheld ground, because the run produced
none of the four. Each was made, and section 1 says how.

The reviewer could not judge the store through the control plane. The site
reader refuses every small number, as the earlier report found.[^6] It refuses
zero as not an identity, and it refuses one, two and three because the
identity they name carries generation 0 while the arena holds generation 1
there. No verb of the demonstration gives a settlement identity for a world
that the seeding built.

## References

[^1]: The painting module, module documentation. `crates/cachette-view/src/paint.rs`
[^2]: PRD-0006, a place belongs to somebody. `docs/product/accepted/prd-0006-a-place-belongs-to-somebody.md`
[^3]: PRD-0008, a unit changes the ground it stands on. `docs/product/accepted/prd-0008-a-unit-changes-the-ground-it-stands-on.md`
[^4]: PRD-0005, a watcher can tell what is happening and why. `docs/product/shipped/prd-0005-a-watcher-can-tell-what-is-happening-and-why.md`
[^5]: Research report 23, demonstration readability review 1. `docs/research/reports/23-demonstration-readability-review-1.md`
[^6]: Research report 24, demonstration readability, resources and weather. `docs/research/reports/24-demonstration-readability-resources-and-weather.md`
[^7]: The demonstration, `build_world` and `Demo.seed`. `python/cachette/demo/app.py`
[^8]: The sending verb, `send_units_to`, which sends a set toward a place and does not promise that the set arrives. `python/cachette/_core.pyi`
[^9]: The painting module, `upgrade_weight`, `UPGRADE_COLOURS`, `UPGRADE_WEIGHT_FLOOR` and `UPGRADE_WEIGHT_CEILING`. `crates/cachette-view/src/paint.rs`
[^10]: The tile panel, `lines`. `crates/cachette-view/src/panel/inspector.rs`
[^11]: The tile reader, `tile_report`, and its upgrade, upgrade progress and upgrade complete entries. `python/cachette/_core.pyi`
[^12]: The glass module, `colour_card`. `crates/cachette-view/src/glass.rs`
[^13]: The painting module, `draw_soldiers`, the disc centre and the disc radius. `crates/cachette-view/src/paint.rs`
[^14]: The painting module, `draw`, `HOLDER_WEIGHT` and `EDGE_WEIGHT`. `crates/cachette-view/src/paint.rs`
[^15]: The guest reader, `stands_in_territory`. `python/cachette/_core.pyi`
[^16]: The painting module, `on_an_edge`, and backlog item 0209. `crates/cachette-view/src/paint.rs`, `docs/backlog/proposed/0209-tell-a-frontier-from-the-edge-of-the-claimed-ground.md`
[^17]: The painting module, `close_run` and `OVER_CAPACITY`. `crates/cachette-view/src/paint.rs`
[^18]: The event panel, module documentation and `lines`. `crates/cachette-view/src/panel/events.rs`
[^19]: The fallen log, `World::fell_log` and `fell_log_columns`. `crates/cachette-core/src/world.rs`, `python/cachette/_core.pyi`
[^20]: The painting module, `KIND_COLOURS` and `FACTION_COLOURS`. `crates/cachette-view/src/paint.rs`
[^21]: Balance register, the wonder work. `docs/reference/balance.md`
