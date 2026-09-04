//! One faction stands on the ground of another, or it does not.
//!
//! The tests go through the public crate API. They drive the world step,
//! because the step is what must derive the relation. A test that called the
//! fold itself would prove that the fold works and not that anything reaches
//! it.[^1]
//!
//! **Every fixture asserts that it produced the case the test needs.** A
//! guest that fails to stand on foreign ground, or a holder that changed
//! hands under it, would leave an assertion that passes for the wrong reason.
//! Each test therefore reads the holder under each unit before it reads the
//! relation.[^2]
//!
//! # References
//!
//! [^1]: Testing rules, drive the real caller. `.claude/rules/testing.md`
//! [^2]: Testing rules, section 2a. `.claude/rules/testing.md`

use cachette_core::holding::Holder;
use cachette_core::{Axial, Entity, FactionId, World, WorldConfig};

/// A world with room for two factions to hold ground far apart.
const TWO_SIDES: WorldConfig = WorldConfig {
    width: 96,
    height: 96,
    seed: 0x00c0_ffee_0123_4567,
    faction_count: 3,
    unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
};

/// The faction that holds the ground in every fixture below.
const HOST: FactionId = FactionId(0);

/// The faction whose unit visits.
const GUEST: FactionId = FactionId(1);

/// How many units of the host stand on the visited tile.
///
/// A challenger raises seven for a unit it stands with. The holder of a tile
/// raises one for each neighbour that holds with it, and seven for each unit
/// of its own that stands there. One host unit is therefore not enough to
/// keep the tile, because the guest matches it and the neighbours may be
/// fewer than six at the edge of a holding. Three host units settle it at
/// every position, so the tile stays with the host and the test measures the
/// relation rather than the spread rule.
const HOST_UNITS_ON_THE_VISITED_TILE: usize = 3;

/// Builds a world in which the host holds a patch of ground.
///
/// Returns the world and the address the host holds in the middle of it.
fn a_world_the_host_holds() -> (World, Axial) {
    let mut world = World::new(TWO_SIDES).expect("the configuration describes a world");
    // A unit takes an intent at the interval its level 1 cell schedules, and
    // it does not move before it has one. Every test here wants the units to
    // act on every tick.[^1]
    //
    // [^1]: ADR-0064, a unit chooses by scoring a small fixed option set, decision D4. `docs/adrs/accepted/adr-0064-a-unit-chooses-by-scoring-a-small-fixed-option-set.md`
    world
        .set_choice_schedule(0)
        .expect("the exponent is inside the range");

    let corner = Axial::new(12, 43);
    for row in 0..8 {
        for column in 0..8 {
            let address = Axial::new(corner.q + column, corner.r + row);
            if world.admits_a_unit(address) {
                let _ = world.spawn_soldier(address, HOST);
            }
        }
    }
    for _ in 0..6 {
        world.step(1).expect("the step must run");
    }

    let middle = Axial::new(corner.q + 4, corner.r + 4);
    assert_eq!(
        world.tile_holder(middle).and_then(Holder::faction),
        Some(HOST),
        "the fixture must leave the host holding the tile the guest visits"
    );
    (world, middle)
}

/// Places units of one faction on one address and returns their identities.
fn stand_at(world: &mut World, address: Axial, faction: FactionId, count: usize) -> Vec<Entity> {
    (0..count)
        .map(|_| {
            world
                .spawn_soldier(address, faction)
                .expect("the ground admits a unit")
        })
        .collect()
}

/// Reports whether the relation says the guest stands on the host's ground.
fn guest_stands_on_host_ground(world: &World) -> bool {
    world
        .stands_in_territory(GUEST, HOST)
        .expect("the relation describes the arena")
}

/// Fails when the fixture did not put the guest on ground the host holds.
///
/// The guest may take a step of its own during the frame, so the check reads
/// where it stands now rather than where it was placed. What the test needs
/// is that the ground under it belongs to the host.
fn assert_the_guest_visits(world: &World, guest: Entity) {
    let standing = world
        .soldiers()
        .address(guest)
        .expect("the guest is a live unit");
    assert_eq!(
        world.tile_holder(standing).and_then(Holder::faction),
        Some(HOST),
        "the fixture must leave the host holding the tile under the guest, \
         and the guest stands at {standing:?}"
    );
}

#[test]
fn a_guest_on_foreign_ground_sets_the_bit_and_leaving_clears_it() {
    let (mut world, middle) = a_world_the_host_holds();
    stand_at(&mut world, middle, HOST, HOST_UNITS_ON_THE_VISITED_TILE);
    let guest = stand_at(&mut world, middle, GUEST, 1)[0];
    world.step(1).expect("the step must run");

    assert_the_guest_visits(&world, guest);
    assert!(
        guest_stands_on_host_ground(&world),
        "the relation must report a guest that stands on the host's ground"
    );

    assert!(world.despawn_soldier(guest), "the guest was live");
    world.step(1).expect("the step must run");
    assert!(
        !guest_stands_on_host_ground(&world),
        "the relation must stop reporting a guest that has gone"
    );
}

#[test]
fn a_unit_on_its_own_ground_sets_no_bit() {
    let (mut world, middle) = a_world_the_host_holds();
    world.step(1).expect("the step must run");

    assert_eq!(
        world.tile_holder(middle).and_then(Holder::faction),
        Some(HOST),
        "the fixture must leave the host holding its own ground"
    );
    assert!(
        !world.soldiers().is_empty(),
        "the fixture must hold units, or it proves nothing"
    );

    let rows = world.presence_rows().expect("the relation is fresh");
    for (host, row) in rows.iter().enumerate() {
        assert!(
            row.is_empty(),
            "row {host} must be empty when every unit stands at home, and it holds {:#x}",
            row.to_bits()
        );
    }
}

#[test]
fn the_relation_is_directed() {
    let (mut world, middle) = a_world_the_host_holds();
    stand_at(&mut world, middle, HOST, HOST_UNITS_ON_THE_VISITED_TILE);
    let guest = stand_at(&mut world, middle, GUEST, 1)[0];
    world.step(1).expect("the step must run");
    assert_the_guest_visits(&world, guest);

    assert!(
        world
            .stands_in_territory(GUEST, HOST)
            .expect("the relation is fresh"),
        "the guest stands on the host's ground"
    );
    assert!(
        !world
            .stands_in_territory(HOST, GUEST)
            .expect("the relation is fresh"),
        "the host stands on no ground of the guest, so the reverse must be false"
    );
    assert!(
        !world
            .stands_in_territory(HOST, HOST)
            .expect("the relation is fresh"),
        "a faction is never a guest on its own ground"
    );
}

#[test]
fn the_answer_depends_on_who_holds_the_ground_under_the_unit() {
    let (mut world, middle) = a_world_the_host_holds();
    stand_at(&mut world, middle, HOST, HOST_UNITS_ON_THE_VISITED_TILE);
    let guest = stand_at(&mut world, middle, GUEST, 1)[0];
    world.step(1).expect("the step must run");
    assert_the_guest_visits(&world, guest);
    assert!(guest_stands_on_host_ground(&world));

    // The guest moves to ground that nobody holds. The unit still exists and
    // still belongs to the guest faction, so a relation that answered "any
    // unit anywhere" would keep the bit set.
    let outside = far_from(&world, middle);
    world
        .place_soldier(guest, outside)
        .expect("the ground admits a unit");
    world.step(1).expect("the step must run");
    assert!(
        world.tile_holder(outside).and_then(Holder::faction) != Some(HOST),
        "the fixture must move the guest off the host's ground"
    );
    assert!(
        !guest_stands_on_host_ground(&world),
        "the relation must follow the ground under the unit"
    );
}

/// Returns an address that nobody holds, far from a given one.
fn far_from(world: &World, from: Axial) -> Axial {
    let grid = world.grid();
    for row in 0..grid.height() as i32 {
        for column in 0..grid.width() as i32 {
            let address = Axial::new(column, row);
            let distance = (address.q - from.q).abs() + (address.r - from.r).abs();
            if distance > 40
                && world.admits_a_unit(address)
                && world.tile_holder(address) == Some(Holder::NOBODY)
            {
                return address;
            }
        }
    }
    panic!("the world holds no unheld ground far from the holding");
}

#[test]
fn the_relation_is_the_same_at_every_thread_count() {
    let mut answers = Vec::new();
    for threads in [1usize, 2, 12] {
        let mut world = World::new(TWO_SIDES).expect("the configuration describes a world");
        world
            .set_choice_schedule(0)
            .expect("the exponent is inside the range");
        let corner = Axial::new(12, 43);
        for row in 0..8 {
            for column in 0..8 {
                let address = Axial::new(corner.q + column, corner.r + row);
                if world.admits_a_unit(address) {
                    let _ = world.spawn_soldier(address, HOST);
                }
            }
        }
        for _ in 0..6 {
            world.step(threads).expect("the step must run");
        }
        let middle = Axial::new(corner.q + 4, corner.r + 4);
        stand_at(&mut world, middle, HOST, HOST_UNITS_ON_THE_VISITED_TILE);
        stand_at(&mut world, middle, GUEST, 1);
        // The guest must not sit in the last chunk of the arena, or a combine
        // that took the last chunk and dropped the rest would still give the
        // right answer and the comparison below would prove nothing. The
        // padding puts about a third of the population above the guest at
        // every thread count in this test.[^1]
        //
        // [^1]: Testing rules, section 2a. `.claude/rules/testing.md`
        for row in 0..4 {
            for column in 0..8 {
                let address = Axial::new(corner.q + column, corner.r + row);
                if world.admits_a_unit(address) {
                    let _ = world.spawn_soldier(address, HOST);
                }
            }
        }
        world.step(threads).expect("the step must run");

        let rows: Vec<u64> = world
            .presence_rows()
            .expect("the relation is fresh")
            .iter()
            .map(|row| row.to_bits())
            .collect();
        assert!(
            rows[HOST.0 as usize] != 0,
            "the fixture must set a bit at {threads} threads, or the comparison proves nothing"
        );
        answers.push((threads, rows));
    }
    let (_, first) = &answers[0];
    for (threads, rows) in &answers[1..] {
        assert_eq!(
            rows, first,
            "the relation must not depend on the thread count, and it differs at {threads}"
        );
    }
}

#[test]
fn a_read_after_a_change_to_the_population_is_refused() {
    let (mut world, middle) = a_world_the_host_holds();
    assert!(
        world.presence_rows().is_ok(),
        "a world that has stepped answers"
    );
    stand_at(&mut world, middle, GUEST, 1);
    assert!(
        world.presence_rows().is_err(),
        "a spawn without a step must be refused rather than answered"
    );
    assert!(
        world.stands_in_territory(GUEST, HOST).is_err(),
        "the one-pair read must refuse the same way"
    );
    world.step(1).expect("the step must run");
    assert!(
        world.presence_rows().is_ok(),
        "a step makes the relation fresh again"
    );
}

#[test]
fn a_world_that_never_stepped_answers() {
    let world = World::new(TWO_SIDES).expect("the configuration describes a world");
    let rows = world
        .presence_rows()
        .expect("a new world answers rather than refusing");
    assert!(
        rows.iter().all(|row| row.is_empty()),
        "a world with no unit holds an empty relation"
    );
}
