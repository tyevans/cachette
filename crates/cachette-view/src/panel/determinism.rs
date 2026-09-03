//! The panel that says what the run is doing.
//!
//! Determinism is the one property this project cannot recover once it is
//! lost.[^1] A watcher needs to see the run, not trust it. This panel states
//! the seed that fixes the run, the tick the run has reached, and the extent
//! of the world the run holds. Each of these is a stored field, not a sum
//! over the world.[^2]
//!
//! This file is the whole panel. It states its own name, its own title and
//! its own lines, and the standard draws it.[^3]
//!
//! # Why the panel does not show the state hash
//!
//! The golden test hashes the whole world state and compares it against a
//! stored file, and that hash is the strongest evidence that a run
//! repeats.[^4] A panel may not compute it.
//!
//! The hash folds the tile value column, the terrain, the resources, the
//! soldier arena, the settlements and every other store, one field at a
//! time. Each fold walks the store it folds. The cost follows the extent of
//! the world and the size of the population, so a panel that read this hash
//! every frame would start a pass over the world on every frame.[^2] A panel
//! reads what the engine already holds, at a bounded number of addresses,
//! and this hash is not one of them.
//!
//! The same holds for the invariant check. It walks the tile values, the
//! upgrades, the soldier arena, the settlements, the characters, the
//! holdings, the rates, the positions and the cohorts, in turn, and it does
//! this to prove a property the drawing pass does not need. The panel does
//! not call it.
//!
//! # References
//!
//! [^1]: Project orientation, hard invariants. `CLAUDE.md`
//! [^2]: ADR-0070, the head-up display reports what the drawing pass read, decision D1. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
//! [^3]: The panel standard. `crates/cachette-view/src/panel/mod.rs`
//! [^4]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`

use super::{Line, Panel, View};
use crate::hud::grouped;

/// The panel that says what the run is doing.
pub struct Determinism;

impl Panel for Determinism {
    fn name(&self) -> &'static str {
        "determinism"
    }

    fn title(&self) -> &'static str {
        "THE RUN"
    }

    fn lines(&self, view: &View<'_>) -> Vec<Line> {
        let world = view.world;
        let config = world.config();
        let soldiers = world.soldiers();

        vec![
            Line::heading("SEED"),
            Line::row("seed", format!("0x{:016x}", config.seed)),
            Line::note("seed and tick repeat a run"),
            Line::Rule,
            Line::heading("WORLD"),
            Line::row("tick", grouped(world.tick().0)),
            Line::row("width", grouped(u64::from(config.width))),
            Line::row("height", grouped(u64::from(config.height))),
            Line::row("factions", grouped(u64::from(config.faction_count))),
            Line::row("changed tiles", grouped(world.stored_tile_changes() as u64)),
            Line::row("events, last tick", grouped(world.event_log().len() as u64)),
            Line::Rule,
            Line::heading("SOLDIER ARENA"),
            Line::row("live", grouped(u64::from(soldiers.len()))),
            Line::row("slots", grouped(u64::from(soldiers.slot_count()))),
            Line::row("retired", grouped(u64::from(soldiers.retired_count()))),
            Line::row("revision", grouped(soldiers.revision())),
            Line::row("arena id", grouped(soldiers.identity())),
            Line::Rule,
            Line::heading("FRAME"),
            Line::row("width", grouped(view.frame_width as u64)),
            Line::row("height", grouped(view.frame_height as u64)),
            Line::note("the state hash is a full pass"),
            Line::note("this panel omits it"),
        ]
    }
}

#[cfg(test)]
mod tests {
    use cachette_core::{World, WorldConfig};

    use super::*;
    use crate::paint::Camera;
    use crate::panel::{lines_that_do_not_fit, says, Set};

    /// Builds a small world for the fixture.
    fn world(seed: u64) -> World {
        World::new(WorldConfig {
            width: 8,
            height: 8,
            seed,
            faction_count: 4,
            unit_capacity: 64,
        })
        .expect("a small extent describes a world")
    }

    fn view(world: &World) -> View<'_> {
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
            pointer: None,
        }
    }

    #[test]
    fn the_panel_states_the_tick_the_world_holds() {
        let mut world = world(1);
        world.step(1).expect("one tick advances a small world");
        world.step(1).expect("a second tick advances a small world");

        let view = view(&world);
        let said = says(&view, Set::EMPTY.with("determinism").unwrap());

        assert!(said
            .iter()
            .any(|line| line == &format!("tick: {}", grouped(world.tick().0))));
    }

    /// A panel that read a constant would pass a test that reads one seed.
    /// Two worlds built from different seeds must read back two different
    /// lines, or the test cannot tell a real reading from a fixed string.
    #[test]
    fn the_panel_tells_two_seeds_apart() {
        let low = world(0x0000_0000_0000_0001);
        let high = world(0xffff_ffff_ffff_fffe);

        let low_view = view(&low);
        let high_view = view(&high);
        let low_said = says(&low_view, Set::EMPTY.with("determinism").unwrap());
        let high_said = says(&high_view, Set::EMPTY.with("determinism").unwrap());

        assert!(low_said
            .iter()
            .any(|line| line == "seed: 0x0000000000000001"));
        assert!(high_said
            .iter()
            .any(|line| line == "seed: 0xfffffffffffffffe"));
        assert_ne!(low_said, high_said);
    }

    /// A seed of zero is one glyph and proves nothing about a seed with
    /// every bit set. This checks the row at its widest, not at its
    /// typical value.[^1]
    ///
    /// # References
    ///
    /// [^1]: Testing Rules, section 2a. `.claude/rules/testing.md`
    #[test]
    fn no_line_is_cut_at_the_widest_plausible_values() {
        let worst_count = grouped(u64::from(u32::MAX));

        let lines = [
            Line::row("seed", format!("0x{:016x}", u64::MAX)),
            Line::row("tick", grouped(999_999_999)),
            Line::row("width", grouped(4_096)),
            Line::row("height", grouped(4_096)),
            Line::row("factions", grouped(63)),
            Line::row("changed tiles", grouped(16_777_216)),
            Line::row("events, last tick", worst_count.clone()),
            Line::row("live", worst_count.clone()),
            Line::row("slots", worst_count.clone()),
            Line::row("retired", worst_count.clone()),
            Line::row("revision", worst_count.clone()),
            Line::row("arena id", worst_count),
            Line::row("width", grouped(7_680)),
            Line::row("height", grouped(4_320)),
        ];

        for line in &lines {
            assert!(!line.is_cut(), "line was cut: {line:?}");
        }
    }

    #[test]
    fn the_panel_itself_produces_no_cut_line_for_a_fresh_world() {
        let world = world(0x0123_4567_89ab_cdef);
        let view = view(&world);

        let bad = lines_that_do_not_fit(&view, Set::EMPTY.with("determinism").unwrap());
        assert!(bad.is_empty(), "cut lines: {bad:?}");
    }
}
