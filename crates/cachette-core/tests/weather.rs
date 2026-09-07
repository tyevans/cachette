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
use cachette_core::{Axial, Entity, FactionId, Tick, World, WorldConfig, SUBSYSTEM_CENSUS};

/// Returns one census count by name.
fn census(world: &World, name: &str) -> i64 {
    SUBSYSTEM_CENSUS
        .iter()
        .find(|row| row.name == name)
        .map(|row| (row.read)(world))
        .expect("the census holds the row")
}

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
    // **A cell counts as wet at a share of the ceiling of the air plane, and
    // that ceiling rose with the published saturation curve.** So a cell needs
    // longer to gather that much water than it did, and the frame count here
    // is a property of the fixture rather than of the rule.[^1]
    //
    // [^1]: ADR-0177, the row axis of a world is a latitude that the world states, decision D4. `docs/adrs/draft/adr-0177-the-row-axis-of-a-world-is-a-latitude-that-the-world-states.md`
    const FRAMES: usize = 1024;
    for _ in 0..FRAMES {
        world.step(4).expect("the step must run");
    }
    assert!(
        world.weather().raised() > 0,
        "nothing lifted water in {FRAMES} frames"
    );
    assert!(
        world.weather().ground_total().0 > 0,
        "no water reached the ground in {FRAMES} frames"
    );
    assert!(
        world.weather().wet_cells() > 0,
        "no cell became wet in {FRAMES} frames, and the wettest cell holds {} drops \
         against a mark of {}",
        (0..world.weather().cells().tile_count())
            .map(|cell| world.weather().ground_at(cell).0)
            .max()
            .unwrap_or(0),
        weather::WET_MARK.0
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

// The resolution of the weather is a parameter of the world, and one of the
// values it takes gives each tile a cell of its own. The tests below hold the
// same properties at that pitch. A test at one pitch measures the pitch and
// not the field.[^1]
//
// [^1]: Testing rules, section 2a. `.claude/rules/testing.md`

/// Builds a coastal world that carries one weather cell for each tile.
fn per_tile_world() -> World {
    let world = World::with_weather_scale(
        WorldConfig {
            width: PER_TILE_EXTENT,
            height: PER_TILE_EXTENT,
            seed: WET_SEED,
            faction_count: 2,
            unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
        },
        weather::WeatherScale::PER_TILE,
    )
    .expect("the extent must describe a world");
    assert!(
        water_tiles(&world) > 0,
        "the fixture holds no water, so nothing lifts"
    );
    // The whole lattice carries a margin that no reader sees, so the count
    // that must match the world is the count of the inner lattice.
    assert_eq!(
        world.weather().lattice().inner().tile_count(),
        PER_TILE_EXTENT * PER_TILE_EXTENT,
        "the lattice does not hold one cell for each tile"
    );
    world
}

/// The extent of the per-tile world.
///
/// It is smaller than the coastal world, because a per-tile lattice runs the
/// whole world on every transport pass.
const PER_TILE_EXTENT: u32 = 48;

#[test]
fn a_per_tile_field_gives_one_answer_at_any_thread_count() {
    let mut planes = Vec::new();
    for threads in THREAD_COUNTS {
        let mut world = per_tile_world();
        for _ in 0..16 {
            world.step(threads).expect("the step must run");
        }
        planes.push((
            threads,
            world.weather().air_plane().to_vec(),
            world.weather().ground_plane().to_vec(),
            world.weather().wind_plane().to_vec(),
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
        assert_eq!(first.3, other.3, "the wind differs at {} threads", other.0);
        assert_eq!(first.4, other.4, "the hash differs at {} threads", other.0);
    }
}

#[test]
fn a_per_tile_field_conserves_water_exactly() {
    let mut world = per_tile_world();
    for frame in 0..24 {
        world.step(4).expect("the step must run");
        let field = world.weather();
        let accounted = field.air_total().0 + field.ground_total().0 + field.evaporated();
        assert_eq!(
            accounted,
            field.raised(),
            "the account does not balance at frame {frame}"
        );
        assert!(
            field.air_plane().iter().all(|drops| drops.0 >= 0),
            "a cell holds negative air at frame {frame}"
        );
    }
    assert!(
        world.weather().raised() > 0,
        "nothing lifted water at the per-tile pitch"
    );
}

/// The weather still varies over a per-tile map, rather than wetting it alike.
///
/// **A finer lattice that stops travelling is worse than a coarse one that
/// moves.** A field that reached the same value everywhere would pass the
/// account test and tell a watcher nothing.
#[test]
fn a_per_tile_field_still_separates_wet_ground_from_dry() {
    let mut world = per_tile_world();
    for _ in 0..48 {
        world.step(4).expect("the step must run");
    }
    let plane = world.weather().ground_plane();
    let low = plane.iter().map(|drops| drops.0).min().unwrap_or(0);
    let high = plane.iter().map(|drops| drops.0).max().unwrap_or(0);
    assert!(
        high > low,
        "every tile holds the same water, so the field is not a field"
    );
    let wet = world.weather().wet_cells();
    // The count covers the world, so the ceiling is the cell count of the
    // world and not the cell count of the whole lattice. The lattice carries
    // a margin that no reader sees.
    assert!(
        wet > 0 && wet < world.weather().lattice().inner().tile_count(),
        "the map is wet everywhere or dry everywhere: {wet} cells"
    );
}

/// The wind turns at the per-tile pitch, and does not sit at rest.
///
/// **A finer lattice holds a smaller temperature difference between two
/// neighbours.** The divisor that turns that difference into a wind therefore
/// follows the cell side. A fixed divisor left every cell still, and a still
/// field carries no water anywhere.
#[test]
fn a_per_tile_field_raises_a_wind() {
    let mut world = per_tile_world();
    for _ in 0..48 {
        world.step(4).expect("the step must run");
    }
    let moving = world
        .weather()
        .wind_plane()
        .iter()
        .filter(|wind| !wind.is_still())
        .count();
    let cells = world.weather().wind_plane().len();
    // **A wind over a few cells is not a wind over the map.** A divisor that
    // did not follow the cell side left one cell in eighteen moving, and the
    // rest at rest, so a count of moving cells is what tells the two apart.
    // A test that asked only for a nonzero fastest speed passed under that
    // defect.
    assert!(
        moving * 2 > cells,
        "only {moving} of {cells} cells carry a wind, so the field mostly sits"
    );
}

/// A tile reads the weather of its own cell at the per-tile pitch.
///
/// **The readers of the field must keep working at any resolution.** The
/// gather resolve asks whether the cell of a tile is wet, and the drawing
/// asks the same reader for the air and the wind.
#[test]
fn a_tile_reads_the_weather_of_its_own_cell() {
    let mut world = per_tile_world();
    for _ in 0..48 {
        world.step(4).expect("the step must run");
    }
    let mut different = false;
    let mut first = None;
    for address in addresses(&world) {
        let Some(air) = world.air_at(address) else {
            continue;
        };
        assert!(world.ground_water_at(address).is_some());
        assert!(world.wind_at(address).is_some());
        assert!(world.ground_is_wet(address).is_some());
        match first {
            None => first = Some(air),
            Some(seen) if seen != air => different = true,
            Some(_) => {}
        }
    }
    assert!(
        different,
        "every tile reads the same air, so the reader answers for one cell"
    );
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
fn cold_ground_holds_less_air_than_warm_ground() {
    let mut world = coastal_world();
    for _ in 0..8 {
        world.step(4).expect("the step must run");
    }
    // The ground of each weather cell, folded from the tiles the way the
    // world folds it. The air met no cooling on the way, so the numerator
    // reads the cell alone.
    let mut under = vec![weather::CellGround::EMPTY; world.weather().air_plane().len().max(1)];
    for address in addresses(&world) {
        let (Some(tile), Some(index)) = (
            world.tile_terrain(address),
            world
                .grid()
                .index_of(address)
                .and_then(|at| world.weather_cell_of(at)),
        ) else {
            continue;
        };
        if let Some(slot) = under.get_mut(index as usize) {
            *slot = slot.combine(weather::CellGround::of_tile(tile));
        }
    }
    // **The reference is the mean land height of this world, not sea level.**
    // The published lapse rate is measured about the mean height the balance
    // constants already average over, so a cell reads its own height against
    // that mean. The engine computes the reference over the same plane, so
    // the test asks for it the way the engine does.[^2]
    //
    // [^2]: ADR-0182, decision D5. `docs/adrs/draft/adr-0182-the-temperature-a-cell-is-driven-toward-is-a-published-energy-balance.md`
    let reference = weather::mean_land_height_over(&under);
    let mut lowest = i64::MAX;
    let mut highest = i64::MIN;
    for ground in under {
        if ground.tiles() == 0 {
            continue;
        }
        // **The ground supplies a cooling and not a heat.** It is a signed
        // number of hundredths of a degree below the balance, so the test
        // reads it directly rather than through the capacity of a warmth it
        // no longer produces.[^1]
        //
        // [^1]: ADR-0182, the temperature a cell is driven toward is a published energy balance, decision D4. `docs/adrs/draft/adr-0182-the-temperature-a-cell-is-driven-toward-is-a-published-energy-balance.md`
        let cooling = i64::from(weather::relief_cooling_of(
            ground,
            weather::HeightRange::DEFAULT,
            reference,
        ));
        lowest = lowest.min(cooling);
        highest = highest.max(cooling);
    }
    assert!(
        highest > lowest,
        "every cell stands at one height, so the ground does nothing"
    );
}

/// Cooling along the wind adds to the share that falls.
///
/// This is the part that height alone cannot give. Air that arrives from a
/// warmer cell drops more than air that arrives from a cell of its own
/// temperature, so the near side of a ridge is wet and the far side is
/// dry.[^1]
///
/// # References
///
/// [^1]: ADR-0162, water enters the air where it is hot, and it falls where the air cools, decision D2. `docs/adrs/accepted/adr-0162-water-enters-the-air-where-it-is-hot-and-falls-where-the-air-cools.md`
#[test]
fn air_that_cooled_on_the_way_holds_less_than_air_that_did_not() {
    let base = weather::capacity_at(cachette_core::HEAT_CEILING / 4);
    let no_cooling = weather::travelling_capacity(base, 0, 0);
    let cooled = weather::travelling_capacity(base, cachette_core::HEAT_CEILING / 2, 0);
    assert!(
        cooled.0 < no_cooling.0,
        "cooling shed nothing, so a ridge has no near side and no far side"
    );
    // Air that lost capacity holds less, so the water it carries above the
    // smaller mark leaves the air. That excess is what the settle pass pours
    // onto the ground.
    assert!(
        no_cooling.0 - cooled.0 > 0,
        "the shed moved no water out of the air of a cooling cell"
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
    // The faction holds the ground because it owns a city there. A unit
    // gives its faction no claim on the tile it stands on.[^1]
    //
    // [^1]: ADR-0150, held ground is the ground within reach of a city its faction owns, decision D1. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
    world
        .found_settlement(address, FactionId(0))
        .expect("the ground admits a city");
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
        "the faction holds no ground in this cell after the founding"
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

/// The census row counts the storms a god raised, and one call is one storm.
///
/// **The row once read whether the raised total stood above zero.** That
/// reader answers one for a run that raised one storm and one for a run that
/// raised a hundred, so the fixture raises two and asserts the count. A
/// refused call raises nothing, and the row must not count it.[^3]
///
/// # References
///
/// [^3]: Findings register, FND-498. `docs/FINDINGS.md`
#[test]
fn the_census_counts_each_storm_a_god_raised() {
    let (mut world, _unit, address) = a_congregation_on_the_ground();
    assert_eq!(
        census(&world, "storms_raised"),
        0,
        "a dry world raised none"
    );
    let far = Axial::new(DRY_EXTENT as i32 - 1, DRY_EXTENT as i32 - 1);
    assert!(
        world.inflict_weather(FactionId(0), &[far], 2).is_err(),
        "the fixture needs a refusal here"
    );
    assert_eq!(
        census(&world, "storms_raised"),
        0,
        "a refused call counted as a storm"
    );
    world
        .inflict_weather(FactionId(0), &[address], 2)
        .expect("the faction holds this ground");
    assert_eq!(census(&world, "storms_raised"), 1);
    // The god waits out its cooldown and strikes again. A row that read
    // whether any water stands would still say one here.
    for _ in 0..weather::COOLDOWN_TICKS {
        world.step(1).expect("the step must run");
    }
    world
        .inflict_weather(FactionId(0), &[address], 2)
        .expect("the cooldown has run out");
    assert_eq!(
        census(&world, "storms_raised"),
        2,
        "the row must count each storm and not whether one stands"
    );
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
        frames * u64::from(weather::WeatherScale::LEVEL_1.transport_passes()),
        "the solve did not run the fixed count on every frame"
    );
}

// The temperature is carried state that a season and the sky drive. The tests
// below test what the value depends on, and not only that it repeats. A
// deterministic defect passes both determinism tests, so each field of the
// driver gets its own test.[^1]
//
// [^1]: Testing rules, section 2. `.claude/rules/testing.md`

/// The temperature of a fixed cell moves as the season goes past.
///
/// **This is the property the project owner asked for.** A field whose driver
/// is fixed gives a wind that settles and a plume that never moves. The
/// temperature must therefore change at one place over a run.
#[test]
fn the_temperature_of_one_cell_changes_over_a_run() {
    let mut world = coastal_world();
    let mut seen: Vec<i32> = Vec::new();
    for tick in 1..=256 {
        world.step(4).expect("the step must run");
        if tick % 16 == 0 {
            // The reading names the first cell of the world. Cell zero of
            // the plane is a margin cell, and no watcher sees one.
            let cell = world
                .weather()
                .lattice()
                .whole_of_inner(0)
                .expect("the world holds a first cell");
            seen.push(world.weather().warmth_at(cell));
        }
    }
    let low = seen.iter().copied().min().unwrap_or(0);
    let high = seen.iter().copied().max().unwrap_or(0);
    assert!(
        high > low,
        "one cell held one temperature for 256 frames, so nothing drives it: {seen:?}"
    );
}

/// The season is in the tick, so one cell answers differently later.
#[test]
fn the_season_is_keyed_on_the_tick() {
    let latitude = 45 * weather::LATITUDE_FINE;
    let early = weather::season_at(Tick(0), latitude);
    let mut moved = false;
    for tick in 1..512u64 {
        if weather::season_at(Tick(tick), latitude) != early {
            moved = true;
            break;
        }
    }
    assert!(moved, "the season answers the same at every tick");
}

/// The season is in the latitude, so two cells differ at one tick.
///
/// **A term that added the same degrees to every cell would move nothing.**
/// The wind answers to the difference between two cells, and one offset added
/// to every cell cancels in that difference exactly.
#[test]
fn the_season_varies_across_the_lattice_at_one_tick() {
    let height = 16;
    let latitudes = weather::Latitudes::PLANET;
    let readings: Vec<i32> = (0..height)
        .map(|at| weather::season_at(Tick(0), latitudes.of_row(at, height)))
        .collect();
    let low = readings.iter().copied().min().unwrap_or(0);
    let high = readings.iter().copied().max().unwrap_or(0);
    assert!(
        high > low,
        "the season answers the same at every cell, so it cancels in the wind: {readings:?}"
    );
}

/// The sun swings between two limits and it never wraps.
///
/// **A wrap is a seam.** A warm band that marched along an axis and wrapped
/// jumped the whole extent of the map every lap, and the temperature field
/// held a discontinuity there whatever the shape of the band. The declination
/// turns back at each limit instead, so the largest move it makes in one tick
/// is small and the field holds no seam.
#[test]
fn the_sun_swings_between_two_limits_and_never_wraps() {
    let readings: Vec<i32> = (0..2 * weather::SEASON_PERIOD_TICKS as u64)
        .map(|tick| weather::declination_at(Tick(tick)))
        .collect();
    let step = readings
        .windows(2)
        .map(|pair| (pair[1] - pair[0]).abs())
        .max()
        .unwrap_or(0);
    // The declination crosses the equator fastest. It covers four amplitudes
    // in one period, so the mean step is small and the largest is a fraction
    // of one degree.
    assert!(
        step <= weather::LATITUDE_FINE / 2,
        "the sun moved {step} hundredths of a degree in one tick, so the field holds a seam"
    );
    let low = readings.iter().copied().min().unwrap_or(0);
    let high = readings.iter().copied().max().unwrap_or(0);
    assert!(
        high > 20 * weather::LATITUDE_FINE && low < -20 * weather::LATITUDE_FINE,
        "the sun swung from {low} to {high}, which is no season"
    );
}

/// The season falls away from the sun without a step in its slope.
///
/// **A step in the slope is what put the vertical bands into the wind.** The
/// pressure gradient reads the slope of the temperature, so a profile whose
/// slope jumps at one row gives a wind that reverses along that row and paints
/// a straight line across the map. The smooth fall has no such row.
#[test]
fn the_season_slope_holds_no_step() {
    let height = 256;
    let latitudes = weather::Latitudes::PLANET;
    for tick in [0u64, 137, 512, 1024, 1500] {
        let readings: Vec<i32> = (0..height)
            .map(|at| weather::season_at(Tick(tick), latitudes.of_row(at, height)))
            .collect();
        // The slope is read over a block of rows, because the truncation of
        // one row is a whole degree and a single difference is mostly that.
        const BLOCK: usize = 16;
        let slopes: Vec<i32> = readings
            .chunks_exact(BLOCK)
            .map(|block| block[BLOCK - 1] - block[0])
            .collect();
        let bend = slopes
            .windows(2)
            .map(|pair| (pair[1] - pair[0]).abs())
            .max()
            .unwrap_or(0);
        // The old profile was a triangle, so its slope ran at one sign over
        // half the map and the other sign over the rest, and it reversed
        // between two neighbouring blocks. That reversal is a bend of twice
        // the block slope, which was about thirty degrees here.
        //
        // **The published profile carries a bend of its own, and it is
        // real.** The insolation is nothing at all through the polar night,
        // so the season part clamps over the polar rows and the annual mean
        // is nearly flat there. The bar sits above that bend and below the
        // reversal of a triangle.
        assert!(
            bend <= 20,
            "the slope of the season bent by {bend} degrees over {BLOCK} rows \
             at tick {tick}, so it holds a step: {slopes:?}"
        );
    }
}

/// The poles are cold, and no rule says so.
///
/// **One term gives the season and the cold poles together.** The heating of
/// a cell reads how far it sits from where the sun stands, and the sun never
/// reaches a pole. So a pole is far from the sun for the whole swing and the
/// middle of the world is near it for the whole swing. A second rule that
/// made the poles cold would state the same fact twice.
#[test]
fn the_poles_are_colder_than_the_middle_over_a_whole_year() {
    let height = 64;
    let latitudes = weather::Latitudes::PLANET;
    let over_a_year = |row: u32| -> i64 {
        (0..weather::SEASON_PERIOD_TICKS as u64)
            .map(|tick| {
                i64::from(weather::season_at(
                    Tick(tick),
                    latitudes.of_row(row, height),
                ))
            })
            .sum()
    };
    let north = over_a_year(0);
    let middle = over_a_year(height / 2);
    let south = over_a_year(height - 1);
    assert!(
        middle > north && middle > south,
        "the middle took {middle} over a year, the poles {north} and {south}"
    );
}

/// The sun reaches neither pole, so each of them has a colder half year.
#[test]
fn each_pole_holds_its_own_winter() {
    let height = 64;
    let latitudes = weather::Latitudes::PLANET;
    let reading =
        |row: u32, tick: u64| weather::season_at(Tick(tick), latitudes.of_row(row, height));
    let quarter = weather::SEASON_PERIOD_TICKS as u64 / 4;
    // A quarter turn after the start the sun stands at the limit nearest the
    // last row, and three quarters after it stands at the limit nearest the
    // first row. So the two poles hold their summers half a period apart.
    let first_summer = reading(0, 3 * quarter);
    let first_winter = reading(0, quarter);
    let last_summer = reading(height - 1, quarter);
    let last_winter = reading(height - 1, 3 * quarter);
    assert!(
        first_summer > first_winter,
        "the first row read {first_summer} in summer and {first_winter} in winter"
    );
    assert!(
        last_summer > last_winter,
        "the last row read {last_summer} in summer and {last_winter} in winter"
    );
}

/// The cloud is in the air, so a full sky is colder than an empty one.
#[test]
fn a_full_sky_takes_more_warmth_than_an_empty_one() {
    let empty = weather::cloud_at(weather::Drops(0), weather::AIR_SATURATION);
    let full = weather::cloud_at(weather::AIR_SATURATION, weather::AIR_SATURATION);
    assert!(
        full > empty,
        "the cloud takes the same at every quantity of air"
    );
}

/// The temperature lags its driver, and one pass never reaches it.
///
/// The lag is the whole reason the field carries the temperature rather than
/// deriving it. A pass that assigned the asked temperature would hold the
/// driver of the moment, and no warm parcel would outlive the cell it left.
#[test]
fn the_temperature_does_not_reach_its_driver_in_one_pass() {
    let mut world = coastal_world();
    // The field starts at the middle of the scale. One step cannot carry a
    // cell whose driver sits at an end of the scale all the way there.
    world.step(4).expect("the step must run");
    let plane: Vec<i32> = world.weather().warmth_plane().to_vec();
    assert!(
        plane
            .iter()
            .all(|degrees| *degrees > 0 && *degrees < cachette_core::HEAT_CEILING),
        "one pass carried a cell to an end of the scale, so nothing lags: {plane:?}"
    );
}

/// The carry cannot run away at any wind and over any number of passes.
///
/// The six shares of one cell sum to less than one whole, so the new
/// temperature lies inside the range of the temperatures the pass read. The
/// temperature is not conserved, so no account reports a defect in it, and the
/// bound is the only thing a test can check.
#[test]
fn the_temperature_stays_inside_its_scale_over_a_long_run() {
    let mut world = coastal_world();
    for _ in 0..512 {
        world.step(4).expect("the step must run");
        for degrees in world.weather().warmth_plane() {
            assert!(
                (0..=cachette_core::HEAT_CEILING).contains(degrees),
                "a cell left the temperature scale at {degrees}"
            );
        }
    }
}

/// The peak of the air plane travels, rather than settling on one cell.
///
/// **This is the measurement the project owner made.** The peak sat on one
/// cell from tick 41 to the end of a 400 tick run, because the heat was
/// derived from ground that does not move. The peak must now move, and keep
/// moving.
///
/// **The run covers half a season.** The sun takes 2048 ticks to complete one
/// swing, and a run shorter than half of that watches one part of the swing
/// and calls a slow field a pinned one.
#[test]
fn the_peak_of_the_air_plane_keeps_moving() {
    let mut world = World::new(WorldConfig {
        width: 256,
        height: 256,
        seed: 0x2f,
        faction_count: 4,
        unit_capacity: 1024,
    })
    .expect("the extent must describe a world");
    let mut peaks: Vec<usize> = Vec::new();
    for tick in 1..=(weather::SEASON_PERIOD_TICKS / 2) {
        world.step(1).expect("the step must run");
        if tick % 20 != 0 {
            continue;
        }
        let plane = world.weather().air_plane();
        let mut best = 0usize;
        for cell in 0..plane.len() {
            if plane[cell].0 > plane[best].0 {
                best = cell;
            }
        }
        peaks.push(best);
    }
    let mut seen = peaks.clone();
    seen.sort_unstable();
    seen.dedup();
    assert!(
        seen.len() >= 4,
        "the peak visited {} cells in 400 frames, so the weather is pinned: {peaks:?}",
        seen.len()
    );
    // The peak must keep moving, and not move once and stop. The second half
    // of the run has to move as well as the first.
    let late = &peaks[peaks.len() / 2..];
    let mut moves = 0usize;
    for pair in late.windows(2) {
        if pair[0] != pair[1] {
            moves += 1;
        }
    }
    assert!(
        moves >= 3,
        "the peak moved {moves} times in the second half of the run: {late:?}"
    );
}

// **There is no test here that the temperature reaches the state hash.** The
// claim needs two fields whose only difference is the temperature, and this
// interface cannot build that pair. Every frame moves the pass counts and the
// wind as well, and the level 1 summary that would hold the ground still has
// no constructor a test can call. A test that stepped one world and found two
// unequal hashes would pass with the temperature removed from the hash, and a
// test that cannot fail is decoration.[^2]
//
// [^2]: Testing rules, section 1. `.claude/rules/testing.md`

/// Air that climbs drops more than air that ran level, and air that
/// descended drops less.
///
/// **This is the one term that couples the height map to the flow.** The flow
/// is horizontal over a flat lattice, so without it a parcel blown at a range
/// of hills passes through it and no cloud forms over land.
#[test]
fn air_that_climbed_holds_less_than_air_that_ran_level() {
    let base = weather::capacity_at(cachette_core::HEAT_CEILING / 2);
    let level = weather::travelling_capacity(base, 0, 0);
    let climbed = weather::travelling_capacity(base, 0, cachette_core::Fix32::ONE.0 / 4);
    let fell = weather::travelling_capacity(base, 0, -cachette_core::Fix32::ONE.0 / 4);
    assert!(
        climbed.0 < level.0,
        "air that climbed held as much as air that ran level, \
         so the height map does not reach the water"
    );
    assert!(
        fell.0 > level.0,
        "air that descended held no more than air that ran level, \
         so the lee of a range is as wet as the windward side"
    );
}

/// The air over a cell never stands above the saturation mark after a solve.
///
/// **The transport was the one site that could carry a cell past the mark.**
/// The lift bounds the source and not the sum of what several winds deliver
/// into one convergence cell, so such a cell climbed without a bound and one
/// outlier set the top of every ramp that read the plane.
#[test]
fn the_air_never_stands_above_the_saturation_mark() {
    // **The fixture runs at the per-tile pitch on purpose.** The coarse world
    // holds a lattice a few cells across, so no wind converges on one cell
    // and the plane never approaches the mark. The test passed against the
    // defect it exists to catch until the fixture was changed.[^1]
    //
    // [^1]: Testing Rules, section 2a. `.agents/rules/testing.md`
    let mut world = World::with_weather_scale(
        WorldConfig {
            width: WET_EXTENT,
            height: WET_EXTENT,
            seed: WET_SEED,
            faction_count: 2,
            unit_capacity: 1024,
        },
        weather::WeatherScale::PER_TILE,
    )
    .expect("the extent must describe a world");
    for _ in 0..20 {
        world.step(4).expect("the step must run");
        let highest = world
            .weather()
            .air_plane()
            .iter()
            .map(|drops| drops.0)
            .max()
            .unwrap_or(0);
        assert!(
            highest <= weather::AIR_SATURATION.0,
            "a cell held {highest} drops against a mark of {}",
            weather::AIR_SATURATION.0
        );
    }
    assert!(
        world.weather().check_account(),
        "the rain that left the air is not accounted"
    );
}

/// Shallow water is a warmer cell than deep water, and deep water lags.
///
/// A tile counts as water when its height falls below one mark, and the count
/// past that mark carries no depth, so nothing else in the field can tell a
/// shelf from an abyss.
#[test]
fn shallow_water_lags_less_than_deep_water() {
    let mark = i64::from(cachette_core::terrain::HEIGHT_WATER.0);
    let sea = |height: i64| weather::CellGround {
        height_total: height * 1024,
        water_height_total: height * 1024,
        tiles: 1024,
        open_tiles: 0,
    };
    let shelf = sea(mark - mark / 8);
    let abyss = sea(mark / 8);
    // **This test held a second clause and a record removed it.** It asserted
    // that a shelf stands warmer than an abyss in the mean. The sea moderates
    // a coast by holding its heat rather than by sitting at a different mean,
    // and the field carries that as the lag below. A second term for it would
    // be one fact in two places, so the driver no longer holds one.[^1]
    //
    // [^1]: ADR-0182, the temperature a cell is driven toward is a published energy balance, decision D4. `docs/adrs/draft/adr-0182-the-temperature-a-cell-is-driven-toward-is-a-published-energy-balance.md`
    assert!(
        weather::lag_of(abyss) > weather::lag_of(shelf),
        "an abyss tracks the season as fast as a shelf does"
    );
}

/// A cold sky holding little water is grey, and a warm sky holding the same
/// water is clear.
///
/// **This is the whole of the missing physics in one assertion.** Warm air
/// holds a lot of water and cold air holds very little, so the visible cloud
/// is the air held against the capacity of that air, and never the air held
/// against one mark that every cell shares. Without it a high latitude is
/// structurally cloudless. Nothing lifts water there, and the water that
/// arrives stands below a mark that the tropics set.
#[test]
fn one_quantity_of_water_is_cloud_when_it_is_cold_and_clear_when_it_is_warm() {
    let cold = cachette_core::HEAT_CEILING / 8;
    let warm = cachette_core::HEAT_CEILING;
    let polar = weather::capacity_at(cold);
    let tropical = weather::capacity_at(warm);
    assert!(
        polar.0 < tropical.0,
        "the cold sky holds {} drops and the warm sky holds {}, so the capacity ignores the cold",
        polar.0,
        tropical.0
    );
    // One quantity of water. It fills the cold sky and not the warm one.
    let water = weather::Drops(polar.0);
    assert_eq!(
        weather::cloud_at(water, polar),
        weather::cloud_at(weather::AIR_SATURATION, weather::AIR_SATURATION),
        "the cold sky is not fully overcast at its own capacity"
    );
    assert!(
        weather::cloud_at(water, tropical) < weather::cloud_at(water, polar) / 4,
        "the same water shades the warm sky nearly as much as the cold one"
    );
}

/// A sky that loses capacity puts its water on the ground, never into
/// nothing.
///
/// **This is the account across a temperature change, and it is the hard
/// part.** The capacity of a cell falls when the cell cools, so air that was
/// invisible vapour on one solve stands above the mark on the next. The pass
/// must move that water onto the ground of the same cell. A pass that
/// assigned the air to the capacity, or that scaled it, would destroy water,
/// and nothing else in the field would notice.
///
/// The test drives the engine and not the capacity function. A god fills the
/// sky over held ground far past what any capacity allows, and the account
/// must hold on every tick while the field carries that water away.[^1]
///
/// # References
///
/// [^1]: Testing Rules, section 5. `.agents/rules/testing.md`
#[test]
fn a_sky_that_loses_capacity_puts_its_water_on_the_ground() {
    let (mut world, _unit, address) = a_congregation_on_the_ground();
    let before = world.weather().air_total().0 + world.weather().ground_total().0;
    let storm = world
        .inflict_weather(FactionId(0), &[address], weather::STRENGTH_CEILING)
        .expect("the faction holds the ground");
    assert!(storm.drops > 0, "the god raised no water");
    assert!(
        world.weather().air_total().0 + world.weather().ground_total().0 > before,
        "the storm put no water into the world"
    );
    for _ in 0..64 {
        world.step(4).expect("the step must run");
        assert!(
            world.weather().check_account(),
            "water was created or destroyed while the sky lost capacity"
        );
    }
    let field = world.weather();
    assert_eq!(
        field.raised(),
        field.air_total().0 + field.ground_total().0 + field.evaporated(),
        "the account is not exact after the storm drained"
    );
}

/// The air over a cell never stands above the capacity of that cell.
///
/// The capacity replaced one mark that every cell shared, so the bound is now
/// a different figure for each cell. A cell above its own capacity would
/// paint as more than a whole sky, and the excess would be water that the
/// field forgot to rain out.
#[test]
fn the_air_never_stands_above_the_capacity_of_its_own_cell() {
    let mut world = World::with_weather_scale(
        WorldConfig {
            width: WET_EXTENT,
            height: WET_EXTENT,
            seed: WET_SEED,
            faction_count: 2,
            unit_capacity: 1024,
        },
        weather::WeatherScale::PER_TILE,
    )
    .expect("the extent must describe a world");
    for _ in 0..20 {
        world.step(4).expect("the step must run");
    }
    let field = world.weather();
    let mut worst = 0i64;
    let mut worst_cell = 0u32;
    for cell in 0..field.air_plane().len() as u32 {
        let over = field.air_at(cell).0 - field.capacity_at_cell(cell).0;
        if over > worst {
            worst = over;
            worst_cell = cell;
        }
    }
    assert!(
        worst <= 0,
        "cell {worst_cell} stands {worst} drops above its own capacity"
    );
    // The share that the overlay paints follows from the same pair, so it
    // never passes a whole sky.
    let highest = (0..field.air_plane().len() as u32)
        .map(|cell| field.cloud_share_at(cell))
        .max()
        .unwrap_or(0);
    assert!(
        highest <= weather::CLOUD_SHARE_WHOLE,
        "a cell painted {highest} of a whole sky of {}",
        weather::CLOUD_SHARE_WHOLE
    );
}

/// The inland high latitudes hold cloud.
///
/// **This is the acceptance test the owner asked for.** He watches at one
/// cell for each tile, and he reports that the inland north and the inland
/// south never hold cloud. Cloud happened only over water and near the
/// equator. The cause was structural. The lift rose with the heat, the fall
/// rose with the cold, and the capacity was one constant. Water could only be
/// created near the sun, and it was destroyed as soon as it travelled toward
/// a pole.
///
/// The test reads the two polar bands of rows, over the land cells alone, and
/// asks whether a watcher would see a sky there. **The bars are set against a
/// measurement of the same world under the old rule**, which held a mean of
/// five parts of a sky in 255 and left 97 cells in every hundred blank. The
/// commit that made this change holds both readings.
///
/// **The fixture is a world whose polar rows hold open sea, and the seed is
/// its own.** The published saturation curve carries the water poleward and
/// rains most of it out on the way, so a polar continental interior is a
/// desert and holds no sky whatever the model does. That is the published
/// behaviour and not a defect. The assertion here is about whether the model
/// destroys the water that does arrive, so the fixture must supply water at
/// the pole.[^1]
///
/// **This asserts a joint property and isolates nothing.** The capacity, the
/// lift and the water the ground gives back all reach it, and putting any one
/// of them back alone does not always fail it. Three other tests carry the
/// capacity rule on its own, and each of them fails when the capacity goes
/// back to one constant.
///
/// # References
///
/// [^1]: Testing Rules, section 2a. `.agents/rules/testing.md`
#[test]
fn the_inland_high_latitudes_hold_cloud() {
    let mut world = World::with_weather_scale(
        WorldConfig {
            width: WET_EXTENT,
            height: WET_EXTENT,
            seed: POLAR_SEA_SEED,
            faction_count: 2,
            unit_capacity: 1024,
        },
        weather::WeatherScale::PER_TILE,
    )
    .expect("the extent must describe a world");
    // **The field must be given time to spin up.** The world starts dry, the
    // sea fills its own sky one frame in two, and the transport then has to
    // carry that water inland. A short run measures the fixture warming up
    // and not the field it settles into.
    for _ in 0..SPIN_UP_TICKS {
        world.step(4).expect("the step must run");
    }
    let field = world.weather();
    // **The walk goes over the world and not over the whole lattice.** The
    // lattice carries a margin of cells that no watcher sees, and a plane is
    // indexed by the whole lattice. A walk that reads a plane at a world
    // address therefore reads the wrong cell by the margin width, and it
    // counts margin cells as polar land.
    let lattice = field.lattice();
    let inner = lattice.inner();
    let high = inner.height();
    // A polar band is the outer tenth of the rows at each end. The sun swings
    // along the row axis, so the first row and the last row are the poles.
    let mut polar: Vec<i64> = Vec::new();
    for index in 0..inner.tile_count() {
        let Some(address) = inner.address_of(cachette_core::TileIdx(index)) else {
            continue;
        };
        let Some(cell) = lattice.whole_of_inner(index) else {
            continue;
        };
        let row = address.r.max(0) as u32;
        if row * 10 >= high && (row + 1) * 10 <= high * 9 {
            continue;
        }
        let Some(tile) = world.tile_terrain(address) else {
            continue;
        };
        if !tile.kind.is_passable() {
            continue;
        }
        polar.push(field.cloud_share_at(cell));
    }
    assert!(polar.len() > 32, "the fixture holds no polar land");
    let mean = polar.iter().sum::<i64>() / polar.len() as i64;
    let blank = polar
        .iter()
        .filter(|share| **share * 16 < weather::CLOUD_SHARE_WHOLE)
        .count();
    assert!(
        mean * 16 >= weather::CLOUD_SHARE_WHOLE,
        "the polar land holds a mean of {mean} of a whole sky of {}",
        weather::CLOUD_SHARE_WHOLE
    );
    assert!(
        blank * 4 <= polar.len() * 3,
        "{blank} of {} polar land cells hold under a sixteenth of a sky, mean {mean}",
        polar.len()
    );
}

/// The ticks that the acceptance fixture runs before it reads the field.
const SPIN_UP_TICKS: usize = 300;

/// A seed whose first and last rows hold open sea.
///
/// **A polar cloud test needs a polar sea.** The published saturation curve
/// makes the air of a cold cell hold very little, so the poleward transport
/// rains out most of what it carries and a landlocked polar interior stays
/// dry. That is a polar desert, which is a real climate, so the fixture that
/// asks about polar cloud must supply water at the pole.[^1]
///
/// # References
///
/// [^1]: Testing Rules, section 2a. `.agents/rules/testing.md`
const POLAR_SEA_SEED: u64 = 0x2f;

/// The saturation curve is the published one, at the published temperatures.
///
/// **The report states the capacity at eight temperatures, and this test
/// holds every one of them.** The bar is a twentieth of one part in a
/// hundred, which is far under the quantisation of a whole drop at the cold
/// end, so the rows there carry a bar of one drop instead.
///
/// The earlier curve doubled the capacity at a fixed temperature step. That
/// curve fails this test at every row, because the published doubling width
/// runs from about seven kelvin at the cold end to about fourteen at the warm
/// end.
///
/// # References
///
/// [^1]: Research report 30, the published atmospheric math, section 1.5. `docs/research/reports/30-the-published-atmospheric-math.md`
#[test]
fn the_saturation_curve_holds_the_published_table() {
    // The warmth, and the drops the published curve gives at it.
    let published = [
        (0u32, 2i64),
        (20, 6),
        (60, 64),
        (100, 422),
        (140, 2048),
        (180, 7822),
        (220, 24720),
        (255, 59672),
    ];
    for (warmth, asked) in published {
        let held = weather::capacity_at(warmth as i32).0;
        // A tenth of one part in a hundred, and never under one drop.
        let bar = (asked / 1000).max(1);
        assert!(
            (held - asked).abs() <= bar,
            "the capacity at warmth {warmth} is {held} drops, and the published curve gives {asked}"
        );
    }
}

/// The saturation curve does not double at a fixed temperature step.
///
/// **This is the claim the published curve refutes.** An earlier design held
/// that the capacity doubles at one fixed step, and built its integer form on
/// that. The published doubling width runs from about seven kelvin to about
/// fourteen over the range, which is a factor above two.
///
/// # References
///
/// [^1]: Research report 30, the published atmospheric math, section 1.3. `docs/research/reports/30-the-published-atmospheric-math.md`
#[test]
fn the_saturation_curve_doubles_at_no_fixed_step() {
    // Twenty degrees of warmth is ten kelvin on the declared scale.
    let step = 20;
    let ratio = |warmth: i32| -> i64 {
        let low = weather::capacity_at(warmth).0.max(1);
        let high = weather::capacity_at(warmth + step).0;
        high * 1000 / low
    };
    let cold = ratio(60);
    let warm = ratio(220);
    assert!(
        cold > warm * 3 / 2,
        "ten kelvin multiplies the capacity by {cold} in thousandths at the cold end \
         and by {warm} at the warm end, so a fixed doubling would fit"
    );
}

/// The capacity never stands above the ceiling of the plane.
#[test]
fn the_capacity_never_passes_the_ceiling() {
    for warmth in -8..=(cachette_core::HEAT_CEILING + 8) {
        let held = weather::capacity_at(warmth).0;
        assert!(
            held >= 2 && held <= weather::AIR_SATURATION.0,
            "the capacity at warmth {warmth} is {held} drops"
        );
    }
}

/// The insolation holds the polar day, so at the solstice the pole takes more
/// daily energy than the equator.
///
/// **No function of the distance from where the sun stands can hold this
/// shape.** Such a function peaks under the sun and falls away from it. The
/// published geometry rises again toward the summer pole, because the day
/// there never ends. The earlier model was a distance function, and it gave a
/// pole that was cold in every part of the year.
///
/// The test holds the integer form against the published table, at six
/// latitudes and three points of the year. Every entry is in watts for each
/// square metre.
///
/// # References
///
/// [^1]: Research report 30, the published atmospheric math, section 4.3. `docs/research/reports/30-the-published-atmospheric-math.md`
#[test]
fn the_insolation_holds_the_published_table() {
    let obliquity = 2344;
    // The latitude in whole degrees, then the June solstice, the equinox and
    // the December solstice.
    let published = [
        (0i32, 397i64, 433i64, 397i64),
        (20, 470, 407, 285),
        (40, 498, 331, 150),
        (60, 492, 216, 23),
        (80, 533, 75, 0),
        (90, 541, 0, 0),
    ];
    for (degrees, june, equinox, december) in published {
        let latitude = degrees * weather::LATITUDE_FINE;
        for (declination, asked) in [(obliquity, june), (0, equinox), (-obliquity, december)] {
            let held = weather::daily_insolation(latitude, declination);
            assert!(
                (held - asked).abs() <= 2,
                "the insolation at {degrees} degrees at a declination of {declination} \
                 is {held} watts, and the published table gives {asked}"
            );
        }
    }
    // The claim that no distance function can hold: the solstice pole stands
    // above the solstice equator.
    let pole = weather::daily_insolation(90 * weather::LATITUDE_FINE, obliquity);
    let equator = weather::daily_insolation(0, obliquity);
    assert!(
        pole * 100 > equator * 130,
        "the solstice pole takes {pole} watts and the equator {equator}, \
         so the polar day is missing"
    );
}

/// The pole holds a summer and a winter, and they are half a year apart.
#[test]
fn each_pole_holds_a_summer_of_its_own() {
    let quarter = weather::SEASON_PERIOD_TICKS as u64 / 4;
    let pole = 90 * weather::LATITUDE_FINE;
    let summer = weather::season_at(Tick(quarter), pole);
    let winter = weather::season_at(Tick(3 * quarter), pole);
    assert!(
        summer > winter + 60,
        "the pole read {summer} in its summer and {winter} in its winter"
    );
    // The two poles are opposite. One is in summer while the other is in
    // winter.
    let other = weather::season_at(Tick(quarter), -pole);
    assert!(
        summer > other + 60,
        "the two poles read {summer} and {other} at the same tick"
    );
}

/// The annual mean of the sun term falls from the equator to the pole.
///
/// This is the belt of a latitude, and it is what puts a warm middle and cold
/// ends on the map. It is a separate claim from the season, which reverses
/// across the equator.
#[test]
fn the_sun_term_falls_from_the_equator_to_the_pole() {
    let over_a_year = |latitude: i32| -> i64 {
        (0..weather::SEASON_PERIOD_TICKS as u64)
            .map(|tick| i64::from(weather::season_at(Tick(tick), latitude)))
            .sum()
    };
    let readings: Vec<i64> = (0..=6)
        .map(|step| over_a_year(step * 15 * weather::LATITUDE_FINE))
        .collect();
    for pair in readings.windows(2) {
        assert!(
            pair[0] >= pair[1],
            "the sun term rises toward the pole: {readings:?}"
        );
    }
    assert!(
        readings[0] > readings[6],
        "the sun term is flat from the equator to the pole: {readings:?}"
    );
}

/// The banded pressure holds three extremes in each hemisphere.
///
/// **Three belts need three pressure extremes, and one temperature profile
/// that falls from the equator to the pole has two.** So the field imposes
/// them: a low at the equator, a high at thirty degrees, a low at sixty and a
/// high at each pole. That is the published order, and the high at thirty is
/// what makes a desert belt.
///
/// # References
///
/// [^1]: Research report 30, the published atmospheric math, sections 5.1 and 5.3. `docs/research/reports/30-the-published-atmospheric-math.md`
#[test]
fn the_banded_pressure_holds_the_published_belts() {
    let at = |degrees: i32| weather::band_pressure_at(degrees * weather::LATITUDE_FINE);
    let equator = at(0);
    let subtropics = at(30);
    let temperate = at(60);
    let pole = at(90);
    assert!(
        subtropics > equator,
        "the subtropics hold {subtropics} and the equator {equator}, so no high stands at thirty"
    );
    assert!(
        subtropics > temperate,
        "the subtropics hold {subtropics} and sixty degrees {temperate}, so no low stands at sixty"
    );
    assert!(
        pole > temperate,
        "the pole holds {pole} and sixty degrees {temperate}, so no high stands at the pole"
    );
    // The two hemispheres carry the same shape, because the offset reads a
    // cosine of the latitude and a cosine is even.
    for degrees in [0, 15, 30, 45, 60, 75, 90] {
        assert_eq!(
            at(degrees),
            at(-degrees),
            "the two hemispheres differ at {degrees} degrees"
        );
    }
}

/// A narrow latitude span gives a flat sun term.
///
/// **The region reading and the planet reading differ by one constant and not
/// by a model.** A world that spans three degrees holds one belt, and every
/// latitude term goes flat inside it. A world that spans the globe holds all
/// of them. The same code answers both.
///
/// # References
///
/// [^1]: Research report 30, the published atmospheric math, section 9. `docs/research/reports/30-the-published-atmospheric-math.md`
#[test]
fn a_narrow_span_gives_a_flat_latitude_term() {
    let height = 128;
    let region = weather::Latitudes::new(45 * weather::LATITUDE_FINE, 3 * weather::LATITUDE_FINE)
        .expect("three degrees fits on the globe");
    let planet = weather::Latitudes::PLANET;
    let spread = |latitudes: weather::Latitudes| -> i32 {
        let readings: Vec<i32> = (0..height)
            .map(|row| weather::season_at(Tick(0), latitudes.of_row(row, height)))
            .collect();
        readings.iter().copied().max().unwrap_or(0) - readings.iter().copied().min().unwrap_or(0)
    };
    let across_a_region = spread(region);
    let across_a_planet = spread(planet);
    assert!(
        across_a_region <= 6,
        "a three degree span spread the sun term over {across_a_region} degrees of heat"
    );
    assert!(
        across_a_planet > 8 * across_a_region.max(1),
        "a whole planet spread it over {across_a_planet}, against {across_a_region} for a region"
    );
    // **A region holds no belt of its own, because no belt is that narrow.**
    // The banded pressure across three degrees is one steady slope, so the
    // region carries one prevailing wind rather than three. The reading is
    // monotone across the world, and the planet reading is not.
    let belts = |latitudes: weather::Latitudes| -> Vec<i32> {
        (0..height)
            .map(|row| weather::band_pressure_at(latitudes.of_row(row, height)))
            .collect()
    };
    // A turn is a change of sign in the slope. The slope is zero over a run
    // of rows at each extreme, because the reading is a whole number, so the
    // walk carries the last slope it saw rather than reading one pair.
    let turns = |readings: &[i32]| -> usize {
        let mut seen = 0i32;
        let mut count = 0usize;
        for pair in readings.windows(2) {
            let slope = (pair[1] - pair[0]).signum();
            if slope == 0 {
                continue;
            }
            if seen != 0 && slope != seen {
                count += 1;
            }
            seen = slope;
        }
        count
    };
    let across_a_region = turns(&belts(region));
    let across_a_planet = turns(&belts(planet));
    assert_eq!(
        across_a_region, 0,
        "a three degree span holds a turn in the banded pressure, so it holds a belt boundary"
    );
    assert!(
        across_a_planet >= 4,
        "a whole planet holds {across_a_planet} turns in the banded pressure, and it needs six"
    );
}

/// A row of the world carries a latitude, and the two ends carry the two ends
/// of the span.
#[test]
fn the_rows_of_a_world_cover_its_latitude_span() {
    let height = 64;
    let latitudes = weather::Latitudes::PLANET;
    let readings: Vec<i32> = (0..height)
        .map(|row| latitudes.of_row(row, height))
        .collect();
    for pair in readings.windows(2) {
        assert!(pair[0] < pair[1], "the latitude does not rise with the row");
    }
    let low = readings[0];
    let high = readings[height as usize - 1];
    assert!(
        low < -85 * weather::LATITUDE_FINE && high > 85 * weather::LATITUDE_FINE,
        "the rows ran from {low} to {high}, which is not pole to pole"
    );
    // A world that states a centre stands there.
    let region = weather::Latitudes::new(-40 * weather::LATITUDE_FINE, 4 * weather::LATITUDE_FINE)
        .expect("four degrees at forty south fits on the globe");
    let middle = region.of_row(height / 2, height);
    assert!(
        (middle + 40 * weather::LATITUDE_FINE).abs() <= weather::LATITUDE_FINE,
        "the middle row of a region centred at forty south stands at {middle}"
    );
}

/// A span that does not fit on the globe is refused.
#[test]
fn a_span_that_passes_a_pole_is_refused() {
    assert!(weather::Latitudes::new(0, 2 * weather::LATITUDE_POLE + 1).is_err());
    assert!(weather::Latitudes::new(0, -1).is_err());
    assert!(
        weather::Latitudes::new(80 * weather::LATITUDE_FINE, 40 * weather::LATITUDE_FINE).is_err()
    );
    assert!(
        weather::Latitudes::new(80 * weather::LATITUDE_FINE, 4 * weather::LATITUDE_FINE).is_ok()
    );
}

/// The banded pressure is a table over the world rows, read in the space of
/// the whole lattice.
///
/// **A whole-lattice index and a world address are different things.** The
/// lattice carries a margin, so a reader that takes one for the other samples
/// the wrong cells. That mistake cost a day of wrong diagnosis once, and the
/// findings register holds it.[^1]
///
/// The test drives the engine, then reads the offset of each cell of the
/// whole lattice against the offset that the latitude of its own world row
/// gives.
///
/// # References
///
/// [^1]: Findings register, FND-569. `docs/FINDINGS.md`
#[test]
fn the_banded_pressure_follows_the_world_row_and_not_the_lattice_row() {
    let mut world = World::with_weather_scale(
        WorldConfig {
            width: 64,
            height: 64,
            seed: POLAR_SEA_SEED,
            faction_count: 1,
            unit_capacity: 64,
        },
        weather::WeatherScale::PER_TILE,
    )
    .expect("the extent must describe a world");
    world.step(1).expect("the step must run");
    let field = world.weather();
    assert!(
        field.lattice().ring() > 0,
        "the fixture carries no margin, so it cannot catch the mistake"
    );
    let mut spread = 0;
    let mut lowest = i32::MAX;
    let mut highest = i32::MIN;
    for cell in 0..field.cells().tile_count() {
        let asked = weather::band_pressure_at(field.latitude_at(cell));
        let held = field.band_at(cell);
        assert_eq!(
            held, asked,
            "cell {cell} holds a banded pressure of {held}, and its own latitude asks {asked}"
        );
        lowest = lowest.min(held);
        highest = highest.max(held);
    }
    spread += highest - lowest;
    assert!(
        spread > 0,
        "the banded pressure is flat over the whole world, so nothing imposed a belt"
    );
    // A margin cell beyond a pole carries the latitude of the pole, in the
    // same way that it carries the temperature term of the pole.
    let inner = field.lattice().inner();
    let first = field
        .lattice()
        .whole_of_inner(0)
        .expect("the world holds a first cell");
    let corner = field
        .cells()
        .index_of(cachette_core::hex::Axial::new(0, 0))
        .expect("the lattice holds its own corner");
    assert_eq!(
        field.latitude_at(corner.0),
        field.latitude_at(first),
        "a margin cell beyond the pole reads a latitude that no world row holds"
    );
    assert!(inner.height() > 0);
}

/// Returns the highest and the lowest sun term over the globe and the year.
fn sun_reach() -> (i32, i32) {
    let mut top = i32::MIN;
    let mut floor = i32::MAX;
    for degree in -90..=90 {
        let latitude = degree * weather::LATITUDE_FINE;
        for tick in 0..weather::SEASON_PERIOD_TICKS as u64 {
            let value = weather::season_at(Tick(tick), latitude);
            top = top.max(value);
            floor = floor.min(value);
        }
    }
    (top, floor)
}

/// The sun term reaches the whole swing the heat scale reserves, at each end.
///
/// **This is the test that a written normaliser fails.** The belt peaks at the
/// equator, where the season is near nothing. The season peaks at the middle
/// latitudes, where the belt is near nothing. So the sum of the two amplitudes
/// names a swing that no place and no moment holds. The term therefore
/// normalises the sum against the reach that its own geometry gives, and that
/// reach maps onto the reserved swing exactly.
#[test]
fn the_sun_term_reaches_the_whole_swing_that_the_scale_reserves() {
    let (top, floor) = sun_reach();
    assert_eq!(
        top,
        weather::SEASON_SWING,
        "the warmest place at the warmest moment reads {top} degrees of sun"
    );
    assert_eq!(
        floor,
        -weather::SEASON_SWING,
        "the coldest place at the coldest moment reads {floor} degrees of sun"
    );
}

/// The warmest sun stands in the subtropics and not at the equator.
///
/// The published geometry puts the highest daily energy of the year over the
/// summer subtropics. The belt is still strong there and the season is near
/// its own peak. A term that peaked at the equator would put the hottest
/// ground of the world in the rain belt.
#[test]
fn the_warmest_sun_stands_between_the_equator_and_the_middle_latitude() {
    let over_a_year = |latitude: i32| -> i32 {
        (0..weather::SEASON_PERIOD_TICKS as u64)
            .map(|tick| weather::season_at(Tick(tick), latitude))
            .max()
            .unwrap_or(0)
    };
    let equator = over_a_year(0);
    let subtropics = over_a_year(33 * weather::LATITUDE_FINE);
    let far = over_a_year(60 * weather::LATITUDE_FINE);
    assert!(
        subtropics > equator && subtropics > far,
        "the sun reads {equator} at the equator, {subtropics} in the subtropics \
         and {far} at sixty degrees"
    );
}

/// A world that spans three degrees reads a flat slice of the same sun.
///
/// **The normalisers are properties of the globe and not of the world.** A
/// normaliser read from the world would stretch a four percent change across
/// the whole scale, and a narrow world would then hold every climate zone
/// inside three degrees. The reach that the normaliser divides by comes from
/// the whole globe, so a narrow span keeps a narrow spread and no constant
/// moves with the span.
#[test]
fn a_narrow_span_reads_a_flat_slice_of_the_same_sun() {
    let region = weather::Latitudes::new(45 * weather::LATITUDE_FINE, 3 * weather::LATITUDE_FINE)
        .expect("three degrees fits on the globe");
    let rows = 64u32;
    let mut top = i32::MIN;
    let mut floor = i32::MAX;
    for row in 0..rows {
        let value = weather::season_at(Tick(0), region.of_row(row, rows));
        top = top.max(value);
        floor = floor.min(value);
    }
    assert!(
        top - floor < weather::SEASON_SWING / 4,
        "a three degree world spreads the sun by {}, and the globe reserves {}",
        top - floor,
        weather::SEASON_SWING
    );
}

/// A full sky takes away what the published effect of cloud is worth on the
/// scale of the sun.
///
/// **The cloud swing is derived and nothing writes it down.** A cloud that
/// shades the ground is worth what the sun it shades is worth. This test
/// measures what one watt of insolation is worth in degrees of warmth, through
/// the public terms alone, and asks that a full sky match the published effect
/// of cloud on that scale.
///
/// The measurement is a slope between two latitudes, and the annual mean of
/// the season part is not exactly nothing at either one. So the test holds a
/// band rather than an equality. A figure written down in place of the
/// derivation was outside the band by a factor of about three, which is the
/// error this test exists to catch.
#[test]
fn a_full_sky_is_worth_the_published_effect_of_cloud() {
    let ticks = weather::SEASON_PERIOD_TICKS as u64;
    // The sun term summed over one year, at one latitude.
    let sun_over_a_year = |latitude: i32| -> i64 {
        (0..ticks)
            .map(|tick| i64::from(weather::season_at(Tick(tick), latitude)))
            .sum()
    };
    // The published insolation summed over the same year, at the same
    // latitude. Both sums walk the same ticks, so the tick count cancels.
    let watts_over_a_year = |latitude: i32| -> i64 {
        (0..ticks)
            .map(|tick| weather::daily_insolation(latitude, weather::declination_at(Tick(tick))))
            .sum()
    };
    let warm = 0;
    let cool = 30 * weather::LATITUDE_FINE;
    let degrees = sun_over_a_year(warm) - sun_over_a_year(cool);
    let watts = watts_over_a_year(warm) - watts_over_a_year(cool);
    assert!(
        degrees > 0 && watts > 0,
        "the equator must hold more sun and more watts than thirty degrees"
    );

    // The published net effect of cloud is near twenty watts for each square
    // metre, over a cover near sixty-eight hundredths. A whole sky is the one
    // divided by the other.
    let full_sky = 20 * 100 / 68;
    let asked = full_sky * degrees / watts;
    let held = i64::from(weather::cloud_at(
        weather::Drops(1024),
        weather::Drops(1024),
    ));
    assert!(
        held * 4 >= asked * 3 && held * 3 <= asked * 4,
        "a full sky takes {held} degrees, and the published effect on the scale \
         of the sun asks near {asked}"
    );
}
