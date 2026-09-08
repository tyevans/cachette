//! A faction reads its surroundings as an egocentric ring stack.
//!
//! The stack has one width on every world, so a policy trained on one world
//! fits another. A field over the world lattice cannot do that, because the
//! lattice grows with the world.[^1]
//!
//! Every test here drives the world step. The step runs the observation pass
//! that fills the fog layers, so a test that filled a layer itself would
//! prove that the reader works and not that anything reaches it.[^2]
//!
//! **Every fixture states the distribution it needs and fails when the world
//! does not give it.** A world chosen to look right supplies no extreme, so
//! the assertion would never receive the input that fails it.[^3]
//!
//! # References
//!
//! [^1]: Findings register, FND-670. `docs/FINDINGS.md`
//! [^2]: Testing Rules, section 5. `.agents/rules/testing.md`
//! [^3]: Testing Rules, section 2a. `.agents/rules/testing.md`

use cachette_core::obs_frontier::FRONTIER_SLOTS;
use cachette_core::obs_ring::{cell_of_delta, ring_of_cell, RING_STACK_CELLS, RING_STACK_CHANNELS};
use cachette_core::obs_ring_stack::{
    AREA_CHANNEL, OWN_HELD_CHANNEL, OWN_REACH_CHANNEL, OWN_SETTLEMENT_CHANNEL, RING_STACK_SLOTS,
};
use cachette_core::obs_token::TOKEN_SLOTS;
use cachette_core::{Axial, Entity, FactionId, SightRules, World, WorldConfig};

/// The faction that reads in every fixture below.
const READER: FactionId = FactionId(0);

/// The faction that stands where the reader cannot see it.
const STRANGER: FactionId = FactionId(1);

/// The exponent that keeps a unit still.
///
/// A unit takes a movement intent at the interval its cell schedules. A long
/// interval stops a unit taking one inside a short test, so each test below
/// measures the reader and not the movement pass.
const KEEP_STILL: u32 = 12;

/// The sight radius that keeps the fog of the reader small.
///
/// A small radius is what the cost test needs. A faction that sees the whole
/// world gives the pass nothing to skip, so the test would measure the
/// fixture rather than the bound.
const SHORT_SIGHT: u32 = 3;

/// Builds a world of one size in which no unit takes a movement intent.
fn a_still_world(width: u32, height: u32, seed: u64) -> World {
    let mut world = World::new(WorldConfig {
        width,
        height,
        seed,
        faction_count: 3,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    })
    .expect("the configuration describes a world");
    world
        .set_choice_schedule(KEEP_STILL)
        .expect("the exponent is inside the range");
    world.set_sight_rules(SightRules::new(SHORT_SIGHT, 1, 16, 0));
    world
}

/// Returns an address that admits a unit, near the one asked for.
///
/// The terrain comes from the seed, so the tile a test names may hold water.
/// The search is a spiral over the rings around the address, and it takes the
/// first tile of the lowest ring. The order is fixed, so two runs return one
/// answer.
fn ground_near(world: &World, wanted: Axial, bound: i32) -> Axial {
    for ring in 0..=bound {
        for column in -ring..=ring {
            for row in -ring..=ring {
                if column.abs().max(row.abs()) != ring {
                    continue;
                }
                let candidate = Axial::new(wanted.q + column, wanted.r + row);
                if world.admits_a_unit(candidate) {
                    return candidate;
                }
            }
        }
    }
    panic!("the fixture found no ground that admits a unit near {wanted:?}");
}

/// Puts a unit of one faction on a tile and returns its identity.
fn a_unit_at(world: &mut World, address: Axial, faction: FactionId) -> Entity {
    world
        .spawn_soldier(address, faction)
        .expect("the fixture places a unit on ground that admits one")
}

/// Builds a world with one settlement of the reader, and returns its place.
///
/// The settlement is what gives the reader held ground, and held ground is
/// what gives the frame a centre. A fixture without one would read the world
/// centre instead, and every offset in the test would mean something else.
fn a_world_with_a_city(width: u32, height: u32, seed: u64, at: Axial) -> (World, Axial) {
    let mut world = a_still_world(width, height, seed);
    let place = ground_near(&world, at, 12);
    a_unit_at(&mut world, place, READER);
    world
        .found_settlement(place, READER)
        .expect("the fixture founds a settlement on ground that admits one");
    world.step(1).expect("the step runs");
    assert!(
        world.holding().tiles_held_by(READER).count() > 0,
        "the fixture must give the reader held ground, or the frame has no centre"
    );
    (world, place)
}

/// Reads the ring stack of the reader.
fn stack_of(world: &World, faction: FactionId) -> cachette_core::obs_ring_stack::RingStack {
    world
        .faction_ring_stack(faction)
        .expect("the number names a faction of this world")
}

#[test]
fn the_three_blocks_hold_the_positions_the_layout_states() {
    assert_eq!(RING_STACK_SLOTS, 3775);
    assert_eq!(RING_STACK_SLOTS, RING_STACK_CELLS * RING_STACK_CHANNELS);
    assert_eq!(FRONTIER_SLOTS, 32);
    assert_eq!(TOKEN_SLOTS, 624);
}

#[test]
fn every_position_of_the_stack_belongs_to_one_cell_and_one_channel() {
    let (world, _) = a_world_with_a_city(64, 64, 0x00c0_ffee_0123_4567, Axial::new(32, 32));
    let stack = stack_of(&world, READER);
    assert_eq!(stack.slots().len(), RING_STACK_SLOTS as usize);

    let mut covered = vec![0u32; RING_STACK_SLOTS as usize];
    for cell in 0..RING_STACK_CELLS {
        for channel in 1..=RING_STACK_CHANNELS {
            let position = (cell * RING_STACK_CHANNELS + channel - 1) as usize;
            covered[position] += 1;
            assert_eq!(
                stack.channel(cell, channel),
                stack.slots()[position],
                "cell {cell} channel {channel}"
            );
        }
    }
    for (position, count) in covered.iter().enumerate() {
        assert_eq!(*count, 1, "position {position} belongs to one channel only");
    }
}

#[test]
fn every_position_of_the_stack_lies_inside_the_declared_bound() {
    let (world, _) = a_world_with_a_city(64, 64, 0x00c0_ffee_0123_4567, Axial::new(32, 32));
    let stack = stack_of(&world, READER);
    for (position, value) in stack.slots().iter().enumerate() {
        assert!(
            (-65536..=65536).contains(value),
            "position {position} holds {value}"
        );
    }
}

/// The city of the reader reads the same channel on two worlds of very
/// different size.
///
/// This is the whole point of the block. The centre of the frame is the
/// centroid of the ground the faction holds, and the terrain of each world
/// puts that centroid in a different place. The cell that carries the city
/// therefore differs between the two worlds. What must not differ is the
/// value the channel reads, because one settlement is one settlement on
/// every world.
#[test]
fn one_city_reads_one_settlement_channel_on_two_worlds() {
    let small = a_world_with_a_city(48, 48, 0x0bad_c0de_1111_2222, Axial::new(24, 24));
    let large = a_world_with_a_city(512, 512, 0x0bad_c0de_1111_2222, Axial::new(256, 256));

    let small_stack = stack_of(&small.0, READER);
    let large_stack = stack_of(&large.0, READER);
    assert_eq!(
        small_stack.slots().len(),
        large_stack.slots().len(),
        "the stack has one width on every world"
    );
    assert_ne!(
        small.0.grid().tile_count(),
        large.0.grid().tile_count(),
        "the fixture must build two worlds of different size"
    );

    let small_cell = cell_of_place(&small_stack, small.1);
    let large_cell = cell_of_place(&large_stack, large.1);
    assert!(
        small_stack.channel(small_cell, OWN_SETTLEMENT_CHANNEL) > 0,
        "the cell that carries the city of the small world reads no settlement"
    );
    assert_eq!(
        small_stack.channel(small_cell, OWN_SETTLEMENT_CHANNEL),
        large_stack.channel(large_cell, OWN_SETTLEMENT_CHANNEL),
        "one settlement reads one value, whatever the size of the world"
    );
}

/// Returns the cell of a stack that one world address falls in.
fn cell_of_place(stack: &cachette_core::obs_ring_stack::RingStack, place: Axial) -> u32 {
    let centre = stack.centre();
    cell_of_delta(Axial::new(place.q - centre.q, place.r - centre.r))
}

/// The same arrangement in two parts of one world reads the same channel.
///
/// The frame is translation invariant. A faction that stands in the north of
/// a world reads what the same faction reads in the south, for every channel
/// that follows the arrangement rather than the terrain.
#[test]
fn the_settlement_channel_agrees_when_the_arrangement_moves() {
    let north = a_world_with_a_city(256, 256, 0x1234_5678_9abc_def0, Axial::new(60, 60));
    let south = a_world_with_a_city(256, 256, 0x1234_5678_9abc_def0, Axial::new(190, 190));

    let first = stack_of(&north.0, READER);
    let second = stack_of(&south.0, READER);

    assert_ne!(
        first.centre(),
        second.centre(),
        "the fixture must place the two cities apart, or the test compares one place with itself"
    );

    let first_cell = cell_of_place(&first, north.1);
    let second_cell = cell_of_place(&second, south.1);
    assert!(
        first.channel(first_cell, OWN_SETTLEMENT_CHANNEL) > 0,
        "the cell that carries the northern city reads no settlement"
    );
    assert_eq!(
        first.channel(first_cell, OWN_SETTLEMENT_CHANNEL),
        second.channel(second_cell, OWN_SETTLEMENT_CHANNEL),
        "the settlement channel does not follow the place"
    );

    let first_held: i64 = (0..RING_STACK_CELLS)
        .map(|cell| first.channel(cell, OWN_HELD_CHANNEL))
        .filter(|value| *value > 0)
        .count() as i64;
    let second_held: i64 = (0..RING_STACK_CELLS)
        .map(|cell| second.channel(cell, OWN_HELD_CHANNEL))
        .filter(|value| *value > 0)
        .count() as i64;
    assert!(
        first_held > 0 && second_held > 0,
        "both readers hold ground"
    );
}

/// The pass reads no block that the faction has never observed.
///
/// The faction holds one city in one corner of a wide world and sees three
/// tiles around each of its units. The blocks it has observed are therefore a
/// small part of the lattice, and the pass must read only those.
#[test]
fn the_pass_reads_only_the_blocks_the_faction_observed() {
    let (world, _) = a_world_with_a_city(256, 256, 0x00c0_ffee_0123_4567, Axial::new(12, 12));
    let layout = world.observation().layout();
    let observed = world
        .observation()
        .remembered_layer(READER)
        .map_or(0, |layer| layer.populated_blocks().len() as i64);

    assert!(
        observed > 0,
        "the fixture must give the reader some observed ground"
    );
    assert!(
        observed * 8 < i64::from(layout.block_count()),
        "the fixture must leave most of the lattice unobserved, or the bound is not tested: \
         {observed} observed of {} blocks",
        layout.block_count()
    );

    let stack = stack_of(&world, READER);
    let cost = stack.cost();
    assert!(
        cost.summary_blocks <= observed,
        "the pass read {} blocks and the faction observed {observed}",
        cost.summary_blocks
    );
    assert!(
        cost.summary_blocks < i64::from(layout.block_count()),
        "the pass read every block of the lattice"
    );
    assert_eq!(
        cost.near_tiles, 169,
        "the near pass reads the whole near band, which is a constant"
    );
}

/// A change on ground the faction has never seen does not reach its stack.
///
/// This is the containment test. The paired assertion below proves that the
/// comparison is live: a change on ground the faction does see must reach the
/// stack.
#[test]
fn a_change_the_faction_cannot_see_does_not_reach_the_stack() {
    let (mut world, city) =
        a_world_with_a_city(256, 256, 0x00c0_ffee_0123_4567, Axial::new(12, 12));
    let before = stack_of(&world, READER).slots().to_vec();

    let far = ground_near(&world, Axial::new(240, 240), 12);
    assert!(
        !world.faction_has_seen(READER, far),
        "the fixture must choose ground the reader has never seen"
    );
    a_unit_at(&mut world, far, STRANGER);
    world.step(1).expect("the step runs");
    let after = stack_of(&world, READER).slots().to_vec();
    assert_eq!(
        before, after,
        "a stranger on ground the reader has never seen changed its stack"
    );

    let near = ground_near(&world, Axial::new(city.q + 1, city.r), 4);
    assert!(
        world.faction_sees_now(READER, near),
        "the fixture must choose ground the reader sees now, or the paired assertion proves nothing"
    );
    a_unit_at(&mut world, near, STRANGER);
    world.step(1).expect("the step runs");
    let visible = stack_of(&world, READER).slots().to_vec();
    assert_ne!(
        after, visible,
        "a stranger on ground the reader watches did not reach its stack"
    );
}

/// The reader reads its own border even where it is not looking.
///
/// A faction that read its own territory through the fog of the present frame
/// would watch that border move as its units move.[^1]
///
/// The fixture asserts that the reader holds ground it does not see this
/// frame. Without that case the assertion would never receive the input that
/// fails it.
///
/// # References
///
/// [^1]: Findings register, FND-671. `docs/FINDINGS.md`
#[test]
fn the_reader_reads_the_ground_it_holds_and_does_not_watch() {
    let (world, _) = a_world_with_a_city(96, 96, 0x0f0f_0f0f_0f0f_0f0f, Axial::new(48, 48));
    let unwatched = world
        .holding()
        .tiles_held_by(READER)
        .filter_map(|tile| world.grid().address_of(tile))
        .filter(|address| !world.faction_sees_now(READER, *address))
        .count();
    assert!(
        unwatched > 0,
        "the fixture must give the reader held ground it does not watch, \
         or the assertion never meets the case"
    );

    let stack = stack_of(&world, READER);
    let held: i64 = (0..RING_STACK_CELLS)
        .map(|cell| stack.channel(cell, OWN_HELD_CHANNEL))
        .sum();
    assert!(
        held > 0,
        "the reader reads none of the ground it holds, which is the flicker defect"
    );
}

/// A faction that holds nothing still reads a whole stack.
///
/// The extreme matters. A faction with no city takes the centroid of its
/// units, and a faction with neither takes the centre of the world. Both must
/// give an array of the declared width.
#[test]
fn a_faction_that_holds_nothing_reads_a_whole_stack() {
    let mut world = a_still_world(64, 64, 0x00c0_ffee_0123_4567);
    world.step(1).expect("the step runs");
    assert_eq!(
        world.holding().tiles_held_by(READER).count(),
        0,
        "the fixture must give the reader no held ground"
    );
    let empty = stack_of(&world, READER);
    assert_eq!(empty.slots().len(), RING_STACK_SLOTS as usize);
    assert_eq!(
        empty.centre(),
        Axial::new(32, 32),
        "a faction with no ground and no unit takes the centre of the world"
    );

    let camp = ground_near(&world, Axial::new(10, 50), 8);
    a_unit_at(&mut world, camp, READER);
    world.step(1).expect("the step runs");
    let camped = stack_of(&world, READER);
    assert_eq!(camped.slots().len(), RING_STACK_SLOTS as usize);
    assert_eq!(
        camped.centre(),
        camp,
        "a faction with one unit and no ground takes the place of that unit"
    );
}

/// A far ring of a small world reads no area, and the same ring of a large
/// world reads some.
///
/// The area channel is the channel that lets one layout serve every world
/// size. A policy reads it and learns that it gates the rest.
#[test]
fn the_area_channel_separates_a_small_world_from_a_large_one() {
    let small = a_world_with_a_city(48, 48, 0x0bad_c0de_1111_2222, Axial::new(24, 24));
    let large = a_world_with_a_city(2048, 2048, 0x0bad_c0de_1111_2222, Axial::new(1024, 1024));
    let small_stack = stack_of(&small.0, READER);
    let large_stack = stack_of(&large.0, READER);

    let far = (0..RING_STACK_CELLS)
        .find(|cell| ring_of_cell(*cell) == 9)
        .expect("ring 9 has a cell");
    assert_eq!(
        small_stack.channel(far, AREA_CHANNEL),
        0,
        "ring 9 lies outside a 48 by 48 world"
    );
    assert!(
        large_stack.channel(far, AREA_CHANNEL) > 0,
        "ring 9 lies inside a 2048 by 2048 world"
    );
}
