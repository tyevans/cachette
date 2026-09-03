//! The engine orders a gather, and no caller does.
//!
//! The gather order is a control-plane verb, and it stays one.[^1] The choice
//! pass is now a second writer of the same column: a unit whose option is
//! `forage` holds an order for food, and a unit that chose anything else holds
//! none. The two writes happen in one pass, for the same units, on the same
//! frame, so no second stage can disagree with the first.[^2]
//!
//! Every test here drives the step. None calls the gather resolve.[^3]
//!
//! **The fixture is built for these tests.** It does not copy the world of the
//! demonstration binary, because that world is chosen to look right and not to
//! produce an extreme.[^4] It holds a cell whose tiles carry food, a cell whose
//! tiles carry none, and units that have lost the whole of their need, because
//! the `forage` row is driven by what a unit lacks and a unit at full need
//! scores zero for it whatever the ground carries.
//!
//! # References
//!
//! [^1]: ADR-0073, gathering is admitted by sort-then-admit against the tile, decision D1. `docs/adrs/accepted/adr-0073-gathering-is-admitted-by-sort-then-admit-against-the-tile.md`
//! [^2]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
//! [^3]: Testing rules, section 5. `.claude/rules/testing.md`
//! [^4]: Testing rules, section 2a. `.claude/rules/testing.md`

use cachette_core::choose::{self, ChoiceSchedule, SCORE_FLOOR};
use cachette_core::cohort::{NeedRule, NEED_FULL};
use cachette_core::resource::{Amount, RecoveryRules, ResourceKind};
use cachette_core::{Axial, Entity, FactionId, Fix32, World, WorldConfig};

/// The extent of every fixture world.
const EXTENT: u32 = 256;

/// The seed of every fixture world.
///
/// Each test asserts the property of the ground that it depends on, so a
/// change to the generator fails the fixture rather than the assertion.
const SEED: u64 = 7;

/// The option index of the row that scores the food of a cell.
///
/// The index is the tie-break position of the row.[^1]
///
/// # References
///
/// [^1]: ADR-0004, iteration order is explicit, decisions D1 and D3. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
const FORAGE: u8 = 2;

/// The thread counts that the equivalence test runs at.
const THREAD_COUNTS: [usize; 3] = [1, 2, 12];

/// Builds a world of many level 1 cells, with the choice on every tick.
fn world() -> World {
    let mut world = World::new(WorldConfig {
        width: EXTENT,
        height: EXTENT,
        seed: SEED,
        faction_count: 2,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    })
    .expect("the extent must describe a world");
    world
        .set_choice_schedule(0)
        .expect("the exponent is inside the range");
    world
}

/// Drains the need of every unit in one tick, and lets nobody die of it.
///
/// The `forage` row is driven by what a unit lacks. A fixture that left the
/// need alone would measure the drive and not the ground.
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

/// Puts every weight on one option and none on the others.
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

/// Returns the level 1 cell that covers one address.
fn cell_of(world: &World, address: Axial) -> u32 {
    let layout = world.pyramid().layout();
    let tile = world
        .grid()
        .index_of(address)
        .expect("the address is inside the world");
    layout.block_of_key(layout.key_of(tile).expect("the tile is inside the world"))
}

/// Returns every address of a world, in index order.
fn addresses(world: &World) -> Vec<Axial> {
    let grid = world.grid();
    (0..grid.tile_count())
        .map(|index| Axial::new((index % grid.width()) as i32, (index / grid.width()) as i32))
        .collect()
}

/// Returns the open address in the middle of each cell, with the food of the
/// cell and the food of that tile.
///
/// The address sits away from the block edge, so a unit that acts on its
/// choice is still inside the cell it read a few frames later.
fn middle_of_each_cell(world: &World) -> Vec<(u32, Axial, Fix32, u32)> {
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
            let here = world.tile_stock(address, ResourceKind::Food)?;
            Some((cell, address, food, here.0))
        })
        .collect()
}

/// Returns an open address of one cell whose tile carries no food.
///
/// The scan runs in the index order of the cell and takes the first tile that
/// fits, so the answer is fixed.[^1]
///
/// # References
///
/// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
fn bare_address_of(world: &World, cell: u32) -> Axial {
    let layout = world.pyramid().layout();
    let edge = layout.block_edge();
    let first_column = (cell % layout.blocks_wide()) * edge;
    let first_row = (cell / layout.blocks_wide()) * edge;
    for row in first_row..first_row + edge {
        for column in first_column..first_column + edge {
            let address = Axial::new(column as i32, row as i32);
            if world.admits_a_unit(address)
                && world.tile_stock(address, ResourceKind::Food) == Some(Amount::ZERO)
            {
                return address;
            }
        }
    }
    panic!("cell {cell} holds no open tile that carries no food");
}

/// Returns the weight that puts the score floor between two food readings.
///
/// The arithmetic is exact. A weight built this way makes the food of the cell
/// the only thing that decides whether a unit acts.
fn weight_between(low: Fix32, high: Fix32) -> Fix32 {
    let middle = (i64::from(low.0) + i64::from(high.0)) / 2;
    assert!(middle > 0, "the fixture holds no food anywhere");
    Fix32(((i64::from(SCORE_FLOOR.0) << 16) / middle) as i32)
}

/// Builds the fixture: a rich cell, a bare cell, and a hungry unit on each.
///
/// Returns the world, the unit that stands on food, and the unit that does
/// not.
fn a_rich_tile_and_a_bare_one() -> (World, Entity, Entity) {
    let mut world = world();
    starve(&mut world);
    let mut cells = middle_of_each_cell(&world);
    cells.sort_unstable_by_key(|(_, _, food, _)| *food);
    let (poor_cell, _, poor_food, _) = cells[0];
    let (_, rich_address, rich_food, rich_here) = cells[cells.len() - 1];
    // The bare tile is chosen for the property the test needs, which is that
    // it carries no food at all. The middle of the poorest cell need not have
    // that property, so the fixture searches for a tile that does.
    let poor_address = bare_address_of(&world, poor_cell);
    assert!(rich_here > 0, "the rich tile carries no food");
    assert!(
        i64::from(rich_food.0) >= i64::from(poor_food.0) * 4,
        "the fixture holds no food contrast: {poor_food:?} against {rich_food:?}"
    );
    only(&mut world, FORAGE, weight_between(poor_food, rich_food));

    let fed = world
        .spawn_soldier(rich_address, FactionId(0))
        .expect("the open tile admits a unit");
    let hungry = world
        .spawn_soldier(poor_address, FactionId(0))
        .expect("the open tile admits a unit");
    (world, fed, hungry)
}

#[test]
fn the_engine_orders_a_gather_and_a_unit_that_chose_otherwise_holds_none() {
    // Both halves in one run. Nothing in this test orders a gather, and the
    // gather log is what proves that the engine did.[^1]
    //
    // [^1]: Findings register, FND-181. `docs/FINDINGS.md`
    let (mut world, forager, idle) = a_rich_tile_and_a_bare_one();
    // The need falls in the pass that runs after the choice, so the first
    // frame reads a unit that still holds a whole need and the `forage` row
    // scores zero for it. The run is long enough for the need to reach the
    // choice. A unit that acts on its choice also moves, so the events are
    // gathered over the run rather than read from the last frame.
    let mut events = Vec::new();
    for _ in 0..6 {
        world.step(1).expect("the step must run");
        events.extend_from_slice(world.gather_log());
    }
    for unit in [forager, idle] {
        assert_eq!(
            world.soldiers().need(unit),
            Some(Fix32::ZERO),
            "a unit kept its need, so the forage option scores zero for it"
        );
    }

    assert_eq!(
        world.soldier_intent(forager).expect("alive"),
        Some(FORAGE),
        "the unit on food did not choose to forage"
    );
    assert_eq!(
        world.soldier_intent(idle).expect("alive"),
        None,
        "the unit on bare ground chose to forage"
    );
    assert_eq!(
        world.gather_order(forager),
        Some(Some(ResourceKind::Food)),
        "the choice pass wrote no gather order for the unit that forages"
    );
    assert_eq!(
        world.gather_order(idle),
        Some(None),
        "a unit that chose no option holds a gather order"
    );
    assert!(
        events.iter().any(|event| event.unit == forager.to_bits()),
        "no caller ordered a gather and the engine produced none either"
    );
    assert!(
        !events.iter().any(|event| event.unit == idle.to_bits()),
        "a unit that chose no option took a resource"
    );
    assert!(
        world
            .soldier_carry(forager)
            .expect("alive")
            .of(ResourceKind::Food)
            .0
            > 0,
        "the unit that forages carries nothing"
    );
}

#[test]
fn a_unit_that_forages_bare_ground_takes_nothing() {
    // The tile test stays in the resolve. The unit holds the order, stands on
    // ground that carries no food, and produces no event.[^1]
    //
    // [^1]: ADR-0073, gathering is admitted by sort-then-admit against the tile, decision D3. `docs/adrs/accepted/adr-0073-gathering-is-admitted-by-sort-then-admit-against-the-tile.md`
    let (mut world, _, idle) = a_rich_tile_and_a_bare_one();
    world.order_gather(idle, ResourceKind::Food);
    // Nothing chooses, so the order the caller gave is the one the resolve
    // reads.
    only(&mut world, FORAGE, Fix32::ZERO);
    let standing = world.soldiers().address(idle).expect("alive");
    assert_eq!(
        world.tile_stock(standing, ResourceKind::Food),
        Some(Amount::ZERO)
    );

    world.step(1).expect("the step must run");
    assert_eq!(
        world
            .soldier_carry(idle)
            .expect("alive")
            .of(ResourceKind::Food)
            .0,
        0,
        "a unit took food from ground that holds none"
    );
    assert!(
        !world
            .gather_log()
            .iter()
            .any(|event| event.unit == idle.to_bits()),
        "a grant with no amount reached the log"
    );
}

#[test]
fn a_unit_whose_cell_does_not_choose_keeps_the_order_it_held() {
    // The engine writes the order only for a unit whose cell chooses on that
    // frame, which is the set the intent pass already writes.[^1]
    //
    // [^1]: ADR-0064, a unit chooses by scoring a small fixed option set, decision D4. `docs/adrs/accepted/adr-0064-a-unit-chooses-by-scoring-a-small-fixed-option-set.md`
    let (mut world, _, idle) = a_rich_tile_and_a_bare_one();
    let schedule = ChoiceSchedule::new(4).expect("the exponent is inside the range");
    world
        .set_choice_schedule(schedule.period_log2())
        .expect("the exponent is inside the range");
    let cell = cell_of(&world, world.soldiers().address(idle).expect("alive"));
    world.order_gather(idle, ResourceKind::Food);

    // Step over frames the cell does not choose on. The order must survive
    // every one of them.
    let mut held = 0;
    while !schedule.chooses_now(cell, world.tick().0.wrapping_add(1)) {
        world.step(1).expect("the step must run");
        assert_eq!(
            world.gather_order(idle),
            Some(Some(ResourceKind::Food)),
            "the order did not survive a frame on which the cell held"
        );
        held += 1;
    }
    assert!(
        held > 0,
        "the fixture gave the cell no frame on which to hold"
    );

    // The frame the cell chooses on is the frame the choice replaces the
    // order. The unit stands on bare ground, so it chooses nothing and the
    // choice clears what the caller gave.
    world.step(1).expect("the step must run");
    assert_eq!(
        world.soldier_intent(idle).expect("alive"),
        None,
        "the unit chose an option on bare ground"
    );
    assert_eq!(
        world.gather_order(idle),
        Some(None),
        "the choice did not replace the order that the caller gave"
    );
}

#[test]
fn a_deposit_falls_and_then_rises() {
    let (mut world, forager, _) = a_rich_tile_and_a_bare_one();
    // The recovery period is a parameter of the world, and the test states the
    // one it needs rather than waiting out the content value.
    let rules = RecoveryRules::from_ticks([Some(1), None, None]).expect("no period is zero");
    world.set_recovery_rules(rules);
    // A unit that acts on its choice also moves, so the test names the deposit
    // that the engine actually drew from rather than the tile the unit started
    // on.
    let mut taken_from = None;
    for _ in 0..8 {
        world.step(1).expect("the step must run");
        if let Some(event) = world.gather_log().first() {
            taken_from = Some(event.tile);
            break;
        }
    }
    let tile = taken_from.expect("the engine ordered no gather");
    let standing = world
        .grid()
        .address_of(tile)
        .expect("the tile is inside the world");
    let before = world
        .original_stock(standing, ResourceKind::Food)
        .expect("the tile is inside the world");
    let drawn = world
        .tile_stock(standing, ResourceKind::Food)
        .expect("the tile is inside the world");
    assert!(
        drawn < before,
        "the deposit did not fall: {before:?} {drawn:?}"
    );
    assert!(
        !world.depletion().is_empty(),
        "the depletion set did not grow"
    );

    // Nobody forages now, so the recovery alone moves the deposit.
    only(&mut world, FORAGE, Fix32::ZERO);
    world.step(1).expect("the step must run");
    assert_eq!(
        world.gather_order(forager),
        Some(None),
        "the choice did not clear the order of a unit that chose nothing"
    );
    for _ in 0..4 {
        world.step(1).expect("the step must run");
    }
    let given_back = world
        .tile_stock(standing, ResourceKind::Food)
        .expect("the tile is inside the world");
    assert!(
        given_back > drawn,
        "the deposit did not rise: {drawn:?} {given_back:?}"
    );
    assert!(world.check_invariants(), "conservation does not hold");
}

#[test]
fn the_gather_log_is_the_same_at_every_thread_count() {
    let mut logs = Vec::new();
    for threads in THREAD_COUNTS {
        let (mut world, _, _) = a_rich_tile_and_a_bare_one();
        let mut joined: Vec<u8> = Vec::new();
        for _ in 0..5 {
            world.step(threads).expect("the step must run");
            joined.extend_from_slice(world.gather_log_bytes());
        }
        assert!(!joined.is_empty(), "the fixture produced no gather event");
        logs.push(joined);
    }
    assert_eq!(logs[0], logs[1]);
    assert_eq!(logs[0], logs[2]);
}

#[test]
fn a_crowd_works_a_deposit_down_and_the_world_still_balances() {
    // The negative feedback of the loop, at the smallest size that shows it.
    // The food falls where the crowd stands, and the summary of the cell falls
    // with it.
    let mut world = world();
    starve(&mut world);
    let mut cells = middle_of_each_cell(&world);
    cells.sort_unstable_by_key(|(_, _, food, _)| *food);
    let (rich_cell, rich_address, rich_food, _) = cells[cells.len() - 1];
    let (_, _, poor_food, _) = cells[0];
    only(&mut world, FORAGE, weight_between(poor_food, rich_food));
    for _ in 0..8 {
        world
            .spawn_soldier(rich_address, FactionId(0))
            .expect("the open tile admits a unit");
    }

    let before = world
        .pyramid()
        .cell(rich_cell)
        .expect("the cell exists")
        .food_total();
    for _ in 0..6 {
        world.step(2).expect("the step must run");
    }
    let after = world
        .pyramid()
        .cell(rich_cell)
        .expect("the cell exists")
        .food_total();
    assert!(
        after.0 < before.0,
        "the crowd did not work the deposit down: {before:?} {after:?}"
    );
    assert!(world.check_invariants(), "conservation does not hold");
}
