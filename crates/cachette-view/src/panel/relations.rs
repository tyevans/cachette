//! The panel that shows what each faction feels toward each other faction.
//!
//! A watcher who looks at the map cannot see a relation. A declaration of war
//! changes no tile and no colour until the first meeting resolves, and a
//! peace changes none at all. This panel states the relation of every ordered
//! pair and the band it sits in, and it names the crossings of the war edge
//! on the last tick, so a war is visible the moment it is declared.
//!
//! This file is the whole panel. It states its own name, its own title and
//! its own lines, and the standard draws it.[^1]
//!
//! # What the panel reads
//!
//! The panel reads the relation of each ordered pair and its band number
//! through the same readers the Python `relation` verb reads.[^2] The band
//! number counts the edges at or below the value, and the world holds the
//! edges. **The panel copies no edge.** It reads the number the world
//! returns and gives it a word, so a world with other edges changes the word
//! with no edit here. The engine holds no band name, and this crate is the
//! presenter that holds them.[^3]
//!
//! The matrix holds one entry for each ordered pair, and the faction ceiling
//! bounds the pairs. The panel shows a fixed number of pairs and says how many
//! more there are. The crossings of the last tick come from the relation
//! log, which the step empties before it runs. The panel starts no pass over
//! a tile and no pass over a unit.[^4]
//!
//! When the caller sets a pointer, the panel reads the holder of the pointed
//! tile, which is one array read, and it puts the rows of that faction first.
//!
//! # References
//!
//! [^1]: The panel standard. `crates/cachette-view/src/panel/mod.rs`
//! [^2]: ADR-0067, the viewer reads the world and never writes to it, decision D1. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
//! [^3]: ADR-0146, a faction relation is one signed integer per ordered pair, and a pass reads a threshold, decision D2. `docs/adrs/draft/adr-0146-a-faction-relation-is-one-signed-integer-per-ordered-pair-and-a-pass-reads-a-threshold.md`
//! [^4]: ADR-0070, the head-up display reports what the drawing pass read, decision D1. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`

use cachette_core::{FactionId, RelationCrossed, World};

use super::market::factions_in_order;
use super::{Line, Panel, View};
use crate::hud::grouped;

/// How many ordered pairs the panel shows.
///
/// The bound is the panel's own. The deck cuts a panel that runs past the
/// foot of the frame, and a pair below the cut would be lost in silence. The
/// panel stops here and says how many more there are.
pub const PAIR_ROWS: usize = 12;

/// How many crossings of the last tick the panel shows.
pub const CROSSING_ROWS: usize = 6;

/// What the panel says when the world holds one faction.
pub const ONE_FACTION_NOTE: &str = "the world holds one faction.";

/// What the panel says when no pair crossed the war edge on the last tick.
pub const NO_CROSSING_NOTE: &str = "no crossing on the last tick.";

/// The words for the band numbers, from the war band upward.
///
/// The number is what the world returns. Zero is below the war edge, and
/// each edge at or below the value adds one. A number past the table is a
/// defect in the engine, and the panel shows it as one.
const BAND_WORDS: [&str; 4] = ["war", "tension", "peace", "alliance"];

/// The panel that shows what each faction feels toward each other faction.
pub struct Relations;

impl Panel for Relations {
    fn name(&self) -> &'static str {
        "relations"
    }

    fn title(&self) -> &'static str {
        "RELATIONS"
    }

    fn lines(&self, view: &View<'_>) -> Vec<Line> {
        let world = view.world;
        let factions = factions_in_order(world, view.pointer);

        let mut lines = vec![Line::heading("PAIRS")];
        if factions.len() < 2 {
            lines.push(Line::note(ONE_FACTION_NOTE));
        }
        let pairs = factions.len() * factions.len().saturating_sub(1);
        let mut shown = 0;
        'pairs: for from in &factions {
            for to in &factions {
                if from == to {
                    continue;
                }
                if shown == PAIR_ROWS {
                    break 'pairs;
                }
                lines.push(pair_row(world, *from, *to));
                shown += 1;
            }
        }
        if pairs > shown {
            lines.push(Line::note(format!(
                "and {} more.",
                grouped((pairs - shown) as u64)
            )));
        }

        lines.push(Line::Rule);
        lines.push(Line::heading("CROSSINGS"));
        let log = world.relation_log();
        if log.is_empty() {
            lines.push(Line::note(NO_CROSSING_NOTE));
        }
        // The log lies in the order the crossings happened, and the panel
        // shows the most recent first.
        for crossed in log.iter().rev().take(CROSSING_ROWS) {
            lines.push(Line::note(crossing_says(crossed)));
        }
        if log.len() > CROSSING_ROWS {
            lines.push(Line::note(format!(
                "and {} more.",
                grouped((log.len() - CROSSING_ROWS) as u64)
            )));
        }
        lines
    }
}

/// Returns the row for one ordered pair: the pair and its band as the label,
/// and the value against the right edge.
fn pair_row(world: &World, from: FactionId, to: FactionId) -> Line {
    let value = world
        .relation(from, to)
        .map_or_else(|| "-".to_string(), |value| value.to_string());
    let band = world.relation_band(from, to).map_or("-", band_word);
    Line::row(format!("{} -> {} {band}", from.0, to.0), value)
}

/// Returns the word for a band number the world returned.
fn band_word(band: u8) -> &'static str {
    BAND_WORDS.get(usize::from(band)).copied().unwrap_or("?")
}

/// Returns what one crossing of the war edge says.
fn crossing_says(crossed: &RelationCrossed) -> String {
    let from = crossed.from_faction.0;
    let to = crossed.to_faction.0;
    if crossed.is_declaration() {
        format!("{from} declared war on {to}")
    } else {
        format!("{from} made peace with {to}")
    }
}

#[cfg(test)]
mod tests {
    use cachette_core::types::FACTION_CEILING;
    use cachette_core::{Axial, FactionId, World, WorldConfig};

    use super::*;
    use crate::paint::Camera;
    use crate::panel::market::tests::a_held_tile;
    use crate::panel::{lines_that_do_not_fit, says, Set};

    const EXTENT: u32 = 64;

    fn world_of(faction_count: u16) -> World {
        World::new(WorldConfig {
            width: EXTENT,
            height: EXTENT,
            seed: 0x0123_4567_89ab_cdef,
            faction_count,
            unit_capacity: 4096,
        })
        .expect("a small extent describes a world")
    }

    fn world() -> World {
        world_of(3)
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

    fn relations_say(world: &World, pointer: Option<Axial>) -> Vec<String> {
        says(&view(world, pointer), Set::EMPTY.with("relations").unwrap())
    }

    #[test]
    fn a_fresh_world_is_at_peace_with_no_crossing() {
        let world = world();
        let said = relations_say(&world, None);
        // Three factions make six ordered pairs, and every one is at the
        // peace edge with no crossing behind it.
        assert!(
            said.iter().any(|line| line == "0 -> 1 peace: 0"),
            "{said:?}"
        );
        assert!(
            said.iter().any(|line| line == "2 -> 1 peace: 0"),
            "{said:?}"
        );
        assert_eq!(
            said.iter().filter(|line| line.contains(" -> ")).count(),
            6,
            "{said:?}"
        );
        assert!(said.iter().any(|line| line == NO_CROSSING_NOTE), "{said:?}");
        assert!(
            !said.iter().any(|line| line == ONE_FACTION_NOTE),
            "{said:?}"
        );
        assert!(
            !said.iter().any(|line| line.starts_with("and ")),
            "{said:?}"
        );
    }

    #[test]
    fn one_faction_has_no_pair_and_says_so() {
        let world = world_of(1);
        let said = relations_say(&world, None);
        assert!(said.iter().any(|line| line == ONE_FACTION_NOTE), "{said:?}");
        assert!(!said.iter().any(|line| line.contains(" -> ")), "{said:?}");
    }

    /// The word follows the edges the world holds and not a copy. The test
    /// moves every edge and checks that a value at each new edge takes the
    /// word of the band above it.
    #[test]
    fn the_band_word_follows_the_edges_the_world_holds() {
        let mut world = world();
        let mut rules = world.relation_rules();
        rules.war_edge = -100;
        rules.peace_edge = 50;
        rules.alliance_edge = 300;
        world.set_relation_rules(rules);

        assert!(world.set_relation(FactionId(0), FactionId(1), -101));
        assert!(world.set_relation(FactionId(0), FactionId(2), -100));
        assert!(world.set_relation(FactionId(1), FactionId(0), 50));
        assert!(world.set_relation(FactionId(1), FactionId(2), 300));
        let said = relations_say(&world, None);
        assert!(
            said.iter().any(|line| line == "0 -> 1 war: -101"),
            "{said:?}"
        );
        assert!(
            said.iter().any(|line| line == "0 -> 2 tension: -100"),
            "{said:?}"
        );
        assert!(
            said.iter().any(|line| line == "1 -> 0 peace: 50"),
            "{said:?}"
        );
        assert!(
            said.iter().any(|line| line == "1 -> 2 alliance: 300"),
            "{said:?}"
        );
    }

    #[test]
    fn a_crossing_of_the_war_edge_is_named_most_recent_first() {
        let mut world = world();
        let war = world.relation_rules().war_edge;
        assert!(world.set_relation(FactionId(2), FactionId(0), war - 1));
        let said = relations_say(&world, None);
        assert!(
            said.iter().any(|line| line == "2 declared war on 0"),
            "{said:?}"
        );
        assert!(
            !said.iter().any(|line| line == NO_CROSSING_NOTE),
            "{said:?}"
        );

        assert!(world.set_relation(FactionId(2), FactionId(0), war));
        let said = relations_say(&world, None);
        let heading = said.iter().position(|line| line == "CROSSINGS").unwrap();
        assert_eq!(said[heading + 1], "2 made peace with 0", "{said:?}");
        assert_eq!(said[heading + 2], "2 declared war on 0", "{said:?}");
    }

    #[test]
    fn the_step_empties_the_crossings() {
        let mut world = world();
        let war = world.relation_rules().war_edge;
        assert!(world.set_relation(FactionId(0), FactionId(1), war - 1));
        world.step(1).expect("the step must run");
        let said = relations_say(&world, None);
        assert!(said.iter().any(|line| line == NO_CROSSING_NOTE), "{said:?}");
        // The drift may have moved the entry on the tick, so the row is
        // checked against what the world holds after the step.
        let value = world.relation(FactionId(0), FactionId(1)).unwrap();
        let band = band_word(world.relation_band(FactionId(0), FactionId(1)).unwrap());
        let row = format!("0 -> 1 {band}: {value}");
        assert!(said.contains(&row), "{said:?}");
    }

    #[test]
    fn a_pointer_puts_the_pointed_faction_first() {
        let mut world = world();
        let held = a_held_tile(&mut world, FactionId(2));
        let said = relations_say(&world, Some(held));
        let first = said
            .iter()
            .find(|line| line.contains(" -> "))
            .expect("a pair is named");
        assert!(first.starts_with("2 -> "), "{said:?}");
    }

    #[test]
    fn the_pairs_stop_at_the_bound_and_count_the_rest() {
        let world = world_of(8);
        let said = relations_say(&world, None);
        assert_eq!(
            said.iter().filter(|line| line.contains(" -> ")).count(),
            PAIR_ROWS,
            "{said:?}"
        );
        let more = format!("and {} more.", grouped((8 * 7 - PAIR_ROWS) as u64));
        assert!(said.contains(&more), "{said:?}");
    }

    /// A cut line states something other than what it was given, silently.
    /// The worst pair is two factions at the ceiling, and the worst value is
    /// either end of the integer range with the longest band word.[^1]
    ///
    /// # References
    ///
    /// [^1]: Testing Rules, section 2a. `.claude/rules/testing.md`
    #[test]
    fn no_line_is_cut_at_the_worst_plausible_numbers() {
        let last = FACTION_CEILING - 1;
        let before = FACTION_CEILING - 2;
        let lines = [
            Line::row(format!("{last} -> {before} alliance"), i32::MIN.to_string()),
            Line::row(format!("{last} -> {before} tension"), i32::MAX.to_string()),
            Line::note(format!("{last} declared war on {before}")),
            Line::note(format!("{last} made peace with {before}")),
            Line::note(format!(
                "and {} more.",
                grouped(u64::from(FACTION_CEILING) * u64::from(FACTION_CEILING - 1))
            )),
        ];
        for line in &lines {
            assert!(!line.is_cut(), "line was cut: {line:?}");
        }
    }

    #[test]
    fn the_panel_itself_produces_no_cut_line() {
        let mut world = world();
        let held = a_held_tile(&mut world, FactionId(0));
        assert!(world.set_relation(FactionId(0), FactionId(1), i32::MIN));
        let bad = lines_that_do_not_fit(
            &view(&world, Some(held)),
            Set::EMPTY.with("relations").unwrap(),
        );
        assert!(bad.is_empty(), "cut lines: {bad:?}");
    }
}
