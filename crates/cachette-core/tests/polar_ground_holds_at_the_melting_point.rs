//! Ground that carries ice stays at the melting point until the ice is gone.
//!
//! The weather field drives the temperature of a cell toward what the world
//! asks of it. Nothing used to stop a polar summer, because the insolation
//! anomaly peaks at a pole, so the seasonal amplitude of the model grows
//! poleward while the annual mean flattens. On a real planet the surface of a
//! polar cell stays near the melting point through its summer, because the
//! energy that arrives melts the ice standing there rather than warming the
//! ground under it.[^1] [^2]
//!
//! **The engine must invoke the clamp, so the world test starts at the
//! engine.** The first tests move one input of the rule and watch the answer
//! move. The last one steps a world and reads its poles.[^3]
//!
//! # References
//!
//! [^1]: ADR-0190, ground that carries ice stays at the melting point until the ice is gone. `docs/adrs/draft/adr-0190-ground-that-carries-ice-stays-at-the-melting-point-until-the-ice-is-gone.md`
//! [^2]: Findings register, FND-619. `docs/FINDINGS.md`
//! [^3]: Testing rules, sections 2, 2a and 5. `.agents/rules/testing.md`

use cachette_core::weather::{
    frozen_hold, HEAT_CEILING, MELTING_WARMTH, MELT_COST, SEASON_PERIOD_TICKS,
};
use cachette_core::{TileIdx, WeatherScale, World, WorldConfig, LATITUDE_FINE};

/// The threads that one step of the world test runs on.
const THREADS: usize = 4;

/// The extent of the world that the last test builds.
///
/// **The fixture must reach polar ground that carries ice, and a world chosen
/// to look right does not supply it.** This world spans the whole globe, so
/// its first row and its last row stand at a pole. The field also starts every
/// cell at the middle of the heat scale, which is below the melting point, so
/// the poles bank freezing from the first pass.[^1]
///
/// # References
///
/// [^1]: Testing rules, section 2a. `.agents/rules/testing.md`
const EXTENT: u32 = 64;

#[test]
fn a_cell_below_the_melting_point_banks_what_it_froze() {
    // The deposit is the depth below the melting point, and one solve adds one
    // depth. A cell ten units below banks ten.
    let (_, bank) = frozen_hold(MELTING_WARMTH - 10, 0);
    assert_eq!(
        bank, 10,
        "one solve banks the depth below the melting point"
    );
    let (_, deeper) = frozen_hold(MELTING_WARMTH - 40, bank);
    assert_eq!(deeper, 50, "a second solve adds its own depth to the first");
}

#[test]
fn the_deposit_costs_the_cell_nothing() {
    // The freezing of a real surface stalls it briefly, and the ice then cools
    // freely from its own top. This field carries the surface, so the deposit
    // never holds a cold cell up.
    let (held, bank) = frozen_hold(MELTING_WARMTH - 30, 0);
    assert_eq!(held, MELTING_WARMTH - 30, "the cell keeps its temperature");
    assert_eq!(bank, 30, "and it banks the depth it stands at");
}

#[test]
fn a_cell_above_the_melting_point_banks_nothing() {
    let (held, bank) = frozen_hold(MELTING_WARMTH + 3, 0);
    assert_eq!(bank, 0, "a cell above the melting point grows no ice");
    assert_eq!(held, MELTING_WARMTH + 3, "so nothing holds it down");
}

#[test]
fn a_bank_holds_a_warm_cell_at_the_melting_point() {
    // The withdrawal gives up the whole rise above the melting point while the
    // bank pays for it.
    let bank = 40 * MELT_COST;
    let (held, left) = frozen_hold(MELTING_WARMTH + 6, bank);
    assert_eq!(held, MELTING_WARMTH, "the cell falls back to the point");
    assert_eq!(
        left,
        bank - 6 * MELT_COST,
        "and the six units it gave up spend what those six cost to grow"
    );
}

#[test]
fn a_cell_with_no_bank_stays_where_it_stands() {
    // **The zero of the term is the melting point of water.** A cell that has
    // never stood below it is untouched, and so is a cell whose ice is gone.
    let (held, bank) = frozen_hold(MELTING_WARMTH + 6, 0);
    assert_eq!(held, MELTING_WARMTH + 6, "no ice stands, so nothing melts");
    assert_eq!(bank, 0, "and a cell above the point banks nothing");
}

#[test]
fn a_part_bank_buys_a_part_of_the_fall() {
    // The clamp is not a switch. A bank that pays for two units gives back two
    // and leaves the rest.
    let (held, left) = frozen_hold(MELTING_WARMTH + 6, 2 * MELT_COST);
    assert_eq!(
        held,
        MELTING_WARMTH + 4,
        "the cell keeps the rise the bank could not pay for"
    );
    assert_eq!(left, 0, "and the bank is spent");
}

#[test]
fn a_cell_at_the_melting_point_spends_nothing() {
    let bank = 100 * MELT_COST;
    let (held, left) = frozen_hold(MELTING_WARMTH, bank);
    assert_eq!(held, MELTING_WARMTH, "the cell is already at the point");
    assert_eq!(left, bank, "so it spends none of its ice");
}

#[test]
fn the_bank_is_bounded() {
    // The bound is what one whole season period deposits when a cell stands
    // the whole heat scale below the melting point for the whole of it.
    let ceiling = SEASON_PERIOD_TICKS * i64::from(HEAT_CEILING);
    let (_, bank) = frozen_hold(0, ceiling);
    assert_eq!(bank, ceiling, "a full bank takes no more");
}

#[test]
fn the_pole_of_a_stepped_world_holds_at_the_melting_point() {
    // **The engine is obliged to invoke the clamp, so the test starts at the
    // engine.** It builds a world, steps it through a whole season period, and
    // reads the poles of the world it stepped.
    let mut world = World::with_weather_scale(
        WorldConfig {
            width: EXTENT,
            height: EXTENT,
            seed: 0x2f,
            faction_count: 1,
            unit_capacity: 64,
        },
        WeatherScale::from_bits(0).expect("the scale describes a lattice"),
    )
    .expect("the settings describe a world");

    let lattice = world.weather().lattice();
    let inner = lattice.inner();
    let high = inner.height();
    let latitudes = world.weather().latitudes();

    // The cells of the world that stand poleward of seventy degrees, in
    // ascending index order.
    let mut polar: Vec<u32> = Vec::new();
    for index in 0..inner.tile_count() {
        let Some(address) = inner.address_of(TileIdx(index)) else {
            continue;
        };
        let Some(cell) = lattice.whole_of_inner(index) else {
            continue;
        };
        let degrees = (latitudes.of_row(address.r.max(0) as u32, high) / LATITUDE_FINE).abs();
        if degrees >= 70 {
            polar.push(cell);
        }
    }
    assert!(
        !polar.is_empty(),
        "the fixture must reach polar ground, and this world holds none"
    );

    // **The claim, read at every tick and not only at the end.** A cell that
    // ends a solve above the melting point must hold a bank too small to buy
    // one unit of fall. A larger bank means the clamp let the cell through
    // with ice still standing.
    //
    // One whole season period covers one summer at each pole.
    let mut carrying = 0usize;
    let mut warmest = i32::MIN;
    for _ in 0..SEASON_PERIOD_TICKS {
        world.step(THREADS).expect("the step must run");
        let field = world.weather();
        for cell in &polar {
            let held = field.warmth_at(*cell);
            let bank = i64::from(field.frozen_at(*cell));
            if bank >= MELT_COST {
                carrying += 1;
                if held > warmest {
                    warmest = held;
                }
            }
            assert!(
                held <= MELTING_WARMTH || bank < MELT_COST,
                "the polar cell {cell} stands at {held} against a melting \
                 point of {MELTING_WARMTH} while it carries a bank of {bank}"
            );
        }
    }

    // **The fixture must reach the case it tests.** A world whose poles never
    // froze would pass the assertion above and measure nothing.
    assert!(
        carrying > 0,
        "the fixture reached none of the case it tests: not one of the {} \
         polar cells ever carried one melt cost of ice",
        polar.len()
    );
    assert!(
        warmest <= MELTING_WARMTH,
        "the warmest polar reading under ice was {warmest}"
    );
}
