//! A faction plans its roads and zones with one solver.
//!
//! A plan is a bounded list of projects. A project is one tile and one
//! category. One solver writes the plan at the controller stage, in a fixed
//! pass count. A unit builds a category that asks for no held ground only
//! inside a project, and an idle unit takes the nearest project.[^1]
//!
//! Every test here goes through the public interface of the world.[^2] Each
//! one reaches an extreme rather than the typical case: two paths that tie on
//! cost, two projects at one distance from one unit, a plan at its bound, a
//! pair past the search radius, and a world with nothing to plan.[^3]
//!
//! **A test that reads the work done cannot tell a stopped build from a
//! raised one**, so a test of the refusal reads whether an entry exists at
//! all.[^4]
//!
//! # References
//!
//! [^1]: ADR-0152, a faction plans its roads and zones with one solver, decisions D1 to D5. `docs/adrs/accepted/adr-0152-a-faction-plans-its-roads-and-zones-with-one-solver.md`
//! [^2]: Testing rules, section 6. `.agents/rules/testing.md`
//! [^3]: Testing rules, section 2a. `.agents/rules/testing.md`
//! [^4]: Findings register, FND-492. `docs/FINDINGS.md`

use cachette_core::holding::ReachRules;
use cachette_core::plan::PlanRules;
use cachette_core::upgrade::{BuildRefusal, UpgradeCategory};
use cachette_core::{Axial, Entity, FactionId, Project, World, WorldConfig, SUBSYSTEM_CENSUS};

/// The extent that these tests read.
///
/// The extent is wider than the coarsest lattice spacing of the ground
/// generator, so the world holds every kind of ground.[^1]
///
/// # References
///
/// [^1]: Findings register, FND-054. `docs/FINDINGS.md`
const WIDTH: u32 = 96;
/// The number of rows of that extent.
const HEIGHT: u32 = 96;
/// The seed that these tests read.
const SEED: u64 = 641;
/// The faction that plans.
const ZERO: FactionId = FactionId(0);

/// Builds a world with no city and no plan.
fn bare(seed: u64) -> World {
    let mut world = World::new(WorldConfig {
        width: WIDTH,
        height: HEIGHT,
        seed,
        faction_count: 4,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    })
    .expect("the extent must describe a world");
    world
        .set_choice_schedule(0)
        .expect("the exponent is inside the range");
    // The default reach, not a reach that covers the world. A faction that
    // held every tile would win on territory, and the game end stops every
    // controller, so the fixture would then measure a stopped controller.[^1]
    //
    // [^1]: ADR-0148, a game end is recorded once and stops the controllers. `docs/adrs/accepted/adr-0148-a-game-end-is-recorded-once-and-stops-the-controllers.md`
    world.set_reach_rules(ReachRules::DEFAULT);
    world
}

/// Returns every address of the extent, in row-major order.
fn addresses() -> Vec<Axial> {
    let mut all = Vec::with_capacity((WIDTH * HEIGHT) as usize);
    for r in 0..HEIGHT {
        for q in 0..WIDTH {
            all.push(Axial::new(q as i32, r as i32));
        }
    }
    all
}

/// Returns the first address that admits a unit, from one corner.
fn open_from(world: &World, from: Axial) -> Axial {
    addresses()
        .into_iter()
        .filter(|address| world.admits_a_unit(*address))
        .min_by_key(|address| (address.distance(from), address.q, address.r))
        .expect("the world admits a unit somewhere")
}

/// Builds a world with two cities of one faction, far enough apart that a
/// road between them is a road and not one tile.
fn two_cities(seed: u64) -> (World, Axial, Axial) {
    let mut world = bare(seed);
    let seat = open_from(&world, Axial::new(20, 20));
    let other = open_from(&world, Axial::new(26, 20));
    assert_ne!(seat, other, "the fixture needs two places");
    // **The two places must sit inside the search radius.** A pair further
    // apart than the radius yields no project, and the fixture would then
    // measure the radius rather than the path.
    assert!(
        seat.distance(other) <= world.plan_rules().radius(),
        "the fixture put its two places {} steps apart, past the radius of {}",
        seat.distance(other),
        world.plan_rules().radius()
    );
    // **The founding verb records the seat, and the settlement verb does
    // not.** A faction with no seat receives no evaluation, so a fixture that
    // only founded a settlement would measure a controller that never
    // ran.[^1]
    //
    // [^1]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decision D7. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
    world
        .found_group_at(seat, 4, ZERO)
        .expect("the ground admits a founding");
    world
        .found_settlement(other, ZERO)
        .expect("the ground admits a second city");
    (world, seat, other)
}

/// Returns one census count by name.
fn census(world: &World, name: &str) -> i64 {
    SUBSYSTEM_CENSUS
        .iter()
        .find(|row| row.name == name)
        .map(|row| (row.read)(world))
        .expect("the census holds the row")
}

/// Returns the plan of the first faction as a plain list.
fn plan_of(world: &World) -> Vec<Project> {
    world.plan_of(ZERO).to_vec()
}

// ---------------------------------------------------------------------------
// D1. The plan is bounded, ordered and hashed
// ---------------------------------------------------------------------------

#[test]
fn a_plan_holds_its_projects_in_tile_order_and_one_project_for_each_tile() {
    let mut world = bare(SEED);
    let places: Vec<Axial> = addresses()
        .into_iter()
        .filter(|address| world.admits_a_unit(*address))
        .take(6)
        .collect();
    // The writes go in descending tile order, so the ascending order of the
    // plan is the register's own and never the order the caller wrote in.
    for address in places.iter().rev() {
        world
            .zone_project(ZERO, *address, UpgradeCategory::ROAD)
            .expect("the plan takes a road anywhere");
    }
    let plan = plan_of(&world);
    assert_eq!(plan.len(), places.len());
    assert!(
        plan.windows(2).all(|pair| pair[0].tile < pair[1].tile),
        "the plan is not in ascending tile order: {plan:?}"
    );
    assert!(
        plan.iter().all(|project| project.padding == [0; 3]),
        "a project carries undeclared padding"
    );

    // A second write for one tile replaces the project and adds no row.
    let before = plan.len();
    world
        .zone_project(ZERO, places[0], UpgradeCategory::ROAD)
        .expect("the plan takes the tile again");
    assert_eq!(
        world.plan_of(ZERO).len(),
        before,
        "a tile gained a second project"
    );
    assert!(world.check_invariants());
}

/// The drop row and the refusal row count one act once each.
///
/// **A reader adds the two rows.** They once overlapped, because a write past
/// the bound raised both, and a run that dropped a thousand projects then read
/// two thousand turned-away writes.[^4]
///
/// The fixture reaches both acts in one world: a write the bound drops, and a
/// write that names a faction the register does not hold.
///
/// # References
///
/// [^4]: Findings register, FND-496. `docs/FINDINGS.md`
#[test]
fn a_drop_and_a_refusal_are_counted_apart() {
    let mut world = bare(SEED);
    world.set_plan_rules(world.plan_rules().with_bound(2));
    let places: Vec<Axial> = addresses()
        .into_iter()
        .filter(|address| world.admits_a_unit(*address))
        .take(3)
        .collect();
    for address in places.iter().take(2) {
        world
            .zone_project(ZERO, *address, UpgradeCategory::ROAD)
            .expect("the plan is not full yet");
    }
    let dropped = census(&world, "projects_dropped");
    let refused = census(&world, "projects_refused");

    // The bound drops this write. It is a drop and nothing else.
    assert!(world
        .zone_project(ZERO, places[2], UpgradeCategory::ROAD)
        .is_err());
    assert_eq!(
        census(&world, "projects_dropped"),
        dropped + 1,
        "the drop row must count the write the bound turned away"
    );
    assert_eq!(
        census(&world, "projects_refused"),
        refused,
        "one act raised both rows, so the two cannot be added"
    );

    // A write that names no faction is a refusal and nothing else.
    // The world holds four factions, so this number names none of them.
    let stranger = FactionId(4);
    assert!(world
        .zone_project(stranger, places[0], UpgradeCategory::ROAD)
        .is_err());
    assert_eq!(
        census(&world, "projects_refused"),
        refused + 1,
        "the refusal row must count the write that named no faction"
    );
    assert_eq!(
        census(&world, "projects_dropped"),
        dropped + 1,
        "a refusal that is not a drop raised the drop row"
    );
    assert!(world.check_invariants());
}

#[test]
fn a_plan_at_its_bound_drops_the_next_project_and_counts_the_drop() {
    // The extreme: the plan is full. A fixture with a large bound would never
    // reach the drop, so the bound is set to what the fixture writes.
    let mut world = bare(SEED);
    world.set_plan_rules(world.plan_rules().with_bound(3));
    let places: Vec<Axial> = addresses()
        .into_iter()
        .filter(|address| world.admits_a_unit(*address))
        .take(4)
        .collect();
    for address in places.iter().take(3) {
        world
            .zone_project(ZERO, *address, UpgradeCategory::ROAD)
            .expect("the plan is not full yet");
    }
    assert_eq!(census(&world, "projects_dropped"), 0);
    assert!(world
        .zone_project(ZERO, places[3], UpgradeCategory::ROAD)
        .is_err());
    assert_eq!(world.plan_of(ZERO).len(), 3, "the plan grew past its bound");
    assert_eq!(census(&world, "projects_dropped"), 1);
    assert!(world.check_invariants());
}

#[test]
fn the_plan_enters_the_state_hash() {
    let mut world = bare(SEED);
    let before = world.state_hash().finish();
    let address = open_from(&world, Axial::new(10, 10));
    world
        .zone_project(ZERO, address, UpgradeCategory::ROAD)
        .expect("the plan takes a road anywhere");
    let after = world.state_hash().finish();
    assert_ne!(
        before, after,
        "two worlds that differ in a plan hash the same"
    );

    // The same plan in another faction is another world.
    let mut other = bare(SEED);
    other
        .zone_project(FactionId(1), address, UpgradeCategory::ROAD)
        .expect("the plan takes a road anywhere");
    assert_ne!(
        after,
        other.state_hash().finish(),
        "a project of one faction hashes as a project of another"
    );
}

// ---------------------------------------------------------------------------
// D2. The solver runs a fixed pass count
// ---------------------------------------------------------------------------

#[test]
fn the_solver_runs_the_same_pass_count_whatever_the_input() {
    // Two worlds that differ as much as the fixture can make them: one has a
    // seat and two cities to join, the other has one city and nothing to
    // plan. Both run the same number of passes for each faction with a seat.
    let (mut busy, _, _) = two_cities(SEED);
    let mut quiet = bare(SEED);
    let seat = open_from(&quiet, Axial::new(20, 20));
    quiet
        .found_group_at(seat, 4, ZERO)
        .expect("the ground admits a founding");

    // A third world can write nothing at all: its bound is zero, so every
    // write is dropped. A solver that stopped when a pass wrote nothing would
    // run fewer passes here than in the busy world.
    let mut full = bare(SEED);
    let full_seat = open_from(&full, Axial::new(20, 20));
    full.found_group_at(full_seat, 4, ZERO)
        .expect("the ground admits a founding");
    full.set_plan_rules(full.plan_rules().with_bound(0));

    let passes = i64::from(busy.plan_rules().solver_passes());
    for tick in 1..=4 {
        busy.step(1).expect("the step runs");
        quiet.step(1).expect("the step runs");
        full.step(1).expect("the step runs");
        assert_eq!(
            census(&busy, "plan_passes"),
            passes * tick,
            "the busy world ran a different number of passes"
        );
        assert_eq!(
            census(&quiet, "plan_passes"),
            census(&busy, "plan_passes"),
            "two worlds ran a different number of passes"
        );
        assert_eq!(
            census(&full, "plan_passes"),
            census(&busy, "plan_passes"),
            "a world that can write nothing ran a different number of passes"
        );
    }
}

#[test]
fn a_faction_with_two_unconnected_places_zones_a_way_between_them() {
    let (mut world, seat, other) = two_cities(SEED);
    world.step(1).expect("the step runs");
    let plan = plan_of(&world);
    assert!(
        !plan.is_empty(),
        "the solver planned nothing for two cities"
    );
    assert!(
        plan.iter()
            .all(|project| project.category == UpgradeCategory::ROAD),
        "the solver zoned a category that does not join two places: {plan:?}"
    );
    // The projects lie between the two places: each is inside the search
    // radius of the seat, and one of them touches the far city.
    let radius = world.plan_rules().radius();
    for project in &plan {
        let address = world
            .grid()
            .address_of(project.tile)
            .expect("a project names a tile of this world");
        assert!(
            address.distance(seat) <= radius,
            "a project sits outside the search radius"
        );
    }
    let reaches = plan.iter().any(|project| {
        world
            .grid()
            .address_of(project.tile)
            .is_some_and(|address| address.distance(other) <= 1)
    });
    assert!(reaches, "no project reaches the second city: {plan:?}");
    assert_eq!(census(&world, "projects_zoned"), plan.len() as i64);
}

#[test]
fn the_same_world_gives_the_same_plan_on_every_run() {
    let (mut first, _, _) = two_cities(SEED);
    let (mut second, _, _) = two_cities(SEED);
    for _ in 0..3 {
        first.step(1).expect("the step runs");
        second.step(1).expect("the step runs");
    }
    assert_eq!(
        plan_of(&first),
        plan_of(&second),
        "two runs planned differently"
    );
    assert!(!plan_of(&first).is_empty(), "the fixture planned nothing");
}

#[test]
fn the_solver_gives_one_plan_at_every_thread_count() {
    let plans: Vec<Vec<Project>> = [1usize, 2, 12]
        .into_iter()
        .map(|threads| {
            let (mut world, _, _) = two_cities(SEED);
            for _ in 0..3 {
                world.step(threads).expect("the step runs");
            }
            plan_of(&world)
        })
        .collect();
    assert_eq!(plans[0], plans[1], "one thread and two threads disagree");
    assert_eq!(
        plans[1], plans[2],
        "two threads and twelve threads disagree"
    );
    assert!(!plans[0].is_empty(), "the fixture planned nothing");
}

#[test]
fn a_place_past_the_search_radius_yields_no_project() {
    // The extreme: the second city is outside the window the search builds,
    // so no path reaches it and no project names its tile.
    //
    // **A place is not a settlement.** A settlement past the radius is
    // reached, because a way between two settlements chains windows. The tile
    // of a settlement still takes no project, because a settlement carries a
    // settlement and never a way. The window itself still answers nothing
    // past its radius, which is what this test reads.
    let (mut world, _, other) = two_cities(SEED);
    world.set_plan_rules(world.plan_rules().with_bound(64));
    let tight = world.plan_rules();
    world.set_plan_rules(PlanRules::new(
        tight.bound(),
        tight.solver_passes(),
        tight.projects_per_pass(),
        tight.path_passes(),
        1,
    ));
    world.step(1).expect("the step runs");
    let far = world
        .grid()
        .index_of(other)
        .expect("the second city is inside the world");
    assert!(
        world
            .plan_of(ZERO)
            .iter()
            .all(|project| project.tile != far),
        "a project named a place past the radius"
    );
}

// ---------------------------------------------------------------------------
// D3. A path is deterministic, and a tie takes the lower tile index
// ---------------------------------------------------------------------------

#[test]
fn two_ways_that_tie_on_cost_resolve_by_the_lower_tile_index() {
    // The extreme: a pair whose two ends have more than one cheapest way
    // between them. The fixture searches the world for such a pair rather
    // than assuming one, because a pair with one cheapest way would let the
    // cost decide and the tie rule would never fire.
    let world = bare(SEED);
    let (from, to, rival) =
        a_tied_pair(&world).expect("the world holds a pair with two cheapest ways");
    let path = world.planned_path(from, to);
    assert!(!path.is_empty(), "the search found no way");
    assert_eq!(*path.first().expect("the way is not empty"), from);
    assert_eq!(*path.last().expect("the way is not empty"), to);
    for pair in path.windows(2) {
        assert_eq!(pair[0].distance(pair[1]), 1, "the way skips a tile");
    }
    // The step into the far end came from the neighbour with the lower tile
    // index, and the rival neighbour ties with it on cost.
    let last = path[path.len() - 2];
    let taken = world
        .grid()
        .index_of(last)
        .expect("the way stays inside the world");
    let other = world
        .grid()
        .index_of(rival)
        .expect("the rival is inside the world");
    assert!(
        taken < other,
        "the way came through tile {taken:?} when tile {other:?} ties and is lower"
    );
}

/// Returns a pair of places with two cheapest ways between them.
///
/// The pair is a start, a far end, and a second neighbour of the far end that
/// costs the same to reach as the one the way took. A fixture that assumed
/// such a pair would measure whatever the ground gave it.[^1]
///
/// # References
///
/// [^1]: Testing rules, section 2a. `.agents/rules/testing.md`
fn a_tied_pair(world: &World) -> Option<(Axial, Axial, Axial)> {
    // The search is bounded, so a fixture that cannot find a tie fails
    // quickly rather than walking the world.
    let region: Vec<Axial> = addresses()
        .into_iter()
        .filter(|address| (10..40).contains(&address.q) && (10..40).contains(&address.r))
        .collect();
    for from in region.iter().copied() {
        if !world.admits_a_unit(from) {
            continue;
        }
        for to in region
            .iter()
            .copied()
            .filter(|address| address.distance(from) >= 3 && address.distance(from) <= 6)
        {
            if !world.admits_a_unit(to) {
                continue;
            }
            let path = world.planned_path(from, to);
            if path.len() < 3 {
                continue;
            }
            let last = path[path.len() - 2];
            let taken = world.planned_path_cost(from, last)?;
            let rival = world
                .grid()
                .neighbours(to)
                .into_iter()
                .flatten()
                .filter(|side| *side != last && world.admits_a_unit(*side))
                .find(|side| world.planned_path_cost(from, *side) == Some(taken));
            if let Some(rival) = rival {
                return Some((from, to, rival));
            }
        }
    }
    None
}

// ---------------------------------------------------------------------------
// D3 and D4. A build outside a project is refused
// ---------------------------------------------------------------------------

#[test]
fn a_build_that_no_project_zones_is_refused_by_the_verb_and_ignored_by_the_pass() {
    let mut world = bare(SEED);
    let address = open_from(&world, Axial::new(10, 10));
    let unit = world
        .spawn_soldier(address, ZERO)
        .expect("the ground admits a unit");

    // **The verb refuses.** A caller learns at once, and the refusal counts.
    let before = census(&world, "projects_refused");
    assert!(
        world.order_build(unit, UpgradeCategory::ROAD).is_err(),
        "the verb took a road that no project zones"
    );
    assert_eq!(
        world.build_order(unit),
        Some(None),
        "a refused order was stored"
    );
    assert!(census(&world, "projects_refused") > before);

    // **The pass ignores.** The order is given while a project stands, and
    // the project is then cleared. The pass drops the intent, so no entry
    // appears. A test that read the work done could not tell a stopped build
    // from a raised one, so this reads whether an entry exists at all.[^1]
    //
    // [^1]: Findings register, FND-492. `docs/FINDINGS.md`
    world
        .zone_project(ZERO, address, UpgradeCategory::ROAD)
        .expect("the plan takes a road anywhere");
    world
        .order_build(unit, UpgradeCategory::ROAD)
        .expect("the plan zones the tile");
    assert!(
        world.clear_project(ZERO, address),
        "the plan held the project"
    );
    for _ in 0..12 {
        world.step(1).expect("the step runs");
    }
    assert_eq!(
        world.upgrade_at(address),
        None,
        "a road grew on a tile that no project zones"
    );
}

#[test]
fn a_build_the_plan_zones_finishes_and_the_census_counts_it() {
    let mut world = bare(SEED);
    let address = open_from(&world, Axial::new(10, 10));
    world
        .zone_project(ZERO, address, UpgradeCategory::ROAD)
        .expect("the plan takes a road anywhere");
    // **A crowd, not one builder.** A unit wanders off its tile on the next
    // step, so one builder spreads one unit of work over many tiles and
    // nothing ever finishes. The crowd finishes the work in the first tick,
    // and the fixture then measures the build and not the walk.
    let crowd: Vec<Entity> = (0..24)
        .map(|_| {
            world
                .spawn_soldier(address, ZERO)
                .expect("a spawn may over-fill a tile")
        })
        .collect();
    for unit in &crowd {
        world
            .order_build(*unit, UpgradeCategory::ROAD)
            .expect("the plan zones the tile");
    }
    for _ in 0..4 {
        world.step(1).expect("the step runs");
        if world.finished_upgrade(address).is_some() {
            break;
        }
    }
    assert_eq!(
        world.finished_upgrade(address),
        Some(UpgradeCategory::ROAD),
        "the zoned road never finished"
    );
    assert!(world.check_invariants());
}

// ---------------------------------------------------------------------------
// D5. An idle unit takes the nearest project
// ---------------------------------------------------------------------------

#[test]
fn two_projects_at_one_distance_from_a_unit_resolve_by_the_lower_tile_index() {
    // The extreme: the unit stands exactly between two projects, in opposite
    // directions. A fixture that put one project nearer would never reach the
    // tie rule, and the assertion would then measure the distance.
    //
    // The test drives the assignment the controller calls, and not a rule of
    // its own.[^1]
    //
    // [^1]: Testing rules, section 5. `.agents/rules/testing.md`
    let mut world = bare(SEED);
    let seat = open_from(&world, Axial::new(30, 30));
    let span = 4;
    let west = Axial::new(seat.q - span, seat.r);
    let east = Axial::new(seat.q + span, seat.r);
    assert_eq!(
        west.distance(seat),
        east.distance(seat),
        "the fixture is not a tie"
    );
    for place in [west, east] {
        world
            .zone_project(ZERO, place, UpgradeCategory::ROAD)
            .expect("the plan takes a road anywhere");
    }
    let west_tile = world
        .grid()
        .index_of(west)
        .expect("the place is inside the world");
    let east_tile = world
        .grid()
        .index_of(east)
        .expect("the place is inside the world");
    let lower = west_tile.min(east_tile);
    let unit = world
        .spawn_soldier(seat, ZERO)
        .expect("the ground admits a unit");

    let taken = world
        .project_for(ZERO, unit)
        .expect("the plan holds two projects");
    assert_eq!(
        taken.tile, lower,
        "the tie went to the higher tile index: took {taken:?}"
    );
    // A unit of another faction takes nothing from this plan.
    let stranger = world
        .spawn_soldier(seat, FactionId(1))
        .expect("the ground admits a unit");
    assert_eq!(world.project_for(ZERO, stranger), None);
}

#[test]
fn the_controller_sends_its_idle_units_to_the_projects_it_zoned() {
    let (mut world, seat, _) = two_cities(SEED);
    // A second faction, far away, so that no faction holds the world and the
    // game end does not stop the controllers before the order applies.
    // **No evaluation draws.** The demonstration controller otherwise draws a
    // category and orders the whole faction to build it where it stands, and
    // that build takes the tile a project zones. The fixture removes the
    // competing draw so that it measures the project order alone.
    world.set_controller_evaluations(0);
    let far = open_from(&world, Axial::new(80, 80));
    world
        .found_group_at(far, 4, FactionId(1))
        .expect("the ground admits a founding");
    world.step(1).expect("the step runs");
    assert!(!plan_of(&world).is_empty(), "the solver planned nothing");
    // The founding put units at the seat, and the seat carries no project.
    // The order sends a unit toward the project nearest to it, and the build
    // order sticks on the tick the unit stands on a project. The stage
    // repeats the order on every tick, so the fixture steps until it does.
    //
    // The demonstration controller also draws a category and orders the whole
    // faction to build it where it stands. That order competes with this one,
    // so the fixture reads whether the project order ever reached a unit and
    // not what the last tick left.
    let mut took = false;
    let mut sent = false;
    for _ in 0..40 {
        world.step(1).expect("the step runs");
        sent |= world
            .soldiers()
            .iter()
            .filter(|unit| world.soldiers().faction(*unit) == Some(ZERO))
            .any(|unit| world.soldiers().sent(unit) != Some(None));
        took |= world
            .soldiers()
            .iter()
            .filter(|unit| world.soldiers().faction(*unit) == Some(ZERO))
            .any(|unit| world.build_order(unit) == Some(Some(UpgradeCategory::ROAD)));
    }
    assert!(
        sent,
        "the order sent no unit toward a project after {seat:?} was seated"
    );
    assert!(
        took,
        "no unit of the faction ever took the category its project names"
    );
}

// ---------------------------------------------------------------------------
// The chain end to end: the demonstration world finishes a road
// ---------------------------------------------------------------------------

/// The ticks each run below is given to finish one project.
///
/// The bound is a test bound and not a balance value. A run that closes the
/// chain closes it long before the bound, and the number only stops a broken
/// engine from running for ever.
const RUN_TICKS: u32 = 600;

/// The seeds that the run below takes.
///
/// **The chain is seed dependent, so one seed measures one world.** The list
/// holds eight seeds that were written before any of them was run, so no seed
/// here was chosen for the answer it gives.
const RUN_SEEDS: [u64; 8] = [
    0xf37f_d6bd_d8b6_4a3a,
    0xea33_5e28_791d_60da,
    0x58d0_24d9_7012_dd6a,
    0x0000_0000_0000_0001,
    0x0123_4567_89ab_cdef,
    0xdead_beef_cafe_f00d,
    0x0000_0000_0000_02a1,
    0xffff_ffff_ffff_ffff,
];

/// The seeds of the list above that must close the chain.
///
/// **The bar is not every seed, because the chain does not close at every
/// seed.** A faction whose units never reach a clean project tile finishes
/// nothing, and half of the list holds such a world. The open item holds the
/// two causes.[^1]
///
/// The bar is the reading of the run. The same eight seeds close at one seed
/// with the plan clause removed, so the bar fails on the defect by a wide
/// margin. Raise it when the open item lands.
///
/// # References
///
/// [^1]: Backlog item 0502. `docs/backlog/proposed/0502-let-a-faction-re-aim-its-project-order-and-keep-its-plan-live.md`
const RUN_SEEDS_THAT_MUST_CLOSE: usize = 4;

/// Seeds a demonstration world, runs it, and reports what it finished.
///
/// Returns the projects the plan counted finished and the roads that stand.
fn run_a_demonstration_world(seed: u64) -> (i64, usize) {
    let mut world = World::new(WorldConfig {
        width: WIDTH,
        height: HEIGHT,
        seed,
        faction_count: 3,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    })
    .expect("the extent must describe a world");
    // The engine seeds itself, as the demonstration does. No verb of this
    // test founds anything, and no draw is turned off.
    let seated = world.seed_world().expect("the world seeds once");
    assert!(
        seated.iter().any(|outcome| outcome.founding().is_some()),
        "the seeding put no faction on the ground at seed {seed:#018x}"
    );
    for _ in 0..RUN_TICKS {
        world.step(1).expect("the step runs");
    }
    assert!(world.check_invariants());
    // A count is not a road. The census could rise for a project a caller
    // cleared, so the run reads the ground as well.
    let standing = addresses()
        .into_iter()
        .filter(|address| world.finished_upgrade(*address) == Some(UpgradeCategory::ROAD))
        .count();
    (census(&world, "projects_finished"), standing)
}

/// The demonstration world runs, and a road the plan zoned stands at the end.
///
/// **This test drives the engine and not the mechanism.** It seeds the world
/// the demonstration seeds, it calls no verb of its own, and it removes no
/// draw. Every earlier test of this file either called the build verb by hand
/// or turned the evaluation draws off, so none of them ever ran the whole
/// chain: the solver zones, the controller orders, the pass builds, and the
/// plan counts the project finished.[^1] [^2]
///
/// The runs finished one project between them across eight seeds before the
/// plan bound every build. The finding holds the census of the run that showed
/// it.[^3]
///
/// # References
///
/// [^1]: Testing rules, section 5. `.agents/rules/testing.md`
/// [^2]: Recurring defect shapes, shape 3. `.agents/rules/recurring-defects.md`
/// [^3]: Findings register, FND-496. `docs/FINDINGS.md`
#[test]
fn the_demonstration_world_finishes_a_project_and_a_road_stands() {
    let mut closed = 0usize;
    let mut report = String::new();
    for seed in RUN_SEEDS {
        let (finished, standing) = run_a_demonstration_world(seed);
        if finished > 0 && standing > 0 {
            closed += 1;
        }
        report.push_str(&format!(
            "\n  {seed:#018x}: finished {finished}, roads standing {standing}"
        ));
    }
    assert!(
        closed >= RUN_SEEDS_THAT_MUST_CLOSE,
        "the chain closed at {closed} of {} seeds in {RUN_TICKS} ticks, \
         and the bar is {RUN_SEEDS_THAT_MUST_CLOSE}:{report}",
        RUN_SEEDS.len()
    );
}

/// A project of the faction refuses a build order that names another
/// category, whatever the row asks for.
///
/// **The plan is the bound on what a unit builds, and not only on where it
/// reaches.** Without this clause a faction plants one category on a tile its
/// own plan zones for another. A tile carries one upgrade, so the project can
/// then never be built, and the plan holds a project that nothing can
/// finish.[^1] [^2]
///
/// # References
///
/// [^1]: ADR-0152, a faction plans its roads and zones with one solver, decisions D3 and D4. `docs/adrs/accepted/adr-0152-a-faction-plans-its-roads-and-zones-with-one-solver.md`
/// [^2]: Findings register, FND-496. `docs/FINDINGS.md`
#[test]
fn a_project_refuses_a_build_order_that_names_another_category() {
    let mut world = bare(SEED);
    let address = open_from(&world, Axial::new(10, 10));
    // The faction holds the ground, so the ground rule permits the terrace.
    // Only the project stands between the order and the tile.
    world
        .found_group_at(address, 4, ZERO)
        .expect("the ground admits a founding");
    world
        .zone_project(ZERO, address, UpgradeCategory::ROAD)
        .expect("the plan takes a road anywhere");
    let unit = world
        .spawn_soldier(address, ZERO)
        .expect("a spawn may over-fill a tile");
    assert_eq!(
        world.order_build(unit, UpgradeCategory::TERRACE),
        Err(BuildRefusal::ProjectHoldsAnother {
            zoned: UpgradeCategory::ROAD,
            asked: UpgradeCategory::TERRACE,
        }),
        "the project of the faction did not refuse the competing category"
    );
    // The category the project names is still permitted, so the clause
    // refuses the competing order and nothing else.
    world
        .order_build(unit, UpgradeCategory::ROAD)
        .expect("the project zones the road");
    // The pass applies the same rule, so a build the verb refused finishes
    // nothing. The tile carries no terrace after the world steps.
    for _ in 0..4 {
        world.step(1).expect("the step runs");
    }
    assert_ne!(
        world.upgrade_at(address).map(|site| site.category),
        Some(UpgradeCategory::TERRACE),
        "the pass built the category the verb refused"
    );
}

// ---------------------------------------------------------------------------
// D3. A way joins two settlements that stand further apart than one window
// ---------------------------------------------------------------------------

/// How far apart the two settlements of the join fixture stand.
///
/// **The gap must pass the search radius, or the fixture measures nothing.**
/// One window centred on either settlement then holds the other, and a single
/// window answers the join. The founding rule keeps two settlements of one
/// faction further apart than that, so a fixture inside the radius models a
/// world the founding verb never builds.[^1]
///
/// The gap is the founding distance, so the fixture stands at the closest two
/// settlements of one faction ever stand.
///
/// # References
///
/// [^1]: The founding distance. `crates/cachette-core/src/founding.rs`
const JOIN_GAP: u32 = 16;

/// How many ticks the join fixture runs.
///
/// A way of the gap above holds about that many tiles, and each tile takes
/// the work of one road. The count is the ticks the builders need, with room
/// for the ground that sends the way round.
const JOIN_TICKS: u32 = 900;

/// How many builders the join fixture founds.
///
/// A unit builds the project nearest to it, so a way of many tiles needs many
/// units. The count is large enough that every tile of the way carries a
/// builder before the run ends.
const JOIN_BUILDERS: u32 = 8;

/// Builds a world with two cities of one faction, further apart than one
/// window is wide.
///
/// The seat carries a group of builders. The second city stands past the
/// search radius, so no single window holds both, and the solver reaches it
/// only by chaining windows.
fn two_far_cities(seed: u64) -> (World, Axial, Axial) {
    let mut world = bare(seed);
    let seat = open_from(&world, Axial::new(20, 40));
    let other = open_from(&world, Axial::new(20 + JOIN_GAP as i32, 40));
    assert_ne!(seat, other, "the fixture needs two places");
    // **The extreme this fixture reaches is the gap.** A fixture inside the
    // radius measures one window, and one window is what the founding rule
    // never gives.[^1]
    //
    // [^1]: Testing rules, section 2a. `.agents/rules/testing.md`
    assert!(
        seat.distance(other) > world.plan_rules().radius(),
        "the fixture put its two places {} steps apart, inside the radius of {}",
        seat.distance(other),
        world.plan_rules().radius()
    );
    world
        .found_group_at(seat, JOIN_BUILDERS, ZERO)
        .expect("the ground admits a founding");
    world
        .found_settlement(other, ZERO)
        .expect("the ground admits a second city");
    // A second faction, far away, so that no faction holds the world. A game
    // end stops every controller, and a stopped controller plans nothing and
    // orders nobody.[^1]
    //
    // [^1]: ADR-0148, a game end is recorded once and stops the controllers. `docs/adrs/accepted/adr-0148-a-game-end-is-recorded-once-and-stops-the-controllers.md`
    let far = open_from(&world, Axial::new(80, 80));
    world
        .found_group_at(far, 4, FactionId(1))
        .expect("the ground admits a second faction");
    // **No evaluation draws.** The demonstration controller otherwise draws a
    // category and orders the whole faction to build it where it stands, and
    // that order competes with the project order for the same units. The
    // fixture removes the competing draw so that it measures the way, in the
    // same way the project order test does.
    world.set_controller_evaluations(0);
    (world, seat, other)
}

/// Reports whether a road that stands joins two addresses.
///
/// The walk crosses a tile that carries a finished road, and it ends when it
/// touches the far address. A settlement carries a settlement and never a
/// road, so a walk that asked the far tile for a road would join nothing.
fn a_road_joins(world: &World, from: Axial, to: Axial) -> bool {
    let mut seen = vec![from];
    let mut queue = vec![from];
    while let Some(here) = queue.pop() {
        for neighbour in world.grid().neighbours(here).into_iter().flatten() {
            if neighbour == to {
                return true;
            }
            if seen.contains(&neighbour) {
                continue;
            }
            if world.finished_upgrade(neighbour) != Some(UpgradeCategory::ROAD) {
                continue;
            }
            seen.push(neighbour);
            queue.push(neighbour);
        }
    }
    false
}

/// The solver zones a way to a settlement that stands past one window.
///
/// **This is the hole the chain of windows closes.** The solver anchored one
/// window on the seat, and it asked for a way between the seat and a place
/// inside that window. Two settlements of one faction stand further apart
/// than the window is wide, so no settlement was ever a candidate and no
/// faction ever planned a way to its own second city.[^1]
///
/// # References
///
/// [^1]: ADR-0152, a faction plans its roads and zones with one solver, decision D3. `docs/adrs/accepted/adr-0152-a-faction-plans-its-roads-and-zones-with-one-solver.md`
#[test]
fn a_faction_zones_a_way_to_a_settlement_past_one_window() {
    let (mut world, seat, other) = two_far_cities(SEED);
    world.step(1).expect("the step runs");
    let plan = plan_of(&world);
    assert!(
        !plan.is_empty(),
        "the solver planned nothing for two cities"
    );
    let radius = world.plan_rules().radius();
    let places: Vec<Axial> = plan
        .iter()
        .filter(|project| project.category == UpgradeCategory::ROAD)
        .filter_map(|project| world.grid().address_of(project.tile))
        .collect();
    assert!(
        places.iter().any(|address| address.distance(seat) > radius),
        "no project stands past one window of the seat, so no chain ran: {places:?}"
    );
    assert!(
        places.iter().any(|address| address.distance(other) <= 1),
        "no project reaches the second city: {places:?}"
    );
}

/// The chain of windows gives one way at every thread count.
///
/// The way between two settlements is the one target that reads more than one
/// window, so it is the one that a thread count could reorder. The chain runs
/// in one order and it takes the same number of windows in every world, so
/// three runs at three thread counts give one plan.[^1] [^2]
///
/// # References
///
/// [^1]: ADR-0001, one binary gives one answer at any thread count, decision D1. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
/// [^2]: ADR-0005, a solver runs a fixed iteration count, decision D1. `docs/adrs/accepted/adr-0005-a-solver-runs-a-fixed-iteration-count.md`
#[test]
fn the_chain_of_windows_gives_one_way_at_every_thread_count() {
    let plans: Vec<Vec<Project>> = [1usize, 2, 12]
        .into_iter()
        .map(|threads| {
            let (mut world, _, _) = two_far_cities(SEED);
            for _ in 0..3 {
                world.step(threads).expect("the step runs");
            }
            plan_of(&world)
        })
        .collect();
    assert_eq!(plans[0], plans[1], "one thread and two threads disagree");
    assert_eq!(
        plans[1], plans[2],
        "two threads and twelve threads disagree"
    );
    assert!(!plans[0].is_empty(), "the fixture planned nothing");
}

/// The chain takes the same number of windows whatever the world holds.
///
/// **The hop count is derived and it never reads the world.** It is the plan
/// bound divided by the window radius, rounded up, and each hop relaxes its
/// window the fixed pass count the rules give. A world with two settlements
/// far apart and a world with one settlement therefore run the same solver
/// passes, and a reader sees that from outside.[^1]
///
/// # References
///
/// [^1]: ADR-0005, a solver runs a fixed iteration count, decision D1. `docs/adrs/accepted/adr-0005-a-solver-runs-a-fixed-iteration-count.md`
#[test]
fn the_chain_takes_a_hop_count_the_world_cannot_change() {
    let rules = PlanRules::DEFAULT;
    assert_eq!(
        rules.join_hops(),
        rules.bound().div_ceil(rules.radius()),
        "the hop count is not the bound over the radius"
    );
    // A far pair and a lone seat are the two ends of what a world can hold.
    // Both worlds seat two factions, because the census counts the passes of
    // every faction and a world with fewer seats would run fewer passes for
    // that reason and not for the hop count.
    let (mut far, _, _) = two_far_cities(SEED);
    let mut lone = bare(SEED);
    let seat = open_from(&lone, Axial::new(20, 40));
    lone.found_group_at(seat, 4, ZERO)
        .expect("the ground admits a founding");
    let far_away = open_from(&lone, Axial::new(80, 80));
    lone.found_group_at(far_away, 4, FactionId(1))
        .expect("the ground admits a second faction");
    lone.set_controller_evaluations(0);
    let passes = i64::from(far.plan_rules().solver_passes()) * 2;
    for tick in 1..=4 {
        far.step(1).expect("the step runs");
        lone.step(1).expect("the step runs");
        assert_eq!(
            census(&far, "plan_passes"),
            passes * tick,
            "the world with a far pair ran a different number of passes"
        );
        assert_eq!(
            census(&lone, "plan_passes"),
            census(&far, "plan_passes"),
            "two worlds ran a different number of passes"
        );
    }
}

/// Two settlements of one faction end joined by a road that stands.
///
/// **This test drives the engine and not the mechanism.** It founds the two
/// cities and it then only steps the world. The solver zones the way, the
/// controller sends the idle units, the build pass raises each road, and the
/// walk at the end reads the ground rather than the plan.[^1]
///
/// # References
///
/// [^1]: Testing rules, sections 5 and 6. `.agents/rules/testing.md`
#[test]
fn two_settlements_of_one_faction_end_joined_by_a_road() {
    let (mut world, seat, other) = two_far_cities(SEED);
    for _ in 0..JOIN_TICKS {
        world.step(1).expect("the step runs");
    }
    assert!(world.check_invariants());
    let standing = addresses()
        .into_iter()
        .filter(|address| world.finished_upgrade(*address) == Some(UpgradeCategory::ROAD))
        .count();
    assert!(
        a_road_joins(&world, seat, other),
        "no road joins the two cities after {JOIN_TICKS} ticks, \
         with {standing} roads standing and {} projects left.\n{}",
        plan_of(&world).len(),
        join_report(&world, seat, other)
    );
}

/// Returns what the run left on the ground between two cities.
///
/// The report says how far the road that starts at the first city reaches
/// toward the second, what each plan project asks for, and where the standing
/// roads lie. A bare count cannot tell a way that stops short from a set of
/// roads on other ground.
fn join_report(world: &World, seat: Axial, other: Axial) -> String {
    let mut seen = vec![seat];
    let mut queue = vec![seat];
    while let Some(here) = queue.pop() {
        for neighbour in world.grid().neighbours(here).into_iter().flatten() {
            if seen.contains(&neighbour) {
                continue;
            }
            if world.finished_upgrade(neighbour) != Some(UpgradeCategory::ROAD) {
                continue;
            }
            seen.push(neighbour);
            queue.push(neighbour);
        }
    }
    let nearest = seen
        .iter()
        .map(|address| address.distance(other))
        .min()
        .unwrap_or(u32::MAX);
    let mut categories: Vec<String> = Vec::new();
    for category in [
        UpgradeCategory::ROAD,
        UpgradeCategory::TERRACE,
        UpgradeCategory::LODGING,
    ] {
        let held = plan_of(world)
            .iter()
            .filter(|project| project.category == category)
            .count();
        categories.push(format!("{category:?}={held}"));
    }
    let on_way: usize = addresses()
        .into_iter()
        .filter(|address| world.finished_upgrade(*address) == Some(UpgradeCategory::ROAD))
        .filter(|address| {
            address.distance(seat) + address.distance(other) <= seat.distance(other) + 4
        })
        .count();
    format!(
        "  the road that touches the seat holds {} tiles and reaches within {nearest} of \
         the far city\n  the plan holds {}\n  {on_way} standing roads lie near the line \
         between the two cities",
        seen.len() - 1,
        categories.join(", ")
    )
}
