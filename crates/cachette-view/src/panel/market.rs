//! The panel that shows what each faction offers and wants.
//!
//! A watcher who looks at the map cannot see a board. A faction that
//! advertises stone for food changes no tile and no colour, and the
//! negotiation that follows changes none either. This panel states the
//! board of every faction and the count of its live negotiations, so a trade
//! is visible before it moves a single unit.
//!
//! This file is the whole panel. It states its own name, its own title and
//! its own lines, and the standard draws it.[^1]
//!
//! # What the panel reads
//!
//! The panel reads the board of each faction through the same reader the
//! Python `market` verb reads, and it reads the negotiation plane through
//! the same reader the Python `trade_book` verb reads.[^2] A board holds a
//! fixed number of rows, and the plane holds one row for each ordered pair
//! of factions. The world holds at most sixty-three factions, so both walks
//! cost the same at any population and at any world size. The panel starts
//! no pass over a tile and no pass over a unit.[^3]
//!
//! When the caller sets a pointer, the panel reads the holder of the pointed
//! tile, which is one array read, and it puts that faction first.
//!
//! # References
//!
//! [^1]: The panel standard. `crates/cachette-view/src/panel/mod.rs`
//! [^2]: ADR-0067, the viewer reads the world and never writes to it, decision D1. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
//! [^3]: ADR-0070, the head-up display reports what the drawing pass read, decision D1. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`

use cachette_core::{FactionId, Holder, ResourceKind, World};

use super::{Line, Panel, View};
use crate::hud::grouped;
use crate::paint::faction_colour;

/// What the panel says when no faction has advertised.
///
/// The market table allocates nothing until the first write, so every board
/// is empty. The note says why, so a reader does not take a quiet market for
/// a broken pass.
pub const QUIET_NOTE: &str = "no faction has advertised.";

/// What the panel says of a faction whose board is empty.
const EMPTY_BOARD: &str = "the board is empty.";

/// The panel that shows what each faction offers and wants.
pub struct Market;

impl Panel for Market {
    fn name(&self) -> &'static str {
        "market"
    }

    fn title(&self) -> &'static str {
        "MARKET"
    }

    fn lines(&self, view: &View<'_>) -> Vec<Line> {
        let world = view.world;
        let factions = factions_in_order(world, view.pointer);

        let mut lines = Vec::new();
        if factions
            .iter()
            .all(|faction| world.market(*faction).is_empty())
        {
            lines.push(Line::note(QUIET_NOTE));
        }

        for faction in factions {
            lines.push(Line::Rule);
            lines.push(Line::swatch(
                faction_colour(faction),
                format!("faction {}", faction.0),
                String::new(),
            ));
            let board = world.market(faction);
            let mut advertised = false;
            for advert in board.iter().filter(|advert| !advert.is_empty()) {
                advertised = true;
                let verb = if advert.wants == 0 { "offers" } else { "wants" };
                lines.push(Line::row(
                    format!("{verb} {}", good(advert.good)),
                    grouped(u64::from(advert.quantity)),
                ));
                lines.push(Line::row(
                    format!("for {}", good(advert.asking_good)),
                    grouped(u64::from(advert.asking_quantity)),
                ));
            }
            if !advertised {
                lines.push(Line::note(EMPTY_BOARD));
            }
            lines.push(Line::row(
                "open contracts",
                grouped(open_contracts(world, faction)),
            ));
        }

        lines
    }
}

/// Returns every faction of the world, with the pointed faction first.
///
/// The rest keep their number order, so the list is stable between two
/// frames that point at the same tile.
pub(crate) fn factions_in_order(
    world: &World,
    pointer: Option<cachette_core::Axial>,
) -> Vec<FactionId> {
    let count = world.config().faction_count.max(1);
    let pointed = pointer
        .and_then(|at| world.tile_holder(at))
        .and_then(Holder::faction)
        .filter(|faction| faction.0 < count);
    let mut factions = Vec::with_capacity(usize::from(count));
    if let Some(first) = pointed {
        factions.push(first);
    }
    factions.extend(
        (0..count)
            .map(FactionId)
            .filter(|faction| Some(*faction) != pointed),
    );
    factions
}

/// Returns how many live negotiations and contracts name one faction.
///
/// The book holds one row for each ordered pair, at the index of the
/// proposer times the faction count plus the responder. A row counts when it
/// is live and the faction sits on either side of it.
fn open_contracts(world: &World, faction: FactionId) -> u64 {
    let count = usize::from(world.config().faction_count.max(1));
    world
        .trade_book()
        .iter()
        .enumerate()
        .filter(|(index, row)| {
            row.is_live()
                && (index / count == usize::from(faction.0)
                    || index % count == usize::from(faction.0))
        })
        .count() as u64
}

/// Returns the name of a resource kind, or its number when it names none.
fn good(kind: u8) -> String {
    match ResourceKind::from_u8(kind) {
        Some(ResourceKind::Food) => "food".to_string(),
        Some(ResourceKind::Wood) => "wood".to_string(),
        Some(ResourceKind::Stone) => "stone".to_string(),
        None => format!("kind {kind}"),
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use cachette_core::types::FACTION_CEILING;
    use cachette_core::{Advert, Axial, FactionId, World, WorldConfig};

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

    /// Fills a band of the world with one faction, steps until the faction
    /// holds ground, and returns one tile it holds that admits a unit.
    pub(crate) fn a_held_tile(world: &mut World, faction: FactionId) -> Axial {
        for row in 20..40 {
            for column in 20..40 {
                let at = Axial::new(column, row);
                if world.admits_a_unit(at) {
                    world
                        .spawn_soldier(at, faction)
                        .expect("the address and the faction are valid");
                }
            }
        }
        for _ in 0..4 {
            world.step(1).expect("the step must run");
        }
        let grid = world.grid();
        world
            .holding()
            .tiles_held_by(faction)
            .map(|held| {
                Axial::new(
                    (held.0 % grid.width()) as i32,
                    (held.0 / grid.width()) as i32,
                )
            })
            .find(|at| world.admits_a_unit(*at))
            .expect("the faction holds a tile with room after four ticks")
    }

    fn market_says(world: &World, pointer: Option<Axial>) -> Vec<String> {
        says(&view(world, pointer), Set::EMPTY.with("market").unwrap())
    }

    #[test]
    fn a_quiet_market_says_so() {
        let world = world();
        let said = market_says(&world, None);
        assert!(said.iter().any(|line| line == QUIET_NOTE), "{said:?}");
        assert!(said.iter().any(|line| line == "faction 2: "), "{said:?}");
        assert!(
            said.iter()
                .filter(|line| *line == "open contracts: 0")
                .count()
                == 3,
            "{said:?}"
        );
    }

    #[test]
    fn an_advert_names_the_good_the_quantity_and_the_asking_price() {
        let mut world = world();
        world
            .advertise(
                FactionId(1),
                &[Advert::new(2, 1_500, 0, 0, 40), Advert::new(0, 7, 1, 1, 3)],
            )
            .expect("the board takes two rows");
        let said = market_says(&world, None);
        assert!(!said.iter().any(|line| line == QUIET_NOTE), "{said:?}");
        assert!(
            said.iter().any(|line| line == "offers stone: 1 500"),
            "{said:?}"
        );
        assert!(said.iter().any(|line| line == "for food: 40"), "{said:?}");
        assert!(said.iter().any(|line| line == "wants food: 7"), "{said:?}");
        assert!(said.iter().any(|line| line == "for wood: 3"), "{said:?}");
    }

    #[test]
    fn an_offer_counts_as_an_open_contract_for_both_parties() {
        let mut world = world();
        let held = a_held_tile(&mut world, FactionId(0));
        world
            .spawn_soldier(held, FactionId(1))
            .expect("the held tile admits a unit");
        assert!(world.stands_in_territory_of(FactionId(1), FactionId(0)));
        world
            .offer_trade(FactionId(1), FactionId(0), 0, 5, 2, 5, 10)
            .expect("a faction with presence may offer");

        let said = market_says(&world, None);
        assert_eq!(
            said.iter()
                .filter(|line| *line == "open contracts: 1")
                .count(),
            2,
            "{said:?}"
        );
        assert!(
            said.iter().any(|line| line == "open contracts: 0"),
            "{said:?}"
        );
    }

    #[test]
    fn a_pointer_puts_the_pointed_faction_first() {
        let mut world = world();
        let held = a_held_tile(&mut world, FactionId(2));
        let said = market_says(&world, Some(held));
        let first = said
            .iter()
            .find(|line| line.starts_with("faction "))
            .expect("a faction is named");
        assert_eq!(first, "faction 2: ", "{said:?}");
        assert_eq!(
            said.iter()
                .filter(|line| line.starts_with("faction "))
                .count(),
            3,
            "{said:?}"
        );
    }

    /// A cut line states something other than what it was given, silently.
    /// The worst quantity a row can hold is the ceiling of its field, and the
    /// worst contract count is every ordered pair of the faction ceiling.[^1]
    ///
    /// # References
    ///
    /// [^1]: Testing Rules, section 2a. `.claude/rules/testing.md`
    #[test]
    fn no_line_is_cut_at_the_worst_plausible_numbers() {
        let worst_quantity = grouped(u64::from(u32::MAX));
        let pairs = u64::from(FACTION_CEILING) * u64::from(FACTION_CEILING - 1);
        let lines = [
            Line::swatch(0x00ff_00ff, "faction 62", String::new()),
            Line::row("offers stone", worst_quantity.clone()),
            Line::row("wants stone", worst_quantity.clone()),
            Line::row("for stone", worst_quantity.clone()),
            Line::row("for kind 255", worst_quantity),
            Line::row("open contracts", grouped(pairs)),
        ];
        for line in &lines {
            assert!(!line.is_cut(), "line was cut: {line:?}");
        }
    }

    #[test]
    fn the_panel_itself_produces_no_cut_line() {
        let mut world = world();
        world
            .advertise(FactionId(0), &[Advert::new(1, u32::MAX, 0, 2, u32::MAX)])
            .expect("the board takes one row");
        let bad = lines_that_do_not_fit(&view(&world, None), Set::EMPTY.with("market").unwrap());
        assert!(bad.is_empty(), "cut lines: {bad:?}");
    }
}
