//! The panel that shows the tile the watcher pointed at.
//!
//! The window has no cursor of its own. The caller that owns the mouse
//! passes the tile it points at, and this panel names it.[^1] The panel
//! never guesses a tile. When the caller gives no pointer, or the pointer
//! names an address outside the world, the panel says so instead of
//! showing the middle of the window, because the head-up display already
//! reports that tile and two panels that named different tiles under one
//! heading would mislead a watcher.[^2]
//!
//! This file is the whole panel. It states its own name, its own title and
//! its own lines, and the standard draws it.[^3]
//!
//! # What the panel reads
//!
//! The tile fields come from the head-up display's own tile reader, which
//! this panel reuses rather than reading the ground a second time.[^4] The
//! capacity and the holder are one address each, read straight from the
//! world. The unit list reads the spatial bridge at the same one address,
//! bounded to a small named number of rows, so the cost never follows the
//! population of the tile.[^5]
//!
//! # References
//!
//! [^1]: ADR-0094, the caller owns the camera and the pixels, decision D1. `docs/adrs/draft/adr-0094-the-caller-owns-the-camera-and-the-pixels.md`
//! [^2]: ADR-0070, the head-up display reports what the drawing pass read, decision D2. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
//! [^3]: The panel standard. `crates/cachette-view/src/panel/mod.rs`
//! [^4]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
//! [^5]: ADR-0070, the head-up display reports what the drawing pass read, decision D1. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`

use cachette_core::{Entity, Holder, ResourceKind, World};

use super::{Line, Panel, View};
use crate::hud::{grouped, name_of, resource_name, TileReadout};

/// What the panel says when the watcher points at nothing.
const NO_POINTER: &str = "no tile is pointed at.";

/// The number of units the panel names by row.
///
/// A tile can hold many units. The panel reads at most this many, so its
/// cost never follows the population of the tile.[^1]
///
/// # References
///
/// [^1]: ADR-0070, the head-up display reports what the drawing pass read, decision D1. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
const UNIT_ROWS: usize = 6;

/// The panel that shows the tile the watcher pointed at.
pub struct Inspector;

impl Panel for Inspector {
    fn name(&self) -> &'static str {
        "inspector"
    }

    fn title(&self) -> &'static str {
        "THE TILE"
    }

    fn lines(&self, view: &View<'_>) -> Vec<Line> {
        let Some(pointer) = view.pointer else {
            return vec![Line::note(NO_POINTER)];
        };
        let world = view.world;
        let Some(tile) = TileReadout::of(world, pointer) else {
            return vec![
                Line::note("the pointed tile is outside"),
                Line::note("the world."),
            ];
        };

        let mut lines = vec![
            Line::row(
                "tile",
                format!("q {}  r {}", tile.address().q, tile.address().r),
            ),
            Line::row("ground", name_of(tile.kind()).to_string()),
        ];

        for kind in ResourceKind::ALL {
            lines.push(Line::row(
                format!("{} left", resource_name(kind)),
                deposit(&tile, kind),
            ));
        }

        lines.push(Line::row(
            "it admits",
            match world.tile_capacity(pointer) {
                Some(capacity) => grouped(u64::from(capacity)),
                None => "-".to_string(),
            },
        ));
        lines.push(Line::row(
            "held by",
            match world.tile_holder(pointer).and_then(Holder::faction) {
                Some(faction) => format!("faction {}", faction.0),
                None => "nobody".to_string(),
            },
        ));
        lines.push(Line::row(
            "units here",
            match tile.units() {
                Some(count) => grouped(u64::from(count)),
                None => "-".to_string(),
            },
        ));

        lines.extend(unit_lines(world, pointer));

        lines
    }
}

/// Returns one deposit as text: what is left, of what the ground gave.
///
/// A tile the ground gave nothing of returns a word and not a pair of
/// zeroes, so a reader never mistakes an ungenerated deposit for a drained
/// one.[^1]
///
/// # References
///
/// [^1]: ADR-0070, the head-up display reports what the drawing pass read, decision D2. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
fn deposit(tile: &TileReadout, kind: ResourceKind) -> String {
    let generated = tile.generated(kind);
    if generated == 0 {
        return "none here".to_string();
    }
    format!("{} of {generated}", tile.stock(kind))
}

/// Returns the rows that name the units standing on one tile.
///
/// The read stops at a fixed number of units, so a crowded tile costs the
/// panel no more than an empty one.[^1] A bridge that no longer describes
/// the world names no unit, and the panel then adds no row rather than one
/// that shows a stale answer.[^2]
///
/// # References
///
/// [^1]: ADR-0070, the head-up display reports what the drawing pass read, decision D1. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
/// [^2]: ADR-0070, the head-up display reports what the drawing pass read, decision D2. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
fn unit_lines(world: &World, pointer: cachette_core::Axial) -> Vec<Line> {
    let Ok(soldiers) = world.soldiers_on(pointer) else {
        return Vec::new();
    };
    if soldiers.is_empty() {
        return Vec::new();
    }

    let mut lines = vec![Line::Rule, Line::heading("UNITS")];
    for (position, &entity) in soldiers.iter().take(UNIT_ROWS).enumerate() {
        lines.push(Line::row(
            format!("unit {}", position + 1),
            unit_summary(world, entity),
        ));
    }
    lines
}

/// Returns what one unit is doing, for one row of the tile inspector.
///
/// Returns "gone" when the identity has already died between the bridge
/// build and this read, because the bridge answers with the last address
/// it knew and the world answers for the identity as it stands now.[^1]
///
/// # References
///
/// [^1]: ADR-0014, entity identity is an index plus a generation, decision D2. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
fn unit_summary(world: &World, entity: Entity) -> String {
    let Some(faction) = world.soldiers().faction(entity) else {
        return "gone".to_string();
    };
    match world.unit_condition(entity) {
        Some(condition) => format!("f{} {condition}", faction.0),
        None => format!("f{}", faction.0),
    }
}

#[cfg(test)]
mod tests {
    use cachette_core::{Axial, FactionId, World, WorldConfig};

    use super::*;
    use crate::paint::Camera;
    use crate::panel::{lines_that_do_not_fit, says, Line as PanelLine, Set};

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

    fn view(world: &World, pointer: Option<Axial>) -> View<'_> {
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

    #[test]
    fn with_no_pointer_the_panel_names_no_tile_and_shows_no_zero() {
        let world = world(1);
        let view = view(&world, None);
        let said = says(&view, Set::EMPTY.with("inspector").unwrap());

        assert!(said.iter().any(|line| line == NO_POINTER));
        assert!(!said.iter().any(|line| line.contains(": 0")));
    }

    #[test]
    fn with_a_pointer_outside_the_world_the_panel_says_so() {
        let world = world(1);
        let view = view(&world, Some(Axial { q: 99, r: 99 }));
        let said = says(&view, Set::EMPTY.with("inspector").unwrap());

        assert!(said.iter().any(|line| line.contains("outside")));
    }

    #[test]
    fn with_a_real_tile_the_ground_the_stock_and_the_holder_match() {
        let world = world(2);
        let place = Axial { q: 3, r: 2 };

        let view = view(&world, Some(place));
        let said = says(&view, Set::EMPTY.with("inspector").unwrap());

        let kind = world.tile_kind(place).expect("the tile lies in the world");
        assert!(said
            .iter()
            .any(|line| line == &format!("ground: {}", name_of(kind))));
        let held = match world.tile_holder(place).and_then(Holder::faction) {
            Some(faction) => format!("held by: faction {}", faction.0),
            None => "held by: nobody".to_string(),
        };
        assert!(said.iter().any(|line| line == &held));

        for resource in ResourceKind::ALL {
            let stock = world
                .tile_stock(place, resource)
                .expect("the tile lies in the world");
            let generated = world
                .original_stock(place, resource)
                .expect("the tile lies in the world");
            let expected = if generated.0 == 0 {
                "none here".to_string()
            } else {
                format!("{} of {}", stock.0, generated.0)
            };
            assert!(said
                .iter()
                .any(|line| line == &format!("{} left: {expected}", resource_name(resource))));
        }
    }

    #[test]
    fn with_units_standing_on_the_tile_the_count_matches() {
        let mut world = world(1);
        let place = Axial { q: 1, r: 1 };
        for _ in 0..3 {
            world
                .spawn_soldier(place, FactionId(0))
                .expect("the tile admits a soldier");
        }
        world.rebuild_bridge(1).expect("the bridge rebuilds");

        let view = view(&world, Some(place));
        let said = says(&view, Set::EMPTY.with("inspector").unwrap());

        assert!(said.iter().any(|line| line == "units here: 3"));
        assert!(said.iter().any(|line| line == "unit 1: f0 fed"));
        assert!(said.iter().any(|line| line == "unit 3: f0 fed"));
    }

    /// A cut line states something other than what it was given, silently.
    /// A fixture built only from small numbers would never reach the edge
    /// of a row, so this test asserts on the widest values the panel could
    /// plausibly be asked to show.[^1]
    ///
    /// # References
    ///
    /// [^1]: Testing Rules, section 2a. `.claude/rules/testing.md`
    #[test]
    fn no_line_is_cut_at_the_widest_plausible_values() {
        let worst_coordinate = format!("q {}  r {}", 999_999, 999_999);
        let worst_stock = format!("{} of {}", 65_535, 65_535);
        let worst_capacity = grouped(u64::from(u32::MAX));
        let worst_holder = format!("faction {}", u16::MAX);
        let worst_count = grouped(1_000_000);
        let worst_unit = format!("f{} starved", u16::MAX);

        let lines = [
            PanelLine::row("tile", worst_coordinate),
            PanelLine::row("ground", "mountain"),
            PanelLine::row("food left", worst_stock.clone()),
            PanelLine::row("wood left", worst_stock.clone()),
            PanelLine::row("stone left", worst_stock),
            PanelLine::row("it admits", worst_capacity),
            PanelLine::row("held by", worst_holder),
            PanelLine::row("units here", worst_count),
            PanelLine::row(format!("unit {UNIT_ROWS}"), worst_unit),
        ];

        for line in &lines {
            assert!(!line.is_cut(), "line was cut: {line:?}");
        }
    }

    #[test]
    fn the_panel_itself_produces_no_cut_line_for_a_crowded_tile() {
        let mut world = world(1);
        let place = Axial { q: 0, r: 0 };
        for _ in 0..UNIT_ROWS {
            world
                .spawn_soldier(place, FactionId(0))
                .expect("the tile admits a soldier");
        }
        world.rebuild_bridge(1).expect("the bridge rebuilds");

        let view = view(&world, Some(place));
        let bad = lines_that_do_not_fit(&view, Set::EMPTY.with("inspector").unwrap());
        assert!(bad.is_empty(), "cut lines: {bad:?}");
    }
}
