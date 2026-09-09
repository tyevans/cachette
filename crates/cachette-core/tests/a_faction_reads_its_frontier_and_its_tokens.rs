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

use cachette_core::faction_observation::observation_schema;
use cachette_core::obs_frontier::FRONTIER_SLOTS;
use cachette_core::obs_token::{
    RivalPowers, RIVAL_CHANNELS, RIVAL_CHANNEL_NAMES, RIVAL_TOKENS, SETTLEMENT_CHANNELS,
    SETTLEMENT_TOKENS, SITE_CHANNELS, SITE_TOKENS, THREAT_CHANNELS, THREAT_CHANNEL_NAMES,
    THREAT_TOKENS, TOKEN_SLOTS,
};
use cachette_core::site::CommodityId;
use cachette_core::unit_type::{UnitTypeId, UnitTypeRow, WORKER_ROW};
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

/// The power vectors a test passes when it reads no relative ratio.
///
/// **The gather pass of the observation owns those vectors, and it states
/// the fog rule of each.** A vector built in this file from an unfogged
/// reader would state one of those rules a second time, and the copies could
/// then disagree. The tests here read the geometry, the order and the cost
/// of the sets, and an empty vector reaches every one of them. The tests
/// that read a ratio read the published array, so the gather pass supplies
/// the value.
const NO_POWER: [i64; 0] = [];

/// Returns the power vectors for a test that reads no relative ratio.
fn no_powers() -> RivalPowers<'static> {
    RivalPowers {
        held_tiles: &NO_POWER,
        units: &NO_POWER,
        strength: &NO_POWER,
        upgrades: &NO_POWER,
        renown: &NO_POWER,
        wonder: &NO_POWER,
        leader: 0,
        confidence: Fix32::ZERO,
    }
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
        .faction_entity_tokens(READER, &stack, &frontier, &no_powers())
        .expect("the number names a faction of this world");
    (frontier.slots().to_vec(), tokens.slots().to_vec())
}

/// Returns the positions of one token set of the published array.
///
/// **This reads the array a policy reads**, so the values come from the
/// gather pass and not from a fixture. The schema names the field of each
/// set and says where it starts.
fn published_set(world: &World, set: &str) -> Vec<i64> {
    let field = observation_schema()
        .row(set)
        .expect("the layout holds a field of that name");
    let array = world
        .faction_observation(READER)
        .expect("the number names a faction of this world");
    let start = field.start as usize;
    array[start..start + field.positions as usize].to_vec()
}

/// Returns one channel of one token of a published set.
fn channel_of(tokens: &[i64], names: &[&str], token: usize, channel: &str) -> i64 {
    let width = names.len();
    let at = names
        .iter()
        .position(|name| *name == channel)
        .expect("the set holds a channel of that name");
    tokens[token * width + at]
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
        .faction_entity_tokens(READER, &stack, &frontier, &no_powers())
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

/// The rival that every fixture below reads about.
const RIVAL: FactionId = FactionId(1);

/// The type number of the unit that fights weakly.
const WEAK: u8 = 0;

/// The type number of the unit that fights strongly.
const STRONG: u8 = 1;

/// Returns a worker row that fights with the given attack.
const fn fighter(attack: Fix32) -> UnitTypeRow {
    UnitTypeRow {
        attack,
        armour: Fix32::ZERO,
        ..WORKER_ROW
    }
}

/// Founds a settlement of one faction near one address, and returns the
/// place.
fn a_city_of_near(world: &mut World, faction: FactionId, wanted: Axial) -> Axial {
    let place = ground_near(world, wanted, 14);
    world
        .spawn_soldier(place, faction)
        .expect("the fixture places a unit on ground that admits one");
    world
        .found_settlement(place, faction)
        .expect("the fixture founds a settlement on ground that admits one");
    place
}

/// Spawns units of one faction and one type on one address.
fn spawn(world: &mut World, address: Axial, faction: FactionId, unit_type: u8, count: u32) {
    let kind = UnitTypeId::from_u8(unit_type).expect("the number names a row of the table");
    for _ in 0..count {
        let unit = world
            .spawn_soldier(address, faction)
            .expect("the ground admits a unit and the faction is inside the world");
        assert!(world.set_unit_type(unit, kind), "the unit is alive");
    }
}

/// Builds a world in which the reader sees one city of one rival.
///
/// **The fixture must reach the case, and it asserts that it did.** A rival
/// whose city the reader cannot see this frame gets no token at all, so the
/// ratio channels would then read zero for a reason that is not the writer.
fn a_world_with_a_rival_in_view(seed: u64) -> (World, Axial) {
    let mut world = a_still_world(96, 96, seed);
    world
        .define_unit_type(WEAK, fighter(Fix32::from_int(1)))
        .expect("the weak row is inside the table");
    let mine = a_city_of_near(&mut world, READER, Axial::new(48, 48));
    let theirs = a_city_of_near(&mut world, RIVAL, Axial::new(mine.q + 3, mine.r));
    world.step(1).expect("the step runs");
    let rivals = published_set(&world, "token_rivals");
    assert_eq!(
        channel_of(&rivals, RIVAL_CHANNEL_NAMES, 0, "validity"),
        65536,
        "the fixture must put a city of the rival in view of the reader"
    );
    (world, theirs)
}

/// A rival token carries the power quantities of its subject.
///
/// Eleven of the twelve relative ratios read zero, and seven of them had a
/// value in the gather pass that nothing wrote into the token.[^1] The test
/// asserts the direction of each ratio, so a writer that read the own value
/// and the rival value the wrong way round fails it.
///
/// # References
///
/// [^1]: The audit of the observation, section 2.4. `docs/research/what-a-policy-cannot-see.md`
#[test]
fn a_rival_token_carries_the_power_ratios_of_its_subject() {
    let (mut world, theirs) = a_world_with_a_rival_in_view(0x00d0_0d1e_2222_3333);
    spawn(&mut world, theirs, RIVAL, WEAK, 6);
    world.step(1).expect("the step runs");
    let rivals = published_set(&world, "token_rivals");

    assert!(
        world.population_of(RIVAL) > world.population_of(READER),
        "the fixture must give the rival the larger army, and it gave {} against {}",
        world.population_of(RIVAL),
        world.population_of(READER)
    );
    assert!(
        channel_of(&rivals, RIVAL_CHANNEL_NAMES, 0, "unit_ratio") > 0,
        "the rival holds more units, so the unit ratio leans to the rival"
    );
    assert!(
        channel_of(&rivals, RIVAL_CHANNEL_NAMES, 0, "strength_ratio") > 0,
        "the rival holds more of one type, so the strength ratio leans the \
         way the unit ratio leans"
    );
    assert!(
        channel_of(&rivals, RIVAL_CHANNEL_NAMES, 0, "held_tile_ratio") != 0,
        "the reader sees ground of its own and ground of the rival, so the \
         held tile ratio is not zero"
    );
    assert!(
        channel_of(&rivals, RIVAL_CHANNEL_NAMES, 0, "observation_confidence") > 0,
        "the reader sees ground this frame, so the confidence is above zero"
    );
}

/// The war flag of a rival token follows the relation.
#[test]
fn the_war_flag_of_a_rival_token_follows_the_relation() {
    let (mut world, _) = a_world_with_a_rival_in_view(0x00d0_0d1e_4444_5555);
    let peace = published_set(&world, "token_rivals");
    assert!(
        !world.at_war(READER, RIVAL),
        "the fixture must start the pair out of the war band"
    );
    assert_eq!(
        channel_of(&peace, RIVAL_CHANNEL_NAMES, 0, "war"),
        0,
        "a pair at peace reads no war"
    );

    let edge = world.relation_rules().war_edge;
    assert!(
        world.set_relation(READER, RIVAL, edge - 1),
        "the two numbers name two factions of the world"
    );
    world.step(1).expect("the step runs");
    assert!(
        world.at_war(READER, RIVAL),
        "the fixture must put the pair in the war band"
    );
    let war = published_set(&world, "token_rivals");
    assert_eq!(
        channel_of(&war, RIVAL_CHANNEL_NAMES, 0, "war"),
        65536,
        "a pair in the war band reads war"
    );
}

/// The balance of a threat token compares strength, and not the headcount.
///
/// **The channel published a unit count under a strength name.**[^1] The
/// fixture gives the reader the larger crowd and the rival the stronger
/// army, so a balance that counted units leans to the reader and a balance
/// that reads strength leans to the rival. A count and a strength disagree
/// in direction here, which is the case the assertion needs.
///
/// # References
///
/// [^1]: The audit of the observation, section 2.5. `docs/research/what-a-policy-cannot-see.md`
#[test]
fn the_balance_of_a_threat_token_reads_strength_and_not_the_headcount() {
    let mut world = a_still_world(96, 96, 0x00d0_0d1e_6666_7777);
    world
        .define_unit_type(WEAK, fighter(Fix32::from_int(1)))
        .expect("the weak row is inside the table");
    world
        .define_unit_type(STRONG, fighter(Fix32::from_int(9)))
        .expect("the strong row is inside the table");
    let mine = a_city_of_near(&mut world, READER, Axial::new(48, 48));
    let theirs = ground_near(&world, Axial::new(mine.q + 2, mine.r), 6);
    spawn(&mut world, mine, READER, WEAK, 4);
    spawn(&mut world, theirs, RIVAL, STRONG, 1);
    world.step(1).expect("the step runs");

    let threats = published_set(&world, "token_threat_clusters");
    assert_eq!(
        channel_of(&threats, THREAT_CHANNEL_NAMES, 0, "validity"),
        65536,
        "the fixture must put a cluster of rival units in view of the reader"
    );
    assert!(
        world.population_of(READER) > world.population_of(RIVAL),
        "the fixture must give the reader the larger crowd"
    );
    assert!(
        channel_of(&threats, THREAT_CHANNEL_NAMES, 0, "strength") > 0,
        "the cluster holds a rival unit, so its strength is above zero"
    );
    assert!(
        channel_of(&threats, THREAT_CHANNEL_NAMES, 0, "own_strength") > 0,
        "the reader holds units in the disc, so its own strength is above zero"
    );
    assert!(
        channel_of(&threats, THREAT_CHANNEL_NAMES, 0, "strength_balance") < 0,
        "one unit of nine outweighs four units of one, so the balance leans \
         to the rival"
    );
}
