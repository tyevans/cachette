//! A faction seated on an island reaches ground beyond the water.
//!
//! A faction whose seat sits on a small landmass cannot leave it while every
//! water tile refuses every unit. It settles no new ground, it reaches no
//! enemy, and it trades over no road, so several subsystems are dead for it
//! through no fault of its own.
//!
//! The engine now carries a water crossing column on the unit type table.
//! Zero means cannot, in the way every capability column of that table
//! reads.[^1] The terrain capacity table takes the column and answers how
//! many units of that type one water tile holds, so the capacity table stays
//! the one declaration of which ground admits a unit.[^2]
//!
//! These tests go through the public interface of the crate.[^3]
//!
//! **The measurement compares two unit types, not two engines.** Each run
//! spawns one cohort at the same place and sends it at the same tile through
//! the same verb. The runs differ in the type the cohort carries, and in
//! nothing else.[^4]
//!
//! # References
//!
//! [^1]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decision D2. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
//! [^2]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D4. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
//! [^3]: Testing rules, section 6. `.agents/rules/testing.md`
//! [^4]: Testing rules, section 2a. `.agents/rules/testing.md`

use std::collections::{BTreeSet, VecDeque};

use cachette_core::production::BuildCostRow;
use cachette_core::site::CommodityId;
use cachette_core::terrain::{TileKind, SOME_WATER_CROSSING};
use cachette_core::types::Fix32;
use cachette_core::unit_type::{UnitTypeId, UnitTypeRow, MARINER, UNIT_TYPE_COUNT, WORKER};
use cachette_core::{Axial, Entity, FactionId, World, WorldConfig};

/// The extent of the fixture world.
///
/// The coarsest lattice of the terrain generator spans sixty-four tiles, so
/// this world holds four lattice cells along each axis. It therefore holds
/// water as well as open ground, and it holds landmasses of more than one
/// size.[^1]
///
/// # References
///
/// [^1]: Findings register, FND-054. `docs/FINDINGS.md`
const EXTENT: u32 = 256;

/// The seed of the fixture world.
///
/// The seed is not tuned. The fixture asks this world for its smallest island
/// and seats the faction there, so any seed whose world holds an island
/// serves.
const SEED: u64 = 1;

/// The faction the measurement follows.
const ISLAND: FactionId = FactionId(0);

/// The destination plane that the island faction crosses water on.
///
/// The crossing plane of a faction is its number raised by the faction count,
/// and the fixture holds two factions.
const CROSSING_PLANE: u16 = 2;

/// The size of the founding group.
const GROUP: u32 = 8;

/// The number of units in the cohort that the measurement sends.
const COHORT: usize = 12;

/// The housing that each founded site of the fixture starts with.
///
/// A site whose housing equals its residents holds no free place, so its
/// production queue advances the work of its front entry for ever and never
/// finishes it. The default housing seats a founding group and no more.
const FIXTURE_HOUSING: u32 = 256;

/// The food that the island site earns in one tick, in the fixed-point scale.
///
/// The value is far above what a small island reaches on its own, so the
/// fixture measures where a unit goes rather than what it eats.
const FIXTURE_FOOD_RATE: Fix32 = Fix32::from_int(256);

/// The work that one entry of the production queue takes in the fixture.
///
/// Every row of the build cost table takes it, so no type is cheaper to build
/// than another. The default table gives every type one placeholder, and the
/// fixture lowers that one figure for every type at once.
const FIXTURE_BUILD_WORK: u32 = 1;

/// The number of ticks each run of the measurement makes.
const TICKS: u64 = 400;

/// The threads each step runs on.
const THREADS: usize = 4;

/// The edge of one level 1 cell, in tiles.
///
/// The value follows from the block edge exponent that a world is built with,
/// so the fixture holds no second copy of it.
const BLOCK_SIDE: u32 = 1 << cachette_core::bridge::BLOCK_BITS_DEFAULT;

/// The largest landmass that counts as an island for this measurement.
const ISLAND_CEILING: usize = 4096;

/// The smallest landmass that the fixture seats a group on.
const ISLAND_FLOOR: usize = 64;

/// Builds the fixture world.
///
/// The `crossing` flag says whether the mariner row keeps its crossing. A
/// world built with the flag clear holds the table the engine carried before
/// the column existed, because every other row states a crossing of zero.
fn fixture_world(crossing: bool) -> World {
    let mut world = World::new(WorldConfig {
        width: EXTENT,
        height: EXTENT,
        seed: SEED,
        faction_count: 2,
        ..WorldConfig::default()
    })
    .expect("the fixture config must build a world");
    world.set_founding_housing(FIXTURE_HOUSING);
    for unit_type in 0..UNIT_TYPE_COUNT {
        let row = BuildCostRow {
            work: FIXTURE_BUILD_WORK,
            ..BuildCostRow::PLACEHOLDER
        };
        world
            .define_build_cost(unit_type as u8, row)
            .expect("the number names a row of the table");
    }
    if !crossing {
        world
            .define_unit_type(MARINER.0, UnitTypeRow::NONE)
            .expect("the mariner names a row of the table");
    }
    world
}

/// Returns the passable tiles that a walker reaches from one address without
/// entering water.
///
/// The walk is a breadth-first search over the tile lattice. It is a
/// measurement and not a pass of the engine, so it may read the whole world.
fn land_reach(world: &World, from: Axial) -> BTreeSet<Axial> {
    let mut seen = BTreeSet::new();
    let mut queue = VecDeque::new();
    if !world.admits_a_unit(from) {
        return seen;
    }
    seen.insert(from);
    queue.push_back(from);
    while let Some(here) = queue.pop_front() {
        for direction in 0..6 {
            let Some(there) = world.grid().neighbour(here, direction) else {
                continue;
            };
            if !world.admits_a_unit(there) || seen.contains(&there) {
                continue;
            }
            seen.insert(there);
            queue.push_back(there);
        }
    }
    seen
}

/// Returns the number of tiles a walker reaches when water admits it.
///
/// The walk asks the capacity table with a nonzero crossing, so it states no
/// passability rule of its own.
fn crossing_reach(world: &World, from: Axial) -> usize {
    let mut seen = BTreeSet::new();
    let mut queue = VecDeque::new();
    let admits = |address: Axial| {
        world
            .tile_kind(address)
            .is_some_and(|kind| kind.is_passable_for(SOME_WATER_CROSSING))
    };
    if !admits(from) {
        return 0;
    }
    seen.insert(from);
    queue.push_back(from);
    while let Some(here) = queue.pop_front() {
        for direction in 0..6 {
            let Some(there) = world.grid().neighbour(here, direction) else {
                continue;
            };
            if !admits(there) || seen.contains(&there) {
                continue;
            }
            seen.insert(there);
            queue.push_back(there);
        }
    }
    seen.len()
}

/// Returns the smallest landmass of a world that could hold a founding group.
///
/// **The fixture is constructed and it is not drawn.** The founding survey
/// scores a place by what grows around it, so it seats every faction on the
/// best ground of the largest continent. No seed it chooses seats a faction
/// on an island, and a fixture built from one would measure the survey rather
/// than the crossing.[^1]
///
/// # References
///
/// [^1]: Testing rules, section 2a. `.agents/rules/testing.md`
fn smallest_island(world: &World) -> BTreeSet<Axial> {
    let grid = world.grid();
    let mut seen: BTreeSet<Axial> = BTreeSet::new();
    let mut best: Option<BTreeSet<Axial>> = None;
    for q in 0..grid.width() as i32 {
        for r in 0..grid.height() as i32 {
            let at = Axial::new(q, r);
            if seen.contains(&at) || !world.admits_a_unit(at) {
                continue;
            }
            let mass = land_reach(world, at);
            for address in &mass {
                seen.insert(*address);
            }
            if mass.len() < ISLAND_FLOOR {
                continue;
            }
            if best.as_ref().is_none_or(|held| mass.len() < held.len()) {
                best = Some(mass);
            }
        }
    }
    best.unwrap_or_default()
}

/// Returns the place on one landmass that the founding survey ranks first.
fn seat_of(world: &World, island: &BTreeSet<Axial>) -> Axial {
    let places: Vec<Axial> = island.iter().copied().collect();
    let survey = world
        .survey_places(&places, GROUP, &[])
        .expect("the island holds places to survey");
    survey
        .chosen()
        .map_or(places[0], |candidate| candidate.address())
}

/// Returns the nearest ground that lies off one landmass.
///
/// The scan is a measurement of the fixture and not a pass of the engine. It
/// walks the world in ascending address order and keeps the nearest address,
/// so a tie takes the lower address and the answer is a property of the
/// world.
fn nearest_shore_beyond(world: &World, island: &BTreeSet<Axial>, from: Axial) -> Axial {
    let grid = world.grid();
    let mut best: Option<(u32, Axial)> = None;
    for q in 0..grid.width() as i32 {
        for r in 0..grid.height() as i32 {
            let at = Axial::new(q, r);
            if island.contains(&at) || !world.admits_a_unit(at) {
                continue;
            }
            let span = from.distance(at);
            // **The target must lie in another cell of the level 1.** A field
            // over cells holds no direction inside the cell that seeds it, so
            // a target in the seed cell steers nobody and the cohort takes the
            // keyed draw instead. The block edge is a property of the engine,
            // and the fixture reads it rather than naming a number.
            if span <= BLOCK_SIDE {
                continue;
            }
            if best.is_none_or(|(held, _)| span < held) {
                best = Some((span, at));
            }
        }
    }
    best.expect("the world holds ground beyond the island").1
}

/// What one run of the measurement produced.
struct Run {
    /// The number of tiles that a unit of the faction stood on at any tick.
    visited: usize,
    /// The land tiles the faction reached that lie off its own island.
    land_beyond: usize,
    /// The water tiles the faction stood on.
    on_water: usize,
    /// The people of the faction at the last tick.
    population: u32,
    /// The settlements the faction holds at the last tick.
    settlements: usize,
    /// The tick a unit of the cohort first stood on the tile it was sent to,
    /// or nothing when none of them ever did.
    ///
    /// **This is the last mile.** The destination field holds one direction
    /// for each level 1 cell, so a cohort that reached the cell of its target
    /// had arrived within a block of it and not at it. The approach field
    /// resolves that block at the pitch of one tile.[^1]
    ///
    /// [^1]: Findings register, FND-315. `docs/FINDINGS.md`
    stood_on_target: Option<u64>,
}

/// Seats the island faction and returns the world and the seat.
fn seated_world(crossing: bool, island: &BTreeSet<Axial>) -> (World, Axial) {
    let mut world = fixture_world(crossing);
    let seat = seat_of(&world, island);
    let site = world
        .found_group_at(seat, GROUP, ISLAND)
        .expect("the island must seat the group")
        .settlement();
    world
        .set_production_rate(site, CommodityId(0), FIXTURE_FOOD_RATE)
        .expect("the rate is above zero and the commodity is in the set");
    let _ = world.found_run(GROUP, FactionId(1));
    (world, seat)
}

/// Spawns one cohort of a named type, sends it, and records where it went.
///
/// The cohort goes through the send verb that the controller and a Python
/// caller both call. The two runs of the measurement differ in the type the
/// cohort carries and in nothing else.
fn measure(crossing: bool, unit_type: UnitTypeId, island: &BTreeSet<Axial>) -> Run {
    let (mut world, seat) = seated_world(crossing, island);
    // **The controller of the island faction stands down for this
    // measurement.** The controller climbs the crossing plane with a set of
    // its own, and it would re-aim the cohort at a place of its own choosing.
    // The order this run makes is the order the measurement is about, and a
    // separate test drives the controller instead.
    world.set_externally_controlled(ISLAND, true);
    let cohort: Vec<Entity> = (0..COHORT)
        .map(|_| {
            world
                .spawn_soldier(seat, ISLAND)
                .expect("the seat admits the cohort")
        })
        .collect();
    world.set_unit_type_set(&cohort, unit_type);
    let target = nearest_shore_beyond(&world, island, seat);
    world
        .send_units_to(&cohort, &[target], CROSSING_PLANE)
        .expect("the world holds the plane and every unit is live");

    let mut visited = BTreeSet::new();
    let mut on_water = BTreeSet::new();
    let mut stood_on_target = None;
    for tick in 0..TICKS {
        world.step(THREADS).expect("the step must run");
        for unit in world.soldiers().iter_faction(ISLAND) {
            let Some(address) = world.soldiers().address(unit) else {
                continue;
            };
            visited.insert(address);
            if world.tile_kind(address) == Some(TileKind::Water) {
                on_water.insert(address);
            }
            if address == target && stood_on_target.is_none() {
                stood_on_target = Some(tick);
            }
        }
    }
    let land_beyond = visited
        .iter()
        .filter(|at| !island.contains(at) && world.admits_a_unit(**at))
        .count();
    Run {
        visited: visited.len(),
        land_beyond,
        on_water: on_water.len(),
        population: world.population_of(ISLAND),
        settlements: world
            .settlements()
            .iter()
            .filter(|held| world.settlements().faction(*held) == Some(ISLAND))
            .count(),
        stood_on_target,
    }
}

#[test]
fn the_fixture_seats_a_faction_on_a_small_landmass() {
    let world = fixture_world(true);
    let island = smallest_island(&world);
    assert!(!island.is_empty(), "the world must hold an island");
    let seat = seat_of(&world, &island);
    let whole = crossing_reach(&world, seat);
    let target = nearest_shore_beyond(&world, &island, seat);
    println!("seat {seat:?}, landmass {} tiles", island.len());
    println!("a crossing unit reaches {whole} tiles");
    println!(
        "the nearest ground beyond the island is {target:?}, {} tiles away",
        seat.distance(target)
    );
    assert!(
        island.len() <= ISLAND_CEILING,
        "the fixture must seat the faction on a small landmass"
    );
    assert!(
        whole > island.len() * 4,
        "the water must hide most of the world from a walker"
    );
    assert_eq!(
        land_reach(&world, seat),
        island,
        "the seat must lie on the landmass the fixture measured"
    );
}

#[test]
fn a_faction_on_an_island_reaches_ground_beyond_the_water() {
    let world = fixture_world(true);
    let island = smallest_island(&world);
    let before = measure(false, WORKER, &island);
    let after = measure(true, MARINER, &island);
    println!("island {} tiles", island.len());
    println!(
        "before: visited {}, land beyond {}, on water {}, sites {}, people {}, \
         stood on the target at {:?}",
        before.visited,
        before.land_beyond,
        before.on_water,
        before.settlements,
        before.population,
        before.stood_on_target
    );
    println!(
        "after:  visited {}, land beyond {}, on water {}, sites {}, people {}, \
         stood on the target at {:?}",
        after.visited,
        after.land_beyond,
        after.on_water,
        after.settlements,
        after.population,
        after.stood_on_target
    );
    assert_eq!(
        before.on_water, 0,
        "a cohort with no crossing must never stand on water"
    );
    assert_eq!(
        before.land_beyond, 0,
        "a cohort with no crossing must never leave its island"
    );
    assert!(
        after.on_water > 0,
        "a cohort of mariners must cross the water"
    );
    assert!(
        after.land_beyond > 0,
        "a cohort of mariners must reach ground beyond its island"
    );
    // **The test can now assert arrival, and it could not before.** The
    // destination field holds one direction for each level 1 cell, so the
    // cohort reached the cell that held its target and then took the uniform
    // keyed draw inside it. The approach field resolves that cell at the
    // pitch of one tile.[^5]
    //
    // [^5]: Findings register, FND-315. `docs/FINDINGS.md`
    assert!(
        after.stood_on_target.is_some(),
        "a cohort of mariners must stand on the tile it was sent to"
    );
}

#[test]
fn the_controller_builds_a_mariner_and_sends_it_across() {
    let world = fixture_world(true);
    let island = smallest_island(&world);
    let (mut world, _) = seated_world(true, &island);
    let mut built = false;
    let mut sent = false;
    for _ in 0..TICKS {
        world.step(THREADS).expect("the step must run");
        for unit in world.soldiers().iter_faction(ISLAND) {
            if world.soldiers().unit_type(unit) != Some(MARINER) {
                continue;
            }
            built = true;
            if world.soldiers().sent(unit) == Some(Some(CROSSING_PLANE)) {
                sent = true;
            }
        }
    }
    assert!(
        built,
        "the controller must queue and build a mariner of its own accord"
    );
    assert!(
        sent,
        "the controller must send the mariner it built on the crossing plane"
    );
}

#[test]
fn a_crossing_run_gives_one_answer_at_any_thread_count() {
    let world = fixture_world(true);
    let island = smallest_island(&world);
    let mut standing = Vec::new();
    for threads in [1usize, 2, 7] {
        let (mut world, seat) = seated_world(true, &island);
        let cohort: Vec<Entity> = (0..COHORT)
            .map(|_| {
                world
                    .spawn_soldier(seat, ISLAND)
                    .expect("the seat admits the cohort")
            })
            .collect();
        world.set_unit_type_set(&cohort, MARINER);
        let target = nearest_shore_beyond(&world, &island, seat);
        world
            .send_units_to(&cohort, &[target], CROSSING_PLANE)
            .expect("the world holds the plane and every unit is live");
        for _ in 0..96 {
            world.step(threads).expect("the step must run");
        }
        let mut seen: Vec<(i32, i32)> = world
            .soldiers()
            .iter_faction(ISLAND)
            .filter_map(|unit| world.soldiers().address(unit))
            .map(|at| (at.q, at.r))
            .collect();
        seen.sort_unstable();
        standing.push(seen);
    }
    assert_eq!(standing[0], standing[1], "one thread and two must agree");
    assert_eq!(standing[0], standing[2], "one thread and seven must agree");
}

#[test]
fn a_faction_that_crossed_can_found_on_the_ground_it_reached() {
    let world = fixture_world(true);
    let island = smallest_island(&world);
    let (mut world, seat) = seated_world(true, &island);
    world.set_externally_controlled(ISLAND, true);
    let cohort: Vec<Entity> = (0..COHORT)
        .map(|_| {
            world
                .spawn_soldier(seat, ISLAND)
                .expect("the seat admits the cohort")
        })
        .collect();
    world.set_unit_type_set(&cohort, MARINER);
    let target = nearest_shore_beyond(&world, &island, seat);
    world
        .send_units_to(&cohort, &[target], CROSSING_PLANE)
        .expect("the world holds the plane and every unit is live");
    let mut landed = None;
    for _ in 0..TICKS {
        world.step(THREADS).expect("the step must run");
        if landed.is_some() {
            continue;
        }
        // The walk is over the cohort in the order the spawn made it, so the
        // place the test picks is a property of the run.
        landed = cohort
            .iter()
            .filter_map(|unit| world.soldiers().address(*unit))
            .find(|at| !island.contains(at) && world.admits_a_unit(*at));
    }
    let place = landed.expect("a mariner must stand on ground beyond the island");
    println!("a mariner reached {place:?}");

    // **The founding survey ranks the place the cohort reached.** The survey
    // reads the ground and never asks how a group would walk there, so ground
    // across water is a place it can choose once a unit can arrive.
    let survey = world
        .survey_places(&[place], GROUP, &[seat])
        .expect("the survey must run");
    println!(
        "the survey ranks it eligible: {:?}",
        survey.chosen().is_some()
    );

    let founded = world.found_settlement(place, ISLAND);
    assert!(
        founded.is_ok(),
        "a faction must be able to found on the ground it reached: {founded:?}"
    );
    let sites = world
        .settlements()
        .iter()
        .filter(|held| world.settlements().faction(*held) == Some(ISLAND))
        .count();
    println!("the faction now holds {sites} sites");
    assert_eq!(sites, 2, "the faction must hold a second site");
}

/// The destination plane that the island faction settles on.
///
/// The settling plane of a faction is its number raised by twice the faction
/// count, and the fixture holds two factions.
const SETTLING_PLANE: u16 = 4;

#[test]
fn the_controller_sends_a_settler_across_the_water() {
    // **The crossing and the founding compose through the type table alone.**
    // The settler row holds a water crossing and a settle group, so a settler
    // crosses the water as a mariner does. No rule in the engine names either
    // type, and nothing here drives a verb: the faction runs under its own
    // controller, which queues the settler and sends it on the settling
    // plane.
    //
    // **The settler does not reach the far shore of this fixture, and the
    // test does not claim that it does.** The island of this world sits
    // behind a sea about thirteen tiles wide on the route the target names,
    // and a lone settler starves after four or five ticks on water. The
    // founding beyond the water is proven from a place a unit reached, by
    // the test above. What is open is the target choice: the settling target
    // comes from a bounded sample drawn over the whole world, so it names a
    // place on the far mainland rather than the near shore that a mariner
    // reaches.
    let world = fixture_world(true);
    let island = smallest_island(&world);
    let (mut world, _) = seated_world(true, &island);
    let mut sent = false;
    let mut crossed = false;
    for _ in 0..TICKS {
        world.step(THREADS).expect("the step must run");
        for unit in world.soldiers().iter_faction(ISLAND) {
            if world.soldiers().sent(unit) != Some(Some(SETTLING_PLANE)) {
                continue;
            }
            sent = true;
            let Some(at) = world.soldiers().address(unit) else {
                continue;
            };
            if world.tile_kind(at) == Some(TileKind::Water) {
                crossed = true;
            }
        }
    }
    assert!(
        sent,
        "the controller must send a settler on the settling plane of its own accord"
    );
    assert!(
        crossed,
        "a settler must stand on water, which is what the crossing column buys it"
    );
}

/// The world invariant reads the crossing column of the unit, and not the
/// ground alone.
///
/// **A check that reads the ground alone states a second, stricter rule.**
/// The movement pass admits a step by the terrain capacity table, and that
/// table takes the crossing column of the type that steps. A check that drops
/// the column refuses a mariner the movement pass has just admitted, so a run
/// in which a mariner crosses loses an invariant the engine never broke.[^6]
///
/// The test reads the rule in both directions on one world. It steps until a
/// mariner stands on water, and it asserts that the world holds its
/// invariants. It then puts the defect back: it demotes that mariner to a
/// worker, whose row states no crossing, and it asserts that the invariant now
/// fails. A check that reads the ground alone fails the first assertion. A
/// check that reads nothing fails the second.[^7]
///
/// # References
///
/// [^6]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decision D2. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
/// [^7]: Testing rules, section 2a. `.agents/rules/testing.md`
#[test]
fn the_invariant_admits_a_mariner_on_water_and_refuses_a_worker_there() {
    let world = fixture_world(true);
    let island = smallest_island(&world);
    let (mut world, seat) = seated_world(true, &island);
    // The controller stands down, so the cohort keeps the order this test
    // gives it and the run reads one send.
    world.set_externally_controlled(ISLAND, true);
    let cohort: Vec<Entity> = (0..COHORT)
        .map(|_| {
            world
                .spawn_soldier(seat, ISLAND)
                .expect("the seat admits the cohort")
        })
        .collect();
    world.set_unit_type_set(&cohort, MARINER);
    let target = nearest_shore_beyond(&world, &island, seat);
    world
        .send_units_to(&cohort, &[target], CROSSING_PLANE)
        .expect("the world holds the plane and every unit is live");

    let mut afloat = None;
    for _ in 0..TICKS {
        world.step(THREADS).expect("the step must run");
        assert!(
            world.check_invariants(),
            "the world lost an invariant while a mariner crossed"
        );
        afloat = cohort.iter().copied().find(|unit| {
            world
                .soldiers()
                .address(*unit)
                .is_some_and(|at| world.tile_kind(at) == Some(TileKind::Water))
        });
        if afloat.is_some() {
            break;
        }
    }
    let afloat = afloat.expect("a mariner of the cohort must stand on open water");
    let at = world
        .soldiers()
        .address(afloat)
        .expect("the mariner is live");
    println!("the mariner stands on water at {at:?}");

    // Put the defect back. The worker row states no crossing, so the same
    // unit on the same tile is a unit the ground refuses.
    assert!(
        world.set_unit_type(afloat, WORKER),
        "the arena must take the worker row for a live unit"
    );
    assert!(
        !world.check_invariants(),
        "the invariant admitted a worker that stands on open water"
    );
}
