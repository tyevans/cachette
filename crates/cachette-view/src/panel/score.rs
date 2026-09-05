//! The panel that shows the running score and whether the game has ended.
//!
//! A watcher who looks at coloured ground cannot tell who leads, and cannot
//! tell that the game is over. The engine records a game end once, and the
//! picture changes nothing when it does. This panel states the territory
//! score of every faction, the four weights that bias its choices, the tick
//! limit and the ticks that remain, and the winner when there is one.
//!
//! This file is the whole panel. It states its own name, its own title and
//! its own lines, and the standard draws it.[^1]
//!
//! # What the panel reads
//!
//! The panel reads the score of each faction through the same reader the
//! Python `score` verb reads, which is the running total of held tiles.[^2]
//! It reads the weights, the tick limit and the game end record through the
//! readers of the same names. Every one of them is a stored field or a
//! bounded loop over the factions. The world holds at most sixty-three
//! factions, so the panel costs the same at any population and at any world
//! size. It starts no pass over a tile and no pass over a unit.[^3]
//!
//! When the caller sets a pointer, the panel reads the holder of the pointed
//! tile, which is one array read, and it puts that faction first.
//!
//! # References
//!
//! [^1]: The panel standard. `crates/cachette-view/src/panel/mod.rs`
//! [^2]: ADR-0067, the viewer reads the world and never writes to it, decision D1. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
//! [^3]: ADR-0070, the head-up display reports what the drawing pass read, decision D1. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`

use super::market::factions_in_order;
use super::{Line, Panel, View};
use crate::hud::grouped;
use crate::paint::faction_colour;

/// What the panel says of a path number that names no path.
const NO_PATH: &str = "unknown path";

/// The panel that shows the running score and whether the game has ended.
pub struct Score;

impl Panel for Score {
    fn name(&self) -> &'static str {
        "score"
    }

    fn title(&self) -> &'static str {
        "SCORE"
    }

    fn lines(&self, view: &View<'_>) -> Vec<Line> {
        let world = view.world;
        let tick = world.tick().0;
        let limit = world.tick_limit();
        let end = world.game_end();

        let mut lines = vec![
            Line::heading("GAME"),
            Line::row("tick", grouped(tick)),
            Line::row("tick limit", grouped(limit)),
        ];
        if end.is_set() {
            lines.push(Line::row("ended", "yes"));
            lines.push(Line::row("winner", format!("faction {}", end.winner.0)));
            lines.push(Line::row(
                "path",
                end.win_path().map_or(NO_PATH, |path| path.name()),
            ));
            lines.push(Line::row("at tick", grouped(end.tick.0)));
        } else {
            lines.push(Line::row("ended", "no"));
            lines.push(Line::row(
                "ticks remaining",
                grouped(limit.saturating_sub(tick)),
            ));
        }

        for faction in factions_in_order(world, view.pointer) {
            lines.push(Line::Rule);
            let score = world
                .score(faction)
                .map_or_else(|| "-".to_string(), |held| grouped(held.max(0) as u64));
            lines.push(Line::swatch(
                faction_colour(faction),
                format!("faction {}", faction.0),
                score,
            ));
            if let Some(weights) = world.faction_weights(faction) {
                lines.push(Line::row("war", weights.war.to_string()));
                lines.push(Line::row("trade", weights.trade.to_string()));
                lines.push(Line::row("build", weights.build.to_string()));
                lines.push(Line::row("renown", weights.renown.to_string()));
            }
        }
        lines
    }
}

#[cfg(test)]
mod tests {
    use cachette_core::{Axial, FactionId, World, WorldConfig};

    use super::*;
    use crate::paint::Camera;
    use crate::panel::market::tests::a_held_tile;
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

    fn score_says(world: &World, pointer: Option<Axial>) -> Vec<String> {
        says(&view(world, pointer), Set::EMPTY.with("score").unwrap())
    }

    #[test]
    fn a_fresh_game_has_not_ended_and_counts_down() {
        let world = world();
        let limit = world.tick_limit();
        let said = score_says(&world, None);
        assert!(said.iter().any(|line| line == "ended: no"), "{said:?}");
        assert!(said.iter().any(|line| line == "tick: 0"), "{said:?}");
        let remaining = format!("ticks remaining: {}", grouped(limit));
        assert!(said.contains(&remaining), "{said:?}");
        assert!(said.iter().any(|line| line == "faction 2: 0"), "{said:?}");
    }

    #[test]
    fn the_score_is_the_held_tile_count_and_the_weights_are_named() {
        let mut world = world();
        a_held_tile(&mut world, FactionId(1));
        let held = world.holding_of(FactionId(1));
        assert!(held > 0);
        let weights = world
            .faction_weights(FactionId(1))
            .expect("the faction exists");

        let said = score_says(&world, None);
        let score = format!("faction 1: {}", grouped(held as u64));
        assert!(said.contains(&score), "{said:?}");
        // The weights are drawn from the seed, so the test reads them back
        // and checks that each row states what the engine holds.
        let at = said.iter().position(|line| *line == score).unwrap();
        assert_eq!(said[at + 1], format!("war: {}", weights.war), "{said:?}");
        assert_eq!(
            said[at + 2],
            format!("trade: {}", weights.trade),
            "{said:?}"
        );
        assert_eq!(
            said[at + 3],
            format!("build: {}", weights.build),
            "{said:?}"
        );
        assert_eq!(
            said[at + 4],
            format!("renown: {}", weights.renown),
            "{said:?}"
        );
    }

    #[test]
    fn a_game_end_names_the_winner_the_path_and_the_tick() {
        let mut world = world();
        world.set_tick_limit(2);
        a_held_tile(&mut world, FactionId(2));
        let end = world.game_end();
        assert!(end.is_set(), "the territory reader fires at the limit");

        let said = score_says(&world, None);
        assert!(said.iter().any(|line| line == "ended: yes"), "{said:?}");
        let winner = format!("winner: faction {}", end.winner.0);
        assert!(said.contains(&winner), "{said:?}");
        assert!(
            said.iter().any(|line| line == "path: territory"),
            "{said:?}"
        );
        let at = format!("at tick: {}", grouped(end.tick.0));
        assert!(said.contains(&at), "{said:?}");
        assert!(
            !said.iter().any(|line| line.starts_with("ticks remaining")),
            "{said:?}"
        );
    }

    #[test]
    fn a_pointer_puts_the_pointed_faction_first() {
        let mut world = world();
        let held = a_held_tile(&mut world, FactionId(2));
        let said = score_says(&world, Some(held));
        let first = said
            .iter()
            .find(|line| line.starts_with("faction "))
            .expect("a faction is named");
        assert!(first.starts_with("faction 2: "), "{said:?}");
    }

    /// A cut line states something other than what it was given, silently.
    /// The worst score is every tile of a world at the target scale. The
    /// worst tick is one tick each millisecond for a year, which is more
    /// ticks than any run of this project has taken. A weight is one byte.[^1]
    ///
    /// # References
    ///
    /// [^1]: Testing Rules, section 2a. `.claude/rules/testing.md`
    #[test]
    fn no_line_is_cut_at_the_worst_plausible_numbers() {
        let worst_tick = grouped(1_000 * 60 * 60 * 24 * 366);
        let lines = [
            Line::row("tick", worst_tick.clone()),
            Line::row("tick limit", worst_tick.clone()),
            Line::row("ticks remaining", worst_tick.clone()),
            Line::row("at tick", worst_tick),
            Line::row("winner", "faction 62".to_string()),
            Line::row("path", "wealth_or_wonder".to_string()),
            Line::swatch(0x00ff_00ff, "faction 62", grouped(16_777_216)),
            Line::row("renown", u8::MAX.to_string()),
        ];
        for line in &lines {
            assert!(!line.is_cut(), "line was cut: {line:?}");
        }
    }

    #[test]
    fn the_panel_itself_produces_no_cut_line() {
        let mut world = world();
        let held = a_held_tile(&mut world, FactionId(0));
        let bad =
            lines_that_do_not_fit(&view(&world, Some(held)), Set::EMPTY.with("score").unwrap());
        assert!(bad.is_empty(), "cut lines: {bad:?}");
    }
}
