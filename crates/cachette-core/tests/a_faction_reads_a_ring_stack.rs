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

use cachette_core::holding::ReachRules;
use cachette_core::obs_frontier::FRONTIER_SLOTS;
use cachette_core::obs_ring::{
    cell_of_delta, ring_of_cell, FIRST_FAR_RING, RING_STACK_CELLS, RING_STACK_CHANNELS,
};
use cachette_core::obs_ring_stack::{
    AREA_CHANNEL, OWN_HELD_CHANNEL, OWN_SETTLEMENT_CHANNEL, RING_STACK_CHANNEL_NAMES,
    RING_STACK_SLOTS,
};
use cachette_core::obs_token::TOKEN_SLOTS;
use cachette_core::unit_type::SOLDIER;
use cachette_core::{Axial, Entity, FactionId, SightRules, TileKind, World, WorldConfig};

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

/// The sight radius that lets one unit observe the ground around it.
///
/// The dead-channel fixture below needs a faction that watches ground far
/// from its city, so its units see further than the cost fixture allows.
const WIDE_SIGHT: u32 = 6;

/// The hex distance from the city at which the fixture places a far unit.
///
/// Ring 4 starts at distance 8, so this distance lands in the far band on
/// every world the fixture builds.
const FAR_STEP: i32 = 24;

/// The channels whose source is the ground or a unit.
///
/// **Each one must read a value in the near band and in the far band.** The
/// near pass reads level 0 tiles and covers rings 0 to 3. The far pass reads
/// the summary of a block and covers rings 4 and above. A channel that only
/// one of the two fills reads zero in the other while the gate channel still
/// says the cell lies inside the world, and a reader then takes that zero for
/// a real absence.[^1]
///
/// # References
///
/// [^1]: What a policy cannot see, section 2.3. `docs/research/what-a-policy-cannot-see.md`
const GROUND_CHANNELS: [&str; 17] = [
    "area_inside_world",
    "observed_share",
    "seen_now_share",
    "open_share",
    "water_share",
    "mean_height",
    "height_spread",
    "food_density",
    "ground_water_density",
    "value_density",
    "resource_share",
    "hazard_share",
    "unclaimed_open_share",
    "own_unit_density",
    "rival_unit_density",
    "own_strength_density",
    "rival_strength_density",
];

/// The channels whose source is a structure of a faction.
///
/// A faction holds the ground inside the reach of its cities, and the frame
/// centres on that ground. Its own settlements and its own held ground are
/// therefore near by construction, and a fixture cannot promise one of them
/// in the far band without a second empire. The memory age is a clock for
/// each block, and the near band of this fixture lies inside blocks the
/// reader watches, so its age is zero there.
///
/// Each channel here must read a value somewhere in the stack.
const STRUCTURE_CHANNELS: [&str; 6] = [
    "memory_age",
    "own_held_share",
    "own_reach_share",
    "own_settlements",
    "rival_held_share",
    "rival_settlements",
];

/// The channels this fixture supplies no source for, and why.
///
/// A finished upgrade needs a build order and the ticks that finish it, and
/// this fixture runs too few ticks to finish one. The upgrade pass walks the
/// upgrade sites of the world and maps each one to the cell it stands in, in
/// the same way the settlement pass does, so no band bounds it. The
/// settlement channels beside it are proven below.
const UNSUPPLIED_CHANNELS: [&str; 2] = ["own_upgrades", "rival_upgrades"];

/// Returns a world in which every quantity the ring stack reports exists.
///
/// **A channel can read zero because the world is dull rather than because
/// the writer is missing.** The fixture therefore states each distribution it
/// needs and fails when the world does not give it.[^1]
///
/// The world holds one city of the reader, a unit of the reader far from that
/// city, a rival unit beside each of the two, a fire beside each of the two,
/// and one block that the reader saw once and no longer watches.
///
/// **Every unit of it is a soldier, and the reach of a city is short.** The
/// worker row of the unit type table carries no attack and no armour, so a
/// world of workers holds no strength at all. A city whose reach covers
/// everything its units watch leaves no unclaimed open ground.
///
/// # References
///
/// [^1]: Testing Rules, section 2a. `.agents/rules/testing.md`
fn a_world_that_supplies_every_quantity() -> World {
    let mut world = a_still_world(160, 160, 0x00c0_ffee_0123_4567);
    world.set_sight_rules(SightRules::new(WIDE_SIGHT, 1, 16, 0));
    world.set_reach_rules(ReachRules::new(2, 1, 2));
    let city = shore_near(&world, Axial::new(80, 80), 24);
    a_unit_at(&mut world, city, READER);
    world
        .found_settlement(city, READER)
        .expect("the fixture founds a settlement on ground that admits one");
    let beside_city = ground_near(&world, Axial::new(city.q + 1, city.r), 3);
    let garrison = a_unit_at(&mut world, beside_city, READER);
    world.step(1).expect("the step runs");

    let walked = ground_near(&world, Axial::new(city.q - FAR_STEP, city.r), 6);
    let walker = a_unit_at(&mut world, walked, READER);
    world.step(1).expect("the step runs");
    assert!(
        world.despawn_soldier(walker),
        "the fixture must remove the unit that leaves the memory behind"
    );
    world.step(1).expect("the step runs");
    world.step(1).expect("the step runs");
    assert!(
        world.faction_has_seen(READER, walked) && !world.faction_sees_now(READER, walked),
        "the fixture must leave one far place remembered and unwatched"
    );

    let scouted = shore_near(&world, Axial::new(city.q + FAR_STEP, city.r), 8);
    let scout = a_unit_at(&mut world, scouted, READER);
    let beside_scout = ground_near(&world, Axial::new(scouted.q + 2, scouted.r), 3);
    let far_rival = a_unit_at(&mut world, beside_scout, STRANGER);
    let facing_city = ground_near(&world, Axial::new(city.q + 4, city.r), 3);
    let near_rival = a_unit_at(&mut world, facing_city, STRANGER);
    world
        .found_settlement(beside_scout, STRANGER)
        .expect("the fixture founds a rival settlement on ground that admits one");
    for unit in [garrison, scout, far_rival, near_rival] {
        assert!(
            world.set_unit_type(unit, SOLDIER),
            "the fixture must give each unit a type that carries a strength"
        );
    }
    world.step(1).expect("the step runs");

    burn_ground_near(&mut world, city);
    burn_ground_near(&mut world, scouted);
    world
}

/// Returns a place near an address that admits a unit and that water
/// adjoins.
///
/// **The water share of a cell is zero on a world with no water in it**, and
/// a fixture that reads zero there measures its own terrain rather than the
/// writer.[^1] The search is a spiral over the rings around the address, and
/// it takes the first place that admits a unit and holds water within three
/// steps.
///
/// # References
///
/// [^1]: Testing Rules, section 2a. `.agents/rules/testing.md`
fn shore_near(world: &World, wanted: Axial, bound: i32) -> Axial {
    for ring in 0..=bound {
        for column in -ring..=ring {
            for row in -ring..=ring {
                if column.abs().max(row.abs()) != ring {
                    continue;
                }
                let candidate = Axial::new(wanted.q + column, wanted.r + row);
                if world.admits_a_unit(candidate) && water_within(world, candidate, 3) {
                    return candidate;
                }
            }
        }
    }
    panic!("the fixture found no shore near {wanted:?}");
}

/// Reports whether water stands within a number of steps of one place.
fn water_within(world: &World, place: Axial, reach: i32) -> bool {
    for column in -reach..=reach {
        for row in -reach..=reach {
            let here = Axial::new(place.q + column, place.r + row);
            if world.tile_kind(here) == Some(TileKind::Water) {
                return true;
            }
        }
    }
    false
}

/// Sets one tile alight, on ground the reader watches near an address.
///
/// The terrain comes from the seed, so the ground beside the address may
/// carry no fuel. The search is a spiral over the rings around the address,
/// and it takes the first tile that catches and that the reader watches.
fn burn_ground_near(world: &mut World, wanted: Axial) {
    for ring in 0..=4i32 {
        for column in -ring..=ring {
            for row in -ring..=ring {
                if column.abs().max(row.abs()) != ring {
                    continue;
                }
                let candidate = Axial::new(wanted.q + column, wanted.r + row);
                if !world.faction_sees_now(READER, candidate) {
                    continue;
                }
                let Some(tile) = world.grid().index_of(candidate) else {
                    continue;
                };
                if world.ignite(tile) {
                    return;
                }
            }
        }
    }
    panic!("the fixture found no ground that catches near {wanted:?}");
}

/// Returns the number of the channel of one name.
fn channel_of(name: &str) -> u32 {
    let position = RING_STACK_CHANNEL_NAMES
        .iter()
        .position(|held| *held == name)
        .unwrap_or_else(|| panic!("the channel table holds no channel called {name:?}"));
    position as u32 + 1
}

/// Returns the largest value one channel reads over a band of the frame.
fn largest_in_band(
    stack: &cachette_core::obs_ring_stack::RingStack,
    channel: u32,
    far: bool,
) -> i64 {
    (0..RING_STACK_CELLS)
        .filter(|cell| (ring_of_cell(*cell) >= FIRST_FAR_RING) == far)
        .map(|cell| stack.channel(cell, channel).abs())
        .max()
        .unwrap_or(0)
}

/// Every channel of the table belongs to one of the three lists above.
///
/// This is what keeps the two tests below from going stale. A channel added
/// to the table and to no list fails here, so nobody can add a channel and
/// leave it unwatched.
#[test]
fn every_channel_of_the_table_states_where_it_must_read_a_value() {
    let mut listed: Vec<&str> = GROUND_CHANNELS
        .iter()
        .chain(STRUCTURE_CHANNELS.iter())
        .chain(UNSUPPLIED_CHANNELS.iter())
        .copied()
        .collect();
    let mut declared: Vec<&str> = RING_STACK_CHANNEL_NAMES.to_vec();
    listed.sort_unstable();
    declared.sort_unstable();
    assert_eq!(
        listed, declared,
        "each channel of the table belongs to exactly one of the three lists"
    );
    assert_eq!(
        listed.len(),
        RING_STACK_CHANNELS as usize,
        "the lists hold one entry for each channel"
    );
}

/// No channel of the stack reads zero in every cell of a live world.
///
/// A channel of the literal zero declares a real bound and publishes a
/// constant, and the check that finds a reserved field cannot see it. A
/// reward term reads such a position as truth.[^1]
///
/// # References
///
/// [^1]: What a policy cannot see, section 2.2. `docs/research/what-a-policy-cannot-see.md`
#[test]
fn no_channel_of_the_stack_reads_zero_in_every_cell() {
    let world = a_world_that_supplies_every_quantity();
    let stack = stack_of(&world, READER);
    for name in GROUND_CHANNELS.iter().chain(STRUCTURE_CHANNELS.iter()) {
        let channel = channel_of(name);
        let largest = (0..RING_STACK_CELLS)
            .map(|cell| stack.channel(cell, channel).abs())
            .max()
            .unwrap_or(0);
        assert!(
            largest > 0,
            "the channel {name:?} reads zero in all {RING_STACK_CELLS} cells, \
             and it declares a real bound"
        );
    }
}

/// Every channel that follows the ground reads a value beyond ring 3.
///
/// The near pass reads tiles and the far pass reads block summaries. A
/// channel that only the near pass fills is dark in 120 of the 151 cells,
/// and the gate channel cannot say so. The gate answers for a whole cell,
/// and the failure sits one level below it, at one channel of that cell.[^1]
///
/// # References
///
/// [^1]: What a policy cannot see, section 2.3. `docs/research/what-a-policy-cannot-see.md`
#[test]
fn every_ground_channel_reads_a_value_in_the_near_band_and_the_far_band() {
    let world = a_world_that_supplies_every_quantity();
    let stack = stack_of(&world, READER);
    let far_inside = (0..RING_STACK_CELLS)
        .filter(|cell| ring_of_cell(*cell) >= FIRST_FAR_RING)
        .filter(|cell| stack.channel(*cell, AREA_CHANNEL) > 0)
        .count();
    assert!(
        far_inside > 0,
        "the fixture must build a world whose frame reaches ring 4, \
         or the far band assertion never meets the case"
    );

    for name in &GROUND_CHANNELS {
        let channel = channel_of(name);
        assert!(
            largest_in_band(&stack, channel, false) > 0,
            "the channel {name:?} reads zero in every cell of rings 0 to 3"
        );
        assert!(
            largest_in_band(&stack, channel, true) > 0,
            "the channel {name:?} reads zero in every cell of ring 4 and above, \
             and {far_inside} of those cells lie inside the world"
        );
    }
}

/// The frame of the training world reaches the far band.
///
/// The five channels that the far pass once left dark cost nothing on a world
/// too small to reach ring 4. A 48 by 48 world is the one the training runs
/// use, so this states what it measures rather than assuming it.
#[test]
fn the_frame_of_a_small_world_reaches_the_far_band() {
    let (world, _) = a_world_with_a_city(48, 48, 0x0bad_c0de_1111_2222, Axial::new(24, 24));
    let stack = stack_of(&world, READER);
    let far_inside = (0..RING_STACK_CELLS)
        .filter(|cell| ring_of_cell(*cell) >= FIRST_FAR_RING)
        .filter(|cell| stack.channel(*cell, AREA_CHANNEL) > 0)
        .count();
    assert!(
        far_inside > 0,
        "a 48 by 48 world reaches no cell of ring 4, so the far band costs nothing"
    );
}
