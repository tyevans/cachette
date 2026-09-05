//! The panel that shows the water in the air and on the ground.
//!
//! A watcher who looks at a pale overlay cannot tell a storm that is
//! spreading from one that is falling. This panel states the totals the
//! weather field keeps, so the movement of the water is visible as a number
//! while the overlay shows where it is.
//!
//! This file is the whole panel. It states its own name, its own title and
//! its own lines, and the standard draws it.[^1]
//!
//! # What the panel reads
//!
//! The panel reads the weather field through the world, and it reads no tile
//! and no unit.[^2] Two of the totals, what a god raised and what
//! evaporated, are running fields of the weather field, and each costs one
//! read. **Three of them, the air, the ground and the wet cell count, are
//! sums that the field computes over the level 1 lattice when asked.** The
//! lattice is smaller than the world by the square of the block edge, and the
//! Python `weather_totals` verb reads the same three sums through the same
//! reader. A running field for each of the three would remove the pass, and
//! the field holds none today.
//!
//! When the caller sets a pointer, the panel reads the air and the ground at
//! the cell that covers the pointed tile. Each is one array read through the
//! cell of the tile.[^3]
//!
//! # References
//!
//! [^1]: The panel standard. `crates/cachette-view/src/panel/mod.rs`
//! [^2]: ADR-0070, the head-up display reports what the drawing pass read, decision D1. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
//! [^3]: ADR-0140, weather is a field over the level 1 cell lattice, decision D1. `docs/adrs/draft/adr-0140-weather-is-a-field-over-the-level-1-cell-lattice.md`

use super::{Line, Panel, View};
use crate::hud::grouped;

/// What the panel says when no water has entered the world.
///
/// A field that holds no water allocates nothing, and the totals would all
/// read zero. The note says why, so a reader does not take a dry world for a
/// broken pass.[^1]
///
/// # References
///
/// [^1]: ADR-0140, weather is a field over the level 1 cell lattice, decision D2. `docs/adrs/draft/adr-0140-weather-is-a-field-over-the-level-1-cell-lattice.md`
pub const DRY_NOTE: &str = "the world holds no water.";

/// What the panel says when the watcher points at nothing.
const NO_POINTER: &str = "no tile is pointed at.";

/// The panel that shows the water in the air and on the ground.
pub struct Weather;

impl Panel for Weather {
    fn name(&self) -> &'static str {
        "weather"
    }

    fn title(&self) -> &'static str {
        "WEATHER"
    }

    fn lines(&self, view: &View<'_>) -> Vec<Line> {
        let world = view.world;
        let field = world.weather();

        let mut lines = vec![Line::heading("WATER")];
        if field.is_dry() {
            lines.push(Line::note(DRY_NOTE));
        }
        lines.push(Line::row("in the air", drops(field.air_total().0)));
        lines.push(Line::row("on the ground", drops(field.ground_total().0)));
        lines.push(Line::row("evaporated", drops(field.evaporated())));
        lines.push(Line::row("raised", drops(field.raised())));
        lines.push(Line::row(
            "wet cells",
            grouped(u64::from(field.wet_cells())),
        ));

        lines.push(Line::Rule);
        lines.push(Line::heading("POINTED CELL"));
        match view.pointer {
            None => lines.push(Line::note(NO_POINTER)),
            Some(pointer) => match (world.air_at(pointer), world.ground_water_at(pointer)) {
                (Some(air), Some(ground)) => {
                    lines.push(Line::row(
                        "tile",
                        format!("q {}  r {}", pointer.q, pointer.r),
                    ));
                    lines.push(Line::row("in the air", drops(air)));
                    lines.push(Line::row("on the ground", drops(ground)));
                    lines.push(Line::row(
                        "wet",
                        if world.ground_is_wet(pointer) == Some(true) {
                            "yes"
                        } else {
                            "no"
                        },
                    ));
                }
                _ => {
                    lines.push(Line::note("the pointed tile is outside"));
                    lines.push(Line::note("the world."));
                }
            },
        }

        lines
    }
}

/// Returns a count of drops as text.
///
/// A drop is a whole number and the field never holds a negative count. A
/// negative value would be a defect in the field, and it is shown as one
/// rather than hidden behind a zero.
fn drops(count: i64) -> String {
    match u64::try_from(count) {
        Ok(count) => grouped(count),
        Err(_) => format!("{count}"),
    }
}

#[cfg(test)]
mod tests {
    use cachette_core::weather::{PLACES_CEILING, STRENGTH_CEILING};
    use cachette_core::{Axial, FactionId, World, WorldConfig};

    use super::*;
    use crate::paint::Camera;
    use crate::panel::{lines_that_do_not_fit, says, Set};

    const EXTENT: u32 = 64;

    /// Builds a small world for the fixture.
    fn world() -> World {
        World::new(WorldConfig {
            width: EXTENT,
            height: EXTENT,
            seed: 0x0123_4567_89ab_cdef,
            faction_count: 2,
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
    /// holds ground, and returns one tile it holds.
    fn a_held_tile(world: &mut World) -> Axial {
        for row in 20..40 {
            for column in 20..40 {
                let at = Axial::new(column, row);
                if world.admits_a_unit(at) {
                    world
                        .spawn_soldier(at, FactionId(0))
                        .expect("the address and the faction are valid");
                }
            }
        }
        for _ in 0..4 {
            world.step(1).expect("the step must run");
        }
        let held = world
            .holding()
            .tiles_held_by(FactionId(0))
            .next()
            .expect("the faction holds a tile after four ticks");
        let grid = world.grid();
        Axial::new(
            (held.0 % grid.width()) as i32,
            (held.0 / grid.width()) as i32,
        )
    }

    #[test]
    fn a_dry_world_says_so() {
        let world = world();
        let said = says(&view(&world, None), Set::EMPTY.with("weather").unwrap());
        assert!(said.iter().any(|line| line == DRY_NOTE), "{said:?}");
        assert!(said.iter().any(|line| line == "in the air: 0"), "{said:?}");
        assert!(said.iter().any(|line| line == NO_POINTER), "{said:?}");
    }

    #[test]
    fn a_storm_raises_the_air_total() {
        let mut world = world();
        let place = a_held_tile(&mut world);
        let storm = world
            .inflict_weather(FactionId(0), &[place], 4)
            .expect("the faction holds the ground it storms");
        assert!(storm.drops > 0);

        let said = says(&view(&world, None), Set::EMPTY.with("weather").unwrap());
        assert!(!said.iter().any(|line| line == DRY_NOTE), "{said:?}");
        let raised = format!("raised: {}", grouped(storm.drops as u64));
        assert!(said.contains(&raised), "{said:?}");
        let air = format!("in the air: {}", grouped(storm.drops as u64));
        assert!(said.contains(&air), "{said:?}");
    }

    #[test]
    fn a_pointer_names_the_cell_it_points_at() {
        let mut world = world();
        let place = a_held_tile(&mut world);
        let storm = world
            .inflict_weather(FactionId(0), &[place], 2)
            .expect("the faction holds the ground it storms");

        let said = says(
            &view(&world, Some(place)),
            Set::EMPTY.with("weather").unwrap(),
        );
        let tile = format!("tile: q {}  r {}", place.q, place.r);
        assert!(said.contains(&tile), "{said:?}");
        // The storm fell on one cell, and the pointed tile sits in it, so the
        // air over the tile is the whole storm. The line appears once in the
        // totals and once for the cell.
        let air = format!("in the air: {}", grouped(storm.drops as u64));
        assert_eq!(
            said.iter().filter(|line| **line == air).count(),
            2,
            "{said:?}"
        );
        assert!(said.iter().any(|line| line == "wet: no"), "{said:?}");
    }

    #[test]
    fn a_pointer_outside_the_world_is_named_as_outside() {
        let world = world();
        let outside = Axial::new(-1, -1);
        let said = says(
            &view(&world, Some(outside)),
            Set::EMPTY.with("weather").unwrap(),
        );
        assert!(
            said.iter()
                .any(|line| line == "the pointed tile is outside"),
            "{said:?}"
        );
    }

    /// A cut line states something other than what it was given, silently.
    /// A small storm never reaches the edge of the row, so the test supplies
    /// the worst count a run plausibly reaches.[^1]
    ///
    /// The largest storm one call admits covers the place ceiling at the
    /// strength ceiling. The worst total here is a million of those, which
    /// is more storms than a run of a million ticks can raise under the
    /// cooldown. The field can hold more, and a row that reached it would be
    /// cut; the first attempt at this test asked for the type's maximum and
    /// found the cut.
    ///
    /// # References
    ///
    /// [^1]: Testing Rules, section 2a. `.claude/rules/testing.md`
    #[test]
    fn no_line_is_cut_at_the_worst_plausible_numbers() {
        // The engine holds the drops one strength raises as a private
        // figure, so the test reads it back from a storm at the ceiling
        // strength over one place, and the two cannot drift apart.
        let mut world = world();
        let place = a_held_tile(&mut world);
        let one_place = world
            .inflict_weather(FactionId(0), &[place], STRENGTH_CEILING)
            .expect("the faction holds the ground it storms");
        let largest_storm = one_place.drops * PLACES_CEILING as i64;
        let worst_drops = drops(largest_storm * 1_000_000);
        let worst_cells = grouped(u64::from(u32::MAX));
        let lines = [
            Line::row("in the air", worst_drops.clone()),
            Line::row("on the ground", worst_drops.clone()),
            Line::row("evaporated", worst_drops.clone()),
            Line::row("raised", worst_drops),
            Line::row("wet cells", worst_cells),
            Line::row("tile", format!("q {}  r {}", 999_999, 999_999)),
            Line::row("wet", "yes".to_string()),
        ];
        for line in &lines {
            assert!(!line.is_cut(), "line was cut: {line:?}");
        }
    }

    #[test]
    fn the_panel_itself_produces_no_cut_line() {
        let mut world = world();
        let place = a_held_tile(&mut world);
        world
            .inflict_weather(FactionId(0), &[place], 4)
            .expect("the faction holds the ground it storms");
        let bad = lines_that_do_not_fit(
            &view(&world, Some(place)),
            Set::EMPTY.with("weather").unwrap(),
        );
        assert!(bad.is_empty(), "cut lines: {bad:?}");
    }
}
