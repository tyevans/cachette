//! A unit chooses an action by scoring a fixed option set.
//!
//! The tests here drive the engine. The choice runs inside the step, so a
//! test that called the scoring function directly would prove that the
//! function works and not that anything reaches it.[^1]
//!
//! Each fixture builds the world that produces the value under test. None of
//! them copies the world of the demonstration binary, because that world is
//! chosen to look right and not to produce an extreme.[^2]
//!
//! # References
//!
//! [^1]: Testing rules, section 5. `.claude/rules/testing.md`
//! [^2]: Testing rules, section 2a. `.claude/rules/testing.md`

use cachette_core::choose::{self, ChoiceSchedule, SCORE_FLOOR};
use cachette_core::cohort::{NeedRule, NEED_FULL};
use cachette_core::resource::ResourceKind;
use cachette_core::terrain::TileKind;
use cachette_core::{Axial, Entity, FactionId, Fix32, World, WorldConfig, NO_INTENT};

/// The extent of the one-cell world.
///
/// A level 1 cell covers a square of tiles, and this extent is exactly one
/// of them. A unit in this world cannot leave its cell, so a test may move
/// the population and still read one summary.
const ONE_CELL_EXTENT: u32 = 32;

/// The extent of the many-cell world.
const MANY_CELL_EXTENT: u32 = 256;

/// The seed of every fixture world.
///
/// The seed was chosen because its worlds hold the extremes these tests
/// need: a cell whose tiles all admit a unit, and a set of cells whose mean
/// heights differ by a factor of eight. Each test asserts the extreme it
/// depends on, so a change to the generator fails the fixture rather than
/// the assertion.
const SEED: u64 = 7;

/// Builds a world of one level 1 cell, with the choice on every tick.
fn one_cell_world() -> World {
    let mut world = World::new(WorldConfig {
        width: ONE_CELL_EXTENT,
        height: ONE_CELL_EXTENT,
        seed: SEED,
        faction_count: 2,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    })
    .expect("the extent must describe a world");
    world
        .set_choice_schedule(0)
        .expect("the exponent is inside the range");
    assert_eq!(
        world.pyramid().len(),
        1,
        "the fixture must hold exactly one cell"
    );
    world
}

/// Builds a world of many level 1 cells, with the choice on every tick.
fn many_cell_world() -> World {
    let mut world = World::new(WorldConfig {
        width: MANY_CELL_EXTENT,
        height: MANY_CELL_EXTENT,
        seed: SEED,
        faction_count: 2,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    })
    .expect("the extent must describe a world");
    world
        .set_choice_schedule(0)
        .expect("the exponent is inside the range");
    assert!(
        world.pyramid().len() > 8,
        "the fixture must hold many cells"
    );
    world
}

/// Returns every address of a world, in index order.
fn addresses(world: &World) -> Vec<Axial> {
    let grid = world.grid();
    (0..grid.tile_count())
        .map(|index| Axial::new((index % grid.width()) as i32, (index / grid.width()) as i32))
        .collect()
}

/// Returns the level 1 cell that covers one address.
fn cell_of(world: &World, address: Axial) -> u32 {
    let layout = world.pyramid().layout();
    let tile = world
        .grid()
        .index_of(address)
        .expect("the address is inside the world");
    layout.block_of_key(layout.key_of(tile).expect("the tile is inside the world"))
}

/// Returns one open address inside each cell that holds open ground.
fn one_open_address_of_each_cell(world: &World) -> Vec<(u32, Axial)> {
    let mut found: Vec<(u32, Axial)> = Vec::new();
    for address in addresses(world) {
        if !world.admits_a_unit(address) {
            continue;
        }
        let cell = cell_of(world, address);
        if !found.iter().any(|(known, _)| *known == cell) {
            found.push((cell, address));
        }
    }
    found.sort_unstable_by_key(|(cell, _)| *cell);
    found
}

/// Puts a unit on every open tile of a world, up to the capacity of each.
fn crowd(world: &mut World) -> Vec<Entity> {
    let mut placed = Vec::new();
    for address in addresses(world) {
        if !world.admits_a_unit(address) {
            continue;
        }
        let capacity = world.tile_kind(address).map_or(0, TileKind::capacity);
        for ordinal in 0..capacity {
            if let Ok(unit) = world.spawn_soldier(address, FactionId((ordinal % 2) as u16)) {
                placed.push(unit);
            }
        }
    }
    placed
}

/// Puts one unit on every open tile of a stride, and leaves room to move.
///
/// A world filled to capacity refuses every move, whatever any unit chose.
/// A test about movement therefore needs room, and this fixture leaves it.
fn populate(world: &mut World, stride: usize) -> Vec<Entity> {
    let mut placed = Vec::new();
    let open: Vec<Axial> = addresses(world)
        .into_iter()
        .filter(|address| world.admits_a_unit(*address))
        .collect();
    for (ordinal, address) in open.into_iter().enumerate() {
        if ordinal % stride != 0 {
            continue;
        }
        if let Ok(unit) = world.spawn_soldier(address, FactionId((ordinal % 2) as u16)) {
            placed.push(unit);
        }
    }
    placed
}

/// Puts every weight at zero, then gives one option the weight given.
fn only(world: &mut World, option: u8, weight: Fix32) {
    for index in 0..choose::OPTION_COUNT as u8 {
        world
            .set_option_weight(index, Fix32::ZERO)
            .expect("the index is inside the set");
    }
    world
        .set_option_weight(option, weight)
        .expect("the index is inside the set");
}

#[test]
fn a_unit_chooses_by_reading_the_world() {
    // A choice that ignores the world gives every unit the same answer,
    // because every unit here carries the same need and the same weights.
    // The only thing that differs is the ground under each of them.
    let mut world = many_cell_world();
    let cells = one_open_address_of_each_cell(&world);
    assert!(cells.len() > 8, "the fixture must reach many cells");
    let units: Vec<Entity> = cells
        .iter()
        .map(|(_, address)| {
            world
                .spawn_soldier(*address, FactionId(0))
                .expect("the open tile admits a unit")
        })
        .collect();

    world.step(2).expect("the step must run");

    let mut intents: Vec<Option<u8>> = units
        .iter()
        .map(|unit| world.soldier_intent(*unit).expect("nothing despawned it"))
        .collect();
    intents.sort_unstable();
    intents.dedup();
    assert!(
        intents.len() > 1,
        "every unit chose the same option, so the choice read nothing: {intents:?}"
    );
}

#[test]
fn a_unit_responds_to_the_ground() {
    // The fixture puts one unit on the highest ground of the world and one
    // on the lowest, and gives weight to the option that reads the height
    // alone. The weight sits between the two, so the ground alone decides
    // which unit acts and which holds.
    let mut world = many_cell_world();
    let cells = one_open_address_of_each_cell(&world);
    let mut heights: Vec<(Fix32, Axial)> = cells
        .iter()
        .map(|(cell, address)| {
            (
                world
                    .pyramid()
                    .cell(*cell)
                    .expect("the cell is inside the level")
                    .mean_height()
                    .expect("the cell holds tiles"),
                *address,
            )
        })
        .collect();
    heights.sort_unstable_by_key(|(height, _)| *height);
    let (low, low_address) = heights[0];
    let (high, high_address) = heights[heights.len() - 1];
    assert!(
        high.0 >= low.0 * 2,
        "the fixture holds no height contrast: {low:?} against {high:?}"
    );

    // The weight puts the floor between the two heights. The arithmetic is
    // exact, so the middle is a whole number of the fixed-point scale.
    let middle = i64::from(low.0 + high.0) / 2;
    let weight = Fix32(((i64::from(SCORE_FLOOR.0) << 16) / middle) as i32);
    // The option that reads the mean height is the third of the set.
    let climb = 2u8;
    only(&mut world, climb, weight);

    let on_high = world
        .spawn_soldier(high_address, FactionId(0))
        .expect("the open tile admits a unit");
    let on_low = world
        .spawn_soldier(low_address, FactionId(0))
        .expect("the open tile admits a unit");
    world.step(2).expect("the step must run");

    assert_eq!(
        world.soldier_intent(on_high).expect("alive"),
        Some(climb),
        "the unit on the high ground found nothing"
    );
    assert_eq!(
        world.soldier_intent(on_low).expect("alive"),
        None,
        "the unit on the low ground acted on ground that is not there"
    );
}

#[test]
fn a_unit_responds_to_another_unit() {
    // The fixture changes one value of the world: how many units stand in
    // the cell. Nothing else moves. The choice of the first unit must change
    // with it.
    let mut world = one_cell_world();
    // The option that reads the units of the cell is the fourth of the set.
    let join = 3u8;
    only(&mut world, join, Fix32::ONE);

    let alone = world
        .spawn_soldier(first_open_address(&world), FactionId(0))
        .expect("the open tile admits a unit");
    world.step(2).expect("the step must run");
    assert_eq!(
        world.soldier_intent(alone).expect("alive"),
        None,
        "one unit in a whole cell was company enough"
    );

    let crowd = crowd(&mut world);
    assert!(crowd.len() > 1000, "the fixture put {} units", crowd.len());
    world.step(2).expect("the step must run");
    world.step(2).expect("the step must run");

    assert_eq!(
        world.soldier_intent(alone).expect("alive"),
        Some(join),
        "the crowd arrived and the unit did not answer it"
    );
}

/// Returns the first address of a world that admits a unit.
fn first_open_address(world: &World) -> Axial {
    addresses(world)
        .into_iter()
        .find(|address| world.admits_a_unit(*address))
        .expect("the fixture world holds open ground")
}

#[test]
fn a_world_with_nothing_to_respond_to_produces_no_movement() {
    // Every option scores zero, so every score is below the floor. Without
    // the floor the tie-break would give every unit option zero, and the
    // whole population would walk one way.[^1]
    //
    // [^1]: Findings register, FND-014. `docs/FINDINGS.md`
    let mut world = one_cell_world();
    // The units keep their need through the run, so the drive of every
    // option stays at its full value and only the weights are at zero.
    world
        .set_economy_schedule(1024, 0)
        .expect("the period is inside the range");
    for index in 0..choose::OPTION_COUNT as u8 {
        world
            .set_option_weight(index, Fix32::ZERO)
            .expect("the index is inside the set");
    }
    let units = populate(&mut world, 4);
    assert!(!units.is_empty(), "the fixture put nobody in the world");
    let before: Vec<Axial> = units
        .iter()
        .map(|unit| world.soldiers().address(*unit).expect("alive"))
        .collect();

    for _ in 0..24 {
        world.step(2).expect("the step must run");
    }

    for (unit, start) in units.iter().zip(&before) {
        assert_eq!(
            world.soldiers().address(*unit).expect("alive"),
            *start,
            "a unit moved with nothing to move towards"
        );
        assert_eq!(
            world.soldier_intent(*unit).expect("alive"),
            None,
            "a unit holds an intent it never earned"
        );
    }
    assert!(world.check_invariants(), "the world broke an invariant");

    // No unit is stuck. The hold lasts while the world gives nothing, and it
    // ends when the world gives something.
    only(&mut world, 0, Fix32::ONE);
    for _ in 0..2 {
        world.step(2).expect("the step must run");
    }
    let moved = units
        .iter()
        .zip(&before)
        .filter(|(unit, start)| world.soldiers().address(**unit).expect("alive") != **start)
        .count();
    assert!(
        moved > 0,
        "the units never started again, so they are stuck"
    );
}

#[test]
fn the_tie_breaks_by_the_lowest_option_index() {
    // The fixture builds the tie rather than hoping for one. Two options
    // both saturate at the top of the range, so their scores are equal to
    // the bit. A third option carries the same weight and a smaller field,
    // so it loses and shows that the tie sits at the top.
    let mut world = one_cell_world();
    for index in [0u8, 2, 3] {
        world
            .set_option_weight(index, Fix32::MAX)
            .expect("the index is inside the set");
    }
    world
        .set_option_weight(1, Fix32::ZERO)
        .expect("the index is inside the set");
    let units = crowd(&mut world);
    let unit = units[0];
    world.step(2).expect("the step must run");
    world.step(2).expect("the step must run");

    let why = world.explain_choice(unit).expect("alive");
    assert_eq!(
        why.scores[0], why.scores[3],
        "the fixture built no tie: {:?}",
        why.scores
    );
    assert!(
        why.scores[0] > why.floor,
        "the tie sits below the floor, so the floor decided and not the tie"
    );
    assert!(
        why.scores[2] < why.scores[0],
        "every option tied, so the fixture proves nothing about the index"
    );
    assert_eq!(
        why.best, 0,
        "the tie went to option {} and not to the lowest index",
        why.best
    );
    assert_eq!(world.soldier_intent(unit).expect("alive"), Some(0));
}

#[test]
fn a_watcher_asks_why_a_unit_chose_what_it_chose() {
    let mut world = one_cell_world();
    let unit = world
        .spawn_soldier(first_open_address(&world), FactionId(0))
        .expect("the open tile admits a unit");
    world.step(1).expect("the step must run");

    let why = world.explain_choice(unit).expect("alive");
    assert_eq!(why.floor, SCORE_FLOOR);
    assert_eq!(why.need, NEED_FULL);
    assert_eq!(why.intent, why.best);
    assert_eq!(
        why.best_name(),
        Some(choose::OPTIONS[why.best as usize].name),
        "the explanation named no option"
    );
    // The answer states what the unit read and what it weighed, so a reader
    // can repeat the arithmetic.
    for (index, weight) in why.weights.iter().enumerate() {
        assert_eq!(
            *weight,
            world
                .option_weight(index as u8)
                .expect("the index is inside the set")
        );
    }
    assert!(
        why.fields.iter().any(|field| *field != Fix32::ZERO),
        "the unit read a cell that holds nothing"
    );
}

#[test]
fn changing_a_weight_that_the_world_cannot_pay_moves_no_hash() {
    // A score is transient. Nothing stores it, so a weight that cannot
    // change any intent must change no byte of the state hash.
    //
    // The option that reads the unmet need is the second of the set. Every
    // unit here is at full need, so that option scores zero at any weight.
    // The test asserts the need rather than assuming it.
    let hash_of = |weight_of_forage: Fix32, weight_of_climb: Fix32| {
        let mut world = one_cell_world();
        world
            .set_economy_schedule(1024, 0)
            .expect("the period is inside the range");
        world
            .set_option_weight(1, weight_of_forage)
            .expect("the index is inside the set");
        world
            .set_option_weight(2, weight_of_climb)
            .expect("the index is inside the set");
        let units = populate(&mut world, 4);
        for _ in 0..6 {
            world.step(2).expect("the step must run");
        }
        for unit in &units {
            assert_eq!(
                world.soldiers().need(*unit),
                Some(NEED_FULL),
                "a unit lost need, so the unmet option is no longer inert"
            );
        }
        world.state_hash().finish()
    };

    let plain = hash_of(Fix32::ZERO, Fix32::ONE);
    let paid = hash_of(Fix32::MAX, Fix32::ONE);
    assert_eq!(
        plain, paid,
        "a weight that no unit could act on changed the state"
    );

    // The fixture reaches the case. A weight the world can pay does move the
    // hash, so the assertion above is not measuring an inert world.
    let different = hash_of(Fix32::ZERO, Fix32::MAX);
    assert_ne!(
        plain, different,
        "no weight changes this world, so the fixture proves nothing"
    );
}

/// Runs the fixture at one thread count and returns what it chose.
fn intents_at(threads: usize, seed: u64) -> (Vec<Option<u8>>, u64) {
    let mut world = World::new(WorldConfig {
        width: ONE_CELL_EXTENT * 2,
        height: ONE_CELL_EXTENT * 2,
        seed,
        faction_count: 2,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    })
    .expect("the extent must describe a world");
    let units = populate(&mut world, 2);
    assert!(units.len() > threads, "the fixture must fill every slot");
    // The world keeps its default interval, so the run covers the stagger as
    // well as the scoring. It runs past one whole interval, so every cell
    // has chosen at least once by the end.
    for _ in 0..(world.choice_schedule().period() + 2) {
        world.step(threads).expect("the step must run");
    }
    (
        units
            .iter()
            .map(|unit| world.soldier_intent(*unit).expect("alive"))
            .collect(),
        world.state_hash().finish(),
    )
}

#[test]
fn the_choice_is_the_same_at_one_two_and_twelve_threads() {
    for seed in [7u64, 23] {
        let (one, hash_one) = intents_at(1, seed);
        let (two, hash_two) = intents_at(2, seed);
        let (twelve, hash_twelve) = intents_at(12, seed);
        assert!(
            one.iter().any(Option::is_some),
            "seed {seed} chose nothing, so the run proves nothing"
        );
        assert_eq!(one, two, "the choice differed at two threads, seed {seed}");
        assert_eq!(
            one, twelve,
            "the choice differed at twelve threads, seed {seed}"
        );
        assert_eq!(hash_one, hash_two, "the state differed, seed {seed}");
        assert_eq!(hash_one, hash_twelve, "the state differed, seed {seed}");
    }
}

/// The interval that the stagger test runs at.
const STAGGER_PERIOD_LOG2: u32 = 5;

#[test]
fn every_unit_of_one_cell_chooses_on_the_same_frame() {
    // The stagger key is the level 1 cell. A key on the identity of the unit
    // would give the units of one cell different frames, which scatters the
    // active units through the arena.[^1]
    //
    // [^1]: Findings register, FND-023. `docs/FINDINGS.md`
    let mut world = many_cell_world();
    world
        .set_choice_schedule(STAGGER_PERIOD_LOG2)
        .expect("the exponent is inside the range");
    let schedule =
        ChoiceSchedule::new(STAGGER_PERIOD_LOG2).expect("the exponent is inside the range");

    // Two cells of different phase. The phase is a pure function of the cell
    // index, so the fixture picks the pair rather than searching for one.
    let open = one_open_address_of_each_cell(&world);
    let first = open[0];
    let second = *open
        .iter()
        .find(|(cell, _)| {
            choose::stagger_phase(*cell, STAGGER_PERIOD_LOG2)
                != choose::stagger_phase(first.0, STAGGER_PERIOD_LOG2)
        })
        .expect("the fixture must hold two cells of different phase");

    let mut units: Vec<(u32, Entity)> = Vec::new();
    for (cell, address) in [first, second] {
        let capacity = world.tile_kind(address).map_or(0, TileKind::capacity);
        assert!(capacity >= 2, "one unit in a cell agrees with nobody");
        for _ in 0..capacity {
            units.push((
                cell,
                world
                    .spawn_soldier(address, FactionId(0))
                    .expect("the open tile admits a unit"),
            ));
        }
    }

    let mut first_frame: Vec<Option<u64>> = vec![None; units.len()];
    for frame in 1..=schedule.period() {
        world.step(2).expect("the step must run");
        for (index, (_, unit)) in units.iter().enumerate() {
            if first_frame[index].is_none() && world.soldier_intent(*unit).expect("alive").is_some()
            {
                first_frame[index] = Some(frame);
            }
        }
    }

    for (cell, _) in [first, second] {
        let frames: Vec<Option<u64>> = units
            .iter()
            .zip(&first_frame)
            .filter(|((owner, _), _)| *owner == cell)
            .map(|(_, frame)| *frame)
            .collect();
        assert!(
            frames[0].is_some(),
            "cell {cell} never chose inside one interval"
        );
        assert!(
            frames.iter().all(|frame| *frame == frames[0]),
            "the units of cell {cell} chose on different frames: {frames:?}"
        );
    }

    let frame_of = |cell: u32| {
        units
            .iter()
            .zip(&first_frame)
            .find(|((owner, _), _)| *owner == cell)
            .map(|(_, frame)| *frame)
            .expect("the cell holds units")
    };
    assert_ne!(
        frame_of(first.0),
        frame_of(second.0),
        "two cells of different phase chose on one frame"
    );
}

#[test]
fn an_interval_above_the_ceiling_is_refused() {
    let mut world = one_cell_world();
    assert!(world
        .set_choice_schedule(choose::PERIOD_LOG2_CEILING + 1)
        .is_err());
    assert!(world
        .set_option_weight(choose::OPTION_COUNT as u8, Fix32::ONE)
        .is_err());
    assert_eq!(world.option_weight(choose::OPTION_COUNT as u8), None);
    assert!(NO_INTENT as usize > choose::OPTION_COUNT);
}

/// The option index of the row that scores the food of a cell.
///
/// The index is the tie-break position of the row, and it did not change when
/// the row changed the field it reads.[^1]
///
/// # References
///
/// [^1]: ADR-0004, iteration order is explicit, decisions D1 and D3. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
const FORAGE: u8 = 1;

/// Drains the need of every unit in one tick, and lets nobody die of it.
///
/// The `forage` row is driven by what a unit lacks, so a unit at full need
/// scores zero for it whatever the ground carries. A fixture that left the
/// need alone would measure the drive and not the field.
///
/// The decay takes a whole need in one tick, and the bound sits at the top of
/// the range, so the run is short and no unit reaches the death scan. The
/// economy runs on every tick, because the need falls in the pass that the
/// schedule gates.
fn starve(world: &mut World) {
    world
        .set_economy_schedule(1, 0)
        .expect("the period is inside the range");
    let rule = NeedRule::new(
        NEED_FULL,
        NEED_FULL,
        Fix32(NEED_FULL.0 / 2),
        Fix32(NEED_FULL.0 / 16),
        Fix32::MAX,
    )
    .expect("every rate is at or above zero");
    world.set_need_rule(rule);
}

/// Returns the open address closest to the middle of each cell, with the mean
/// food of that cell.
///
/// The address sits away from the edge of its block, so a unit that acts on
/// its choice is still inside the cell it read a few frames later.
fn middle_of_each_cell(world: &World) -> Vec<(u32, Axial, Fix32)> {
    let layout = world.pyramid().layout();
    let edge = i64::from(layout.block_edge());
    let mut best: Vec<(u32, Axial, i64)> = Vec::new();
    for address in addresses(world) {
        if !world.admits_a_unit(address) {
            continue;
        }
        let cell = cell_of(world, address);
        let column = i64::from(address.q) % edge;
        let row = i64::from(address.r) % edge;
        let from_middle = (column * 2 - edge).abs() + (row * 2 - edge).abs();
        match best.iter_mut().find(|(known, _, _)| *known == cell) {
            Some(entry) if entry.2 > from_middle => *entry = (cell, address, from_middle),
            Some(_) => {}
            None => best.push((cell, address, from_middle)),
        }
    }
    best.sort_unstable_by_key(|(cell, _, _)| *cell);
    best.into_iter()
        .filter_map(|(cell, address, _)| {
            let food = world.pyramid().cell(cell)?.mean_food()?;
            Some((cell, address, food))
        })
        .collect()
}

#[test]
fn a_hungry_unit_forages_where_there_is_food_and_holds_where_there_is_none() {
    // The fixture changes one value of the world: the food that the tiles of
    // the cell hold. Nothing else differs between the two units. This is the
    // test that a pinned food total must fail, and it is what tells a value
    // that a stage reads from a value that a stage stores and discards.[^1]
    //
    // [^1]: Findings register, FND-181. `docs/FINDINGS.md`
    let mut world = many_cell_world();
    starve(&mut world);
    let mut cells = middle_of_each_cell(&world);
    cells.sort_unstable_by_key(|(_, _, food)| *food);
    let (poor_cell, poor_address, poor_food) = cells[0];
    let (rich_cell, rich_address, rich_food) = cells[cells.len() - 1];
    assert!(
        i64::from(rich_food.0) >= i64::from(poor_food.0) * 4,
        "the fixture holds no food contrast: {poor_food:?} against {rich_food:?}"
    );

    // The weight puts the floor between the two cells, so the food alone
    // decides which unit acts and which holds. The arithmetic is exact.
    let middle = (i64::from(poor_food.0) + i64::from(rich_food.0)) / 2;
    let weight = Fix32(((i64::from(SCORE_FLOOR.0) << 16) / middle) as i32);
    only(&mut world, FORAGE, weight);

    let hungry = world
        .spawn_soldier(rich_address, FactionId(0))
        .expect("the open tile admits a unit");
    let idle = world
        .spawn_soldier(poor_address, FactionId(0))
        .expect("the open tile admits a unit");
    for _ in 0..3 {
        world.step(2).expect("the step must run");
    }

    // The fixture holds. Both units are hungry, and neither has left the cell
    // it read, so the assertion below reads the ground it was aimed at.
    for unit in [hungry, idle] {
        assert_eq!(
            world.soldiers().need(unit),
            Some(Fix32::ZERO),
            "a unit kept its need, so the forage option scores zero for it"
        );
    }
    assert_eq!(
        cell_of(&world, world.soldiers().address(hungry).expect("alive")),
        rich_cell,
        "the unit left the cell it read"
    );
    assert_eq!(
        cell_of(&world, world.soldiers().address(idle).expect("alive")),
        poor_cell,
        "the unit left the cell it read"
    );

    assert_eq!(
        world.soldier_intent(hungry).expect("alive"),
        Some(FORAGE),
        "the unit stood on food and did not forage"
    );
    assert_eq!(
        world.soldier_intent(idle).expect("alive"),
        None,
        "the unit foraged ground that carries no food"
    );
}

#[test]
fn the_forage_option_reads_the_food_that_the_tiles_hold() {
    // The explanation reports the value each option read. The recomputation
    // sums the remaining stock of each tile of the cell through the public
    // interface, so it never asks the summary and the two answers are
    // independent.[^1]
    //
    // [^1]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D2. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
    let mut world = many_cell_world();
    let cells = middle_of_each_cell(&world);
    let (_, address, _) = cells[cells.len() - 1];
    let unit = world
        .spawn_soldier(address, FactionId(0))
        .expect("the open tile admits a unit");
    world.step(1).expect("the step must run");

    let standing = world.soldiers().address(unit).expect("alive");
    let cell = cell_of(&world, standing);
    let layout = world.pyramid().layout();
    let edge = layout.block_edge();
    let first_column = (cell % layout.blocks_wide()) * edge;
    let first_row = (cell / layout.blocks_wide()) * edge;
    let (mut food, mut tiles) = (0i64, 0i64);
    for row in first_row..first_row + edge {
        for column in first_column..first_column + edge {
            let at = Axial::new(column as i32, row as i32);
            let Some(stock) = world.tile_stock(at, ResourceKind::Food) else {
                continue;
            };
            tiles += 1;
            food += i64::from(stock.0);
        }
    }
    assert!(food > 0, "the cell carries no food, so the reading is zero");

    let why = world.explain_choice(unit).expect("alive");
    assert_eq!(
        i64::from(why.fields[FORAGE as usize].0),
        (food << 16) / tiles,
        "the forage option read something other than the food of the cell"
    );
    assert_eq!(
        choose::OPTIONS[FORAGE as usize].name,
        "forage",
        "the row that reads the food is not the forage row"
    );
}
