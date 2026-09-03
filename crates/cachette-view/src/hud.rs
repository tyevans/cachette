//! The panel that says what is happening.
//!
//! A person who watches coloured cells move can see that the world runs. They
//! cannot see which tick it is, where they are looking, who the colours are,
//! or what the step costs. This module writes those numbers over the picture.
//!
//! # What the panel may say
//!
//! Every number here comes from one of three places: a value the engine
//! already exposes, a value the viewer computed for itself, or a count the
//! drawing pass produced while it painted. The panel starts no pass of its
//! own over the world, and it asks the engine for nothing that the engine
//! does not already hold.[^1] [^2]
//!
//! That rule decides the content. The panel says how many soldiers stand in
//! the window, because the drawing pass counted them as it painted them. It
//! does not say how many soldiers each faction has in the whole world,
//! because nothing knows that without reading every soldier. A label says
//! which of the two a number is, so a reader never mistakes one for the
//! other.
//!
//! # The reading is separate from the writing
//!
//! A readout is a set of numbers. Painting a readout is a function of the
//! readout alone, so the same readout gives the same pixels. The reading
//! happens once, against the world and the finished canvas.
//!
//! # References
//!
//! [^1]: ADR-0067, the viewer reads the world and never writes to it, decision D2. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
//! [^2]: ADR-0070, the head-up display reports what the drawing pass read, decision D1. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`

use cachette_core::founding::{Founding, FoundingError, FoundingOutcome, Provision};
use cachette_core::pyramid::CellSummary;
use cachette_core::resource::{ResourceKind, RESOURCE_KIND_COUNT};
use cachette_core::terrain::{TileKind, KIND_COUNT};
use cachette_core::{
    Axial, ChoiceExplanation, CommodityId, Entity, FactionId, Fix32, NeedCondition, World,
    NO_INTENT, OPTIONS, OPTION_COUNT,
};

use crate::metrics::Metrics;
use crate::paint::{faction_colour, kind_colour, Camera, Canvas, Focus, COLOURED_FACTIONS};
use crate::text;

// The panel geometry and the panel colours are declared once, in the panel
// standard. This module reads them from there. A second copy of the width or
// of the line height would be one fact in two places, with nothing to fail
// when the copies drift.[^3]
//
// [^3]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
use crate::panel::{
    self, BAR_HEIGHT, EDGE, HEADING, LABEL, LINE, MARGIN, PAD, PANEL, PANEL_WEIGHT, SWATCH,
    SWATCH_GAP, TITLE, VALUE, VALUE_COLUMN, WIDTH as PANEL_WIDTH,
};

/// What a cost row says when no measurement was taken.
///
/// A cost is a measurement, and a measurement that nobody took is absent. The
/// record forbids the window from estimating a number it does not have, and a
/// mean over no measurement is an estimate of the worst kind, because it looks
/// like a reading of zero.[^1]
///
/// # References
///
/// [^1]: ADR-0070, the head-up display reports what the drawing pass read, decision D2. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
pub(crate) const NOT_MEASURED: &str = "not measured yet";

/// Returns a mean and a worst in microseconds, or the words that say neither
/// was taken.
fn pair_of_micros(mean: f64, worst: f64, count: u64) -> String {
    if count == 0 {
        NOT_MEASURED.to_string()
    } else {
        format!("{mean:.0} / {worst:.0} us")
    }
}

/// What one founding chose, as the panel states it.
///
/// The founding runs once, before the first frame. The program that owns the
/// loop keeps the report it returned, and the panel borrows it. The engine
/// holds no copy, because a field that existed to be drawn would be the
/// violation the boundary record names.[^1]
///
/// Every quantity here is the one the survey read. Nothing recomputes a
/// score, so no copy can disagree with the choice that was made.[^2]
///
/// # References
///
/// [^1]: ADR-0067, the viewer reads the world and never writes to it, decision D2. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
/// [^2]: ADR-0075, the founding choice reads a bounded sample of the world, decision D5. `docs/adrs/accepted/adr-0075-the-founding-choice-reads-a-bounded-sample-of-the-world.md`
#[derive(Clone, Copy, Debug)]
pub struct FoundingReport {
    faction: FactionId,
    place: Axial,
    shown: bool,
    considered: usize,
    chosen: Provision,
    other: Option<(Axial, Provision)>,
}

impl FoundingReport {
    /// Reads one founding for the panel.
    ///
    /// Returns `None` when the survey names no chosen place. The panel then
    /// says nothing about that founding, rather than stating a quantity that
    /// nothing computed.[^1]
    ///
    /// The window test is the viewer's own arithmetic over the camera. It
    /// reads the column range of one row and starts no loop over the world.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0070, the head-up display reports what the drawing pass read, decision D2. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
    /// [^2]: ADR-0070, the head-up display reports what the drawing pass read, decision D1. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
    #[must_use]
    pub fn of(
        faction: FactionId,
        founding: &Founding,
        world: &World,
        camera: Camera,
        canvas: &Canvas,
    ) -> Option<Self> {
        let chosen = founding.survey().chosen()?;
        let other = founding
            .survey()
            .rejected()
            .first()
            .map(|candidate| (candidate.address(), candidate.provision()));
        Some(Self {
            faction,
            place: founding.place(),
            shown: shows(founding.place(), world, camera, canvas),
            considered: founding.survey().considered(),
            chosen: chosen.provision(),
            other,
        })
    }

    /// Returns the faction that founded.
    #[must_use]
    pub const fn faction(&self) -> FactionId {
        self.faction
    }

    /// Returns the place the founding chose.
    #[must_use]
    pub const fn place(&self) -> Axial {
        self.place
    }

    /// Reports whether the window covers the chosen place.
    #[must_use]
    pub const fn shown(&self) -> bool {
        self.shown
    }

    /// Returns the number of places the founding compared.
    #[must_use]
    pub const fn considered(&self) -> usize {
        self.considered
    }

    /// Returns the quantities the survey read at the chosen place.
    #[must_use]
    pub const fn chosen(&self) -> Provision {
        self.chosen
    }

    /// Returns one place the founding did not choose, and its quantities.
    ///
    /// Returns `None` when the survey compared one place only.
    #[must_use]
    pub const fn other(&self) -> Option<(Axial, Provision)> {
        self.other
    }
}

/// What the tile under the middle of the window holds.
///
/// The middle of the window is the crosshair. The viewer has no cursor, so
/// the panel reports the tile a watcher scrolled to rather than one they
/// pointed at.[^1]
///
/// Every quantity is one the engine already holds, read at one address. The
/// section starts no pass over the world.[^2]
///
/// # References
///
/// [^1]: Decisions register, DEC-077. `docs/DECISIONS.md`
/// [^2]: ADR-0070, the head-up display reports what the drawing pass read, decision D1. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
#[derive(Clone, Copy, Debug)]
pub struct TileReadout {
    address: Axial,
    kind: TileKind,
    capacity: u32,
    holder: Option<FactionId>,
    /// What the tile still holds, in resource kind order.
    stock: [u32; RESOURCE_KIND_COUNT],
    /// What the ground generated for the tile, in resource kind order.
    generated: [u32; RESOURCE_KIND_COUNT],
    /// The units on the tile, or `None` when the engine could not say.
    units: Option<u32>,
}

impl TileReadout {
    /// Reads the tile at one address.
    ///
    /// Returns `None` when the address lies outside the world.
    ///
    /// The unit count comes from the spatial structure at one tile. A
    /// structure that no longer describes the world gives no count, and the
    /// panel then states none rather than a zero a reader cannot tell from a
    /// true one.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0070, the head-up display reports what the drawing pass read, decision D2. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
    #[must_use]
    pub fn of(world: &World, address: Axial) -> Option<Self> {
        let kind = world.tile_kind(address)?;
        let mut stock = [0u32; RESOURCE_KIND_COUNT];
        let mut generated = [0u32; RESOURCE_KIND_COUNT];
        for resource in ResourceKind::ALL {
            stock[resource.index()] = world.tile_stock(address, resource)?.0;
            generated[resource.index()] = world.original_stock(address, resource)?.0;
        }
        Some(Self {
            address,
            kind,
            capacity: world.tile_capacity(address)?,
            holder: world
                .tile_holder(address)
                .and_then(cachette_core::Holder::faction),
            stock,
            generated,
            units: world
                .soldier_count_on(address)
                .ok()
                .and_then(|count| u32::try_from(count).ok()),
        })
    }

    /// Returns the address of the tile.
    #[must_use]
    pub const fn address(&self) -> Axial {
        self.address
    }

    /// Returns the ground of the tile.
    #[must_use]
    pub const fn kind(&self) -> TileKind {
        self.kind
    }

    /// Returns what the tile still holds of one resource.
    #[must_use]
    pub fn stock(&self, kind: ResourceKind) -> u32 {
        self.stock[kind.index()]
    }

    /// Returns what the ground generated for the tile, of one resource.
    ///
    /// A stock below this and a generated value above zero mean that
    /// somebody gathered here and that the deposit has not fully
    /// recovered.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D4. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
    #[must_use]
    pub fn generated(&self, kind: ResourceKind) -> u32 {
        self.generated[kind.index()]
    }

    /// Returns the units the engine says stand on the tile.
    #[must_use]
    pub const fn units(&self) -> Option<u32> {
        self.units
    }
}

/// Why one unit chose what it chose.
///
/// The engine holds a verb that reports every score, the value each option
/// read from the level 1 cell, the weight each option carried, and the floor
/// an option had to clear. It recomputes the answer from the world as it
/// stands, because it stores no score.[^1]
///
/// The panel names the unit nearest the middle of the window, which the
/// drawing pass fixed while it painted.[^2]
///
/// # References
///
/// [^1]: ADR-0064, a unit chooses by scoring a small fixed option set, decision D2. `docs/adrs/accepted/adr-0064-a-unit-chooses-by-scoring-a-small-fixed-option-set.md`
/// [^2]: Decisions register, DEC-077. `docs/DECISIONS.md`
#[derive(Clone, Copy, Debug)]
pub struct ChoiceReadout {
    focus: Focus,
    explanation: Option<ChoiceExplanation>,
}

impl ChoiceReadout {
    /// Asks the engine why the focused unit chose what it chose.
    ///
    /// The call names one unit. It reads that unit, the level 1 cell that
    /// covers its tile, and the weight table. It starts no pass over the
    /// world.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0070, the head-up display reports what the drawing pass read, decision D1. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
    #[must_use]
    pub fn of(world: &World, focus: Focus) -> Self {
        Self {
            focus,
            explanation: world.explain_choice(focus.entity()),
        }
    }

    /// Returns the unit the panel names.
    #[must_use]
    pub const fn focus(&self) -> Focus {
        self.focus
    }

    /// Returns the answer the engine gave.
    ///
    /// Returns `None` when the engine would say nothing about the unit. The
    /// panel then says that, rather than printing four zeroes a reader would
    /// take for real scores.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0070, the head-up display reports what the drawing pass read, decision D2. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
    #[must_use]
    pub const fn explanation(&self) -> Option<ChoiceExplanation> {
        self.explanation
    }
}

/// What one site produces, holds and owes.
///
/// This is the loop the engine closed and nothing could see: the survey reads
/// the ground, the founding sets a production rate from it, the rate fills a
/// store, the store feeds the units of that site, and a unit that is not fed
/// gains a deficit and ends.[^1]
///
/// The list is as long as the settlement arena, which the founding sized by
/// the faction count. It is not a pass over the world.[^2]
///
/// # References
///
/// [^1]: What a unit does in a tick, section 1. `docs/research/what-a-unit-does-in-a-tick.md`
/// [^2]: ADR-0070, the head-up display reports what the drawing pass read, decision D1. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
#[derive(Clone, Copy, Debug)]
pub struct SiteReadout {
    faction: FactionId,
    place: Axial,
    store: Fix32,
    production: Fix32,
    upkeep: Fix32,
    /// What the cohorts of this site asked for and what the store gave, on
    /// the last tick that could not serve them in full.
    rationed: Option<(i64, i64)>,
    /// How many positions this site declares.
    seats: u32,
    /// How many of those positions a live unit holds.
    seats_held: u32,
}

impl SiteReadout {
    /// Returns the faction that holds the site.
    #[must_use]
    pub const fn faction(&self) -> FactionId {
        self.faction
    }

    /// Returns the tile the site stands on.
    #[must_use]
    pub const fn place(&self) -> Axial {
        self.place
    }

    /// Returns what the store holds of the ration commodity.
    #[must_use]
    pub const fn store(&self) -> Fix32 {
        self.store
    }

    /// Returns how many positions the site declares.
    #[must_use]
    pub const fn seats(&self) -> u32 {
        self.seats
    }

    /// Returns how many of those positions a live unit holds.
    #[must_use]
    pub const fn seats_held(&self) -> u32 {
        self.seats_held
    }

    /// Returns what the site adds each time the rate pass runs.
    #[must_use]
    pub const fn production(&self) -> Fix32 {
        self.production
    }

    /// Returns what the site owes each time the rate pass runs.
    #[must_use]
    pub const fn upkeep(&self) -> Fix32 {
        self.upkeep
    }

    /// Returns what the cohorts asked for and what they got, when the last
    /// draw could not serve them in full.
    ///
    /// Returns `None` when the last draw served every cohort of this site.
    #[must_use]
    pub const fn rationed(&self) -> Option<(i64, i64)> {
        self.rationed
    }
}

/// The number of sites the panel reads.
///
/// The panel reads this many and no more, whatever the world holds. A world
/// of a thousand sites would otherwise cost the panel a loop over all of
/// them, which is the growth the panel record forbids.[^1]
///
/// The panel states how many sites the world holds beside the rows, so a
/// reader knows the list is the first few and not all of them.[^2]
///
/// # References
///
/// [^1]: ADR-0070, the head-up display reports what the drawing pass read, decision D1. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
/// [^2]: ADR-0070, the head-up display reports what the drawing pass read, decision D2. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
const SITE_ROWS: usize = 6;

/// Reads the first few sites of the world for the panel.
///
/// The walk stops at a fixed number of sites, so the cost does not follow the
/// world.[^1] The arena reports how many it holds without a walk, and the
/// panel states that number.
///
/// The ration rows come from the log of the draw that just ran. The engine
/// keeps that log for one tick, so a site that was served in full states no
/// shortfall.[^2]
///
/// # References
///
/// [^1]: ADR-0070, the head-up display reports what the drawing pass read, decision D1. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
/// [^2]: ADR-0063, a need is a rate with a threshold, and crossing it is a fact, decision D3. `docs/adrs/accepted/adr-0063-a-need-is-a-rate-with-a-threshold-and-crossing-it-is-a-fact.md`
/// Returns how many positions a site declares and how many a unit holds.
///
/// **A position is a seat at a site, not a thing on the map.** The engine
/// holds a fixed-width row of them for each site, so this reads one row and
/// counts it. It starts no pass over the units and no pass over the
/// world.[^1]
///
/// A seat with no holder is counted in the first number and not the second,
/// so a watcher sees the difference between a site with no seats and a site
/// whose seats are empty. Those are different facts and a single number
/// would hide one of them.
///
/// # References
///
/// [^2]: ADR-0014, entity identity is an index plus a generation, decision D2. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
///
/// # References
///
/// [^1]: ADR-0070, the head-up display reports what the drawing pass read, decision D1. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
fn seats_of(world: &World, site: Entity) -> (u32, u32) {
    let Some(rows) = world.site_positions(site) else {
        return (0, 0);
    };
    // The row is read for its length and its kinds. The holder is asked for
    // through the engine's resolving reader, one index at a time.
    let existing: Vec<usize> = rows
        .iter()
        .enumerate()
        .filter(|(_, position)| position.exists())
        .map(|(index, _)| index)
        .collect();
    let seats = existing.len() as u32;
    // **The stored identity is not the question. Whether it resolves is.**
    // A position keeps the bits of its holder after that holder dies, until
    // the pass that releases the dead runs. Counting a non-zero field as a
    // holder would report a seat as filled by a unit that no longer exists,
    // and a watcher would read a fully staffed site where one is short. The
    // engine's reader compares the generation and answers nothing for a dead
    // holder, so the viewer asks it rather than reading the field.[^2]
    let held = existing
        .iter()
        .filter(|index| world.position_holder(site, **index).is_some())
        .count() as u32;
    (seats, held)
}

/// Returns what the character tier holds, in one walk of it.
///
/// **The record permits this walk and no other.** A character tier holds a
/// bounded population and a caller may walk it; the mass tier does not and a
/// caller may not.[^1] So the cost of this follows the number of characters,
/// which is a small named few, and never the population.
///
/// The three answers are the live count, the faction and birth tick of the
/// newest character, and they all come from one pass. Reading them separately
/// would walk the tier three times for one picture.
///
/// # References
///
/// [^1]: ADR-0054, an entity belongs to one of three tiers, declared at creation, decision D1. `docs/adrs/accepted/adr-0054-an-entity-belongs-to-one-of-three-tiers-declared-at-creation.md`
fn read_characters(world: &World) -> (u32, Option<(FactionId, u64)>) {
    let arena = world.characters();
    let mut live = 0;
    let mut newest: Option<(FactionId, u64)> = None;
    for character in arena.iter() {
        live += 1;
        let (Some(faction), Some(birth)) = (arena.faction(character), arena.birth(character))
        else {
            continue;
        };
        if newest.is_none_or(|(_, held)| birth.0 > held) {
            newest = Some((faction, birth.0));
        }
    }
    (live, newest)
}

fn read_sites(world: &World) -> Vec<SiteReadout> {
    // The engine holds one commodity, and it is the one the cohorts draw.
    // The identifier is the engine's, not a second name for it here.
    let commodity = CommodityId(0);
    let arena = world.settlements();
    arena
        .iter()
        .take(SITE_ROWS)
        .filter_map(|site| {
            let place = arena.address(site)?;
            let identity = site.to_bits();
            let (seats, seats_held) = seats_of(world, site);
            Some(SiteReadout {
                faction: arena.faction(site)?,
                place,
                store: arena.store(site)?.quantity(commodity)?,
                production: world.production_rate(site, commodity)?,
                upkeep: world.upkeep_rate(site, commodity)?,
                rationed: world
                    .rationed_log()
                    .iter()
                    .find(|event| event.site == identity)
                    .map(|event| (event.demanded.0, event.granted.0)),
                // The seats of this site, counted on the walk this function
                // already makes. The walk stops at a fixed number of sites
                // and a site declares a fixed number of seats, so the cost
                // follows neither the world nor the population.[^1]
                seats,
                seats_held,
            })
        })
        .collect()
}

/// Reports whether the window covers one tile.
///
/// The camera gives the row range of the window, and the column range of one
/// row. Both are the viewer's own arithmetic. The call reads no tile and no
/// unit.[^1]
///
/// # References
///
/// [^1]: ADR-0070, the head-up display reports what the drawing pass read, decision D1. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
fn shows(address: Axial, world: &World, camera: Camera, canvas: &Canvas) -> bool {
    if address.q < 0 || address.r < 0 {
        return false;
    }
    let (first_row, last_row) = camera.visible_rows(world, canvas);
    let row = address.r as u32;
    if row < first_row || row >= last_row {
        return false;
    }
    let (first_column, last_column) = camera.visible_columns(row, world, canvas);
    let column = address.q as u32;
    column >= first_column && column < last_column
}

/// What the panel says.
///
/// A readout holds numbers and nothing else. It is read once against the
/// world, the camera and the finished canvas, and then it is only drawn.
#[derive(Clone, Debug)]
pub struct Readout {
    tick: u64,
    world_width: u32,
    world_height: u32,
    factions: u16,
    soldiers_live: u32,
    centre: Axial,
    tile_pixels: f32,
    columns_shown: u32,
    rows_shown: u32,
    tiles_painted: u32,
    soldiers_painted: u32,
    blocks_read: u32,
    blocks_skipped: u32,
    crowd_worst: u32,
    tiles_at_capacity: u32,
    by_faction: [u32; COLOURED_FACTIONS],
    by_kind: [u32; KIND_COUNT],
    region: Option<CellSummary>,
    tile: Option<TileReadout>,
    choice: Option<ChoiceReadout>,
    sites: Vec<SiteReadout>,
    sites_held: u32,
    seats: u32,
    seats_taken: u32,
    characters: u32,
    newest_character: Option<(FactionId, u64)>,
    promoted_now: u32,
    promoted_deeds: Option<u64>,
    foundings: Vec<FoundingReport>,
    refusals: Vec<(FactionId, FoundingError)>,
    units_short: u32,
    units_carrying: u32,
    carried_by_kind: [u32; RESOURCE_KIND_COUNT],
    units_housed: u32,
    rationings: u32,
    rationed_short: i64,
    units_ended: usize,
    canvas_height: usize,
    step_mean: f64,
    step_worst: f64,
    draw_mean: f64,
    draw_worst: f64,
    steps_measured: u64,
    frames_measured: u64,
    rate: f64,
    busy: f64,
}

impl Readout {
    /// Reads what the panel will say.
    ///
    /// Call this after the drawing pass. The canvas carries the counts of
    /// that pass, and a readout taken before it would report the pass before
    /// last.
    ///
    /// The world is a shared reference, so the compiler refuses a write to
    /// it.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0067, the viewer reads the world and never writes to it, decision D1. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
    #[must_use]
    pub fn of(
        world: &World,
        camera: Camera,
        canvas: &Canvas,
        metrics: &Metrics,
        outcomes: &[FoundingOutcome],
    ) -> Self {
        let grid = world.grid();
        let (first_row, last_row) = camera.visible_rows(world, canvas);
        let middle_row = (first_row + last_row) / 2;
        let (first_column, last_column) = camera.visible_columns(middle_row, world, canvas);
        // The site list is read once, here, and both the rows and the seat
        // totals below are derived from it. Reading it twice would walk the
        // arena twice and could give two answers.
        let sites = read_sites(world);
        // The character tier, walked once. The record permits a walk of this
        // tier and of no other.
        let (characters, newest_character) = read_characters(world);
        let seats: u32 = sites.iter().map(SiteReadout::seats).sum();
        let seats_taken: u32 = sites.iter().map(SiteReadout::seats_held).sum();

        Self {
            // The tick is the engine's own counter. The viewer reads it and
            // keeps no copy of its own, because two counters for one number
            // is one fact in two places.
            tick: world.tick().0,
            world_width: grid.width(),
            world_height: grid.height(),
            factions: world.config().faction_count,
            soldiers_live: world.soldiers().len(),
            centre: camera.tile_at(canvas.width() as f32 / 2.0, canvas.height() as f32 / 2.0),
            tile_pixels: camera.tile_width,
            columns_shown: last_column.saturating_sub(first_column),
            rows_shown: last_row.saturating_sub(first_row),
            tiles_painted: canvas.tiles_painted(),
            soldiers_painted: canvas.soldiers_painted(),
            blocks_read: canvas.blocks_read(),
            blocks_skipped: canvas.blocks_skipped(),
            crowd_worst: canvas.crowd_worst(),
            tiles_at_capacity: canvas.tiles_at_capacity(),
            by_faction: *canvas.painted_by_faction(),
            by_kind: *canvas.painted_by_kind(),
            // The level 1 cell that covers the tile under the middle of the
            // window. The camera reports that tile, and the engine reports
            // the cell. Neither number is the viewer's.
            canvas_height: canvas.height(),
            region: world.summary_covering(
                camera.tile_at(canvas.width() as f32 / 2.0, canvas.height() as f32 / 2.0),
            ),
            // The tile under the middle of the window, read at one address.
            // Every quantity is one the engine holds.[^6]
            //
            // [^6]: ADR-0070, the head-up display reports what the drawing pass read, decision D1. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
            tile: TileReadout::of(
                world,
                camera.tile_at(canvas.width() as f32 / 2.0, canvas.height() as f32 / 2.0),
            ),
            // The drawing pass fixed which unit this is while it painted, so
            // finding it costs nothing here. The engine then answers for that
            // one unit.[^6]
            choice: canvas.focus().map(|focus| ChoiceReadout::of(world, focus)),
            // One row for each site the world holds. The founding seats one
            // for each faction, so the list is bounded by the faction
            // count.[^6]
            sites,
            // The arena reports how many sites it holds without a walk over
            // them, so the panel states the whole count beside the few rows
            // it read.[^6]
            sites_held: world.settlements().len(),
            // The seats of the sites the panel read, summed over that same
            // bounded walk. This is a count of the sites read and not of the
            // world, and the row that shows it says so.[^4]
            seats,
            seats_taken,
            characters,
            newest_character,
            // **The log holds the promotions of this frame and no other.**
            // It is how a watcher sees a soldier become a character at the
            // moment it happens, rather than reading a number that went up.
            promoted_now: world.promoted_log().len() as u32,
            promoted_deeds: world.promoted_log().first().map(|event| event.deeds),
            // The caller founded the run and kept what the founding
            // returned. The panel borrows that value and recomputes no part
            // of it, so the list is as long as the caller made it and the
            // layout never assumes one founding.
            foundings: outcomes
                .iter()
                .filter_map(|outcome| {
                    let founding = outcome.founding()?;
                    FoundingReport::of(outcome.faction(), founding, world, camera, canvas)
                })
                .collect(),
            // A refused faction founded nowhere. The panel names it all the
            // same, because a run that seats three of four factions and
            // states three foundings tells a watcher nothing about the
            // fourth.[^3]
            //
            // [^3]: ADR-0076, a founding keeps a fixed distance from the foundings before it, decision D2. `docs/adrs/accepted/adr-0076-a-founding-keeps-a-fixed-distance-from-the-foundings-before-it.md`
            refusals: outcomes
                .iter()
                .filter_map(|outcome| match outcome.result() {
                    Ok(_) => None,
                    Err(error) => Some((outcome.faction(), *error)),
                })
                .collect(),
            // The drawing pass counted these while it painted. They are
            // counts of the window.[^4]
            //
            // [^4]: ADR-0070, the head-up display reports what the drawing pass read, decision D1. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
            units_short: canvas.units_short(),
            units_carrying: canvas.units_carrying(),
            carried_by_kind: *canvas.carried_by_kind(),
            units_housed: canvas.units_housed(),
            // The store of a site rations when it cannot serve what its
            // cohorts asked for. The engine holds the log of the step that
            // just ran, so this is a count of the world and the label of its
            // row says so.[^5] It happens twice in a tick in the
            // demonstration world and no watcher could see it.[^6]
            //
            // [^6]: Backlog item 0274. `docs/backlog/complete/0274-show-the-load-a-unit-carries-and-the-home-it-keeps.md`
            rationings: world.rationed_log().len() as u32,
            rationed_short: world
                .rationed_log()
                .iter()
                // The engine states that a rationed draw always granted less
                // than it was asked for, so the difference is never negative.
                // The floor is here because a shortfall that read as a large
                // positive number would be a wrong answer presented as a
                // right one, and this costs one comparison.
                .map(|event| event.demanded.0.saturating_sub(event.granted.0).max(0))
                .sum(),
            // The engine holds the log of the scan that just ran. This is a
            // count of the world, and the label of its row says so.[^5]
            //
            // [^5]: ADR-0070, the head-up display reports what the drawing pass read, decision D2. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
            units_ended: world.starved_log().len(),
            step_mean: metrics.step_mean_micros(),
            step_worst: metrics.step_worst_micros(),
            draw_mean: metrics.draw_mean_micros(),
            draw_worst: metrics.draw_worst_micros(),
            // The counts travel with the means, because a mean over no
            // measurement is not a small number. It is an absent one.[^6]
            //
            // [^6]: ADR-0070, the head-up display reports what the drawing pass read, decision D2. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
            steps_measured: metrics.ticks(),
            frames_measured: metrics.frames(),
            rate: metrics.ticks_each_second(),
            busy: metrics.busy_percent(),
        }
    }

    /// Returns the tick the engine has reached.
    #[must_use]
    pub const fn tick(&self) -> u64 {
        self.tick
    }

    /// Returns the tile under the middle of the window.
    #[must_use]
    pub const fn centre(&self) -> Axial {
        self.centre
    }

    /// Returns the soldiers the world holds.
    #[must_use]
    pub const fn soldiers_live(&self) -> u32 {
        self.soldiers_live
    }

    /// Returns the soldiers the window shows.
    #[must_use]
    pub const fn soldiers_painted(&self) -> u32 {
        self.soldiers_painted
    }

    /// Returns the soldiers the window shows, one count for each colour.
    ///
    /// This is a census of the window. It is not a census of the world, and
    /// the panel labels it so.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0070, the head-up display reports what the drawing pass read, decision D2. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
    #[must_use]
    pub const fn by_faction(&self) -> &[u32; COLOURED_FACTIONS] {
        &self.by_faction
    }

    /// Returns the largest number of units the drawing pass painted on one
    /// tile.
    ///
    /// The number counts the window. The panel says so, because a reader who
    /// wants the largest number on any tile of the world must learn that the
    /// panel has no such number.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0070, the head-up display reports what the drawing pass read, decision D2. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
    #[must_use]
    pub const fn crowd_worst(&self) -> u32 {
        self.crowd_worst
    }

    /// Returns the painted tiles that hold at least as many units as they
    /// admit.
    ///
    /// The capacity composes the ground with the finished upgrade, which is
    /// what admission reads. The viewer asks the engine and holds neither
    /// table.[^1] [^3] The number counts the window.[^2]
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

    /// Returns the tiles the last drawing pass painted.
    ///
    /// The count is of the window. The panel states it beside the counts by
    /// kind, and a test reads the two together.
    #[must_use]
    pub const fn tiles_painted(&self) -> u32 {
        self.tiles_painted
    }

    /// Returns the tiles of each kind that the last drawing pass painted.
    ///
    /// The index is the kind number that the engine fixes. The panel names
    /// each kind against this count, so a person can say what the ground in
    /// the window is.[^1]
    ///
    /// # References
    ///
    /// [^1]: PRD-0003, a developer sees a world worth looking at. `docs/product/accepted/prd-0003-a-developer-sees-a-world-worth-looking-at.md`
    #[must_use]
    pub const fn by_kind(&self) -> &[u32; KIND_COUNT] {
        &self.by_kind
    }

    /// Returns the level 1 summary of the region under the middle of the
    /// window.
    ///
    /// The value is the engine's. The panel turns it into text and hands
    /// nothing back.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0067, the viewer reads the world and never writes to it, decision D3. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
    #[must_use]
    pub const fn region(&self) -> Option<CellSummary> {
        self.region
    }

    /// Returns what the tile under the middle of the window holds.
    ///
    /// Returns `None` when the middle of the window lies outside the world.
    #[must_use]
    pub const fn tile(&self) -> Option<TileReadout> {
        self.tile
    }

    /// Returns why the unit nearest the middle of the window chose what it
    /// chose.
    ///
    /// Returns `None` when the drawing pass painted no unit.
    #[must_use]
    pub const fn choice(&self) -> Option<ChoiceReadout> {
        self.choice
    }

    /// Returns what each of the first few sites produces, holds and owes.
    ///
    /// The list stops at a fixed number of rows, so a large world costs the
    /// panel no more than a small one.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0070, the head-up display reports what the drawing pass read, decision D1. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
    #[must_use]
    pub fn sites(&self) -> &[SiteReadout] {
        &self.sites
    }

    /// Returns the number of sites the world holds.
    ///
    /// This is a count of the world, and the panel labels it so. The arena
    /// gives it at once, so it costs no walk.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0070, the head-up display reports what the drawing pass read, decision D1. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
    #[must_use]
    pub const fn sites_held(&self) -> u32 {
        self.sites_held
    }

    /// Returns what each founding chose.
    ///
    /// The list is as long as the caller made it. A run with two foundings
    /// gives two reports, and the panel states both.[^1]
    ///
    /// # References
    ///
    /// [^1]: Blockers register, BLK-018. `docs/BLOCKERS.md`
    #[must_use]
    pub fn foundings(&self) -> &[FoundingReport] {
        &self.foundings
    }

    /// Returns each faction that found no place, and the reason.
    #[must_use]
    pub fn refusals(&self) -> &[(FactionId, FoundingError)] {
        &self.refusals
    }

    /// Returns the number of drawn units that a shortage holds.
    ///
    /// This counts the units the frame painted, and never the units of the
    /// world.
    #[must_use]
    pub const fn units_short(&self) -> u32 {
        self.units_short
    }

    /// Returns the number of units that the last scan ended.
    ///
    /// This is a count of the world, not of the window. The engine keeps the
    /// log of one scan, so the number falls back to zero on a tick that ends
    /// nobody.
    ///
    /// A watcher cannot see a unit at the moment a shortage ends it, because
    /// the engine scans inside the step that takes the unit to the bound.
    /// This row is the whole record of a death that the window holds.[^1]
    ///
    /// # References
    ///
    /// [^1]: Findings register, FND-119. `docs/FINDINGS.md`
    /// Returns how many characters the world holds.
    ///
    /// This is a count of the world. The viewer walks the character tier,
    /// which the record permits because that tier is bounded.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0054, an entity belongs to one of three tiers, declared at creation, decision D1. `docs/adrs/accepted/adr-0054-an-entity-belongs-to-one-of-three-tiers-declared-at-creation.md`
    #[must_use]
    pub const fn characters(&self) -> u32 {
        self.characters
    }

    /// Returns the faction and birth tick of the newest character.
    ///
    /// **This is what lets a promotion stay on the glass after the frame it
    /// happened in.** The log holds one frame, so a card driven by the log
    /// alone would show a promotion for a thirteenth of a second and then
    /// forget it. The birth tick is stored, so the viewer can say how long
    /// ago it was without keeping a memory of its own.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0067, the viewer reads the world and never writes to it, decision D2. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
    #[must_use]
    pub const fn newest_character(&self) -> Option<(FactionId, u64)> {
        self.newest_character
    }

    /// Returns how many units the last step promoted.
    #[must_use]
    pub const fn promoted_now(&self) -> u32 {
        self.promoted_now
    }

    /// Returns the deeds that earned the first promotion of the last step.
    #[must_use]
    pub const fn promoted_deeds(&self) -> Option<u64> {
        self.promoted_deeds
    }

    /// Returns how many positions the sites the panel read declare.
    ///
    /// A position is a seat at a site. This counts the sites the panel walked,
    /// which is a fixed number, so it is neither a count of the world nor of
    /// the window and the row that shows it says which.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0070, the head-up display reports what the drawing pass read, decision D2. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
    #[must_use]
    pub const fn seats(&self) -> u32 {
        self.seats
    }

    /// Returns how many of those positions a live unit holds.
    #[must_use]
    pub const fn seats_taken(&self) -> u32 {
        self.seats_taken
    }

    /// Returns the number of painted units that carry something.
    ///
    /// This is a count of the window. The drawing asked at every unit it
    /// painted, on the loop that already ran.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0070, the head-up display reports what the drawing pass read, decision D1. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
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

    /// Returns the number of sites that rationed in the last step.
    ///
    /// This is a count of the world and not of the window, because the engine
    /// holds the log of the step that just ran.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0070, the head-up display reports what the drawing pass read, decision D2. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
    #[must_use]
    pub const fn rationings(&self) -> u32 {
        self.rationings
    }

    /// Returns how much the rationed draws of the last step fell short by.
    ///
    /// **The value is in accumulator units, not in whole goods.** An
    /// accumulator is fixed point at a scale of 65536, so a caller that
    /// printed this number raw would state a quantity about sixty-five
    /// thousand times the real one. The panel formats it, and a caller that
    /// wants the quantity divides by the scale itself, exactly.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0002, simulated and aggregated state holds no floating point number. `docs/adrs/REGISTRY.md`
    #[must_use]
    pub const fn rationed_short(&self) -> i64 {
        self.rationed_short
    }

    #[must_use]
    pub const fn units_ended(&self) -> usize {
        self.units_ended
    }

    /// Returns the number of factions the world holds.
    ///
    /// The colour table is larger than most worlds need, so a legend sized by
    /// the table names a colour that no faction uses. A reader would then look
    /// for a faction that is not there.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D2. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
    #[must_use]
    pub const fn factions(&self) -> u16 {
        self.factions
    }

    /// Returns the mean cost of a step on this machine, in microseconds.
    ///
    /// The figure describes a development machine. It is not evidence about
    /// the target platform, and the row that states it says so.[^1]
    ///
    /// # References
    ///
    /// [^1]: Project orientation, the target platform. `CLAUDE.md`
    #[must_use]
    pub const fn step_mean(&self) -> f64 {
        self.step_mean
    }

    /// Returns the mean cost of a drawing on this machine, in microseconds.
    #[must_use]
    pub const fn draw_mean(&self) -> f64 {
        self.draw_mean
    }

    /// Returns the number of ticks the run reached in one second.
    #[must_use]
    pub const fn rate(&self) -> f64 {
        self.rate
    }

    /// Returns the number of steps the mean step cost was taken over.
    #[must_use]
    pub const fn steps_measured(&self) -> u64 {
        self.steps_measured
    }

    /// Returns the number of drawings the mean drawing cost was taken over.
    ///
    /// **A drawing cannot measure itself.** The cost of one frame is recorded
    /// after that frame has been drawn, so the cost the window states is the
    /// mean of the frames before it. The first frame of a run has no frame
    /// before it, and a picture written by one call to the drawing has none
    /// either. In both cases this returns zero, and the caller states that the
    /// number is absent rather than printing a mean over nothing.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0070, the head-up display reports what the drawing pass read, decision D2. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
    #[must_use]
    pub const fn frames_measured(&self) -> u64 {
        self.frames_measured
    }

    /// Returns a cost in microseconds, or the words that say it was not taken.
    ///
    /// A count of zero means no measurement exists. Printing `0.0 ms` for that
    /// case states a measurement that nobody took.[^1]
    ///
    /// # References
    ///
    /// [^1]: Definition of Done, report the work honestly. `.claude/rules/definition-of-done.md`
    #[must_use]
    pub fn cost_in_millis(micros: f64, count: u64) -> String {
        if count == 0 {
            NOT_MEASURED.to_string()
        } else {
            format!("{:.1} ms", micros / 1000.0)
        }
    }

    /// Returns the tiles the window shows, across and down.
    #[must_use]
    pub const fn extent_shown(&self) -> (u32, u32) {
        (self.columns_shown, self.rows_shown)
    }

    /// Returns the number of legend rows the panel draws.
    ///
    /// A faction beyond the colour table shares a colour with an earlier one,
    /// so the legend stops at the table and the panel says that it did.
    fn legend_rows(&self) -> usize {
        (self.factions as usize).clamp(1, COLOURED_FACTIONS)
    }
}

/// The kinds of ground, in the order the engine numbers them.
///
/// The engine gives a kind a number and the viewer reads it back, so the two
/// orders must agree. A test asserts that each entry sits at its own number,
/// because a table that silently drifts from the numbering paints the wrong
/// colour and names the wrong ground.[^1]
///
/// # References
///
/// [^1]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
pub(crate) const KINDS: [TileKind; KIND_COUNT] = [
    TileKind::Water,
    TileKind::Plain,
    TileKind::Forest,
    TileKind::Hill,
    TileKind::Mountain,
];

/// One line of the panel.
///
/// The panel is a list of these. The list is built once and it is the only
/// statement of what the panel holds. The height is summed from it and the
/// painting walks it, so the two cannot disagree.
///
/// An earlier version stated the height with its own arithmetic while the
/// painting produced the same geometry line by line. That is one fact in two
/// places, and nothing failed when the copies drifted.[^1]
///
/// # References
///
/// [^1]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
#[derive(Clone, Debug)]
enum Line {
    /// The name of the viewer, at twice the glyph size.
    Title(&'static str),
    /// A dim line that runs the width of the panel.
    Note(&'static str),
    /// A hairline between two sections.
    Rule,
    /// The name of a section.
    Heading(&'static str),
    /// A label on the left and a value against the right edge.
    Row(&'static str, String),
    /// A colour swatch, the faction it stands for, and its count.
    Legend(usize, u32),
    /// A colour swatch, the kind of ground it stands for, and its count.
    Ground(TileKind, u32),
    /// A colour swatch, the faction it stands for, and what that faction got
    /// when the run founded.
    Founded(FactionId, String),
    /// The bar that shows the shares of the visible units.
    Bar,
}

impl Line {
    /// Returns the height this line occupies, in pixels.
    const fn height(&self) -> i32 {
        match self {
            Self::Title(_) => 18,
            Self::Note(_)
            | Self::Heading(_)
            | Self::Row(_, _)
            | Self::Legend(_, _)
            | Self::Ground(_, _)
            | Self::Founded(_, _) => LINE,
            Self::Rule => 8,
            Self::Bar => BAR_HEIGHT + 8,
        }
    }
}

/// Returns what the panel says, as one line for each heading and row.
///
/// A heading gives its own name. A row gives its label and its value,
/// separated by a colon. The reading walks the same list the painting walks,
/// so a test that reads this reads what a watcher sees.[^1]
///
/// The glass has the same reading, and the pair is what lets a test say where
/// a quantity is shown rather than only that it was read.[^2] A count that
/// leaves the glass because it never moves must arrive in the panel, and a
/// count that is on neither surface is a loss rather than restraint.
///
/// The list is before the cut, so this says what the panel holds and not what
/// fits in the window it was last drawn at.
///
/// # References
///
/// [^1]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
/// [^2]: ADR-0093, the window shows what changes, decision D1. `docs/adrs/draft/adr-0093-the-window-shows-what-changes.md`
#[must_use]
pub fn says(readout: &Readout) -> Vec<String> {
    readout
        .all_lines()
        .iter()
        .filter_map(|line| match line {
            Line::Rule | Line::Bar => None,
            Line::Title(text) | Line::Note(text) | Line::Heading(text) => Some((*text).to_owned()),
            Line::Row(label, value) => Some(format!("{label}: {value}")),
            Line::Legend(faction, count) => Some(format!("faction {faction}: {count}")),
            Line::Ground(kind, count) => Some(format!("{}: {count}", name_of(*kind))),
            Line::Founded(faction, value) => Some(format!("faction {}: {value}", faction.0)),
        })
        .collect()
}

impl Readout {
    /// Builds the lines the panel holds.
    ///
    /// This is the whole content of the panel, in order. Nothing else states
    /// what the panel says.
    ///
    /// Each label names one thing, and no two labels name the same thing with
    /// different words. A reader must be able to tell a count of the world
    /// from a count of the window by the label alone, never by the section it
    /// sits under.
    fn lines(&self) -> Vec<Line> {
        cut_to_fit(self.all_lines(), self.canvas_height)
    }

    /// Returns every line the panel holds, before any cut.
    ///
    /// **One list is the only statement of what the panel holds.** The cut,
    /// the height and the painting are all derived from this, so a caller
    /// that asks how tall the panel is and a caller that draws it cannot
    /// disagree.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0070, the head-up display reports what the drawing pass read, decision D1. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
    fn all_lines(&self) -> Vec<Line> {
        let mut lines = vec![
            Line::Title("CACHETTE"),
            Line::Note("watching the world run"),
            Line::Rule,
            Line::Heading("WORLD"),
            Line::Row("tick", grouped(self.tick)),
            Line::Row(
                "extent",
                format!("{} x {} tiles", self.world_width, self.world_height),
            ),
            Line::Row("units alive", grouped(u64::from(self.soldiers_live))),
            Line::Rule,
            // The shortage rows sit high, because a watcher who opens the
            // window watches units end. A section near the foot is the first
            // thing a short window cuts.
            Line::Heading("SHORTAGE"),
            // Two counts of the window and one of the world. The label of
            // each row says which, and a note under each pair repeats it.
            // A reader must never learn it from the heading.[^2]
            //
            // [^2]: ADR-0070, the head-up display reports what the drawing pass read, decision D2. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
            Line::Row("short in window", grouped(u64::from(self.units_short))),
            Line::Row("ended in world", grouped(self.units_ended as u64)),
        ];

        // The store of a site rations when it cannot serve what its cohorts
        // asked for. It happens more than once in a tick in the demonstration
        // world, and until these rows the only way to see it was to read the
        // log.[^11] This is a count of the world, and the label says so.[^2]
        //
        // **The rows appear only when a site rationed.** A row that reported
        // a permanent zero would take a row from every panel of every run to
        // say nothing, and it would still say nothing if the subsystem behind
        // it broke.[^12] The count is what decides, so the rows cannot
        // outlive the behaviour they report.
        //
        // [^11]: Backlog item 0274. `docs/backlog/complete/0274-show-the-load-a-unit-carries-and-the-home-it-keeps.md`
        // [^12]: Testing Rules, section 2a. `.claude/rules/testing.md`
        if self.rationings > 0 {
            lines.push(Line::Row(
                "sites rationed in world",
                grouped(u64::from(self.rationings)),
            ));
            lines.push(Line::Row(
                "  they fell short by",
                accumulated(self.rationed_short),
            ));
        }

        // **A soldier became a character because of what it did.** The
        // panel names the running count and, on the frame it happens, the
        // deeds that earned the promotion. The section appears only once
        // somebody has been promoted, so it cannot report a permanent zero
        // and it cannot go on reporting one if the pass breaks.[^12]
        if self.characters > 0 {
            lines.push(Line::Rule);
            lines.push(Line::Heading("THE CHARACTERS"));
            lines.push(Line::Row(
                "characters in world",
                grouped(u64::from(self.characters)),
            ));
            if self.promoted_now > 0 {
                lines.push(Line::Row(
                    "promoted this frame",
                    grouped(u64::from(self.promoted_now)),
                ));
                if let Some(deeds) = self.promoted_deeds {
                    lines.push(Line::Row("  for deeds", grouped(deeds)));
                }
            }
            if let Some((faction, birth)) = self.newest_character {
                // **The value column is narrow and a founded row does not
                // cut.** A longer value here wrote past the right edge of the
                // panel, which a test caught. The words go in the note; the
                // value stays short.
                lines.push(Line::Founded(
                    faction,
                    grouped(self.tick.saturating_sub(birth)),
                ));
                lines.push(Line::Note("the newest, in ticks ago."));
            }
            lines.push(Line::Note("counts of the world, not the"));
            lines.push(Line::Note("window."));
        }

        // What the drawn units are hauling and where they live. Both are
        // counts of the window, taken on the loop that painted them.[^11]
        //
        // **The section appears only when something is being carried.** A
        // section that reported a permanent zero would take a row from every
        // panel of every run to say nothing, and it would still say nothing
        // if the subsystem behind it broke.[^12]
        //
        // [^12]: Testing Rules, section 2a. `.claude/rules/testing.md`
        if self.units_carrying > 0 {
            lines.push(Line::Rule);
            lines.push(Line::Heading("WHAT THEY CARRY"));
            lines.push(Line::Row(
                "carrying in window",
                fraction_of(self.units_carrying, self.soldiers_painted),
            ));
            for (label, kind) in [
                ("food carried", ResourceKind::Food),
                ("wood carried", ResourceKind::Wood),
                ("stone carried", ResourceKind::Stone),
            ] {
                lines.push(Line::Row(
                    label,
                    grouped(u64::from(self.carried_by_kind[kind as usize])),
                ));
            }
            lines.push(Line::Row(
                "with a home site",
                fraction_of(self.units_housed, self.soldiers_painted),
            ));
            lines.push(Line::Note("counted at the units drawn."));
        }

        // The tile a watcher scrolled to, the unit nearest it, and the sites
        // that feed the world. All three sit above the view rows, because a
        // section near the foot is the first thing a short window cuts.[^4]
        //
        // [^4]: Backlog item 0188. `docs/backlog/complete/0188-show-the-food-of-a-tile-and-the-reason-a-unit-chose.md`
        lines.extend(tile_lines(self.tile.as_ref()));
        lines.extend(choice_lines(self.choice.as_ref()));
        lines.extend(site_lines(
            &self.sites,
            self.sites_held,
            (self.seats_taken, self.seats),
        ));

        // One row for each faction the run answered, seated or refused. A
        // panel that listed the foundings alone would say nothing about a
        // faction that found no place, and a watcher would read three rows
        // in a world of four factions and learn nothing from the gap.[^3]
        //
        // [^3]: ADR-0076, a founding keeps a fixed distance from the foundings before it, decision D2. `docs/adrs/accepted/adr-0076-a-founding-keeps-a-fixed-distance-from-the-foundings-before-it.md`
        if !self.foundings.is_empty() || !self.refusals.is_empty() {
            lines.push(Line::Rule);
            lines.push(Line::Heading("WHO FOUNDED"));
            for founding in &self.foundings {
                lines.push(Line::Founded(
                    founding.faction,
                    format!("q {}  r {}", founding.place.q, founding.place.r),
                ));
            }
            for (faction, error) in &self.refusals {
                lines.push(Line::Founded(*faction, refusal_text(*error)));
            }
            lines.push(Line::Note("a ring marks each place drawn."));
        }

        lines.extend([
            Line::Rule,
            Line::Heading("VIEW"),
            Line::Row(
                "centre tile",
                format!("q {}  r {}", self.centre.q, self.centre.r),
            ),
            Line::Row("zoom", format!("{:.0} px a tile", self.tile_pixels)),
            Line::Row(
                "showing",
                format!("{} x {} tiles", self.columns_shown, self.rows_shown),
            ),
            Line::Row("tiles drawn", grouped(u64::from(self.tiles_painted))),
            Line::Row("units drawn", grouped(u64::from(self.soldiers_painted))),
            Line::Rule,
            Line::Heading("CROWDING IN THE WINDOW"),
            // Both rows name the window in the label. The panel holds no
            // count of the world, and a reader must learn that from the
            // label alone rather than from the heading above it.[^2]
            //
            // [^2]: ADR-0070, the head-up display reports what the drawing pass read, decision D2. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
            Line::Row("most on a drawn tile", grouped(u64::from(self.crowd_worst))),
            Line::Row(
                "drawn tiles at capacity",
                grouped(u64::from(self.tiles_at_capacity)),
            ),
            Line::Note("of the drawn tiles only. the"),
            Line::Note("panel counts no other tile."),
            Line::Rule,
            Line::Heading("FACTIONS IN THE WINDOW"),
        ]);

        for (slot, count) in self.by_faction.iter().enumerate().take(self.legend_rows()) {
            lines.push(Line::Legend(slot, *count));
        }
        lines.push(Line::Bar);

        // The region rows sit under their own heading, because a count of a
        // region is a third kind of count beside a count of the world and a
        // count of the window. A reader must tell them apart by the label.
        if let Some(region) = self.region {
            lines.extend([
                Line::Rule,
                Line::Heading("REGION UNDER THE CROSSHAIR"),
                Line::Row("tiles", grouped(region.tiles().unsigned_abs())),
                Line::Row("open ground", grouped(region.open_tiles().unsigned_abs())),
                Line::Row("units here", grouped(region.units().unsigned_abs())),
                Line::Row("units a tile", fraction(region.units_for_each_open_tile())),
                Line::Row("open share", fraction(region.open_share())),
                Line::Row("mean height", fraction(region.mean_height())),
            ]);
        }

        lines.push(Line::Rule);
        lines.push(Line::Heading("GROUND IN THE WINDOW"));
        // Every kind gets a row, including a kind the window does not hold.
        // A row that disappears at zero would let a reader believe the world
        // has four kinds of ground.
        for (ordinal, count) in self.by_kind.iter().enumerate() {
            lines.push(Line::Ground(KINDS[ordinal], *count));
        }

        lines.extend([
            Line::Rule,
            Line::Heading("COST ON THIS MACHINE"),
            Line::Row(
                "step",
                pair_of_micros(self.step_mean, self.step_worst, self.steps_measured),
            ),
            Line::Row(
                "draw",
                pair_of_micros(self.draw_mean, self.draw_worst, self.frames_measured),
            ),
            Line::Row(
                "rate",
                if self.steps_measured == 0 {
                    NOT_MEASURED.to_string()
                } else {
                    format!("{:.1} a second", self.rate)
                },
            ),
            Line::Row("busy", format!("{:.0} in 100", self.busy)),
            // Two rows, not one. A single row that named both counts did not
            // fit the value column at every zoom, and the clip that kept it
            // inside the panel cut the last word in half.
            Line::Row("blocks read", grouped(u64::from(self.blocks_read))),
            Line::Row("blocks skipped", grouped(u64::from(self.blocks_skipped))),
            Line::Rule,
            Line::Note("mean and worst, one run, one"),
            Line::Note("machine. not the target."),
        ]);

        // One section for each founding the caller holds. The panel never
        // says "the founding", because a run may found more than one group.
        // A row that named one founding would state a false thing the moment
        // a second group founds.[^1]
        //
        // [^1]: Blockers register, BLK-018. `docs/BLOCKERS.md`
        //
        // These sections sit last. They describe a choice the engine made
        // once, before the first frame, and they cost twelve rows for each
        // faction. Every section that reports the world as it stands now
        // therefore comes first, and the cut takes the history rather than
        // the present.[^5]
        //
        // [^5]: Backlog item 0188. `docs/backlog/complete/0188-show-the-food-of-a-tile-and-the-reason-a-unit-chose.md`
        for (ordinal, founding) in self.foundings.iter().enumerate() {
            lines.extend([
                Line::Rule,
                Line::Heading("FOUNDING"),
                Line::Row(
                    "number",
                    format!("{} of {}", ordinal + 1, self.foundings.len()),
                ),
                Line::Row("places compared", grouped(founding.considered as u64)),
                Line::Row(
                    "it took",
                    format!("q {}  r {}", founding.place.q, founding.place.r),
                ),
                Line::Row(
                    "in the window",
                    String::from(if founding.shown { "yes" } else { "no" }),
                ),
            ]);
            match founding.other {
                // A survey of one place has nothing to compare. The panel
                // says so and states the quantities of the chosen place
                // alone, rather than printing a second column of zeroes that
                // a reader would take for a real place.
                None => {
                    lines.push(Line::Note("it compared no other place."));
                    lines.push(Line::Heading("WHAT IT TOOK"));
                    lines.extend(provision_rows(founding.chosen, None));
                }
                Some((address, provision)) => {
                    lines.push(Line::Row(
                        "it left",
                        format!("q {}  r {}", address.q, address.r),
                    ));
                    lines.push(Line::Heading("TOOK / LEFT"));
                    lines.extend(provision_rows(founding.chosen, Some(provision)));
                }
            }
        }

        lines
    }

    /// Returns the height a picture needs to hold the whole panel.
    ///
    /// **A caller that writes the panel to a file uses this instead of
    /// guessing a constant.** The panel grows with the faction count, with
    /// the number of foundings, and with every section that a count switches
    /// on, so a constant that fits today cuts tomorrow and says so only at
    /// the foot of the picture, where nobody generating one reads it.[^1]
    ///
    /// The height is a function of the readout, and the readout is taken
    /// against a canvas, so a caller that resizes may find the answer has
    /// moved: a taller picture paints more units, and a section that a count
    /// switches on adds lines. A caller that wants the whole panel asks
    /// again after it resizes.
    ///
    /// # References
    ///
    /// [^1]: Backlog item 0275. `docs/backlog/complete/0275-size-the-panel-picture-to-the-panel.md`
    #[must_use]
    pub fn height_for_whole_panel(&self) -> usize {
        let content: i32 = self.all_lines().iter().map(Line::height).sum();
        (content + MARGIN * 2 + PAD * 2).max(1) as usize
    }

    /// Returns the height of the panel in pixels.
    ///
    /// The height is the sum of the lines. It follows the content, so a world
    /// with two factions gets a shorter panel than a world with six.
    fn height(&self) -> i32 {
        PAD * 2 + self.lines().iter().map(Line::height).sum::<i32>()
    }
}

/// Returns the reason a faction found no place, short enough for the value
/// column.
///
/// The engine gives the reason. The viewer shortens it for the column and
/// invents none, so a reason the engine adds later reaches this function and
/// not the panel, and the compiler says so.
fn refusal_text(error: FoundingError) -> String {
    match error {
        FoundingError::EmptyGroup => String::from("no people"),
        FoundingError::NoPlaceFound(drawn) => {
            format!("read {}, took none", grouped(u64::from(drawn)))
        }
        FoundingError::OutsideWorld(_) => String::from("outside"),
        FoundingError::Order(_) => String::from("no order"),
        FoundingError::Person(_) => String::from("no person"),
        FoundingError::Seat(_) => String::from("no seat"),
    }
}

/// Returns the rows that compare what two places can reach.
///
/// Every quantity is the one the survey read. The panel restates the report
/// and derives nothing from it, so no number here can disagree with the
/// choice that was made.[^1]
///
/// The chosen place and the place that was left share one row for each
/// quantity, because a watcher compares them. A heading above the rows says
/// which value is which.[^2]
///
/// The fields are the five the score weighs. The room a place holds is not
/// one of them, so the panel does not state it as a reason for the choice.
///
/// # References
///
/// [^1]: ADR-0075, the founding choice reads a bounded sample of the world, decision D5. `docs/adrs/accepted/adr-0075-the-founding-choice-reads-a-bounded-sample-of-the-world.md`
/// [^2]: ADR-0070, the head-up display reports what the drawing pass read, decision D2. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
fn provision_rows(took: Provision, left: Option<Provision>) -> Vec<Line> {
    let pair = |chosen: u32, other: fn(Provision) -> u32| match left {
        None => grouped(u64::from(chosen)),
        Some(left) => format!(
            "{} / {}",
            grouped(u64::from(chosen)),
            grouped(u64::from(other(left)))
        ),
    };
    vec![
        Line::Row("food", pair(took.food.0, |place| place.food.0)),
        Line::Row("wood", pair(took.wood.0, |place| place.wood.0)),
        Line::Row("stone", pair(took.stone.0, |place| place.stone.0)),
        Line::Row(
            "open ground",
            pair(took.open_ground, |place| place.open_ground),
        ),
        Line::Row(
            "water beside",
            pair(took.water_edge, |place| place.water_edge),
        ),
    ]
}

/// Returns the rows that say what the tile under the crosshair holds.
///
/// The food row is the one this section exists for. A watcher reads the
/// colour of the ground to find the food, and reads this row to learn how
/// much of it is left and how much the ground put there.[^1]
///
/// A tile the ground gave nothing of says so, rather than printing "0 of 0",
/// which a reader would take for a drained deposit.
///
/// # References
///
/// [^1]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D4. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
fn tile_lines(tile: Option<&TileReadout>) -> Vec<Line> {
    let mut lines = vec![Line::Rule, Line::Heading("TILE UNDER THE CROSSHAIR")];
    let Some(tile) = tile else {
        lines.push(Line::Note("the middle of the window is"));
        lines.push(Line::Note("outside the world."));
        return lines;
    };
    lines.push(Line::Row(
        "tile",
        format!("q {}  r {}", tile.address.q, tile.address.r),
    ));
    lines.push(Line::Row("ground", name_of(tile.kind).to_string()));
    // The label is a phrase and not the name of the kind, and the panel row
    // takes a static string, so these stay written out. The name of a kind
    // for new code is `resource_name` above.
    for (label, kind) in [
        ("food left", ResourceKind::Food),
        ("wood left", ResourceKind::Wood),
        ("stone left", ResourceKind::Stone),
    ] {
        lines.push(Line::Row(label, deposit(tile, kind)));
    }
    lines.push(Line::Note("left of what the ground gave."));
    lines.push(Line::Row(
        "units here",
        match tile.units {
            None => "-".to_string(),
            Some(count) => grouped(u64::from(count)),
        },
    ));
    lines.push(Line::Row("it admits", grouped(u64::from(tile.capacity))));
    lines.push(Line::Row(
        "held by",
        match tile.holder {
            None => "nobody".to_string(),
            Some(faction) => format!("faction {}", faction.0),
        },
    ));
    lines
}

/// Returns one deposit as text: what is left, of what the ground gave.
///
/// A tile the ground gave nothing of returns a word and not a pair of
/// zeroes.[^1]
///
/// # References
///
/// [^1]: ADR-0070, the head-up display reports what the drawing pass read, decision D2. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
/// Returns the name of a resource kind.
///
/// One name for one kind, in one place. A second table of these names would
/// be one fact in two places, and nothing would fail when the copies
/// disagreed.[^1]
///
/// # References
///
/// [^1]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
#[must_use]
pub const fn resource_name(kind: ResourceKind) -> &'static str {
    match kind {
        ResourceKind::Food => "food",
        ResourceKind::Wood => "wood",
        ResourceKind::Stone => "stone",
    }
}

/// Returns "a of b", for a count against the number the pass painted.
fn fraction_of(part: u32, whole: u32) -> String {
    format!("{part} of {whole}")
}

fn deposit(tile: &TileReadout, kind: ResourceKind) -> String {
    let gave = tile.generated(kind);
    if gave == 0 {
        return "none here".to_string();
    }
    format!("{} of {}", tile.stock(kind), gave)
}

/// Returns the rows that say why one unit chose what it chose.
///
/// The engine answers. The viewer states the answer and derives no part of
/// it, so no row here can disagree with the choice that was made.[^1]
///
/// Each option gets one row: the score it reached, and the value it read from
/// the level 1 cell. A mark names the option the scores select. A reader who
/// wants to know why a unit did not forage reads the forage row and sees
/// either a low field or a low weight.
///
/// # References
///
/// [^1]: ADR-0064, a unit chooses by scoring a small fixed option set, decision D2. `docs/adrs/accepted/adr-0064-a-unit-chooses-by-scoring-a-small-fixed-option-set.md`
fn choice_lines(choice: Option<&ChoiceReadout>) -> Vec<Line> {
    let mut lines = vec![Line::Rule, Line::Heading("WHY THE NEAREST UNIT CHOSE")];
    let Some(choice) = choice else {
        lines.push(Line::Note("the window holds no unit."));
        return lines;
    };
    let focus = choice.focus();
    lines.push(Line::Note("the unit nearest the middle."));
    lines.push(Line::Founded(
        focus.faction(),
        format!("q {}  r {}", focus.address().q, focus.address().r),
    ));
    lines.push(Line::Row(
        "state",
        match focus.condition() {
            None => "-".to_string(),
            Some(NeedCondition::Fed) => "fed".to_string(),
            Some(NeedCondition::Short) => "short".to_string(),
            Some(NeedCondition::Starved) => "starved".to_string(),
        },
    ));

    let Some(answer) = choice.explanation() else {
        lines.push(Line::Note("the engine explains nothing"));
        lines.push(Line::Note("about this unit."));
        return lines;
    };
    lines.push(Line::Row("its cell", grouped(u64::from(answer.cell))));
    lines.push(Line::Row("what it needs", fraction(Some(answer.need))));
    lines.push(Line::Row("a score must beat", fraction(Some(answer.floor))));
    lines.push(Line::Row(
        "it carries",
        option_name(answer.intent).to_string(),
    ));
    lines.push(Line::Row(
        "it would take",
        option_name(answer.best).to_string(),
    ));
    lines.push(Line::Row(
        "it chooses again",
        String::from(if answer.chooses_next_frame {
            "next tick"
        } else {
            "not yet"
        }),
    ));
    lines.push(Line::Heading("SCORE / WHAT IT READ"));
    for (index, option) in OPTIONS.iter().enumerate().take(OPTION_COUNT) {
        let mark = if index as u8 == answer.best { " <" } else { "" };
        lines.push(Line::Row(
            option.name,
            format!(
                "{} / {}{mark}",
                fraction(Some(answer.scores[index])),
                fraction(Some(answer.fields[index]))
            ),
        ));
    }
    lines
}

/// Returns the name of one option, or a word for no option at all.
///
/// The names come from the engine's table. The viewer holds no second table
/// of them, so an option the engine adds reaches this row with no edit
/// here.[^1]
///
/// # References
///
/// [^1]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
pub(crate) fn option_name(option: u8) -> &'static str {
    if option == NO_INTENT {
        return "nothing";
    }
    match OPTIONS.get(option as usize) {
        None => "-",
        Some(row) => row.name,
    }
}

/// Returns the rows that say what each site produces, holds and owes.
///
/// This is the loop that closed and that nothing could see. A watcher reads
/// the store fall when the rate is below the upkeep, and reads the ration row
/// when the store could not serve the units of that site.[^1]
///
/// A world with no site states that it has none, rather than showing an empty
/// heading a reader would take for a broken panel.
///
/// # References
///
/// [^1]: What a unit does in a tick, section 1. `docs/research/what-a-unit-does-in-a-tick.md`
fn site_lines(sites: &[SiteReadout], held: u32, seats: (u32, u32)) -> Vec<Line> {
    let mut lines = vec![Line::Rule, Line::Heading("THE SITES")];
    lines.push(Line::Row("sites in world", grouped(u64::from(held))));
    // **The per-site rows below are bounded and this total is not.** The
    // panel lists the first few sites, so at any real site count the rows
    // cannot state what the world holds. This row is a count of the world and
    // its label says so.[^2]
    //
    // The total is here rather than on the glass because it holds still: a
    // pass opens the seats and fills them in the same tick, and the count
    // reads the same on every frame after that.[^3]
    //
    // [^2]: ADR-0070, the head-up display reports what the drawing pass read,
    //       decision D2. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
    // [^3]: ADR-0093, the window shows what changes, decision D1.
    //       `docs/adrs/draft/adr-0093-the-window-shows-what-changes.md`
    if seats.1 > 0 {
        lines.push(Line::Row(
            "seats held in world",
            format!("{} of {}", seats.0, seats.1),
        ));
    }
    if sites.is_empty() {
        lines.push(Line::Note("the world holds no site."));
        return lines;
    }
    if sites.len() < held as usize {
        lines.push(Line::Note("the first few only."));
    }
    lines.push(Line::Note("store, then rate less upkeep."));
    for site in sites {
        // The place alone, because a longer value runs into the faction
        // name on the left of the same row.
        lines.push(Line::Founded(
            site.faction,
            format!("q {}  r {}", site.place.q, site.place.r),
        ));
        // **A seat with no holder is not the same fact as no seat.** The row
        // states both numbers, so a watcher can tell a site that declares no
        // position from one whose positions are empty.
        if site.seats > 0 {
            lines.push(Line::Row(
                "  seats held",
                format!("{} of {}", site.seats_held, site.seats),
            ));
        }
        lines.push(Line::Row(
            "  store",
            format!(
                "{} {}{}",
                fraction(Some(site.store)),
                if site.production.0 >= site.upkeep.0 {
                    "+"
                } else {
                    "-"
                },
                fraction(Some(
                    Fix32(site.production.0.abs_diff(site.upkeep.0) as i32)
                ))
            ),
        ));
        if let Some((asked, got)) = site.rationed {
            lines.push(Line::Row(
                "  it rationed",
                format!("{} of {}", accumulated(got), accumulated(asked)),
            ));
        }
    }
    lines
}

/// Returns an accumulated fixed-point total as text.
///
/// The accumulator is 64 bits wide and the reading is a decimal. The
/// conversion happens here, and nothing formatted is handed back to the
/// engine.[^1]
///
/// # References
///
/// [^1]: ADR-0067, the viewer reads the world and never writes to it, decision D3. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
pub fn accumulated(total: i64) -> String {
    format!("{:.1}", total as f64 / 65536.0)
}

/// Returns as much of the list as the window has room for.
///
/// The panel's height follows its content, and the content grows with the
/// faction count and with every section. A window shorter than the content
/// cuts the bottom off, and the panel then states a rectangle it did not
/// paint.
///
/// The list is shortened here rather than at the drawing, because one list is
/// the only statement of what the panel holds and both the height and the
/// painting are derived from it. Shortening it in one place keeps the two in
/// step.
///
/// **The last line says the panel was cut.** A number that is missing and
/// says so is a number a reader knows to look elsewhere for. A number that is
/// missing in silence is the failure the record forbids for a number the
/// panel cannot afford, and a number below the edge of the window is exactly
/// that.[^1]
///
/// # References
///
/// [^1]: ADR-0070, the head-up display reports what the drawing pass read, decision D2. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
fn cut_to_fit(lines: Vec<Line>, canvas_height: usize) -> Vec<Line> {
    let notice = Line::Note(CUT_NOTICE);
    // The panel sits below the margin and must end above the far edge.
    let room = canvas_height as i32 - MARGIN * 2 - PAD * 2;
    let whole: i32 = lines.iter().map(Line::height).sum();
    if whole <= room {
        return lines;
    }

    let mut kept: Vec<Line> = Vec::with_capacity(lines.len());
    let mut used = notice.height();
    for line in lines {
        let height = line.height();
        if used + height > room {
            break;
        }
        used += height;
        kept.push(line);
    }
    kept.push(notice);
    kept
}

/// What the panel says when the window cut it.
const CUT_NOTICE: &str = "window too short. panel cut.";

/// Returns a fixed-point reading as text, to two decimal places.
///
/// A reading that the engine could not give returns a dash rather than a
/// zero. A mean over no tile is not zero, and printing it as zero gives a
/// reader a number it cannot tell from a true one.[^1]
///
/// The conversion to a decimal happens here. Rendering is outside simulated
/// state, and nothing formatted is handed back to the engine.[^2]
///
/// # References
///
/// [^1]: ADR-0024, every summary field is declared extensive or intensive, decision D5. `docs/adrs/accepted/adr-0024-every-summary-field-is-declared-extensive-or-intensive.md`
/// [^2]: ADR-0067, the viewer reads the world and never writes to it, decision D3. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
pub(crate) fn fraction(reading: Option<Fix32>) -> String {
    match reading {
        None => "-".to_string(),
        Some(value) => format!("{:.2}", f64::from(value.0) / 65536.0),
    }
}

/// Returns the width in pixels that the value column gives a value.
///
/// The standard states it. This module does not compute it a second time.[^1]
///
/// # References
///
/// [^1]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
const fn value_span() -> i32 {
    panel::value_room()
}

/// Returns every line of the panel that the drawing has to cut.
///
/// **This covers every line kind, and that is the point.** The drawing cuts
/// each kind to the room it has, so no text can reach the map whatever an
/// author writes. A cut is still a defect, because the panel then states
/// something other than what it was given, and it does so in silence. This
/// function is how a test sees one.[^1]
///
/// # References
///
/// [^1]: Backlog item 0300. `docs/backlog/complete/0300-cut-every-panel-line-to-the-width-of-the-panel.md`
#[must_use]
pub fn lines_that_do_not_fit(readout: &Readout) -> Vec<String> {
    let label_room = panel::VALUE_COLUMN;
    let swatch_room = label_room - SWATCH - SWATCH_GAP;
    readout
        .lines()
        .iter()
        .filter_map(|line| match line {
            Line::Rule | Line::Bar => None,
            Line::Title(text) if !panel::fits(text, panel::text_room(), 2) => {
                Some((*text).to_owned())
            }
            Line::Note(text) | Line::Heading(text) if !panel::fits(text, panel::text_room(), 1) => {
                Some((*text).to_owned())
            }
            Line::Row(label, value) if !panel::fits(label, label_room, 1) || !value_fits(value) => {
                Some(format!("{label}: {value}"))
            }
            Line::Legend(slot, count)
                if !panel::fits(&format!("faction {slot}"), swatch_room, 1) =>
            {
                Some(format!("faction {slot}: {count}"))
            }
            Line::Ground(kind, count) if !panel::fits(name_of(*kind), swatch_room, 1) => {
                Some(format!("{}: {count}", name_of(*kind)))
            }
            Line::Founded(faction, value)
                if !panel::fits(&format!("faction {}", faction.0), swatch_room, 1)
                    || !value_fits(value) =>
            {
                Some(format!("faction {}: {value}", faction.0))
            }
            _ => None,
        })
        .collect()
}

/// Returns every value the panel would have to cut to fit its column.
///
/// The panel cuts a value that does not fit, so that text can never be
/// written over the panel edge. A cut value is still a defect: it states
/// something other than the number it was given, and it does so silently.
///
/// This function is how a test sees the cut. A test that only checked the
/// panel edge would pass because of the cut rather than in spite of it.
#[must_use]
pub fn values_that_do_not_fit(readout: &Readout) -> Vec<String> {
    readout
        .lines()
        .iter()
        .filter_map(|line| match line {
            Line::Row(_, value) if !value_fits(value) => Some(value.clone()),
            _ => None,
        })
        .collect()
}

/// Says whether a value fits the column the panel gives it.
///
/// A test calls this on a string of its own, to prove that the check above
/// can answer no. A check with no proven failure mode is decoration.[^1]
///
/// # References
///
/// [^1]: Testing Rules, a determinism test must be able to fail. `.claude/rules/testing.md`
#[must_use]
pub fn value_fits(value: &str) -> bool {
    text::width_of(value, 1) <= value_span()
}

/// Draws the panel over a canvas that already holds the world.
///
/// The function reads the readout and nothing else, so one readout always
/// gives one picture.
///
/// It walks the same list of lines that the height was summed from, so the
/// panel cannot paint past the rectangle it states.
pub fn draw(readout: &Readout, canvas: &mut Canvas) {
    let (left, top, width, height) = bounds(readout);
    let text_left = left + PAD;
    let text_right = left + width - PAD;

    canvas.shade(left, top, width, height, PANEL, PANEL_WEIGHT);
    outline(canvas, left, top, width, height);

    let lines = readout.lines();
    let mut pen = top + PAD;
    for line in &lines {
        paint_line(canvas, text_left, text_right, pen, line, readout);
        pen += line.height();
    }
}

/// Paints one line at the given position.
fn paint_line(
    canvas: &mut Canvas,
    left: i32,
    right: i32,
    pen: i32,
    line: &Line,
    readout: &Readout,
) {
    match line {
        Line::Title(name) => {
            panel::write_fitted(canvas, left, right, pen, name, 2, TITLE);
        }
        Line::Note(note) => {
            panel::write_fitted(canvas, left, right, pen, note, 1, LABEL);
        }
        Line::Rule => canvas.block(left, pen + 3, right - left, 1, EDGE),
        Line::Heading(name) => {
            panel::write_fitted(canvas, left, right, pen, name, 1, HEADING);
        }
        Line::Row(label, value) => row(canvas, left, right, pen, label, value),
        Line::Legend(slot, count) => legend_row(canvas, left, right, pen, *slot, *count),
        Line::Ground(kind, count) => ground_row(canvas, left, right, pen, *kind, *count),
        Line::Founded(faction, value) => founded_row(canvas, left, right, pen, *faction, value),
        Line::Bar => bar(canvas, left, right, pen, readout),
    }
}

/// Returns the rectangle the panel occupies, as a left, top, width and
/// height in pixels.
///
/// The height follows the content, so a caller cannot know it without asking.
/// The panel writes nothing outside this rectangle, and a test reads it to
/// check that.
#[must_use]
pub fn bounds(readout: &Readout) -> (i32, i32, i32, i32) {
    (MARGIN, MARGIN, PANEL_WIDTH, readout.height())
}

/// Draws the panel edge.
fn outline(canvas: &mut Canvas, x: i32, y: i32, width: i32, height: i32) {
    canvas.block(x, y, width, 1, EDGE);
    canvas.block(x, y + height - 1, width, 1, EDGE);
    canvas.block(x, y, 1, height, EDGE);
    canvas.block(x + width - 1, y, 1, height, EDGE);
}

/// Draws a label and its value.
///
/// The value sits against the right edge. A value too wide for the column is
/// cut rather than written over the panel edge, so text never escapes the
/// panel whatever a caller passes.
///
/// The cut is a guard, not a layout. A value that reaches it states something
/// other than the number it was given, and a test reads the same lines to
/// find one.
fn row(canvas: &mut Canvas, left: i32, right: i32, pen: i32, label: &str, value: &str) {
    // The label is bounded by the panel edge and not by the value column. A
    // label wider than the column is legible, because the value is written
    // against the right edge and the two only meet when the pair is wider than
    // the whole row. The bound that matters here is the one that stops text
    // reaching the map.
    let column = left + VALUE_COLUMN;
    panel::write_fitted(canvas, left, right, pen, label, 1, LABEL);
    panel::write_against_right(canvas, column, right, pen, value, VALUE);
}

/// Draws one legend row: a colour swatch, the faction, and its count.
///
/// The row names a colour, not a faction identity beyond the colour table. A
/// world with more factions than colours reuses a colour.
fn legend_row(canvas: &mut Canvas, left: i32, right: i32, pen: i32, slot: usize, count: u32) {
    let colour = faction_colour(cachette_core::FactionId(slot as u16));
    canvas.block(left, pen, SWATCH, SWATCH, colour);
    let column = left + VALUE_COLUMN;
    let text_left = left + SWATCH + SWATCH_GAP;
    panel::write_fitted(
        canvas,
        text_left,
        right,
        pen,
        &format!("faction {slot}"),
        1,
        LABEL,
    );
    panel::write_against_right(
        canvas,
        column,
        right,
        pen,
        &grouped(u64::from(count)),
        VALUE,
    );
}

/// Draws one founding row: a colour swatch, the faction, and what it got.
///
/// The swatch takes the colour of the faction from the one table the viewer
/// owns, so a watcher matches the row against the mark on the picture and
/// against the units of that faction.[^1]
///
/// # References
///
/// [^1]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
fn founded_row(
    canvas: &mut Canvas,
    left: i32,
    right: i32,
    pen: i32,
    faction: FactionId,
    value: &str,
) {
    canvas.block(left, pen, SWATCH, SWATCH, faction_colour(faction));
    let column = left + VALUE_COLUMN;
    let text_left = left + SWATCH + SWATCH_GAP;
    panel::write_fitted(
        canvas,
        text_left,
        right,
        pen,
        &format!("faction {}", faction.0),
        1,
        LABEL,
    );
    panel::write_against_right(canvas, column, right, pen, value, VALUE);
}

/// Draws one ground row: a colour swatch, the kind, and how many tiles of it
/// the last draw painted.
///
/// The name is the viewer's. The engine numbers the kinds and says nothing
/// about what to call them or how to colour them.[^1]
///
/// # References
///
/// [^1]: ADR-0067, the viewer reads the world and never writes to it, decision D2. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
fn ground_row(canvas: &mut Canvas, left: i32, right: i32, pen: i32, kind: TileKind, count: u32) {
    canvas.block(left, pen, SWATCH, SWATCH, kind_colour(kind));
    let column = left + VALUE_COLUMN;
    let text_left = left + SWATCH + SWATCH_GAP;
    panel::write_fitted(canvas, text_left, right, pen, name_of(kind), 1, LABEL);
    panel::write_against_right(
        canvas,
        column,
        right,
        pen,
        &grouped(u64::from(count)),
        VALUE,
    );
}

/// Returns the name the panel gives one kind of ground.
///
/// The product record asks that the kinds be few and that a person be able to
/// name them.[^1] This is where they are named.
///
/// # References
///
/// [^1]: PRD-0003, a developer sees a world worth looking at. `docs/product/accepted/prd-0003-a-developer-sees-a-world-worth-looking-at.md`
#[must_use]
pub const fn name_of(kind: TileKind) -> &'static str {
    match kind {
        TileKind::Water => "water",
        TileKind::Plain => "plain",
        TileKind::Forest => "forest",
        TileKind::Hill => "hill",
        TileKind::Mountain => "mountain",
    }
}

/// Draws the bar that shows the shares of the units in the window.
///
/// A window with no unit draws the empty bar rather than nothing, so the
/// panel keeps its shape and the reader sees that the answer is zero.
fn bar(canvas: &mut Canvas, left: i32, right: i32, pen: i32, readout: &Readout) {
    let rows = readout.legend_rows();
    let total: u32 = readout.by_faction.iter().take(rows).sum();
    let span = right - left;
    canvas.block(left, pen + 2, span, BAR_HEIGHT, EDGE);
    if total == 0 {
        return;
    }
    let mut filled = 0;
    for (slot, count) in readout.by_faction.iter().enumerate().take(rows) {
        let share = (i64::from(*count) * i64::from(span) / i64::from(total)) as i32;
        let colour = faction_colour(cachette_core::FactionId(slot as u16));
        canvas.block(left + filled, pen + 2, share, BAR_HEIGHT, colour);
        filled += share;
    }
}

/// Writes a number with a space between each group of three digits.
///
/// A tick count and a soldier count both reach six digits. A run of six
/// digits is hard to read at eight pixels a glyph.
#[must_use]
pub fn grouped(value: u64) -> String {
    let digits = value.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            out.push(' ');
        }
        out.push(digit);
    }
    out
}

/// The character that stands for a pixel the panel did not write.
const GROUND: char = '.';

/// The character that stands for a colour no part of the panel writes with.
const UNKNOWN: char = '?';

/// Returns every colour the panel writes with, and the character that stands
/// for it in a picture of the layout.
///
/// The list is the whole ink of the panel. A colour that is missing from it
/// paints as an unknown mark, so a new colour shows up in a stored picture
/// rather than passing as one of the old ones.
///
/// Two roles share one amber. The title and the fourth faction swatch are the
/// same colour, so they take the same character. A picture cannot separate
/// two things that a person cannot separate either.
#[must_use]
pub fn ink_key() -> Vec<(u32, char)> {
    let mut key = vec![
        (EDGE, '#'),
        (HEADING, 'h'),
        (LABEL, 'l'),
        (VALUE, 'v'),
        (TITLE, 'y'),
    ];
    for (slot, mark) in ['r', 'b', 'g', 'y', 'm', 'c'].iter().enumerate() {
        key.push((faction_colour(cachette_core::FactionId(slot as u16)), *mark));
    }
    for (kind, mark) in KINDS.iter().zip(['W', 'P', 'F', 'H', 'M']) {
        key.push((kind_colour(*kind), mark));
    }
    key
}

/// Returns a picture of the panel's layout, one character for each pixel.
///
/// # What the picture holds
///
/// A pixel becomes a character. A pixel the panel left alone becomes the
/// ground character, and so does a pixel that the panel only shaded. Every
/// other pixel is ink, and takes the character of the colour it was written
/// in.
///
/// The ground character therefore covers the world under the panel. A picture
/// made this way says where the panel put ink, and says nothing about what
/// the ground looks like, so a change to the terrain colours does not change
/// it.[^1]
///
/// The trailing ground of each row is cut. The position of a character in a
/// row is still its pixel column, because only the tail is missing.
///
/// # Why a picture and not a list of numbers
///
/// The other tests of the panel read one line at a time. A line that is
/// correct on its own can still sit over the line above it, drift out of its
/// column, or fall off the bottom of the window. A picture shows the layout
/// as a whole, which is the only way those three are visible.
///
/// # Panics
///
/// Panics when the two canvases are not the same size. The bare canvas holds
/// the same drawing without the panel, so a different size means the caller
/// passed two unrelated pictures.
///
/// # References
///
/// [^1]: Backlog item 0037. `docs/backlog/complete/0037-check-the-panel-layout-against-a-stored-picture.md`
#[must_use]
pub fn ink_map(panelled: &Canvas, bare: &Canvas) -> String {
    assert!(
        panelled.width() == bare.width() && panelled.height() == bare.height(),
        "the two canvases must be the same size",
    );
    let key = ink_key();
    let width = panelled.width();
    let mut out = String::new();
    for row in 0..panelled.height() {
        let mut line = String::with_capacity(width);
        for column in 0..width {
            let index = row * width + column;
            let over = panelled.pixels()[index];
            let under = bare.pixels()[index];
            let mark = if over == under || over == crate::paint::mix(under, PANEL, PANEL_WEIGHT) {
                GROUND
            } else {
                key.iter()
                    .find(|(ink, _)| *ink == over)
                    .map_or(UNKNOWN, |(_, mark)| *mark)
            };
            line.push(mark);
        }
        out.push_str(line.trim_end_matches(GROUND));
        out.push('\n');
    }
    out
}
