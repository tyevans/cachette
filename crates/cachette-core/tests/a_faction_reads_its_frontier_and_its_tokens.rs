//! A faction reads its frontier by sector, and it reads its subjects as
//! tokens.
//!
//! The frontier block says where a faction can grow and where it is under
//! threat, on the twelve-sector axis of the egocentric ring frame. The token
//! block publishes four fixed-size sets, and each token holds the channels of
//! one subject.
//!
//! **A token position carries no identity.** Each set is ordered by a game
//! quantity, so position `k` means the `k`th subject on that quantity and
//! never one particular subject. The tests below assert that: they change the
//! quantity and watch the positions swap.[^1]
//!
//! Every test drives the world step, so the fog layers come from the
//! observation pass and not from the test.[^2]
//!
//! # References
//!
//! [^1]: Findings register, FND-647. `docs/FINDINGS.md`
//! [^2]: Testing Rules, section 5. `.agents/rules/testing.md`

use cachette_core::obs_frontier::FRONTIER_SLOTS;
use cachette_core::obs_token::{
    RIVAL_CHANNELS, RIVAL_TOKENS, SETTLEMENT_CHANNELS, SETTLEMENT_TOKENS, SITE_CHANNELS,
    SITE_TOKENS, THREAT_CHANNELS, THREAT_TOKENS, TOKEN_SLOTS,
};
use cachette_core::site::CommodityId;
use cachette_core::{Axial, Entity, FactionId, Fix32, SightRules, World, WorldConfig};

/// The faction that reads in every fixture below.
const READER: FactionId = FactionId(0);

/// The exponent that keeps a unit still.
const KEEP_STILL: u32 = 12;

/// The commodity that the store test writes.
const FIRST_COMMODITY: CommodityId = CommodityId(0);

/// Builds a world in which no unit takes a movement intent.
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
    world.set_sight_rules(SightRules::new(4, 1, 16, 0));
    world
}

/// Returns an address that admits a unit, near the one asked for.
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

/// Founds a settlement of the reader near one address and returns it.
fn a_city_near(world: &mut World, wanted: Axial) -> (Entity, Axial) {
    let place = ground_near(world, wanted, 14);
    world
        .spawn_soldier(place, READER)
        .expect("the fixture places a unit on ground that admits one");
    world
        .found_settlement(place, READER)
        .expect("the fixture founds a settlement on ground that admits one");
    let settlement = world
        .settlement_on(place)
        .expect("the fixture just founded a settlement here");
    (settlement, place)
}

/// Reads the three blocks of one faction.
fn blocks_of(world: &World) -> (Vec<i64>, Vec<i64>) {
    let stack = world
        .faction_ring_stack(READER)
        .expect("the number names a faction of this world");
    let frontier = world
        .faction_frontier(READER, &stack)
        .expect("the number names a faction of this world");
    let tokens = world
        .faction_entity_tokens(READER, &stack, &frontier)
        .expect("the number names a faction of this world");
    (frontier.slots().to_vec(), tokens.slots().to_vec())
}

/// Builds a world with two cities of the reader, at different offsets from
/// the centroid of the ground it holds.
///
/// **Two cities at different offsets is the case the token order test
/// needs.** One city gives one token, and one token cannot swap with
/// anything. Two cities at equal offsets give two tokens whose geometry
/// channels agree, and the test could then not see which subject holds which
/// slot. The fixture asserts that the two geometries differ.
fn a_world_with_two_cities(seed: u64) -> (World, Entity, Entity) {
    let mut world = a_still_world(160, 160, seed);
    let (first, _) = a_city_near(&mut world, Axial::new(20, 20));
    let (second, _) = a_city_near(&mut world, Axial::new(120, 70));
    world.step(1).expect("the step runs");
    assert_ne!(first, second, "the fixture must found two settlements");
    (world, first, second)
}

/// Returns the geometry channels of one settlement token slot.
///
/// The channels are the ring index, the sector pair and the hex distance.
/// They name where the subject stands, so they tell the test which subject
/// holds the slot without reading an identity out of the array.
fn geometry_of_slot(tokens: &[i64], slot: usize) -> &[i64] {
    let width = SETTLEMENT_CHANNELS as usize;
    &tokens[slot * width + 1..slot * width + 5]
}

#[test]
fn the_two_blocks_hold_the_positions_the_layout_states() {
    assert_eq!(FRONTIER_SLOTS, 32);
    assert_eq!(
        TOKEN_SLOTS,
        SETTLEMENT_TOKENS * SETTLEMENT_CHANNELS
            + RIVAL_TOKENS * RIVAL_CHANNELS
            + THREAT_TOKENS * THREAT_CHANNELS
            + SITE_TOKENS * SITE_CHANNELS
    );
    assert_eq!(TOKEN_SLOTS, 624);
}

#[test]
fn every_position_of_the_two_blocks_lies_inside_the_declared_bound() {
    let (world, _, _) = a_world_with_two_cities(0x00c0_ffee_0123_4567);
    let (frontier, tokens) = blocks_of(&world);
    assert_eq!(frontier.len(), FRONTIER_SLOTS as usize);
    assert_eq!(tokens.len(), TOKEN_SLOTS as usize);
    for (position, value) in frontier.iter().enumerate() {
        assert!(
            (-65536..=65536).contains(value),
            "frontier position {position} holds {value}"
        );
    }
    for (position, value) in tokens.iter().enumerate() {
        assert!(
            (-65536..=65536).contains(value),
            "token position {position} holds {value}"
        );
    }
}

/// A settlement token slot means the `k`th store, and never one settlement.
///
/// The test gives the first city the larger store, then gives it to the
/// second. The subject that holds the first slot must change. A slot that
/// meant a particular settlement would not move.
#[test]
fn a_settlement_token_slot_follows_the_store_and_not_the_settlement() {
    let (mut world, first, second) = a_world_with_two_cities(0x0bad_c0de_1111_2222);
    world
        .set_settlement_store(first, FIRST_COMMODITY, Fix32::from_int(400))
        .expect("the commodity lies inside the commodity set");
    world
        .set_settlement_store(second, FIRST_COMMODITY, Fix32::from_int(10))
        .expect("the commodity lies inside the commodity set");
    let (_, before) = blocks_of(&world);

    world
        .set_settlement_store(first, FIRST_COMMODITY, Fix32::from_int(10))
        .expect("the commodity lies inside the commodity set");
    world
        .set_settlement_store(second, FIRST_COMMODITY, Fix32::from_int(400))
        .expect("the commodity lies inside the commodity set");
    let (_, after) = blocks_of(&world);

    assert_ne!(
        geometry_of_slot(&before, 0),
        geometry_of_slot(&before, 1),
        "the fixture must place the two cities at different offsets, \
         or the test cannot see which subject holds which slot"
    );
    assert_eq!(
        geometry_of_slot(&before, 0),
        geometry_of_slot(&after, 1),
        "the city that lost the larger store moved to the second slot"
    );
    assert_eq!(
        geometry_of_slot(&before, 1),
        geometry_of_slot(&after, 0),
        "the city that gained the larger store moved to the first slot"
    );
}

/// A missing token reads zero in every channel, and the validity flag says
/// so.
///
/// A faction with one settlement leaves seven settlement slots empty. A
/// reader must tell a missing token from a token whose subject holds nothing.
#[test]
fn a_missing_token_reads_zero_in_every_channel() {
    let mut world = a_still_world(64, 64, 0x00c0_ffee_0123_4567);
    a_city_near(&mut world, Axial::new(32, 32));
    world.step(1).expect("the step runs");
    let (_, tokens) = blocks_of(&world);

    let width = SETTLEMENT_CHANNELS as usize;
    assert_eq!(
        tokens[0], 65536,
        "the first settlement token is valid, or the fixture found no settlement"
    );
    for slot in 1..SETTLEMENT_TOKENS as usize {
        let token = &tokens[slot * width..(slot + 1) * width];
        assert!(
            token.iter().all(|value| *value == 0),
            "settlement slot {slot} is missing and must read zero in every channel"
        );
    }
}

/// The token discs cost the same on a small world and on a large one.
///
/// Each token reads a disc of fixed radius, and the token count is fixed, so
/// the whole of the disc reading is a constant.
#[test]
fn the_token_discs_cost_the_same_on_two_worlds() {
    let mut small = a_still_world(64, 64, 0x1234_5678_9abc_def0);
    a_city_near(&mut small, Axial::new(32, 32));
    small.step(1).expect("the step runs");

    let mut large = a_still_world(1024, 1024, 0x1234_5678_9abc_def0);
    a_city_near(&mut large, Axial::new(512, 512));
    large.step(1).expect("the step runs");

    assert_ne!(
        small.grid().tile_count(),
        large.grid().tile_count(),
        "the fixture must build two worlds of different size"
    );

    let small_cost = disc_cost(&small);
    let large_cost = disc_cost(&large);
    assert!(small_cost > 0, "the disc pass read no tile");
    assert_eq!(
        small_cost, large_cost,
        "the disc cost followed the size of the world"
    );
}

/// Returns the level 0 tiles the token discs of one world read.
fn disc_cost(world: &World) -> i64 {
    let stack = world
        .faction_ring_stack(READER)
        .expect("the number names a faction of this world");
    let frontier = world
        .faction_frontier(READER, &stack)
        .expect("the number names a faction of this world");
    world
        .faction_entity_tokens(READER, &stack, &frontier)
        .expect("the number names a faction of this world")
        .disc_tiles()
}

/// The frontier walk costs the perimeter of the reader and no more.
///
/// The fixture asserts that the reader holds ground and that the world holds
/// far more tiles than the reader holds, so the assertion meets the case it
/// must reject.
#[test]
fn the_frontier_walk_costs_the_perimeter_and_not_the_world() {
    let mut world = a_still_world(256, 256, 0x0f0f_0f0f_0f0f_0f0f);
    a_city_near(&mut world, Axial::new(24, 24));
    world.step(1).expect("the step runs");

    let held = world.holding().tiles_held_by(READER).count() as i64;
    assert!(held > 0, "the fixture must give the reader held ground");
    assert!(
        held * 8 < i64::from(world.grid().tile_count()),
        "the fixture must leave most of the world unheld, or the bound is not tested"
    );

    let stack = world
        .faction_ring_stack(READER)
        .expect("the number names a faction of this world");
    let frontier = world
        .faction_frontier(READER, &stack)
        .expect("the number names a faction of this world");
    assert!(
        frontier.perimeter_tiles() > 0,
        "a faction that holds ground has a perimeter"
    );
    assert!(
        frontier.perimeter_tiles() <= held,
        "the walk kept {} tiles of a holding of {held}",
        frontier.perimeter_tiles()
    );
}

/// A faction that holds nothing still reads both blocks in full.
#[test]
fn a_faction_that_holds_nothing_reads_both_blocks() {
    let mut world = a_still_world(64, 64, 0x00c0_ffee_0123_4567);
    world.step(1).expect("the step runs");
    assert_eq!(
        world.holding().tiles_held_by(READER).count(),
        0,
        "the fixture must give the reader no held ground"
    );
    let (frontier, tokens) = blocks_of(&world);
    assert_eq!(frontier.len(), FRONTIER_SLOTS as usize);
    assert_eq!(tokens.len(), TOKEN_SLOTS as usize);
    assert!(
        tokens.iter().all(|value| *value == 0),
        "a faction with no settlement, no rival in view and no site reads no token"
    );
}
