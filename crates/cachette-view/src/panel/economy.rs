//! The panel that shows what each settlement holds, makes and owes.
//!
//! A watcher who looks at a settlement mark cannot see its store fall. The
//! store feeds the units of the site, and a unit that is not fed ends. This
//! panel states the store, the production rate, the upkeep rate and the
//! housing of each settlement, so a site that starves is visible before its
//! units are gone.
//!
//! This file is the whole panel. It states its own name, its own title and
//! its own lines, and the standard draws it.[^1]
//!
//! # What the panel reads
//!
//! The panel reads each settlement through the same readers the Python
//! `site_economy` verb reads: the store, the production rate, the upkeep
//! rate, the address and the faction.[^2] It walks the settlement arena for a
//! fixed number of sites and stops, and it reads the arena length for the
//! count it did not show. The walk never follows the extent of the world or
//! the population.[^3] A settlement the panel has no room for is counted and
//! named as absent, never dropped in silence.[^4]
//!
//! When the caller sets a pointer, the panel reads the settlement on the
//! pointed tile, which is one array read, and it puts that settlement first.
//!
//! # References
//!
//! [^1]: The panel standard. `crates/cachette-view/src/panel/mod.rs`
//! [^2]: ADR-0067, the viewer reads the world and never writes to it, decision D1. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
//! [^3]: ADR-0070, the head-up display reports what the drawing pass read, decision D1. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
//! [^4]: ADR-0070, the head-up display reports what the drawing pass read, decision D2. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`

use cachette_core::{CommodityId, Entity, World};

use super::{Line, Panel, View};
use crate::hud::{fraction, grouped, seats_of};
use crate::paint::faction_colour;

/// How many settlements the panel shows.
///
/// The bound is the panel's own. The deck cuts a panel that runs past the
/// foot of the frame, and a site below the cut would be lost in silence. The
/// panel stops here and says how many more there are.
pub const SITE_ROWS: usize = 4;

/// What the panel says when no settlement stands.
pub const NO_SITE_NOTE: &str = "no settlement stands.";

/// The panel that shows what each settlement holds, makes and owes.
pub struct Economy;

impl Panel for Economy {
    fn name(&self) -> &'static str {
        "economy"
    }

    fn title(&self) -> &'static str {
        "ECONOMY"
    }

    fn lines(&self, view: &View<'_>) -> Vec<Line> {
        let world = view.world;
        let arena = world.settlements();

        let mut lines = vec![Line::row("settlements", grouped(u64::from(arena.len())))];
        if arena.is_empty() {
            lines.push(Line::note(NO_SITE_NOTE));
            return lines;
        }

        let pointed = view.pointer.and_then(|at| world.settlement_on(at));
        let mut shown = 0usize;
        for site in pointed
            .into_iter()
            .chain(arena.iter().filter(|site| Some(*site) != pointed))
            .take(SITE_ROWS)
        {
            shown += 1;
            site_lines(world, site, &mut lines);
        }

        let more = usize::try_from(arena.len())
            .unwrap_or(usize::MAX)
            .saturating_sub(shown);
        if more > 0 {
            lines.push(Line::Rule);
            lines.push(Line::note(format!("and {} more.", grouped(more as u64))));
        }
        lines
    }
}

/// Appends the lines of one settlement.
///
/// The engine holds one commodity, and it is the one the cohorts draw. The
/// identifier is the engine's, not a second name for it here.
fn site_lines(world: &World, site: Entity, lines: &mut Vec<Line>) {
    let arena = world.settlements();
    let commodity = CommodityId(0);
    let (Some(place), Some(faction)) = (arena.address(site), arena.faction(site)) else {
        return;
    };
    lines.push(Line::Rule);
    lines.push(Line::swatch(
        faction_colour(faction),
        format!("q {}  r {}", place.q, place.r),
        format!("faction {}", faction.0),
    ));
    let store = arena.store(site).and_then(|held| held.quantity(commodity));
    lines.push(Line::row("store", fraction(store)));
    lines.push(Line::row(
        "production",
        fraction(world.production_rate(site, commodity)),
    ));
    lines.push(Line::row(
        "upkeep",
        fraction(world.upkeep_rate(site, commodity)),
    ));
    let (seats, held) = seats_of(world, site);
    lines.push(Line::row(
        "housed",
        format!(
            "{} of {}",
            grouped(u64::from(held)),
            grouped(u64::from(seats))
        ),
    ));
}

#[cfg(test)]
mod tests {
    use cachette_core::{Axial, FactionId, Fix32, World, WorldConfig};

    use super::*;
    use crate::paint::Camera;
    use crate::panel::{lines_that_do_not_fit, says, Set};

    const EXTENT: u32 = 64;

    fn world() -> World {
        World::new(WorldConfig {
            width: EXTENT,
            height: EXTENT,
            seed: 0x0123_4567_89ab_cdef,
            faction_count: 3,
            unit_capacity: 4096,
        })
        .expect("a small extent describes a world")
    }

    fn view<'a>(world: &'a World, pointer: Option<Axial>) -> View<'a> {
        View {
            world,
            camera: Camera {
                tile_width: 1.0,
                tile_height: 1.0,
                origin_x: 0.0,
                origin_y: 0.0,
            },
            frame_width: 800,
            frame_height: 600,
            focus: None,
            pointer,
        }
    }

    /// Founds one settlement of a faction on the first tile of a row that
    /// admits one, and returns the site and its address.
    fn found(world: &mut World, faction: FactionId, row: i32) -> (Entity, Axial) {
        for column in 0..EXTENT as i32 {
            let at = Axial::new(column, row);
            if let Ok(site) = world.found_settlement(at, faction) {
                return (site, at);
            }
        }
        panic!("a row of the fixture admits a settlement");
    }

    fn economy_says(world: &World, pointer: Option<Axial>) -> Vec<String> {
        says(&view(world, pointer), Set::EMPTY.with("economy").unwrap())
    }

    #[test]
    fn a_world_with_no_settlement_says_so() {
        let world = world();
        let said = economy_says(&world, None);
        assert!(said.iter().any(|line| line == "settlements: 0"), "{said:?}");
        assert!(said.iter().any(|line| line == NO_SITE_NOTE), "{said:?}");
    }

    #[test]
    fn a_settlement_names_its_store_its_rates_and_its_housing() {
        let mut world = world();
        let (site, at) = found(&mut world, FactionId(1), 10);
        world
            .set_settlement_store(site, CommodityId(0), Fix32(3 * 65_536 + 65_536 / 2))
            .expect("the commodity exists");
        world
            .set_production_rate(site, CommodityId(0), Fix32(2 * 65_536))
            .expect("the commodity exists");
        world
            .set_upkeep_rate(site, CommodityId(0), Fix32(65_536 / 4))
            .expect("the commodity exists");

        let said = economy_says(&world, None);
        let head = format!("q {}  r {}: faction 1", at.q, at.r);
        assert!(said.contains(&head), "{said:?}");
        assert!(said.iter().any(|line| line == "store: 3.50"), "{said:?}");
        assert!(
            said.iter().any(|line| line == "production: 2.00"),
            "{said:?}"
        );
        assert!(said.iter().any(|line| line == "upkeep: 0.25"), "{said:?}");
        assert!(
            said.iter().any(|line| line.starts_with("housed: ")),
            "{said:?}"
        );
        assert!(
            !said.iter().any(|line| line.starts_with("and ")),
            "{said:?}"
        );
    }

    #[test]
    fn a_pointer_puts_the_pointed_settlement_first() {
        let mut world = world();
        for row in 0..SITE_ROWS as i32 {
            found(&mut world, FactionId(0), row);
        }
        let (_, at) = found(&mut world, FactionId(2), 30);

        let said = economy_says(&world, Some(at));
        let first = said
            .iter()
            .find(|line| line.starts_with("q "))
            .expect("a settlement is named");
        assert_eq!(
            *first,
            format!("q {}  r {}: faction 2", at.q, at.r),
            "{said:?}"
        );
    }

    #[test]
    fn the_settlements_past_the_bound_are_counted() {
        let mut world = world();
        for row in 0..(SITE_ROWS as i32 + 3) {
            found(&mut world, FactionId(0), row);
        }
        let said = economy_says(&world, None);
        assert!(said.iter().any(|line| line == "and 3 more."), "{said:?}");
        assert_eq!(
            said.iter().filter(|line| line.starts_with("q ")).count(),
            SITE_ROWS,
            "{said:?}"
        );
    }

    /// A cut line states something other than what it was given, silently.
    /// The worst address is the corner of a world at the target scale, and
    /// the worst store is the ceiling of the fixed-point type.[^1]
    ///
    /// # References
    ///
    /// [^1]: Testing Rules, section 2a. `.claude/rules/testing.md`
    #[test]
    fn no_line_is_cut_at_the_worst_plausible_numbers() {
        let worst_fix = fraction(Some(Fix32(i32::MIN)));
        let lines = [
            Line::row("settlements", grouped(16_777_216)),
            Line::swatch(0x00ff_00ff, "q 4096  r 4096", "faction 62".to_string()),
            Line::row("store", worst_fix.clone()),
            Line::row("production", worst_fix.clone()),
            Line::row("upkeep", worst_fix),
            Line::row(
                "housed",
                format!("{} of {}", grouped(65_535), grouped(65_535)),
            ),
            Line::note(format!("and {} more.", grouped(16_777_216))),
        ];
        for line in &lines {
            assert!(!line.is_cut(), "line was cut: {line:?}");
        }
    }

    #[test]
    fn the_panel_itself_produces_no_cut_line() {
        let mut world = world();
        let (site, at) = found(&mut world, FactionId(0), 10);
        world
            .set_settlement_store(site, CommodityId(0), Fix32(i32::MIN))
            .expect("the commodity exists");
        let bad =
            lines_that_do_not_fit(&view(&world, Some(at)), Set::EMPTY.with("economy").unwrap());
        assert!(bad.is_empty(), "cut lines: {bad:?}");
    }
}
