//! The controller advertises, prices a contract and opens a trade route.
//!
//! Every test here drives the engine through `step` and reads a board, the
//! negotiation plane, the census or the carrier list afterwards.[^1] None of
//! them calls the trade verbs for the controller.
//!
//! **Two keyed draws are new**, and each gets one test for each field of its
//! key, because the determinism tests cannot tell a draw keyed on the wrong
//! field from a correct one.[^2]
//!
//! **The fixture reaches the extreme.** One faction holds a site whose store
//! is at the ceiling of the scale, and the other holds a site whose store is
//! empty. A fixture that gave both sites a middling store would post no row
//! at all, and every assertion below would pass on an empty board.[^3]
//!
//! # References
//!
//! [^1]: Testing Rules, sections 5 and 6. `.agents/rules/testing.md`
//! [^2]: Testing Rules, section 2. `.agents/rules/testing.md`
//! [^3]: Testing Rules, section 2a. `.agents/rules/testing.md`

use std::collections::BTreeSet;

use cachette_core::controller::{asking_good_of, wants_trade_step};
use cachette_core::resource::{Amount, ResourceKind, RESOURCE_KIND_COUNT};
use cachette_core::unit_type::SOLDIER;
use cachette_core::Verb;
use cachette_core::{
    Advert, Axial, CommodityId, Entity, FactionId, FactionWeights, Fix32, Tick, World, WorldConfig,
    ADVERT_OFFERS, ADVERT_WANTS, TRADE_BOUND, WORK_COMMODITY,
};

/// The people each founding settles.
const GROUP: u32 = 8;

/// The commodity that every kind of work fills. It is read from the declared
/// map, so this file holds no second copy of the number.[^1]
///
/// # References
///
/// [^1]: Findings register, FND-191. `docs/FINDINGS.md`
const GOODS: CommodityId = WORK_COMMODITY[0];

/// The store the seller holds.
///
/// **It is far above the mark and below the stock target.** A store at the
/// ceiling of the scale would pass the target that the wealth reader fires
/// on, the game would end on the first tick, and the controller would then
/// emit nothing at all.[^1]
///
/// # References
///
/// [^1]: ADR-0148, a game end is recorded once and stops the controllers, decision D4. `docs/adrs/accepted/adr-0148-a-game-end-is-recorded-once-and-stops-the-controllers.md`
const FULL: Fix32 = Fix32::from_int(256);

/// The mark that separates a surplus from a shortfall in these tests.
const MARK: u32 = 8;

const ZERO: FactionId = FactionId(0);
const ONE: FactionId = FactionId(1);

fn config(factions: u16, seed: u64) -> WorldConfig {
    WorldConfig {
        width: 48,
        height: 48,
        seed,
        faction_count: factions,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    }
}

/// Founds one group for each faction, far enough apart that the survey takes
/// the second after the first.
fn seat(world: &mut World, seated: u16) {
    let grid = world.grid();
    let mut taken: Vec<Axial> = Vec::new();
    for faction in 0..seated {
        let mut founded = false;
        for spacing in [12, 0] {
            for index in 0..grid.tile_count() {
                let address =
                    Axial::new((index % grid.width()) as i32, (index / grid.width()) as i32);
                if !world.admits_a_unit(address) {
                    continue;
                }
                if taken.iter().any(|place| {
                    (place.q - address.q).abs() < spacing || (place.r - address.r).abs() < spacing
                }) {
                    continue;
                }
                if world
                    .found_group_at(address, GROUP, FactionId(faction))
                    .is_ok()
                {
                    taken.push(address);
                    founded = true;
                    break;
                }
            }
            if founded {
                break;
            }
        }
        assert!(founded, "faction {faction} must find a place");
    }
}

/// Returns one count of the census by name.
fn census(world: &World, name: &str) -> i64 {
    world
        .subsystem_census()
        .into_iter()
        .find(|(row, _)| *row == name)
        .expect("the row exists")
        .1
}

/// Writes the store of the site that one faction trades from.
fn set_store(world: &mut World, faction: FactionId, quantity: Fix32) {
    let site = world
        .trading_site_of(faction)
        .expect("the faction holds a site");
    world
        .set_settlement_store(site, GOODS, quantity)
        .expect("the commodity is inside the set");
}

/// Puts a unit of each faction on ground the other holds.
///
/// The offer verb, the counter verb and the acceptance verb all ask that a
/// unit of the speaker stands in the territory of the listener. The holding
/// rule takes such a tile for the speaker on the next step, so the fixture
/// renews the presence before every step.
fn renew_presence(world: &mut World, one: FactionId, other: FactionId) {
    for (speaker, listener) in [(one, other), (other, one)] {
        if world.stands_in_territory_of(speaker, listener) {
            continue;
        }
        let grid = world.grid();
        let mut place = None;
        for index in 0..grid.tile_count() {
            let address = Axial::new((index % grid.width()) as i32, (index / grid.width()) as i32);
            if world.tile_holder(address) == Some(cachette_core::holding::Holder::of(listener))
                && world.admits_a_unit(address)
            {
                place = Some(address);
                break;
            }
        }
        if let Some(place) = place {
            let _ = world.spawn_soldier(place, speaker);
        }
    }
}

/// Holds the pair at the peace edge.
///
/// A faction now reaches the war band inside the run of this fixture, and a
/// pair at war holds no negotiation. A fixture that let the relation fall
/// would end its run with an empty board, so it would measure the war path
/// and not the trade path. The fixture writes the relation before every step,
/// in the same way it writes the two stores and the presence.
fn hold_the_peace(world: &mut World, one: FactionId, other: FactionId) {
    let edge = world.relation_rules().peace_edge;
    assert!(
        world.set_relation(one, other, edge),
        "the pair names factions"
    );
    assert!(
        world.set_relation(other, one, edge),
        "the pair names factions"
    );
}

/// Founds the two groups a few tiles apart.
///
/// **The two sites sit in one level 1 cell.** A destination field holds one
/// direction for each cell, so a unit sent to a tile arrives in the cell of
/// that tile and then walks by its own choice. A fixture that put the two
/// sites in two distant cells would measure the walk and not the trade.
fn seat_close(world: &mut World) {
    let grid = world.grid();
    let mut first = None;
    for index in 0..grid.tile_count() {
        let address = Axial::new((index % grid.width()) as i32, (index / grid.width()) as i32);
        if !world.admits_a_unit(address) {
            continue;
        }
        match first {
            None => {
                if world.found_group_at(address, GROUP, ZERO).is_ok() {
                    first = Some(address);
                }
            }
            Some(place) => {
                if (address.q - place.q).abs() + (address.r - place.r).abs() > 4 {
                    continue;
                }
                if world.found_group_at(address, GROUP, ONE).is_ok() {
                    return;
                }
            }
        }
    }
    panic!("the fixture found no pair of places close together");
}

/// Builds a world in which one faction holds a full store and the other holds
/// an empty one, and both post a board on every tick.
fn a_trading_world(seed: u64) -> World {
    let mut world = World::new(config(2, seed)).expect("the extent describes a world");
    seat_close(&mut world);
    world.set_surplus_mark(MARK);
    world
        .set_advertisement_schedule(1, 0)
        .expect("the period is inside the range");
    world.step(1).expect("the step runs");
    // **The extreme the carrier rule needs.** The lowest identities of each
    // faction take a type whose carry capacity is zero. A fixture in which
    // every unit carries would never show a carrier rule that reads no
    // capability column, because the lowest identity would carry anyway.
    for faction in [ZERO, ONE] {
        let mut units: Vec<Entity> = world.soldiers().iter_faction(faction).collect();
        units.sort_unstable_by_key(|unit| unit.to_bits());
        units.truncate(2);
        world.set_unit_type_set(&units, SOLDIER);
    }
    world
}

// ---------------------------------------------------------------------------
// The keyed draws
// ---------------------------------------------------------------------------

/// The stores that make every good tie, so the tie-break draw decides.
const TIED: [i64; RESOURCE_KIND_COUNT] = [4; RESOURCE_KIND_COUNT];

#[test]
fn the_asking_good_draw_depends_on_the_tick() {
    let mut seen = BTreeSet::new();
    for tick in 0..64u64 {
        seen.insert(asking_good_of(99, Tick(tick), ZERO, 3, &TIED, 0));
    }
    assert!(
        seen.len() > 1,
        "one faction, one draw index, sixty-four ticks: the good must change with the tick"
    );
}

#[test]
fn the_asking_good_draw_depends_on_the_faction() {
    let mut seen = BTreeSet::new();
    for faction in 0..64u16 {
        seen.insert(asking_good_of(99, Tick(7), FactionId(faction), 3, &TIED, 0));
    }
    assert!(
        seen.len() > 1,
        "one tick, one draw index, sixty-four factions: the good must change with the faction"
    );
}

#[test]
fn the_asking_good_draw_depends_on_the_draw_index() {
    let mut seen = BTreeSet::new();
    for draw in 0..64u32 {
        seen.insert(asking_good_of(99, Tick(7), ZERO, draw, &TIED, 0));
    }
    assert!(
        seen.len() > 1,
        "one tick, one faction, sixty-four draw indexes: the good must change with the index"
    );
}

#[test]
fn the_asking_good_never_names_the_good_of_its_own_row() {
    for good in 0..RESOURCE_KIND_COUNT as u8 {
        for tick in 0..32u64 {
            let asking = asking_good_of(5, Tick(tick), ONE, 2, &TIED, good);
            assert_ne!(asking, good, "a faction never asks for what it is selling");
        }
    }
}

/// A weight vector whose trade weight makes the step draw answer both ways.
const MIXED: FactionWeights = FactionWeights {
    war: 1,
    trade: 4,
    build: 1,
    renown: 1,
    settle: 1,
};

#[test]
fn the_trade_step_draw_depends_on_the_tick() {
    let mut seen = BTreeSet::new();
    for tick in 0..64u64 {
        seen.insert(wants_trade_step(31, Tick(tick), ZERO, 4, MIXED));
    }
    assert_eq!(
        seen.len(),
        2,
        "one faction, one draw index, sixty-four ticks: the answer must change with the tick"
    );
}

#[test]
fn the_trade_step_draw_depends_on_the_faction() {
    let mut seen = BTreeSet::new();
    for faction in 0..64u16 {
        seen.insert(wants_trade_step(31, Tick(9), FactionId(faction), 4, MIXED));
    }
    assert_eq!(
        seen.len(),
        2,
        "one tick, one draw index, sixty-four factions: the answer must change with the faction"
    );
}

#[test]
fn the_trade_step_draw_depends_on_the_draw_index() {
    let mut seen = BTreeSet::new();
    for draw in 0..64u32 {
        seen.insert(wants_trade_step(31, Tick(9), ZERO, draw, MIXED));
    }
    assert_eq!(
        seen.len(),
        2,
        "one tick, one faction, sixty-four draw indexes: the answer must change with the index"
    );
}

// ---------------------------------------------------------------------------
// The board
// ---------------------------------------------------------------------------

#[test]
fn a_board_is_written_only_on_its_schedule_tick() {
    let mut world = World::new(config(2, 21)).expect("the extent describes a world");
    seat(&mut world, 2);
    world
        .set_advertisement_schedule(5, 2)
        .expect("the period is inside the range");
    // The census row is a total for the run, so the test reads what one tick
    // added to it and not the row itself.
    let mut before = census(&world, "boards_written");
    for _ in 0..21 {
        world.step(1).expect("the step runs");
        let tick = world.tick().0;
        let after = census(&world, "boards_written");
        let written = after - before;
        before = after;
        if tick % 5 == 2 {
            assert!(written > 0, "tick {tick} is a schedule tick and wrote none");
        } else {
            assert_eq!(
                written, 0,
                "tick {tick} is no schedule tick and wrote {written}"
            );
        }
    }
}

#[test]
fn a_surplus_posts_an_offer_and_a_shortfall_posts_a_want() {
    // The extreme: one site at the ceiling of the scale and one at zero.
    let mut world = a_trading_world(23);
    set_store(&mut world, ZERO, FULL);
    set_store(&mut world, ONE, Fix32::ZERO);
    world.step(1).expect("the step runs");

    let seller: Vec<Advert> = world.market(ZERO).to_vec();
    let buyer: Vec<Advert> = world.market(ONE).to_vec();
    assert!(
        seller
            .iter()
            .any(|row| !row.is_empty() && row.wants == ADVERT_OFFERS),
        "the full store must post an offer: {seller:?}"
    );
    assert!(
        buyer
            .iter()
            .any(|row| !row.is_empty() && row.wants == ADVERT_WANTS),
        "the empty store must post a want: {buyer:?}"
    );
    for row in seller.iter().chain(buyer.iter()) {
        if row.is_empty() {
            continue;
        }
        assert_ne!(
            row.asking_good, row.good,
            "a row never asks for its own good"
        );
    }
}

// ---------------------------------------------------------------------------
// The negotiation
// ---------------------------------------------------------------------------

/// Drives the engine until the pair holds a bound contract, and returns the
/// tick count it took.
///
/// The fixture pins the two stores and the presence before every step, so the
/// engine reads one extreme on every tick.
fn drive_to_a_contract(world: &mut World, bound: u64) -> Option<u64> {
    for tick in 1..=bound {
        set_store(
            world,
            ZERO,
            Fix32::from_int(i16::try_from(MARK).unwrap_or(0) + 2),
        );
        set_store(world, ONE, Fix32::ZERO);
        renew_presence(world, ZERO, ONE);
        world.step(1).expect("the step runs");
        if world
            .trade_book()
            .iter()
            .any(|row| row.status == TRADE_BOUND)
        {
            return Some(tick);
        }
    }
    None
}

#[test]
fn two_controllers_bind_a_contract_and_the_census_counts_it() {
    let mut world = a_trading_world(29);
    let took = drive_to_a_contract(&mut world, 400).expect("the pair must bind a contract");
    assert!(took > 0);
    let bound = world
        .trade_book()
        .iter()
        .find(|row| row.status == TRADE_BOUND)
        .copied()
        .expect("a bound row exists");
    assert!(bound.give_amount > 0 && bound.take_amount > 0);
    assert_ne!(bound.give_kind, bound.take_kind, "the two sides differ");
}

#[test]
fn no_offer_crosses_a_war_pair() {
    let mut world = a_trading_world(31);
    // The two factions declare war before anybody speaks. The offer verb
    // refuses a pair in the war band, and the controller never asks.
    // One step below the war edge is not enough. The drift lifts an entry
    // toward peace by one step on every drift period, so a pair set one below
    // the edge leaves the war band on the first drift and the fixture then
    // measures a pair at peace. Eight steps below holds the pair inside the
    // band for the whole loop.
    let edge = world.relation_rules().war_edge - 8;
    world.set_relation(ZERO, ONE, edge);
    world.set_relation(ONE, ZERO, edge);
    assert!(world.at_war(ZERO, ONE));
    for _ in 0..120 {
        set_store(&mut world, ZERO, FULL);
        set_store(&mut world, ONE, Fix32::ZERO);
        renew_presence(&mut world, ZERO, ONE);
        world.set_relation(ZERO, ONE, edge);
        world.set_relation(ONE, ZERO, edge);
        world.step(1).expect("the step runs");
        assert_eq!(
            census(&world, "offers_made"),
            0,
            "a pair at war never receives an offer: at_war={} relation={:?}",
            world.at_war(ZERO, ONE),
            world.relation(ZERO, ONE)
        );
        assert!(
            world
                .controller_log()
                .iter()
                .all(|entry| world.action_schema().verb_of(entry.action) != Some(Verb::Trade)),
            "the controller asked for a refusal it could see coming"
        );
        assert!(
            world.trade_book().iter().all(|row| row.status == 0),
            "a pair at war holds no negotiation"
        );
    }
}

/// Returns the faction that speaks next in the live negotiation of the pair,
/// or `None` when the pair holds none.
///
/// The row for a pair holds the terms in one orientation. The party that
/// opened the pair speaks first, so the other party answers an offer and the
/// party that opened answers a counteroffer.
fn speaker_due(world: &World) -> Option<FactionId> {
    for (proposer, responder) in [(ZERO, ONE), (ONE, ZERO)] {
        let row = world.trade_row(proposer, responder)?;
        if !row.is_live() || row.is_bound() {
            continue;
        }
        return Some(if row.status == 1 { responder } else { proposer });
    }
    None
}

#[test]
fn no_second_negotiation_opens_with_one_pair() {
    let mut world = a_trading_world(37);
    let mut opened = 0i64;
    let mut answered = 0u32;
    for _ in 0..200 {
        set_store(&mut world, ZERO, FULL);
        set_store(&mut world, ONE, Fix32::ZERO);
        renew_presence(&mut world, ZERO, ONE);
        let due = speaker_due(&world);
        world.step(1).expect("the step runs");
        // **A faction that is not due to speak takes no step.** The pair
        // already holds a live negotiation, and the stage never opens a
        // second one with the same pair. A stage that read no live row would
        // ask the verb, and the verb would refuse.
        for faction in [ZERO, ONE] {
            let spoke = world.controller_log().iter().any(|entry| {
                entry.faction == faction
                    && world.action_schema().verb_of(entry.action) == Some(Verb::Trade)
            });
            if !spoke {
                continue;
            }
            match due {
                None => {}
                Some(speaker) => {
                    assert_eq!(
                        speaker, faction,
                        "a faction spoke while the pair waited on the other"
                    );
                    answered += 1;
                }
            }
        }
        opened += census(&world, "offers_made");
        let live = world
            .trade_book()
            .iter()
            .filter(|row| row.is_live())
            .count();
        assert!(live <= 1, "one pair holds at most one live negotiation");
        // One faction takes at most one negotiation step on one tick.
        for faction in [ZERO, ONE] {
            let steps = world
                .controller_log()
                .iter()
                .filter(|entry| {
                    entry.faction == faction
                        && world.action_schema().verb_of(entry.action) == Some(Verb::Trade)
                })
                .count();
            assert!(steps <= 1, "one faction takes one step: {steps}");
        }
    }
    assert!(
        answered > 0,
        "the fixture never reached a live negotiation, so it proved nothing"
    );
    assert!(opened > 0, "the fixture never opened a negotiation");
}

// ---------------------------------------------------------------------------
// The carriers
// ---------------------------------------------------------------------------

/// Returns what the live contract of the world has been paid, on both sides.
///
/// A row that failed or settled earlier keeps what it was paid, so the reader
/// looks at the bound row alone. A helper that took the largest number in the
/// plane would answer with the payment of an older contract.
fn delivered(world: &World) -> u32 {
    world
        .trade_book()
        .iter()
        .filter(|row| row.is_bound())
        .map(|row| row.given.saturating_add(row.taken))
        .max()
        .unwrap_or(0)
}

#[test]
fn a_contract_binds_and_a_carrier_delivers() {
    let mut world = a_trading_world(41);
    world.set_contract_term(4000);
    // **The carry mark must stay above what a carrier picks up on the way.**
    // A unit that holds a home and a load at the mark is laden, and a laden
    // unit walks home rather than where it was sent, so a carrier under a low
    // mark never arrives. The fixture raises the mark, so the test measures
    // the walk and not the return.
    world.set_carry_mark(Amount(u32::MAX));
    let bound_at = drive_to_a_contract(&mut world, 400).expect("the pair must bind a contract");
    let width = u32::from(world.faction_count().max(1));
    let mut assigned = false;
    let mut moved = None;
    for _ in 0..600 {
        renew_presence(&mut world, ZERO, ONE);
        // **The fixture supplies the load and nothing else.** The engine
        // assigned the carrier, the engine walks it, and the engine moves the
        // quantity when it arrives.
        let owed: Vec<(Entity, ResourceKind)> = world
            .carrier_units()
            .into_iter()
            .filter_map(|(unit, index, faction)| {
                let row = world.trade_book().get(index as usize)?;
                // The row index is the proposer times the faction count plus
                // the responder, so the quotient names the party that opened
                // the pair and owes the give side.
                let kind = if index / width == u32::from(faction.0) {
                    row.give_kind
                } else {
                    row.take_kind
                };
                Some((unit, ResourceKind::from_u8(kind)?))
            })
            .collect();
        for (unit, kind) in owed {
            world.order_gather(unit, kind);
        }
        if !world.carrier_assignments().is_empty() {
            assigned = true;
        }
        let before = delivered(&world);
        world.step(1).expect("the step runs");
        // Every carrier the controller assigned is a live unit of the faction
        // that owes, its type carries, and it is sent on the plane of its
        // faction. A carrier that failed any of those would be a row that the
        // engine wrote and nothing acted on.
        for (unit, _, faction) in world.carrier_units() {
            assert_eq!(world.soldiers().faction(unit), Some(faction));
            assert_eq!(world.sent_to(unit), Some(Some(faction.0)));
            let unit_type = world.soldiers().unit_type(unit).expect("the unit is live");
            assert!(
                world.unit_types().row(unit_type).carry_capacity > 0,
                "a unit that cannot carry was assigned as a carrier"
            );
        }
        if delivered(&world) > before {
            moved = Some(delivered(&world) - before);
            break;
        }
    }
    assert!(
        assigned,
        "the controller assigned no carrier after the contract bound at tick {bound_at}"
    );
    assert!(
        moved.is_some(),
        "no quantity moved on the bound contract inside the tick bound"
    );
    assert!(
        census(&world, "carriers_assigned") >= 0,
        "the census row answers"
    );
}

#[test]
fn a_carrier_is_released_when_the_contract_ends() {
    let mut world = a_trading_world(43);
    world.set_contract_term(20);
    drive_to_a_contract(&mut world, 400).expect("the pair must bind a contract");
    let mut held = false;
    for _ in 0..120 {
        renew_presence(&mut world, ZERO, ONE);
        world.step(1).expect("the step runs");
        if !world.carrier_assignments().is_empty() {
            held = true;
        }
        let live = world.trade_book().iter().any(|row| row.is_bound());
        if held && !live {
            // The contract ended. The carriers go one tick later, because the
            // release runs at the controller stage of the next step.
            world.step(1).expect("the step runs");
            assert!(
                world.carrier_assignments().is_empty(),
                "a contract that ended releases every carrier: {:?}",
                world.carrier_assignments()
            );
            return;
        }
    }
    panic!("the fixture never assigned a carrier and then ended the contract");
}

// ---------------------------------------------------------------------------
// Determinism
// ---------------------------------------------------------------------------

#[test]
fn the_trade_is_the_same_at_one_two_and_twelve_threads() {
    let mut hashes = Vec::new();
    for threads in [1usize, 2, 12] {
        let mut world = a_trading_world(47);
        for _ in 0..120 {
            set_store(&mut world, ZERO, FULL);
            set_store(&mut world, ONE, Fix32::ZERO);
            renew_presence(&mut world, ZERO, ONE);
            hold_the_peace(&mut world, ZERO, ONE);
            world.step(threads).expect("the step runs");
        }
        assert!(
            world.trade_book().iter().any(|row| row.is_live()),
            "the fixture ran with no trade in flight"
        );
        hashes.push(world.state_hash());
    }
    assert_eq!(hashes[0], hashes[1], "one thread and two must agree");
    assert_eq!(hashes[1], hashes[2], "two threads and twelve must agree");
}
