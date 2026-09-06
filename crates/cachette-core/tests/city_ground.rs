//! Held ground is the ground within reach of a city its faction owns.
//!
//! A tile is held by the faction of the nearest city that reaches it. Two
//! cities at one distance resolve by the lower settlement slot. A tile no
//! city reaches is held by nobody. The reach of a city is a base plus one
//! step for each block of finished upgrades on the ground it held at the end
//! of the previous step, capped at a bound. A build outside the builder's own
//! ground is refused unless the kind is a road.[^1]
//!
//! **Every fixture here is built for an extreme, not for a typical world.**
//! A city at the world edge, two cities at one distance from one tile, a
//! faction with ground and no city, and a city at the bound of its reach. A
//! fixture that models the typical case never supplies the input that fails
//! the assertion.[^2]
//!
//! The tests drive the world step and the public verbs, because the step and
//! the verbs are what must invoke the rules.[^3]
//!
//! # References
//!
//! [^1]: ADR-0150, held ground is the ground within reach of a city its faction owns, decisions D1, D2, D4 and D6. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
//! [^2]: Testing rules, section 2a. `.claude/rules/testing.md`
//! [^3]: Testing rules, drive the real caller. `.claude/rules/testing.md`

use cachette_core::holding::{Holder, ReachRules};
use cachette_core::trade::Consideration;
use cachette_core::upgrade::UpgradeCategory;
use cachette_core::{Axial, Entity, FactionId, World, WorldConfig};

/// The extent of the worlds below.
const EXTENT: u32 = 48;

/// Builds a world of the extent, with the reach rules the test names.
fn world(seed: u64, factions: u16, rules: ReachRules) -> World {
    let mut world = World::new(WorldConfig {
        width: EXTENT,
        height: EXTENT,
        seed,
        faction_count: factions,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    })
    .expect("the extent must describe a world");
    world.set_reach_rules(rules);
    world
}

/// Returns every address of a world, in tile index order.
fn addresses(world: &World) -> Vec<Axial> {
    let grid = world.grid();
    (0..grid.tile_count())
        .map(|index| Axial::new((index % grid.width()) as i32, (index / grid.width()) as i32))
        .collect()
}

/// Returns the tiles a city of this reach holds, counted by a full pass.
///
/// The count is the passable tiles inside the reach that lie inside the
/// world. It is the answer the rewrite must produce, derived a second way.
fn inside_the_reach(world: &World, seat: Axial, reach: u32) -> usize {
    addresses(world)
        .into_iter()
        .filter(|address| address.distance(seat) <= reach && world.admits_a_unit(*address))
        .count()
}

#[test]
fn a_city_at_the_world_edge_holds_only_the_ground_inside_the_world() {
    // The extreme is the edge. A rule that walked a disc without asking the
    // grid would count tiles outside the world, and a rule that stopped at
    // the first refusal would hold fewer than the ground inside it.
    let rules = ReachRules::new(3, 1, 6);
    for seed in 0..40u64 {
        let mut field = world(seed, 2, rules);
        let corner = Axial::new(0, 0);
        if !field.admits_a_unit(corner) {
            continue;
        }
        let site = field
            .found_settlement(corner, FactionId(0))
            .expect("the corner admits a unit");
        field.step(2).expect("the step must run");
        assert!(field.check_invariants());

        let expected = inside_the_reach(&field, corner, rules.base());
        assert!(
            expected > 1,
            "the corner of seed {seed} reaches one tile, so the edge case is thin"
        );
        assert_eq!(
            field.holding_of(FactionId(0)) as usize,
            expected,
            "the city at the world edge holds the wrong count"
        );
        assert_eq!(field.city_reach(site), Some(rules.base()));
        // A tile one step outside the reach is held by nobody, whatever the
        // ground is.
        let outside = Axial::new(0, rules.base() as i32 + 1);
        assert_eq!(
            field.tile_holder(outside).and_then(Holder::faction),
            None,
            "a tile outside the reach is held"
        );
        return;
    }
    panic!("no seed below 40 puts open ground at the corner");
}

#[test]
fn two_cities_at_one_distance_give_the_tile_to_the_lower_slot() {
    // The tie is the extreme. The city of the higher faction is founded
    // first, so it takes the lower slot. A rule that resolved by the faction
    // identifier would answer the other way, and a rule that resolved by the
    // order it visited the tiles would answer either way.
    let rules = ReachRules::new(4, 1, 8);
    for seed in 0..60u64 {
        let mut field = world(seed, 3, rules);
        let Some((left, middle, right)) = addresses(&field).into_iter().find_map(|address| {
            let left = address;
            let middle = Axial::new(address.q + 2, address.r);
            let right = Axial::new(address.q + 4, address.r);
            (field.admits_a_unit(left) && field.admits_a_unit(middle) && field.admits_a_unit(right))
                .then_some((left, middle, right))
        }) else {
            continue;
        };
        // The first founding takes settlement slot zero.
        let first = field
            .found_settlement(right, FactionId(2))
            .expect("the ground admits a city");
        let second = field
            .found_settlement(left, FactionId(1))
            .expect("the ground admits a city");
        assert!(
            field.settlements().slot_of(first) < field.settlements().slot_of(second),
            "the fixture did not put the higher faction in the lower slot"
        );
        assert_eq!(middle.distance(left), middle.distance(right));

        field.step(1).expect("the step must run");
        assert!(field.check_invariants());
        assert_eq!(
            field.tile_holder(middle).and_then(Holder::faction),
            Some(FactionId(2)),
            "the tile at one distance from two cities did not go to the lower slot"
        );
        return;
    }
    panic!("no seed below 60 gives three open tiles in a row");
}

#[test]
fn a_faction_with_ground_and_no_city_holds_nothing_after_one_step() {
    // The rule reads no previous holder, so ground cannot outlive the city
    // that reached it. The fixture must first hold ground, or the assertion
    // measures a faction that never held any.
    let mut field = world(7, 2, ReachRules::new(3, 1, 6));
    let seat = addresses(&field)
        .into_iter()
        .find(|address| field.admits_a_unit(*address))
        .expect("the world admits a unit somewhere");
    let site = field
        .found_settlement(seat, FactionId(0))
        .expect("the ground admits a city");
    field.step(1).expect("the step must run");
    let held = field.holding_of(FactionId(0));
    assert!(held > 0, "the fixture never held ground");

    assert!(field.destroy_settlement(site));
    field.step(1).expect("the step must run");
    assert!(field.check_invariants());
    assert_eq!(
        field.holding_of(FactionId(0)),
        0,
        "a faction with no city still holds ground"
    );
    assert_eq!(field.holding().held_tiles(), 0);
    assert_eq!(field.city_reach(site), None);
}

/// Zones one road project and puts one builder on the tile.
///
/// **The plan comes first.** A road asks for no held ground, and a category
/// that asks for no held ground is laid only inside a project, so a fixture
/// that ordered the build alone would measure the refusal.[^1]
///
/// # References
///
/// [^1]: ADR-0152, a faction plans its roads and zones with one solver, decisions D3 and D4. `docs/adrs/accepted/adr-0152-a-faction-plans-its-roads-and-zones-with-one-solver.md`
fn build_a_road(field: &mut World, address: Axial, faction: FactionId) -> Entity {
    let unit = field
        .spawn_soldier(address, faction)
        .expect("the ground admits a unit");
    assert!(
        field
            .zone_project(faction, address, UpgradeCategory::ROAD)
            .is_ok(),
        "the plan refused a road project at {address:?}"
    );
    assert!(
        field.order_build(unit, UpgradeCategory::ROAD).is_ok(),
        "the verb refused a road at {address:?}"
    );
    unit
}

#[test]
fn a_finished_upgrade_extends_the_reach_by_one_step_and_never_past_the_cap() {
    // Three cases in one fixture: an unfinished upgrade adds nothing, the
    // count crossing the block adds exactly one step, and the cap stops the
    // growth. Each needs the fixture to reach the state before it.
    let rules = ReachRules::new(2, 1, 3);
    let mut field = world(11, 2, rules);
    let seat = addresses(&field)
        .into_iter()
        .find(|address| {
            field.admits_a_unit(*address)
                && field.admits_a_unit(Axial::new(address.q + 1, address.r))
                && field.admits_a_unit(Axial::new(address.q + 2, address.r))
        })
        .expect("the world holds a seat with open ground beside it");
    let site = field
        .found_settlement(seat, FactionId(0))
        .expect("the ground admits a city");
    field.step(1).expect("the step must run");
    assert_eq!(field.city_reach(site), Some(2), "the base reach is wrong");
    let base_ground = field.holding_of(FactionId(0));

    // One road under construction. The reach must not move.
    let first = Axial::new(seat.q + 1, seat.r);
    build_a_road(&mut field, first, FactionId(0));
    field.step(1).expect("the step must run");
    assert!(
        field
            .upgrade_at(first)
            .is_some_and(|site| !site.is_complete()),
        "the fixture finished the road at once, so the unfinished case is untested"
    );
    assert_eq!(
        field.city_reach(site),
        Some(2),
        "an unfinished upgrade extended the reach"
    );

    // The road finishes, and the reach grows by exactly one step.
    for _ in 0..cachette_core::DEFAULT_UPGRADE_TABLE.work_above(UpgradeCategory::ROAD, 0) {
        field.step(1).expect("the step must run");
    }
    assert_eq!(field.finished_upgrade(first), Some(UpgradeCategory::ROAD));
    assert_eq!(
        field.city_reach(site),
        Some(3),
        "one finished upgrade did not extend the reach by one step"
    );
    let grown = field.holding_of(FactionId(0));
    assert!(
        grown > base_ground,
        "the reach grew and the ground did not follow"
    );

    // A second finished road would earn another step, and the cap refuses
    // it. Without the cap the reach would answer four.
    let second = Axial::new(seat.q + 2, seat.r);
    build_a_road(&mut field, second, FactionId(0));
    for _ in 0..=cachette_core::DEFAULT_UPGRADE_TABLE.work_above(UpgradeCategory::ROAD, 0) {
        field.step(1).expect("the step must run");
    }
    assert_eq!(field.finished_upgrade(second), Some(UpgradeCategory::ROAD));
    assert_eq!(
        field.city_reach(site),
        Some(rules.cap()),
        "the reach passed the cap"
    );
    assert_eq!(
        field.holding_of(FactionId(0)),
        grown,
        "the ground grew past the cap"
    );
    assert!(field.check_invariants());
}

#[test]
fn the_verb_refuses_a_build_off_own_ground_and_permits_a_zoned_road() {
    let mut field = world(11, 3, ReachRules::new(2, 1, 4));
    let seat = addresses(&field)
        .into_iter()
        .find(|address| field.admits_a_unit(*address))
        .expect("the world admits a unit somewhere");
    field
        .found_settlement(seat, FactionId(0))
        .expect("the ground admits a city");
    field.step(1).expect("the step must run");
    assert_eq!(
        field.tile_holder(seat).and_then(Holder::faction),
        Some(FactionId(0)),
        "the fixture did not give the ground to the first faction"
    );

    // A unit of another faction stands on ground the first faction holds.
    let guest = field
        .spawn_soldier(seat, FactionId(1))
        .expect("the ground admits a unit");
    assert!(
        field.order_build(guest, UpgradeCategory::TERRACE).is_err(),
        "the verb took a terrace on ground the builder does not hold"
    );
    assert_eq!(field.build_order(guest), Some(None));
    // A road asks for no held ground, so the ground rule lets it cross. The
    // plan is the second bound, and no project zones this tile yet.
    assert!(
        field.order_build(guest, UpgradeCategory::ROAD).is_err(),
        "the verb took a road that no project zones"
    );
    assert!(
        field
            .zone_project(FactionId(1), seat, UpgradeCategory::ROAD)
            .is_ok(),
        "the plan refused a road project on ground another faction holds"
    );
    assert!(
        field.order_build(guest, UpgradeCategory::ROAD).is_ok(),
        "the verb refused a road that the plan zones"
    );
    assert_eq!(field.build_order(guest), Some(Some(UpgradeCategory::ROAD)));

    // The owner of the ground may build anything on it.
    let owner = field
        .spawn_soldier(seat, FactionId(0))
        .expect("the ground admits a unit");
    assert!(field.order_build(owner, UpgradeCategory::TERRACE).is_ok());

    // The set form counts what it refused, and that count is what the
    // controller reads.
    let refused = field.order_build_set(&[guest], UpgradeCategory::TERRACE);
    assert_eq!(refused, 1, "the set form took a build it must refuse");
}

#[test]
fn a_build_stops_when_the_ground_changes_hands() {
    // The pass applies the rule on every step, so a build that was permitted
    // when it was ordered stops when its ground goes. Only the pass can
    // catch this, because the verb ran before the ground moved.
    let mut field = world(11, 2, ReachRules::new(2, 1, 4));
    let seat = addresses(&field)
        .into_iter()
        .find(|address| field.admits_a_unit(*address))
        .expect("the world admits a unit somewhere");
    let site = field
        .found_settlement(seat, FactionId(0))
        .expect("the ground admits a city");
    field.step(1).expect("the step must run");
    let unit = field
        .spawn_soldier(seat, FactionId(0))
        .expect("the ground admits a unit");
    assert!(field.order_build(unit, UpgradeCategory::TERRACE).is_ok());
    field.step(1).expect("the step must run");
    let before = field
        .upgrade_at(seat)
        .expect("the builder started the terrace")
        .progress;
    assert!(before.0 > 0, "the fixture built nothing");

    // The build pass runs before the rewrite, so the step that takes the
    // ground away still builds once. The step after it must not.
    assert!(field.destroy_settlement(site));
    field.step(1).expect("the step must run");
    assert_eq!(
        field.tile_holder(seat).and_then(Holder::faction),
        None,
        "the ground did not change hands"
    );
    let stalled = field
        .upgrade_at(seat)
        .expect("the entry stays after the ground goes")
        .progress;
    assert!(stalled.0 >= before.0, "the entry lost the work it did");
    field.step(1).expect("the step must run");
    assert_eq!(
        field.upgrade_at(seat).expect("the entry stays").progress,
        stalled,
        "the build went on after its ground went"
    );
}

#[test]
fn the_controller_build_order_is_refused_and_counted_when_the_faction_holds_nothing() {
    // The refusal must reach the controller through the verb it shares with
    // a Python caller, and the refused count is what a watcher reads.[^1]
    //
    // [^1]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decision D3. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
    let mut field = world(11, 2, ReachRules::new(2, 1, 4));
    let seat = addresses(&field)
        .into_iter()
        .find(|address| field.admits_a_unit(*address))
        .expect("the world admits a unit somewhere");
    let founding = field
        .found_group_at(seat, 8, FactionId(0))
        .expect("the ground carries a founding");
    // The second faction founds as well, and keeps its city. Without it the
    // domination reader ends the game on the first tick, the controller then
    // emits nothing, and this test would measure the game end.[^2]
    //
    // [^2]: ADR-0148, a game end is recorded once and stops the controllers, decision D3. `docs/adrs/accepted/adr-0148-a-game-end-is-recorded-once-and-stops-the-controllers.md`
    let rival = addresses(&field)
        .into_iter()
        .find(|address| address.distance(seat) > 12 && field.admits_a_unit(*address))
        .expect("the world holds open ground away from the first seat");
    field
        .found_group_at(rival, 8, FactionId(1))
        .expect("the ground carries a second founding");
    // The first faction keeps its seat and its people, and loses its city. It
    // then holds no ground, so every build but a road is refused.
    assert!(field.destroy_settlement(founding.settlement()));

    let mut refused_builds = 0u32;
    let mut build_commands = 0u32;
    for _ in 0..80 {
        field.step(1).expect("the step must run");
        for entry in field.controller_log() {
            if entry.kind != cachette_core::controller::COMMAND_BUILD {
                continue;
            }
            build_commands += 1;
            if entry.applied == 0 {
                refused_builds += 1;
            }
        }
    }
    assert!(
        build_commands > 0,
        "the controller ordered no build, so the refusal was never reached"
    );
    assert!(
        refused_builds > 0,
        "the controller ordered {build_commands} builds and none was refused"
    );
}

#[test]
fn traded_land_outside_every_creditor_city_is_unheld_after_the_next_step() {
    // The rewrite reads no previous holder, so a traded tile keeps its new
    // holder only where the cities give it. The tile is chosen outside the
    // reach of the creditor's only city, which is the case the record calls
    // out.[^1]
    //
    // [^1]: ADR-0150, held ground is the ground within reach of a city its faction owns, decision D6. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
    let rules = ReachRules::new(2, 1, 4);
    let mut field = world(11, 3, rules);
    let open: Vec<Axial> = addresses(&field)
        .into_iter()
        .filter(|address| field.admits_a_unit(*address))
        .collect();
    let debtor_seat = open[0];
    let creditor_seat = *open
        .iter()
        .find(|address| address.distance(debtor_seat) > 4 * rules.cap())
        .expect("the world holds open ground far from the first seat");
    let debtor_city = field
        .found_settlement(debtor_seat, FactionId(0))
        .expect("the ground admits a city");
    field
        .found_settlement(creditor_seat, FactionId(1))
        .expect("the ground admits a city");
    field.step(1).expect("the step must run");

    // The traded tile is ground the debtor holds and no city of the creditor
    // reaches. That is the case the record calls out.
    let traded = *open
        .iter()
        .find(|address| {
            field.tile_holder(**address).and_then(Holder::faction) == Some(FactionId(0))
                && address.distance(creditor_seat) > rules.cap()
                && field.upgrade_at(**address).is_none()
        })
        .expect("the debtor holds ground outside the reach of the creditor");
    let tile = field
        .grid()
        .index_of(traded)
        .expect("the address is inside the world");

    // A trade needs a unit of each party on the ground of the other.
    let creditor_ground = *open
        .iter()
        .find(|address| {
            field.tile_holder(**address).and_then(Holder::faction) == Some(FactionId(1))
        })
        .expect("the creditor holds ground");
    field
        .spawn_soldier(creditor_ground, FactionId(0))
        .expect("the ground admits a unit");
    field
        .spawn_soldier(traded, FactionId(1))
        .expect("the ground admits a unit");
    field
        .offer_consideration(
            FactionId(0),
            FactionId(1),
            Consideration::land(vec![tile]),
            Consideration::relation(0, 1),
            50,
        )
        .expect("the offer must be accepted");
    field
        .accept_trade(FactionId(1), FactionId(0))
        .expect("the acceptance must succeed");
    field.step(1).expect("the step must run");
    assert_eq!(
        field.tile_holder(traded).and_then(Holder::faction),
        Some(FactionId(1)),
        "the land side did not write the holder"
    );
    assert_eq!(field.holds(FactionId(1), traded), Some(true));

    // The debtor loses its city, so no city of either faction reaches the
    // traded tile. The next rewrite reads no previous holder, so the tile is
    // held by nobody.
    assert!(field.destroy_settlement(debtor_city));
    field.step(1).expect("the step must run");
    assert_eq!(
        field.tile_holder(traded).and_then(Holder::faction),
        None,
        "the traded tile outside every city of the creditor stayed held"
    );
    assert!(field.check_invariants());
}
