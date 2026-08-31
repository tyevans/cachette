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
//! # References
//!
//! [^1]: ADR-0002, simulated and aggregated state holds no floating point number, decision D4. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
//! [^2]: ADR-0067, the viewer reads the world and never writes to it, decision D3. `docs/adrs/draft/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
//! [^3]: ADR-0017, the world is a rhombus, so a tile index is raw axial, decision D4. `docs/adrs/accepted/adr-0017-the-world-is-a-rhombus-so-a-tile-index-is-raw-axial.md`

use cachette_core::{Axial, BridgeError, World};

/// The colour of the space outside the world.
const BACKGROUND: u32 = 0x0010_1418;

/// The colour of a tile, before its value shades it.
const TILE_BASE: u32 = 0x0026_3238;

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

/// The smallest tile size the viewer will show, in pixels.
const MIN_TILE: f32 = 2.0;

/// The largest tile size the viewer will show, in pixels.
const MAX_TILE: f32 = 64.0;

/// The tile size the viewer opens with, in pixels.
const OPENING_TILE: f32 = 12.0;

/// The factor one zoom press applies to the tile size.
const ZOOM_STEP: f32 = 1.1;

/// A pixel buffer that the viewer paints and the window shows.
pub struct Canvas {
    width: usize,
    height: usize,
    pixels: Vec<u32>,
    tiles_painted: u32,
    soldiers_painted: u32,
    blocks_read: u32,
    blocks_skipped: u32,
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
    /// [^2]: ADR-0067, the viewer reads the world and never writes to it, decision D2. `docs/adrs/draft/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
    #[must_use]
    pub const fn tiles_painted(&self) -> u32 {
        self.tiles_painted
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

    /// Fills the whole canvas with the background.
    ///
    /// The counts reset here, so they always describe one draw.
    pub fn clear(&mut self) {
        self.pixels.fill(BACKGROUND);
        self.tiles_painted = 0;
        self.soldiers_painted = 0;
        self.blocks_read = 0;
        self.blocks_skipped = 0;
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
/// [^1]: ADR-0067, the viewer reads the world and never writes to it, decision D2. `docs/adrs/draft/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
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
    /// [^1]: ADR-0067, the viewer reads the world and never writes to it, decision D3. `docs/adrs/draft/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
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
    /// [^1]: ADR-0067, the viewer reads the world and never writes to it, decision D2. `docs/adrs/draft/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
    #[must_use]
    pub fn panned(self, across: f32, down: f32) -> Self {
        Self {
            origin_x: self.origin_x - across,
            origin_y: self.origin_y - down,
            ..self
        }
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

/// Shades a tile by its value.
///
/// The value is a fixed-point number in the engine. The viewer reads its raw
/// bits and turns them into a brightness, which is a conversion out of exact
/// arithmetic and never back into it.
fn tile_colour(raw: i32) -> u32 {
    let shade = ((raw >> 8) & 0x3f) as u32;
    let base = TILE_BASE;
    let red = ((base >> 16) & 0xff) + shade;
    let green = ((base >> 8) & 0xff) + shade;
    let blue = (base & 0xff) + shade;
    (red.min(0xff) << 16) | (green.min(0xff) << 8) | blue.min(0xff)
}

/// Draws the world onto the canvas.
///
/// The viewer reads the world through the public interface and writes
/// nothing to it. The argument is a shared reference, so the compiler
/// enforces that.[^1]
///
/// # References
///
/// [^1]: ADR-0067, the viewer reads the world and never writes to it, decision D1. `docs/adrs/draft/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
pub fn draw(world: &World, camera: Camera, canvas: &mut Canvas) -> Result<(), BridgeError> {
    canvas.clear();
    let grid = world.grid();
    let values = world.tile_values();

    let tile_side = (camera.tile_width * 0.92).max(1.0) as i32;
    let (first_row, last_row) = camera.visible_rows(world, canvas);
    for row in first_row..last_row {
        let (first_column, last_column) = camera.visible_columns(row, world, canvas);
        for column in first_column..last_column {
            let address = Axial::new(column as i32, row as i32);
            let Some(index) = grid.index_of(address) else {
                continue;
            };
            let raw = values[index.0 as usize].0;
            let (x, y) = camera.centre_of(address);
            canvas.fill_rect(
                x as i32 - tile_side / 2,
                y as i32 - tile_side / 2,
                tile_side,
                tile_side,
                tile_colour(raw),
            );
            canvas.tiles_painted += 1;
        }
    }

    let radius = ((camera.tile_width * 0.3) as i32).max(1);
    draw_soldiers(world, camera, canvas, radius, first_row, last_row)
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
    first_row: u32,
    last_row: u32,
) -> Result<(), BridgeError> {
    let arena = world.soldiers();
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
                let colour = FACTION_COLOURS[(faction.0 as usize) % FACTION_COLOURS.len()];
                canvas.fill_disc(x as i32, y as i32, radius, colour);
                canvas.soldiers_painted += 1;
            }
        }
    }
    Ok(())
}
