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

use cachette_core::founding::{Founding, Provision};
use cachette_core::pyramid::CellSummary;
use cachette_core::terrain::{TileKind, KIND_COUNT};
use cachette_core::{Axial, Fix32, World};

use crate::metrics::Metrics;
use crate::paint::{faction_colour, kind_colour, Camera, Canvas, COLOURED_FACTIONS};
use crate::text;

/// The gap between the window edge and the panel, in pixels.
const MARGIN: i32 = 14;

/// The width of the panel, in pixels.
const PANEL_WIDTH: i32 = 268;

/// The gap between the panel edge and its text, in pixels.
const PAD: i32 = 13;

/// The distance between one line of text and the next, in pixels.
const LINE: i32 = 12;

/// The offset of the value column from the panel text edge, in pixels.
const VALUE_COLUMN: i32 = 96;

/// The height of the bar that shows the faction shares, in pixels.
const BAR_HEIGHT: i32 = 5;

/// The colour of the panel, mixed over the world.
const PANEL: u32 = 0x0009_0e12;

/// How much of the panel colour covers the world under it.
const PANEL_WEIGHT: u8 = 224;

/// The colour of the panel edge and of the rules between sections.
const EDGE: u32 = 0x0027_3a44;

/// The colour of a section heading.
const HEADING: u32 = 0x0074_a6ba;

/// The colour of a label.
const LABEL: u32 = 0x0069_7d87;

/// The colour of a value.
const VALUE: u32 = 0x00d6_e4ea;

/// The colour of the title.
const TITLE: u32 = 0x00e8_c84a;

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
    pub fn of(founding: &Founding, world: &World, camera: Camera, canvas: &Canvas) -> Option<Self> {
        let chosen = founding.survey().chosen()?;
        let other = founding
            .survey()
            .rejected()
            .first()
            .map(|candidate| (candidate.address(), candidate.provision()));
        Some(Self {
            place: founding.place(),
            shown: shows(founding.place(), world, camera, canvas),
            considered: founding.survey().considered(),
            chosen: chosen.provision(),
            other,
        })
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
    foundings: Vec<FoundingReport>,
    canvas_height: usize,
    step_mean: f64,
    step_worst: f64,
    draw_mean: f64,
    draw_worst: f64,
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
        foundings: &[Founding],
    ) -> Self {
        let grid = world.grid();
        let (first_row, last_row) = camera.visible_rows(world, canvas);
        let middle_row = (first_row + last_row) / 2;
        let (first_column, last_column) = camera.visible_columns(middle_row, world, canvas);

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
            // The caller founded the run and kept what the founding
            // returned. The panel borrows that value and recomputes no part
            // of it, so the list is as long as the caller made it and the
            // layout never assumes one founding.
            foundings: foundings
                .iter()
                .filter_map(|founding| FoundingReport::of(founding, world, camera, canvas))
                .collect(),
            step_mean: metrics.step_mean_micros(),
            step_worst: metrics.step_worst_micros(),
            draw_mean: metrics.draw_mean_micros(),
            draw_worst: metrics.draw_worst_micros(),
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

    /// Returns the painted tiles that hold at least as many units as their
    /// ground admits.
    ///
    /// The capacity is a property of the terrain, and the viewer reads it
    /// from there.[^1] The number counts the window.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D4. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
    /// [^2]: ADR-0070, the head-up display reports what the drawing pass read, decision D2. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
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
const KINDS: [TileKind; KIND_COUNT] = [
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
            | Self::Ground(_, _) => LINE,
            Self::Rule => 8,
            Self::Bar => BAR_HEIGHT + 8,
        }
    }
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
            Line::Note("panel has no count of the world."),
            Line::Rule,
            Line::Heading("FACTIONS IN THE WINDOW"),
        ];

        for (slot, count) in self.by_faction.iter().enumerate().take(self.legend_rows()) {
            lines.push(Line::Legend(slot, *count));
        }
        lines.push(Line::Bar);

        // One section for each founding the caller holds. The panel never
        // says "the founding", because a run may found more than one group.
        // A row that named one founding would state a false thing the moment
        // a second group founds.[^1]
        //
        // [^1]: Blockers register, BLK-018. `docs/BLOCKERS.md`
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
                format!("{:.0} / {:.0} us", self.step_mean, self.step_worst),
            ),
            Line::Row(
                "draw",
                format!("{:.0} / {:.0} us", self.draw_mean, self.draw_worst),
            ),
            Line::Row("rate", format!("{:.1} a second", self.rate)),
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

        cut_to_fit(lines, self.canvas_height)
    }

    /// Returns the height of the panel in pixels.
    ///
    /// The height is the sum of the lines. It follows the content, so a world
    /// with two factions gets a shorter panel than a world with six.
    fn height(&self) -> i32 {
        PAD * 2 + self.lines().iter().map(Line::height).sum::<i32>()
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
fn fraction(reading: Option<Fix32>) -> String {
    match reading {
        None => "-".to_string(),
        Some(value) => format!("{:.2}", f64::from(value.0) / 65536.0),
    }
}

/// Returns the width in pixels that the value column gives a value.
const fn value_span() -> i32 {
    PANEL_WIDTH - PAD * 2 - VALUE_COLUMN
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
            canvas.write(left, pen, name, 2, TITLE);
        }
        Line::Note(note) => {
            canvas.write(left, pen, note, 1, LABEL);
        }
        Line::Rule => canvas.block(left, pen + 3, right - left, 1, EDGE),
        Line::Heading(name) => {
            canvas.write(left, pen, name, 1, HEADING);
        }
        Line::Row(label, value) => row(canvas, left, right, pen, label, value),
        Line::Legend(slot, count) => legend_row(canvas, left, right, pen, *slot, *count),
        Line::Ground(kind, count) => ground_row(canvas, left, right, pen, *kind, *count),
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
    canvas.write(left, pen, label, 1, LABEL);

    let column = left + VALUE_COLUMN;
    let cells = ((right - column) / text::GLYPH_WIDTH).max(0) as usize;
    let value: String = value.chars().take(cells).collect();

    let start = column.max(right - text::width_of(&value, 1));
    canvas.write(start, pen, &value, 1, VALUE);
}

/// Draws one legend row: a colour swatch, the faction, and its count.
///
/// The row names a colour, not a faction identity beyond the colour table. A
/// world with more factions than colours reuses a colour.
fn legend_row(canvas: &mut Canvas, left: i32, right: i32, pen: i32, slot: usize, count: u32) {
    let colour = faction_colour(cachette_core::FactionId(slot as u16));
    canvas.block(left, pen, text::GLYPH_HEIGHT, text::GLYPH_HEIGHT, colour);
    canvas.write(left + 14, pen, &format!("faction {slot}"), 1, LABEL);
    let value = grouped(u64::from(count));
    canvas.write(right - text::width_of(&value, 1), pen, &value, 1, VALUE);
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
    canvas.block(
        left,
        pen,
        text::GLYPH_HEIGHT,
        text::GLYPH_HEIGHT,
        kind_colour(kind),
    );
    canvas.write(left + 14, pen, name_of(kind), 1, LABEL);
    let value = grouped(u64::from(count));
    canvas.write(right - text::width_of(&value, 1), pen, &value, 1, VALUE);
}

/// Returns the name the panel gives one kind of ground.
///
/// The product record asks that the kinds be few and that a person be able to
/// name them.[^1] This is where they are named.
///
/// # References
///
/// [^1]: PRD-0003, a developer sees a world worth looking at. `docs/product/accepted/prd-0003-a-developer-sees-a-world-worth-looking-at.md`
const fn name_of(kind: TileKind) -> &'static str {
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
fn grouped(value: u64) -> String {
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
