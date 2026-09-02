//! Level 1 of the pyramid equals the level below it.
//!
//! Level 0 is the only source of truth, and a level 1 cell is the exact
//! combination of the tiles it covers.[^1] [^2] The equality is exact and not
//! approximate, and it is a test rather than a comment.[^3]
//!
//! **An intensive reading is the division of two extensive fields.** The
//! weighting is therefore automatic: a cell that covers four hundred tiles
//! contributes four hundred to the denominator.[^4] The test that matters
//! most here combines two cells of different extents and asserts that the
//! reading is the weighted answer and not the mean of the two means. That
//! defect gives a number of the right type, in a plausible range, that moves
//! plausibly as the world changes.
//!
//! The tests see only the public crate API.[^5]
//!
//! # References
//!
//! [^1]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D1. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
//! [^2]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D2. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
//! [^3]: ADR-0023, an aggregate combines exactly, in any order, decision D5. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
//! [^4]: ADR-0024, every summary field is declared extensive or intensive, decision D3. `docs/adrs/accepted/adr-0024-every-summary-field-is-declared-extensive-or-intensive.md`
//! [^5]: Testing policy. `docs/TESTING.md`

use cachette_core::pyramid::CellSummary;
use cachette_core::resource::ResourceKind;
use cachette_core::{Axial, FactionId, Fix32, World, WorldConfig};
use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;

/// The thread counts that the pyramid tests run at.
const THREAD_COUNTS: [usize; 3] = [1, 2, 12];

/// The extent of the fixture world.
///
/// The extent is not a multiple of the block edge, so the world has cells at
/// its edge that cover fewer tiles than a full block. Those cells are the
/// case that an unweighted reading gets wrong, and a world whose extent
/// divides the block edge would supply none.
const EXTENT: u32 = 100;

/// The seed of the fixture world.
const SEED: u64 = 0x0cac_4e77_0042;

/// Builds a world and puts soldiers on the open ground of it.
fn peopled(seed: u64) -> World {
    let mut world = World::new(WorldConfig {
        width: EXTENT,
        height: EXTENT,
        seed,
        faction_count: 3,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    })
    .expect("the extent describes a world");
    let grid = world.grid();
    let open: Vec<Axial> = (0..grid.tile_count())
        .map(|index| Axial::new((index % grid.width()) as i32, (index / grid.width()) as i32))
        .filter(|address| world.admits_a_unit(*address))
        .collect();
    assert!(open.len() > 64, "the seed left {} open tiles", open.len());
    for (ordinal, address) in open.iter().enumerate().step_by(7) {
        world
            .spawn_soldier(*address, FactionId((ordinal % 3) as u16))
            .expect("the open tile admits a unit");
    }
    world.rebuild_pyramid(1).expect("the rebuild must succeed");
    world
}

/// Asserts that a stored cell equals the recomputation of its block.
fn assert_cell_matches(world: &World, block: u32) {
    let layout = world.pyramid().layout();
    let grid = layout.grid();
    let edge = layout.block_edge();
    let first_column = (block % layout.blocks_wide()) * edge;
    let first_row = (block / layout.blocks_wide()) * edge;

    let (mut tiles, mut open, mut units) = (0i64, 0i64, 0i64);
    let (mut value_total, mut height_total) = (0i64, 0i64);
    let mut food_total = 0i64;
    for row in first_row..first_row + edge {
        for column in first_column..first_column + edge {
            let address = Axial::new(column as i32, row as i32);
            if grid.index_of(address).is_none() {
                continue;
            }
            let Some(ground) = world.tile_terrain(address) else {
                continue;
            };
            tiles += 1;
            open += i64::from(world.admits_a_unit(address));
            units += world
                .soldier_count_on(address)
                .expect("the structure answers") as i64;
            value_total += i64::from(world.tile_value(address).expect("the tile has a value").0);
            height_total += i64::from(ground.height.0);
            // The recomputation reads what the tile still holds, which is
            // what the generator gave it less what the ledger says was taken.
            // A summary that held the original stock instead would agree with
            // this until somebody gathered.
            food_total += i64::from(
                world
                    .tile_stock(address, ResourceKind::Food)
                    .expect("the tile is inside the world")
                    .0,
            );
        }
    }

    let cell = world.pyramid().cell(block).expect("the block names a cell");
    assert_eq!(cell.tiles(), tiles, "block {block}: the tile count differs");
    assert_eq!(
        cell.open_tiles(),
        open,
        "block {block}: the open tile count differs"
    );
    assert_eq!(cell.units(), units, "block {block}: the unit count differs");
    assert_eq!(
        cell.value_total().0,
        value_total,
        "block {block}: the value total differs"
    );
    assert_eq!(
        cell.height_total().0,
        height_total,
        "block {block}: the height total differs"
    );
    assert_eq!(
        cell.food_total().0,
        food_total,
        "block {block}: the food total differs"
    );
}

#[test]
fn every_cell_equals_the_tiles_it_covers() {
    // This is the equality that ADR-0022 D2 states and ADR-0023 D5 requires a
    // test for. The recomputation reads level 0 through the public interface
    // and never asks the pyramid, so the two answers are independent.
    let world = peopled(SEED);
    let count = world.pyramid().len() as u32;
    assert!(count > 4, "the world holds only {count} cells");
    for block in 0..count {
        assert_cell_matches(&world, block);
    }
}

#[test]
fn the_fixture_holds_a_cell_that_covers_fewer_tiles_than_a_block() {
    // An edge cell is the case an unweighted reading gets wrong. A world
    // whose extent divides the block edge supplies none, and every weighting
    // assertion below would then pass on cells of equal extent.
    let world = peopled(SEED);
    let full = world.pyramid().layout().block_edge() as i64
        * i64::from(world.pyramid().layout().block_edge());
    let short = world
        .pyramid()
        .cells()
        .iter()
        .filter(|cell| cell.tiles() > 0 && cell.tiles() < full)
        .count();
    assert!(
        short > 0,
        "every cell covers a full block, so the weighting is untested"
    );
}

#[test]
fn the_whole_world_is_the_combination_of_its_cells() {
    // The total is what level 2 would hold for a world of one region. It must
    // equal a sweep of level 0.
    let world = peopled(SEED);
    let total = world.pyramid().total();

    let grid = world.grid();
    let mut tiles = 0i64;
    let mut units = 0i64;
    for index in 0..grid.tile_count() {
        let address = Axial::new((index % grid.width()) as i32, (index / grid.width()) as i32);
        tiles += 1;
        units += world
            .soldier_count_on(address)
            .expect("the structure answers") as i64;
    }
    assert_eq!(total.tiles(), tiles, "the world covers a different extent");
    assert_eq!(
        total.units(),
        i64::from(world.soldiers().len()),
        "the level counts a different population from the arena"
    );
    assert_eq!(units, i64::from(world.soldiers().len()));
}

#[test]
fn an_intensive_reading_is_weighted_by_the_ground_it_covers() {
    // The defect this catches: combining two means rather than the sums they
    // came from. It gives a number of the right type in a plausible range.
    //
    // Two cells of different extents, whose mean heights differ. The mean of
    // the combination must be the weighted answer, and it must differ from
    // the unweighted average of the two means.
    let world = peopled(SEED);
    let full = i64::from(world.pyramid().layout().block_edge())
        * i64::from(world.pyramid().layout().block_edge());

    let wide = *world
        .pyramid()
        .cells()
        .iter()
        .find(|cell| cell.tiles() == full)
        .expect("the world holds a cell that covers a whole block");
    let narrow = *world
        .pyramid()
        .cells()
        .iter()
        .filter(|cell| cell.tiles() > 0 && cell.tiles() * 2 < full)
        .max_by_key(|cell| {
            // The pair must differ in mean height, or the weighted answer
            // and the unweighted one agree and the test proves nothing.
            let mine = cell.mean_height().map_or(0, |height| i64::from(height.0));
            let theirs = wide.mean_height().map_or(0, |height| i64::from(height.0));
            (mine - theirs).abs()
        })
        .expect("the world holds a cell that covers part of a block");

    let combined = wide.combine(narrow);
    assert_eq!(
        combined.tiles(),
        wide.tiles() + narrow.tiles(),
        "the extents did not add"
    );

    let weighted = Fix32(
        ((wide.height_total().0 + narrow.height_total().0) / (wide.tiles() + narrow.tiles()))
            as i32,
    );
    assert_eq!(
        combined.mean_height(),
        Some(weighted),
        "the reading is not the weighted mean of the two cells"
    );

    let wide_mean = i64::from(wide.mean_height().expect("the cell covers tiles").0);
    let narrow_mean = i64::from(narrow.mean_height().expect("the cell covers tiles").0);
    assert_ne!(
        wide_mean, narrow_mean,
        "the two cells have one mean height, so the weighting is untested"
    );
    let unweighted = (wide_mean + narrow_mean) / 2;
    assert_ne!(
        i64::from(weighted.0),
        unweighted,
        "the weighted answer equals the mean of the means, so this pair \
         cannot tell the two apart"
    );
}

/// The seed of the world that the food tests read.
///
/// The seed was chosen by a scan, because the food total needs a spread that
/// the general fixture does not hold. Its world holds a cell that covers a
/// whole block and carries no food at all, and a cell that covers a whole
/// block and carries the most food in the world. A fixture whose cells all
/// carry a middling amount would let a food total that read the wrong tiles
/// pass.[^1]
///
/// # References
///
/// [^1]: Testing rules, section 2a. `.claude/rules/testing.md`
const FOOD_SEED: u64 = 23;

/// Returns an open address whose tile still holds food.
fn a_food_deposit(world: &World) -> Axial {
    let grid = world.grid();
    (0..grid.tile_count())
        .map(|index| Axial::new((index % grid.width()) as i32, (index / grid.width()) as i32))
        .find(|address| {
            world.admits_a_unit(*address)
                && world
                    .tile_stock(*address, ResourceKind::Food)
                    .is_some_and(|stock| stock.0 > 0)
        })
        .expect("the fixture world holds a food deposit under open ground")
}

#[test]
fn the_food_fixture_holds_a_cell_with_no_food_and_a_cell_with_the_most() {
    // A defect lives at an extreme of the distribution. A world whose cells
    // all carry a middling amount of food supplies no extreme, so the
    // recomputation below would never meet an empty cell or a full one.[^1]
    //
    // [^1]: Testing rules, section 2a. `.claude/rules/testing.md`
    let world = peopled(FOOD_SEED);
    let full = i64::from(world.pyramid().layout().block_edge())
        * i64::from(world.pyramid().layout().block_edge());
    let whole: Vec<&CellSummary> = world
        .pyramid()
        .cells()
        .iter()
        .filter(|cell| cell.tiles() == full)
        .collect();
    let lowest = whole
        .iter()
        .map(|cell| cell.food_total().0)
        .min()
        .expect("the world holds a cell that covers a whole block");
    let highest = whole
        .iter()
        .map(|cell| cell.food_total().0)
        .max()
        .expect("the world holds a cell that covers a whole block");
    assert_eq!(
        lowest, 0,
        "no whole cell of the fixture is empty of food, so the low end is untested"
    );
    assert!(
        highest > 512,
        "the fixture holds no food contrast: the fullest cell carries {highest}"
    );
}

#[test]
fn every_cell_of_the_food_fixture_equals_the_tiles_it_covers() {
    // The recomputation reads the remaining stock of each tile through the
    // public interface and never asks the pyramid, so the two answers are
    // independent.[^1]
    //
    // [^1]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D2. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
    let world = peopled(FOOD_SEED);
    for block in 0..world.pyramid().len() as u32 {
        assert_cell_matches(&world, block);
    }
}

#[test]
fn a_gather_lowers_the_food_of_the_cell_it_took_from() {
    // The food total is the stock the tiles still hold, and not the stock the
    // ground generated. A summary that held the original stock agrees with a
    // tile reader until somebody gathers, and this is the case that tells the
    // two apart.[^1]
    //
    // The test drives the engine. A rebuild that no step reaches would leave
    // the summary correct and unreachable.[^2]
    //
    // [^1]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D4. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
    // [^2]: Testing rules, section 5. `.claude/rules/testing.md`
    let mut world = peopled(FOOD_SEED);
    let address = a_food_deposit(&world);
    let before = world
        .summary_covering(address)
        .expect("the address is inside the world")
        .food_total()
        .0;

    let unit = world
        .spawn_soldier(address, FactionId(0))
        .expect("the ground admits a unit");
    assert!(world.order_gather(unit, ResourceKind::Food));
    world.step(2).expect("the step must run");

    let taken: i64 = world
        .gather_log()
        .iter()
        .map(|event| i64::from(event.amount))
        .sum();
    assert!(
        taken > 0,
        "the unit took nothing, so the test proves nothing"
    );

    // The unit may have moved after it gathered, so the cell that lost the
    // food is the cell that covers the tile the unit took from.
    let after = world
        .summary_covering(address)
        .expect("the address is inside the world")
        .food_total()
        .0;
    assert_eq!(
        after,
        before - taken,
        "the cell kept the food that a unit carried away"
    );
    assert!(world.check_invariants(), "the world broke an invariant");
}

/// Returns an open address whose tile still holds a stock of one kind.
fn a_deposit_of(world: &World, kind: ResourceKind) -> Axial {
    let grid = world.grid();
    (0..grid.tile_count())
        .map(|index| Axial::new((index % grid.width()) as i32, (index / grid.width()) as i32))
        .find(|address| {
            world.admits_a_unit(*address)
                && world
                    .tile_stock(*address, kind)
                    .is_some_and(|stock| stock.0 > 0)
        })
        .expect("the fixture world holds a deposit of that kind under open ground")
}

#[test]
fn a_gather_of_another_kind_leaves_the_food_of_the_cell_alone() {
    // The ledger holds one entry for each tile and each kind, and the food
    // total reads the food entries alone. A total that summed every entry of
    // the run would fall when a unit took wood, and the food test above
    // cannot see that, because it gathers food.
    let mut world = peopled(FOOD_SEED);
    let address = a_deposit_of(&world, ResourceKind::Wood);
    let before = world
        .summary_covering(address)
        .expect("the address is inside the world")
        .food_total()
        .0;

    let unit = world
        .spawn_soldier(address, FactionId(0))
        .expect("the ground admits a unit");
    assert!(world.order_gather(unit, ResourceKind::Wood));
    world.step(2).expect("the step must run");

    let taken: i64 = world
        .gather_log()
        .iter()
        .map(|event| i64::from(event.amount))
        .sum();
    assert!(
        taken > 0,
        "the unit took nothing, so the test proves nothing"
    );
    assert_eq!(
        world
            .summary_covering(address)
            .expect("the address is inside the world")
            .food_total()
            .0,
        before,
        "the wood a unit took came off the food of the cell"
    );
}

#[test]
fn the_mean_food_divides_by_every_tile_and_not_by_the_open_ground() {
    // The ground gives every tile a food stock, and water holds a stock of
    // zero. The denominator is therefore the tile count. A mean that borrowed
    // the open tile count would report a richer cell than the ground carries,
    // and it would report a number of the right type in a plausible
    // range.[^1]
    //
    // [^1]: ADR-0024, every summary field is declared extensive or intensive, decision D4. `docs/adrs/accepted/adr-0024-every-summary-field-is-declared-extensive-or-intensive.md`
    let world = peopled(FOOD_SEED);
    let mixed = *world
        .pyramid()
        .cells()
        .iter()
        .filter(|cell| cell.open_tiles() > 0 && cell.open_tiles() < cell.tiles())
        .max_by_key(|cell| cell.food_total().0)
        .expect("the fixture holds a cell that covers water and dry ground");
    assert!(
        mixed.food_total().0 > 0,
        "the cell carries no food, so both denominators give zero"
    );

    let over_every_tile = (mixed.food_total().0 << 16) / mixed.tiles();
    let over_the_open_ground = (mixed.food_total().0 << 16) / mixed.open_tiles();
    assert_ne!(
        over_every_tile, over_the_open_ground,
        "the cell holds no water, so the two denominators agree and this \
         pair cannot tell them apart"
    );
    assert_eq!(
        i64::from(mixed.mean_food().expect("the cell covers tiles").0),
        over_every_tile,
        "the mean food divided by the wrong extent"
    );
}

#[test]
fn a_summary_over_no_tile_reports_no_reading() {
    // A mean over nothing is not zero. Reporting it as zero gives a caller an
    // answer it cannot tell from a true one.
    let empty = CellSummary::IDENTITY;
    assert_eq!(empty.tiles(), 0);
    assert_eq!(empty.mean_value(), None);
    assert_eq!(empty.mean_food(), None);
    assert_eq!(empty.mean_height(), None);
    assert_eq!(empty.open_share(), None);
    assert_eq!(empty.units_for_each_open_tile(), None);
}

#[test]
fn the_identity_leaves_a_summary_unchanged() {
    let world = peopled(SEED);
    let cell = world.pyramid().cell(0).expect("the world holds a cell");
    assert_eq!(cell.combine(CellSummary::IDENTITY), cell);
    assert_eq!(CellSummary::IDENTITY.combine(cell), cell);
}

#[test]
fn removing_a_summary_undoes_combining_it() {
    // The combine operation has an inverse, which is what would permit
    // repairing a cell rather than rebuilding it. Nothing takes that path
    // yet, and the property is what makes it available.
    let world = peopled(SEED);
    let first = world.pyramid().cell(0).expect("the world holds a cell");
    let second = world.pyramid().cell(1).expect("the world holds two cells");
    assert_eq!(first.combine(second).remove(second), first);
    assert_eq!(first.combine(second).remove(first), second);
}

#[test]
fn the_accumulator_is_wider_than_a_level_0_field() {
    // A tile field summed over the tile count of the target world overflows a
    // 32-bit accumulator. The level 1 accumulator is 64 bits, and this is the
    // test that says so rather than a comment.
    let world = peopled(SEED);
    let cell = *world
        .pyramid()
        .cells()
        .iter()
        .max_by_key(|cell| cell.tiles())
        .expect("the world holds a cell");
    assert!(cell.tiles() > 0);

    // The fold doubles, so it reaches the ceiling of a 32-bit field in a few
    // steps rather than in four million.
    let mut total = cell;
    let mut doublings = 0u32;
    while total.tiles() <= i64::from(u32::MAX) {
        total = total.combine(total);
        doublings += 1;
        assert!(doublings < 40, "the fold did not reach the ceiling");
    }
    assert_eq!(
        total.tiles(),
        cell.tiles() << doublings,
        "the tile count wrapped or saturated before the ceiling"
    );
    assert_eq!(
        total.height_total().0,
        cell.height_total().0 << doublings,
        "the height accumulator wrapped or saturated before the ceiling"
    );
    assert!(
        total.tiles() > i64::from(u32::MAX),
        "the accumulator stopped at the width of the field it sums"
    );
}

#[test]
fn the_level_does_not_depend_on_the_thread_count() {
    // The rebuild itself runs on one thread, so rebuilding twice and
    // comparing would compare a run against itself and prove nothing. The
    // level is a function of level 0, and level 0 is stepped in parallel, so
    // the comparison that means something drives the step at each thread
    // count and reads the level it produced.
    let expected: Vec<CellSummary> = {
        let mut world = peopled(SEED);
        for _ in 0..6 {
            world.step(THREAD_COUNTS[0]).expect("the step must run");
        }
        world.pyramid().cells().to_vec()
    };
    for threads in &THREAD_COUNTS[1..] {
        let mut world = peopled(SEED);
        for _ in 0..6 {
            world.step(*threads).expect("the step must run");
        }
        assert_eq!(
            world.pyramid().cells(),
            expected.as_slice(),
            "the level differs at {threads} threads"
        );
    }
}

#[test]
fn the_engine_rebuilds_the_level_at_the_barrier() {
    // A capability nothing invokes ships inert. The engine is obligated to
    // maintain this level, so the test drives the engine and then reads the
    // level, rather than calling the rebuild itself.
    let mut world = peopled(SEED);
    let before: Vec<i64> = world
        .pyramid()
        .cells()
        .iter()
        .map(|cell| cell.units())
        .collect();

    let mut moved = false;
    for _ in 0..8 {
        world.step(2).expect("the step must run");
        let after: Vec<i64> = world
            .pyramid()
            .cells()
            .iter()
            .map(|cell| cell.units())
            .collect();
        if after != before {
            moved = true;
        }
        // Whatever moved, every cell still equals the tiles it covers.
        for block in 0..world.pyramid().len() as u32 {
            assert_cell_matches(&world, block);
        }
        assert!(world.check_invariants());
        assert_eq!(
            world.pyramid().total().units(),
            i64::from(world.soldiers().len()),
            "the level lost a unit that the arena still holds"
        );
    }
    assert!(
        moved,
        "eight frames moved no unit between cells, so the barrier rebuild is untested"
    );
}

proptest! {
    #![proptest_config(ProptestConfig {
        failure_persistence: Some(Box::new(FileFailurePersistence::Direct(
            concat!(env!("CARGO_MANIFEST_DIR"), "/tests/pyramid.proptest-regressions"),
        ))),
        cases: 24,
        ..ProptestConfig::default()
    })]

    /// The combination of a set does not depend on how the set was grouped.
    ///
    /// This is exact associativity, stated directly. A combine that was
    /// associative only to within a rounding error would fail it.
    #[test]
    fn the_combination_does_not_depend_on_the_grouping(
        seed in any::<u64>(),
        split in 1usize..8,
    ) {
        let world = peopled(seed % 64);
        let cells = world.pyramid().cells();
        prop_assume!(cells.len() > split);

        let whole = cells
            .iter()
            .fold(CellSummary::IDENTITY, |total, cell| total.combine(*cell));

        let (head, tail) = cells.split_at(split);
        let left = head
            .iter()
            .fold(CellSummary::IDENTITY, |total, cell| total.combine(*cell));
        let right = tail
            .iter()
            .fold(CellSummary::IDENTITY, |total, cell| total.combine(*cell));
        prop_assert_eq!(left.combine(right), whole);
    }

    /// The combination of a set does not depend on the order the values
    /// arrived in.
    #[test]
    fn the_combination_does_not_depend_on_the_order(seed in any::<u64>()) {
        let world = peopled(seed % 64);
        let cells = world.pyramid().cells();

        let forward = cells
            .iter()
            .fold(CellSummary::IDENTITY, |total, cell| total.combine(*cell));
        let backward = cells
            .iter()
            .rev()
            .fold(CellSummary::IDENTITY, |total, cell| total.combine(*cell));
        prop_assert_eq!(forward, backward);
    }
}
