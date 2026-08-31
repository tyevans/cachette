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

use cachette_core::{Axial, World};

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

/// A pixel buffer that the viewer paints and the window shows.
pub struct Canvas {
    width: usize,
    height: usize,
    pixels: Vec<u32>,
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

    /// Fills the whole canvas with the background.
    pub fn clear(&mut self) {
        self.pixels.fill(BACKGROUND);
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
pub fn draw(world: &World, camera: Camera, canvas: &mut Canvas) {
    canvas.clear();
    let grid = world.grid();
    let values = world.tile_values();

    let tile_side = (camera.tile_width * 0.92).max(1.0) as i32;
    for row in 0..grid.height() {
        for column in 0..grid.width() {
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
        }
    }

    let radius = ((camera.tile_width * 0.3) as i32).max(1);
    let arena = world.soldiers();
    for soldier in arena.iter() {
        let Some(address) = arena.address(soldier) else {
            continue;
        };
        let Some(faction) = arena.faction(soldier) else {
            continue;
        };
        let (x, y) = camera.centre_of(address);
        let colour = FACTION_COLOURS[(faction.0 as usize) % FACTION_COLOURS.len()];
        canvas.fill_disc(x as i32, y as i32, radius, colour);
    }
}
