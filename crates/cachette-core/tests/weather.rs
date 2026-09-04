//! Weather, and the power of a god to inflict it on a place.
//!
//! Every test here drives the public crate interface. None reaches into the
//! weather module and none calls the solve directly, because the engine is
//! obligated to call the solve and a test that called it would prove only
//! that the mechanism works.[^1]
//!
//! **The fixture for the effect test holds no water at all.** The sea is the
//! only thing besides a god that puts water into the air, so a fixture with a
//! coastline would wet the control world as well and the assertion would
//! measure the fixture rather than the storm.[^2] The fixture asserts that
//! property of itself, so a change to the ground generator fails the fixture
//! and not the assertion.
//!
//! # References
//!
//! [^1]: Testing rules, section 5. `.claude/rules/testing.md`
//! [^2]: Testing rules, section 2a. `.claude/rules/testing.md`

use cachette_core::resource::{Amount, RecoveryRules, ResourceKind};
use cachette_core::terrain::TileKind;
use cachette_core::weather::{self, WeatherError};
use cachette_core::{Axial, Entity, FactionId, World, WorldConfig};

/// The thread counts that every equivalence test runs at.
const THREAD_COUNTS: [usize; 3] = [1, 2, 12];

/// The extent of the world that holds a coastline.
const WET_EXTENT: u32 = 128;

/// The seed of the world that holds a coastline.
const WET_SEED: u64 = 0x0123_4567_89ab_cdef;

/// The extent of the world that holds no water.
const DRY_EXTENT: u32 = 64;

/// The seed of the world that holds no water.
const DRY_SEED: u64 = 2;

/// Builds a world that holds open water somewhere.
fn coastal_world() -> World {
    let world = World::new(WorldConfig {
        width: WET_EXTENT,
        height: WET_EXTENT,
        seed: WET_SEED,
        faction_count: 2,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    })
    .expect("the extent must describe a world");
    assert!(
        water_tiles(&world) > 0,
        "the fixture holds no water, so nothing lifts"
    );
    world
}

/// Builds a world that holds no open water at all.
fn inland_world() -> World {
    let world = World::new(WorldConfig {
        width: DRY_EXTENT,
        height: DRY_EXTENT,
        seed: DRY_SEED,
        faction_count: 2,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    })
    .expect("the extent must describe a world");
    assert_eq!(
        water_tiles(&world),
        0,
        "the fixture holds water, so the sea would wet the control world"
    );
    world
}

/// Returns how many tiles of a world are open water.
fn water_tiles(world: &World) -> u32 {
    let mut count = 0;
    for address in addresses(world) {
        if world.tile_kind(address) == Some(TileKind::Water) {
            count += 1;
        }
    }
    count
}

/// Returns every address of a world, in index order.
fn addresses(world: &World) -> Vec<Axial> {
    let grid = world.grid();
    (0..grid.tile_count())
        .map(|index| Axial::new((index % grid.width()) as i32, (index / grid.width()) as i32))
        .collect()
}

/// The resource that the effect test gathers.
///
/// Stone is the richest deposit this ground carries, and the test needs a
/// deposit that outlasts one gather at both rates.
const KIND: ResourceKind = ResourceKind::Stone;

/// The stock that the tile of the effect test must carry.
const ENOUGH: Amount = Amount(8);

/// Returns the first open address that carries enough stock, in index order.
///
/// The scan order is fixed, so the answer does not depend on anything but the
/// ground.[^1]
///
/// # References
///
/// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
fn first_rich_tile(world: &World) -> Axial {
    for address in addresses(world) {
        if !world.admits_a_unit(address) {
            continue;
        }
        if world.tile_stock(address, KIND) >= Some(ENOUGH) {
            return address;
        }
    }
    panic!("the fixture carries no deposit worth gathering");
}

/// Stops a deposit from recovering, so a gather reads what it took.
fn no_recovery(world: &mut World) {
    world.set_recovery_rules(RecoveryRules::NONE);
}

#[test]
fn the_world_makes_weather_without_a_caller() {
    let mut world = coastal_world();
    assert!(
        world.weather().is_dry(),
        "a world holds no water before it runs"
    );
    for _ in 0..64 {
        world.step(4).expect("the step must run");
    }
    assert!(
        world.weather().raised() > 0,
        "nothing lifted water in 64 frames"
    );
    assert!(
        world.weather().ground_total().0 > 0,
        "no water reached the ground in 64 frames"
    );
    assert!(
        world.weather().wet_cells() > 0,
        "no cell became wet in 64 frames"
    );
}

#[test]
fn the_weather_varies_over_the_map() {
    let mut world = coastal_world();
    for _ in 0..64 {
        world.step(4).expect("the step must run");
    }
    let plane = world.weather().ground_plane();
    let low = plane.iter().map(|drops| drops.0).min().unwrap_or(0);
    let high = plane.iter().map(|drops| drops.0).max().unwrap_or(0);
    assert!(
        high > low,
        "every cell holds the same water, so the field is not a field"
    );
}

#[test]
fn the_weather_varies_over_time() {
    let mut world = coastal_world();
    for _ in 0..32 {
        world.step(4).expect("the step must run");
    }
    let early: Vec<i64> = world
        .weather()
        .ground_plane()
        .iter()
        .map(|drops| drops.0)
        .collect();
    for _ in 0..32 {
        world.step(4).expect("the step must run");
    }
    let late: Vec<i64> = world
        .weather()
        .ground_plane()
        .iter()
        .map(|drops| drops.0)
        .collect();
    assert_ne!(early, late, "the field did not move in 32 frames");
}

#[test]
fn two_runs_from_one_seed_agree_exactly() {
    let mut first = coastal_world();
    let mut second = coastal_world();
    for _ in 0..48 {
        first.step(4).expect("the step must run");
        second.step(4).expect("the step must run");
    }
    assert_eq!(
        first.weather().ground_plane(),
        second.weather().ground_plane()
    );
    assert_eq!(first.weather().air_plane(), second.weather().air_plane());
    assert_eq!(first.state_hash(), second.state_hash());
}

#[test]
fn the_field_gives_one_answer_at_any_thread_count() {
    let mut planes = Vec::new();
    for threads in THREAD_COUNTS {
        let mut world = coastal_world();
        for _ in 0..24 {
            world.step(threads).expect("the step must run");
        }
        planes.push((
            threads,
            world.weather().air_plane().to_vec(),
            world.weather().ground_plane().to_vec(),
            world.state_hash().finish(),
        ));
    }
    let first = &planes[0];
    for other in &planes[1..] {
        assert_eq!(first.1, other.1, "the air differs at {} threads", other.0);
        assert_eq!(
            first.2, other.2,
            "the ground differs at {} threads",
            other.0
        );
        assert_eq!(first.3, other.3, "the hash differs at {} threads", other.0);
    }
}

#[test]
fn the_water_account_balances_at_every_frame() {
    let mut world = coastal_world();
    for frame in 0..48 {
        world.step(4).expect("the step must run");
        let field = world.weather();
        let accounted = field.air_total().0 + field.ground_total().0 + field.evaporated();
        assert_eq!(
            accounted,
            field.raised(),
            "the account does not balance at frame {frame}"
        );
        assert!(
            world.check_invariants(),
            "the world lost an invariant at frame {frame}"
        );
    }
    assert!(
        world.weather().evaporated() > 0,
        "nothing left the ground, so the field only grows"
    );
}

#[test]
fn no_cell_holds_a_negative_quantity() {
    let mut world = coastal_world();
    for _ in 0..48 {
        world.step(4).expect("the step must run");
    }
    assert!(world.weather().air_plane().iter().all(|drops| drops.0 >= 0));
    assert!(world
        .weather()
        .ground_plane()
        .iter()
        .all(|drops| drops.0 >= 0));
}

#[test]
fn a_dry_world_stores_nothing() {
    let mut world = inland_world();
    for _ in 0..32 {
        world.step(4).expect("the step must run");
    }
    assert!(
        world.weather().is_dry(),
        "a world with no sea and no god put water in the air"
    );
    assert!(world.weather().air_plane().is_empty());
    assert_eq!(world.weather().raised(), 0);
}

#[test]
fn the_lift_draw_is_keyed_on_the_frame() {
    let answers: Vec<bool> = (0..64)
        .map(|tick| weather::cell_lifts(7, cachette_core::Tick(tick), 3, 1024, 0))
        .collect();
    assert!(
        answers.iter().any(|answer| *answer),
        "an all-water cell never lifted in 64 frames"
    );
    assert!(
        answers.iter().any(|answer| !*answer),
        "an all-water cell lifted on every one of 64 frames"
    );
}

#[test]
fn the_lift_draw_is_keyed_on_the_cell() {
    let tick = cachette_core::Tick(11);
    // A cell that is half water lifts on some frames and not on others, so a
    // draw keyed on the cell answers differently for different cells at one
    // frame. A draw that ignored the cell would answer the same for all.
    let answers: Vec<bool> = (0..64)
        .map(|cell| weather::cell_lifts(7, tick, cell, 1024, 512))
        .collect();
    assert!(
        answers.iter().any(|answer| *answer),
        "no cell lifted at this frame"
    );
    assert!(
        answers.iter().any(|answer| !*answer),
        "every cell lifted at this frame, so the cell is not in the key"
    );
}

#[test]
fn the_lift_draw_is_keyed_on_the_seed() {
    let tick = cachette_core::Tick(11);
    let answers: Vec<bool> = (0..64)
        .map(|seed| weather::cell_lifts(seed, tick, 3, 1024, 512))
        .collect();
    assert!(answers.iter().any(|answer| *answer));
    assert!(
        answers.iter().any(|answer| !*answer),
        "every seed lifted, so the seed is not in the key"
    );
}

#[test]
fn high_ground_takes_more_out_of_the_air_than_low_ground() {
    let mut world = coastal_world();
    for _ in 0..8 {
        world.step(4).expect("the step must run");
    }
    let mut lowest = i64::MAX;
    let mut highest = i64::MIN;
    for cell in 0..world.pyramid().len() as u32 {
        let Some(summary) = world.pyramid().cell(cell) else {
            continue;
        };
        let numerator = weather::fall_numerator(summary);
        lowest = lowest.min(numerator);
        highest = highest.max(numerator);
    }
    assert!(
        highest > lowest,
        "every cell takes the same share out of the air, so the ground does nothing"
    );
}

/// Builds an inland world in which one faction holds ground, and returns the
/// world, the unit and the tile it stands on.
///
/// The unit is spawned on a tile that carries food, and the world is stepped
/// until the holding spread has stamped the cell for that faction. A god may
/// act only on ground its own people hold, so the fixture must reach that
/// state before the verb is called at all.
fn a_congregation_on_the_ground() -> (World, Entity, Axial) {
    let mut world = inland_world();
    no_recovery(&mut world);
    let address = first_rich_tile(&world);
    let unit = world
        .spawn_soldier(address, FactionId(0))
        .expect("the tile admits a unit");
    for _ in 0..16 {
        world.step(1).expect("the step must run");
    }
    // The unit walks while the holding spreads, and the test needs it back on
    // the deposit it was spawned on.
    world
        .place_soldier(unit, address)
        .expect("the tile admits the unit");
    world.rebuild_bridge(1).expect("the rebuild must run");
    assert!(
        world
            .holders_near(address)
            .is_some_and(|mask| mask.contains(FactionId(0))),
        "the faction holds no ground in this cell after sixteen frames"
    );
    (world, unit, address)
}

#[test]
fn a_god_may_not_strike_ground_its_faction_does_not_hold() {
    let (mut world, _unit, _address) = a_congregation_on_the_ground();
    // The far corner sits in a cell that no unit of this faction reached in
    // sixteen frames, and the fixture asserts that.
    let far = Axial::new(DRY_EXTENT as i32 - 1, DRY_EXTENT as i32 - 1);
    assert!(
        !world
            .holders_near(far)
            .is_some_and(|mask| mask.contains(FactionId(0))),
        "the faction reached the far corner, so the fixture proves nothing"
    );
    let refusal = world.inflict_weather(FactionId(0), &[far], 2);
    assert_eq!(refusal, Err(WeatherError::GroundNotHeld(far)));
    assert_eq!(
        world.weather().raised(),
        0,
        "a refusal put water in the air"
    );
}

#[test]
fn one_refusal_leaves_the_world_unchanged() {
    let (mut world, _unit, address) = a_congregation_on_the_ground();
    let far = Axial::new(DRY_EXTENT as i32 - 1, DRY_EXTENT as i32 - 1);
    let before = world.state_hash();
    // The good place comes first, so a call that wrote as it resolved would
    // have written before it met the place it refuses.
    let refusal = world.inflict_weather(FactionId(0), &[address, far], 2);
    assert_eq!(refusal, Err(WeatherError::GroundNotHeld(far)));
    assert_eq!(
        world.state_hash(),
        before,
        "a refused call changed the world"
    );
    assert_eq!(
        world.weather().ready_at(FactionId(0)),
        Some(cachette_core::Tick(0)),
        "a refused call spent the cooldown"
    );
}

#[test]
fn a_god_waits_between_one_storm_and_the_next() {
    let (mut world, _unit, address) = a_congregation_on_the_ground();
    let storm = world
        .inflict_weather(FactionId(0), &[address], 2)
        .expect("the faction holds this ground");
    assert_eq!(storm.cells, 1);
    assert!(storm.drops > 0);
    let refusal = world.inflict_weather(FactionId(0), &[address], 2);
    assert_eq!(
        refusal,
        Err(WeatherError::StillCooling {
            ready_at: storm.ready_at
        })
    );
}

#[test]
fn the_verb_refuses_a_strength_outside_its_range() {
    let (mut world, _unit, address) = a_congregation_on_the_ground();
    assert_eq!(
        world.inflict_weather(FactionId(0), &[address], 0),
        Err(WeatherError::StrengthOutOfRange(0))
    );
    assert_eq!(
        world.inflict_weather(
            FactionId(0),
            &[address],
            weather::STRENGTH_CEILING.saturating_add(1)
        ),
        Err(WeatherError::StrengthOutOfRange(
            weather::STRENGTH_CEILING + 1
        ))
    );
}

#[test]
fn the_verb_refuses_more_places_than_one_call_carries() {
    let (mut world, _unit, address) = a_congregation_on_the_ground();
    let places = vec![address; weather::PLACES_CEILING + 1];
    assert_eq!(
        world.inflict_weather(FactionId(0), &places, 1),
        Err(WeatherError::TooManyPlaces(weather::PLACES_CEILING + 1))
    );
}

#[test]
fn the_verb_answers_once_for_a_whole_set() {
    let (mut world, _unit, address) = a_congregation_on_the_ground();
    // Two places in one cell are one place, so the report counts one cell.
    let storm = world
        .inflict_weather(FactionId(0), &[address, address], 1)
        .expect("the faction holds this ground");
    assert_eq!(storm.cells, 1, "one cell took water twice");
}

#[test]
fn a_god_wets_the_ground_it_strikes() {
    let (mut world, _unit, address) = a_congregation_on_the_ground();
    assert_eq!(world.ground_is_wet(address), Some(false));
    world
        .inflict_weather(FactionId(0), &[address], weather::STRENGTH_CEILING)
        .expect("the faction holds this ground");
    assert!(
        world.air_at(address).unwrap_or(0) > 0,
        "the storm put no water in the air"
    );
    world.step(1).expect("the step must run");
    assert_eq!(
        world.ground_is_wet(address),
        Some(true),
        "the storm did not reach the ground in one frame"
    );
}

#[test]
fn a_gatherer_on_ground_a_god_wet_takes_more() {
    let (mut wet, wet_unit, address) = a_congregation_on_the_ground();
    let (mut dry, dry_unit, _) = a_congregation_on_the_ground();

    wet.inflict_weather(FactionId(0), &[address], weather::STRENGTH_CEILING)
        .expect("the faction holds this ground");
    // The storm reaches the ground at the end of the next frame, and the
    // gather resolve of the frame after that reads it.
    wet.step(1).expect("the step must run");
    dry.step(1).expect("the step must run");
    assert_eq!(wet.ground_is_wet(address), Some(true));
    assert_eq!(dry.ground_is_wet(address), Some(false));

    let wet_took = one_gather(&mut wet, wet_unit);
    let dry_took = one_gather(&mut dry, dry_unit);
    assert!(dry_took > 0, "the control world gathered nothing at all");
    assert!(
        wet_took > dry_took,
        "wet ground gave {wet_took} and dry ground gave {dry_took}"
    );
}

/// Orders one gather, runs one frame, and returns what the unit took.
///
/// The order is a control-plane verb and the resolve is the engine's, so the
/// test orders and steps rather than calling the resolve.[^1]
///
/// # References
///
/// [^1]: Testing rules, section 5. `.claude/rules/testing.md`
fn one_gather(world: &mut World, unit: Entity) -> u32 {
    assert!(
        world.order_gather(unit, KIND),
        "the unit must take the order"
    );
    world.step(1).expect("the step must run");
    world
        .gather_log()
        .iter()
        .filter(|event| event.unit == unit.to_bits())
        .map(|event| event.amount)
        .sum()
}

#[test]
fn the_solve_runs_a_fixed_number_of_passes() {
    let (mut world, _unit, address) = a_congregation_on_the_ground();
    // The world holds no water until the god acts, so the pass count stands at
    // zero however many frames have run.
    assert_eq!(world.weather().passes(), 0);
    world
        .inflict_weather(FactionId(0), &[address], 1)
        .expect("the faction holds this ground");
    let frames = 12u64;
    for _ in 0..frames {
        world.step(1).expect("the step must run");
    }
    assert_eq!(
        world.weather().passes(),
        frames * u64::from(weather::PASSES_FOR_EACH_SOLVE),
        "the solve did not run the fixed count on every frame"
    );
}
