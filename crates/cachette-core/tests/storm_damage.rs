//! What a storm takes from the world it passes over.
//!
//! A storm puts a pressure deficit on the cells it reaches. This file checks
//! what that deficit costs: the food a tile carries, the people who stand in
//! the open, and the condition of what a faction built.
//!
//! **The fixture reaches both sides of every rule.** A world under one small
//! storm holds tiles the storm covers and tiles it does not, so an assertion
//! that only read the covered tiles would pass against a pass that harmed
//! every tile of the world. A fixture built from the world that looks right
//! would not reach that case.[^1]
//!
//! **The draw tests move one field of the key at a time.** A determinism test
//! cannot tell a correct draw from a consistently wrong one, so each field of
//! the key needs a test that changes it and watches the answer change.[^2]
//!
//! # References
//!
//! [^1]: Testing rules, section 2a. `.agents/rules/testing.md`
//! [^2]: Testing rules, section 2. `.agents/rules/testing.md`

use cachette_core::storm;
use cachette_core::{
    Axial, CycloneSetting, Entity, FactionId, Latitudes, ResourceKind, TileKind, UpgradeCategory,
    UpgradeRow, WeatherScale, World, WorldConfig, DEFAULT_UPGRADE_TABLE,
};

/// The extent of the fixture world.
///
/// The world is wide enough to hold weather cells that no storm reaches, so
/// every test can read a control tile beside the tiles the storm covers.
const EDGE: u32 = 64;

/// The seed of the fixture world.
const SEED: u64 = 11;

/// Builds the fixture world.
///
/// **The world spans the globe, and it runs one weather cell for each tile.**
/// Two properties of the fixture need both. A test reads a world that carries
/// no storm of its own, and the span decides the temperature gradient that
/// admits one: the default span puts that gradient at its floor, so the field
/// raises fronts everywhere. A storm also drifts about one cell of its own
/// lattice in a tick, so a coarse lattice carries the eye off the world, or
/// onto water, before the test reads it.[^1]
///
/// # References
///
/// [^1]: Testing rules, section 2a. `.agents/rules/testing.md`
fn world() -> World {
    World::with_weather_scale(
        WorldConfig {
            width: EDGE,
            height: EDGE,
            seed: SEED,
            faction_count: 2,
            unit_capacity: 512,
            latitude_centre: Latitudes::PLANET.centre(),
            latitude_span: Latitudes::PLANET.span(),
        },
        WeatherScale::PER_TILE,
    )
    .expect("the extent must describe a world")
}

/// Returns the passable addresses of a world, in tile index order.
fn dry_ground(world: &World) -> Vec<Axial> {
    let mut dry = Vec::new();
    for row in 0..world.grid().height() {
        for column in 0..world.grid().width() {
            let here = Axial::new(column as i32, row as i32);
            if world.tile_kind(here).is_some_and(TileKind::is_passable) {
                dry.push(here);
            }
        }
    }
    dry
}

/// Raises one small storm over the fixture world and steps it once.
///
/// **The storm reaches part of the world and not the whole of it.** The small
/// setting stands on one cell, and the world holds many, so the caller reads a
/// covered set and a clear set that are both non-empty.
///
/// The eye drifts on the tick, so the caller reads which tiles the storm
/// covers rather than assuming the tile it was raised over.
fn a_world_under_one_storm() -> (World, Vec<Axial>, Vec<Axial>) {
    let mut field = world();
    field.step(1).expect("the step must run");
    let dry = dry_ground(&field);
    // **The eye starts near the middle of the world.** A storm raised on the
    // border drifts off the world in one tick, and the fixture then reaches no
    // tile under a storm at all.
    let middle = Axial::new((EDGE / 2) as i32, (EDGE / 2) as i32);
    let eye = *dry
        .iter()
        .min_by_key(|here| (here.q - middle.q).abs() + (here.r - middle.r).abs())
        .expect("the world holds passable ground");
    field
        .raise_cyclone(eye, CycloneSetting::SEVERE.with_reach(8))
        .expect("the place lies inside the world");
    field.step(1).expect("the step must run");
    let (under, clear): (Vec<Axial>, Vec<Axial>) = dry
        .into_iter()
        .partition(|here| field.tile_under_a_storm(*here) == Some(true));
    assert!(
        !under.is_empty(),
        "the fixture must reach a tile the storm carries"
    );
    assert!(
        !clear.is_empty(),
        "the fixture must reach a tile that no storm carries"
    );
    (field, under, clear)
}

/// Returns the food that a set of tiles carries in total.
fn food_over(field: &World, places: &[Axial]) -> i64 {
    places
        .iter()
        .filter_map(|here| field.tile_stock(*here, ResourceKind::Food))
        .map(|amount| i64::from(amount.0))
        .sum()
}

#[test]
fn a_storm_flattens_the_food_under_it_and_leaves_the_rest_alone() {
    let (mut field, under, clear) = a_world_under_one_storm();
    let before_under = food_over(&field, &under);
    let before_clear = food_over(&field, &clear);
    assert!(
        before_under > 0,
        "the fixture must reach ground that carries food under the storm"
    );
    field.step(1).expect("the step must run");
    let after_under = food_over(&field, &under);
    let after_clear = food_over(&field, &clear);
    assert!(
        after_under < before_under,
        "a storm must flatten the food the tiles under it carry: {before_under} stood before and {after_under} stands now"
    );
    assert_eq!(
        after_clear, before_clear,
        "a storm must leave the food of a tile it does not reach alone"
    );
}

#[test]
fn the_world_still_balances_after_a_storm_flattens_food() {
    // **Food a storm flattens leaves the world.** The ledger records what left
    // a tile, and the conservation check balances that against what units
    // hold, what a delivery moved and what left the world. A pass that wrote
    // the take and named no destination breaks the account of the world, and
    // this is the check that catches it.
    let (mut field, under, _) = a_world_under_one_storm();
    assert!(
        food_over(&field, &under) > 0,
        "the fixture must reach ground that carries food under the storm"
    );
    for _ in 0..24 {
        field.step(1).expect("the step must run");
        assert!(
            field.check_invariants(),
            "the world stopped balancing while a storm flattened food"
        );
    }
}

#[test]
fn a_world_with_no_storm_loses_no_food() {
    let mut field = world();
    field.step(1).expect("the step must run");
    let ground = dry_ground(&field);
    let before = food_over(&field, &ground);
    field.step(1).expect("the step must run");
    assert_eq!(
        food_over(&field, &ground),
        before,
        "a world that carries no storm must lose no food to one"
    );
}

/// Fills a set of tiles with soldiers of one faction, and returns them.
fn garrison(field: &mut World, places: &[Axial], most: usize) -> Vec<u64> {
    let mut placed = Vec::new();
    for here in places.iter().take(most) {
        match field.spawn_soldier(*here, FactionId(0)) {
            Ok(unit) => placed.push(unit.to_bits()),
            Err(_) => continue,
        }
    }
    placed
}

/// Puts as many soldiers on one tile as the tile admits, and returns them.
fn fill_tile(field: &mut World, place: Axial) -> Vec<u64> {
    let mut placed = Vec::new();
    while let Ok(unit) = field.spawn_soldier(place, FactionId(0)) {
        placed.push(unit.to_bits());
        if placed.len() > 256 {
            break;
        }
    }
    placed
}

#[test]
fn a_storm_takes_units_caught_in_the_open() {
    let (mut field, under, _) = a_world_under_one_storm();
    let placed = garrison(&mut field, &under, 256);
    assert!(
        placed.len() > 64,
        "the fixture must put enough people in the storm for a bounded chance to fire: {} stand",
        placed.len()
    );
    let mut lost = 0usize;
    for _ in 0..12 {
        field.step(1).expect("the step must run");
        lost += field.units_lost_to_storms().len();
    }
    assert!(
        lost > 0,
        "a storm must take some of the people caught in the open"
    );
    let living = placed
        .iter()
        .filter(|identity| field.resolve_soldier(**identity).is_ok())
        .count();
    assert_eq!(
        living + lost,
        placed.len(),
        "every unit the log names must be gone from the world, and no other"
    );
}

#[test]
fn a_world_with_no_storm_takes_nobody() {
    let mut field = world();
    field.step(1).expect("the step must run");
    let ground = dry_ground(&field);
    let placed = garrison(&mut field, &ground, 256);
    assert!(!placed.is_empty(), "the fixture must place people");
    for _ in 0..12 {
        field.step(1).expect("the step must run");
        assert!(
            field.units_lost_to_storms().is_empty(),
            "a world that carries no storm must take nobody"
        );
    }
}

#[test]
fn the_ground_under_a_storm_gets_wetter_than_the_ground_beside_it() {
    let (mut field, under, clear) = a_world_under_one_storm();
    let eye = *under.first().expect("the storm covers a tile");
    let away = *clear.last().expect("a tile stands outside the storm");
    let wet_before = field
        .ground_water_at(eye)
        .expect("the address is inside the world");
    let dry_before = field
        .ground_water_at(away)
        .expect("the address is inside the world");
    for _ in 0..8 {
        field.step(1).expect("the step must run");
    }
    let wet_after = field
        .ground_water_at(eye)
        .expect("the address is inside the world");
    let dry_after = field
        .ground_water_at(away)
        .expect("the address is inside the world");
    // **The storm creates no water. It makes the air pour out what it holds.**
    // So the ground under the eye must gain more than the ground beside it,
    // and the test compares the two rises rather than reading one figure.
    assert!(
        wet_after - wet_before > dry_after - dry_before,
        "the ground under a storm must rise further than the ground beside it: the eye moved by {} and the control moved by {}",
        wet_after - wet_before,
        dry_after - dry_before
    );
}

#[test]
fn the_wear_of_a_storm_follows_the_deficit_and_reaches_nothing_outside_it() {
    assert_eq!(
        storm::wear_at(0),
        0,
        "a cell that no storm reaches must lose no condition"
    );
    let shallow = storm::wear_at(8);
    let deep = storm::wear_at(i32::try_from(storm::DEPTH_WHOLE).expect("the ceiling fits"));
    assert!(
        shallow > 0 && shallow < deep,
        "a shallow edge must take less than a deep eye: {shallow} against {deep}"
    );
    assert_eq!(
        deep,
        storm::WEAR_AT_FULL_DEPTH,
        "the deepest eye must take the rate the module states"
    );
    // **Two storms over one cell sum their deficits, and the harm still
    // clamps.** A cell under two eyes must not lose twice the rate.
    assert_eq!(
        storm::wear_at(i32::try_from(storm::DEPTH_WHOLE * 2).expect("twice the ceiling fits")),
        deep,
        "the harm of a cell under two eyes must be the harm of one eye"
    );
}

#[test]
fn a_storm_never_takes_more_food_than_a_tile_carries() {
    for carried in [0i64, 1, 2, 7, 8, 9, 1000] {
        let taken = storm::food_lost_at(
            i32::try_from(storm::DEPTH_WHOLE).expect("the ceiling fits"),
            carried,
        );
        assert!(
            (0..=carried).contains(&taken),
            "a tile carrying {carried} must lose between nothing and all of it, and it lost {taken}"
        );
    }
    assert_eq!(
        storm::food_lost_at(0, 1000),
        0,
        "a tile that no storm reaches must lose no food"
    );
}

/// Counts the units of a set that one tick of a storm takes.
fn taken_of(frame: u64, identities: &[u64], deficit: i32) -> usize {
    identities
        .iter()
        .filter(|unit| storm::takes_unit(SEED, frame, **unit, deficit))
        .count()
}

/// Returns the answers of one frame, for a set of identities.
fn answers_of(frame: u64, identities: &[u64], deficit: i32) -> Vec<bool> {
    identities
        .iter()
        .map(|unit| storm::takes_unit(SEED, frame, *unit, deficit))
        .collect()
}

/// The whole identity of a unit, from its slot and its generation.
///
/// The identity puts the generation above the slot, so a slot reused at a
/// later generation names a different unit.
fn identity(slot: u32, generation: u32) -> u64 {
    (u64::from(generation) << 32) | u64::from(slot)
}

#[test]
fn the_loss_draw_follows_the_frame() {
    let deficit = i32::try_from(storm::DEPTH_WHOLE).expect("the ceiling fits");
    let people: Vec<u64> = (0..512).map(|slot| identity(slot, 1)).collect();
    let first = answers_of(1, &people, deficit);
    let second = answers_of(2, &people, deficit);
    assert_ne!(
        first, second,
        "a unit that stands in one storm for two ticks must draw twice, and the two draws must not agree everywhere"
    );
}

#[test]
fn the_loss_draw_follows_the_generation_and_not_the_slot() {
    let deficit = i32::try_from(storm::DEPTH_WHOLE).expect("the ceiling fits");
    let dead: Vec<u64> = (0..512).map(|slot| identity(slot, 1)).collect();
    let born: Vec<u64> = (0..512).map(|slot| identity(slot, 2)).collect();
    let before = answers_of(1, &dead, deficit);
    let after = answers_of(1, &born, deficit);
    assert_ne!(
        before, after,
        "a unit spawned into the slot of a unit the storm took must draw its own answer"
    );
}

#[test]
fn the_loss_draw_follows_the_deficit() {
    let people: Vec<u64> = (0..2048).map(|slot| identity(slot, 1)).collect();
    let deep = taken_of(
        1,
        &people,
        i32::try_from(storm::DEPTH_WHOLE).expect("the ceiling fits"),
    );
    let shallow = taken_of(1, &people, 8);
    assert_eq!(
        taken_of(1, &people, 0),
        0,
        "a cell that no storm reaches must take nobody"
    );
    assert!(
        shallow < deep,
        "a shallow edge must take fewer people than a deep eye: {shallow} against {deep}"
    );
}

#[test]
fn shelter_is_what_answers_a_storm() {
    // The chance is stated out of a whole, and a unit under shelter draws
    // nothing at all. The engine skips the draw for a sheltered tile, so the
    // rate below is what a unit in the open faces and the sheltered one faces
    // none of it.
    let deficit = i32::try_from(storm::DEPTH_WHOLE).expect("the ceiling fits");
    assert!(
        storm::loss_chance_at(deficit) > 0,
        "a unit in the open under the deepest eye must face a real chance"
    );
    assert!(
        storm::loss_chance_at(deficit) < storm::CHANCE_WHOLE,
        "a unit in the open must not fall for certain"
    );
}

/// A world in which one faction has finished one road, with a bare tile
/// beside it.
///
/// **The fixture reaches a built tile and a bare tile in one weather cell.** A
/// fixture with only a built tile could not show that shelter answers a storm,
/// and a fixture with only bare ground could not show a site falling at all.
struct Built {
    field: World,
    /// The tile that carries the finished road.
    road: Axial,
    /// A bare tile beside it.
    bare: Axial,
    /// The unit that built the road. It stands on the road.
    builder: Entity,
}

/// Builds the road fixture, or returns `None` when the seed gives no seat
/// with two open tiles beside it.
fn a_world_with_one_road(seed: u64) -> Option<Built> {
    // **The world spans the globe for the reason the fixture world does.**
    // The default span puts the gradient that admits a storm at its floor, so
    // the field raises fronts of its own over the road, and the road then
    // falls to weather the test did not place.
    let mut field = World::new(WorldConfig {
        width: EDGE,
        height: EDGE,
        seed,
        faction_count: 2,
        unit_capacity: 512,
        latitude_centre: Latitudes::PLANET.centre(),
        latitude_span: Latitudes::PLANET.span(),
    })
    .expect("the extent must describe a world");
    // **The road admits a crowd, because the test needs one.** A bare tile of
    // this world admits one or two units, and the chance a storm takes one is
    // a few parts in a thousand, so a fixture at the ordinary capacity reaches
    // no draw at all and measures itself.[^1]
    //
    // [^1]: Testing rules, section 2a. `.agents/rules/testing.md`
    let wide = UpgradeRow {
        capacity_change: 64,
        ..DEFAULT_UPGRADE_TABLE
            .row(UpgradeCategory::ROAD, 1)
            .expect("the default table holds the first road level")
    };
    field
        .define_upgrade_row(UpgradeCategory::ROAD.0, 1, wide)
        .expect("the category and the level are in the table");
    field.step(1).expect("the step must run");
    let seat = dry_ground(&field).into_iter().find(|here| {
        field.admits_a_unit(*here)
            && field.admits_a_unit(Axial::new(here.q + 1, here.r))
            && field.admits_a_unit(Axial::new(here.q + 2, here.r))
    })?;
    field.found_settlement(seat, FactionId(0)).ok()?;
    field.step(1).expect("the step must run");
    let road = Axial::new(seat.q + 1, seat.r);
    let bare = Axial::new(seat.q + 2, seat.r);
    field
        .zone_project(FactionId(0), road, UpgradeCategory::ROAD)
        .ok()?;
    let builder = field.spawn_soldier(road, FactionId(0)).ok()?;
    field.order_build(builder, UpgradeCategory::ROAD).ok()?;
    for _ in 0..=DEFAULT_UPGRADE_TABLE.work_above(UpgradeCategory::ROAD, 0) {
        field.step(1).expect("the step must run");
    }
    if field.finished_upgrade(road) != Some(UpgradeCategory::ROAD) {
        return None;
    }
    if field.upgrade_at(bare).is_some() {
        return None;
    }
    Some(Built {
        field,
        road,
        bare,
        builder,
    })
}

/// Returns the first seed that builds the road fixture, or fails the test.
fn a_seed_that_builds_a_road() -> u64 {
    for seed in 0..80u64 {
        if a_world_with_one_road(seed).is_some() {
            return seed;
        }
    }
    panic!("no seed under eighty gives a seat with two open tiles beside it");
}

/// Holds one storm over a place for one tick, and steps the world.
///
/// The eye drifts and a storm dies of age, so this raises a new one whenever
/// the place stands clear. It calls the public verb that a god calls, so the
/// test drives the engine and never the field.
fn step_under_a_storm(field: &mut World, place: Axial) {
    if field.storm_depth_at(place) == Some(0) {
        let _ = field.raise_cyclone(place, CycloneSetting::SEVERE);
    }
    field.step(1).expect("the step must run");
}

/// Returns the tile index of an address of the fixture world.
fn tile_of(field: &World, address: Axial) -> cachette_core::TileIdx {
    field
        .grid()
        .index_of(address)
        .expect("the address is inside the world")
}

#[test]
fn a_storm_destroys_a_site_only_by_driving_its_condition_to_nothing() {
    let mut built =
        a_world_with_one_road(a_seed_that_builds_a_road()).expect("the seed builds the fixture");
    // The builder leaves, so nothing repairs the road. A worker that stayed
    // would mend what the storm takes, and that is the other test.
    assert!(built.field.despawn_soldier(built.builder));
    let full = built
        .field
        .upgrade_condition(built.road)
        .expect("the road stands");
    let road_tile = tile_of(&built.field, built.road);

    step_under_a_storm(&mut built.field, built.road);
    step_under_a_storm(&mut built.field, built.road);
    let after_two = built.field.upgrade_condition(built.road);
    assert!(
        after_two.is_some(),
        "a storm destroyed a site at full condition, so a threshold and not the wear ended it"
    );
    assert!(
        after_two.expect("the road stands") < full,
        "a storm stood over a site and took no condition from it"
    );

    let mut collapsed = None;
    for _ in 0..400 {
        step_under_a_storm(&mut built.field, built.road);
        if let Some(entry) = built
            .field
            .collapsed_log()
            .iter()
            .find(|entry| entry.tile == road_tile)
        {
            collapsed = Some(*entry);
            break;
        }
    }
    let entry = collapsed.expect("the road never fell to a storm that stood over it");
    assert_eq!(
        entry.cause,
        cachette_core::event::WEAR_CAUSE_STORM,
        "the storm took the last of the condition and the log named another cause"
    );
    assert_eq!(
        built.field.upgrade_at(built.road),
        None,
        "the log recorded a collapse and the site still stands"
    );
}

#[test]
fn a_site_a_worker_keeps_in_repair_outlives_one_a_storm_finds_neglected() {
    let seed = a_seed_that_builds_a_road();

    let mut neglected = a_world_with_one_road(seed).expect("the seed builds the fixture");
    assert!(neglected.field.despawn_soldier(neglected.builder));
    let mut fell_at = None;
    for tick in 1..=400u32 {
        step_under_a_storm(&mut neglected.field, neglected.road);
        if neglected.field.upgrade_at(neglected.road).is_none() {
            fell_at = Some(tick);
            break;
        }
    }
    let fell_at = fell_at.expect("the neglected road never fell to a storm");

    let mut kept = a_world_with_one_road(seed).expect("the seed builds the fixture");
    // The worker stays and holds its build order, so it mends what the storm
    // takes. The order is the public verb, and the repair is the engine's own
    // loop.
    assert!(kept
        .field
        .order_build(kept.builder, UpgradeCategory::ROAD)
        .is_ok());
    for _ in 1..=fell_at {
        step_under_a_storm(&mut kept.field, kept.road);
    }
    assert!(
        kept.field.upgrade_at(kept.road).is_some(),
        "a site under repair fell as fast as a neglected one, so the repair buys nothing"
    );
}

#[test]
fn a_unit_under_a_finished_upgrade_is_not_caught_in_the_open() {
    let mut built =
        a_world_with_one_road(a_seed_that_builds_a_road()).expect("the seed builds the fixture");
    let seed = built.field.config().seed;
    // The builder holds its order and mends the road, so the shelter stands
    // for the whole run. A road that fell partway through would end the case
    // the test is about.
    assert!(built
        .field
        .order_build(built.builder, UpgradeCategory::ROAD)
        .is_ok());
    let mut sheltered = fill_tile(&mut built.field, built.road);
    sheltered.push(built.builder.to_bits());
    let exposed = fill_tile(&mut built.field, built.bare);
    assert!(
        sheltered.len() > 32 && !exposed.is_empty(),
        "the fixture must put a crowd under the shelter and people in the open: {} sheltered and {} exposed",
        sheltered.len(),
        exposed.len()
    );
    let road_tile = tile_of(&built.field, built.road);

    // **The run is long because the sheltered set is small.** A tile admits
    // two units, and the chance a storm takes one is a few parts in a
    // thousand, so a short run reaches no draw at all and the guard below
    // would then be the thing that failed.
    let mut would_have_fallen = 0usize;
    let mut ticks_under_shelter = 0usize;
    for _ in 0..1200 {
        // The order is re-issued because the engine clears it when the site
        // reaches full condition, and a worker with no order mends nothing.
        let _ = built
            .field
            .order_build(built.builder, UpgradeCategory::ROAD);
        step_under_a_storm(&mut built.field, built.road);
        if built.field.upgrade_at(built.road).is_none() {
            break;
        }
        ticks_under_shelter += 1;
        let deficit = built
            .field
            .storm_depth_at(built.road)
            .expect("the address is inside the world");
        let frame = built.field.tick().0;
        // **The fixture must reach the case.** Without this count the test
        // would pass against a world in which no draw ever fired, and it would
        // then measure the fixture and not the shelter rule.
        would_have_fallen += sheltered
            .iter()
            .filter(|unit| storm::takes_unit(seed, frame, **unit, deficit))
            .count();
        assert!(
            built
                .field
                .units_lost_to_storms()
                .iter()
                .all(|lost| lost.tile != road_tile),
            "a storm took a unit standing under a finished upgrade"
        );
    }
    assert!(
        ticks_under_shelter > 200,
        "the road stood for {ticks_under_shelter} ticks, which is too few for the guard below"
    );
    assert!(
        would_have_fallen > 0,
        "no draw would have taken a sheltered unit, so the test proves nothing about shelter"
    );
    assert!(
        !built.field.units_lost_to_storms().is_empty()
            || exposed
                .iter()
                .any(|unit| built.field.resolve_soldier(*unit).is_err()),
        "the storm took nobody at all, so the fixture proves nothing about the open"
    );
}
