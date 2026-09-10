//! The latitude span of a world, and what reads it.
//!
//! A map is one region of a planet. The settings state the latitude of the
//! middle row and the latitude from the first row to the last, both in
//! hundredths of a degree. The weather then reads the span rather than the
//! raw row.[^1]
//!
//! Every test here drives the world. None calls the weather solve and none
//! calls the climate spin with a span of its own, because the world is
//! obligated to hand both of them the span, and a test that handed it over
//! itself would prove only that the mechanism works.[^2]
//!
//! **The fixtures stand at latitudes that are far apart.** A pressure belt of
//! a planet is tens of degrees wide, so two spans three degrees apart would
//! read almost the same sky and the assertions would measure the fixture.[^3]
//!
//! # References
//!
//! [^1]: ADR-0177, the row axis of a world is a latitude that the world states, decision D1. `docs/adrs/draft/adr-0177-the-row-axis-of-a-world-is-a-latitude-that-the-world-states.md`
//! [^2]: Testing rules, section 5. `.claude/rules/testing.md`
//! [^3]: Testing rules, section 2a. `.claude/rules/testing.md`

use cachette_core::weather::{Latitudes, WeatherScale, LATITUDE_FINE, LATITUDE_POLE};
use cachette_core::{World, WorldConfig, WorldError};

/// The extent of every world below.
///
/// The world is square, so the row axis holds as many rows as the column axis
/// holds columns. A pole to pole span over this many rows puts about one and a
/// half degrees between two rows, which resolves every pressure belt.
const SIDE: u32 = 128;

/// The seed of every world below.
const SEED: u64 = 0x0C11_A7E0_0BAD_F00D;

/// Returns the settings of a world that states nothing about its latitudes.
fn config() -> WorldConfig {
    WorldConfig {
        width: SIDE,
        height: SIDE,
        seed: SEED,
        faction_count: 1,
        unit_capacity: 64,
        ..WorldConfig::DEFAULT
    }
}

/// Returns the banded pressure the engine holds, in world row order.
///
/// The reading walks one column of the world from the first row to the last.
/// It takes the value the weather field computed, so nothing here repeats the
/// belt arithmetic.
fn belts(world: &World) -> Vec<i32> {
    let field = world.weather();
    let lattice = field.lattice();
    let inner = lattice.inner();
    (0..inner.height())
        .map(|row| {
            let cell = lattice
                .whole_of_inner(row * inner.width())
                .expect("the world holds this cell");
            field.band_at(cell)
        })
        .collect()
}

/// Returns the number of times the slope of a reading changes sign.
///
/// A belt boundary is a turn. The slope is zero over a run of rows at each
/// extreme, because the reading is a whole number, so the walk carries the
/// last slope it saw rather than reading one pair.
fn turns(readings: &[i32]) -> usize {
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
}

/// A world that states no span holds no belt boundary.
///
/// **This is the symptom the span exists to remove.** A world that spanned
/// the globe put a low at the equator, a high at thirty degrees and a low at
/// sixty on every map, at the same rows every time. A region holds no belt of
/// its own, because no belt is three degrees wide, so the banded pressure runs
/// as one steady slope and the world carries one prevailing wind.
#[test]
fn a_world_that_states_no_span_holds_no_belt_boundary() {
    let world = World::with_weather_scale(config(), WeatherScale::PER_TILE)
        .expect("the extent must describe a world");
    let readings = belts(&world);
    assert!(
        readings.len() > 1,
        "the fixture holds {} rows, so it cannot hold a turn",
        readings.len()
    );
    assert_eq!(
        turns(&readings),
        0,
        "a world that states no span holds a turn in the banded pressure, \
         so it holds a belt boundary"
    );
}

/// A world that states the planet span still holds the belts.
///
/// The offset that imposes the three circulation cells stays in the engine. A
/// caller that wants a planet states the span from pole to pole and gets the
/// published belts.
#[test]
fn a_world_that_states_the_planet_span_holds_the_belts() {
    let world = World::with_weather_scale(
        WorldConfig {
            latitude_centre: Latitudes::PLANET.centre(),
            latitude_span: Latitudes::PLANET.span(),
            ..config()
        },
        WeatherScale::PER_TILE,
    )
    .expect("the extent must describe a world");
    assert!(
        turns(&belts(&world)) >= 4,
        "a whole planet holds {} turns in the banded pressure, and it needs six",
        turns(&belts(&world))
    );
}

/// The weather field of a world carries the span the settings stated.
#[test]
fn the_weather_field_carries_the_stated_span() {
    let settings = WorldConfig {
        latitude_centre: -60 * LATITUDE_FINE,
        latitude_span: 2 * LATITUDE_FINE,
        ..config()
    };
    let world =
        World::with_weather_scale(settings, WeatherScale::DEFAULT).expect("the world builds");
    let held = world.weather().latitudes();
    assert_eq!(held.centre(), settings.latitude_centre);
    assert_eq!(held.span(), settings.latitude_span);
}

/// The climate spin reads the span of the world it spins.
///
/// **The spin builds a weather field of its own.** That field is a second
/// place where a span could be stated, and nothing fails when two such places
/// disagree. So this test builds two worlds that differ in nothing but the
/// span, and asks that the two climates differ. A spin that reached for a
/// default would run one sky for both worlds and give one climate.[^1]
///
/// The two centres stand a hemisphere apart, so the sun over one is the
/// winter sun over the other.
///
/// # References
///
/// [^1]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
#[test]
fn the_climate_spin_reads_the_span_of_its_world() {
    // The spin discards its opening ticks as a transient, so a run shorter
    // than that records nothing at all.
    let ticks = 96;
    let north = World::with_climate(config(), WeatherScale::DEFAULT, ticks, 1)
        .expect("the northern world builds");
    let south = World::with_climate(
        WorldConfig {
            latitude_centre: -45 * LATITUDE_FINE,
            ..config()
        },
        WeatherScale::DEFAULT,
        ticks,
        1,
    )
    .expect("the southern world builds");
    assert!(
        !north.climate().is_quiet() && !south.climate().is_quiet(),
        "a spin stored nothing, so the fixture reaches no reading"
    );
    assert_ne!(
        north.climate().cells(),
        south.climate().cells(),
        "two worlds a hemisphere apart hold one climate, \
         so the spin does not read the span of its world"
    );
}

/// A span that does not fit on the globe is a refusal and not a panic.
#[test]
fn a_span_that_passes_a_pole_is_refused() {
    let asked = WorldConfig {
        latitude_centre: 80 * LATITUDE_FINE,
        latitude_span: 40 * LATITUDE_FINE,
        ..config()
    };
    let refused = World::new(asked)
        .err()
        .unwrap_or_else(|| panic!("the world takes a span that passes a pole"));
    assert_eq!(
        refused,
        WorldError::LatitudesOutsideTheGlobe {
            centre: asked.latitude_centre,
            span: asked.latitude_span,
        }
    );
    let wider = WorldConfig {
        latitude_centre: 0,
        latitude_span: 2 * LATITUDE_POLE + 1,
        ..config()
    };
    assert!(World::new(wider).is_err());
    let whole = WorldConfig {
        latitude_centre: 0,
        latitude_span: 2 * LATITUDE_POLE,
        ..config()
    };
    assert!(World::new(whole).is_ok());
}
