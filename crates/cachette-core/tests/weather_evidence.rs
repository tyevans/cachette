//! Weather evidence: the five assertions that items 0499, 0500 and 0501 asked for.
//!
//! Three backlog items closed with their engine implementation finished and the
//! physical evidence they specified omitted.[^1] [^2] [^3] This test suite supplies
//! that evidence.
//!
//! # The Five Invariants
//!
//! 1. The wind lags the pressure: a cell whose pressure gradient falls to zero
//!    keeps a non-zero wind on the following tick.[^4]
//! 2. Drag brings wind to rest: under zero pressure gradient, drag monotonically
//!    reduces wind speed to zero within a finite tick bound.[^5]
//! 3. Advection conserves total air: over arbitrary random winds and random air
//!    planes, directional transport conserves the drop count exactly.[^6]
//! 4. Rain arrives heterogeneously: without an imposed storm, a coastal world
//!    holds both wet and dry cells simultaneously at a single tick.[^7]
//! 5. Orographic precipitation: air climbing high ground sheds capacity, leaving
//!    more ground water on the windward side than on the lee side.[^7]
//!
//! # References
//!
//! [^1]: Backlog item 0499. `docs/backlog/complete/0499-give-every-level-1-cell-a-wind-that-carries-its-momentum.md`
//! [^2]: Backlog item 0500. `docs/backlog/complete/0500-carry-the-air-water-along-the-wind.md`
//! [^3]: Backlog item 0501. `docs/backlog/complete/0501-make-rain-arrive-rather-than-sit-everywhere.md`
//! [^4]: ADR-0160, the wind is carried state, and the pressure gradient accelerates it, decision D2. `docs/adrs/accepted/adr-0160-the-wind-is-carried-state-and-the-pressure-gradient-accelerates-it.md`
//! [^5]: ADR-0160, the wind is carried state, and the pressure gradient accelerates it, decision D3. `docs/adrs/accepted/adr-0160-the-wind-is-carried-state-and-the-pressure-gradient-accelerates-it.md`
//! [^6]: ADR-0161, water rides the wind, and every transfer is an exact integer move, decision D1. `docs/adrs/accepted/adr-0161-water-rides-the-wind-and-every-transfer-is-an-exact-integer-move.md`
//! [^7]: ADR-0162, water enters the air where it is hot, and it falls where the air cools, decisions D1 and D2. `docs/adrs/accepted/adr-0162-water-enters-the-air-where-it-is-hot-and-falls-where-the-air-cools.md`

use cachette_core::hex::{Axial, Grid};
use cachette_core::padded::PaddedLattice;
use cachette_core::terrain::{TerrainTile, TileKind};
use cachette_core::types::{Fix32, Tick};
use cachette_core::weather::{
    drag, CellGround, CycloneSetting, Drops, Latitudes, WeatherField, WeatherScale, Wind,
    CYCLONE_DEPTH_CEILING, SPEED_CEILING,
};
use cachette_core::world::{World, WorldConfig};
use proptest::prelude::*;

/// Builds a weather field with flat latitudes and uniform zero elevation.
///
/// Under this field, temperature and banded circulation offsets are uniform, so
/// the natural pressure gradient across every cell is zero.
fn flat_field(extent: u32) -> (WeatherField, Vec<CellGround>) {
    let grid = Grid::new(extent, extent).expect("grid builds");
    let lattice = PaddedLattice::new(grid, 1).expect("lattice builds");
    let field = WeatherField::with_latitudes(
        lattice,
        WeatherScale::PER_TILE,
        Latitudes::new(0, 0).expect("flat latitudes build"),
        1,
    )
    .expect("field builds");
    let cells = field.cells().tile_count() as usize;
    let ground = vec![CellGround::EMPTY; cells];
    (field, ground)
}

/// 1. The wind lags the pressure.
///
/// A cell whose pressure difference falls to nothing keeps a non-zero wind on
/// the next tick.
///
/// A storm is raised at tick 0 to create a strong local pressure deficit and
/// accelerate the wind around the eye. On tick 1, the storm expires (life = 1),
/// removing the deficit. The pressure gradient across the neighbour cell drops
/// to zero. Because wind is carried state, the neighbour cell retains non-zero
/// wind on the following tick.
#[test]
fn wind_lags_pressure() {
    let (mut field, ground) = flat_field(12);
    let eye_address = Axial::new(6, 6);
    let eye_idx = field
        .cells()
        .index_of(eye_address)
        .expect("eye address must be inside the lattice");

    field
        .raise_cyclone(
            eye_idx.0,
            CycloneSetting {
                depth: CYCLONE_DEPTH_CEILING,
                radius: 1,
                life: 2,
            },
        )
        .expect("the cyclone must be accepted");

    // Tick 0: storm stamps its deficit; neighbours accelerate under pressure difference.
    field
        .solve(Tick(0), 1, &ground, 1)
        .expect("solve must succeed");

    let neighbour_address = field
        .cells()
        .neighbour(eye_address, 0)
        .expect("neighbour exists");
    let neighbour_idx = field
        .cells()
        .index_of(neighbour_address)
        .expect("neighbour index exists")
        .0;

    let wind_at_storm = field.wind_at(neighbour_idx);
    assert_ne!(
        wind_at_storm,
        Wind::STILL,
        "wind must accelerate under the pressure deficit of the storm"
    );
    assert!(
        wind_at_storm.speed() > 0,
        "wind speed must be positive during the storm"
    );

    // Tick 1: storm has expired. The depression is zero everywhere.
    // In a flat field with uniform ground, the pressure difference across
    // the neighbour cell is now zero.
    field
        .solve(Tick(1), 1, &ground, 1)
        .expect("solve must succeed");

    let wind_after_gradient_fell = field.wind_at(neighbour_idx);
    assert_ne!(
        wind_after_gradient_fell,
        Wind::STILL,
        "wind must lag pressure: momentum persists when the pressure gradient falls to zero"
    );
    assert!(
        wind_after_gradient_fell.speed() > 0,
        "wind speed must remain positive on the tick following gradient cessation"
    );
}

/// 2. Drag brings a wind to rest.
///
/// Under zero pressure gradient, drag bleeds momentum each tick. Speed decreases
/// monotonically and reaches [`Wind::STILL`] within a finite tick count.
#[test]
fn drag_brings_wind_to_rest() {
    let (mut field, ground) = flat_field(12);
    let eye_address = Axial::new(6, 6);
    let eye_idx = field
        .cells()
        .index_of(eye_address)
        .expect("eye address must be inside the lattice");

    field
        .raise_cyclone(
            eye_idx.0,
            CycloneSetting {
                depth: CYCLONE_DEPTH_CEILING,
                radius: 1,
                life: 2,
            },
        )
        .expect("the cyclone must be accepted");

    // Step tick 0 to build initial wind.
    field
        .solve(Tick(0), 1, &ground, 1)
        .expect("solve must succeed");

    let mut prev_fastest = field.fastest();
    assert!(prev_fastest > 0, "field must have non-zero wind initially");

    let mut settled_at: Option<u64> = None;
    for tick in 1..=32 {
        field
            .solve(Tick(tick), 1, &ground, 1)
            .expect("solve must succeed");
        let fastest = field.fastest();
        assert!(
            fastest <= prev_fastest,
            "speed must monotonically decrease under zero gradient: {fastest} > {prev_fastest} at tick {tick}"
        );
        prev_fastest = fastest;
        if fastest == 0 {
            settled_at = Some(tick);
            break;
        }
    }

    assert!(
        settled_at.is_some(),
        "drag must bring the wind to complete rest within 32 ticks"
    );
    assert_eq!(
        field.fastest(),
        0,
        "every cell of the field must have settled to Wind::STILL"
    );
}

/// Drag operator brings any vector to rest monotonically.
#[test]
fn drag_operator_settles_all_valid_speeds() {
    for q in [
        -SPEED_CEILING,
        -SPEED_CEILING / 2,
        0,
        SPEED_CEILING / 2,
        SPEED_CEILING,
    ] {
        for r in [
            -SPEED_CEILING,
            -SPEED_CEILING / 2,
            0,
            SPEED_CEILING / 2,
            SPEED_CEILING,
        ] {
            let mut wind = Wind { q, r };
            let mut prev_speed = wind.speed();
            let mut steps = 0;
            while !wind.is_still() && steps < 64 {
                wind = drag(wind);
                let speed = wind.speed();
                assert!(
                    speed < prev_speed,
                    "drag must strictly reduce speed until rest: {speed} >= {prev_speed}"
                );
                prev_speed = speed;
                steps += 1;
            }
            assert_eq!(wind, Wind::STILL, "drag must reach STILL from ({q}, {r})");
        }
    }
}

// 3. The air total is unchanged over random winds and random air planes.
//
// Property test: advective transport over the lattice conserves the sum of air
// drops exactly across arbitrary winds and air distributions.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]
    #[test]
    fn air_total_conserved_over_random_winds_and_air_planes(
        air_values in prop::collection::vec(0i64..=1_000_000, 64),
        wind_q in prop::collection::vec(-SPEED_CEILING..=SPEED_CEILING, 64),
        wind_r in prop::collection::vec(-SPEED_CEILING..=SPEED_CEILING, 64),
    ) {
        let extent = 8;
        let grid = Grid::new(extent, extent).expect("grid builds");
        let lattice = PaddedLattice::new(grid, 0).expect("lattice builds");
        let mut field = WeatherField::new(lattice, WeatherScale::PER_TILE, 1).expect("field builds");

        let drops: Vec<Drops> = air_values.iter().copied().map(Drops).collect();
        let winds: Vec<Wind> = wind_q
            .iter()
            .zip(wind_r.iter())
            .map(|(&q, &r)| Wind { q, r })
            .collect();

        field.set_air_plane(&drops);
        field.set_wind_plane(&winds);

        let total_before = field.air_total().0;
        field.transport(1);
        let total_after = field.air_total().0;

        prop_assert_eq!(
            total_before,
            total_after,
            "air drops must be exactly conserved across transport: before={}, after={}",
            total_before,
            total_after
        );
    }
}

/// 4. Some cells are wet and some are dry at one tick, with no storm raised.
///
/// A coastal world develops an uneven rain distribution through solar heating,
/// evaporation and cooling. At a single settled tick, the map is neither entirely
/// wet nor entirely dry.
#[test]
fn some_cells_wet_and_some_dry_without_storm() {
    let mut world = World::with_weather_scale(
        WorldConfig {
            width: 48,
            height: 48,
            seed: 0x0123_4567_89ab_cdef,
            faction_count: 2,
            unit_capacity: 1024,
            ..WorldConfig::DEFAULT
        },
        WeatherScale::PER_TILE,
    )
    .expect("world builds");

    for _ in 0..48 {
        world.step(1).expect("world steps");
    }

    let wet_cells = world.weather().wet_cells();
    let total_cells = world.weather().lattice().inner().tile_count();

    assert!(
        wet_cells > 0,
        "some cells must be wet in a coastal world with rainfall: wet={wet_cells}"
    );
    assert!(
        wet_cells < total_cells,
        "some cells must remain dry, but all {total_cells} cells are wet"
    );
}

/// 5. The near side of high ground holds more water than the far side.
///
/// Under a steady wind blowing across a ridge, air climbing the windward slope
/// sheds capacity and pours water onto the near side. Descending air on the lee
/// slope warms and holds its moisture, leaving the far side with less ground
/// water.
#[test]
fn near_side_of_high_ground_holds_more_water_than_far_side() {
    // Build a 1-D profile along lattice axis 0 (direction 0: Axial(1, 0)):
    // - Column 0 (x=2): Upwind low ground (height 0)
    // - Column 1 (x=3): Windward slope / high ridge (height 1)
    // - Column 2 (x=4): Lee slope / far side low ground (height 0)
    let extent = 8;
    let (mut field, mut ground) = flat_field(extent);

    // Set ridge elevation: high ground at column 3.
    let ridge_tile = TerrainTile {
        height: Fix32::from_int(1),
        moisture: Fix32::ZERO,
        kind: TileKind::Plain,
    };
    for row in 0..field.cells().height() {
        if let Some(idx) = field.cells().index_of(Axial::new(3, row as i32)) {
            ground[idx.0 as usize] = CellGround::of_tile(ridge_tile);
        }
    }

    // Set steady wind blowing along direction 0 (positive q).
    let steady_wind = Wind {
        q: SPEED_CEILING / 2,
        r: 0,
    };
    let winds = vec![steady_wind; ground.len()];
    field.set_wind_plane(&winds);

    // Provide moisture in the air across all cells.
    let moist_air = vec![Drops(50_000); ground.len()];
    field.set_air_plane(&moist_air);

    // Run settle: climbing air sheds capacity on the windward slope.
    field.settle(&ground);

    let near_address = Axial::new(3, 4);
    let far_address = Axial::new(4, 4);
    let near_cell = field.cells().index_of(near_address).expect("near cell").0;
    let far_cell = field.cells().index_of(far_address).expect("far cell").0;

    let near_water = field.ground_at(near_cell).0;
    let far_water = field.ground_at(far_cell).0;

    assert!(
        near_water > far_water,
        "near side of high ground ({near_water} drops) must hold more water \
         than the far side ({far_water} drops) due to orographic precipitation"
    );
}
