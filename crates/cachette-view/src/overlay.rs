//! The overlay deck: the map shows one quantity, and the caller chooses it.
//!
//! # What an overlay is
//!
//! An overlay is one quantity of the world, painted over every drawn tile as a
//! strength of one colour. The map carries one at a time. A single picture that
//! carried every quantity at once could carry none of them, which a research
//! report measured three separate times.[^1]
//!
//! # The one declaration site
//!
//! **The list this module holds is the registration.** An overlay states its
//! own name, its own colour, its own span and its own value, and it appears in
//! that list. Nothing else says that it exists: the boundary answers the names
//! from the list, and the demonstration takes its keys from the answer. One
//! fact declared twice, with nothing that fails when the copies drift, is the
//! defect shape this project records first.[^2]
//!
//! This is the pattern the panel deck already follows.[^3]
//!
//! # A presenter chooses, it does not draw
//!
//! **One renderer feeds every presenter.**[^4] A caller names an overlay from
//! the list this crate published. It cannot name one the renderer does not
//! hold, and it cannot supply a rule of its own for painting one, so no
//! presenter becomes a second renderer of the same world.
//!
//! # A value on the level 1 lattice
//!
//! Weather lives on the level 1 cell lattice, and a cell is many tiles a side.
//! A cell value painted flat across its tiles draws a grid of blocks that
//! follows nothing in the world.[^1] An overlay that reads a cell says so, and
//! the drawing then interpolates between the four nearest cell centres, so the
//! quantity reads as a field.
//!
//! The interpolation is floating point arithmetic. It runs in the viewer, no
//! value formed from it returns to the engine, and it is a pure function of the
//! world and the address, so two runs of one frame give one picture.[^5]
//!
//! # What an overlay costs
//!
//! An overlay adds reads to the drawing pass. A tile overlay adds the reads its
//! value names. A cell overlay adds four cell reads to each tile it paints.
//! **No figure for that cost is stated here.** Every cost figure in this
//! project is derived rather than measured, and the register holds the blocker
//! that says so.[^6]
//!
//! # References
//!
//! [^1]: Research report 24, demonstration readability, resources and weather. `docs/research/reports/24-demonstration-readability-resources-and-weather.md`
//! [^2]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
//! [^3]: The panel standard, the registration list. `crates/cachette-view/src/panel/mod.rs`
//! [^4]: ADR-0094, the caller owns the camera and the pixels, decision D5. `docs/adrs/draft/adr-0094-the-caller-owns-the-camera-and-the-pixels.md`
//! [^5]: ADR-0067, the viewer reads the world and never writes to it, decision D3. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
//! [^6]: Blockers register, BLK-007. `docs/BLOCKERS.md`

use cachette_core::resource::ResourceKind;
use cachette_core::terrain::TerrainTile;
use cachette_core::upgrade::UPGRADE_LEVEL_COUNT;
use cachette_core::weather::Drops;
use cachette_core::{Axial, FactionId, Holder, World};

use crate::paint::faction_colour;

/// The strength an overlay paints at its high value, of 255.
///
/// The overlay is mixed into the ground **before** the holder colour, so this
/// number does not compete with the holder weight. A wash over the finished
/// pixel had to stay weak to leave the holder visible, and it did not.[^1]
///
/// # References
///
/// [^1]: Research report 24, defect 4. `docs/research/reports/24-demonstration-readability-resources-and-weather.md`
pub const FULL_STRENGTH: u8 = 210;

/// The strength an overlay paints just above its low value, of 255.
///
/// A value at the bottom of the span paints nothing, so a watcher tells an
/// empty tile from a nearly empty one. The floor lifts the first step above
/// the ground it sits on, because a step of one of 255 is invisible.
pub const LEAST_STRENGTH: u8 = 24;

/// The top of the colour ramp of a tile stock, in resource units.
///
/// **This is a display scale and not the engine's ceiling.** The engine holds
/// a ceiling for each ground and each resource, and it does not publish one. A
/// tile above this value paints at full strength, and the key names the number,
/// so a reader is never told a false maximum.
pub const STOCK_TOP: i64 = 16;

/// The colour of the moisture overlay.
const MOISTURE_COLOUR: u32 = 0x0034_8fd8;

/// The colour of the air overlay.
const AIR_COLOUR: u32 = 0x00d8_e8f8;

/// The colour of the food overlay.
const FOOD_COLOUR: u32 = 0x008e_d94a;

/// The colour of the wood overlay.
const WOOD_COLOUR: u32 = 0x00b8_7333;

/// The colour of the stone overlay.
const STONE_COLOUR: u32 = 0x00c3_ccd4;

/// The colour of the height overlay.
const HEIGHT_COLOUR: u32 = 0x00f0_e2a8;

/// The colour of the upgrade level overlay.
///
/// The overlay paints one colour at a strength that follows the level, so a
/// watcher reads the level from the depth of one hue. The hue is the viewer's
/// own, and the engine holds none.
const UPGRADE_LEVEL_COLOUR: u32 = 0x00e8_c46a;

/// The colour of the crowding overlay.
const CROWD_COLOUR: u32 = 0x00ff_5a3c;

/// The colour an overlay paints where it has no thing to name.
const NOTHING_NAMED: u32 = 0x0044_5058;

/// The full share, in parts of 255.
///
/// The crowding overlay reports a share of the capacity of the tile, so its
/// span runs from nothing to this.
const FULL_SHARE: i64 = 255;

/// What one tile offers an overlay.
///
/// The drawing pass generated the ground of this tile once, and it hands the
/// result here rather than letting an overlay generate it again. Two
/// generations of one ground give one answer, so the second is pure cost.[^1]
///
/// An overlay that reads a cell is asked at addresses the pass did not paint,
/// and the ground of those addresses is absent. A cell overlay reads no ground,
/// so nothing is lost.
///
/// # References
///
/// [^1]: Backlog item 0210, generate the ground of a drawn tile once. `docs/backlog/complete/0210-generate-the-ground-of-a-drawn-tile-once.md`
#[derive(Clone, Copy)]
pub struct At<'a> {
    /// The world, borrowed shared.
    pub world: &'a World,
    /// The address of the tile.
    pub address: Axial,
    /// The ground the drawing pass generated, when this is a painted tile.
    pub ground: Option<TerrainTile>,
}

impl At<'_> {
    /// Returns the ground of this address, generating it when it is absent.
    fn ground(&self) -> Option<TerrainTile> {
        match self.ground {
            Some(ground) => Some(ground),
            None => self.world.tile_terrain(self.address),
        }
    }

    /// Returns what this tile still holds of one resource.
    fn stock(&self, kind: ResourceKind) -> i64 {
        let Some(ground) = self.ground() else {
            return 0;
        };
        self.world
            .tile_stock_of_ground(self.address, ground.kind, kind)
            .map_or(0, |amount| i64::from(amount.0))
    }
}

/// The low and the high value an overlay paints between, for one frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Span {
    /// The value that paints nothing.
    pub low: i64,
    /// The value that paints at full strength.
    pub high: i64,
}

impl Span {
    /// Builds a span, and keeps the high above the low.
    ///
    /// A span of no width would divide by zero, and every value in it would be
    /// both the bottom and the top of the ramp.
    #[must_use]
    pub const fn new(low: i64, high: i64) -> Self {
        Self {
            low,
            high: if high > low { high } else { low + 1 },
        }
    }

    /// Returns the strength a value paints at, of 255.
    #[must_use]
    pub fn strength(self, value: i64) -> u8 {
        if value <= self.low {
            return 0;
        }
        let over = (value - self.low).min(self.high - self.low);
        let range = i64::from(FULL_STRENGTH - LEAST_STRENGTH);
        let lifted = i64::from(LEAST_STRENGTH) + over * range / (self.high - self.low);
        u8::try_from(lifted.clamp(0, 255)).unwrap_or(FULL_STRENGTH)
    }
}

/// One overlay of the deck.
///
/// **An overlay is one entry of the registration list.** It states its name,
/// the words its key writes, the span it paints between and the value of one
/// tile. Nothing else has to change when an overlay joins.
pub trait Layer: Sync {
    /// Returns the name a caller selects this overlay by.
    ///
    /// The name is lower case and holds no space, because a caller types it.
    fn name(&self) -> &'static str;

    /// Returns the unit the key names beside the low and the high value.
    fn unit(&self) -> &'static str;

    /// Returns the value of one tile.
    fn value(&self, at: At<'_>) -> i64;

    /// Returns the low and the high value this overlay paints between.
    ///
    /// This is asked once for a frame and never for a tile, so the cost of the
    /// answer does not follow the window.
    fn span(&self, world: &World) -> Span;

    /// Reports whether the value lives on the level 1 cell lattice.
    ///
    /// A cell value paints as a field. The drawing interpolates between the
    /// four nearest cell centres, so the map does not draw a grid of blocks.
    fn on_cells(&self) -> bool {
        false
    }

    /// Returns the colour a value paints in.
    ///
    /// Most overlays paint one colour at a strength. An overlay whose value
    /// names a thing rather than a quantity gives that thing its own colour.
    fn colour(&self, value: i64) -> u32;

    /// Returns the strength a value paints at, of 255.
    ///
    /// The ramp of the span is the answer for a quantity. An overlay whose
    /// value names a thing paints every named thing alike.
    fn strength(&self, value: i64, span: Span) -> u8 {
        span.strength(value)
    }
}

/// The water on the ground of the cell that covers each tile.
struct Moisture;

impl Layer for Moisture {
    fn name(&self) -> &'static str {
        "moisture"
    }

    fn unit(&self) -> &'static str {
        "drops on the ground"
    }

    fn value(&self, at: At<'_>) -> i64 {
        at.world.ground_water_at(at.address).unwrap_or(0)
    }

    fn span(&self, world: &World) -> Span {
        Span::new(0, highest(world.weather().ground_plane()))
    }

    fn on_cells(&self) -> bool {
        true
    }

    fn colour(&self, _value: i64) -> u32 {
        MOISTURE_COLOUR
    }
}

/// The water in the air above the cell that covers each tile.
struct Air;

impl Layer for Air {
    fn name(&self) -> &'static str {
        "air"
    }

    fn unit(&self) -> &'static str {
        "drops in the air"
    }

    fn value(&self, at: At<'_>) -> i64 {
        at.world.air_at(at.address).unwrap_or(0)
    }

    fn span(&self, world: &World) -> Span {
        Span::new(0, highest(world.weather().air_plane()))
    }

    fn on_cells(&self) -> bool {
        true
    }

    fn colour(&self, _value: i64) -> u32 {
        AIR_COLOUR
    }
}

/// Returns the largest value of a weather plane.
///
/// The scan reads the cell lattice once for a frame. It reads no tile, so its
/// cost follows the cells of the world and never the window or the
/// population.[^1]
///
/// # References
///
/// [^1]: Blockers register, BLK-007. `docs/BLOCKERS.md`
fn highest(plane: &[Drops]) -> i64 {
    plane.iter().map(|drops| drops.0).max().unwrap_or(0)
}

/// What a tile still holds of one resource.
struct Stock {
    /// The resource this overlay reads.
    kind: ResourceKind,
    /// The name a caller selects it by.
    name: &'static str,
    /// The unit the key names.
    unit: &'static str,
    /// The colour it paints.
    colour: u32,
}

impl Layer for Stock {
    fn name(&self) -> &'static str {
        self.name
    }

    fn unit(&self) -> &'static str {
        self.unit
    }

    fn value(&self, at: At<'_>) -> i64 {
        at.stock(self.kind)
    }

    fn span(&self, _world: &World) -> Span {
        Span::new(0, STOCK_TOP)
    }

    fn colour(&self, _value: i64) -> u32 {
        self.colour
    }
}

/// How high the ground of each tile stands.
struct Height;

impl Layer for Height {
    fn name(&self) -> &'static str {
        "height"
    }

    fn unit(&self) -> &'static str {
        "of the full range, in 65536ths"
    }

    fn value(&self, at: At<'_>) -> i64 {
        at.ground().map_or(0, |ground| i64::from(ground.height.0))
    }

    fn span(&self, _world: &World) -> Span {
        // The generated height is at least zero and below one, at the fixed
        // point scale the engine states. The span is that range and not a
        // measurement, so nothing here can go stale.
        Span::new(0, 1 << 16)
    }

    fn colour(&self, _value: i64) -> u32 {
        HEIGHT_COLOUR
    }
}

/// Which faction holds each tile.
struct HolderLayer;

impl Layer for HolderLayer {
    fn name(&self) -> &'static str {
        "holder"
    }

    fn unit(&self) -> &'static str {
        "the faction, or nobody"
    }

    fn value(&self, at: At<'_>) -> i64 {
        at.world
            .tile_holder(at.address)
            .and_then(Holder::faction)
            .map_or(0, |faction| i64::from(faction.0) + 1)
    }

    fn span(&self, world: &World) -> Span {
        Span::new(0, i64::from(world.faction_count()))
    }

    fn colour(&self, value: i64) -> u32 {
        if value <= 0 {
            return NOTHING_NAMED;
        }
        u16::try_from(value - 1).map_or(NOTHING_NAMED, |slot| faction_colour(FactionId(slot)))
    }

    fn strength(&self, value: i64, _span: Span) -> u8 {
        // The value names a faction and not a quantity, so faction 3 is not
        // three times faction 1. Every held tile paints alike, and an unheld
        // tile paints nothing.
        if value > 0 {
            FULL_STRENGTH
        } else {
            0
        }
    }
}

/// The level of the upgrade on each tile, and whether one stands.
///
/// **The value is the level, and one for a site under work.** A site under
/// work stands at level zero, and zero is also the tile that carries nothing,
/// so the bare level could not tell a first build from a bare tile.
///
/// The span runs to one above the level count of the table, so the overlay
/// scales with the table and not with a number this file holds.[^1] It read
/// the category ordinal before, which is not a level at all.[^2]
///
/// # References
///
/// [^1]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
/// [^2]: ADR-0151, an upgrade is a category with a ground fit and a level, decisions D1 and D5. `docs/adrs/accepted/adr-0151-an-upgrade-is-a-category-with-a-ground-fit-and-a-level.md`
struct Upgrade;

impl Layer for Upgrade {
    fn name(&self) -> &'static str {
        "upgrade"
    }

    fn unit(&self) -> &'static str {
        "the level, and one under work"
    }

    fn value(&self, at: At<'_>) -> i64 {
        at.world
            .upgrade_at(at.address)
            .map_or(0, |site| i64::from(site.level) + 1)
    }

    fn span(&self, _world: &World) -> Span {
        Span::new(0, i64::try_from(UPGRADE_LEVEL_COUNT).unwrap_or(1) + 1)
    }

    fn colour(&self, _value: i64) -> u32 {
        UPGRADE_LEVEL_COLOUR
    }
}

/// How full each tile is of the units it admits.
struct Crowding;

impl Layer for Crowding {
    fn name(&self) -> &'static str {
        "crowding"
    }

    fn unit(&self) -> &'static str {
        "of the capacity of the tile, in 255ths"
    }

    fn value(&self, at: At<'_>) -> i64 {
        let capacity = at.world.tile_capacity(at.address).unwrap_or(0);
        if capacity == 0 {
            return 0;
        }
        let standing = at.world.soldier_count_on(at.address).unwrap_or(0);
        let standing = i64::try_from(standing).unwrap_or(i64::MAX);
        (standing * FULL_SHARE / i64::from(capacity)).min(FULL_SHARE)
    }

    fn span(&self, _world: &World) -> Span {
        // The share is of the capacity of the tile, and the engine answers
        // that capacity. Nothing here restates a capacity the engine holds.
        Span::new(0, FULL_SHARE)
    }

    fn colour(&self, _value: i64) -> u32 {
        CROWD_COLOUR
    }
}

/// The overlays the viewer knows, in the order a caller cycles them.
///
/// **This list is the registration.** An overlay that is not here cannot be
/// named, cannot be drawn, and does not reach the boundary. An overlay that is
/// here needs no edit anywhere else.
#[must_use]
pub fn registered() -> &'static [&'static (dyn Layer + 'static)] {
    &[
        &Moisture,
        &Air,
        &Stock {
            kind: ResourceKind::Food,
            name: "food",
            unit: "food on the tile",
            colour: FOOD_COLOUR,
        },
        &Stock {
            kind: ResourceKind::Wood,
            name: "wood",
            unit: "wood on the tile",
            colour: WOOD_COLOUR,
        },
        &Stock {
            kind: ResourceKind::Stone,
            name: "stone",
            unit: "stone on the tile",
            colour: STONE_COLOUR,
        },
        &Height,
        &HolderLayer,
        &Upgrade,
        &Crowding,
    ]
}

/// Returns the names of the registered overlays, in their order.
#[must_use]
pub fn names() -> Vec<&'static str> {
    registered().iter().map(|layer| layer.name()).collect()
}

/// Returns the overlay of this name, or nothing when none carries it.
///
/// A caller that named an overlay the renderer does not hold is refused rather
/// than given a frame with no wash on it. Those two look the same.
#[must_use]
pub fn named(name: &str) -> Option<&'static (dyn Layer + 'static)> {
    registered()
        .iter()
        .copied()
        .find(|layer| layer.name() == name)
}

/// Returns the value one overlay paints at one tile.
///
/// **A value that lives on a level 1 cell does not paint as a rectangle.** A
/// cell is many tiles a side, so a flat read draws the lattice rather than the
/// world.[^1] This reads the four nearest cell centres and interpolates between
/// them, and the quantity then reads as a field with no straight edge that the
/// world does not have.
///
/// The answer is a pure function of the world and the address. Two runs of one
/// frame therefore give one picture.
///
/// # References
///
/// [^1]: Research report 24, defect 6. `docs/research/reports/24-demonstration-readability-resources-and-weather.md`
#[must_use]
pub fn value_of(
    layer: &dyn Layer,
    world: &World,
    address: Axial,
    ground: Option<TerrainTile>,
) -> i64 {
    if !layer.on_cells() {
        return layer.value(At {
            world,
            address,
            ground,
        });
    }
    let layout = world.pyramid().layout();
    let edge = layout.block_edge();
    if edge <= 1 {
        return layer.value(At {
            world,
            address,
            ground: None,
        });
    }
    let grid = world.grid();
    let last_column = i64::from(layout.blocks_wide().saturating_sub(1));
    let last_row = i64::from(layout.blocks_high().saturating_sub(1));
    // The centre of a cell sits half a cell edge inside it. A tile at that
    // centre carries the cell's own value, and a tile half an edge to either
    // side sits half way to the next centre.
    let side = edge as f32;
    let across = (address.q as f32 + 0.5) / side - 0.5;
    let down = (address.r as f32 + 0.5) / side - 0.5;
    let left = across.floor();
    let top = down.floor();
    let share_across = across - left;
    let share_down = down - top;
    let column = left as i64;
    let row = top as i64;

    let at_centre = |cell_column: i64, cell_row: i64| -> f32 {
        let cell_column = cell_column.clamp(0, last_column) as u32;
        let cell_row = cell_row.clamp(0, last_row) as u32;
        let q = (cell_column * edge + edge / 2).min(grid.width().saturating_sub(1));
        let r = (cell_row * edge + edge / 2).min(grid.height().saturating_sub(1));
        layer.value(At {
            world,
            address: Axial::new(q as i32, r as i32),
            ground: None,
        }) as f32
    };

    let upper =
        at_centre(column, row) * (1.0 - share_across) + at_centre(column + 1, row) * share_across;
    let lower = at_centre(column, row + 1) * (1.0 - share_across)
        + at_centre(column + 1, row + 1) * share_across;
    (upper * (1.0 - share_down) + lower * share_down).round() as i64
}

/// What the drawing pass painted of one overlay.
///
/// The span is what the overlay declared for the frame. The lowest and the
/// highest are what the pass met in the window. **A key that named only the
/// span could not tell an empty overlay from a broken one**, and a subsystem
/// that produces no instance looks the same as a subsystem nothing draws.[^1]
///
/// # References
///
/// [^1]: Backlog item 0278, say what the demonstration world never produced. `docs/backlog/complete/0278-say-what-the-demonstration-world-never-produced.md`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Reading {
    /// The name of the overlay that was on.
    pub name: &'static str,
    /// The unit the key names.
    pub unit: &'static str,
    /// The low and the high the overlay declared.
    pub span: Span,
    /// The lowest value the pass painted.
    pub lowest: i64,
    /// The highest value the pass painted.
    pub highest: i64,
    /// How many tiles the pass took a value for.
    pub tiles: u32,
}

impl Reading {
    /// Opens a reading for one overlay and one frame.
    #[must_use]
    pub fn opening(layer: &'static dyn Layer, span: Span) -> Self {
        Self {
            name: layer.name(),
            unit: layer.unit(),
            span,
            lowest: i64::MAX,
            highest: i64::MIN,
            tiles: 0,
        }
    }

    /// Takes one painted value into the reading.
    pub fn saw(&mut self, value: i64) {
        self.lowest = self.lowest.min(value);
        self.highest = self.highest.max(value);
        self.tiles = self.tiles.saturating_add(1);
    }

    /// Reports whether the overlay found nothing in the window.
    ///
    /// A frame that painted no tile, and a frame in which every tile held
    /// nothing, are both nothing to see. The key says so in words either way.
    #[must_use]
    pub const fn found_nothing(&self) -> bool {
        self.tiles == 0 || (self.lowest == 0 && self.highest == 0)
    }
}
