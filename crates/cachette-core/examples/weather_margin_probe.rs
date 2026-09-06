//! What the weather margin does to the border of the world, and what it costs.
//!
//! The weather lattice is larger than the world. The extra is a ring of cells
//! on all four sides that the solve steps and no reader sees. The ring exists
//! so that the border of the world has real upwind: without it, air leaves the
//! lattice through one edge and nothing arrives through the other, so the
//! cells beside an edge stay starved.
//!
//! The probe reports two things at a range of ring widths.
//!
//! The first is the shape of the field at the edge of the world. Three
//! readings describe it.
//!
//! The border air is the mean water in the air over the outermost ring of
//! cells of the world, and the middle air is the mean over the middle
//! quarter of it.
//!
//! The edge step is the mean difference between the outermost ring and the
//! ring inside it, as a share of the mean difference between two rings well
//! inside the world. A field with no edge defect reads near one hundred: the
//! outermost ring then differs from its neighbour by as much as any pair of
//! neighbours differ. A bare lattice reads high, because the outermost ring
//! sends water to no outside and therefore holds a quantity that no interior
//! cell holds.
//!
//! The border move is the mean change of a border cell from one frame to the
//! next, against the same reading over the middle. A starved border sits
//! still, because nothing arrives to move it.
//!
//! The second is the cell count and the wall clock cost of a run, so that a
//! reader can weigh the repair against what it costs.
//!
//! The probe measures wall clock time. **It is a probe and not a test.** No
//! gate reads it, and nothing here asserts on a duration.[^1]
//!
//! # References
//!
//! [^1]: Testing rules, section 3. `.agents/rules/testing.md`

use std::time::Instant;

use cachette_core::weather::WeatherScale;
use cachette_core::world::{World, WorldConfig};

/// The frames that each run steps before the probe reads it.
///
/// The count is fixed. The field must settle before a border reading means
/// anything, and a run that stopped at a settling test would stop at a
/// different frame on each world.
const FRAMES: u32 = 128;

/// The world extent the probe runs over.
///
/// The demonstration runs at this extent, so the cost the probe reports is the
/// cost the demonstration pays.
const EXTENT: u32 = 128;

fn main() {
    for bits in [0, 5] {
        let scale = WeatherScale::from_bits(bits).expect("the scale is legal");
        println!(
            "== {EXTENT} by {EXTENT} tiles, weather pitch {} tiles, {FRAMES} frames ==",
            scale.side()
        );
        println!(
            "the derived margin is {} cells, which is {} tiles",
            scale.margin_cells(),
            scale.margin_tiles()
        );
        println!(
            "{:>7}  {:>7}  {:>9}  {:>10}  {:>10}  {:>6}  {:>9}  {:>11}  {:>9}",
            "margin",
            "cells",
            "step cost",
            "border air",
            "middle air",
            "share",
            "edge step",
            "border move",
            "wet cells"
        );
        for margin in margins_for(scale) {
            let Some(row) = run(scale, margin) else {
                println!("{margin:>7}  the world refused to build");
                continue;
            };
            println!(
                "{margin:>7}  {:>7}  {:>7}ms  {:>10}  {:>10}  {:>5}%  {:>8}%  {:>10}%  {:>9}",
                row.cells,
                row.millis,
                row.border,
                row.middle,
                row.share,
                row.edge_step,
                row.border_move,
                row.wet
            );
        }
        println!();
    }
}

/// Returns the margins the probe tries at one pitch.
///
/// The list holds zero, the margin the scale derives, and a width on each side
/// of it, so a reader can see whether the derived width is at the knee.
fn margins_for(scale: WeatherScale) -> Vec<u32> {
    let derived = scale.margin_cells();
    let mut widths = vec![0, derived / 2, derived, derived * 2, derived * 4];
    widths.sort_unstable();
    widths.dedup();
    widths
}

/// One row of the report.
struct Row {
    cells: u32,
    millis: u128,
    border: i64,
    middle: i64,
    share: i64,
    edge_step: i64,
    border_move: i64,
    wet: u32,
}

/// Runs one world and reads its border.
fn run(scale: WeatherScale, margin: u32) -> Option<Row> {
    let config = WorldConfig {
        width: EXTENT,
        height: EXTENT,
        seed: 0x9e37_79b9_7f4a_7c15,
        faction_count: 2,
        unit_capacity: 64,
        ..WorldConfig::default()
    };
    let mut world = World::with_weather_margin(config, scale, margin).ok()?;
    let started = Instant::now();
    let mut before: Vec<i64> = Vec::new();
    let mut border_move = 0i64;
    let mut middle_move = 0i64;
    let mut moves = 0i64;
    for frame in 0..FRAMES {
        world.step(4).expect("the step runs");
        // The last quarter of the run carries the move reading. The field
        // has settled by then, so what the reading holds is the weather and
        // not the opening of the run.
        if frame * 4 < FRAMES * 3 {
            continue;
        }
        let now: Vec<i64> = world
            .weather()
            .air_over_world()
            .iter()
            .map(|drops| drops.0)
            .collect();
        if before.len() == now.len() && !now.is_empty() {
            let inner = world.weather().lattice().inner();
            for row in 0..inner.height() {
                for column in 0..inner.width() {
                    let at = (row * inner.width() + column) as usize;
                    let moved = (now[at] - before[at]).abs();
                    if is_border(inner.width(), inner.height(), column, row) {
                        border_move += moved;
                    } else {
                        middle_move += moved;
                    }
                }
            }
            moves += 1;
        }
        before = now;
    }
    let millis = started.elapsed().as_millis();
    let field = world.weather();
    let inner = field.lattice().inner();
    let plane = field.air_over_world();
    let (mut border_total, mut border_count) = (0i64, 0i64);
    let (mut middle_total, mut middle_count) = (0i64, 0i64);
    // The middle is the quarter of the world that stands furthest from every
    // edge. It is the reading the border is compared against, because it is
    // the part of the world that no edge can starve.
    let quarter_across = inner.width() / 4;
    let quarter_down = inner.height() / 4;
    for row in 0..inner.height() {
        for column in 0..inner.width() {
            let at = (row * inner.width() + column) as usize;
            let held = plane.get(at).map_or(0, |drops| drops.0);
            if column == 0 || row == 0 || column + 1 == inner.width() || row + 1 == inner.height() {
                border_total += held;
                border_count += 1;
            }
            if column >= quarter_across
                && row >= quarter_down
                && column < inner.width() - quarter_across
                && row < inner.height() - quarter_down
            {
                middle_total += held;
                middle_count += 1;
            }
        }
    }
    let border = if border_count == 0 {
        0
    } else {
        border_total / border_count
    };
    let middle = if middle_count == 0 {
        0
    } else {
        middle_total / middle_count
    };
    let share = if middle == 0 {
        0
    } else {
        border * 100 / middle
    };
    // The edge step against an interior step. The rings are counted inward
    // from the edge, so ring zero is the outermost.
    let outer = ring_gap(&plane, inner.width(), inner.height(), 0);
    let ordinary = ring_gap(&plane, inner.width(), inner.height(), 3);
    let edge_step = if ordinary == 0 {
        0
    } else {
        outer * 100 / ordinary
    };
    // The border move against the middle move. Both are means over one cell
    // and one frame, so the two are comparable.
    let border_cells = 2 * i64::from(inner.width()) + 2 * i64::from(inner.height()) - 4;
    let middle_cells = i64::from(inner.tile_count()) - border_cells;
    let border_rate = mean(border_move, border_cells * moves);
    let middle_rate = mean(middle_move, middle_cells * moves);
    let border_move = if middle_rate == 0 {
        0
    } else {
        border_rate * 100 / middle_rate
    };
    Some(Row {
        cells: field.lattice().whole().tile_count(),
        millis,
        border,
        middle,
        share,
        edge_step,
        border_move,
        wet: field.wet_cells(),
    })
}

/// Reports whether a cell stands on the outermost ring of the world.
fn is_border(width: u32, height: u32, column: u32, row: u32) -> bool {
    column == 0 || row == 0 || column + 1 == width || row + 1 == height
}

/// Returns the mean absolute difference between one ring of cells and the ring
/// inside it, counting rings inward from the edge of the world.
///
/// The reading walks the top row and the bottom row of the ring, and the left
/// column and the right column of it, so it holds every pair that crosses the
/// ring boundary.
fn ring_gap(plane: &[cachette_core::weather::Drops], width: u32, height: u32, ring: u32) -> i64 {
    if width < 2 * ring + 4 || height < 2 * ring + 4 {
        return 0;
    }
    let at = |column: u32, row: u32| -> i64 {
        plane
            .get((row * width + column) as usize)
            .map_or(0, |drops| drops.0)
    };
    let mut total = 0i64;
    let mut count = 0i64;
    for column in ring..(width - ring) {
        total += (at(column, ring) - at(column, ring + 1)).abs();
        total += (at(column, height - 1 - ring) - at(column, height - 2 - ring)).abs();
        count += 2;
    }
    for row in ring..(height - ring) {
        total += (at(ring, row) - at(ring + 1, row)).abs();
        total += (at(width - 1 - ring, row) - at(width - 2 - ring, row)).abs();
        count += 2;
    }
    mean(total, count)
}

/// Returns a mean, and zero when nothing was counted.
fn mean(total: i64, count: i64) -> i64 {
    if count <= 0 {
        0
    } else {
        total / count
    }
}
