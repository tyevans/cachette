//! The panel that shows the population, the holdings and the stores.
//!
//! A watcher who looks at coloured cells cannot see a faction die. The last
//! unit ends, the colour is gone from the picture, and nothing on the glass
//! ever said the count reached zero. This panel states the population of
//! every faction, so that fall is visible before the faction is gone.[^1]
//!
//! This file is the whole panel. It states its own name, its own title and
//! its own lines, and the standard draws it.[^2]
//!
//! # What the panel reads
//!
//! Every number here is a running total or a bounded loop over the factions.
//! The world holds at most sixty-three factions, so a loop over them costs
//! the same at any population and at any world size.[^3] The panel starts no
//! pass over a tile and no pass over a unit.
//!
//! # References
//!
//! [^1]: ADR-0070, the head-up display reports what the drawing pass read, decision D2. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
//! [^2]: The panel standard. `crates/cachette-view/src/panel/mod.rs`
//! [^3]: ADR-0070, the head-up display reports what the drawing pass read, decision D1. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`

use cachette_core::FactionId;

use super::{Line, Panel, View};
use crate::hud::grouped;
use crate::paint::faction_colour;

/// The panel that shows the population, the holdings and the stores.
pub struct Statistics;

impl Panel for Statistics {
    fn name(&self) -> &'static str {
        "statistics"
    }

    fn title(&self) -> &'static str {
        "STATISTICS"
    }

    fn lines(&self, view: &View<'_>) -> Vec<Line> {
        let world = view.world;
        let faction_count = usize::from(world.config().faction_count).max(1);

        let mut lines = vec![
            Line::heading("POPULATION"),
            Line::row("in the world", grouped(u64::from(world.soldiers().len()))),
        ];

        // The running total, read once for each faction. The array is sized
        // to the ceiling of sixty-three, and the loop stops at the count the
        // world was built with, so the cost never follows the population.[^1]
        //
        // [^1]: ADR-0070, the head-up display reports what the drawing pass read, decision D1. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
        let population = world.population_by_faction();
        for (index, count) in population.iter().enumerate().take(faction_count) {
            lines.push(Line::swatch(
                faction_colour(FactionId(index as u16)),
                format!("faction {index}"),
                grouped(u64::from(*count)),
            ));
        }

        lines.push(Line::Rule);
        lines.push(Line::heading("HOLDINGS"));
        // The world keeps one running total of held tiles, so this reads it
        // rather than summing the loop below a second time.[^1]
        //
        // [^1]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
        let held_tiles = world.holding().held_tiles().max(0);
        lines.push(Line::row("tiles held", grouped(held_tiles as u64)));
        for index in 0..faction_count {
            let faction = FactionId(index as u16);
            let held = world.holding_of(faction).max(0);
            lines.push(Line::swatch(
                faction_colour(faction),
                format!("faction {index}"),
                grouped(held as u64),
            ));
        }

        lines.push(Line::Rule);
        lines.push(Line::heading("SITES"));
        lines.push(Line::row(
            "sites",
            grouped(u64::from(world.settlements().len())),
        ));
        // The store of a site rations when it cannot serve what its cohorts
        // asked for, and a unit ends when a shortage carries it to the
        // bound. Both logs hold the last step only, so the row appears
        // solely when the last step produced one. A permanent zero would
        // still say something even if the pass behind it broke.[^1]
        //
        // [^1]: Testing Rules, section 2a. `.claude/rules/testing.md`
        if !world.rationed_log().is_empty() {
            lines.push(Line::row(
                "rationed last tick",
                grouped(world.rationed_log().len() as u64),
            ));
        }
        if !world.starved_log().is_empty() {
            lines.push(Line::row(
                "ended last tick",
                grouped(world.starved_log().len() as u64),
            ));
        }

        lines
    }
}

#[cfg(test)]
mod tests {
    use cachette_core::{Axial, FactionId, World, WorldConfig};

    use super::*;
    use crate::paint::Camera;
    use crate::panel::{says, Set};

    /// Builds a small world for the fixture.
    fn world(faction_count: u16) -> World {
        World::new(WorldConfig {
            width: 8,
            height: 8,
            seed: 0x0123_4567_89ab_cdef,
            faction_count,
            unit_capacity: 64,
        })
        .expect("a small extent describes a world")
    }

    /// Returns the address of a tile index inside the fixture world.
    fn address(index: u32) -> Axial {
        Axial {
            q: (index % 8) as i32,
            r: (index / 8) as i32,
        }
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

    /// Spawns a number of soldiers of one faction, at distinct addresses.
    fn spawn(world: &mut World, faction: FactionId, count: u32) -> Vec<cachette_core::Entity> {
        (0..count)
            .map(|index| {
                world
                    .spawn_soldier(address(index), faction)
                    .expect("the tile admits a soldier")
            })
            .collect()
    }

    #[test]
    fn the_lines_name_each_factions_population() {
        let mut world = world(3);
        spawn(&mut world, FactionId(0), 5);
        spawn(&mut world, FactionId(1), 2);

        let view = view(&world);
        let said = says(&view, Set::EMPTY.with("statistics").unwrap());

        assert!(said.iter().any(|line| line == "faction 0: 5"));
        assert!(said.iter().any(|line| line == "faction 1: 2"));
        assert!(said.iter().any(|line| line == "faction 2: 0"));
        assert!(said.iter().any(|line| line == "in the world: 7"));
    }

    /// A faction that loses its last unit reads zero.
    ///
    /// This is the case the panel exists for: a faction that dies is
    /// invisible on the picture until a reader counts its colour by eye.
    #[test]
    fn a_faction_that_loses_its_last_unit_reads_zero() {
        let mut world = world(2);
        let soldiers = spawn(&mut world, FactionId(0), 3);
        for soldier in soldiers {
            assert!(world.despawn_soldier(soldier));
        }

        let view = view(&world);
        let said = says(&view, Set::EMPTY.with("statistics").unwrap());

        assert!(said.iter().any(|line| line == "faction 0: 0"));
    }

    /// A cut line states something other than what it was given, silently.
    /// A test that only ran the fixture's small numbers would never see it,
    /// because a small number never reaches the edge of the row.[^1]
    ///
    /// # References
    ///
    /// [^1]: Testing Rules, section 2a. `.claude/rules/testing.md`
    #[test]
    fn no_line_is_cut_at_the_worst_plausible_numbers() {
        let worst_population = grouped(u64::from(u32::MAX));
        let worst_tiles = grouped(16_777_216);

        let lines = [
            Line::row("in the world", worst_population.clone()),
            Line::swatch(0x00ff_00ff, "faction 62", worst_population.clone()),
            Line::row("tiles held", worst_tiles.clone()),
            Line::swatch(0x00ff_00ff, "faction 62", worst_tiles.clone()),
            Line::row("sites", worst_tiles.clone()),
            Line::row("rationed last tick", worst_tiles.clone()),
            Line::row("ended last tick", worst_population),
        ];

        for line in &lines {
            assert!(!line.is_cut(), "line was cut: {line:?}");
        }
    }

    #[test]
    fn the_panel_itself_produces_no_cut_line_for_a_full_faction_count() {
        let mut world = world(63);
        spawn(&mut world, FactionId(0), 4);
        let view = view(&world);

        let bad =
            crate::panel::lines_that_do_not_fit(&view, Set::EMPTY.with("statistics").unwrap());
        assert!(bad.is_empty(), "cut lines: {bad:?}");
    }
}
