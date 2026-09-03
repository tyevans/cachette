//! The resource field, and what a unit takes from it.
//!
//! The test goes through the public crate API. It reaches into no internal
//! module.[^1]
//!
//! Three families of test live here, and they answer different questions.
//!
//! The first family asks what the stock depends on. A determinism test cannot
//! tell a correct field from a consistently wrong one, so each field of the
//! draw key gets its own test: change the field, and the output must
//! change.[^2] The row component of the address is the one that a perturbed
//! build drops, which is how these tests are proved able to fail.[^2]
//!
//! The second family asks whether the ground shapes the field. Water holds
//! nothing, a wooded tile holds the most wood, and no kind covers the world.
//!
//! The third family drives the engine and inspects what it did. A capability
//! that nothing reaches through the engine ships inert.[^3]
//!
//! # References
//!
//! [^1]: Testing rules, section 6. `.claude/rules/testing.md`
//! [^2]: Testing rules, section 2. `.claude/rules/testing.md`
//! [^3]: Testing rules, section 5. `.claude/rules/testing.md`

use cachette_core::choose::{self, ChoiceSchedule};
use cachette_core::cohort::{NeedRule, NEED_FULL};
use cachette_core::resource::{
    Amount, RecoveryRules, ResourceKind, RESOURCE_KIND_COUNT, TICKS_IN_A_SIMULATED_DAY,
};
use cachette_core::terrain::TileKind;
use cachette_core::{Axial, Entity, FactionId, Fix32, TileIdx, World, WorldConfig};

/// The extent that most tests read.
///
/// The extent is wider than the coarsest lattice spacing of the ground
/// generator, so the world holds every kind of ground. A world smaller than
/// that spacing holds one terrain, and the field then measures the
/// fixture.[^1]
///
/// # References
///
/// [^1]: Findings register, FND-054. `docs/FINDINGS.md`
const WIDTH: u32 = 192;
/// The number of rows of that extent.
const HEIGHT: u32 = 192;
/// The seed that most tests read.
const SEED: u64 = 0x0123_4567_89ab_cdef;
/// The seed of the world that holds an island deposit.
///
/// An island is a tile whose every neighbour refuses a unit. A unit on it
/// never moves, so a test can put two named units on one deposit and know
/// they are still there when the resolve runs. Most worlds hold none, and
/// this one was found by a scan.
const ISLAND_SEED: u64 = 102;

/// Builds the world under test.
fn world(seed: u64) -> World {
    let mut world = World::new(WorldConfig {
        width: WIDTH,
        height: HEIGHT,
        seed,
        faction_count: 4,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    })
    .expect("the extent must describe a world");
    hold_the_choice(&mut world);
    world
}

/// The number of frames that the longest test in this file runs.
const FRAMES: u64 = 200;

/// Puts the choice far enough apart that it does not replace a gather order.
///
/// **The choice pass writes the gather order of a unit whose level 1 cell
/// chooses on that frame, and that write replaces the order a caller
/// gave.**[^1] [^2] Every test in this file gives the order from outside, so
/// the fixture keeps the choice away from the frames under test.
///
/// The phase of a cell is a pure function of the cell index, so the answer is
/// the same on every run. The assertion states it rather than trusting it.
///
/// # References
///
/// [^1]: ADR-0064, a unit chooses by scoring a small fixed option set, decision D4. `docs/adrs/accepted/adr-0064-a-unit-chooses-by-scoring-a-small-fixed-option-set.md`
/// [^2]: Findings register, FND-211. `docs/FINDINGS.md`
fn hold_the_choice(world: &mut World) {
    let schedule =
        ChoiceSchedule::new(choose::PERIOD_LOG2_CEILING).expect("the exponent is inside the range");
    world
        .set_choice_schedule(schedule.period_log2())
        .expect("the exponent is inside the range");
    for cell in 0..world.pyramid().len() as u32 {
        for frame in 1..=FRAMES {
            assert!(
                !schedule.chooses_now(cell, frame),
                "cell {cell} chooses on frame {frame}, so the choice replaces \
                 the gather order that a test gave"
            );
        }
    }
}

/// Takes the whole need of every unit in one tick, and lets nobody die of it.
///
/// The foraging option is driven by what a unit lacks, so a unit at full need
/// scores zero for it whatever the ground carries. A test that needs the
/// engine to order a gather must supply a hungry unit.
///
/// The bound sits at the top of the range, so no unit reaches the death scan
/// inside a short run.
fn make_them_hungry(world: &mut World) {
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
    for option in 0..choose::OPTION_COUNT as u8 {
        world
            .set_option_weight(option, Fix32::ZERO)
            .expect("the index is inside the set");
    }
    world
        .set_option_weight(FORAGE, Fix32::MAX)
        .expect("the index is inside the set");
}

/// The option index of the row that forages.
const FORAGE: u8 = 1;

/// Returns every address of the extent, in row-major order.
fn addresses() -> Vec<Axial> {
    let mut all = Vec::with_capacity((WIDTH * HEIGHT) as usize);
    for r in 0..HEIGHT {
        for q in 0..WIDTH {
            all.push(Axial::new(q as i32, r as i32));
        }
    }
    all
}

/// Returns the stock of every address, for one kind.
fn stocks(world: &World, kind: ResourceKind) -> Vec<u32> {
    addresses()
        .into_iter()
        .map(|address| {
            world
                .original_stock(address, kind)
                .expect("the address is inside the world")
                .0
        })
        .collect()
}

#[test]
fn the_stock_depends_on_the_seed() {
    // The seed is the first field of the draw key. Two worlds that differ
    // only in the seed must hold different resources.
    let one = stocks(&world(SEED), ResourceKind::Food);
    let other = stocks(&world(SEED ^ 0x5555_5555_5555_5555), ResourceKind::Food);
    assert_ne!(one, other, "the seed does not reach the stock draw");
}

#[test]
fn the_stock_depends_on_the_column_of_the_address() {
    // The column component of the address must reach the key. A field that
    // ignored it would repeat down every row, and both determinism tests
    // would still pass.
    let field = world(SEED);
    let row = (HEIGHT / 2) as i32;
    let mut along: Vec<u32> = (0..WIDTH as i32)
        .map(|q| {
            field
                .original_stock(Axial::new(q, row), ResourceKind::Food)
                .expect("the address is inside the world")
                .0
        })
        .collect();
    along.dedup();
    assert!(along.len() > 1, "the column does not reach the stock draw");
}

#[test]
fn the_stock_depends_on_the_row_of_the_address() {
    // The row component is the one the perturbed build drops. A field that
    // ignored it would repeat along every column.
    let field = world(SEED);
    let column = (WIDTH / 2) as i32;
    let mut down: Vec<u32> = (0..HEIGHT as i32)
        .map(|r| {
            field
                .original_stock(Axial::new(column, r), ResourceKind::Food)
                .expect("the address is inside the world")
                .0
        })
        .collect();
    down.dedup();
    assert!(down.len() > 1, "the row does not reach the stock draw");
}

#[test]
fn the_stock_depends_on_the_kind() {
    // The kind is the draw index of the key. Two kinds that shared an index
    // would hold the same amount on every tile that carries both.
    let field = world(SEED);
    let food = stocks(&field, ResourceKind::Food);
    let stone = stocks(&field, ResourceKind::Stone);
    assert_ne!(food, stone, "the kind does not reach the stock draw");
}

#[test]
fn the_field_reads_the_same_in_any_order() {
    // The field is a pure function of the seed and the address, so a caller
    // that walks the world backwards reads the same world.
    let field = world(SEED);
    let forwards = stocks(&field, ResourceKind::Wood);
    let mut backwards: Vec<u32> = addresses()
        .into_iter()
        .rev()
        .map(|address| {
            field
                .original_stock(address, ResourceKind::Wood)
                .expect("the address is inside the world")
                .0
        })
        .collect();
    backwards.reverse();
    assert_eq!(forwards, backwards);
}

#[test]
fn an_address_outside_the_world_holds_no_stock() {
    let field = world(SEED);
    assert_eq!(
        field.original_stock(Axial::new(-1, 0), ResourceKind::Food),
        None
    );
    assert_eq!(
        field.tile_stock(Axial::new(WIDTH as i32, 0), ResourceKind::Food),
        None
    );
}

#[test]
fn water_holds_nothing() {
    // The ground decides what a tile carries, and open water carries nothing
    // of any kind.
    let field = world(SEED);
    let mut water = 0;
    for address in addresses() {
        if field.tile_kind(address) != Some(TileKind::Water) {
            continue;
        }
        water += 1;
        for kind in ResourceKind::ALL {
            assert_eq!(
                field.original_stock(address, kind),
                Some(Amount::ZERO),
                "water at ({}, {}) carries a resource",
                address.q,
                address.r
            );
        }
    }
    assert!(
        water > 0,
        "the world holds no water, so the test proves nothing"
    );
}

#[test]
fn a_wooded_tile_carries_more_wood_than_open_ground() {
    // The terrain must influence the field. A resource spread evenly would
    // give the same mean on every ground.
    let field = world(SEED);
    let mut forest = (0i64, 0i64);
    let mut plain = (0i64, 0i64);
    for address in addresses() {
        let wood = i64::from(
            field
                .original_stock(address, ResourceKind::Wood)
                .expect("the address is inside the world")
                .0,
        );
        match field.tile_kind(address) {
            Some(TileKind::Forest) => {
                forest.0 += wood;
                forest.1 += 1;
            }
            Some(TileKind::Plain) => {
                plain.0 += wood;
                plain.1 += 1;
            }
            _ => {}
        }
    }
    assert!(
        forest.1 > 0 && plain.1 > 0,
        "the world holds one ground only"
    );
    // The comparison is exact, and it holds no division. Two integer means
    // compare by cross-multiplication. The wooded mean must be well above the
    // open mean, because a small difference is what a uniform field with
    // noise in it also produces.
    assert!(
        forest.0 * plain.1 > 2 * plain.0 * forest.1,
        "a wooded tile holds no more wood than open ground"
    );
}

#[test]
fn a_resource_is_not_spread_evenly() {
    // Most tiles carry nothing of a kind. A field that gave every tile a
    // deposit would make no place worth going to.
    let field = world(SEED);
    let stone = stocks(&field, ResourceKind::Stone);
    let carrying = stone.iter().filter(|amount| **amount > 0).count();
    assert!(carrying > 0, "no tile carries stone");
    assert!(
        carrying * 3 < stone.len(),
        "{carrying} of {} tiles carry stone, so the field is spread evenly",
        stone.len()
    );
}

#[test]
fn a_deposit_that_exists_holds_at_least_one() {
    // A deposit of nothing is the same as no deposit. Two ways to say one
    // thing is a defect shape this project keeps meeting.
    let field = world(SEED);
    let ledger = field.depletion();
    assert!(ledger.is_empty());
    for kind in ResourceKind::ALL {
        for amount in stocks(&field, kind) {
            assert!(amount == 0 || amount >= 1);
        }
    }
}

/// Returns the first address that carries a stock of the kind.
fn deposit(world: &World, kind: ResourceKind) -> Axial {
    addresses()
        .into_iter()
        .find(|address| {
            world.admits_a_unit(*address)
                && world.original_stock(*address, kind) > Some(Amount::ZERO)
        })
        .expect("the world must hold a deposit on open ground")
}

#[test]
fn a_world_that_gathered_nothing_stores_nothing() {
    // The engine stores what was taken, and nothing else. A world nobody
    // gathered in must therefore hold no entry at all.
    let mut field = world(SEED);
    for _ in 0..4 {
        field.step(2).expect("the step must run");
    }
    assert!(field.check_invariants());
    assert!(
        field.depletion().is_empty(),
        "the world stored {} entries without a gather",
        field.depletion().len()
    );
}

#[test]
fn a_unit_takes_from_the_tile_it_stands_on() {
    // Drive the engine, then inspect the world. A resolve that no step
    // reaches ships inert.[^1]
    //
    // [^1]: Testing rules, section 5. `.claude/rules/testing.md`
    let mut field = world(SEED);
    let kind = ResourceKind::Wood;
    let address = deposit(&field, kind);
    let before = field
        .tile_stock(address, kind)
        .expect("the address is inside the world");

    let unit = field
        .spawn_soldier(address, FactionId(0))
        .expect("the ground admits a unit");
    assert!(field.order_gather(unit, kind));
    assert_eq!(field.gather_order(unit), Some(Some(kind)));
    field.step(1).expect("the step must run");

    // The unit may have moved, so the tile it took from is the tile the event
    // names. The step gathers after the barrier of its frame.
    let log = field.gather_log().to_vec();
    assert_eq!(log.len(), 1, "the unit took nothing");
    let event = log[0];
    assert_eq!(event.unit, unit.to_bits());
    assert_eq!(ResourceKind::from_u8(event.kind), Some(kind));
    assert!(event.amount > 0);

    let carried = field
        .soldier_carry(unit)
        .expect("the unit is live")
        .of(kind);
    assert_eq!(carried.0, event.amount);

    // The unit took from the tile it stands on now, and not from the tile it
    // left. The resolve runs after the barrier of its frame, so the tile in
    // the event is where the unit ended the frame.
    let standing = field.soldiers().tile(unit).expect("the unit is live");
    assert_eq!(
        event.tile, standing,
        "the unit took from a tile it does not stand on"
    );

    // The stock of that tile fell by exactly what the unit took.
    let where_it_took = field
        .grid()
        .address_of(event.tile)
        .expect("the event names a tile");
    let taken = field
        .taken_from(where_it_took, kind)
        .expect("the address is inside the world");
    assert_eq!(taken.0, event.amount);
    if where_it_took == address {
        let after = field
            .tile_stock(address, kind)
            .expect("the address is inside the world");
        assert_eq!(after.0 + event.amount, before.0);
    }
    assert!(field.check_invariants());
}

#[test]
fn a_unit_with_no_order_takes_nothing() {
    let mut field = world(SEED);
    let kind = ResourceKind::Wood;
    let address = deposit(&field, kind);
    let unit = field
        .spawn_soldier(address, FactionId(0))
        .expect("the ground admits a unit");
    field.step(1).expect("the step must run");
    assert!(field.gather_log().is_empty());
    assert_eq!(
        field.soldier_carry(unit),
        Some(cachette_core::CarryLoad::EMPTY)
    );
    assert!(field.depletion().is_empty());
}

#[test]
fn a_stopped_unit_takes_nothing_more() {
    let mut field = world(SEED);
    let kind = ResourceKind::Wood;
    let address = deposit(&field, kind);
    let unit = field
        .spawn_soldier(address, FactionId(0))
        .expect("the ground admits a unit");
    assert!(field.order_gather(unit, kind));
    field.step(1).expect("the step must run");
    let after_one = field.soldier_carry(unit).expect("the unit is live");
    assert!(field.stop_gather(unit));
    assert_eq!(field.gather_order(unit), Some(None));
    field.step(1).expect("the step must run");
    assert!(field.gather_log().is_empty());
    assert_eq!(field.soldier_carry(unit), Some(after_one));
}

/// Fills one deposit with gatherers and returns the tile, the kind and the
/// units.
///
/// The units all stand on one tile and all gather one kind, so they contend
/// for one deposit. The fixture asserts that the demand exceeds the stock; a
/// fixture that only assumed it would measure itself.[^1]
///
/// # References
///
/// [^1]: Findings register, FND-051. `docs/FINDINGS.md`
fn contended(field: &mut World) -> (Axial, ResourceKind, Vec<Entity>) {
    let kind = ResourceKind::Wood;
    let address = deposit(field, kind);
    let capacity = field.tile_kind(address).map_or(0, TileKind::capacity);
    assert!(capacity > 1, "one gatherer contends with nobody");
    let mut units = Vec::new();
    for ordinal in 0..capacity {
        let unit = field
            .spawn_soldier(address, FactionId((ordinal % 4) as u16))
            .expect("the ground admits a unit");
        assert!(field.order_gather(unit, kind));
        units.push(unit);
    }
    (address, kind, units)
}

#[test]
fn two_units_cannot_take_the_same_unit_of_resource() {
    let mut field = world(SEED);
    let (address, kind, units) = contended(&mut field);
    let stock = field
        .tile_stock(address, kind)
        .expect("the address is inside the world");
    // The fixture must produce the contested case. The demand of the crowd
    // must exceed what the deposit holds, or nobody is refused and the test
    // measures nothing.
    let demand = units.len() as u32 * 4;
    assert!(
        demand > stock.0,
        "the crowd demands {demand} and the deposit holds {}, so nobody contends",
        stock.0
    );

    field.step(1).expect("the step must run");

    // Every grant of the frame came from this deposit, and they sum to what
    // it held. Nothing was created and nothing was lost.
    let tile = field.grid().index_of(address).expect("inside");
    let granted: u32 = field
        .gather_log()
        .iter()
        .filter(|event| event.tile == tile)
        .map(|event| event.amount)
        .sum();
    assert_eq!(granted, stock.0, "the grants do not sum to the deposit");
    assert_eq!(
        field.tile_stock(address, kind),
        Some(Amount::ZERO),
        "the deposit is not empty"
    );
    // At least one gatherer was refused or served short. That is the case the
    // resolve exists for.
    assert!(
        field.gather_log().len() < units.len(),
        "every gatherer took a full share, so nobody contended"
    );
    assert!(field.check_invariants());
}

#[test]
fn an_empty_deposit_gives_nothing() {
    let mut field = world(SEED);
    let (address, kind, _) = contended(&mut field);
    field.step(1).expect("the step must run");
    assert_eq!(field.tile_stock(address, kind), Some(Amount::ZERO));
    let taken = field
        .taken_from(address, kind)
        .expect("the address is inside the world");
    field.step(1).expect("the step must run");
    // The second step may find the gatherers elsewhere, so the assertion is
    // that this tile gave nothing more.
    assert_eq!(field.taken_from(address, kind), Some(taken));
    assert!(field.check_invariants());
}

#[test]
fn every_unit_takes_from_the_tile_the_frame_left_it_on() {
    // The resolve runs after the barrier of its frame, so a unit takes from
    // the tile it ends the frame on and never from the tile it left. A
    // resolve that ran first would give a wrong answer that repeats
    // perfectly, so no determinism test could see it.
    //
    // **This test takes its order from the engine and not from a caller.** A
    // unit only moves when it holds an intent, and the pass that writes the
    // intent writes the gather order beside it, so a fixture that ordered from
    // outside and then let the units choose would have its order replaced.[^1]
    // The units are therefore made hungry, every weight but the one on the
    // foraging option is removed, and the engine orders the gather.
    //
    // [^1]: Findings register, FND-211. `docs/FINDINGS.md`
    let mut field = world(SEED);
    let kind = ResourceKind::Food;
    field
        .set_choice_schedule(0)
        .expect("the exponent is inside the range");
    make_them_hungry(&mut field);
    let patch: Vec<Axial> = addresses()
        .into_iter()
        .filter(|address| field.admits_a_unit(*address))
        .take(96)
        .collect();
    let mut units = Vec::new();
    for address in patch {
        let unit = field
            .spawn_soldier(address, FactionId(0))
            .expect("the ground admits a unit");
        units.push((unit, address));
    }

    // The need falls in a pass that runs after the choice, so the first frame
    // reads a unit that still holds a whole need. The second frame is the one
    // on which the engine orders the gather.
    field.step(1).expect("the step must run");
    field.step(1).expect("the step must run");
    let units: Vec<(Entity, Axial)> = units
        .iter()
        .filter_map(|(unit, _)| Some((*unit, field.soldiers().address(*unit)?)))
        .collect();
    // A unit whose cell carries no food scores zero and orders nothing. The
    // fixture needs gatherers, and it says how many it found.
    let ordered = units
        .iter()
        .filter(|(unit, _)| field.gather_order(*unit) == Some(Some(kind)))
        .count();
    assert!(
        ordered > 0,
        "the engine ordered no gather, so the fixture holds no gatherer"
    );

    field.step(1).expect("the step must run");

    // The fixture must hold units that moved. A population that all stayed
    // put would pass whatever order the step ran in.
    let moved = units
        .iter()
        .filter(|(unit, from)| field.soldiers().address(*unit) != Some(*from))
        .count();
    assert!(moved > 0, "no unit moved, so the frame order is not tested");

    assert!(!field.gather_log().is_empty(), "no unit took anything");
    for event in field.gather_log() {
        let unit = units
            .iter()
            .find(|(unit, _)| unit.to_bits() == event.unit)
            .expect("the event names a unit of this test")
            .0;
        assert_eq!(
            Some(event.tile),
            field.soldiers().tile(unit),
            "a unit took from a tile it does not stand on"
        );
    }
    assert!(field.check_invariants());
}

#[test]
fn what_a_dead_unit_carried_leaves_the_world_on_the_record() {
    // Conservation must still balance when a unit dies with a load. The
    // world records where the load went rather than letting it disappear.
    let mut field = world(SEED);
    let kind = ResourceKind::Wood;
    let address = deposit(&field, kind);
    let unit = field
        .spawn_soldier(address, FactionId(0))
        .expect("the ground admits a unit");
    assert!(field.order_gather(unit, kind));
    field.step(1).expect("the step must run");
    let carried = field
        .soldier_carry(unit)
        .expect("the unit is live")
        .of(kind)
        .0;
    assert!(carried > 0);
    assert!(field.despawn_soldier(unit));
    assert_eq!(field.departed_carry()[kind.index()], u64::from(carried));
    assert!(field.check_invariants());
}

#[test]
fn a_reused_slot_starts_empty() {
    // A slot that kept the load of the dead soldier would hand it to the next
    // one, and conservation would then balance against a unit that never
    // gathered.
    let mut field = world(SEED);
    let kind = ResourceKind::Wood;
    let address = deposit(&field, kind);
    let first = field
        .spawn_soldier(address, FactionId(0))
        .expect("the ground admits a unit");
    assert!(field.order_gather(first, kind));
    field.step(1).expect("the step must run");
    assert!(field.despawn_soldier(first));
    let second = field
        .spawn_soldier(address, FactionId(0))
        .expect("the ground admits a unit");
    assert_eq!(second.index(), first.index(), "the arena opened a new slot");
    assert_ne!(second.generation(), first.generation());
    assert_eq!(
        field.soldier_carry(second),
        Some(cachette_core::CarryLoad::EMPTY)
    );
    assert_eq!(field.gather_order(second), Some(None));
    assert!(field.check_invariants());
}

#[test]
fn the_gather_events_reach_the_state_hash() {
    // A hash that ignored the ledger and the loads would pass the golden test
    // while the resources moved underneath it.
    let mut gathered = world(SEED);
    let kind = ResourceKind::Wood;
    let address = deposit(&gathered, kind);
    let unit = gathered
        .spawn_soldier(address, FactionId(0))
        .expect("the ground admits a unit");
    assert!(gathered.order_gather(unit, kind));
    gathered.step(1).expect("the step must run");

    let mut idle = world(SEED);
    let same = idle
        .spawn_soldier(address, FactionId(0))
        .expect("the ground admits a unit");
    assert_eq!(same, unit);
    idle.step(1).expect("the step must run");

    assert_ne!(gathered.state_hash(), idle.state_hash());
}

/// Returns an island tile that carries a stock of the kind.
///
/// An island is a tile whose every neighbour refuses a unit, so a unit on it
/// never moves. The resolve then reads the same tile on every frame, and the
/// test can put two named units on one deposit and know they stay there.
fn island(world: &World, kind: ResourceKind) -> Option<Axial> {
    addresses().into_iter().find(|address| {
        world.admits_a_unit(*address)
            && world.original_stock(*address, kind) >= Some(Amount(5))
            && world
                .grid()
                .neighbours(*address)
                .iter()
                .all(|side| side.is_none_or(|next| !world.admits_a_unit(next)))
    })
}

#[test]
fn the_resolve_breaks_a_tie_on_the_identity_and_not_on_the_slot() {
    // A slot is reused after a unit dies, so a new unit would otherwise
    // inherit the position that the dead unit held in a contest for a
    // deposit. The generation sits above the slot index in the identity, so a
    // replacement sorts above the unit that outlived the one it replaced.
    //
    // The defect repeats on every run and at every thread count, so no
    // determinism test can see it.[^1]
    //
    // [^1]: Testing rules, section 2. `.claude/rules/testing.md`
    // The seed is chosen for the island, not for realism. A fixture that took
    // the usual seed would hold no island, and the case would then be
    // untested while the test read as though it covered it.[^2]
    //
    // [^2]: Findings register, FND-051. `docs/FINDINGS.md`
    let mut field = world(ISLAND_SEED);
    let kind = ResourceKind::Wood;
    let Some(address) = island(&field, kind) else {
        // The world holds no island that carries wood. The case cannot be
        // built here, and a test that silently passed would say the rule
        // holds.
        panic!("the world holds no island deposit, so the fixture cannot reach the case");
    };

    let first = field
        .spawn_soldier(address, FactionId(0))
        .expect("the ground admits a unit");
    let survivor = field
        .spawn_soldier(address, FactionId(0))
        .expect("the ground admits a unit");
    assert!(field.despawn_soldier(first));
    let replacement = field
        .spawn_soldier(address, FactionId(0))
        .expect("the ground admits a unit");

    // The replacement took the slot of the unit that died, so its slot is
    // below the slot of the survivor and its identity is above.
    assert_eq!(replacement.index(), first.index());
    assert!(replacement.index() < survivor.index());
    assert!(replacement.to_bits() > survivor.to_bits());

    assert!(field.order_gather(survivor, kind));
    assert!(field.order_gather(replacement, kind));
    field.step(1).expect("the step must run");

    // Both units still stand on the island, and both took a share.
    assert_eq!(field.soldiers().address(survivor), Some(address));
    assert_eq!(field.soldiers().address(replacement), Some(address));
    let log = field.gather_log().to_vec();
    assert_eq!(log.len(), 2, "the deposit did not serve both gatherers");
    assert_eq!(
        log[0].unit,
        survivor.to_bits(),
        "the resolve served the replacement first, so it broke the tie on the slot"
    );
    assert_eq!(log[1].unit, replacement.to_bits());
    assert!(field.check_invariants());
}

#[test]
fn the_catalogue_holds_every_kind_once() {
    // The kind numbering is a second declaration of the catalogue, and a
    // number that named no kind would read back as a soldier that gathers
    // nothing.
    assert_eq!(ResourceKind::ALL.len(), RESOURCE_KIND_COUNT);
    for (index, kind) in ResourceKind::ALL.iter().enumerate() {
        assert_eq!(kind.index(), index);
        assert_eq!(ResourceKind::from_u8(kind.to_u8()), Some(*kind));
    }
    assert_eq!(ResourceKind::from_u8(RESOURCE_KIND_COUNT as u8), None);
}

#[test]
fn the_ledger_holds_one_entry_for_each_deposit_that_gave() {
    let mut field = world(SEED);
    let kind = ResourceKind::Wood;
    let address = deposit(&field, kind);
    let unit = field
        .spawn_soldier(address, FactionId(0))
        .expect("the ground admits a unit");
    assert!(field.order_gather(unit, kind));
    field.step(1).expect("the step must run");
    assert_eq!(field.depletion().len(), 1);
    let tile: TileIdx = field.grid().index_of(address).expect("inside");
    assert!(field.depletion().taken(tile, kind).0 > 0);
    assert_eq!(
        field.depletion().taken(tile, ResourceKind::Stone),
        Amount::ZERO
    );
    assert!(field.depletion().check_invariants());
}

// The recovery of a depleted deposit.
//
// Recovery is not growth of an amount. The world stores what units took, and
// recovery ages that stored take away, so a smaller stored take is a fuller
// deposit.[^R1] The tests below drive the engine and then read the world
// through the public interface.[^R2]
//
// The fixture is built for the extremes and not for the typical case. It holds
// a deposit that units emptied, a deposit they only reduced, a stone deposit
// that must never recover, and deposits that nobody touched.[^R3]
//
// [^R1]: ADR-0080, a depleted deposit recovers by ageing the stored take. `docs/adrs/accepted/adr-0080-a-depleted-deposit-recovers-by-ageing-the-stored-take.md`
// [^R2]: Testing rules, sections 5 and 6. `.claude/rules/testing.md`
// [^R3]: Testing rules, section 2a. `.claude/rules/testing.md`

/// The period of food in the tests, in ticks.
const FOOD_PERIOD: u32 = 3;
/// The period of wood in the tests, in ticks.
const WOOD_PERIOD: u32 = 2;

/// The rules that the recovery tests read.
///
/// The periods are short, so a test reaches the case in a few frames. The
/// period is a parameter of the kind, and a caller replaces the whole rule
/// set, so a test states no period that the engine also states.
fn quick_rules() -> RecoveryRules {
    RecoveryRules::from_ticks([Some(FOOD_PERIOD), Some(WOOD_PERIOD), None])
        .expect("no period is zero")
}

/// A world in which units have worked some deposits, and the cases it holds.
///
/// The fixture drives the engine. It empties one deposit with a crowd, and it
/// reduces others with single gatherers. It then stops every gatherer, so
/// nothing takes anything more and recovery is the only thing that moves.
struct Worked {
    /// The world.
    field: World,
    /// A deposit that units emptied.
    emptied: (Axial, ResourceKind),
    /// A deposit that units reduced but did not empty.
    partial: (Axial, ResourceKind),
    /// A stone deposit that units took from.
    stone: (Axial, ResourceKind),
    /// A deposit that nobody touched.
    untouched: (Axial, ResourceKind),
}

/// Returns addresses that carry at least the amount of the kind.
fn deposits(field: &World, kind: ResourceKind, least: u32, count: usize) -> Vec<Axial> {
    addresses()
        .into_iter()
        .filter(|address| {
            field.admits_a_unit(*address)
                && field.original_stock(*address, kind) >= Some(Amount(least))
        })
        .take(count)
        .collect()
}

/// Builds the worked world.
///
/// The world recovers nothing while the units work it, so the fixture states
/// what was taken without recovery moving underneath it. The caller sets the
/// rules it wants afterwards.
fn worked() -> Worked {
    let mut field = world(SEED);
    field.set_recovery_rules(RecoveryRules::NONE);

    // A crowd empties one wood deposit in one frame.
    let (_, _, mut units) = contended(&mut field);
    // Single gatherers reduce other deposits. Several of each kind, because a
    // unit may move before the resolve runs and then take from another tile.
    for (kind, least) in [(ResourceKind::Food, 5), (ResourceKind::Stone, 5)] {
        for address in deposits(&field, kind, least, 8) {
            let unit = field
                .spawn_soldier(address, FactionId(0))
                .expect("the ground admits a unit");
            assert!(field.order_gather(unit, kind));
            units.push(unit);
        }
    }
    field.step(2).expect("the step must run");
    for unit in &units {
        field.stop_gather(*unit);
    }

    // Classify what the frame produced. The units may have moved, so the
    // ledger is the truth about which deposits gave.
    let (mut emptied, mut partial, mut stone) = (None, None, None);
    for entry in field.depletion().entries() {
        let kind = ResourceKind::from_u8((entry.key & 0b11) as u8).expect("the key names a kind");
        let tile = TileIdx((entry.key >> 2) as u32);
        let address = field.grid().address_of(tile).expect("the key names a tile");
        let original = field
            .original_stock(address, kind)
            .expect("the address is inside the world")
            .0;
        if kind == ResourceKind::Stone {
            stone = stone.or(Some((address, kind)));
        } else if entry.taken == original {
            emptied = emptied.or(Some((address, kind)));
        } else if entry.taken > 0 {
            partial = partial.or(Some((address, kind)));
        }
    }
    // A fixture that reached none of these cases would measure itself.[^1]
    //
    // [^1]: Testing rules, section 2a. `.claude/rules/testing.md`
    let emptied = emptied.expect("the fixture holds no emptied deposit");
    let partial = partial.expect("the fixture holds no partly depleted deposit");
    let stone = stone.expect("the fixture holds no worked stone deposit");
    let untouched = *deposits(&field, ResourceKind::Food, 3, 4096)
        .iter()
        .find(|address| field.taken_from(**address, ResourceKind::Food) == Some(Amount::ZERO))
        .expect("the fixture holds no untouched deposit");
    Worked {
        field,
        emptied,
        partial,
        stone,
        untouched: (untouched, ResourceKind::Food),
    }
}

#[test]
fn the_fixture_holds_the_cases_that_the_tests_need() {
    // The fixture is the input of every test below. A fixture that reached
    // only the typical case would never supply the extreme where a defect
    // lives.[^1]
    //
    // [^1]: Testing rules, section 2a. `.claude/rules/testing.md`
    let worked = worked();
    assert_eq!(
        worked.field.tile_stock(worked.emptied.0, worked.emptied.1),
        Some(Amount::ZERO),
        "the emptied deposit holds something"
    );
    let partial_stock = worked
        .field
        .tile_stock(worked.partial.0, worked.partial.1)
        .expect("inside");
    let partial_original = worked
        .field
        .original_stock(worked.partial.0, worked.partial.1)
        .expect("inside");
    assert!(partial_stock > Amount::ZERO);
    assert!(partial_stock < partial_original);
    assert!(worked.field.taken_from(worked.stone.0, worked.stone.1) > Some(Amount::ZERO));
    assert_eq!(
        worked
            .field
            .taken_from(worked.untouched.0, worked.untouched.1),
        Some(Amount::ZERO)
    );
    // The world is far larger than the worked set, so most tiles hold no
    // stored take at all.
    assert!(
        (worked.field.depletion().len() as u32) < worked.field.grid().tile_count() / 100,
        "the fixture worked {} of {} tiles",
        worked.field.depletion().len(),
        worked.field.grid().tile_count()
    );
}

#[test]
fn a_depleted_deposit_holds_more_at_a_later_tick() {
    // The need this work answers. A deposit that a unit took from must hold
    // more later, when nothing takes from it again.
    let mut worked = worked();
    worked.field.set_recovery_rules(quick_rules());
    let (address, kind) = worked.partial;
    let before = worked.field.tile_stock(address, kind).expect("inside");
    for _ in 0..FOOD_PERIOD.max(WOOD_PERIOD) {
        worked.field.step(2).expect("the step must run");
    }
    let after = worked.field.tile_stock(address, kind).expect("inside");
    assert!(
        after > before,
        "the deposit held {} and now holds {}",
        before.0,
        after.0
    );
    assert!(worked.field.check_invariants());
}

#[test]
fn a_deposit_that_units_emptied_recovers() {
    // A deposit that reached nothing recovers in the same way as any other
    // depleted deposit.[^1]
    //
    // [^1]: Decisions register, DEC-050. `docs/DECISIONS.md`
    let mut worked = worked();
    worked.field.set_recovery_rules(quick_rules());
    let (address, kind) = worked.emptied;
    assert_eq!(worked.field.tile_stock(address, kind), Some(Amount::ZERO));
    for _ in 0..(WOOD_PERIOD.max(FOOD_PERIOD) * 2) {
        worked.field.step(2).expect("the step must run");
    }
    assert!(
        worked.field.tile_stock(address, kind) > Some(Amount::ZERO),
        "the emptied deposit stayed empty"
    );
    assert!(worked.field.check_invariants());
}

#[test]
fn recovery_waits_for_the_whole_period() {
    // The period is the simulated time in which a deposit regains one unit. A
    // rule that ignored the elapsed ticks would give the unit back on the
    // first frame, and a rule that dropped the remainder of the division
    // would never give it back at all, because the pass runs on every tick.
    let mut worked = worked();
    let (address, kind) = worked.partial;
    let period = match kind {
        ResourceKind::Food => FOOD_PERIOD,
        _ => WOOD_PERIOD,
    };
    worked.field.set_recovery_rules(quick_rules());
    let start = worked.field.taken_from(address, kind).expect("inside").0;
    assert!(start >= 3, "the fixture took too little to measure a rate");
    for _ in 0..(period - 1) {
        worked.field.step(2).expect("the step must run");
        assert_eq!(
            worked.field.taken_from(address, kind),
            Some(Amount(start)),
            "the deposit recovered before the period had passed"
        );
    }
    worked.field.step(2).expect("the step must run");
    assert_eq!(
        worked.field.taken_from(address, kind),
        Some(Amount(start - 1)),
        "the deposit did not regain one unit at the period"
    );
    for _ in 0..period {
        worked.field.step(2).expect("the step must run");
    }
    assert_eq!(
        worked.field.taken_from(address, kind),
        Some(Amount(start - 2)),
        "the deposit did not regain a second unit at the second period"
    );
}

#[test]
fn a_deposit_never_holds_more_than_it_started_with() {
    // Recovery returns a deposit toward what the generator gave it, and never
    // past it.
    let mut worked = worked();
    worked.field.set_recovery_rules(quick_rules());
    for _ in 0..64 {
        worked.field.step(2).expect("the step must run");
        for entry in worked.field.depletion().entries() {
            let kind =
                ResourceKind::from_u8((entry.key & 0b11) as u8).expect("the key names a kind");
            let tile = TileIdx((entry.key >> 2) as u32);
            let address = worked
                .field
                .grid()
                .address_of(tile)
                .expect("the key names a tile");
            let original = worked
                .field
                .original_stock(address, kind)
                .expect("inside")
                .0;
            let stock = worked.field.tile_stock(address, kind).expect("inside").0;
            assert!(
                stock <= original,
                "the deposit holds {stock} against a start of {original}"
            );
        }
    }
    // Every deposit that recovers has returned to what it started with, and a
    // recovered deposit is not different from one that nobody touched.
    let (address, kind) = worked.partial;
    assert_eq!(
        worked.field.tile_stock(address, kind),
        worked.field.original_stock(address, kind)
    );
    assert!(worked.field.check_invariants());
}

#[test]
fn recovery_never_gives_back_more_than_units_took() {
    // Recovery creates nothing. The total it returns is bounded by the total
    // the units took, for each kind on its own.
    let mut worked = worked();
    worked.field.set_recovery_rules(quick_rules());
    let mut took = [0i64; RESOURCE_KIND_COUNT];
    for event in worked.field.gather_log() {
        let kind = ResourceKind::from_u8(event.kind).expect("the event names a kind");
        took[kind.index()] += i64::from(event.amount);
    }
    for _ in 0..64 {
        worked.field.step(2).expect("the step must run");
        for event in worked.field.gather_log() {
            let kind = ResourceKind::from_u8(event.kind).expect("the event names a kind");
            took[kind.index()] += i64::from(event.amount);
        }
        for kind in ResourceKind::ALL {
            let given = worked.field.depletion().returned(kind).0;
            assert!(
                given <= took[kind.index()],
                "recovery returned {given} of {kind:?} against a take of {}",
                took[kind.index()]
            );
        }
        assert!(worked.field.check_invariants());
    }
    assert!(
        worked.field.depletion().returned(ResourceKind::Food).0 > 0
            || worked.field.depletion().returned(ResourceKind::Wood).0 > 0,
        "nothing recovered, so the bound was never tested"
    );
}

#[test]
fn a_kind_that_states_no_period_does_not_recover() {
    // Stone does not recover. The absent case is a real case, and the engine
    // carries it from the first day.[^1]
    //
    // [^1]: Decisions register, DEC-049. `docs/DECISIONS.md`
    let mut worked = worked();
    worked.field.set_recovery_rules(quick_rules());
    let (address, kind) = worked.stone;
    let took = worked.field.taken_from(address, kind).expect("inside");
    assert!(took > Amount::ZERO);
    for _ in 0..64 {
        worked.field.step(2).expect("the step must run");
    }
    assert_eq!(
        worked.field.taken_from(address, kind),
        Some(took),
        "the stone deposit recovered"
    );
    assert_eq!(
        worked.field.depletion().returned(kind),
        cachette_core::Accum(0)
    );
}

#[test]
fn the_default_rules_state_a_period_for_each_kind_in_one_place() {
    // The period of a kind is one named parameter. A kind may state that it
    // does not recover, and stone does.[^1]
    //
    // [^1]: Decisions register, DEC-049. `docs/DECISIONS.md`
    let field = world(SEED);
    let rules = field.recovery_rules();
    assert_eq!(rules, RecoveryRules::DEFAULT);
    assert!(rules.period_of(ResourceKind::Food).is_some());
    assert!(rules.period_of(ResourceKind::Wood).is_some());
    assert_eq!(rules.period_of(ResourceKind::Stone), None);
    // The period is stated in simulated days and converted to ticks in one
    // place, so a change to the span of a tick moves every period together.
    for kind in [ResourceKind::Food, ResourceKind::Wood] {
        let period = rules.period_of(kind).expect("the kind recovers");
        assert_eq!(period % TICKS_IN_A_SIMULATED_DAY, 0);
    }
    // A period of zero is refused, because it is a second way to say that a
    // deposit was never depleted.
    assert!(RecoveryRules::from_ticks([Some(0), None, None]).is_none());
}

#[test]
fn a_world_that_gathered_nothing_does_no_recovery_work() {
    // A tile that nobody gathered from holds no stored take, so recovery has
    // nothing to age. The cost follows the depleted set and not the tile
    // count.
    let mut field = world(SEED);
    field.set_recovery_rules(quick_rules());
    for _ in 0..8 {
        field.step(2).expect("the step must run");
    }
    assert!(field.depletion().is_empty());
    assert_eq!(
        field.depletion().last_recovery_visits(),
        0,
        "recovery read something in a world that stored nothing"
    );
}

#[test]
fn the_recovery_work_does_not_grow_with_the_extent() {
    // The work of one recovery pass follows the number of depleted deposits.
    // Two worlds that differ only in extent, and that hold the same worked
    // deposits, do the same work.
    fn worked_visits(width: u32, height: u32) -> (u64, usize) {
        let mut field = World::new(WorldConfig {
            width,
            height,
            seed: SEED,
            faction_count: 4,
            unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
        })
        .expect("the extent must describe a world");
        hold_the_choice(&mut field);
        field.set_recovery_rules(quick_rules());
        let kind = ResourceKind::Wood;
        let address = addresses()
            .into_iter()
            .filter(|address| address.q < width as i32 && address.r < height as i32)
            .find(|address| {
                field.admits_a_unit(*address)
                    && field.original_stock(*address, kind) > Some(Amount::ZERO)
            })
            .expect("the world holds a deposit");
        let unit = field
            .spawn_soldier(address, FactionId(0))
            .expect("the ground admits a unit");
        assert!(field.order_gather(unit, kind));
        field.step(2).expect("the step must run");
        field.stop_gather(unit);
        field.step(2).expect("the step must run");
        (
            field.depletion().last_recovery_visits(),
            field.depletion().len(),
        )
    }
    let small = worked_visits(64, 64);
    let large = worked_visits(WIDTH, HEIGHT);
    assert!(small.1 > 0 && large.1 > 0, "neither world worked a deposit");
    assert_eq!(
        small.0, large.0,
        "the recovery pass read {} entries in the small world and {} in the large one",
        small.0, large.0
    );
}

#[test]
fn reading_the_world_does_not_change_it() {
    // A read must not move the world forward. Two reads at one tick give one
    // answer, and the state hash does not move because somebody looked.
    let mut worked = worked();
    worked.field.set_recovery_rules(quick_rules());
    worked.field.step(2).expect("the step must run");
    let (address, kind) = worked.partial;
    let before = worked.field.state_hash().finish();
    let first = worked.field.tile_stock(address, kind);
    let second = worked.field.tile_stock(address, kind);
    assert_eq!(first, second);
    assert_eq!(
        worked.field.taken_from(address, kind),
        worked.field.taken_from(address, kind)
    );
    assert_eq!(worked.field.state_hash().finish(), before);
}

#[test]
fn a_gather_takes_what_the_deposit_holds_at_that_tick() {
    // Admission reads the recovered amount and not the raw stored take. A
    // resolve that read the stale amount would let a unit take what the tile
    // does not hold, and conservation is what fails.
    let mut worked = worked();
    worked.field.set_recovery_rules(quick_rules());
    let (address, kind) = worked.emptied;
    let original = worked.field.original_stock(address, kind).expect("inside");
    // Let the emptied deposit recover, and stop at the first frame on which it
    // holds something. A fixed number of frames would fill a small deposit and
    // the case would then be the full deposit rather than the partial one.
    let mut holding = Amount::ZERO;
    for _ in 0..(WOOD_PERIOD * 2) {
        worked.field.step(2).expect("the step must run");
        holding = worked.field.tile_stock(address, kind).expect("inside");
        if holding > Amount::ZERO {
            break;
        }
    }
    assert!(holding > Amount::ZERO, "the deposit recovered nothing");
    assert!(
        holding < original,
        "the deposit is already full: it holds {} of {}",
        holding.0,
        original.0
    );

    let unit = worked
        .field
        .spawn_soldier(address, FactionId(0))
        .expect("the ground admits a unit");
    assert!(worked.field.order_gather(unit, kind));
    worked.field.step(2).expect("the step must run");
    let took: u32 = worked
        .field
        .gather_log()
        .iter()
        .filter(|event| Some(event.tile) == worked.field.grid().index_of(address))
        .map(|event| event.amount)
        .sum();
    // The deposit recovers one more unit on the frame that the unit gathers
    // on, because recovery runs before the resolve. The unit therefore takes
    // no more than the deposit holds at that tick.
    assert!(
        took <= holding.0 + 1,
        "the unit took {took} from a deposit holding {}",
        holding.0
    );
    assert!(worked.field.check_invariants());
}
