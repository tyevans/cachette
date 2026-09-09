//! A storm that the field carries, moves, and ends.
//!
//! Every test here drives the public crate interface. The engine is obliged
//! to move a storm and to end one, so a test that called the solve directly
//! would prove only that the mechanism works.[^1]
//!
//! **A storm here is imposed and it did not form.** A field of one layer
//! grows no baroclinic eddies, so no low can emerge from it. The tests
//! therefore assert what a placed low does, and never that one appeared out
//! of the field.[^2]
//!
//! **The fixture is a fine lattice and not the default one.** A weather cell
//! of the default pitch over a small world spans thousands of kilometres,
//! which is wider than any storm, and a figure taken there would say nothing
//! about a storm. The tests build the lattice at one cell for each tile, so
//! that a cell is about eighty kilometres and a storm of three cells is the
//! size of a real one.[^3]
//!
//! # References
//!
//! [^1]: Testing rules, section 5. `.agents/rules/testing.md`
//! [^2]: A cyclone. `crates/cachette-core/src/weather.rs`
//! [^3]: Findings register, FND-618. `docs/FINDINGS.md`

use cachette_core::weather::{
    cyclone_capacity, cyclone_forms, cyclone_genesis_cell, cyclone_wander, Drops, WeatherError,
};
use cachette_core::{
    Axial, Cyclone, CycloneSetting, TileIdx, WeatherScale, World, WorldConfig,
    CYCLONE_DEPTH_CEILING, CYCLONE_RADIUS_CEILING,
};

/// The thread counts that every equivalence test runs at.
const THREAD_COUNTS: [usize; 3] = [1, 2, 12];

/// The extent of the world that the storm tests run on.
const EXTENT: u32 = 96;

/// The seed of that world.
const SEED: u64 = 0x2f;

/// The ticks that the field settles for before a test places a storm.
const SETTLE: u64 = 120;

/// Builds a world whose weather lattice holds one cell for each tile.
fn fine_world() -> World {
    let mut world = World::with_weather_scale(
        WorldConfig {
            width: EXTENT,
            height: EXTENT,
            seed: SEED,
            faction_count: 2,
            unit_capacity: 1024,
        },
        WeatherScale::PER_TILE,
    )
    .expect("the extent must describe a world");
    for _ in 0..SETTLE {
        world.step(1).expect("the step must run");
    }
    world
}

/// Returns the place over the warmest cell that the storm tests use.
///
/// A storm needs somewhere warm to stand, and the warmest cell of a world is
/// the one a real storm would choose. The test asks the world for it rather
/// than naming a place that a change to the ground generator would move.
fn warmest_place(world: &World) -> Axial {
    let cells = world.weather().cells();
    let plane = world.weather().warmth_plane();
    let mut best = 0usize;
    for cell in 0..plane.len() {
        if plane[cell] > plane[best] {
            best = cell;
        }
    }
    let inner = world
        .weather_lattice()
        .inner_address_of(best as u32)
        .unwrap_or_else(|| {
            cells
                .address_of(TileIdx(best as u32))
                .expect("the cell is on the lattice")
        });
    Axial::new(inner.q, inner.r)
}

/// Runs the world until the storm with an identity is gone, and returns the
/// ticks it lived.
fn run_to_the_end(world: &mut World, id: u32, ceiling: u64) -> u64 {
    for tick in 0..ceiling {
        world.step(1).expect("the step must run");
        if !world.cyclones().iter().any(|storm| storm.id == id) {
            return tick;
        }
    }
    ceiling
}

/// **A storm ends.** A storm that never dissipates is not a storm.
#[test]
fn a_storm_ends() {
    let mut world = fine_world();
    let place = warmest_place(&world);
    let raised = world
        .raise_cyclone(place, CycloneSetting::TROPICAL)
        .expect("the verb must place a storm");
    let lived = run_to_the_end(
        &mut world,
        raised.id,
        u64::from(CycloneSetting::TROPICAL.life) * 2,
    );
    assert!(
        lived < u64::from(CycloneSetting::TROPICAL.life),
        "the storm lived {lived} ticks against a life of {}",
        CycloneSetting::TROPICAL.life
    );
    assert!(
        !world.cyclones().iter().any(|storm| storm.id == raised.id),
        "the field still carries the storm"
    );
}

/// **A storm that stands still is not a storm.** The eye must leave the cell
/// it was placed on.
#[test]
fn a_storm_travels() {
    let mut world = fine_world();
    let place = warmest_place(&world);
    let raised = world
        .raise_cyclone(place, CycloneSetting::TROPICAL)
        .expect("the verb must place a storm");
    let start = raised.eye();
    let mut furthest = 0u32;
    for _ in 0..64 {
        world.step(1).expect("the step must run");
        let Some(storm) = world.cyclones().iter().find(|one| one.id == raised.id) else {
            break;
        };
        furthest = furthest.max(start.distance(storm.eye()));
    }
    assert!(furthest > 0, "the eye never left the cell it was placed on");
}

/// **The water account holds under the deepest storm the field carries.**
///
/// The engine checks the account on every frame and reads all three planes,
/// so a storm that created or destroyed water would fail at once. The test
/// asserts the reading as well, because the step reports its own refusal
/// through an error and a reader of this test should see the account named.
#[test]
fn the_water_account_holds_under_a_storm() {
    let mut world = fine_world();
    let place = warmest_place(&world);
    let setting = CycloneSetting {
        depth: CYCLONE_DEPTH_CEILING,
        radius: CYCLONE_RADIUS_CEILING,
        life: 64,
    };
    world
        .raise_cyclone(place, setting)
        .expect("the verb must place a storm");
    for tick in 0..96 {
        world.step(1).expect("the step must run");
        assert!(
            world.weather().check_account(),
            "the water account did not balance at tick {tick}"
        );
    }
}

/// **A storm rains harder and blows harder than the field around it.**
///
/// The figures are read over the cells this storm covers against the cells no
/// storm covers, in the same frames. A comparison against the field before the
/// storm would measure the season as well.
///
/// **The test reads the footprint of the storm it raised, and not the
/// depression plane.** That plane carries every storm, and the genesis pass
/// raises storms of its own that are larger than this one and stand
/// elsewhere. A test that read the plane measured those, and it answered for
/// the genesis rate rather than for the storm it placed.[^1]
///
/// # References
///
/// [^1]: Findings register, FND-713. `docs/FINDINGS.md`
#[test]
fn a_storm_rains_and_blows_harder_than_the_field_around_it() {
    let mut world = fine_world();
    let place = warmest_place(&world);
    let raised = world
        .raise_cyclone(place, CycloneSetting::TROPICAL)
        .expect("the verb must place a storm");

    let mut rain_under = 0i64;
    let mut rain_out = 0i64;
    let mut wind_under = 0i64;
    let mut wind_out = 0i64;
    let mut under_cells = 0i64;
    let mut out_cells = 0i64;
    for _ in 0..64 {
        let before: Vec<i64> = world
            .weather()
            .ground_plane()
            .iter()
            .map(|drops| drops.0)
            .collect();
        world.step(1).expect("the step must run");
        let field = world.weather();
        let Some(mine) = field.cyclones().iter().find(|one| one.id == raised.id) else {
            break;
        };
        let eye = mine.eye();
        let reach = mine.radius.max(0) as u32;
        let cells = field.cells();
        for at in 0..field.ground_plane().len() {
            let gained = field.ground_plane()[at].0 - before.get(at).copied().unwrap_or(0);
            let speed = i64::from(field.wind_plane()[at].speed());
            let Some(address) = cells.address_of(TileIdx(at as u32)) else {
                continue;
            };
            if address.distance(eye) <= reach {
                rain_under += gained.max(0);
                wind_under += speed;
                under_cells += 1;
            } else if field.depression_at(at as u32) == 0 {
                rain_out += gained.max(0);
                wind_out += speed;
                out_cells += 1;
            }
        }
    }
    assert!(under_cells > 0, "the storm covered no cell");
    let rain_under = rain_under / under_cells;
    let rain_out = rain_out / out_cells.max(1);
    let wind_under = wind_under / under_cells;
    let wind_out = wind_out / out_cells.max(1);
    assert!(
        rain_under > rain_out,
        "the rain under the storm was {rain_under} for each cell and tick, against {rain_out} elsewhere"
    );
    assert!(
        wind_under > wind_out,
        "the wind under the storm was {wind_under}, against {wind_out} elsewhere"
    );
}

/// **The storm holds the air of its eye under the bound it imposes.**
///
/// The convergence that a storm builds fills the air of the eye, and the
/// bound is what pours that water out. A field that read the deficit for the
/// wind and not for the capacity would still rain more under a storm, because
/// the convergence alone piles water there. So this test asserts the bound
/// itself rather than the rain, and the rain test cannot stand in for it.
#[test]
fn the_air_of_an_eye_stands_under_the_bound_the_storm_imposes() {
    let mut world = fine_world();
    let place = warmest_place(&world);
    let setting = CycloneSetting {
        depth: CYCLONE_DEPTH_CEILING,
        radius: 2,
        life: 64,
    };
    let raised = world
        .raise_cyclone(place, setting)
        .expect("the verb must place a storm");
    let mut checked = 0;
    for _ in 0..48 {
        world.step(1).expect("the step must run");
        let field = world.weather();
        let Some(storm) = field.cyclones().iter().find(|one| one.id == raised.id) else {
            break;
        };
        let Some(at) = field.cells().index_of(storm.eye()) else {
            continue;
        };
        let cell = at.0;
        let deficit = field.depression_at(cell);
        if deficit <= 0 {
            continue;
        }
        let bound = cyclone_capacity(field.capacity_at_cell(cell), deficit);
        assert!(
            field.air_at(cell).0 <= bound.0,
            "the eye held {} drops against a bound of {}",
            field.air_at(cell).0,
            bound.0
        );
        checked += 1;
    }
    assert!(checked > 0, "the test never read an eye");
}

/// **A storm leaves the ambient field where it found it.**
///
/// The mean wind of the whole lattice before the storm and long after it must
/// agree closely. A storm that pinned the field or wrecked it is a defect,
/// whatever it did while it stood.
#[test]
fn the_field_comes_back_after_a_storm() {
    let mut world = fine_world();
    let before = mean_wind(&world);
    let place = warmest_place(&world);
    let raised = world
        .raise_cyclone(place, CycloneSetting::TROPICAL)
        .expect("the verb must place a storm");
    run_to_the_end(&mut world, raised.id, 512);
    for _ in 0..SETTLE {
        world.step(1).expect("the step must run");
    }
    let after = mean_wind(&world);
    assert!(
        (before - after).abs() <= 4,
        "the mean wind was {before} before the storm and {after} after it"
    );
}

/// Returns the mean wind speed over the whole lattice.
fn mean_wind(world: &World) -> i64 {
    let plane = world.weather().wind_plane();
    let mut total = 0i64;
    for wind in plane {
        total += i64::from(wind.speed());
    }
    total / plane.len().max(1) as i64
}

/// **The engine raises a storm of its own.**
///
/// The engine is obliged to try, so the test drives the engine and then reads
/// the count. A test that called the genesis pass directly would prove that
/// the pass works and not that anything reaches it.[^1]
///
/// # References
///
/// [^1]: Testing rules, section 5. `.agents/rules/testing.md`
#[test]
fn the_engine_raises_a_storm_of_its_own() {
    let mut world = fine_world();
    for _ in 0..1024 {
        world.step(1).expect("the step must run");
        if world.weather().cyclones_raised() > 0 {
            return;
        }
    }
    panic!("the engine raised no storm in 1024 ticks");
}

/// **One binary gives one answer at any thread count, with a storm standing.**
#[test]
fn a_storm_gives_one_answer_at_any_thread_count() {
    let mut hashes = Vec::new();
    for threads in THREAD_COUNTS {
        let mut world = fine_world();
        let place = warmest_place(&world);
        world
            .raise_cyclone(place, CycloneSetting::TROPICAL)
            .expect("the verb must place a storm");
        for _ in 0..48 {
            world.step(threads).expect("the step must run");
        }
        hashes.push(world.state_hash());
    }
    assert!(
        hashes.windows(2).all(|pair| pair[0] == pair[1]),
        "the thread counts {THREAD_COUNTS:?} gave the hashes {hashes:?}"
    );
}

/// **The genesis draw reads the frame.** A draw that did not would try on
/// every frame or on none.
#[test]
fn the_genesis_draw_reads_the_frame() {
    let mut tried = 0;
    for tick in 0..512u64 {
        if cyclone_forms(SEED, cachette_core::Tick(tick), 0) {
            tried += 1;
        }
    }
    assert!(
        tried > 0 && tried < 512,
        "the field tried on {tried} of 512 frames"
    );
}

/// **The genesis draw reads the seed.** Two worlds must not raise their
/// storms on the same frames.
#[test]
fn the_genesis_draw_reads_the_seed() {
    let one: Vec<bool> = (0..512)
        .map(|tick| cyclone_forms(SEED, cachette_core::Tick(tick), 0))
        .collect();
    let other: Vec<bool> = (0..512)
        .map(|tick| cyclone_forms(SEED + 1, cachette_core::Tick(tick), 0))
        .collect();
    assert_ne!(one, other, "two seeds tried on the same frames");
}

/// **The genesis cell reads the frame and the seed.**
#[test]
fn the_genesis_cell_reads_the_frame_and_the_seed() {
    let cells = 4096;
    let here = cyclone_genesis_cell(SEED, cachette_core::Tick(7), 0, cells);
    assert_ne!(
        here,
        cyclone_genesis_cell(SEED, cachette_core::Tick(8), 0, cells),
        "the cell did not move with the frame"
    );
    assert_ne!(
        here,
        cyclone_genesis_cell(SEED + 1, cachette_core::Tick(7), 0, cells),
        "the cell did not move with the seed"
    );
}

/// **The genesis draws read the attempt.** One solve makes several attempts,
/// and attempts that shared a key would name one cell and either all go ahead
/// or all stop. The field would then raise one storm each solve at most,
/// whatever the attempt count said.
#[test]
fn the_genesis_draws_read_the_attempt() {
    let cells = 4096;
    let tick = cachette_core::Tick(7);
    let here = cyclone_genesis_cell(SEED, tick, 0, cells);
    assert_ne!(
        here,
        cyclone_genesis_cell(SEED, tick, 1, cells),
        "two attempts of one frame named one cell"
    );
    let gates: Vec<bool> = (0..64)
        .map(|attempt| cyclone_forms(SEED, tick, attempt))
        .collect();
    assert!(
        gates.iter().any(|open| *open) && gates.iter().any(|open| !*open),
        "the attempts of one frame all gave {gates:?}"
    );
}

/// **The genesis draws read nothing else.** A draw that moved when no field
/// of its key moved would hold state, and the field would not repeat.
#[test]
fn the_genesis_draws_repeat_on_one_key() {
    let cells = 4096;
    let tick = cachette_core::Tick(7);
    assert_eq!(
        cyclone_genesis_cell(SEED, tick, 3, cells),
        cyclone_genesis_cell(SEED, tick, 3, cells),
        "one key gave two cells"
    );
    assert_eq!(
        cyclone_forms(SEED, tick, 3),
        cyclone_forms(SEED, tick, 3),
        "one key gave two answers"
    );
}

/// **The wander reads the frame, the identity and the seed.**
///
/// A wander that missed the identity would give a storm born into the slot of
/// a dead one the track of the dead one. A wander that missed the frame would
/// hold every storm on one heading for ever. Neither defect is visible to a
/// determinism test, because both repeat exactly.[^1]
///
/// # References
///
/// [^1]: Testing rules, section 2. `.agents/rules/testing.md`
#[test]
fn the_wander_reads_every_field_of_its_key() {
    let here = cyclone_wander(SEED, cachette_core::Tick(11), 3);
    assert_ne!(
        here,
        cyclone_wander(SEED, cachette_core::Tick(12), 3),
        "the wander did not move with the frame"
    );
    assert_ne!(
        here,
        cyclone_wander(SEED, cachette_core::Tick(11), 4),
        "the wander did not move with the identity"
    );
    assert_ne!(
        here,
        cyclone_wander(SEED + 1, cachette_core::Tick(11), 3),
        "the wander did not move with the seed"
    );
}

/// **A storm that dies frees its slot and never its identity.**
///
/// Two storms raised one after the other must carry different identities,
/// even when the first is gone by the time the second is raised.
#[test]
fn a_second_storm_takes_a_new_identity() {
    let mut world = fine_world();
    let place = warmest_place(&world);
    let first = world
        .raise_cyclone(place, CycloneSetting::SEVERE)
        .expect("the verb must place a storm");
    run_to_the_end(&mut world, first.id, 512);
    let place = warmest_place(&world);
    let second = world
        .raise_cyclone(place, CycloneSetting::SEVERE)
        .expect("the verb must place a second storm");
    assert_ne!(
        first.id, second.id,
        "the second storm took the identity of the first"
    );
}

/// **The two settings are the same object at two points of one parameter
/// set.** The small one is deeper over one cell, and the large one is
/// shallower over several.
#[test]
fn the_two_settings_hold_different_shapes() {
    let severe = Cyclone {
        q_fine: 0,
        r_fine: 0,
        depth: CycloneSetting::SEVERE.depth,
        radius: CycloneSetting::SEVERE.radius,
        life: CycloneSetting::SEVERE.life,
        age: 0,
        id: 0,
    };
    let tropical = Cyclone {
        depth: CycloneSetting::TROPICAL.depth,
        radius: CycloneSetting::TROPICAL.radius,
        life: CycloneSetting::TROPICAL.life,
        ..severe
    };
    let eye = severe.eye();
    assert!(
        severe.deficit_at(eye) > tropical.deficit_at(eye),
        "the small storm is not the deeper of the two at its eye"
    );
    let away = Axial::new(eye.q + 2, eye.r);
    assert_eq!(
        severe.deficit_at(away),
        0,
        "the small storm reached two cells"
    );
    assert!(
        tropical.deficit_at(away) > 0,
        "the large storm did not reach two cells"
    );
    assert!(
        tropical.life > severe.life,
        "the large storm does not last longer"
    );
}

/// **A deeper eye leaves the air less room.** The capacity falls as the
/// deficit rises, and it never falls to nothing.
#[test]
fn a_deeper_eye_leaves_less_room() {
    let base = Drops(64_000);
    assert_eq!(
        cyclone_capacity(base, 0),
        base,
        "a clear cell shed capacity"
    );
    let shallow = cyclone_capacity(base, CYCLONE_DEPTH_CEILING / 4);
    let deep = cyclone_capacity(base, CYCLONE_DEPTH_CEILING);
    assert!(
        shallow.0 < base.0 && deep.0 < shallow.0,
        "the capacities ran {} then {} from {}",
        shallow.0,
        deep.0,
        base.0
    );
    assert!(deep.0 > 0, "the deepest eye left the air no room at all");
}

/// **The verb refuses a setting outside the range and a place outside the
/// world.**
#[test]
fn the_verb_refuses_what_it_cannot_carry() {
    let mut world = fine_world();
    let place = warmest_place(&world);
    assert_eq!(
        world.raise_cyclone(
            place,
            CycloneSetting {
                depth: CYCLONE_DEPTH_CEILING + 1,
                radius: 0,
                life: 8,
            }
        ),
        Err(WeatherError::CycloneSettingOutOfRange)
    );
    assert_eq!(
        world.raise_cyclone(Axial::new(EXTENT as i32 + 4, 0), CycloneSetting::TROPICAL),
        Err(WeatherError::PlaceOutsideWorld(Axial::new(
            EXTENT as i32 + 4,
            0
        )))
    );
}

/// **The field carries a bounded number of storms.**
#[test]
fn the_field_refuses_more_storms_than_it_holds() {
    let mut world = fine_world();
    let place = warmest_place(&world);
    let mut refused = None;
    for _ in 0..(cachette_core::CYCLONE_CEILING + 2) {
        if let Err(error) = world.raise_cyclone(place, CycloneSetting::TROPICAL) {
            refused = Some(error);
            break;
        }
    }
    assert_eq!(refused, Some(WeatherError::TooManyCyclones));
}
