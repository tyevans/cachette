//! The panel that shows what the last tick logged.
//!
//! A person who watches coloured cells move can see that the world runs.
//! They cannot see which unit ended, which site fell short, or which unit
//! earned a promotion. This panel names the last few of each, newest
//! entries first, so a watcher sees what happened rather than a total that
//! went up.[^1]
//!
//! # Which logs this panel reads
//!
//! The engine keeps six logs of the step that just ran. This panel reads
//! four: the units a shortage ended, the sites a draw could not serve in
//! full, the sites that could not pay their upkeep, and the units a step
//! promoted. Each names a discrete happening a watcher can read as a line.
//!
//! The panel does not read the gather log or the tile event log. Both hold
//! one entry for almost every unit or every tile that changed in a tick, at
//! the target scale of the world.[^2] A feed built from either would show
//! the ordinary churn of the simulation on every frame, not a happening
//! worth a watcher's attention. A section that always has entries to show
//! is a total in a different shape, and this panel already refuses that
//! shape for the sections it does show.[^3]
//!
//! # The bound on each section
//!
//! The panel reads a fixed number of entries from each log, so its cost
//! never follows the extent of the world or the population.[^4] It states
//! how many the log held beside the rows it shows, so a reader knows the
//! list is the newest few and not all of them.
//!
//! # References
//!
//! [^1]: The panel standard. `crates/cachette-view/src/panel/mod.rs`
//! [^2]: Project orientation, the target scale. `CLAUDE.md`
//! [^3]: ADR-0070, the head-up display reports what the drawing pass read, decision D2. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
//! [^4]: ADR-0070, the head-up display reports what the drawing pass read, decision D1. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`

use cachette_core::promotion::UnitPromoted;
use cachette_core::{SiteRationed, SiteShortfall, UnitStarved};

use crate::hud::{accumulated, fraction, grouped};

use super::{Line, Panel, View};

/// The number of entries the panel reads from each log.
///
/// This is a bound on the rows a section draws, not a budget for how many
/// entries the log may hold. The log itself may hold one entry for every
/// unit or every site the step touched, and the panel reads this many of
/// them and no more, so its cost does not follow the world.[^1]
///
/// # References
///
/// [^1]: ADR-0070, the head-up display reports what the drawing pass read, decision D1. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
const EVENT_ROWS: usize = 3;

/// Returns the index part of an entity identity that a log stored in bits.
///
/// A log holds the identity as one integer, because the type that decodes
/// it is private to the core crate.[^1] The index is the low half of that
/// integer, and it is the part a watcher reads as the unit or the site
/// number.
///
/// # References
///
/// [^1]: ADR-0085, an entity crosses to Python as one opaque identity that the engine resolves, decision D2. `docs/adrs/accepted/adr-0085-an-entity-crosses-to-python-as-one-opaque-identity.md`
fn index_of(bits: u64) -> u32 {
    bits as u32
}

/// The panel that shows what the last tick logged.
pub struct Events;

impl Panel for Events {
    fn name(&self) -> &'static str {
        "events"
    }

    fn title(&self) -> &'static str {
        "WHAT HAPPENED"
    }

    fn lines(&self, view: &View<'_>) -> Vec<Line> {
        let starved = view.world.starved_log();
        let rationed = view.world.rationed_log();
        let shortfall = view.world.shortfall_log();
        let promoted = view.world.promoted_log();

        let mut lines = Vec::new();
        lines.extend(starved_section(starved));
        lines.extend(rationed_section(rationed));
        lines.extend(shortfall_section(shortfall));
        lines.extend(promoted_section(promoted));

        // A number the panel cannot compute is absent. An empty tick is a
        // fact too, and it is stated rather than left as four missing
        // headings a reader cannot tell from a panel that failed to
        // draw.[^1]
        //
        // [^1]: ADR-0070, the head-up display reports what the drawing pass read, decision D2. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
        if lines.is_empty() {
            lines.push(Line::note("nothing happened this tick"));
        }
        lines
    }
}

/// Returns the lines that report the units a shortage ended, or nothing.
///
/// The section appears only when the log holds an entry. A row for a
/// permanent zero would take a row from every panel of every run to say
/// nothing, and it would go on saying nothing if the pass behind it
/// broke.[^1]
///
/// # References
///
/// [^1]: Testing Rules, section 2a. `.claude/rules/testing.md`
fn starved_section(log: &[UnitStarved]) -> Vec<Line> {
    if log.is_empty() {
        return Vec::new();
    }
    let mut lines = vec![
        Line::heading("UNITS ENDED"),
        Line::row("ended this tick", grouped(log.len() as u64)),
    ];
    for entry in log.iter().take(EVENT_ROWS) {
        lines.push(Line::row(
            format!("unit {}", index_of(entry.unit)),
            format!("deficit {}", fraction(Some(entry.deficit))),
        ));
    }
    lines
}

/// Returns the lines that report the sites a draw could not serve in full,
/// or nothing.
fn rationed_section(log: &[SiteRationed]) -> Vec<Line> {
    if log.is_empty() {
        return Vec::new();
    }
    let mut lines = vec![
        Line::heading("SITES RATIONED"),
        Line::row("rationed this tick", grouped(log.len() as u64)),
    ];
    for entry in log.iter().take(EVENT_ROWS) {
        lines.push(Line::row(
            format!("site {}", index_of(entry.site)),
            format!(
                "{} of {}",
                accumulated(entry.granted.0),
                accumulated(entry.demanded.0)
            ),
        ));
    }
    lines
}

/// Returns the lines that report the sites that could not pay their upkeep,
/// or nothing.
fn shortfall_section(log: &[SiteShortfall]) -> Vec<Line> {
    if log.is_empty() {
        return Vec::new();
    }
    let mut lines = vec![
        Line::heading("UPKEEP SHORT"),
        Line::row("short this tick", grouped(log.len() as u64)),
    ];
    for entry in log.iter().take(EVENT_ROWS) {
        lines.push(Line::row(
            format!("site {}", index_of(entry.site)),
            format!("owed {}", fraction(Some(entry.amount))),
        ));
    }
    lines
}

/// Returns the lines that report the units a step promoted, or nothing.
fn promoted_section(log: &[UnitPromoted]) -> Vec<Line> {
    if log.is_empty() {
        return Vec::new();
    }
    let mut lines = vec![
        Line::heading("PROMOTED"),
        Line::row("promoted this tick", grouped(log.len() as u64)),
    ];
    for entry in log.iter().take(EVENT_ROWS) {
        lines.push(Line::row(
            format!("unit {}", index_of(entry.unit)),
            format!("deeds {}", grouped(entry.deeds)),
        ));
    }
    lines
}

#[cfg(test)]
mod tests {
    use cachette_core::cohort::{NeedRule, NEED_FULL};
    use cachette_core::site::CommodityId;
    use cachette_core::{Axial, Fix32, World, WorldConfig};

    use crate::paint::Camera;
    use crate::panel::{self, Line as PanelLine, Panel, View};

    use super::Events;

    const FOOD: CommodityId = CommodityId(0);

    const CONFIG: WorldConfig = WorldConfig {
        width: 24,
        height: 24,
        seed: 7,
        faction_count: 2,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    };

    /// Returns the open ground of a world, in tile order.
    fn open_ground(world: &World) -> Vec<Axial> {
        let grid = world.grid();
        (0..grid.tile_count())
            .map(|index| Axial::new((index % grid.width()) as i32, (index / grid.width()) as i32))
            .filter(|address| world.admits_a_unit(*address))
            .collect()
    }

    /// Builds the view a panel reads.
    ///
    /// The panel reads no field of the camera or the frame size, so the
    /// values here are arbitrary and not drawn.
    fn view(world: &World) -> View<'_> {
        View {
            world,
            camera: Camera {
                tile_width: 8.0,
                tile_height: 8.0,
                origin_x: 0.0,
                origin_y: 0.0,
            },
            frame_width: 64,
            frame_height: 64,
            focus: None,
            pointer: None,
        }
    }

    #[test]
    fn a_tick_that_ends_nobody_says_so_and_states_no_false_zero() {
        let mut world = World::new(CONFIG).expect("the extent must describe a world");
        world.step(1).expect("the step must run");
        let lines = Events.lines(&view(&world));
        assert_eq!(
            lines,
            vec![PanelLine::note("nothing happened this tick")],
            "an empty tick must say so, and say nothing else"
        );
        assert!(!lines.iter().any(PanelLine::is_cut));
    }

    #[test]
    fn a_shortage_that_ends_a_unit_shows_the_row() {
        let mut world = World::new(CONFIG).expect("the extent must describe a world");
        world
            .set_economy_schedule(2, 0)
            .expect("the period is inside the range");
        let rule = NeedRule::DEFAULT;
        world.set_need_rule(
            NeedRule::new(
                rule.decay(),
                rule.ration(),
                rule.threshold(),
                rule.recovery(),
                NEED_FULL,
            )
            .expect("every rate is at or above zero"),
        );
        let ground = open_ground(&world);
        let site = world
            .found_settlement(ground[0], cachette_core::FactionId(0))
            .expect("the tile is free");
        let unit = world
            .spawn_soldier(ground[1], cachette_core::FactionId(0))
            .expect("the ground admits a unit");
        assert!(world.set_home_site(unit, Some(site)));
        world
            .set_settlement_store(site, FOOD, Fix32(NEED_FULL.0 / 2))
            .expect("the commodity is in the set");

        let mut said = None;
        for _ in 0..48 {
            world.step(1).expect("the step must run");
            if !world.starved_log().is_empty() {
                said = Some(panel::says(
                    &View {
                        world: &world,
                        ..view(&world)
                    },
                    panel::Set::EMPTY
                        .with("events")
                        .expect("events is registered"),
                ));
                break;
            }
        }
        let said = said.expect("the shortage must end the unit inside the run");
        assert!(said.iter().any(|line| line.starts_with("ended this tick")));
        assert!(said.iter().any(|line| line.contains("deficit")));
    }

    #[test]
    fn the_row_count_stays_bounded_when_the_log_is_long() {
        let mut world = World::new(CONFIG).expect("the extent must describe a world");
        world
            .set_economy_schedule(1, 0)
            .expect("the period is inside the range");
        let rule = NeedRule::DEFAULT;
        world.set_need_rule(
            NeedRule::new(
                rule.decay(),
                rule.ration(),
                rule.threshold(),
                rule.recovery(),
                NEED_FULL,
            )
            .expect("every rate is at or above zero"),
        );
        let ground = open_ground(&world);
        let mut units = Vec::new();
        for index in 0..(super::EVENT_ROWS * 4) {
            let site = world
                .found_settlement(ground[index * 2], cachette_core::FactionId(0))
                .expect("the tile is free");
            let unit = world
                .spawn_soldier(ground[index * 2 + 1], cachette_core::FactionId(0))
                .expect("the ground admits a unit");
            assert!(world.set_home_site(unit, Some(site)));
            world
                .set_settlement_store(site, FOOD, Fix32(NEED_FULL.0 / 2))
                .expect("the commodity is in the set");
            units.push(unit);
        }

        let mut rows = None;
        for _ in 0..48 {
            world.step(1).expect("the step must run");
            if world.starved_log().len() > super::EVENT_ROWS {
                rows = Some(Events.lines(&view(&world)));
                break;
            }
        }
        let rows = rows.expect("a tick with more starved units than the bound must arrive");
        let unit_rows = rows
            .iter()
            .filter(|line| matches!(line, PanelLine::Row(label, _) if label.starts_with("unit ")))
            .count();
        assert_eq!(unit_rows, super::EVENT_ROWS);
    }

    #[test]
    fn no_line_is_cut_at_the_widest_plausible_number() {
        // The widest number a section prints is a large accumulator, an
        // index near the top of a 32-bit range, and a large deed count. A
        // format that only looked right on small numbers would still be a
        // defect.
        let wide_index = u32::MAX as u64;
        let starved = starved_line(wide_index, Fix32(i32::MAX));
        assert!(!starved.is_cut());
        let rationed = rationed_line(wide_index, 1 << 40, (1 << 40) - 1);
        assert!(!rationed.is_cut());
        let shortfall = shortfall_line(wide_index, Fix32(i32::MAX));
        assert!(!shortfall.is_cut());
        let promoted = promoted_line(wide_index, u64::MAX / 2);
        assert!(!promoted.is_cut());
    }

    fn starved_line(unit: u64, deficit: Fix32) -> PanelLine {
        PanelLine::row(
            format!("unit {}", super::index_of(unit)),
            format!("deficit {}", crate::hud::fraction(Some(deficit))),
        )
    }

    fn rationed_line(site: u64, demanded: i64, granted: i64) -> PanelLine {
        PanelLine::row(
            format!("site {}", super::index_of(site)),
            format!(
                "{} of {}",
                crate::hud::accumulated(granted),
                crate::hud::accumulated(demanded)
            ),
        )
    }

    fn shortfall_line(site: u64, amount: Fix32) -> PanelLine {
        PanelLine::row(
            format!("site {}", super::index_of(site)),
            format!("owed {}", crate::hud::fraction(Some(amount))),
        )
    }

    fn promoted_line(unit: u64, deeds: u64) -> PanelLine {
        PanelLine::row(
            format!("unit {}", super::index_of(unit)),
            format!("deeds {}", crate::hud::grouped(deeds)),
        )
    }
}
