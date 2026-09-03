//! What a tile-indexed exit direction costs, and what it saves.
//!
//! One backlog item proposes that the engine writes the exit direction once
//! for each tile, so that the movement pass reads one array at the tile index
//! of the unit instead of turning the tile into a cell first. The item states
//! that the added cost is a pass over the tile count, and it states that the
//! cost is the risk of the item and must be measured.[^1]
//!
//! This benchmark takes that measurement. It changes no engine code. It
//! builds the tile-indexed array outside the crate, from the public exit
//! field, so the two shapes are priced against one another before either one
//! is chosen.
//!
//! **The measurement refused the item, and the item is closed.** The
//! benchmark stays so that a later contributor who reaches for this shape can
//! take the figures again rather than argue.[^1] [^3]
//!
//! # The rows
//!
//! **`derive_cells`** is the derivation the engine runs today. It is the
//! baseline that the two write rows are added to.
//!
//! **`tile_write_row_major`** fills a tile-indexed array by walking the world
//! one row at a time. The cell of a tile is a shift and a multiply, so the row
//! pays no lookup, and the writes are sequential.
//!
//! **`tile_write_block_major`** fills the same array by walking one block at a
//! time. It reads each cell entry once, and it writes runs of one block edge.
//!
//! **`lookup_through_the_cell`** and **`lookup_at_the_tile`** read a direction
//! for each live unit, in the order the movement pass reads them. The first is
//! the chain the movement pass runs today. The second is the read the item
//! proposes. The proposed change is worth making when the cheaper write row
//! costs less than the difference between these two.
//!
//! **`cell_derived_from_the_tile`** and **`cell_read_from_a_column`** answer a
//! second item, which stores the cell of a unit beside the tile of the unit
//! and leaves the exit field where it is.[^4] The two rows hold the downstream
//! read fixed and change only how the cell is reached, so the difference
//! between them is the key-and-block arithmetic and nothing else.
//!
//! **`address_of_the_tile_alone`** holds the conversion from a tile index to
//! an address, which is a remainder and a quotient by a width that is not
//! known when the crate is compiled. It says how much of the row above it that
//! one step accounts for.
//!
//! # What it does not measure
//!
//! It does not run a frame. The lookup rows hold the lookup and nothing else,
//! so they overstate the share of a frame that either change can reach. That
//! favours both items, and the refusal of the first one holds against it.
//!
//! **The lookup rows read every live unit, and not only the units that hold an
//! intent.** The movement pass skips a unit with no intent, and the choice
//! schedule leaves most units without one in any single frame. A row over the
//! holders alone would price a change against a fraction of the population.
//!
//! It does not measure resident memory. One row in the other benchmark takes
//! that figure.
//!
//! It does not run on the target platform, and one blocker holds that gap
//! open.[^5]
//!
//! # How to run it
//!
//! ```text
//! cargo bench --bench exit_locality -- 4096x4096 1000000
//! ```
//!
//! Every duration is in nanoseconds. A benchmark does not gate a merge, and
//! no test in this project asserts on time.[^2]
//!
//! # References
//!
//! [^1]: Backlog item 0267, hold the exit direction on the tile. `docs/backlog/complete/0267-hold-the-exit-direction-on-the-tile.md`
//! [^2]: Testing rules, section 3. `.claude/rules/testing.md`
//! [^3]: Findings register, FND-281. `docs/FINDINGS.md`
//! [^4]: Backlog item 0268, hold the cell index on the unit. `docs/backlog/proposed/0268-hold-the-cell-index-on-the-unit.md`
//! [^5]: Blockers register, BLK-007. `docs/BLOCKERS.md`

use std::time::Instant;

use cachette_core::{
    Axial, ExitField, FactionId, Grid, TileIdx, World, WorldConfig, NO_EXIT, OPTION_COUNT,
};

/// The seed that every world in this benchmark takes.
const SEED: u64 = 0x0123_4567_89ab_cdef;

/// The number of factions that every world in this benchmark holds.
const FACTIONS: u16 = 8;

/// The largest number of samples that one row takes.
const MAX_SAMPLES: usize = 9;

/// The smallest number of samples that one row takes.
const MIN_SAMPLES: usize = 3;

/// The time after which a row stops taking samples, in nanoseconds.
const ROW_BUDGET_NS: u128 = 10_000_000_000;

/// The number of frames that the world runs before anything is measured.
const WARMUP_FRAMES: usize = 2;

/// Reads the clock.
///
/// One lint forbids the clock across this workspace, because a simulation
/// that reads a clock gives an answer that depends on the load of the
/// machine.[^1] A benchmark is the one caller that must read it. The
/// allowance sits on this function alone.
///
/// # References
///
/// [^1]: ADR-0005, decision D1. `docs/adrs/REGISTRY.md`
#[allow(clippy::disallowed_methods)]
fn now() -> Instant {
    Instant::now()
}

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let (width, height) = arguments
        .first()
        .and_then(|word| word.split_once('x'))
        .map_or((512, 512), |(width, height)| {
            (width.parse().unwrap_or(512), height.parse().unwrap_or(512))
        });
    let units: u32 = arguments
        .get(1)
        .and_then(|word| word.parse().ok())
        .unwrap_or(10_000);

    let config = WorldConfig {
        width,
        height,
        seed: SEED,
        faction_count: FACTIONS,
        unit_capacity: units.max(1024),
    };
    let mut world = World::new(config).expect("the extent must describe a world");
    let placed = populate_scattered(&mut world, units);
    for _ in 0..WARMUP_FRAMES {
        world.step(1).expect("the step must run");
    }

    let grid = world.grid();
    let layout = world.bridge().layout();
    let cells = Grid::new(layout.blocks_wide(), layout.blocks_high())
        .expect("the block lattice must describe a grid");
    let tiles = grid.tile_count() as usize;

    println!("# what a tile-indexed exit direction costs, and what it saves");
    println!("# target_triple\t{}", target_triple());
    println!("# tiles\t{tiles}");
    println!("# units_placed\t{placed}");
    println!("# level_1_cells\t{}", cells.tile_count());
    println!("# block_edge\t{}", layout.block_edge());
    println!("# option_count\t{OPTION_COUNT}");
    println!("# tile_array_bytes\t{}", tiles.saturating_mul(OPTION_COUNT));
    println!(
        "# cell_array_bytes\t{}",
        cells.tile_count() as usize * OPTION_COUNT
    );
    println!("# every duration is in nanoseconds");
    println!("bench\tsamples\tmin_ns\tmedian_ns\tmax_ns");

    // The derivation as the engine runs it today.
    let mut field = ExitField::new(cells);
    let pyramid = world.pyramid();
    let samples = samples_of(|| {
        let start = now();
        field.derive(pyramid);
        let elapsed = start.elapsed().as_nanos();
        std::hint::black_box(field.exit(0, 0));
        elapsed
    });
    report("derive_cells", &samples);

    // The added pass, in the two shapes it can take.
    let mut written = vec![NO_EXIT; tiles * OPTION_COUNT];
    let width = grid.width();
    let height = grid.height();
    let blocks_wide = layout.blocks_wide();
    let bits = layout.block_bits();
    let samples = samples_of(|| {
        let start = now();
        write_row_major(&mut written, &field, width, height, blocks_wide, bits);
        let elapsed = start.elapsed().as_nanos();
        std::hint::black_box(written[0]);
        elapsed
    });
    report("tile_write_row_major", &samples);

    let samples = samples_of(|| {
        let start = now();
        write_block_major(&mut written, &field, grid, layout.block_edge(), blocks_wide);
        let elapsed = start.elapsed().as_nanos();
        std::hint::black_box(written[0]);
        elapsed
    });
    report("tile_write_block_major", &samples);

    // The units the movement pass would read, in the order it reads them.
    //
    // **Every live unit is read, not only the units that hold an intent.**
    // The movement pass skips a unit with no intent, and the choice schedule
    // leaves most units without one in any single frame. A row over the
    // holders alone would price the change against a fraction of the
    // population and understate what it saves. The row below therefore reads
    // the whole population, which is the most the change can ever reach.
    let soldiers = world.soldiers();
    let mut work: Vec<(u32, u8)> = Vec::new();
    let mut holders = 0usize;
    for soldier in soldiers.iter() {
        let Some(tile) = soldiers.tile(soldier) else {
            continue;
        };
        let option = match soldiers.intent(soldier) {
            Some(Some(option)) => {
                holders += 1;
                option
            }
            // A unit with no intent still reads one entry here, so that the
            // row covers the whole population. The option is taken from the
            // tile, so the read pattern stays the pattern of a real unit.
            _ => (tile.0 % OPTION_COUNT as u32) as u8,
        };
        work.push((tile.0, option));
    }
    println!("# units_read\t{}", work.len());
    println!("# units_holding_an_intent\t{holders}");
    if work.is_empty() {
        println!("# no unit is live, so the two lookup rows say nothing");
        return;
    }

    let samples = samples_of(|| {
        let start = now();
        let mut sum = 0u64;
        for (tile, option) in &work {
            let direction = layout
                .key_of(TileIdx(*tile))
                .and_then(|key| field.exit(layout.block_of_key(key), *option))
                .flatten()
                .unwrap_or(NO_EXIT);
            sum += u64::from(direction);
        }
        let elapsed = start.elapsed().as_nanos();
        std::hint::black_box(sum);
        elapsed
    });
    report("lookup_through_the_cell", &samples);

    let samples = samples_of(|| {
        let start = now();
        let mut sum = 0u64;
        for (tile, option) in &work {
            let direction = written[*tile as usize * OPTION_COUNT + *option as usize];
            sum += u64::from(direction);
        }
        let elapsed = start.elapsed().as_nanos();
        std::hint::black_box(sum);
        elapsed
    });
    report("lookup_at_the_tile", &samples);

    // The second item asks a narrower question. It leaves the exit field
    // where it is, and it stores the cell of a unit beside the tile of the
    // unit, so that a pass reads the cell rather than deriving it. The two
    // rows below hold the downstream read fixed and change only how the cell
    // is reached, so the difference is the key-and-block arithmetic and
    // nothing else.[^1]
    //
    // [^1]: Backlog item 0268, hold the cell index on the unit. `docs/backlog/proposed/0268-hold-the-cell-index-on-the-unit.md`
    let stored: Vec<u32> = work
        .iter()
        .map(|(tile, _)| {
            layout
                .key_of(TileIdx(*tile))
                .map_or(0, |key| layout.block_of_key(key))
        })
        .collect();

    let samples = samples_of(|| {
        let start = now();
        let mut sum = 0u64;
        for (tile, option) in &work {
            let cell = layout
                .key_of(TileIdx(*tile))
                .map_or(0, |key| layout.block_of_key(key));
            sum += u64::from(field.exit(cell, *option).flatten().unwrap_or(NO_EXIT));
        }
        let elapsed = start.elapsed().as_nanos();
        std::hint::black_box(sum);
        elapsed
    });
    report("cell_derived_from_the_tile", &samples);

    let samples = samples_of(|| {
        let start = now();
        let mut sum = 0u64;
        for ((_, option), cell) in work.iter().zip(stored.iter()) {
            sum += u64::from(field.exit(*cell, *option).flatten().unwrap_or(NO_EXIT));
        }
        let elapsed = start.elapsed().as_nanos();
        std::hint::black_box(sum);
        elapsed
    });
    report("cell_read_from_a_column", &samples);

    // Where the difference between the two rows above goes. The conversion
    // from a tile index to an address is a remainder and a quotient by the
    // world width, and the width is not known when the crate is compiled, so
    // both are a hardware division. This row holds that step alone.
    let samples = samples_of(|| {
        let start = now();
        let mut sum = 0u64;
        for (tile, _) in &work {
            let address = grid.address_of(TileIdx(*tile)).unwrap_or(Axial::new(0, 0));
            sum += (address.q as u64) ^ (address.r as u64);
        }
        let elapsed = start.elapsed().as_nanos();
        std::hint::black_box(sum);
        elapsed
    });
    report("address_of_the_tile_alone", &samples);
}

/// Fills the tile-indexed array by walking the world one row at a time.
///
/// The cell of a tile is a shift and a multiply, so no lookup is needed, and
/// the writes run in ascending tile order.
fn write_row_major(
    written: &mut [u8],
    field: &ExitField,
    width: u32,
    height: u32,
    blocks_wide: u32,
    bits: u32,
) {
    let mut at = 0usize;
    for row in 0..height {
        let row_block = (row >> bits) * blocks_wide;
        for column in 0..width {
            let cell = row_block + (column >> bits);
            for option in 0..OPTION_COUNT {
                written[at + option] = field.exit(cell, option as u8).flatten().unwrap_or(NO_EXIT);
            }
            at += OPTION_COUNT;
        }
    }
}

/// Fills the tile-indexed array by walking one block at a time.
///
/// The cell entry is read once for each block, and the writes run in runs of
/// one block edge.
fn write_block_major(
    written: &mut [u8],
    field: &ExitField,
    grid: Grid,
    edge: u32,
    blocks_wide: u32,
) {
    let blocks = blocks_wide * grid.height().div_ceil(edge);
    for block in 0..blocks {
        let mut directions = [NO_EXIT; OPTION_COUNT];
        for (option, slot) in directions.iter_mut().enumerate() {
            *slot = field.exit(block, option as u8).flatten().unwrap_or(NO_EXIT);
        }
        let first_column = (block % blocks_wide) * edge;
        let first_row = (block / blocks_wide) * edge;
        for row in first_row..first_row + edge {
            for column in first_column..first_column + edge {
                let Some(tile) = grid.index_of(Axial::new(column as i32, row as i32)) else {
                    continue;
                };
                let at = tile.0 as usize * OPTION_COUNT;
                written[at..at + OPTION_COUNT].copy_from_slice(&directions);
            }
        }
    }
}

/// Places units across the world at an even stride.
fn populate_scattered(world: &mut World, units: u32) -> u32 {
    if units == 0 {
        return 0;
    }
    let grid = world.grid();
    let width = grid.width();
    let count = grid.tile_count();
    let stride = (count / units).max(1);
    let ceiling = u32::from(world.config().faction_count.max(1));
    let mut placed = 0u32;
    let mut cursor = 0u32;
    for step in 0..units {
        cursor = cursor.max(step.saturating_mul(stride));
        while cursor < count {
            let address = Axial::new((cursor % width) as i32, (cursor / width) as i32);
            cursor += 1;
            if !world.admits_a_unit(address) {
                continue;
            }
            let faction = FactionId((placed % ceiling) as u16);
            if world.spawn_soldier(address, faction).is_ok() {
                placed += 1;
                break;
            }
        }
        if cursor >= count {
            break;
        }
    }
    placed
}

/// Takes samples until the count or the budget runs out.
fn samples_of<F: FnMut() -> u128>(mut once: F) -> Vec<u128> {
    let mut samples = Vec::with_capacity(MAX_SAMPLES);
    let mut spent = 0u128;
    while samples.len() < MAX_SAMPLES {
        let elapsed = once();
        spent += elapsed;
        samples.push(elapsed);
        if samples.len() >= MIN_SAMPLES && spent >= ROW_BUDGET_NS {
            break;
        }
    }
    samples
}

/// Writes one row of the table.
fn report(name: &str, samples: &[u128]) {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let min = sorted.first().copied().unwrap_or(0);
    let max = sorted.last().copied().unwrap_or(0);
    let median = sorted.get(sorted.len() / 2).copied().unwrap_or(0);
    println!("{name}\t{}\t{min}\t{median}\t{max}", sorted.len());
}

/// Returns the triple that this binary was built for.
fn target_triple() -> String {
    format!(
        "{}-{}-{}",
        std::env::consts::ARCH,
        std::env::consts::FAMILY,
        std::env::consts::OS
    )
}
