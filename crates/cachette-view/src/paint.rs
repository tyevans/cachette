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
//! takes a border in the same colour when any of its six neighbours holds
//! differently, so a watcher sees the outline of what a faction holds.
//!
//! **The border does not tell a frontier from a coastline.** Unclaimed ground
//! beside a holding draws the same border as another faction beside it. The
//! two are different facts and the picture states them alike.[^6]
//!
//! The colour comes from the one table this module holds. The engine holds no
//! colour, and a second table would be one fact in two places.[^2] [^4]
//!
//! # The condition of a unit
//!
//! A unit keeps the colour of its faction, and one mark says that a shortage
//! holds it. The mark is a dot at half the radius, in one colour, over the
//! disc of the faction. The faction table stays the only table of colours
//! the viewer keys on a faction.[^4]
//!
//! **The picture cannot show a unit at the moment a shortage ends it.** The
//! engine scans the death plane inside the step that takes the unit to the
//! bound, so a unit that a completed step left alive is fed or short and
//! never starved.[^5] The panel states how many the last scan ended, and
//! that count is the only thing a watcher can read about a death.
//!
//! # References
//!
//! [^1]: ADR-0002, simulated and aggregated state holds no floating point number, decision D4. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
//! [^2]: ADR-0067, the viewer reads the world and never writes to it, decision D3. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
//! [^3]: ADR-0017, the world is a rhombus, so a tile index is raw axial, decision D4. `docs/adrs/accepted/adr-0017-the-world-is-a-rhombus-so-a-tile-index-is-raw-axial.md`
//! [^4]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
//! [^5]: Findings register, FND-119. `docs/FINDINGS.md`
//! [^6]: Backlog item 0209. `docs/backlog/proposed/0209-tell-a-frontier-from-the-edge-of-the-claimed-ground.md`

use cachette_core::cohort::NeedCondition;
use cachette_core::founding::FoundingOutcome;
use cachette_core::hex::NEIGHBOURS;
use cachette_core::resource::{ResourceKind, RESOURCE_KIND_COUNT};
use cachette_core::terrain::{TileKind, KIND_COUNT};
use cachette_core::upgrade::{UpgradeKind, UpgradeSite, UPGRADE_KIND_COUNT};
use cachette_core::{Axial, BridgeError, Entity, FactionId, Holder, World};

use crate::text;
use crate::tween::{between, Motion, Pace};

/// The colour each kind of upgrade tints its tile with, by kind ordinal.
///
/// A road is ochre, a terrace is green, a wonder is pale gold and a store is
/// dark brown. The table is indexed by the ordinal the core gives each kind,
/// so a kind that joins the core without a row here fails to compile rather
/// than drawing in a colour nobody chose.
const UPGRADE_COLOURS: [u32; UPGRADE_KIND_COUNT] =
    [0x00c8_9a4a, 0x0052_b86a, 0x00f0_e0a0, 0x0078_5030];

/// How much of the upgrade colour covers a tile whose build has just begun.
const UPGRADE_WEIGHT_FLOOR: i64 = 56;

/// How much of the upgrade colour covers a tile whose build is finished.
///
/// The progress sets the depth between the floor and this ceiling, so a
/// watcher reads a site under construction as paler than a finished one.
const UPGRADE_WEIGHT_CEILING: i64 = 200;

/// The colour of the water in the air, mixed over the tile under it.
const AIR_COLOUR: u32 = 0x00d8_e8f8;

/// The drops of water in the air at which the overlay stops deepening.
///
/// The unit is drops, and a drop is a whole number in the engine. This is a
/// viewer's choice of where the shade saturates, in the same way the food
/// shade saturates at a stock the viewer chose. It is not the engine's
/// figure for a storm, and nothing here reads back into the engine.[^1]
///
/// # References
///
/// [^1]: ADR-0067, the viewer reads the world and never writes to it, decision D2. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
const AIR_AT_FULL_SHADE: i64 = 4096;

/// How much of the air colour covers a tile at the full shade.
const AIR_WEIGHT_CEILING: i64 = 150;

/// The blue the viewer adds to a tile on wet ground.
///
/// **Wet ground moves the blue channel and never the brightness.** The layer
/// used to take the same number from every channel, which is the number the
/// full food ramp added, so the two cancelled exactly.[^1]
///
/// # References
///
/// [^1]: Research report 24, defect 3. `docs/research/reports/24-demonstration-readability-resources-and-weather.md`
const WET_BLUE_GAIN: i32 = 44;

/// The smallest tile width at which the viewer draws the air overlay, in
/// pixels.
///
/// A layer that covers every tile of the picture carries no information and
/// costs contrast. Below this width the overlay is a wash over the whole
/// window, and a watcher reads a pale world rather than a storm.[^1]
///
/// # References
///
/// [^1]: Research report 23, defect 2. `docs/research/reports/23-demonstration-readability-review-1.md`
const AIR_LEAST_TILE: f32 = 8.0;

/// The smallest weight at which the viewer draws the air overlay.
///
/// The air over a cell at rest gives a weight of a few parts in 255. A cell
/// at rest that still tinted its tiles put an edge on the cell lattice that
/// followed nothing in the world.[^1]
///
/// # References
///
/// [^1]: Research report 24, defect 10. `docs/research/reports/24-demonstration-readability-resources-and-weather.md`
const AIR_LEAST_WEIGHT: u8 = 8;

/// The stride the luxury hue turns by, for each step of the kind ordinal.
///
/// The stride is odd and shares no factor with the wheel, so the kinds the
/// catalogue numbers together do not draw together.
const LUXURY_HUE_STRIDE: u32 = 37;

/// The smallest tile width at which the viewer draws a deposit pip, in
/// pixels.
///
/// A pip needs a few pixels of its own and a margin around it. Below this
/// width the tile has no room, and the pips would read as speckle.[^1]
///
/// # References
///
/// [^1]: Research report 24, defect 5. `docs/research/reports/24-demonstration-readability-resources-and-weather.md`
const PIP_LEAST_TILE: f32 = 16.0;

/// The smallest tile width at which a build site draws its glyph, in pixels.
///
/// A glyph needs a few pixels of shape, and a window at the region scale
/// holds thousands of tiles. Hundreds of small glyphs are speckle, and the
/// wash is the better mark at that zoom because a field of built tiles reads
/// as one field.[^1]
///
/// The width is the width at which the tile also carries a gap and a deposit
/// pip, so the close zooms carry every per-tile mark together and the far
/// zooms carry none of them.
///
/// # References
///
/// [^1]: Research report 25, defect 1. `docs/research/reports/25-demonstration-readability-upgrades-and-units.md`
const SITE_LEAST_TILE: f32 = 16.0;

/// The stock at which a deposit pip draws at its largest.
///
/// This is a property of the picture and not of the world, in the same way
/// the food ramp bound is.
const PIP_AT_FULL_SIZE: i32 = 8;

/// One colour for each kind of resource, in the order of the kinds.
///
/// A pip carries the colour of what the ground holds. The colours are the
/// viewer's own, and the engine holds none.[^1]
///
/// # References
///
/// [^1]: ADR-0067, the viewer reads the world and never writes to it, decision D2. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
const PIP_COLOURS: [u32; RESOURCE_KIND_COUNT] = [
    // Food. Pale green.
    0x00b8_e05a,
    // Wood. Warm brown.
    0x008a_5a2a,
    // Stone. Pale grey.
    0x00c8_ccd0,
];

/// The colour of the space outside the world.
///
/// The gap between two tiles shows this colour, so a caller that counts the
/// grid a watcher sees reads it from here rather than repeating the value.
pub const BACKGROUND: u32 = 0x0010_1418;

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

/// How much a full deposit raises the saturation of a tile, in parts of 255.
///
/// The ground is fixed for the life of a world. The food on it is not: the
/// ground generates a stock, a gatherer takes from it, and the recovery pass
/// gives part of it back.[^1] A watcher therefore reads a deposit drain and
/// recover from the colour of the ground it sits on.
///
/// **The food moves the saturation and the height moves the brightness.**
/// The two used to add into one brightness, and a wet tile with the most food
/// then drew as the same colour as a dry tile with none, because the wet
/// shade took the same number from every channel.[^2] Two layers that move
/// one axis in opposite directions carry no information between them.
///
/// # References
///
/// [^1]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D4. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
/// [^2]: Research report 24, defects 3 and 9. `docs/research/reports/24-demonstration-readability-resources-and-weather.md`
const FOOD_SATURATION: i32 = 150;

/// The food at which a tile draws at its brightest.
///
/// This is a property of the picture and not of the world. It says where the
/// ramp saturates, so a tile that carries more food than this draws the same
/// as a tile that carries exactly this. It is deliberately below the largest
/// stock the ground generates, because the ramp must separate an empty tile
/// from a small deposit, and the deposits a watcher cares about are the ones
/// a crowd can drain.
///
/// It is not a copy of the engine's ceiling. A ceiling that moves leaves this
/// statement true, because the statement is about the colour and not about
/// the stock.[^1]
///
/// # References
///
/// [^1]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
const FOOD_AT_FULL_SHADE: u32 = 8;

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
/// [^1]: PRD-0006, a place belongs to somebody. `docs/product/accepted/prd-0006-a-place-belongs-to-somebody.md`
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

/// Returns the colour the viewer marks a unit that a shortage holds.
///
/// One colour marks the condition, and the faction colour table stays the
/// only table of colours the viewer keys on a faction.[^1] [^2]
///
/// A test reads this rather than a literal, so the mark has one declaration
/// site.[^2]
///
/// # References
///
/// [^1]: ADR-0067, the viewer reads the world and never writes to it, decision D2. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
/// [^2]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
#[must_use]
pub const fn shortage_colour() -> u32 {
    SHORTAGE
}

/// Returns the colour the viewer draws one kind of upgrade in.
///
/// A test and the colour key read this rather than a literal, so the table
/// has one declaration site.[^1]
///
/// # References
///
/// [^1]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
#[must_use]
pub fn upgrade_colour(kind: UpgradeKind) -> u32 {
    UPGRADE_COLOURS[kind.index()]
}

/// Returns the colour the viewer draws the pip of one resource in.
///
/// A test and the colour key read this rather than a literal, so the table
/// has one declaration site.[^1]
///
/// # References
///
/// [^1]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
#[must_use]
pub fn resource_pip_colour(kind: ResourceKind) -> u32 {
    PIP_COLOURS[kind as usize]
}

/// Returns the colour of the rim the viewer draws around a unit disc.
///
/// The rim is what makes a unit visible over ground its own faction tints,
/// and below sixteen pixels a tile it is most of the unit.[^1] A test reads
/// this rather than a literal, so the rim has one declaration site.[^2]
///
/// # References
///
/// [^1]: Research report 23, defect 1. `docs/research/reports/23-demonstration-readability-review-1.md`
/// [^2]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
#[must_use]
pub const fn unit_rim_colour() -> u32 {
    UNIT_RIM
}

/// Returns the smallest weight at which the viewer draws the air overlay.
///
/// A test reads this rather than a literal, so the floor has one declaration
/// site.[^1]
///
/// # References
///
/// [^1]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
#[must_use]
pub const fn air_least_weight() -> u8 {
    AIR_LEAST_WEIGHT
}

/// Returns the colour the viewer mixes over a tile for the water in the air.
///
/// A test reads this rather than a literal, so the colour has one
/// declaration site.[^1]
///
/// # References
///
/// [^1]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
#[must_use]
pub const fn air_colour() -> u32 {
    AIR_COLOUR
}

/// Returns two colours mixed by a weight, one channel at a time.
///
/// A test that must state what a layer would have given reads this rather
/// than repeating the arithmetic.[^1]
///
/// # References
///
/// [^1]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
#[must_use]
pub fn mixed(under: u32, over: u32, weight: u8) -> u32 {
    mix(under, over, weight)
}

/// Returns the colour the viewer marks a place a faction founded in.
///
/// The founding mark carries the colour of the faction that founded, from the
/// one table this module holds. A watcher reads the mark and the units of
/// that faction as one colour.[^1]
///
/// The core of the mark is the same for every faction, so a mark stays
/// visible over any ground.[^2]
///
/// # References
///
/// [^1]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
/// [^2]: PRD-0005, a watcher can tell what is happening and why. `docs/product/shipped/prd-0005-a-watcher-can-tell-what-is-happening-and-why.md`
#[must_use]
pub const fn founding_core_colour() -> u32 {
    FOUNDING_CORE
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

/// The share of the window that one press of a scroll key moves the view.
///
/// **A pan covers a share of what the window shows, not a count of tiles.** A
/// step in tiles is the same number of tiles at every zoom, so it is a
/// different number of pixels. At the smallest tile the camera allows it moved
/// three pixels, and the camera felt stuck. Nothing was slow. The step was the
/// wrong size for the view.[^1]
///
/// The share is the share the old step covered at the zoom the viewer opens
/// on. That step was one and a half tiles of twelve pixels, which is eighteen
/// pixels, and the window the demonstration opens is seven hundred and twenty
/// pixels on its shorter side. Eighteen in seven hundred and twenty is one in
/// forty.
///
/// **The one zoom nobody reported is therefore unchanged, and every other zoom
/// now matches it.** The value preserves a behaviour rather than improving on
/// it, so no part of it was read off a render.
///
/// # References
///
/// [^1]: Findings register, FND-209. `docs/FINDINGS.md`
const PAN_SHARE: f32 = 1.0 / 40.0;

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

/// The colour of the mark on a unit that a shortage holds.
///
/// The engine names the condition. It does not say what a condition looks
/// like, and it never will.[^1]
///
/// The colour is far from every faction colour and from every ground colour,
/// so a watcher reads the mark against the disc it sits on and against the
/// ground behind it. A test asserts that distance.
///
/// # References
///
/// [^1]: ADR-0067, the viewer reads the world and never writes to it, decision D2. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
const SHORTAGE: u32 = 0x00f2_f0d8;

/// The colour at the middle of a founding mark.
///
/// A founding mark is a ring in the faction's colour around this core. The
/// core is one colour for every faction, so a watcher finds a founding on any
/// ground and then reads the ring for the faction that took it.
const FOUNDING_CORE: u32 = 0x0014_0b04;

/// The smallest radius a unit disc takes, in pixels.
///
/// A disc of three tenths of the tile is one pixel across at the region
/// scale, in the colour of the faction, over ground the same faction tints.
/// A watcher then reads a world of no people. The floor holds the disc above
/// the ground speckle at every zoom.[^1]
///
/// # References
///
/// [^1]: Research report 23, defect 1. `docs/research/reports/23-demonstration-readability-review-1.md`
const UNIT_LEAST_RADIUS: i32 = 3;

/// The colour of the rim around a unit disc.
///
/// Every faction colour collides with the ground its own faction holds,
/// because the tint and the disc carry one colour at two weights. The rim is
/// darker than any ground colour, so the shape of a unit reads against the
/// tint under it.[^1]
///
/// # References
///
/// [^1]: Research report 23, defect 1. `docs/research/reports/23-demonstration-readability-review-1.md`
const UNIT_RIM: u32 = 0x0008_0a0c;

/// The tile width from which a crowd shows its count as a badge, in pixels.
///
/// Above this width a tile has room for a number beside the disc. Below it
/// the disc grows with the count instead, because a glyph of six pixels does
/// not fit on a tile of twelve.[^1]
///
/// # References
///
/// [^1]: Research report 25, defect 4. `docs/research/reports/25-demonstration-readability-upgrades-and-units.md`
const CROWD_BADGE_TILE: f32 = 24.0;

/// The tile width from which a unit shows where it came from, in pixels.
///
/// The line runs from the tile the unit left to the tile it stands on. Below
/// this width the line is shorter than the disc and reads as noise.[^1]
///
/// # References
///
/// [^1]: Research report 23, defect 3. `docs/research/reports/23-demonstration-readability-review-1.md`
const HEADING_TILE: f32 = 24.0;

/// The smallest side a founding mark takes, in pixels.
///
/// The mark is three nested rings, so it needs at least five pixels a side.
/// A watcher who zooms out to a tile of two pixels still finds the places
/// that founded.
///
/// **A seven pixel square is the size of a unit and the shape of the grid.**
/// A watcher who did not know where to look did not find it. The floor is
/// the side at which the three rings each separate.[^1]
///
/// # References
///
/// [^1]: Research report 23, defect 10. `docs/research/reports/23-demonstration-readability-review-1.md`
const FOUNDING_LEAST_SIDE: i32 = 15;

/// A pixel buffer that the viewer paints and the window shows.
/// The drawn unit nearest the middle of the window.
///
/// The panel reports why one unit chose what it chose, and it must name a
/// unit to do that. The viewer has no cursor, so the middle of the window is
/// the pointer: a watcher who wants a different unit scrolls until that unit
/// is in the middle.[^1]
///
/// The drawing pass fixes this while it paints. It compares the position it
/// already computed for a unit against the middle of the canvas, which costs
/// one comparison for each unit the pass was already painting. The panel
/// therefore starts no pass over the world to find a unit.[^2]
///
/// The comparison is strict, so the first unit at a distance keeps it. The
/// drawing pass visits the blocks in ascending block order and the units of a
/// block in tile order, so the same world and the same camera name the same
/// unit.[^3]
///
/// # References
///
/// [^1]: Decisions register, DEC-077. `docs/DECISIONS.md`
/// [^2]: ADR-0070, the head-up display reports what the drawing pass read, decision D1. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
/// [^3]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D2. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
#[derive(Clone, Copy, Debug)]
pub struct Focus {
    entity: Entity,
    address: Axial,
    faction: FactionId,
    condition: Option<NeedCondition>,
    reach: i64,
}

impl Focus {
    /// Returns the identity of the unit.
    #[must_use]
    pub const fn entity(&self) -> Entity {
        self.entity
    }

    /// Returns the tile the unit stands on.
    #[must_use]
    pub const fn address(&self) -> Axial {
        self.address
    }

    /// Returns the faction of the unit.
    #[must_use]
    pub const fn faction(&self) -> FactionId {
        self.faction
    }

    /// Returns the condition the engine gives the unit.
    ///
    /// Returns `None` when the engine names no condition for it. The viewer
    /// invents none, because the rule that decides a condition lives in the
    /// engine and a second copy of it here would be one rule in two
    /// places.[^1]
    ///
    /// # References
    ///
    /// [^1]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
    #[must_use]
    pub const fn condition(&self) -> Option<NeedCondition> {
        self.condition
    }

    /// Returns the squared distance in pixels from the middle of the window.
    ///
    /// The square is kept rather than the root, because the comparison needs
    /// no root and the viewer states what it measured.
    #[must_use]
    pub const fn reach(&self) -> i64 {
        self.reach
    }
}

/// Where the pixels of a canvas live.
///
/// A canvas either owns its pixels or writes into memory a caller lent it.
/// The drawing does not know which, and no drawing routine may learn: the
/// difference is the caller's, and the picture is the same either way.
///
/// **The borrowed form is what makes a frame a command.** A caller supplies
/// the memory, the engine writes one frame into it and returns, and the
/// engine keeps no reference afterwards. The borrow checker holds that,
/// rather than a comment.[^1]
///
/// # References
///
/// [^1]: ADR-0094, the caller owns the camera and the pixels, decision D2. `docs/adrs/draft/adr-0094-the-caller-owns-the-camera-and-the-pixels.md`
enum Pixels<'a> {
    /// The canvas allocated the pixels and drops them with itself.
    Owned(Vec<u32>),
    /// The pixels belong to a caller and outlive the canvas.
    Borrowed(&'a mut [u32]),
}

impl Pixels<'_> {
    /// Returns the pixels to read.
    fn get(&self) -> &[u32] {
        match self {
            Self::Owned(owned) => owned,
            Self::Borrowed(borrowed) => borrowed,
        }
    }

    /// Returns the pixels to write.
    fn get_mut(&mut self) -> &mut [u32] {
        match self {
            Self::Owned(owned) => owned,
            Self::Borrowed(borrowed) => borrowed,
        }
    }
}

pub struct Canvas<'a> {
    width: usize,
    height: usize,
    pixels: Pixels<'a>,
    tiles_painted: u32,
    soldiers_painted: u32,
    blocks_read: u32,
    blocks_skipped: u32,
    painted_by_faction: [u32; COLOURED_FACTIONS],
    painted_by_kind: [u32; KIND_COUNT],
    holder_reads: u32,
    ground_reads: u32,
    tiles_held: u32,
    crowd_worst: u32,
    tiles_at_capacity: u32,
    condition_reads: u32,
    units_short: u32,
    carry_reads: u32,
    units_carrying: u32,
    carried_by_kind: [u32; RESOURCE_KIND_COUNT],
    home_reads: u32,
    units_housed: u32,
    foundings_marked: u32,
    focus: Option<Focus>,
}

impl<'a> Canvas<'a> {
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
            pixels: Pixels::Owned(vec![BACKGROUND; width * height]),
            tiles_painted: 0,
            soldiers_painted: 0,
            blocks_read: 0,
            blocks_skipped: 0,
            painted_by_faction: [0; COLOURED_FACTIONS],
            painted_by_kind: [0; KIND_COUNT],
            holder_reads: 0,
            ground_reads: 0,
            tiles_held: 0,
            crowd_worst: 0,
            tiles_at_capacity: 0,
            condition_reads: 0,
            units_short: 0,
            carry_reads: 0,
            units_carrying: 0,
            carried_by_kind: [0; RESOURCE_KIND_COUNT],
            home_reads: 0,
            units_housed: 0,
            foundings_marked: 0,
            focus: None,
        }
    }

    /// Builds a canvas that draws into memory a caller lent it.
    ///
    /// The canvas writes the pixels and never frees them. The caller owns the
    /// memory before the call and owns it afterwards, and the borrow checker
    /// stops the canvas outliving it.[^1]
    ///
    /// The counts start at zero, in the same way they do for an owned canvas.
    /// The pixels are not cleared here, because the drawing pass clears them.
    ///
    /// **The length is checked by the frame command, not here.** A caller
    /// reaches this through that command, which refuses a buffer of the wrong
    /// size and names the size it needed.[^1]
    ///
    /// # Panics
    ///
    /// Panics when either side is zero, or when the slice does not hold one
    /// pixel for each pixel of the canvas. Both are refused before this point
    /// on every path a caller can take.
    ///
    /// # References
    ///
    /// [^1]: ADR-0094, the caller owns the camera and the pixels, decision D2. `docs/adrs/draft/adr-0094-the-caller-owns-the-camera-and-the-pixels.md`
    #[must_use]
    pub fn borrowing(pixels: &'a mut [u32], width: usize, height: usize) -> Self {
        assert!(width > 0 && height > 0, "a canvas needs a positive size");
        assert_eq!(
            pixels.len(),
            width * height,
            "a borrowed canvas needs one pixel for each pixel of the frame"
        );
        Self {
            pixels: Pixels::Borrowed(pixels),
            width,
            height,
            tiles_painted: 0,
            soldiers_painted: 0,
            blocks_read: 0,
            blocks_skipped: 0,
            painted_by_faction: [0; COLOURED_FACTIONS],
            painted_by_kind: [0; KIND_COUNT],
            holder_reads: 0,
            ground_reads: 0,
            tiles_held: 0,
            crowd_worst: 0,
            tiles_at_capacity: 0,
            condition_reads: 0,
            units_short: 0,
            carry_reads: 0,
            units_carrying: 0,
            carried_by_kind: [0; RESOURCE_KIND_COUNT],
            home_reads: 0,
            units_housed: 0,
            foundings_marked: 0,
            focus: None,
        }
    }

    /// Returns the pixels, for the window to show.
    #[must_use]
    pub fn pixels(&self) -> &[u32] {
        self.pixels.get()
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

    /// Returns the number of units the last draw asked for a load.
    ///
    /// The drawing asks at every unit it paints and nowhere else, so the
    /// count is a function of the window and never of the world.[^1] A test
    /// reads it, because a layer that swept the arena would report the same
    /// totals and cost the population.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0070, the head-up display reports what the drawing pass read, decision D1. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
    /// [^2]: Findings register, FND-071. `docs/FINDINGS.md`
    #[must_use]
    pub const fn carry_reads(&self) -> u32 {
        self.carry_reads
    }

    /// Returns the number of units the last draw asked for a home.
    ///
    /// The count is a function of the window, in the same way the load count
    /// is.
    #[must_use]
    pub const fn home_reads(&self) -> u32 {
        self.home_reads
    }

    /// Returns the number of painted units that carry something.
    #[must_use]
    pub const fn units_carrying(&self) -> u32 {
        self.units_carrying
    }

    /// Returns what the painted units carry, one total for each kind.
    #[must_use]
    pub const fn carried_by_kind(&self) -> &[u32; RESOURCE_KIND_COUNT] {
        &self.carried_by_kind
    }

    /// Returns the number of painted units that hold a home site.
    #[must_use]
    pub const fn units_housed(&self) -> u32 {
        self.units_housed
    }

    /// Returns the number of times the last draw generated a ground.
    ///
    /// The ground of a tile is generated from the seed and the address, and
    /// the engine holds no map of it.[^1] The generation is the largest part
    /// of what a drawing costs, so the count of generations is the number
    /// that says whether a change to this layer worked.
    ///
    /// The counter stands at the one site in the drawing that generates a
    /// ground. A test reads it against the count of painted tiles, because a
    /// drawing that generated the ground of each tile twice would paint the
    /// same picture.[^2]
    ///
    /// **The counter counts the calls this layer makes, not the generations
    /// the engine runs.** A reader below this layer that generated a ground
    /// of its own would not appear here.
    ///
    /// # References
    ///
    /// [^1]: ADR-0068, terrain is generated from the seed and is never stored as a map, decision D1. `docs/adrs/accepted/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
    /// [^2]: ADR-0070, the head-up display reports what the drawing pass read, decision D1. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
    #[must_use]
    pub const fn ground_reads(&self) -> u32 {
        self.ground_reads
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
    /// [^1]: PRD-0002, a developer watches the world run. `docs/product/shipped/prd-0002-a-developer-watches-the-world-run.md`
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
    /// [^2]: PRD-0003, a developer sees a world worth looking at. `docs/product/accepted/prd-0003-a-developer-sees-a-world-worth-looking-at.md`
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
    /// **The viewer must not build a census by reading every soldier.** A
    /// count of the whole world is a pass over the whole world, and a panel
    /// starts no such pass.[^2]
    ///
    /// The engine does answer two other questions, and neither derives from
    /// this count. It counts one bounded window of addresses, whether or not
    /// anything drew them.[^3] It also holds a running population for each
    /// faction, which it maintains where a unit is created and where a unit
    /// ends, so that read costs one load.[^4] Use those rather than counting
    /// here.
    ///
    /// # References
    ///
    /// [^1]: PRD-0002, a developer watches the world run. `docs/product/shipped/prd-0002-a-developer-watches-the-world-run.md`
    /// [^2]: ADR-0070, the head-up display reports what the drawing pass read, decision D1. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
    /// [^3]: The window census. `crates/cachette-core/src/census.rs`
    /// [^4]: The per-faction population of the soldier arena. `crates/cachette-core/src/soldier.rs`
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

    /// Returns the painted tiles that hold at least as many units as they
    /// admit.
    ///
    /// The capacity is the composition of the ground and the finished
    /// upgrade, which is what admission reads. The viewer holds no capacity
    /// value of its own and reads neither table directly, so a change to
    /// either one reaches the picture with no edit here.[^1] [^3]
    ///
    /// The count is of the window, in the same way every other count of the
    /// drawing pass is.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D4. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
    /// [^2]: ADR-0070, the head-up display reports what the drawing pass read, decision D2. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
    /// [^3]: Findings register, FND-193. `docs/FINDINGS.md`
    #[must_use]
    pub const fn tiles_at_capacity(&self) -> u32 {
        self.tiles_at_capacity
    }

    /// Fills a rectangle with one colour.
    ///
    /// The head-up display draws its panel with this. A position outside the
    /// canvas is clipped rather than a panic.
    /// Returns the number of conditions the pass read.
    ///
    /// The pass reads the condition of every unit it paints, and of no other
    /// unit. A count above the units painted says that the layer started a
    /// pass of its own, which the panel record forbids.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0070, the head-up display reports what the drawing pass read, decision D1. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
    #[must_use]
    pub const fn condition_reads(&self) -> u32 {
        self.condition_reads
    }

    /// Returns the number of painted units that a shortage holds.
    ///
    /// This counts the units the pass painted, and never the units of the
    /// world. A watcher tells the two apart by the label of the panel row.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0070, the head-up display reports what the drawing pass read, decision D2. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
    #[must_use]
    pub const fn units_short(&self) -> u32 {
        self.units_short
    }

    /// Returns the number of founding marks the pass painted.
    ///
    /// A founding whose place lies outside the window paints no mark and is
    /// not counted, in the same way that a unit outside the window is not
    /// counted.
    #[must_use]
    pub const fn foundings_marked(&self) -> u32 {
        self.foundings_marked
    }

    /// Returns the drawn unit nearest the middle of the window.
    ///
    /// Returns `None` when the pass painted no unit. The panel then says that
    /// the window holds nobody, rather than naming a unit it did not
    /// draw.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0070, the head-up display reports what the drawing pass read, decision D2. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
    #[must_use]
    pub const fn focus(&self) -> Option<Focus> {
        self.focus
    }

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
        Some(self.pixels.get()[y * self.width + x])
    }

    /// Fills the whole canvas with the background.
    ///
    /// The counts reset here, so they always describe one draw.
    pub fn clear(&mut self) {
        self.pixels.get_mut().fill(BACKGROUND);
        self.tiles_painted = 0;
        self.soldiers_painted = 0;
        self.blocks_read = 0;
        self.blocks_skipped = 0;
        self.painted_by_faction = [0; COLOURED_FACTIONS];
        self.painted_by_kind = [0; KIND_COUNT];
        self.holder_reads = 0;
        self.ground_reads = 0;
        self.tiles_held = 0;
        self.crowd_worst = 0;
        self.tiles_at_capacity = 0;
        self.condition_reads = 0;
        self.units_short = 0;
        self.carry_reads = 0;
        self.units_carrying = 0;
        self.carried_by_kind = [0; RESOURCE_KIND_COUNT];
        self.home_reads = 0;
        self.units_housed = 0;
        self.foundings_marked = 0;
        self.focus = None;
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
        self.pixels.get_mut()[y * self.width + x] = colour;
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

    /// Fills a disc divided into one wedge for each colour, inside a rim.
    ///
    /// **A tile that two factions stand on shows both.** The pass used to
    /// paint one disc for each unit at the same centre, so the last unit the
    /// structure gave covered every earlier one and the colour of a shared
    /// tile was the colour of whichever unit came last.[^1]
    ///
    /// The wedges run in the order the caller gives, which is ascending
    /// colour order, so the picture does not depend on the order the units
    /// arrived in.[^2]
    ///
    /// # References
    ///
    /// [^1]: Research report 25, defect 5. `docs/research/reports/25-demonstration-readability-upgrades-and-units.md`
    /// [^2]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    fn fill_wedges(&mut self, cx: i32, cy: i32, radius: i32, colours: &[u32], rim: u32) {
        if colours.is_empty() {
            return;
        }
        let inner = (radius - 1).max(0);
        for row in -radius..=radius {
            for column in -radius..=radius {
                let far = column * column + row * row;
                if far > radius * radius {
                    continue;
                }
                let colour = if far > inner * inner {
                    rim
                } else {
                    colours[wedge_of(column, row, colours.len())]
                };
                self.put(cx + column, cy + row, colour);
            }
        }
    }

    /// Draws a straight line of one pixel between two points.
    ///
    /// The step count comes from the longer side, so the line has no gap.
    fn line(&mut self, from: (f32, f32), to: (f32, f32), colour: u32) {
        let (across, down) = (to.0 - from.0, to.1 - from.1);
        let steps = across.abs().max(down.abs()).ceil().max(1.0);
        let count = steps as i32;
        for step in 0..=count {
            let share = step as f32 / steps;
            self.put(
                (from.0 + across * share) as i32,
                (from.1 + down * share) as i32,
                colour,
            );
        }
    }
}

/// Returns the wedge that a pixel of a disc falls in.
///
/// The angle runs from the direction of the negative horizontal axis, so a
/// disc of two wedges splits left and right and the split does not depend on
/// the arithmetic of the caller.
fn wedge_of(column: i32, row: i32, count: usize) -> usize {
    let angle = (row as f32).atan2(column as f32);
    let share = (angle + std::f32::consts::PI) / (2.0 * std::f32::consts::PI);
    ((share * count as f32) as usize).min(count - 1)
}

/// Where the world sits on the screen.
///
/// The camera holds floating point, and it is the viewer's own. Nothing here
/// reaches the engine.[^1]
///
/// # References
///
/// [^1]: ADR-0067, the viewer reads the world and never writes to it, decision D2. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
/// The size of the surface a camera aims at.
///
/// **A camera verb needs the width and the height of the picture, and nothing
/// else.** It reads no pixel. Taking the size rather than the surface lets a
/// caller that holds no surface still steer: the control plane owns a camera
/// and a window size, asks for a frame once, and never allocates a canvas of
/// its own.[^1]
///
/// # References
///
/// [^1]: ADR-0094, the caller owns the camera and the pixels, decision D3. `docs/adrs/draft/adr-0094-the-caller-owns-the-camera-and-the-pixels.md`
pub trait Extent {
    /// Returns the width in pixels.
    fn width(&self) -> usize;
    /// Returns the height in pixels.
    fn height(&self) -> usize;
}

impl Extent for Canvas<'_> {
    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }
}

/// A picture size with no pixels behind it.
///
/// A caller that steers a camera before it draws needs the size and not the
/// memory. This is that size.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameSize {
    width: usize,
    height: usize,
}

impl FrameSize {
    /// Builds a size.
    #[must_use]
    pub const fn new(width: usize, height: usize) -> Self {
        Self { width, height }
    }
}

impl Extent for FrameSize {
    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }
}

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
    pub fn fitting(world: &World, canvas: &impl Extent) -> Self {
        let (spans_across, spans_down) = drawn_extent(world);
        let by_width = canvas.width() as f32 / spans_across;
        let by_height = canvas.height() as f32 / spans_down;
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

    /// Returns the camera moved by whole presses of a scroll key.
    ///
    /// **This is the call a person drives.** The step is a share of the
    /// window, so one press moves the view by the same part of the picture at
    /// every zoom.[^1] A caller that wants to move by a count of tiles uses
    /// the tile form below, which is what a test wants and what a person does
    /// not.
    ///
    /// The step is square in pixels, and it comes from the shorter side of the
    /// window. A step taken from each side separately would move the view
    /// further across than down, which is a second change that nobody asked
    /// for.
    ///
    /// # References
    ///
    /// [^1]: Findings register, FND-209. `docs/FINDINGS.md`
    #[must_use]
    pub fn nudged(self, across: f32, down: f32, canvas: &impl Extent) -> Self {
        let shorter = canvas.width().min(canvas.height()) as f32;
        let step = shorter * PAN_SHARE;
        self.panned(across * step, down * step)
    }

    /// Returns the camera moved by a whole number of tiles.
    ///
    /// A caller that steers by keyboard uses the press form above. This form
    /// moves by a count of tiles, which changes its pixel distance with the
    /// zoom.
    #[must_use]
    pub fn stepped(self, across: f32, down: f32) -> Self {
        self.panned(across * self.tile_width, down * self.tile_height)
    }

    /// Returns the camera one step closer to the world.
    #[must_use]
    pub fn zoomed_in(self, canvas: &impl Extent) -> Self {
        self.zoomed(ZOOM_STEP, canvas)
    }

    /// Returns the camera one step further from the world.
    #[must_use]
    pub fn zoomed_out(self, canvas: &impl Extent) -> Self {
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
    pub fn looking_at(self, address: Axial, canvas: &impl Extent) -> Self {
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
    pub fn zoomed(self, factor: f32, canvas: &impl Extent) -> Self {
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
    pub fn clamped(self, world: &World, canvas: &impl Extent) -> Self {
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
    pub fn visible_rows(self, world: &World, canvas: &impl Extent) -> (u32, u32) {
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
    pub fn visible_columns(self, row: u32, world: &World, canvas: &impl Extent) -> (u32, u32) {
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

/// Returns the extent of the shape a world draws as, in tiles.
///
/// The world is a rhombus in the index space, so it draws as a
/// parallelogram.[^1] Each row shifts right by half a tile, so the shape is
/// wider than the grid by half its height, and it is never as tall as it is
/// wide unless the world is much taller than it is broad.
///
/// **This is the one statement of that shape.** The camera that fits a world
/// into a canvas reads it, and so does the canvas that suits a world. Two
/// statements of one shape would let a picture leave a void that the camera
/// did not expect.[^2]
///
/// # References
///
/// [^1]: ADR-0017, the world is a rhombus, so a tile index is raw axial, decision D4. `docs/adrs/accepted/adr-0017-the-world-is-a-rhombus-so-a-tile-index-is-raw-axial.md`
/// [^2]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
fn drawn_extent(world: &World) -> (f32, f32) {
    let grid = world.grid();
    let across = grid.width() as f32;
    let down = grid.height() as f32;
    (across + down / 2.0 + 1.0, down + 1.0)
}

/// Returns the canvas size that a world fills with no empty band.
///
/// A world draws as a parallelogram, and a parallelogram does not fill a
/// square. A caller that asks for a square canvas and then fits a world into
/// it gets a picture whose bottom third is empty, because the width binds and
/// the height does not.
///
/// This returns the size whose proportions match the shape, so that fitting
/// the world into it leaves no band. The longer side is the size the caller
/// asked for.
///
/// # Panics
///
/// Panics when the long side is zero. A picture of no size is a programming
/// error in the caller.
#[must_use]
pub fn canvas_for(world: &World, long_side: usize) -> (usize, usize) {
    assert!(long_side > 0, "a picture needs a positive size");
    let (across, down) = drawn_extent(world);
    let longest = across.max(down);
    let scale = long_side as f32 / longest;
    (
        ((across * scale).round() as usize).max(1),
        ((down * scale).round() as usize).max(1),
    )
}

/// Returns the colour of one tile.
///
/// The kind chooses the colour and the height brightens it, so a person reads
/// the relief of the ground as well as its kind.[^1] The food the tile still
/// holds brightens it further, so a person reads where the food is and
/// watches a deposit drain and recover.[^3]
///
/// The height is a fixed-point number in the engine and the food is a whole
/// number of units. The viewer turns both into a brightness. That is a
/// conversion out of exact arithmetic, and nothing here goes back into
/// it.[^2]
///
/// # References
///
/// [^1]: PRD-0003, a developer sees a world worth looking at. `docs/product/accepted/prd-0003-a-developer-sees-a-world-worth-looking-at.md`
/// [^2]: ADR-0067, the viewer reads the world and never writes to it, decision D3. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
/// [^3]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D4. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
fn tile_colour(kind: TileKind, height: i32, food: u32) -> u32 {
    let base = KIND_COLOURS[kind.to_u8() as usize];
    // The height is a fraction of the full range in Q16.16, so the unit is
    // 65536. The shift maps the fraction onto the brightness steps.
    let relief = (height.clamp(0, 0x0001_0000) * HEIGHT_STEPS) >> 16;
    let channel = |offset: u32| (((base >> offset) & 0xff) as i32 + relief).clamp(0, 0xff) as u32;
    let lit = (channel(16) << 16) | (channel(8) << 8) | channel(0);
    // The food is a whole number of units. The ramp saturates at the bound
    // the viewer chose, so a tile above it draws the same as a tile at it.
    let stock = food.min(FOOD_AT_FULL_SHADE) as i32;
    let share = (stock * FOOD_SATURATION) / FOOD_AT_FULL_SHADE as i32;
    saturated(lit, share)
}

/// Returns a colour with its channels moved away from their own mean.
///
/// The mean is the brightness of the colour, and it does not move. The share
/// is in parts of 255, so a share of zero returns the colour unchanged.
///
/// The arithmetic is on whole numbers and the result goes to a pixel and
/// nowhere else.[^1]
///
/// # References
///
/// [^1]: ADR-0067, the viewer reads the world and never writes to it, decision D3. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
fn saturated(colour: u32, share: i32) -> u32 {
    let channel = |offset: u32| ((colour >> offset) & 0xff) as i32;
    let (red, green, blue) = (channel(16), channel(8), channel(0));
    let mean = (red + green + blue) / 3;
    let moved = |value: i32| (mean + (value - mean) * (255 + share) / 255).clamp(0, 0xff) as u32;
    (moved(red) << 16) | (moved(green) << 8) | moved(blue)
}

/// Returns a colour with the blue of wet ground added to it.
///
/// The red and the green do not move, so the layer cannot cancel the food
/// ramp or the relief.[^1]
///
/// # References
///
/// [^1]: Research report 24, defect 3. `docs/research/reports/24-demonstration-readability-resources-and-weather.md`
fn wetted(colour: u32, gain: i32) -> u32 {
    let blue = ((colour & 0xff) as i32 + gain).clamp(0, 0xff) as u32;
    (colour & 0x00ff_ff00) | blue
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
/// [^5]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D4. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
/// [^6]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D1. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
/// [^7]: ADR-0140, weather is a field over the level 1 cell lattice, decision D1. `docs/adrs/draft/adr-0140-weather-is-a-field-over-the-level-1-cell-lattice.md`
/// [^8]: ADR-0090, a tile upgrade is stored sparsely, as the difference from the generated world, decision D1. `docs/adrs/draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md`
pub fn draw(world: &World, camera: Camera, canvas: &mut Canvas) -> Result<(), BridgeError> {
    let mut motion = Motion::none();
    draw_paced(world, camera, canvas, Pace::STILL, &mut motion)
}

/// Draws the world onto the canvas, at a pace the caller sets.
///
/// **This is the one drawing pass.** The call above is this call at a still
/// pace, so there is one renderer and not two.[^9]
///
/// The pace says how far the wall clock is through the current tick. A unit
/// that moved to a neighbouring tile since the table last saw it draws
/// between the two tile centres at that share. A unit the table does not
/// hold, and a unit that jumped further than the reach, draws at its tile.
///
/// The table is the caller's memory of the last frame. The world holds one
/// tick at a time, so the memory cannot be the engine's.[^10]
///
/// **The order in which units are painted does not change.** The pass visits
/// the same blocks and the same units in the same order, and the table is
/// read by key and never iterated.
///
/// # Errors
///
/// Returns an error when the engine's spatial structure no longer describes
/// its soldiers.
///
/// # References
///
/// [^9]: ADR-0094, the caller owns the camera and the pixels, decision D5. `docs/adrs/draft/adr-0094-the-caller-owns-the-camera-and-the-pixels.md`
/// [^10]: ADR-0067, the viewer reads the world and never writes to it, decision D2. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
pub fn draw_paced(
    world: &World,
    camera: Camera,
    canvas: &mut Canvas,
    pace: Pace,
    motion: &mut Motion,
) -> Result<(), BridgeError> {
    canvas.clear();
    let grid = world.grid();
    // The ground is a pure function of the seed and the address, so the
    // viewer computes it for the tiles the window covers and for no other.
    // A sweep of the whole world every frame is what the record calls a
    // design mistake.[^2]
    let terrain = world.terrain();
    // Three switches, read once for each frame, so a world in which nothing
    // has happened pays nothing for the layers that would show it.
    let dry = world.weather().is_dry();
    let any_upgrade = !world.upgrade_sites().is_empty();
    let any_luxury = !world.luxuries().is_empty();

    let (first_row, last_row) = camera.visible_rows(world, canvas);
    for row in first_row..last_row {
        let (first_column, last_column) = camera.visible_columns(row, world, canvas);
        for column in first_column..last_column {
            let address = Axial::new(column as i32, row as i32);
            let Some(tile) = grid.index_of(address) else {
                continue;
            };
            // The ground of the tile. This is the one generation of the
            // ground that the drawing of a tile pays for, and the counter
            // stands at the site that pays it.[^3]
            let Some(ground) = terrain.tile(address) else {
                continue;
            };
            canvas.ground_reads += 1;

            // The stock of a tile is the stock the ground generated, less
            // what somebody took from it. The engine stores the second term
            // only, so the viewer asks for the tiles the window covers and
            // for no other, and a tile nobody touched costs a search that
            // finds nothing.[^5]
            //
            // The reader takes the ground read above. The reader that starts
            // from the address alone would generate the ground a second time,
            // and the two answers would be the same answer.[^6]
            let Some(food) = world.tile_stock_of_ground(address, ground.kind, ResourceKind::Food)
            else {
                continue;
            };
            let mut ground_colour = tile_colour(ground.kind, ground.height.0, food.0);
            // Wet ground draws darker than dry ground. The field answers a
            // tile from the cell that covers it, so every tile of one cell
            // darkens together and the picture shows the lattice.[^7] The
            // read costs one array read through the cell of the tile, and a
            // dry world skips it.
            if !dry && world.ground_is_wet(address) == Some(true) {
                ground_colour = wetted(ground_colour, WET_BLUE_GAIN);
            }
            // The water in the air over this tile, mixed into the ground
            // before the holder takes its share. A storm used to cover the
            // finished pixel at a weight near the holder weight, in a colour
            // no faction uses, so a stormed holding lost its colour.[^15]
            //
            // The overlay is off below a tile width the viewer names, and a
            // weight under the floor draws nothing, so a resting world shows
            // no wash and no cell lattice.[^15] [^16]
            //
            // [^15]: Research report 24, defects 4 and 10. `docs/research/reports/24-demonstration-readability-resources-and-weather.md`
            // [^16]: Research report 23, defect 2. `docs/research/reports/23-demonstration-readability-review-1.md`
            if !dry && camera.tile_width >= AIR_LEAST_TILE {
                let weight = air_weight(world.air_at(address).unwrap_or(0));
                if weight >= AIR_LEAST_WEIGHT {
                    ground_colour = mix(ground_colour, AIR_COLOUR, weight);
                }
            }
            let (left, top, wide, tall) = tile_rect(camera, address);

            // The holder of this tile, read at the tile that is being
            // painted, on the loop that already runs. The layer starts no
            // pass of its own.[^3]
            //
            // The holder is the one value that names the faction which owns
            // a tile, and the layer reads it. A layer that derived a faction
            // from the tile index instead would give a full, still map of
            // holdings that no rule ever made, and it would tint open water,
            // which no faction ever holds.[^4]
            let holder = world.tile_holder(address);
            canvas.holder_reads += 1;
            match holder.and_then(Holder::faction) {
                None => canvas.fill_rect(left, top, wide, tall, ground_colour),
                Some(faction) => {
                    let held = faction_colour(faction);
                    canvas.fill_rect(
                        left,
                        top,
                        wide,
                        tall,
                        mix(ground_colour, held, HOLDER_WEIGHT),
                    );
                    canvas.tiles_held += 1;
                    if on_an_edge(world, address, holder, canvas) {
                        outline(
                            canvas,
                            left,
                            top,
                            wide,
                            tall,
                            mix(ground_colour, held, EDGE_WEIGHT),
                        );
                    }
                }
            }
            // The upgrade on this tile, read at the tile that is being
            // painted. The map is a sorted table of the improved tiles, so
            // the read is one binary search, and a world nobody built in
            // skips it.[^8]
            if any_upgrade {
                if let Some(site) = world.upgrade_at(address) {
                    if site.is_complete() || camera.tile_width < SITE_LEAST_TILE {
                        canvas.shade(
                            left,
                            top,
                            wide,
                            tall,
                            UPGRADE_COLOURS[site.kind.index()],
                            upgrade_weight(site),
                        );
                    } else {
                        // A site under work draws as a glyph and not as a
                        // wash. A wash at the weight the progress gives is
                        // the colour of the ground under it, so a store two
                        // work units into its forty-eight was absent rather
                        // than weak.[^17]
                        //
                        // [^17]: Research report 25, defect 1. `docs/research/reports/25-demonstration-readability-upgrades-and-units.md`
                        mark_site(canvas, left, top, wide, tall, site.kind);
                    }
                }
            }
            // A pip in a corner of the tile for each resource the tile still
            // holds. The ground carried food as a brightness and carried
            // wood and stone not at all.[^18]
            //
            // The pips draw at the close zooms only, so the extra reads
            // follow a window of a few hundred tiles.[^18]
            //
            // [^18]: Research report 24, defect 5. `docs/research/reports/24-demonstration-readability-resources-and-weather.md`
            if camera.tile_width >= PIP_LEAST_TILE {
                mark_deposits(
                    world,
                    canvas,
                    address,
                    ground.kind,
                    food.0,
                    left,
                    top,
                    wide,
                    tall,
                );
            }
            // The luxuries of this tile. The field is a sorted table of the
            // tiles that hold one, so the read is one binary search, and a
            // world with no deposit skips it.
            if any_luxury {
                let set = world.luxuries_at(tile);
                if !set.is_empty() {
                    mark_luxury(
                        canvas,
                        left,
                        top,
                        wide,
                        tall,
                        set.to_bits().trailing_zeros(),
                    );
                }
            }
            canvas.tiles_painted += 1;
            canvas.painted_by_kind[ground.kind.to_u8() as usize] += 1;
        }
    }

    // The floor holds a unit visible below sixteen pixels a tile, where a
    // disc of three tenths of the tile is one pixel of the faction colour
    // over ground that the same faction tints.[^14]
    //
    // [^14]: Research report 23, defect 1. `docs/research/reports/23-demonstration-readability-review-1.md`
    let radius = ((camera.tile_width * 0.3) as i32).max(UNIT_LEAST_RADIUS);
    // The table opens before the pass that paints and closes after it, so a
    // unit the pass did not paint is gone from it when the frame ends.
    motion.begin();
    let painted = draw_soldiers(
        world, camera, canvas, radius, first_row, last_row, pace, motion,
    );
    motion.end();
    painted
}

/// Marks each place that a faction founded.
///
/// A founded place is history. The world holds no record that a place was
/// founded, and this pass adds none: it reads the outcomes that the caller
/// kept when it founded the run.[^1] A caller that founded nothing passes an
/// empty slice, and the pass marks nothing.
///
/// The cost follows the faction count. The pass visits the outcomes and
/// nothing else. It reads no tile, no unit and no summary, so the cost is the
/// same at every zoom and does not change after the founding frame.[^2]
///
/// The mark is a ring around the place, in the faction's colour. The colour
/// comes from the one table this module holds, so the mark of a faction and
/// the units of that faction carry one colour.[^3] A band of one core colour
/// sits inside the ring, so the mark stands out on any ground.
///
/// The ring surrounds the place and does not cover it. A watcher reads the
/// ground and the units of a founded place through the mark.
///
/// A refused faction founded nowhere, so it gets no mark. The panel names it
/// instead.
///
/// Call this after the world pass and before the panel. The world pass clears
/// the canvas, and the panel draws over the picture.
///
/// # References
///
/// [^1]: ADR-0067, the viewer reads the world and never writes to it, decision D2. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
/// [^2]: ADR-0070, the head-up display reports what the drawing pass read, decision D1. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
/// [^3]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
pub fn mark_foundings(camera: Camera, canvas: &mut Canvas, outcomes: &[FoundingOutcome]) {
    // The mark is larger than a tile at every zoom, because a watcher must
    // find it without knowing where to look. It is not a tile read, so it
    // owes the tile grid nothing.
    let side = ((camera.tile_width * 2.0) as i32).max(FOUNDING_LEAST_SIDE);
    for outcome in outcomes {
        let Some(founding) = outcome.founding() else {
            continue;
        };
        let (x, y) = camera.centre_of(founding.place());
        if !canvas.holds(x, y, side / 2) {
            continue;
        }
        let colour = faction_colour(outcome.faction());
        // The seat tile itself, filled in the faction colour with a dark
        // core. A ring alone marked the founding and left the place drawing
        // like every tile beside it, so a watcher read a selection cursor
        // and not a settlement.[^4]
        //
        // [^4]: Research report 23, defect 4, and research report 24, defect 8. `docs/research/reports/23-demonstration-readability-review-1.md`
        let (seat_left, seat_top, seat_wide, seat_tall) = tile_rect(camera, founding.place());
        canvas.fill_rect(seat_left, seat_top, seat_wide, seat_tall, colour);
        let inset = (seat_wide.min(seat_tall) / 4).max(1);
        canvas.fill_rect(
            seat_left + inset,
            seat_top + inset,
            (seat_wide - inset * 2).max(1),
            (seat_tall - inset * 2).max(1),
            FOUNDING_CORE,
        );
        let left = x as i32 - side / 2;
        let top = y as i32 - side / 2;
        outline(canvas, left, top, side, side, colour);
        outline(canvas, left + 1, top + 1, side - 2, side - 2, FOUNDING_CORE);
        outline(canvas, left + 2, top + 2, side - 4, side - 4, colour);
        canvas.foundings_marked += 1;
    }
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
        //
        // Unclaimed ground counts as a difference. A holding therefore shows
        // its whole outline, and not only the part that meets another
        // faction.[^2]
        //
        // [^2]: Backlog item 0209. `docs/backlog/proposed/0209-tell-a-frontier-from-the-edge-of-the-claimed-ground.md`
        edge = edge || beside.unwrap_or(Holder::NOBODY) != holder.unwrap_or(Holder::NOBODY);
    }
    edge
}

/// Returns how much of the upgrade colour covers a tile, from its progress.
///
/// A site that has just begun draws at the floor and a finished site at the
/// ceiling. The progress is a whole number of work units and the work of a
/// kind is a whole number too, so the depth is exact.
fn upgrade_weight(site: UpgradeSite) -> u8 {
    let work = site.kind.work().max(1);
    let done = site.progress.0.clamp(0, work);
    let weight =
        UPGRADE_WEIGHT_FLOOR + (UPGRADE_WEIGHT_CEILING - UPGRADE_WEIGHT_FLOOR) * done / work;
    u8::try_from(weight.clamp(0, 255)).unwrap_or(u8::MAX)
}

/// Returns how much of the air colour covers a tile, from the drops over it.
///
/// The shade saturates at the drops the viewer chose, so a storm above that
/// draws the same as a storm at it.
///
/// A test reads this rather than repeating the arithmetic, so the weight has
/// one declaration site.[^1]
///
/// # References
///
/// [^1]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
#[must_use]
pub fn air_weight(drops: i64) -> u8 {
    let held = drops.clamp(0, AIR_AT_FULL_SHADE);
    u8::try_from(held * AIR_WEIGHT_CEILING / AIR_AT_FULL_SHADE).unwrap_or(u8::MAX)
}

/// Paints the mark of a luxury in the middle of a tile.
///
/// The mark is a square of one third of the tile, and at least one pixel, so
/// the ground shows around it and a watcher still finds it at the smallest
/// zoom.
///
/// The ordinal is the lowest luxury the tile carries. The hue comes from it,
/// because one colour for every kind said that a tile holds a luxury and
/// never which one.[^1]
///
/// # References
///
/// [^1]: Research report 24, defect 7. `docs/research/reports/24-demonstration-readability-resources-and-weather.md`
fn mark_luxury(canvas: &mut Canvas, left: i32, top: i32, wide: i32, tall: i32, ordinal: u32) {
    let side = (wide.min(tall) / 3).max(1);
    let x = left + (wide - side) / 2;
    let y = top + (tall - side) / 2;
    canvas.fill_rect(x, y, side, side, luxury_colour(ordinal));
}

/// Returns the colour the viewer marks one kind of luxury in.
///
/// The hue turns with the ordinal, and the stride is odd, so two kinds that
/// the catalogue numbers together do not draw together. The colour is the
/// viewer's own, and the engine holds none.[^1]
///
/// # References
///
/// [^1]: ADR-0067, the viewer reads the world and never writes to it, decision D2. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
#[must_use]
pub fn luxury_colour(ordinal: u32) -> u32 {
    let position = (ordinal.wrapping_mul(LUXURY_HUE_STRIDE)) % 256;
    let turn = position * 6;
    let sector = turn / 256;
    let rise = turn % 256;
    let fall = 255 - rise;
    let (red, green, blue) = match sector {
        0 => (255, rise, 0),
        1 => (fall, 255, 0),
        2 => (0, 255, rise),
        3 => (0, fall, 255),
        4 => (rise, 0, 255),
        _ => (255, 0, fall),
    };
    (red << 16) | (green << 8) | blue
}

/// Paints the glyph of a build site in the middle of a tile.
///
/// Each kind takes a shape of its own, drawn over a dark square, so a watcher
/// reads a made thing and not a shade of the ground.[^1]
///
/// # References
///
/// [^1]: Research report 25, defect 1. `docs/research/reports/25-demonstration-readability-upgrades-and-units.md`
fn mark_site(canvas: &mut Canvas, left: i32, top: i32, wide: i32, tall: i32, kind: UpgradeKind) {
    // Half the tile. The ground shows around the glyph, so a watcher reads
    // the kind of ground and the thing somebody is making on it at once.
    let side = (wide.min(tall) / 2).max(3);
    let x = left + (wide - side) / 2;
    let y = top + (tall - side) / 2;
    // The dark square is the rim. It separates every glyph from the ground
    // under it, whatever the ground is.
    canvas.fill_rect(x, y, side, side, UNIT_RIM);
    let colour = UPGRADE_COLOURS[kind.index()];
    let bar = (side / 4).max(1);
    match kind {
        // A made way: one bar across the tile.
        UpgradeKind::Road => canvas.fill_rect(x, y + (side - bar) / 2, side, bar, colour),
        // Worked ground: two bars, one above the other.
        UpgradeKind::Terrace => {
            canvas.fill_rect(x, y + bar, side, bar, colour);
            canvas.fill_rect(x, y + side - bar * 2, side, bar, colour);
        }
        // A great work: a diamond.
        UpgradeKind::Wonder => {
            let half = side / 2;
            for row in 0..side {
                let reach = half - (row - half).abs();
                canvas.fill_rect(x + half - reach, y + row, reach * 2 + 1, 1, colour);
            }
        }
        // A storehouse: a solid block inside the square.
        UpgradeKind::Store => canvas.fill_rect(
            x + bar,
            y + bar,
            (side - bar * 2).max(1),
            (side - bar * 2).max(1),
            colour,
        ),
    }
}

/// Paints one pip in a corner of a tile for each resource it still holds.
///
/// The pip grows with the amount, so a watcher reads a rich deposit from a
/// poor one and watches a deposit drain.[^1]
///
/// **A resource the tile no longer holds draws no pip.** The colour of the
/// ground carries the food, and the pips carry all three, so an empty corner
/// and a small pip are the two states a watcher must tell apart.
///
/// The food is the stock the tile pass already read. The other two are one
/// read each, and the caller draws no pip below the tile width it names, so
/// the reads follow a window of a few hundred tiles.[^1]
///
/// # References
///
/// [^1]: Research report 24, defect 5. `docs/research/reports/24-demonstration-readability-resources-and-weather.md`
#[allow(clippy::too_many_arguments)]
fn mark_deposits(
    world: &World,
    canvas: &mut Canvas,
    address: Axial,
    ground: TileKind,
    food: u32,
    left: i32,
    top: i32,
    wide: i32,
    tall: i32,
) {
    let margin = (wide.min(tall) / 6).max(1);
    // A pip reaches a sixth of the tile at the stock the viewer chose. A
    // larger pip crowds a window in which most tiles carry something.
    let most = (wide.min(tall) / 6).max(2);
    for kind in ResourceKind::ALL {
        let held = if kind == ResourceKind::Food {
            food
        } else {
            match world.tile_stock_of_ground(address, ground, kind) {
                Some(stock) => stock.0,
                None => continue,
            }
        };
        if held == 0 {
            continue;
        }
        let side = ((held as i32).min(PIP_AT_FULL_SIZE) * most / PIP_AT_FULL_SIZE).max(2);
        // One corner for each kind, so two kinds on one tile never overlap.
        let (x, y) = match kind as usize {
            0 => (left + margin, top + margin),
            1 => (left + wide - margin - side, top + margin),
            _ => (left + margin, top + tall - margin - side),
        };
        outline(canvas, x - 1, y - 1, side + 2, side + 2, UNIT_RIM);
        canvas.fill_rect(x, y, side, side, PIP_COLOURS[kind as usize]);
    }
}

/// Draws a one pixel border inside a rectangle.
fn outline(canvas: &mut Canvas, left: i32, top: i32, wide: i32, tall: i32, colour: u32) {
    canvas.fill_rect(left, top, wide, 1, colour);
    canvas.fill_rect(left, top + tall - 1, wide, 1, colour);
    canvas.fill_rect(left, top, 1, tall, colour);
    canvas.fill_rect(left + wide - 1, top, 1, tall, colour);
}

/// The gap the drawing leaves between two neighbouring tiles, in pixels.
///
/// The gap is what a watcher reads as the black grid between the tiles. It is
/// one pixel wide, because a gap is a separator and one pixel is the
/// narrowest a separator can be. It is a whole number of pixels, because a
/// fractional separator lands on a different pixel under each tile and the
/// eye reads that as a lattice.[^1]
///
/// # References
///
/// [^1]: Findings register, FND-207. `docs/FINDINGS.md`
const TILE_GAP: i32 = 1;

/// The smallest tile width that carries a gap, in pixels.
///
/// **A separator that takes a quarter of the cell is the picture.** At eight
/// pixels a tile the gap covers about a quarter of the area, the rows step by
/// half a tile, and a watcher reads the stagger of the bricks before the
/// ground.[^1]
///
/// The share the gap takes of a cell is `1 - ((w - 1) / w)^2`, which is about
/// one eighth at this width and about one half near three and a half pixels.
/// The earlier bound was the width at which the gap takes half the cell,
/// which is the width at which it is no longer a separator at all. This is
/// the width at which it stops dominating.
///
/// # References
///
/// [^1]: Research report 23, defect 6. `docs/research/reports/23-demonstration-readability-review-1.md`
const GAP_LEAST_TILE: f32 = 16.0;

/// Returns the gap to leave under a tile of a given width.
///
/// Below the least width the drawing leaves the gap out, and the colour
/// change from one tile to the next is what separates them.[^1]
///
/// # References
///
/// [^1]: Research report 23, defect 6. `docs/research/reports/23-demonstration-readability-review-1.md`
fn gap_for(tile_width: f32) -> i32 {
    if tile_width >= GAP_LEAST_TILE {
        TILE_GAP
    } else {
        0
    }
}

/// Returns the pixel rectangle of one tile, as a left, a top, a width and a
/// height.
///
/// **A tile runs from its own snapped left edge to the snapped left edge of
/// the tile beside it.** A tile is a fractional number of pixels wide at
/// nearly every zoom, because each zoom step multiplies the size by a
/// fraction. A drawing that took one integer width and placed it at a rounded
/// centre left a gap of one pixel under some tiles and two pixels under
/// others, in a pattern that repeated across the picture, and the eye read
/// that pattern as a lattice.[^1]
///
/// Taking the far edge from the neighbour makes the two agree by
/// construction, so the gap is the same under every tile at every zoom. The
/// far edge is not the near edge plus a width. It is the neighbour's own near
/// edge, read the same way, because two snapped values that a reader expects
/// to be equal are one fact in two places unless one of them is the other.
///
/// A test reads the rectangle from here rather than repeating the arithmetic.
/// A second site that computed where a tile lands would be one fact in two
/// places, and nothing would fail when the two disagreed.[^2]
///
/// # References
///
/// [^1]: Findings register, FND-207. `docs/FINDINGS.md`
/// [^2]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
#[must_use]
pub fn tile_rect(camera: Camera, address: Axial) -> (i32, i32, i32, i32) {
    let near = |address: Axial| {
        let (x, y) = camera.centre_of(address);
        (
            (x - camera.tile_width * 0.5).round() as i32,
            (y - camera.tile_height * 0.5).round() as i32,
        )
    };
    let (left, top) = near(address);
    let (right, _) = near(Axial::new(address.q + 1, address.r));
    let (_, bottom) = near(Axial::new(address.q, address.r + 1));
    (
        left,
        top,
        (right - left - gap_for(camera.tile_width)).max(1),
        (bottom - top - gap_for(camera.tile_height)).max(1),
    )
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
/// [^3]: PRD-0002, a developer watches the world run. `docs/product/shipped/prd-0002-a-developer-watches-the-world-run.md`
/// [^4]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
#[allow(clippy::too_many_arguments)]
fn draw_soldiers(
    world: &World,
    camera: Camera,
    canvas: &mut Canvas,
    radius: i32,
    first_row: u32,
    last_row: u32,
    pace: Pace,
    motion: &mut Motion,
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
            // The units of one block arrive in tile order, so the units of
            // one tile are one adjacent run.[^5] The crowd count is the
            // length of that run. It costs one comparison for each unit on a
            // path that already visits every unit it paints, so the count
            // adds no pass over the world.[^6]
            //
            // [^5]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D2. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
            // [^6]: ADR-0070, the head-up display reports what the drawing pass read, decision D1. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
            let mut run: Option<(Axial, u32)> = None;
            // The units of the tile the run holds, kept until the run closes
            // so that the crowd draws as one thing.
            let mut crowd: Vec<Painted> = Vec::new();
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
                // Where this unit is drawn. A unit that moved to a
                // neighbouring tile since the last frame draws between the
                // two tile centres, and every other unit draws at its tile.
                // The table is the viewer's memory, because the world holds
                // one tick at a time.[^13]
                //
                // The visibility test above reads the tile centre, so the
                // tween changes what is drawn and never which units are
                // visited, nor the order they are visited in.
                //
                // [^13]: ADR-0067, the viewer reads the world and never writes to it, decision D2. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
                //
                // The tile the unit stood on at the last frame is also the
                // tail of the heading line. The table answers by key, so its
                // order reaches no pixel.[^13]
                let came_from = motion.moving_from(*soldier).filter(|&from| from != address);
                let (x, y) = match motion.place(*soldier, address, pace.phase) {
                    Some(from) => between(camera.centre_of(from), (x, y), pace.phase),
                    None => (x, y),
                };
                let slot = colour_slot(faction);
                canvas.soldiers_painted += 1;
                // The disc is drawn when the run of this tile closes, not
                // here. The count of a tile decides the radius and the badge,
                // and the factions on it decide the wedges, so nothing can be
                // painted until the run is whole.[^15]
                //
                // [^15]: Research report 25, defects 4 and 5. `docs/research/reports/25-demonstration-readability-upgrades-and-units.md`
                let mut short = false;
                // The condition of this unit, read at the unit that is being
                // painted, on the loop that already runs. The layer starts no
                // pass of its own.[^7]
                //
                // The engine names the condition, and the viewer compares no
                // number against a bound of its own. A viewer that read the
                // accumulator would hold the rule a second time.[^8]
                //
                // [^7]: ADR-0070, the head-up display reports what the drawing pass read, decision D1. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
                // [^8]: ADR-0063, a need is a rate with a threshold, and crossing it is a fact, decision D3. `docs/adrs/accepted/adr-0063-a-need-is-a-rate-with-a-threshold-and-crossing-it-is-a-fact.md`
                //
                // A unit that a completed step left alive is fed or short.
                // The starved arm is here because the engine names three
                // conditions, and it draws the same mark rather than a mark
                // that nothing can reach.[^9]
                //
                // [^9]: Findings register, FND-119. `docs/FINDINGS.md`
                canvas.condition_reads += 1;
                let condition = world.unit_condition(*soldier);
                match condition {
                    None | Some(NeedCondition::Fed) => {}
                    Some(NeedCondition::Short | NeedCondition::Starved) => {
                        canvas.units_short += 1;
                        short = true;
                    }
                }
                // What this unit carries and where it lives, read at the
                // unit that is being painted, on the loop that already runs.
                // Both start a pass of nothing.[^11]
                //
                // **These are the two facts the engine already produced and
                // no watcher could see.** Every unit holds a home site and a
                // share of them haul a load, and until now the only way to
                // learn either was to read the log.[^12]
                //
                // The count of reads is a function of the window, in the same
                // way the holder count is, and a test reads it. A layer that
                // swept the arena would report the same totals and cost the
                // population.[^11]
                //
                // [^11]: ADR-0070, the head-up display reports what the drawing pass read, decision D1. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
                // [^12]: Backlog item 0274. `docs/backlog/complete/0274-show-the-load-a-unit-carries-and-the-home-it-keeps.md`
                canvas.carry_reads += 1;
                if let Some(load) = arena.carry(*soldier) {
                    let mut any = false;
                    for kind in ResourceKind::ALL {
                        let held = load.of(kind).0;
                        canvas.carried_by_kind[kind as usize] += held;
                        any |= held > 0;
                    }
                    if any {
                        canvas.units_carrying += 1;
                    }
                }
                canvas.home_reads += 1;
                if let Some(Some(_)) = arena.home(*soldier) {
                    canvas.units_housed += 1;
                }
                // The unit nearest the middle of the window, fixed on the
                // loop that already paints. The panel names this unit when it
                // reports a choice, and it starts no pass to find it.[^10]
                //
                // [^10]: ADR-0070, the head-up display reports what the drawing pass read, decision D1. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
                let reach = reach_from_middle(canvas, x, y);
                if canvas.focus.is_none_or(|held| reach < held.reach) {
                    canvas.focus = Some(Focus {
                        entity: *soldier,
                        address,
                        faction,
                        condition,
                        reach,
                    });
                }
                // The census is a by-product of the pass that paints. A
                // separate pass over the soldiers would give the same numbers
                // and cost the population.
                canvas.painted_by_faction[slot] += 1;
                match run {
                    Some((held, count)) if held == address => run = Some((held, count + 1)),
                    other => {
                        draw_crowd(canvas, camera, radius, &mut crowd);
                        close_run(canvas, world, camera, other);
                        run = Some((address, 1));
                    }
                }
                crowd.push(Painted {
                    x,
                    y,
                    from: came_from.map(|from| camera.centre_of(from)),
                    slot,
                    short,
                });
            }
            draw_crowd(canvas, camera, radius, &mut crowd);
            close_run(canvas, world, camera, run);
        }
    }
    Ok(())
}

/// One unit the pass painted, held until its tile's run closes.
///
/// The position is where the unit draws, which is its tile centre or a point
/// between two tile centres. The tail is where the unit stood at the last
/// frame, when the table held it.
struct Painted {
    /// Where the disc draws, across.
    x: f32,
    /// Where the disc draws, down.
    y: f32,
    /// Where the unit stood at the last frame, when the table held it.
    from: Option<(f32, f32)>,
    /// The colour slot of the faction.
    slot: usize,
    /// Whether a shortage holds the unit.
    short: bool,
}

/// Returns the radius a crowd of this size draws at.
///
/// Below the badge width a tile has no room for a number, so the disc carries
/// the count instead: a full tile is visibly fuller than a single unit.[^1]
/// The growth is the square root of the count, so the area of the disc
/// follows the count, and it is held inside the tile.
///
/// # References
///
/// [^1]: Research report 25, defect 4. `docs/research/reports/25-demonstration-readability-upgrades-and-units.md`
fn crowd_radius(base: i32, count: u32, tile_width: f32) -> i32 {
    if count <= 1 {
        return base;
    }
    let grown = (base as f32 * (count as f32).sqrt()) as i32;
    let ceiling = ((tile_width * 0.5) as i32).max(base + 2);
    grown.clamp(base, ceiling)
}

/// Draws the units of one tile, and empties the run.
///
/// **The picture of eight units was the picture of one.** Every unit of a
/// tile takes the centre of that tile, so the discs landed on each other
/// exactly and the last one drawn won. This pass draws the run as one crowd:
/// the factions present take a wedge each, the count sets the radius below
/// the badge width, and a badge states the count above it.[^1]
///
/// The heading line runs from the tile the unit left to the point it draws
/// at, so a watcher sees where a unit came from.[^2]
///
/// # References
///
/// [^1]: Research report 25, defects 4 and 5. `docs/research/reports/25-demonstration-readability-upgrades-and-units.md`
/// [^2]: Research report 23, defect 3. `docs/research/reports/23-demonstration-readability-review-1.md`
fn draw_crowd(canvas: &mut Canvas, camera: Camera, radius: i32, crowd: &mut Vec<Painted>) {
    if crowd.is_empty() {
        return;
    }
    // The lines sit under the discs, so a disc is never cut by the line of
    // the unit it belongs to.
    if camera.tile_width >= HEADING_TILE {
        for unit in crowd.iter() {
            if let Some(from) = unit.from {
                canvas.line(from, (unit.x, unit.y), FACTION_COLOURS[unit.slot]);
            }
        }
    }

    // The factions on the tile, in ascending colour order. The order of the
    // wedges is therefore the colour table's and never the order the spatial
    // structure gave the units in.[^3]
    //
    // [^3]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    let mut slots: Vec<usize> = crowd.iter().map(|unit| unit.slot).collect();
    slots.sort_unstable();
    slots.dedup();
    let shared: Vec<u32> = slots.iter().map(|&slot| FACTION_COLOURS[slot]).collect();

    let count = crowd.len() as u32;
    let badged = camera.tile_width >= CROWD_BADGE_TILE;
    let radius = if badged {
        radius
    } else {
        crowd_radius(radius, count, camera.tile_width)
    };
    for unit in crowd.iter() {
        let one = [FACTION_COLOURS[unit.slot]];
        let colours: &[u32] = if shared.len() > 1 { &shared } else { &one };
        canvas.fill_wedges(unit.x as i32, unit.y as i32, radius, colours, UNIT_RIM);
    }
    for unit in crowd.iter() {
        if unit.short {
            canvas.fill_disc(unit.x as i32, unit.y as i32, (radius / 2).max(1), SHORTAGE);
        }
    }
    if badged && count > 1 {
        let word = count.to_string();
        let first = &crowd[0];
        let left = first.x as i32 - text::width_of(&word, 1) / 2;
        let top = first.y as i32 - radius - text::GLYPH_HEIGHT - 2;
        canvas.fill_rect(
            left - 2,
            top - 1,
            text::width_of(&word, 1) + 4,
            text::GLYPH_HEIGHT + 2,
            UNIT_RIM,
        );
        canvas.write(left, top, &word, 1, SHORTAGE);
    }
    crowd.clear();
}

/// Returns the squared distance in pixels from the middle of the canvas.
///
/// The arithmetic is the viewer's own and nothing formed here reaches the
/// engine.[^1]
///
/// # References
///
/// [^1]: ADR-0067, the viewer reads the world and never writes to it, decision D3. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
fn reach_from_middle(canvas: &Canvas, x: f32, y: f32) -> i64 {
    let across = i64::from(x as i32 - (canvas.width() / 2) as i32);
    let down = i64::from(y as i32 - (canvas.height() / 2) as i32);
    across * across + down * down
}

/// Records what one tile's run of painted units means, and marks the tile
/// when the run is longer than the tile admits.
///
/// **The capacity is the composition of the ground and the finished upgrade,
/// not the ground alone.** A made way states a capacity above every value in
/// the terrain table, so a tile that admission legitimately filled to that
/// number would take an over-full mark from the ordinary capacity onward. The
/// mark would then say that a correctly filled tile is broken.[^2]
///
/// The viewer asks the engine's one reader of the two tables, which is the
/// reader admission itself composes from. The viewer holds no capacity value,
/// so a change to either table reaches the picture with no edit here.[^1] [^3]
///
/// A tile with no painted unit closes no run, so an empty tile is neither
/// counted nor marked.
///
/// # References
///
/// [^1]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D4. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
/// [^2]: Findings register, FND-193. `docs/FINDINGS.md`
/// [^3]: ADR-0090, a tile upgrade is stored sparsely, as the difference from the generated world, decision D3. `docs/adrs/draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md`
fn close_run(canvas: &mut Canvas, world: &World, camera: Camera, run: Option<(Axial, u32)>) {
    let Some((address, count)) = run else {
        return;
    };
    canvas.crowd_worst = canvas.crowd_worst.max(count);
    let Some(capacity) = world.tile_capacity(address) else {
        return;
    };
    if count >= capacity {
        canvas.tiles_at_capacity += 1;
    }
    if count > capacity {
        let (left, top, wide, tall) = tile_rect(camera, address);
        outline(canvas, left, top, wide, tall, OVER_CAPACITY);
    }
}
