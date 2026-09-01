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

use cachette_core::resource::{Amount, ResourceKind, RESOURCE_KIND_COUNT};
use cachette_core::terrain::TileKind;
use cachette_core::{Axial, Entity, FactionId, TileIdx, World, WorldConfig};

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
    })
    .expect("the extent must describe a world");
    // The choice interval is not the subject of this file. A unit takes an
    // intent at the interval its level 1 cell schedules, and it does not move
    // before it has one, so a test about movement sets the interval to every
    // tick.[^C]
    //
    // [^C]: ADR-0064, a unit chooses by scoring a small fixed option set, decision D4. `docs/adrs/accepted/adr-0064-a-unit-chooses-by-scoring-a-small-fixed-option-set.md`
    world
        .set_choice_schedule(0)
        .expect("the exponent is inside the range");
    world
}

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
    let mut field = world(SEED);
    let kind = ResourceKind::Food;
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
        assert!(field.order_gather(unit, kind));
        units.push((unit, address));
    }

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
