//! Turns a world into pixels.
//!
//! This module is where floating point begins. Rendering sits outside
//! simulated state, so the arithmetic here is free.[^1] No value that has
//! been a floating point number is ever handed back to the engine.[^2]
//!
//! The world is a rhombus in the index space, so it is a parallelogram on
//! the screen. The skew belongs here. The engine holds no screen
//! position.[^3]
//!
//! # The holder layer
//!
//! A tile that a faction holds takes that faction's colour, mixed over the
//! ground. A tile that nobody holds draws as the ground alone. A held tile
//! whose neighbour has another holder takes a border in the same colour, so
//! a watcher sees where one holding meets another.
//!
//! The colour comes from the one table this module holds. The engine holds no
//! colour, and a second table would be one fact in two places.[^2] [^4]
//!
//! # References
//!
//! [^1]: ADR-0002, simulated and aggregated state holds no floating point number, decision D4. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
//! [^2]: ADR-0067, the viewer reads the world and never writes to it, decision D3. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
//! [^3]: ADR-0017, the world is a rhombus, so a tile index is raw axial, decision D4. `docs/adrs/accepted/adr-0017-the-world-is-a-rhombus-so-a-tile-index-is-raw-axial.md`
//! [^4]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`

use cachette_core::hex::NEIGHBOURS;
use cachette_core::terrain::{Terrain, TileKind, KIND_COUNT};
use cachette_core::{Axial, BridgeError, FactionId, Holder, World};

use crate::text;

/// The colour of the space outside the world.
const BACKGROUND: u32 = 0x0010_1418;

/// One colour for each kind of ground, in the order of the kinds.
///
/// The palette is the viewer's own. The engine says what a tile is and never
/// what it looks like, so a colour has no place in it.[^1] [^2] A later
/// contributor may choose another palette freely: a palette is a property of
/// the picture, and no record binds it.
///
/// The five colours are far enough apart that a person can name each one
/// against the background, and a test asserts that they stay apart.
///
/// # References
///
/// [^1]: ADR-0067, the viewer reads the world and never writes to it, decision D2. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
/// [^2]: ADR-0068, terrain is generated from the seed and is never stored as a map, decision D4. `docs/adrs/accepted/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
const KIND_COLOURS: [u32; KIND_COUNT] = [
    // Water. Deep blue, and the only kind that admits no unit.
    0x0012_3c5e,
    // Plain. Open green, leaning yellow.
    0x0055_6b2a,
    // Forest. Deep green, leaning blue, so that the height shading can never
    // brighten a forest tile into the colour of a plain one. The shading
    // moves the brightness of a tile and never its hue, so two kinds are
    // told apart by hue alone.
    0x001d_4a2b,
    // Hill. Dry ochre.
    0x006e_5a30,
    // Mountain. Bare grey.
    0x0070_7478,
];

/// The number of brightness steps that the height gives a tile.
///
/// The height is a fraction of the full range, and the viewer maps that
/// fraction onto this many steps. The steps are added to each channel, so a
/// tall tile of one kind is brighter than a short tile of the same kind.
const HEIGHT_STEPS: i32 = 56;

/// The number of brightness steps that the tile value gives a tile.
///
/// The ground is fixed for the life of a world, so a picture of the ground
/// alone never moves. The simulated value ripples on top of it, so a person
/// watching a still camera still sees the world step.[^1] The range is small
/// against the height range, so the ripple never hides the ground.
///
/// # References
///
/// [^1]: PRD-0002, a developer watches the world run. `docs/product/shaped/prd-0002-a-developer-watches-the-world-run.md`
const VALUE_STEPS: i32 = 14;

/// One colour for each faction, and one spare.
///
/// A faction is a bit index below the ceiling, and the ceiling is larger than
/// this table. A faction beyond the table wraps to a colour it shares, which
/// is a display limit and not a simulation one.
const FACTION_COLOURS: [u32; 6] = [
    0x00e8_5d4a,
    0x0045_a0e8,
    0x006a_c46a,
    0x00e8_c84a,
    0x00b5_6ae8,
    0x00e8_8fc4,
];

/// The number of colours the viewer can tell apart.
///
/// The legend shows one row for each of them. A faction beyond the table
/// shares a colour, so the legend says so rather than showing a count it
/// cannot separate.
pub const COLOURED_FACTIONS: usize = FACTION_COLOURS.len();

/// How much of the holder's colour covers the ground it holds.
///
/// The ground stays legible under the holding, because a watcher must read
/// the kind of ground and the holder of it at once. The weight is a property
/// of the picture. No record binds it, and a later contributor may change it
/// freely.
const HOLDER_WEIGHT: u8 = 96;

/// How much of the holder's colour covers the edge of a holding.
///
/// The edge is nearly the pure colour, because the edge is what the product
/// record asks a watcher to see.[^1]
///
/// # References
///
/// [^1]: PRD-0006, a place belongs to somebody. `docs/product/shaped/prd-0006-a-place-belongs-to-somebody.md`
const EDGE_WEIGHT: u8 = 230;

/// Returns the colour the viewer draws a faction in.
///
/// The colour is the viewer's own. The engine holds no colour and never
/// will.[^1]
///
/// # References
///
/// [^1]: ADR-0067, the viewer reads the world and never writes to it, decision D2. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
#[must_use]
pub fn faction_colour(faction: FactionId) -> u32 {
    FACTION_COLOURS[colour_slot(faction)]
}

/// Returns the colour the viewer marks an over-filled tile in.
///
/// The mark says that a tile holds more units than its ground admits. The
/// colour is the viewer's own, and the engine holds none.[^1]
///
/// A test reads this rather than a literal, so the mark has one declaration
/// site.[^2]
///
/// # References
///
/// [^1]: ADR-0067, the viewer reads the world and never writes to it, decision D2. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
/// [^2]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
#[must_use]
pub const fn over_capacity_colour() -> u32 {
    OVER_CAPACITY
}

/// Returns the index of the colour a faction shares.
fn colour_slot(faction: FactionId) -> usize {
    (faction.0 as usize) % COLOURED_FACTIONS
}

/// The smallest tile size the viewer will show, in pixels.
const MIN_TILE: f32 = 2.0;

/// The largest tile size the viewer will show, in pixels.
const MAX_TILE: f32 = 64.0;

/// The tile size the viewer opens with, in pixels.
const OPENING_TILE: f32 = 12.0;

/// The factor one zoom press applies to the tile size.
const ZOOM_STEP: f32 = 1.1;

/// The colour of the mark on a tile that holds more units than its ground
/// admits.
///
/// The mark is the viewer's own, in the way every colour here is. The engine
/// says what the ground admits and never what a breach looks like.[^1]
///
/// # References
///
/// [^1]: ADR-0067, the viewer reads the world and never writes to it, decision D2. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
const OVER_CAPACITY: u32 = 0x00ff_2a1e;

/// A pixel buffer that the viewer paints and the window shows.
pub struct Canvas {
    width: usize,
    height: usize,
    pixels: Vec<u32>,
    tiles_painted: u32,
    soldiers_painted: u32,
    blocks_read: u32,
    blocks_skipped: u32,
    painted_by_faction: [u32; COLOURED_FACTIONS],
    painted_by_kind: [u32; KIND_COUNT],
    holder_reads: u32,
    tiles_held: u32,
    crowd_worst: u32,
    tiles_at_capacity: u32,
}

impl Canvas {
    /// Builds a canvas of the given size.
    ///
    /// # Panics
    ///
    /// Panics when either side is zero. A window of no size is a programming
    /// error in the binary, not a condition a user reaches.
    #[must_use]
    pub fn new(width: usize, height: usize) -> Self {
        assert!(width > 0 && height > 0, "a canvas needs a positive size");
        Self {
            width,
            height,
            pixels: vec![BACKGROUND; width * height],
            tiles_painted: 0,
            soldiers_painted: 0,
            blocks_read: 0,
            blocks_skipped: 0,
            painted_by_faction: [0; COLOURED_FACTIONS],
            painted_by_kind: [0; KIND_COUNT],
            holder_reads: 0,
            tiles_held: 0,
            crowd_worst: 0,
            tiles_at_capacity: 0,
        }
    }

    /// Returns the pixels, for the window to show.
    #[must_use]
    pub fn pixels(&self) -> &[u32] {
        &self.pixels
    }

    /// Returns the width in pixels.
    #[must_use]
    pub const fn width(&self) -> usize {
        self.width
    }

    /// Returns the height in pixels.
    #[must_use]
    pub const fn height(&self) -> usize {
        self.height
    }

    /// Returns the number of holders the last draw read.
    ///
    /// The drawing reads the holder of every tile it paints, and the six
    /// neighbours of every tile that somebody holds. The count is therefore a
    /// function of the window and never of the world.[^1] A test reads it to
    /// check that, because a layer that swept the world would still paint the
    /// right picture.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0070, the head-up display reports what the drawing pass read, decision D1. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
    /// [^2]: Findings register, FND-071. `docs/FINDINGS.md`
    #[must_use]
    pub const fn holder_reads(&self) -> u32 {
        self.holder_reads
    }

    /// Returns the number of painted tiles that a faction holds.
    ///
    /// The count is of the window, in the same way every other count of the
    /// drawing pass is.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0070, the head-up display reports what the drawing pass read, decision D2. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
    #[must_use]
    pub const fn tiles_held(&self) -> u32 {
        self.tiles_held
    }

    /// Returns the number of tiles the last draw painted.
    ///
    /// The product record requires that the cost of a drawing follows the
    /// window and not the world.[^1] This count is how a test reads that
    /// requirement. It belongs to the viewer. The engine holds no such
    /// number, and it never will.[^2]
    ///
    /// # References
    ///
    /// [^1]: PRD-0002, a developer watches the world run. `docs/product/shaped/prd-0002-a-developer-watches-the-world-run.md`
    /// [^2]: ADR-0067, the viewer reads the world and never writes to it, decision D2. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
    #[must_use]
    pub const fn tiles_painted(&self) -> u32 {
        self.tiles_painted
    }

    /// Returns the tiles of each kind that the last draw painted.
    ///
    /// The index is the kind number, which the engine fixes and the state
    /// hash reads. The count is the viewer's own: the engine holds no count
    /// that exists for a panel.[^1]
    ///
    /// The panel names each kind against this count, so a person can say what
    /// the ground in the window is rather than guess it from the colours.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0067, the viewer reads the world and never writes to it, decision D2. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
    /// [^2]: PRD-0003, a developer sees a world worth looking at. `docs/product/shaped/prd-0003-a-developer-sees-a-world-worth-looking-at.md`
    #[must_use]
    pub const fn painted_by_kind(&self) -> &[u32; KIND_COUNT] {
        &self.painted_by_kind
    }

    /// Returns the blocks whose units the last draw read.
    ///
    /// A block is read only when the occupancy bitplane says it holds a unit
    /// and the window covers it. The count is the viewer's evidence that its
    /// reading follows the window rather than the population.
    #[must_use]
    pub const fn blocks_read(&self) -> u32 {
        self.blocks_read
    }

    /// Returns the blocks the last draw skipped on the bitplane alone.
    #[must_use]
    pub const fn blocks_skipped(&self) -> u32 {
        self.blocks_skipped
    }

    /// Returns the number of soldiers the last draw painted.
    #[must_use]
    pub const fn soldiers_painted(&self) -> u32 {
        self.soldiers_painted
    }

    /// Returns the soldiers the last draw painted, one count for each colour.
    ///
    /// This is a census of the window, and the drawing pass produced it. The
    /// viewer counts a soldier when it paints one, so the count costs nothing
    /// beyond the draw and grows with the window rather than with the
    /// population.[^1]
    ///
    /// The engine holds no such census, and the viewer must not build one by
    /// reading every soldier. A count of the whole world is a pass over the
    /// whole world.[^2]
    ///
    /// # References
    ///
    /// [^1]: PRD-0002, a developer watches the world run. `docs/product/shaped/prd-0002-a-developer-watches-the-world-run.md`
    /// [^2]: ADR-0070, the head-up display reports what the drawing pass read, decision D1. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
    #[must_use]
    pub const fn painted_by_faction(&self) -> &[u32; COLOURED_FACTIONS] {
        &self.painted_by_faction
    }

    /// Returns the largest number of units the last draw painted on one tile.
    ///
    /// The count is of the window. The drawing pass reads the units of a
    /// block in tile order, so the units of one tile arrive as one adjacent
    /// run, and the length of that run costs one addition for each unit the
    /// pass was already painting. The viewer starts no second pass.[^1]
    ///
    /// This is not the largest number on any tile of the world. Nothing
    /// knows that without reading every unit, so the panel states no such
    /// number rather than an estimate of it.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0070, the head-up display reports what the drawing pass read, decision D1. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
    /// [^2]: ADR-0070, the head-up display reports what the drawing pass read, decision D2. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
    #[must_use]
    pub const fn crowd_worst(&self) -> u32 {
        self.crowd_worst
    }

    /// Returns the painted tiles that hold at least as many units as their
    /// ground admits.
    ///
    /// The capacity comes from the terrain, which is where the engine reads
    /// it too. The viewer holds no capacity value of its own, so a change to
    /// the terrain table reaches the picture with no edit here.[^1]
    ///
    /// The count is of the window, in the same way every other count of the
    /// drawing pass is.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D4. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
    /// [^2]: ADR-0070, the head-up display reports what the drawing pass read, decision D2. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
    #[must_use]
    pub const fn tiles_at_capacity(&self) -> u32 {
        self.tiles_at_capacity
    }

    /// Fills a rectangle with one colour.
    ///
    /// The head-up display draws its panel with this. A position outside the
    /// canvas is clipped rather than a panic.
    pub fn block(&mut self, x: i32, y: i32, width: i32, height: i32, colour: u32) {
        self.fill_rect(x, y, width, height, colour);
    }

    /// Mixes a colour into a rectangle, keeping part of what is under it.
    ///
    /// The head-up display sits over the world. A panel that hid the world
    /// under it would take away the thing the person is watching, so the
    /// panel lets the world show through.
    ///
    /// The weight runs from 0, which changes nothing, to 255, which covers
    /// the world completely.
    pub fn shade(&mut self, x: i32, y: i32, width: i32, height: i32, colour: u32, weight: u8) {
        for row in y..y + height {
            for column in x..x + width {
                if let Some(under) = self.pixel_at(column, row) {
                    self.put(column, row, mix(under, colour, weight));
                }
            }
        }
    }

    /// Writes a line of text, and returns the position after the last glyph.
    ///
    /// The scale multiplies each glyph pixel into a square, so every edge
    /// stays on a pixel boundary.
    ///
    /// # Panics
    ///
    /// Panics when the scale is not positive. A scale of zero draws nothing
    /// and hides the mistake.
    pub fn write(&mut self, x: i32, y: i32, line: &str, scale: i32, colour: u32) -> i32 {
        assert!(scale > 0, "a glyph needs a positive scale");
        let mut pen = x;
        for character in line.chars() {
            let rows = text::glyph(character);
            for (row, bits) in rows.iter().enumerate() {
                for column in 0..text::GLYPH_WIDTH {
                    if bits & (1 << column) == 0 {
                        continue;
                    }
                    self.fill_rect(
                        pen + column * scale,
                        y + row as i32 * scale,
                        scale,
                        scale,
                        colour,
                    );
                }
            }
            pen += text::GLYPH_WIDTH * scale;
        }
        pen
    }

    /// Returns the colour of one pixel, or nothing when it is off the canvas.
    fn pixel_at(&self, x: i32, y: i32) -> Option<u32> {
        if x < 0 || y < 0 {
            return None;
        }
        let (x, y) = (x as usize, y as usize);
        if x >= self.width || y >= self.height {
            return None;
        }
        Some(self.pixels[y * self.width + x])
    }

    /// Fills the whole canvas with the background.
    ///
    /// The counts reset here, so they always describe one draw.
    pub fn clear(&mut self) {
        self.pixels.fill(BACKGROUND);
        self.tiles_painted = 0;
        self.soldiers_painted = 0;
        self.blocks_read = 0;
        self.blocks_skipped = 0;
        self.painted_by_faction = [0; COLOURED_FACTIONS];
        self.painted_by_kind = [0; KIND_COUNT];
        self.holder_reads = 0;
        self.tiles_held = 0;
        self.crowd_worst = 0;
        self.tiles_at_capacity = 0;
    }

    /// Sets one pixel, and ignores a position outside the canvas.
    ///
    /// Clipping here rather than at each caller keeps the drawing routines
    /// free of bounds arithmetic.
    fn put(&mut self, x: i32, y: i32, colour: u32) {
        if x < 0 || y < 0 {
            return;
        }
        let (x, y) = (x as usize, y as usize);
        if x >= self.width || y >= self.height {
            return;
        }
        self.pixels[y * self.width + x] = colour;
    }

    /// Fills a rectangle.
    fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32, colour: u32) {
        for row in y..y + h {
            for column in x..x + w {
                self.put(column, row, colour);
            }
        }
    }

    /// Says whether a disc at this centre can reach the canvas.
    ///
    /// A soldier far outside the window costs one comparison instead of a
    /// square of pixel writes.
    fn holds(&self, x: f32, y: f32, radius: i32) -> bool {
        let reach = radius as f32;
        x + reach >= 0.0
            && y + reach >= 0.0
            && x - reach < self.width as f32
            && y - reach < self.height as f32
    }

    /// Fills a disc, for drawing a soldier.
    fn fill_disc(&mut self, cx: i32, cy: i32, radius: i32, colour: u32) {
        for row in -radius..=radius {
            for column in -radius..=radius {
                if column * column + row * row <= radius * radius {
                    self.put(cx + column, cy + row, colour);
                }
            }
        }
    }
}

/// Where the world sits on the screen.
///
/// The camera holds floating point, and it is the viewer's own. Nothing here
/// reaches the engine.[^1]
///
/// # References
///
/// [^1]: ADR-0067, the viewer reads the world and never writes to it, decision D2. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
#[derive(Clone, Copy, Debug)]
pub struct Camera {
    /// The width of one tile in pixels.
    pub tile_width: f32,
    /// The height of one tile in pixels.
    pub tile_height: f32,
    /// The pixel offset of the tile at the origin.
    pub origin_x: f32,
    /// The pixel offset of the tile at the origin.
    pub origin_y: f32,
}

impl Camera {
    /// Builds a camera that fits the whole world into the canvas.
    ///
    /// The world is a parallelogram, so the drawn width is the tile count
    /// across plus the shear that the rows add.
    #[must_use]
    pub fn fitting(world: &World, canvas: &Canvas) -> Self {
        let grid = world.grid();
        let across = grid.width() as f32;
        let down = grid.height() as f32;

        // Each row shifts right by half a tile, so the parallelogram is wider
        // than the grid by half its height.
        let spans_across = across + down / 2.0;
        let by_width = canvas.width() as f32 / (spans_across + 1.0);
        let by_height = canvas.height() as f32 / (down + 1.0);
        let size = by_width.min(by_height).max(2.0);

        Self {
            tile_width: size,
            tile_height: size,
            origin_x: size * 0.5,
            origin_y: size * 0.5,
        }
    }

    /// Builds a camera with a fixed tile size, at the corner of the world.
    ///
    /// A world larger than the window cannot be fitted and still be read. A
    /// fixed size keeps a tile legible, and the person scrolls to see the
    /// rest.
    #[must_use]
    pub fn at_tile_size(size: f32) -> Self {
        let size = size.clamp(MIN_TILE, MAX_TILE);
        Self {
            tile_width: size,
            tile_height: size,
            origin_x: size * 0.5,
            origin_y: size * 0.5,
        }
    }

    /// Builds the camera the viewer opens with.
    ///
    /// The size is a viewer choice, not a world property, so it lives here
    /// rather than in the binary that draws.
    #[must_use]
    pub fn opening() -> Self {
        Self::at_tile_size(OPENING_TILE)
    }

    /// Returns the camera moved by a whole number of tiles.
    ///
    /// A caller that steers by keyboard thinks in tiles. A caller that
    /// steers by pixels uses the pixel form.
    #[must_use]
    pub fn stepped(self, across: f32, down: f32) -> Self {
        self.panned(across * self.tile_width, down * self.tile_height)
    }

    /// Returns the camera one step closer to the world.
    #[must_use]
    pub fn zoomed_in(self, canvas: &Canvas) -> Self {
        self.zoomed(ZOOM_STEP, canvas)
    }

    /// Returns the camera one step further from the world.
    #[must_use]
    pub fn zoomed_out(self, canvas: &Canvas) -> Self {
        self.zoomed(1.0 / ZOOM_STEP, canvas)
    }

    /// Returns the tile under a screen position.
    ///
    /// The result is an exact integer address. A screen position is a
    /// floating point number, and this is where it stops being one. No
    /// floating point value travels on from here.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0067, the viewer reads the world and never writes to it, decision D3. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
    #[must_use]
    pub fn tile_at(self, x: f32, y: f32) -> Axial {
        let r = (y - self.origin_y) / positive(self.tile_height);
        let q = (x - self.origin_x) / positive(self.tile_width) - r / 2.0;
        Axial::new(q.round() as i32, r.round() as i32)
    }

    /// Returns the camera moved by a pixel offset.
    ///
    /// A positive offset moves the view right and down, so the world moves
    /// left and up. The camera is the viewer's own value, and no part of it
    /// reaches the engine.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0067, the viewer reads the world and never writes to it, decision D2. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
    #[must_use]
    pub fn panned(self, across: f32, down: f32) -> Self {
        Self {
            origin_x: self.origin_x - across,
            origin_y: self.origin_y - down,
            ..self
        }
    }

    /// Returns the camera moved so that an address sits at the middle of the
    /// window.
    ///
    /// The camera is the viewer's own value. This call reads an address and
    /// changes nothing in the world.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0067, the viewer reads the world and never writes to it, decision D2. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
    #[must_use]
    pub fn looking_at(self, address: Axial, canvas: &Canvas) -> Self {
        let (x, y) = self.centre_of(address);
        self.panned(
            x - canvas.width() as f32 * 0.5,
            y - canvas.height() as f32 * 0.5,
        )
    }

    /// Returns the camera with the tile size multiplied, about the canvas centre.
    ///
    /// The tile under the middle of the window stays under the middle of the
    /// window, so a zoom does not throw away what the person was looking at.
    #[must_use]
    pub fn zoomed(self, factor: f32, canvas: &Canvas) -> Self {
        let size = (self.tile_width * factor).clamp(MIN_TILE, MAX_TILE);
        let middle_x = canvas.width() as f32 * 0.5;
        let middle_y = canvas.height() as f32 * 0.5;

        // Read the tile address under the middle, then put it back there.
        let r = (middle_y - self.origin_y) / positive(self.tile_height);
        let q = (middle_x - self.origin_x) / positive(self.tile_width) - r / 2.0;

        Self {
            tile_width: size,
            tile_height: size,
            origin_x: middle_x - (q + r / 2.0) * size,
            origin_y: middle_y - r * size,
        }
    }

    /// Returns the camera held so that the world cannot leave the window.
    ///
    /// A person who scrolls far must be able to scroll back. This keeps at
    /// least half of the smaller of the world and the window on the screen,
    /// in each direction.
    ///
    /// The world is a parallelogram, so the horizontal extent depends on
    /// which rows are on the screen. The vertical bound is therefore settled
    /// first, and the horizontal bound is read from the rows that survive
    /// it.
    #[must_use]
    pub fn clamped(self, world: &World, canvas: &Canvas) -> Self {
        let grid = world.grid();
        let across = (grid.width().max(1) - 1) as f32;
        let down = (grid.height().max(1) - 1) as f32;
        let canvas_x = canvas.width() as f32;
        let canvas_y = canvas.height() as f32;

        let span_y = down * self.tile_height;
        let keep_y = span_y.min(canvas_y) * 0.5;
        let upright = Self {
            origin_y: self.origin_y.clamp(keep_y - span_y, canvas_y - keep_y),
            ..self
        };

        // Each row starts half a tile further right than the row above it.
        // The leftmost visible row gives the left edge, and the rightmost
        // end of the lowest visible row gives the right edge.
        let (first_row, last_row) = upright.visible_rows(world, canvas);
        let lowest = last_row.max(first_row + 1) - 1;
        let left = (first_row as f32 / 2.0) * upright.tile_width;
        let right = (across + lowest as f32 / 2.0) * upright.tile_width;
        let keep_x = (right - left).min(canvas_x) * 0.5;

        Self {
            origin_x: upright
                .origin_x
                .clamp(keep_x - right, canvas_x - keep_x - left),
            ..upright
        }
    }

    /// Returns the rows of the world that the canvas can show.
    ///
    /// The range is a half-open pair. It is derived from the camera and the
    /// canvas, so its length follows the window and not the world.
    #[must_use]
    pub fn visible_rows(self, world: &World, canvas: &Canvas) -> (u32, u32) {
        let height = world.grid().height();
        let scale = positive(self.tile_height);
        let first = ((-self.origin_y) / scale).floor() - 1.0;
        let last = ((canvas.height() as f32 - self.origin_y) / scale).ceil() + 1.0;
        span(first, last, height)
    }

    /// Returns the columns of one row that the canvas can show.
    ///
    /// Each row starts half a tile further right than the row above it, so
    /// the column range depends on the row.
    #[must_use]
    pub fn visible_columns(self, row: u32, world: &World, canvas: &Canvas) -> (u32, u32) {
        let width = world.grid().width();
        let scale = positive(self.tile_width);
        let start = self.origin_x + (row as f32 / 2.0) * self.tile_width;
        let first = ((-start) / scale).floor() - 1.0;
        let last = ((canvas.width() as f32 - start) / scale).ceil() + 1.0;
        span(first, last, width)
    }

    /// Returns the pixel centre of a tile.
    ///
    /// This is the skew. A rhombus in the index space becomes a
    /// parallelogram on the screen, and the row is what shifts the
    /// column.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0017, the world is a rhombus, so a tile index is raw axial, decision D4. `docs/adrs/accepted/adr-0017-the-world-is-a-rhombus-so-a-tile-index-is-raw-axial.md`
    #[must_use]
    pub fn centre_of(self, address: Axial) -> (f32, f32) {
        let q = address.q as f32;
        let r = address.r as f32;
        let x = self.origin_x + (q + r / 2.0) * self.tile_width;
        let y = self.origin_y + r * self.tile_height;
        (x, y)
    }
}

/// Mixes two colours by a weight, one channel at a time.
///
/// The arithmetic is integer, because a colour is a byte triple and there is
/// no reason to leave that. It is not simulated state, so the rule that bans
/// floating point does not reach here either way.[^1]
///
/// # References
///
/// [^1]: ADR-0002, simulated and aggregated state holds no floating point number, decision D4. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
pub(crate) fn mix(under: u32, over: u32, weight: u8) -> u32 {
    let weight = u32::from(weight);
    let rest = 255 - weight;
    let mut mixed = 0;
    for shift in [16, 8, 0] {
        let a = (under >> shift) & 0xff;
        let b = (over >> shift) & 0xff;
        mixed |= ((a * rest + b * weight) / 255) << shift;
    }
    mixed
}

/// Keeps a divisor away from zero.
///
/// A camera with a tile size of zero is a viewer mistake. It must give an
/// empty picture, not a division that produces a value nothing can use.
fn positive(scale: f32) -> f32 {
    if scale > 0.0 {
        scale
    } else {
        f32::MIN_POSITIVE
    }
}

/// Turns a pair of floating point bounds into a range inside the world.
///
/// A cast to an integer saturates in Rust, so a very large camera offset
/// gives a bound at the edge of the world rather than a wrapped number.
fn span(first: f32, last: f32, limit: u32) -> (u32, u32) {
    if limit == 0 || last < 0.0 {
        return (0, 0);
    }
    let first = (first as i64).clamp(0, i64::from(limit)) as u32;
    let last = (last as i64).clamp(0, i64::from(limit)) as u32;
    (first, last.max(first))
}

/// Returns the colour of one tile.
///
/// The kind chooses the colour and the height brightens it, so a person reads
/// the relief of the ground as well as its kind.[^1] The simulated value
/// ripples on top, so a still camera over a stepping world still moves.
///
/// Both the height and the value are fixed-point numbers in the engine. The
/// viewer reads their raw bits and turns them into a brightness. That is a
/// conversion out of exact arithmetic, and nothing here goes back into
/// it.[^2]
///
/// # References
///
/// [^1]: PRD-0003, a developer sees a world worth looking at. `docs/product/shaped/prd-0003-a-developer-sees-a-world-worth-looking-at.md`
/// [^2]: ADR-0067, the viewer reads the world and never writes to it, decision D3. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
fn tile_colour(kind: TileKind, height: i32, value: i32) -> u32 {
    let base = KIND_COLOURS[kind.to_u8() as usize];
    // The height is a fraction of the full range in Q16.16, so the unit is
    // 65536. The shift maps the fraction onto the brightness steps.
    let relief = (height.clamp(0, 0x0001_0000) * HEIGHT_STEPS) >> 16;
    let ripple = (((value >> 10) & 0xff) * VALUE_STEPS) >> 8;
    let shade = relief + ripple;
    let channel = |offset: u32| (((base >> offset) & 0xff) as i32 + shade).clamp(0, 0xff) as u32;
    (channel(16) << 16) | (channel(8) << 8) | channel(0)
}

/// Returns the colour the viewer draws one kind of ground in, at the middle
/// of the height range and with no ripple.
///
/// The head-up display and the tests need the colour of a kind without a
/// tile to read it from. The engine holds no colour, so the reader is
/// here.[^1]
///
/// # References
///
/// [^1]: ADR-0067, the viewer reads the world and never writes to it, decision D2. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
#[must_use]
pub fn kind_colour(kind: TileKind) -> u32 {
    tile_colour(kind, 0x0000_8000, 0)
}

/// Draws the world onto the canvas.
///
/// The viewer reads the world through the public interface and writes
/// nothing to it. The argument is a shared reference, so the compiler
/// enforces that.[^1]
///
/// # References
///
/// [^1]: ADR-0067, the viewer reads the world and never writes to it, decision D1. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
/// [^2]: ADR-0068, terrain is generated from the seed and is never stored as a map, decision D1. `docs/adrs/accepted/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
/// [^3]: ADR-0070, the head-up display reports what the drawing pass read, decision D1. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
/// [^4]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D2. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
pub fn draw(world: &World, camera: Camera, canvas: &mut Canvas) -> Result<(), BridgeError> {
    canvas.clear();
    let grid = world.grid();
    let values = world.tile_values();
    // The ground is a pure function of the seed and the address, so the
    // viewer computes it for the tiles the window covers and for no other.
    // A sweep of the whole world every frame is what the record calls a
    // design mistake.[^2]
    let terrain = world.terrain();

    let tile_side = (camera.tile_width * 0.92).max(1.0) as i32;
    let (first_row, last_row) = camera.visible_rows(world, canvas);
    for row in first_row..last_row {
        let (first_column, last_column) = camera.visible_columns(row, world, canvas);
        for column in first_column..last_column {
            let address = Axial::new(column as i32, row as i32);
            let Some(index) = grid.index_of(address) else {
                continue;
            };
            let value = values[index.0 as usize].0;
            let Some(ground) = terrain.tile(address) else {
                continue;
            };
            let (x, y) = camera.centre_of(address);
            let ground_colour = tile_colour(ground.kind, ground.height.0, value);
            let left = x as i32 - tile_side / 2;
            let top = y as i32 - tile_side / 2;

            // The holder of this tile, read at the tile that is being
            // painted, on the loop that already runs. The layer starts no
            // pass of its own.[^3]
            //
            // This reads the holder of the world. It never reads the tile
            // faction column of the stub system, which is written when the
            // world is built, never changes, and is not a holder. A layer
            // that drew that column would give a full, still map of holdings
            // that no rule ever made.[^4]
            let holder = world.tile_holder(address);
            canvas.holder_reads += 1;
            match holder.and_then(Holder::faction) {
                None => canvas.fill_rect(left, top, tile_side, tile_side, ground_colour),
                Some(faction) => {
                    let held = faction_colour(faction);
                    canvas.fill_rect(
                        left,
                        top,
                        tile_side,
                        tile_side,
                        mix(ground_colour, held, HOLDER_WEIGHT),
                    );
                    canvas.tiles_held += 1;
                    if on_an_edge(world, address, holder, canvas) {
                        outline(
                            canvas,
                            left,
                            top,
                            tile_side,
                            mix(ground_colour, held, EDGE_WEIGHT),
                        );
                    }
                }
            }
            canvas.tiles_painted += 1;
            canvas.painted_by_kind[ground.kind.to_u8() as usize] += 1;
        }
    }

    let radius = ((camera.tile_width * 0.3) as i32).max(1);
    draw_soldiers(
        world, camera, canvas, radius, tile_side, first_row, last_row,
    )
}

/// Reports whether a tile sits on the edge of its holding.
///
/// The six neighbours are fixed offsets, and the edge of the world does not
/// wrap. A neighbour outside the world reads as nobody, so the boundary of
/// the world counts as an edge rather than as a wrap to the far side.[^1]
///
/// The read is of level 0, which is the only truth. A summary level could
/// state a holding that the tiles below it no longer hold.[^2]
///
/// # References
///
/// [^1]: ADR-0017, the world is a rhombus, so a tile index is raw axial, decision D2. `docs/adrs/accepted/adr-0017-the-world-is-a-rhombus-so-a-tile-index-is-raw-axial.md`
/// [^2]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D1. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
fn on_an_edge(world: &World, address: Axial, holder: Option<Holder>, canvas: &mut Canvas) -> bool {
    let mut edge = false;
    for offset in NEIGHBOURS {
        let beside = world.tile_holder(address.add(offset));
        canvas.holder_reads += 1;
        // The loop does not stop at the first difference. A short loop would
        // make the count of reads depend on where the neighbour sits, and the
        // cost of the layer would then follow the shape of the holdings.
        edge = edge || beside.unwrap_or(Holder::NOBODY) != holder.unwrap_or(Holder::NOBODY);
    }
    edge
}

/// Draws a one pixel border inside a square.
fn outline(canvas: &mut Canvas, left: i32, top: i32, side: i32, colour: u32) {
    canvas.fill_rect(left, top, side, 1, colour);
    canvas.fill_rect(left, top + side - 1, side, 1, colour);
    canvas.fill_rect(left, top, 1, side, colour);
    canvas.fill_rect(left + side - 1, top, 1, side, colour);
}

/// Draws the soldiers that stand inside the visible blocks.
///
/// The viewer reads the engine's own spatial structure rather than scanning
/// the population. The structure sorts the units block by block, holds the
/// range of each block, and marks every occupied block in a bitplane.[^1]
/// Testing that bitplane and skipping an empty block is what the bitplane is
/// for.[^2]
///
/// The cost follows the blocks the window covers. It does not follow the
/// population, which is what the product record asks of every viewer
/// read.[^3]
///
/// The viewer builds no index of its own. A second structure that says where
/// a unit stands is one fact in two places, and nothing would fail when the
/// two disagreed.[^4]
///
/// A stale read returns an error rather than a wrong picture. The step
/// rebuilds the structure at the barrier, so a viewer that draws after a step
/// reads a current one. A viewer that draws after moving a soldier itself
/// cannot, and it must not: it would be drawing a world that no longer
/// exists.
///
/// # References
///
/// [^1]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D1. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
/// [^2]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D5. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
/// [^3]: PRD-0002, a developer watches the world run. `docs/product/shaped/prd-0002-a-developer-watches-the-world-run.md`
/// [^4]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
fn draw_soldiers(
    world: &World,
    camera: Camera,
    canvas: &mut Canvas,
    radius: i32,
    tile_side: i32,
    first_row: u32,
    last_row: u32,
) -> Result<(), BridgeError> {
    let arena = world.soldiers();
    let terrain = world.terrain();
    let bridge = world.bridge();
    let layout = bridge.layout();
    let edge = layout.block_edge();
    if edge == 0 || last_row <= first_row {
        return Ok(());
    }

    // Ask once, before trusting the bitplane. The bitplane is an unguarded
    // read: a stale one reports every block empty, so a viewer that skipped
    // on it alone would draw no units and report success. That is a wrong
    // picture presented as a right one, which is worse than a refusal.
    bridge.describes(arena)?;

    let first_block_row = first_row / edge;
    let last_block_row = (last_row - 1) / edge;

    for block_row in first_block_row..=last_block_row.min(layout.blocks_high().saturating_sub(1)) {
        // The column range depends on the row, because a rhombus shears. Take
        // the widest column span of the rows this block covers, so a block is
        // read when any of its rows is visible.
        let row_lo = (block_row * edge).max(first_row);
        let row_hi = ((block_row + 1) * edge - 1).min(last_row - 1);
        let (mut lo, mut hi) = (u32::MAX, 0u32);
        for row in [row_lo, row_hi] {
            let (a, b) = camera.visible_columns(row, world, canvas);
            lo = lo.min(a);
            hi = hi.max(b);
        }
        if hi <= lo {
            continue;
        }

        let first_block_column = lo / edge;
        let last_block_column = ((hi - 1) / edge).min(layout.blocks_wide().saturating_sub(1));
        for block_column in first_block_column..=last_block_column {
            let block = block_row * layout.blocks_wide() + block_column;
            if !bridge.block_is_occupied(block) {
                canvas.blocks_skipped += 1;
                continue;
            }
            // The structure must describe this arena. Drawing a remembered
            // answer would be a picture of a world that no longer exists,
            // and a viewer that drew one silently would be the worst of the
            // three outcomes.
            let units = bridge.in_block(arena, block)?;
            canvas.blocks_read += 1;
            // The units of one block arrive in tile order, so the units of
            // one tile are one adjacent run.[^5] The crowd count is the
            // length of that run. It costs one comparison for each unit on a
            // path that already visits every unit it paints, so the count
            // adds no pass over the world.[^6]
            //
            // [^5]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D2. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
            // [^6]: ADR-0070, the head-up display reports what the drawing pass read, decision D1. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
            let mut run: Option<(Axial, u32)> = None;
            for soldier in units {
                let Some(address) = arena.address(*soldier) else {
                    continue;
                };
                let Some(faction) = arena.faction(*soldier) else {
                    continue;
                };
                let (x, y) = camera.centre_of(address);
                if !canvas.holds(x, y, radius) {
                    continue;
                }
                let slot = colour_slot(faction);
                canvas.fill_disc(x as i32, y as i32, radius, FACTION_COLOURS[slot]);
                canvas.soldiers_painted += 1;
                // The census is a by-product of the pass that paints. A
                // separate pass over the soldiers would give the same numbers
                // and cost the population.
                canvas.painted_by_faction[slot] += 1;
                match run {
                    Some((held, count)) if held == address => run = Some((held, count + 1)),
                    other => {
                        close_run(canvas, terrain, camera, tile_side, other);
                        run = Some((address, 1));
                    }
                }
            }
            close_run(canvas, terrain, camera, tile_side, run);
        }
    }
    Ok(())
}

/// Records what one tile's run of painted units means, and marks the tile
/// when the run is longer than the ground admits.
///
/// The capacity comes from the terrain reader. The viewer holds no capacity
/// value, so the ground states the rule in one place and a change there
/// reaches the picture with no edit here.[^1]
///
/// A tile with no painted unit closes no run, so an empty tile is neither
/// counted nor marked.
///
/// # References
///
/// [^1]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D4. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
fn close_run(
    canvas: &mut Canvas,
    terrain: Terrain,
    camera: Camera,
    tile_side: i32,
    run: Option<(Axial, u32)>,
) {
    let Some((address, count)) = run else {
        return;
    };
    canvas.crowd_worst = canvas.crowd_worst.max(count);
    let Some(ground) = terrain.tile(address) else {
        return;
    };
    let capacity = ground.kind.capacity();
    if count >= capacity {
        canvas.tiles_at_capacity += 1;
    }
    if count > capacity {
        let (x, y) = camera.centre_of(address);
        let left = x as i32 - tile_side / 2;
        let top = y as i32 - tile_side / 2;
        outline(canvas, left, top, tile_side, OVER_CAPACITY);
    }
}
