//! A city changes hands, a city is razed, and a faction leaves the game.
//!
//! A settlement's faction was written at the founding and never again, so a
//! war could take every tile around a city and kill everyone in it while the
//! city stayed with its founder for ever. Three verbs close that: the step
//! captures a site that stands undefended under a rival, a caller razes one,
//! and the step removes a faction that holds no site and no unit.[^1] [^2]
//!
//! **Every fixture here is built for an extreme, not for a typical world.**
//! The site tile is an island, whose every neighbour refuses a unit, so the
//! invader stands where the fixture puts it for as many ticks as the test
//! needs. A fixture built on open ground would measure how far a unit walks
//! before the pass reads it.[^3]
//!
//! The tests drive the world step and the public verbs, because the step and
//! the verbs are what must invoke the rules.[^4]
//!
//! # References
//!
//! [^1]: ADR-0180, a site changes hands or the taker destroys it. `docs/adrs/draft/adr-0180-a-site-changes-hands-or-the-taker-destroys-it.md`
//! [^2]: ADR-0181, a faction that holds no site and no unit leaves the game. `docs/adrs/draft/adr-0181-a-faction-that-holds-no-site-and-no-unit-leaves-the-game.md`
//! [^3]: Testing rules, section 2a. `.agents/rules/testing.md`
//! [^4]: Testing rules, section 5. `.agents/rules/testing.md`

use cachette_core::event::{TAKE_KIND_CAPTURED, TAKE_KIND_RAZED};
use cachette_core::holding::{Holder, LeaseRules, ReachRules};
use cachette_core::site::CommodityId;
use cachette_core::upgrade::UpgradeCategory;
use cachette_core::{Axial, Entity, FactionId, Fix32, RazeError, World, WorldConfig};

/// The extent of the worlds below.
///
/// The extent is wide enough that the generator puts water in it, because
/// every fixture here needs a tile whose every neighbour refuses a unit.
const EXTENT: u32 = 192;

/// The commodity every fixture writes and reads.
const GRAIN: CommodityId = CommodityId(0);

fn world(seed: u64, factions: u16, reach: u32) -> World {
    let mut field = World::new(WorldConfig {
        width: EXTENT,
        height: EXTENT,
        seed,
        faction_count: factions,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    })
    .expect("the extent must describe a world");
    // The reach decides whether a taker keeps a captured city or burns it, so
    // each fixture states the reach it needs. The test never reads a reach
    // value, so this is a fixture choice and not a balance figure.
    field.set_reach_rules(ReachRules::new(reach, 1, reach));
    field
}

/// The reach that puts the rival city far outside the island.
///
/// The fixture places the two cities more than eight steps apart, so a reach
/// of two supplies neither from the other.
const REACH_APART: u32 = 2;

/// The reach that covers the whole fixture world.
///
/// The extent bounds every distance in the world, so a reach of the extent
/// puts every city of a faction within reach of every tile.
const REACH_TOGETHER: u32 = EXTENT;

/// Returns every address of a world, in tile index order.
fn addresses(field: &World) -> Vec<Axial> {
    let grid = field.grid();
    (0..grid.tile_count())
        .map(|index| Axial::new((index % grid.width()) as i32, (index / grid.width()) as i32))
        .collect()
}

/// Returns an island: an open tile whose every neighbour refuses a unit.
///
/// A unit on an island never moves, whatever the choice pass decides, so the
/// fixture holds a unit on one tile for as many ticks as it needs.
fn island(field: &World) -> Option<Axial> {
    addresses(field).into_iter().find(|address| {
        field.admits_a_unit(*address)
            && field
                .grid()
                .neighbours(*address)
                .iter()
                .all(|side| side.is_none_or(|next| !field.admits_a_unit(next)))
    })
}

/// Returns an open tile that is not the island and not beside it.
fn mainland(field: &World, away_from: Axial) -> Option<Axial> {
    addresses(field)
        .into_iter()
        .find(|address| field.admits_a_unit(*address) && address.distance(away_from) > 8)
}

/// What a fixture holds: an island city of faction zero with a finished
/// road, a store, and one resident who lives elsewhere.
struct Fixture {
    field: World,
    seat: Axial,
    site: Entity,
    resident: Entity,
    away: Axial,
    rival: Axial,
}

/// Builds the fixture, or returns `None` when the seed cannot hold it.
///
/// The founding goes through the run verb and not through the raw arena
/// verb, because the run verb is what records the seat of a faction and the
/// elimination pass reads the seat.[^1]
///
/// # References
///
/// [^1]: Testing rules, section 5. `.agents/rules/testing.md`
fn fixture(seed: u64) -> Option<Fixture> {
    fixture_with_reach(seed, REACH_APART)
}

/// Builds the fixture at a stated reach.
fn fixture_with_reach(seed: u64, reach: u32) -> Option<Fixture> {
    let mut field = world(seed, 2, reach);
    // The readers are off while the fixture builds. A world of two factions
    // in which only one has founded ends on domination at once, and the
    // fixture would then measure the reader rather than the capture.
    field.set_win_readers_enabled(false);
    let seat = island(&field)?;
    let away = mainland(&field, seat)?;
    let rival = mainland(&field, seat)
        .into_iter()
        .chain(addresses(&field))
        .find(|address| {
            field.admits_a_unit(*address)
                && address.distance(seat) > 8
                && address.distance(away) > 8
        })?;
    field.found_group_at(rival, 1, FactionId(1)).ok()?;
    let founding = field.found_group_at(seat, 1, FactionId(0)).ok()?;
    let site = founding.settlement();
    let resident = *founding.people().first()?;
    // The founder walks off the island. A unit of the owning faction on the
    // site tile is a garrison, and a garrison refuses every capture.
    field.place_soldier(resident, away).ok()?;

    // A finished road stands on the site tile. It is what proves that an
    // upgrade changes hands with the ground, so the fixture must reach the
    // finished state and not a part-built one.
    let builder = field.spawn_soldier(seat, FactionId(0)).ok()?;
    field
        .zone_project(FactionId(0), seat, UpgradeCategory::ROAD)
        .ok()?;
    field.order_build(builder, UpgradeCategory::ROAD).ok()?;
    for _ in 0..=cachette_core::DEFAULT_UPGRADE_TABLE.work_above(UpgradeCategory::ROAD, 0) {
        field.step(1).expect("the step must run");
    }
    if field.finished_upgrade(seat) != Some(UpgradeCategory::ROAD) {
        return None;
    }
    assert!(field.despawn_soldier(builder));
    if !field.soldiers().contains(resident) || field.soldiers().address(resident) == Some(seat) {
        return None;
    }
    field
        .set_settlement_store(site, GRAIN, Fix32::from_int(1))
        .expect("the commodity is in the set");
    Some(Fixture {
        field,
        seat,
        site,
        resident,
        away,
        rival,
    })
}

/// Returns the first seed that builds the fixture, or fails the test.
fn any_seed() -> u64 {
    for seed in 0..60u64 {
        if fixture(seed).is_some() {
            return seed;
        }
    }
    panic!("no seed below 60 gives an island with a finished road");
}

/// Builds the fixture on the first seed that gives one.
fn any_fixture() -> Fixture {
    fixture(any_seed()).expect("the seed builds the fixture")
}

#[test]
fn a_faction_that_stands_on_an_undefended_city_takes_it_whole() {
    // The taker holds a city that reaches the island, so the rule keeps the
    // city rather than burning it.
    let seed = any_seed();
    let Fixture {
        mut field,
        seat,
        site,
        resident,
        ..
    } = fixture_with_reach(seed, REACH_TOGETHER).expect("the seed builds the fixture");
    assert_eq!(field.settlement_faction(site), Some(FactionId(0)));

    let invader = field
        .spawn_soldier(seat, FactionId(1))
        .expect("the island admits a unit");
    field.step(2).expect("the step must run");
    assert!(field.check_invariants());

    // The control is the same fixture on the same seed with no invader. It
    // says what the store, the road and the resident do without a capture,
    // so the assertions below read the capture and not the tick.
    let mut control =
        fixture_with_reach(seed, REACH_TOGETHER).expect("the seed builds the fixture");
    control.field.step(2).expect("the step must run");

    assert_eq!(
        field.settlement_faction(site),
        Some(FactionId(1)),
        "the city did not change hands"
    );
    assert_eq!(
        control.field.settlement_faction(control.site),
        Some(FactionId(0)),
        "the control changed hands with no invader on the tile"
    );
    assert_eq!(
        field.soldiers().address(invader),
        Some(seat),
        "the island did not hold the invader"
    );
    // The upgrade changes hands with the ground. Nothing removes it and
    // nothing rebuilds it.
    assert_eq!(
        field.finished_upgrade(seat),
        Some(UpgradeCategory::ROAD),
        "the capture destroyed the road"
    );
    assert_eq!(
        field.settlement_store(site, GRAIN),
        control.field.settlement_store(control.site, GRAIN),
        "the capture changed the store"
    );
    // The residents live, and they belong to the taker.
    assert!(
        field.soldiers().contains(resident),
        "the capture killed a resident"
    );
    assert_eq!(
        field.soldier_faction(resident),
        Some(FactionId(1)),
        "a resident kept the faction that lost the city"
    );
    assert_eq!(
        field.site_residents(site),
        control.field.site_residents(control.site),
        "the taker did not receive every resident"
    );
    // The ground follows on the same tick, because the capture runs before
    // the spread.
    assert_eq!(
        field.tile_holder(seat).and_then(Holder::faction),
        Some(FactionId(1)),
        "the ground did not follow the city"
    );
    let taken = field.taken_log();
    assert_eq!(taken.len(), 1, "the capture wrote no event, or wrote two");
    assert_eq!(taken[0].kind, TAKE_KIND_CAPTURED);
    assert_eq!(taken[0].from, FactionId(0));
    assert_eq!(taken[0].to, FactionId(1));
    assert_eq!(taken[0].site, site.to_bits());
}

#[test]
fn a_garrison_of_one_refuses_the_capture() {
    // **This is the defect put back.** Without the defence test the capture
    // pass would take the city from the occupancy alone, and the occupancy
    // names the faction with the most units. Two invaders against one
    // defender would then take a defended city.
    let Fixture {
        mut field,
        seat,
        site,
        ..
    } = fixture_with_reach(any_seed(), REACH_TOGETHER).expect("the seed builds the fixture");
    field
        .spawn_soldier(seat, FactionId(0))
        .expect("the island admits a unit");
    for _ in 0..2 {
        field
            .spawn_soldier(seat, FactionId(1))
            .expect("the island admits a unit");
    }
    field.step(2).expect("the step must run");
    assert!(field.check_invariants());
    assert_eq!(
        field.settlement_faction(site),
        Some(FactionId(0)),
        "a defended city changed hands"
    );
    assert!(
        field.taken_log().is_empty(),
        "a defended city wrote an event"
    );
}

#[test]
fn a_captured_city_beyond_the_takers_reach_is_burned() {
    // **This is the half of the rule that the keeping test cannot show.** The
    // fixture puts the two cities more than eight steps apart and gives every
    // city a reach of two, so no city of the taker supplies the island. The
    // step must burn it rather than keep it.
    let Fixture {
        mut field,
        seat,
        site,
        resident,
        rival,
        ..
    } = fixture_with_reach(any_seed(), REACH_APART).expect("the seed builds the fixture");
    let home = field
        .settlement_on(rival)
        .expect("the rival founded a city");
    let held = field
        .settlement_store(home, GRAIN)
        .expect("the site is live");
    assert!(
        rival.distance(seat) > REACH_APART,
        "the fixture put the two cities within reach of each other"
    );

    field
        .spawn_soldier(seat, FactionId(1))
        .expect("the island admits a unit");
    field.step(2).expect("the step must run");

    assert!(field.check_invariants());
    assert_eq!(
        field.settlement_on(seat),
        None,
        "the step kept a city the taker cannot supply"
    );
    assert_eq!(
        field.finished_upgrade(seat),
        None,
        "the burn left the road standing"
    );
    assert!(
        !field.soldiers().contains(resident),
        "the burn left a resident alive"
    );
    assert!(
        field
            .settlement_store(home, GRAIN)
            .is_some_and(|now| now.0 >= held.0),
        "the burn paid the taker nothing"
    );
    let taken = field.taken_log();
    assert_eq!(taken.len(), 1, "the burn wrote no event, or wrote two");
    assert_eq!(taken[0].kind, TAKE_KIND_RAZED);
    assert_eq!(taken[0].site, site.to_bits());
    assert_eq!(taken[0].to, FactionId(1));
}

#[test]
fn a_taker_that_holds_no_city_keeps_what_it_takes() {
    // **The boundary of the rule, and it is reachable.** A faction that lost
    // every city keeps a field army, and that army can take a capital. The
    // rule asks which city of the taker supplies the captured one, and a
    // taker with none has not failed to reach it. Without this the last army
    // of a beaten faction could never take a city, and a faction that lost
    // every city could never return.
    let Fixture {
        mut field,
        seat,
        site,
        rival,
        ..
    } = fixture_with_reach(any_seed(), REACH_APART).expect("the seed builds the fixture");
    let home = field
        .settlement_on(rival)
        .expect("the rival founded a city");
    // The taker loses its only city, and keeps its army.
    assert!(field.destroy_settlement(home));
    field
        .spawn_soldier(seat, FactionId(1))
        .expect("the island admits a unit");
    field.step(2).expect("the step must run");

    assert!(field.check_invariants());
    assert_eq!(
        field.settlement_faction(site),
        Some(FactionId(1)),
        "a taker with no city of its own burned the city it took"
    );
    assert_eq!(
        field.finished_upgrade(seat),
        Some(UpgradeCategory::ROAD),
        "the capture destroyed the road"
    );
    let taken = field.taken_log();
    assert_eq!(taken.len(), 1, "the capture wrote no event, or wrote two");
    assert_eq!(taken[0].kind, TAKE_KIND_CAPTURED);
}

#[test]
fn a_razed_city_is_gone_and_its_upgrades_with_it() {
    let Fixture {
        mut field,
        seat,
        site,
        resident,
        rival,
        ..
    } = any_fixture();
    let home = field
        .settlement_on(rival)
        .expect("the rival founded a city");
    let carried = field
        .settlement_store(site, GRAIN)
        .expect("the site is live");
    let held = field
        .settlement_store(home, GRAIN)
        .expect("the site is live");
    // The raze runs on the tick the invader lands, before the step reads the
    // tile, because the step would otherwise capture the city first.
    field
        .spawn_soldier(seat, FactionId(1))
        .expect("the island admits a unit");

    field
        .raze_site(site, FactionId(1))
        .expect("the razer stands on an undefended site");
    let taken = field.taken_log().to_vec();
    assert_eq!(taken.len(), 1, "the raze wrote no event, or wrote two");
    assert_eq!(taken[0].kind, TAKE_KIND_RAZED);
    assert_eq!(taken[0].from, FactionId(0));
    assert_eq!(taken[0].to, FactionId(1));
    field.step(2).expect("the step must run");

    assert!(field.check_invariants());
    assert_eq!(
        field.settlement_on(seat),
        None,
        "a razed city still stands on its tile"
    );
    assert_eq!(
        field.settlement_faction(site),
        None,
        "the identity of a razed city still resolves"
    );
    assert_eq!(
        field.finished_upgrade(seat),
        None,
        "the raze left the road standing"
    );
    assert!(
        !field.soldiers().contains(resident),
        "the raze left a resident alive"
    );
    // The ground the razed city held is gone with it. No city of the loser
    // reaches the tile any more.
    assert_ne!(
        field.tile_holder(seat).and_then(Holder::faction),
        Some(FactionId(0)),
        "the razed city kept its ground"
    );
    // The plunder moved. Nothing was made and nothing was lost.
    assert!(
        field
            .settlement_store(home, GRAIN)
            .is_some_and(|now| now.0 >= held.0),
        "the raze took from the razer instead of paying it"
    );
    assert!(carried.0 > 0, "the fixture razed an empty store");
}

#[test]
fn a_raze_refuses_a_defended_site_and_a_site_nobody_stands_on() {
    let Fixture {
        mut field,
        seat,
        site,
        ..
    } = any_fixture();
    assert_eq!(
        field.raze_site(site, FactionId(1)),
        Err(RazeError::NotOccupied),
        "a raze from nowhere was allowed"
    );
    field
        .spawn_soldier(seat, FactionId(0))
        .expect("the island admits a unit");
    field
        .spawn_soldier(seat, FactionId(1))
        .expect("the island admits a unit");
    assert_eq!(
        field.raze_site(site, FactionId(1)),
        Err(RazeError::Defended),
        "a raze of a defended site was allowed"
    );
    assert_eq!(
        field.raze_site(site, FactionId(0)),
        Err(RazeError::OwnSite),
        "a faction razed its own site"
    );
    field.step(2).expect("the step must run");
    assert!(field.check_invariants());
    assert_eq!(
        field.settlement_faction(site),
        Some(FactionId(0)),
        "a refused raze took the city anyway"
    );
}

#[test]
fn no_unit_is_stranded_when_its_home_is_taken_or_razed() {
    // The hazard has two shapes.
    //
    // **A home names a settlement slot, and a razed slot returns to the
    // arena.** A unit that kept the name would draw from the settlement
    // founded next in that slot. The loss clears the home of every unit that
    // names it, and this test drives that path through the raze verb.
    //
    // **A destination is a set of tiles, and a tile does not stop
    // existing.** A raze therefore orphans no destination. What the release
    // must still answer is a unit that arrives, because a sent unit reads no
    // option row and one that stays sent neither gathers nor delivers.[^1]
    //
    // # References
    //
    // [^1]: Findings register, FND-576. `docs/FINDINGS.md`
    let Fixture {
        mut field,
        seat,
        site,
        resident,
        away,
        ..
    } = any_fixture();
    let traveller = field
        .spawn_soldier(away, FactionId(0))
        .expect("the ground admits a unit");
    field
        .send_units_to(&[traveller], &[away], 1)
        .expect("the verb takes a live unit and an address in the world");
    assert_eq!(field.sent_to(traveller), Some(Some(1)));

    field
        .spawn_soldier(seat, FactionId(1))
        .expect("the island admits a unit");
    field
        .raze_site(site, FactionId(1))
        .expect("the razer stands on an undefended site");

    assert!(
        !field.soldiers().contains(resident),
        "the raze left a resident alive"
    );
    assert!(
        field.soldiers().contains(traveller),
        "the raze killed a unit that was not a resident"
    );
    // No live unit names the slot the raze freed. The reader answers the
    // settlement the unit draws from, and it answers nothing here.
    for unit in field.soldiers().iter().collect::<Vec<_>>() {
        assert_ne!(
            field.dwelling_of(unit),
            Some(Some(site)),
            "a unit still names a home site that is gone"
        );
    }
    for _ in 0..4 {
        field.step(2).expect("the step must run");
    }
    assert_eq!(
        field.sent_to(traveller),
        Some(None),
        "a unit that reached its destination was never released"
    );
    assert!(field.check_invariants());
}

#[test]
fn a_faction_with_no_site_and_no_unit_leaves_the_game_and_releases_its_ground() {
    // **The extreme is ground held by a lease and not by a city.** A faction
    // that loses its last city loses the ground its city reached on the next
    // spread, whatever this pass does. A lease at the claim threshold
    // outranks every city, and nothing raises or lowers it once the faction
    // that holds it is gone. That is the ground this pass must release, and
    // a fixture that held ground only through a city would never supply it.
    let Fixture {
        mut field,
        seat,
        site,
        away,
        ..
    } = any_fixture();
    let rules = LeaseRules::new(8, 4, 1, 64, 0, 64, 8);
    field.set_lease_rules(rules);
    // One unit of the losing faction stands away from its city until the
    // lease of that tile reaches the claim threshold.
    let holder = field
        .spawn_soldier(away, FactionId(0))
        .expect("the ground admits a unit");
    for _ in 0..4 {
        field.step(2).expect("the step must run");
    }
    let leased = field.soldiers().address(holder).expect("the unit lives");
    assert_eq!(
        field.tile_lease(leased).map(|(named, _)| named.faction()),
        Some(Some(FactionId(0))),
        "the fixture built no lease for the losing faction"
    );
    assert_eq!(
        field.tile_holder(leased).and_then(Holder::faction),
        Some(FactionId(0)),
        "the lease did not reach the claim threshold"
    );
    assert!(
        field.holding_of(FactionId(0)) > 0,
        "the fixture must start with the losing faction holding ground"
    );
    assert!(!field.is_eliminated(FactionId(0)));

    field
        .spawn_soldier(seat, FactionId(1))
        .expect("the island admits a unit");
    field
        .raze_site(site, FactionId(1))
        .expect("the razer stands on an undefended site");
    // Every remaining unit of the losing faction goes. The elimination reads
    // the site count and the population, and this fixture drives both to
    // zero.
    for unit in field.soldiers().iter().collect::<Vec<_>>() {
        if field.soldier_faction(unit) == Some(FactionId(0)) {
            assert!(field.despawn_soldier(unit));
        }
    }

    field.step(2).expect("the step must run");
    assert!(field.check_invariants());
    assert!(
        field.is_eliminated(FactionId(0)),
        "a faction with no site and no unit stayed in the game"
    );
    assert_eq!(
        field.holding_of(FactionId(0)),
        0,
        "an eliminated faction kept its ground"
    );
    assert_eq!(
        field.tile_lease(leased).map(|(named, _)| named.is_nobody()),
        Some(true),
        "the elimination left a lease that nothing can raise or lower"
    );
    let gone = field.eliminated_log();
    assert_eq!(gone.len(), 1, "the pass wrote no event, or wrote two");
    assert_eq!(gone[0].faction, FactionId(0));
    assert!(gone[0].released > 0, "the pass released no tile");

    // The elimination is recorded once. A second step must write nothing.
    field.step(2).expect("the step must run");
    assert!(
        field.eliminated_log().is_empty(),
        "a faction left the game twice"
    );
    assert_eq!(field.holding_of(FactionId(0)), 0);
}

#[test]
fn an_eliminated_faction_wins_nothing() {
    // **This is the defect put back.** The territory reader compares held
    // ground at the tick limit. Before the elimination pass a faction with
    // nothing alive kept its tiles for ever, so it could win the game on
    // ground it could not defend. Every reader must refuse it.
    let Fixture {
        mut field,
        seat,
        site,
        ..
    } = any_fixture();
    field
        .spawn_soldier(seat, FactionId(1))
        .expect("the island admits a unit");
    field
        .raze_site(site, FactionId(1))
        .expect("the razer stands on an undefended site");
    for unit in field.soldiers().iter().collect::<Vec<_>>() {
        if field.soldier_faction(unit) == Some(FactionId(0)) {
            assert!(field.despawn_soldier(unit));
        }
    }
    field.step(2).expect("the step must run");
    assert!(field.is_eliminated(FactionId(0)));

    // The readers come back on, and the record names the survivor.
    field.set_win_readers_enabled(true);
    let mut end = field.game_end();
    for _ in 0..8 {
        if end.is_set() {
            break;
        }
        field.step(2).expect("the step must run");
        end = field.game_end();
    }
    assert!(
        end.is_set(),
        "no reader fired after a faction left the game"
    );
    assert_eq!(
        end.winner,
        FactionId(1),
        "an eliminated faction won the game"
    );
}
