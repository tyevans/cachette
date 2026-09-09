//! The row structure of the weather field, and the storms that disturb it.
//!
//! The project owner reports that the moisture field collapses into three
//! narrow latitude bands, that the bands stand in the same place whatever
//! extent the world takes, and that the field stops changing once they form.
//! The tests here state the second and the third as numbers.
//!
//! Every test drives the public crate interface. The engine is obliged to
//! solve the field on every step, so a test that called the solve directly
//! would prove only that the mechanism works.[^1]
//!
//! **The fixture is a whole planet at one cell for each tile.** The banded
//! circulation puts its lows at the equator and at sixty degrees, so a world
//! that spans a few degrees holds no band at all and would measure nothing.
//! A lattice at the default pitch over a small world holds one cell, which
//! has no row structure either. The tests therefore ask for the finest
//! lattice on a world wide enough to hold several cells in each band.[^2]
//!
//! # References
//!
//! [^1]: Testing rules, section 5. `.agents/rules/testing.md`
//! [^2]: Testing rules, section 2a. `.agents/rules/testing.md`

use cachette_core::weather::{front_mark, storm_reach, Latitudes};
use cachette_core::{
    WeatherScale, World, WorldConfig, CYCLONE_CEILING, CYCLONE_RADIUS_CEILING, LATITUDE_FINE,
};

/// The seed of every world here.
const SEED: u64 = 0x2f;

/// The solves that a world runs before anything reads it.
///
/// The field starts at the middle of the temperature scale and holds no
/// water, so it needs time to reach the state a watcher sees.
const SETTLE_TICKS: u32 = 200;

/// The threads that a settling run uses. One is enough, because the answer
/// does not depend on the count and a thread-count test states that
/// elsewhere.
const THREADS: usize = 1;

/// The latitude above which a place is outside the tropics.
const TROPIC: i32 = 30 * LATITUDE_FINE;

/// The storms that a whole planet must carry at once.
///
/// **A real globe carries some tens of lows and storms at any moment.** This
/// mark asks for a fraction of that, so it fails only a world that carries
/// almost none.
const STANDING_FLOOR: usize = 2;

/// The solves that the run watches for a storm outside the tropics.
///
/// **A snapshot of a planet is too small a sample for this claim.** A settled
/// world of this extent carries a few storms at once, so whether one of them
/// stands outside the tropics at one moment is a matter of chance. The claim
/// is about what the field raises over time, so the test reads over time.[^1]
///
/// # References
///
/// [^1]: Testing Rules, a fixture supplies the input. `.agents/rules/testing.md`
const WATCH_TICKS: u32 = 400;

/// Builds a world at one weather cell for each tile.
fn world_of(extent: u32) -> World {
    World::with_weather_scale(
        WorldConfig {
            width: extent,
            height: extent,
            seed: SEED,
            faction_count: 4,
            unit_capacity: 1024,
        },
        WeatherScale::PER_TILE,
    )
    .expect("the settings describe a world")
}

/// Returns the mean of a run of numbers, or zero when the run is empty.
fn mean(values: &[i64]) -> i64 {
    if values.is_empty() {
        0
    } else {
        values.iter().sum::<i64>() / values.len() as i64
    }
}

/// Returns the mean air of each row of the world, in row order.
fn air_by_row(world: &World) -> Vec<i64> {
    let field = world.weather();
    let lattice = field.lattice();
    let height = lattice.inner().height();
    let mut rows: Vec<Vec<i64>> = vec![Vec::new(); height as usize];
    for cell in lattice.inner_cells() {
        let row = lattice.inner_row_of(cell) as usize;
        if let Some(slot) = rows.get_mut(row) {
            slot.push(field.air_at(cell).0);
        }
    }
    rows.iter().map(|row| mean(row)).collect()
}

/// Returns the row that holds the most air.
fn wettest_row(world: &World) -> usize {
    let rows = air_by_row(world);
    let mut best = 0;
    for (row, air) in rows.iter().enumerate() {
        if *air > rows[best] {
            best = row;
        }
    }
    best
}

/// Returns the latitude of every standing storm, in hundredths of a degree.
fn storm_latitudes(world: &World) -> Vec<i32> {
    let field = world.weather();
    let lattice = field.lattice();
    let height = lattice.inner().height();
    field
        .cyclones()
        .iter()
        .map(|storm| {
            let row = storm.eye().r.max(0) as u32;
            let inner = row
                .saturating_sub(lattice.ring())
                .min(height.saturating_sub(1));
            field.latitudes().of_row(inner, height)
        })
        .collect()
}

/// Steps a world past the settling phase.
fn settled(extent: u32) -> World {
    let mut world = world_of(extent);
    for _ in 0..SETTLE_TICKS {
        world.step(THREADS).expect("the step must run");
    }
    world
}

/// **A band stands at a latitude, and not at a row.** The world states its
/// latitude span, so the row that holds the wettest air must sit at the same
/// share of the height on two worlds of very different extent.
///
/// The test can fail. A band pinned to a row index would sit at half the
/// height of the small world and at an eighth of the height of the large one,
/// which is a gap of thirty-seven points against a tolerance of ten.
#[test]
fn the_wettest_band_stands_at_one_share_of_the_height() {
    let small = settled(24);
    let large = settled(96);
    let small_height = small.weather().lattice().inner().height();
    let large_height = large.weather().lattice().inner().height();
    let small_share = wettest_row(&small) as i64 * 100 / i64::from(small_height);
    let large_share = wettest_row(&large) as i64 * 100 / i64::from(large_height);
    assert!(
        (small_share - large_share).abs() <= 10,
        "the wettest row sat at {small_share}% of a world of {small_height} rows \
         and at {large_share}% of a world of {large_height} rows"
    );
}

/// **A latitude-only forcing gives a field that is constant along every row,
/// and the only term of this model that varies along a row and moves is a
/// storm.** So a field that carries almost no storms holds the belts and
/// nothing else, whatever else it does.
///
/// The test asks three things of a settled planet. It must carry storms at
/// once, some of them must stand outside the tropics, and the count must stay
/// under the ceiling that the field holds.
///
/// The second is the stronger claim: the warm-sea gate admits nothing in the
/// middle latitudes, because the middle latitudes are neither warm enough nor
/// all sea, and the middle latitudes are where a real field carries most of
/// its travelling weather.
///
/// **The third is the guard that this test did not hold.** A population that
/// stands at the ceiling is a population that the ceiling chose, and the
/// genesis rate behind it can then be any figure at all. A field once ran at
/// forty times the rate it needed, covered a quarter of the planet in storms,
/// and passed a floor of eight without a word.[^1]
///
/// # References
///
/// [^1]: Findings register, FND-712. `docs/FINDINGS.md`
#[test]
fn the_field_carries_storms_outside_the_tropics() {
    let world = settled(48);
    let latitudes = storm_latitudes(&world);
    assert!(
        latitudes.len() >= STANDING_FLOOR,
        "the planet carried {} storms at once, against the floor of {STANDING_FLOOR}",
        latitudes.len()
    );
    assert!(
        latitudes.len() < CYCLONE_CEILING,
        "the planet carried {} storms at once, and the ceiling of {CYCLONE_CEILING} is what held it",
        latitudes.len()
    );
    let mut world = world;
    let mut seen = Vec::new();
    for _ in 0..WATCH_TICKS {
        world.step(THREADS).expect("the step must run");
        seen.extend(storm_latitudes(&world));
    }
    let outside = seen
        .iter()
        .filter(|latitude| latitude.abs() >= TROPIC)
        .count();
    assert!(
        outside > 0,
        "every one of the {} storms the run watched stood inside the tropics",
        seen.len()
    );
}

/// **A storm has one size on the ground, so its reach in cells follows the
/// lattice.** A reach fixed in cells would draw a storm of one size on a
/// coarse lattice and of a very different size on a fine one, and the picture
/// would then depend on the extent of the world.
#[test]
fn the_reach_of_a_storm_follows_the_lattice() {
    let coarse = storm_reach(Latitudes::PLANET, 24);
    let fine = storm_reach(Latitudes::PLANET, 96);
    assert!(
        fine > coarse,
        "a lattice of 96 rows asked for {fine} cells and one of 24 rows for {coarse}"
    );
    assert!(
        coarse >= 1,
        "a lattice of 24 rows asked for {coarse} cells, and a storm covers a cell"
    );
    assert!(
        storm_reach(Latitudes::PLANET, 4096) <= CYCLONE_RADIUS_CEILING,
        "the finest lattice asked for more than the field carries"
    );
}

/// **The front mark is a gradient, so it follows the lattice too.** A mark
/// stated as a difference between two neighbours would admit a front
/// everywhere on a coarse lattice and nowhere on a fine one.
#[test]
fn the_front_mark_follows_the_lattice() {
    let coarse = front_mark(Latitudes::PLANET, 24);
    let fine = front_mark(Latitudes::PLANET, 96);
    assert!(
        coarse > fine,
        "a lattice of 24 rows asked for {coarse} steps and one of 96 rows for {fine}"
    );
    assert!(fine >= 1, "a lattice of 96 rows asked for {fine} steps");
}
