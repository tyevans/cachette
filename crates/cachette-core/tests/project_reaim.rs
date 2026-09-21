//! A faction re-aims its project order and keeps its plan live.
//!
//! A faction holds a bounded plan of zoned projects. An order sends idle units
//! toward their nearest project and assembles a shared seed set for the
//! destination plane of that faction. Sent units climb that shared field.[^1]
//! [^2]
//!
//! These tests assert that:
//! - An unbuildable project on a tile holding another category is cleared
//!   during a solver pass.
//! - A category that does not fit the ground terrain at level 1 is cleared
//!   by `sweep_finished`.
//! - Sending a newly idle unit preserves the destination seeds of existing
//!   units climbing the plane.
//! - When a project is finished or cleared, sent units re-aim to the remaining
//!   projects in the plan.
//! - When a plan holds no remaining projects, destination seeds are cleared and
//!   sent units are released.
//!
//! # References
//!
//! [^1]: ADR-0152, a faction plans its roads and zones with one solver, decisions D1 to D5. `docs/adrs/accepted/adr-0152-a-faction-plans-its-roads-and-zones-with-one-solver.md`
//! [^2]: ADR-0159, a project order names one category and one seed set, decisions D1 and D3. `docs/adrs/accepted/adr-0159-a-project-order-names-one-category-and-one-seed-set.md`
//! [^3]: ADR-0125, the control plane names the seed set of a destination field, decision D4. `docs/adrs/draft/adr-0125-the-control-plane-names-the-seed-set-of-a-destination-field.md`
//! [^4]: Testing rules, section 1 and 2. `.agents/rules/testing.md`

use cachette_core::hex::Grid;
use cachette_core::holding::{Holder, ReachRules};
use cachette_core::plan::{self, Ground, PlanRegister, PlanRules, Project};
use cachette_core::terrain::Terrain;
use cachette_core::types::TileIdx;
use cachette_core::upgrade::{UpgradeCategory, UpgradeMap, UpgradeTable};
use cachette_core::{Axial, FactionId, World, WorldConfig};

/// The extent that these tests read.
const WIDTH: u32 = 96;
/// The number of rows of that extent.
const HEIGHT: u32 = 96;
/// The seed that these tests read.
const SEED: u64 = 641;
/// The faction that plans.
const ZERO: FactionId = FactionId(0);
/// A second faction far away so game end does not stop controllers.
const OTHER: FactionId = FactionId(1);

/// Builds an empty world for testing.
fn bare(seed: u64) -> World {
    let mut world = World::new(WorldConfig {
        width: WIDTH,
        height: HEIGHT,
        seed,
        faction_count: 4,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
        ..WorldConfig::DEFAULT
    })
    .expect("the extent must describe a world");
    world
        .set_choice_schedule(0)
        .expect("the exponent is inside the range");
    world.set_reach_rules(ReachRules::DEFAULT);
    world.set_controller_evaluations(0);
    world
}

/// Returns the first address at or near `from` that admits a unit and fits a
/// road.
fn road_tile_from(world: &World, from: Axial) -> Axial {
    let table = UpgradeTable::default();
    let row = table.row(UpgradeCategory::ROAD, 1).unwrap();
    for radius in 0..12 {
        for step in 0..6 {
            let address = Axial::new(from.q + radius - step, from.r + step);
            if let Some(kind) = world.terrain().kind(address) {
                if row.fits(kind) && world.admits_a_unit(address) {
                    return address;
                }
            }
        }
    }
    panic!("no road-fitting address near {from:?}");
}

/// Builds a test world with two seated factions so that game end does not stop
/// controllers.
fn seeded_fixture(seed: u64) -> (World, Axial) {
    let mut world = bare(seed);
    let seat = road_tile_from(&world, Axial::new(20, 20));
    world
        .found_group_at(seat, 4, ZERO)
        .expect("the ground admits a founding");
    let far = road_tile_from(&world, Axial::new(80, 80));
    world
        .found_group_at(far, 4, OTHER)
        .expect("the ground admits a founding");
    (world, seat)
}

/// Proven-to-fail test: `sweep_finished` clears unbuildable projects.
///
/// A tile carries only one upgrade. A project on a tile that already carries
/// another category cannot be built, and a category that does not fit the
/// ground terrain at level 1 cannot be built. If `sweep_finished` only swept
/// matching completed upgrades, neither unbuildable project would be cleared,
/// and the plan would keep both indefinitely.
#[test]
fn sweep_finished_clears_unbuildable_projects() {
    let grid = Grid::new(32, 32).expect("the extent describes a grid");
    let terrain = Terrain::new(SEED, grid);
    let mut upgrades = UpgradeMap::new();
    let table = UpgradeTable::default();
    let mut plan = PlanRegister::new(2, PlanRules::DEFAULT);

    let tile_a = TileIdx(10);

    // Tile A holds a completed TERRACE.
    let terrace_work = table
        .row(UpgradeCategory::TERRACE, 1)
        .expect("terrace level 1 exists")
        .work;
    upgrades.merge_ascending(
        &[(tile_a, UpgradeCategory::TERRACE, i64::from(terrace_work))],
        &table,
    );
    assert!(
        upgrades.at(tile_a).is_some_and(|site| site.is_complete()),
        "terrace must stand at level 1"
    );

    // ZERO has a project for ROAD on tile_a (which holds TERRACE).
    plan.write(ZERO, Project::new(tile_a, UpgradeCategory::ROAD))
        .unwrap();

    // Tile B is on terrain that refuses ROAD at level 1 (e.g. water).
    let tile_b = (0..grid.tile_count())
        .map(TileIdx)
        .find(|t| {
            grid.address_of(*t)
                .and_then(|a| terrain.kind(a))
                .is_some_and(|kind| !table.row(UpgradeCategory::ROAD, 1).unwrap().fits(kind))
        })
        .expect("must find a tile that refuses road");

    plan.write(ZERO, Project::new(tile_b, UpgradeCategory::ROAD))
        .unwrap();

    assert_eq!(plan.projects_of(ZERO).len(), 2);

    let ground = Ground {
        grid,
        terrain,
        upgrades: &upgrades,
        table: &table,
    };

    plan::sweep_finished(&ground, ZERO, &mut plan);

    assert!(
        plan.projects_of(ZERO).is_empty(),
        "sweep_finished must clear unbuildable projects; left: {:?}",
        plan.projects_of(ZERO)
    );
}

/// An unbuildable project on a tile holding another category is cleared
/// during a solver pass in the simulation.
#[test]
fn unbuildable_project_holding_another_category_is_cleared_during_solver_pass() {
    let mut world = bare(SEED);
    let seat = road_tile_from(&world, Axial::new(10, 10));
    world
        .found_group_at(seat, 4, ZERO)
        .expect("the ground admits a founding");
    let other_site = road_tile_from(&world, Axial::new(80, 80));
    world
        .found_group_at(other_site, 4, OTHER)
        .expect("the ground admits a founding");
    // Step to establish city reach around other_site.
    world.step(2).expect("the step runs");

    // Faction 1 holds ground around other_site.
    // Find a tile held by Faction 1 that fits both ROAD and TERRACE.
    let table = UpgradeTable::default();
    let road_row = table.row(UpgradeCategory::ROAD, 1).unwrap();
    let terrace_row = table.row(UpgradeCategory::TERRACE, 1).unwrap();

    let road_target = world
        .grid()
        .neighbours(other_site)
        .into_iter()
        .flatten()
        .find(|address| {
            world.tile_holder(*address).and_then(Holder::faction) == Some(OTHER)
                && world
                    .terrain()
                    .kind(*address)
                    .is_some_and(|kind| road_row.fits(kind) && terrace_row.fits(kind))
        })
        .expect("must find a neighbour held by faction 1 fitting road and terrace");

    // ZERO zones a road project on that tile.
    world
        .zone_project(ZERO, road_target, UpgradeCategory::ROAD)
        .expect("the plan takes a road anywhere");
    let road_tile = world.grid().index_of(road_target).unwrap();
    assert!(world.plan_of(ZERO).iter().any(|p| p.tile == road_tile));

    // OTHER zones a terrace project on that tile so its builder can build it.
    world
        .zone_project(OTHER, road_target, UpgradeCategory::TERRACE)
        .expect("the plan takes a terrace on held ground");

    // Faction 1 builds a terrace on that same tile.
    let builder = world
        .spawn_soldier(road_target, OTHER)
        .expect("the ground admits a soldier");
    world
        .order_build(builder, UpgradeCategory::TERRACE)
        .expect("faction 1 holds the ground");

    let work = table.work_above(UpgradeCategory::TERRACE, 0);
    for _ in 0..=work {
        world.step(1).expect("the step runs");
        if world.finished_upgrade(road_target) == Some(UpgradeCategory::TERRACE) {
            break;
        }
    }
    assert_eq!(
        world.finished_upgrade(road_target),
        Some(UpgradeCategory::TERRACE),
        "the terrace was not constructed"
    );

    // ZERO runs a solver pass during step(1).
    world.step(1).expect("the step runs");

    // The road project should have been cleared because the tile holds another category.
    assert!(
        !world.plan_of(ZERO).iter().any(|p| p.tile == road_tile),
        "the unbuildable project was not cleared from the plan"
    );
}

/// Sending a newly idle unit preserves the destination seeds of existing
/// walking units on that destination plane.
#[test]
fn sending_a_newly_idle_unit_preserves_destination_seeds_of_walking_units() {
    let (mut world, seat) = seeded_fixture(SEED);
    // Disable automatic solver so only manually zoned projects are in the plan.
    world.set_plan_rules(PlanRules::new(40, 0, 17, 17, 8));

    let target_1 = road_tile_from(&world, Axial::new(seat.q + 3, seat.r));
    let target_2 = road_tile_from(&world, Axial::new(seat.q + 6, seat.r));
    world
        .zone_project(ZERO, target_1, UpgradeCategory::ROAD)
        .expect("the plan takes a road");
    world
        .zone_project(ZERO, target_2, UpgradeCategory::ROAD)
        .expect("the plan takes a road");
    let tile_1 = world.grid().index_of(target_1).unwrap();
    let tile_2 = world.grid().index_of(target_2).unwrap();

    // Spawn unit 1 near target 1.
    let unit_1 = world
        .spawn_soldier(seat, ZERO)
        .expect("ground admits soldier");
    world.step(1).expect("the step runs");

    // Unit 1 is sent to plane 0.
    assert_eq!(
        world.soldiers().sent(unit_1),
        Some(Some(0)),
        "unit 1 was not sent"
    );
    let seeds_before = world.destination_seeds(0).expect("plane 0 exists").to_vec();
    assert!(
        seeds_before.contains(&tile_1),
        "seeds must contain target 1; held: {seeds_before:?}"
    );

    // Spawn unit 2 near target 2.
    let unit_2_spawn = road_tile_from(&world, Axial::new(seat.q + 5, seat.r));
    let unit_2 = world
        .spawn_soldier(unit_2_spawn, ZERO)
        .expect("ground admits soldier");
    assert_eq!(
        world.soldiers().sent(unit_2),
        Some(None),
        "unit 2 must be idle before step"
    );

    // Step the world so controller sends unit 2.
    world.step(1).expect("the step runs");

    assert_eq!(
        world.soldiers().sent(unit_2),
        Some(Some(0)),
        "unit 2 was not sent"
    );

    // Seeds on plane 0 must contain BOTH tile 1 and tile 2.
    let seeds_after = world.destination_seeds(0).expect("plane 0 exists");
    assert!(
        seeds_after.contains(&tile_1),
        "seeds after send must preserve target 1 of unit 1; held: {seeds_after:?}"
    );
    assert!(
        seeds_after.contains(&tile_2),
        "seeds after send must include target 2 of unit 2; held: {seeds_after:?}"
    );
}

/// Sent units climbing the plane re-aim toward remaining projects when a
/// targeted project is cleared.
#[test]
fn sent_units_re_aim_when_targeted_project_is_cleared() {
    let (mut world, seat) = seeded_fixture(SEED);
    // Disable automatic solver so only manually zoned projects are in the plan.
    world.set_plan_rules(PlanRules::new(40, 0, 17, 17, 8));

    let target_1 = road_tile_from(&world, Axial::new(seat.q + 3, seat.r));
    let target_2 = road_tile_from(&world, Axial::new(seat.q + 8, seat.r));
    world
        .zone_project(ZERO, target_1, UpgradeCategory::ROAD)
        .expect("the plan takes a road");
    world
        .zone_project(ZERO, target_2, UpgradeCategory::ROAD)
        .expect("the plan takes a road");
    let tile_1 = world.grid().index_of(target_1).unwrap();
    let tile_2 = world.grid().index_of(target_2).unwrap();

    let unit = world
        .spawn_soldier(seat, ZERO)
        .expect("ground admits soldier");
    world.step(1).expect("the step runs");

    assert_eq!(world.soldiers().sent(unit), Some(Some(0)));
    let seeds_before = world.destination_seeds(0).expect("plane 0 exists");
    assert!(
        seeds_before.contains(&tile_1),
        "seeds must contain target 1; held: {seeds_before:?}"
    );

    // Clear target 1 from the plan.
    assert!(world.clear_project(ZERO, target_1));
    assert_eq!(world.plan_of(ZERO).len(), 1);

    // Step the world. Sent unit should re-aim toward target 2.
    world.step(1).expect("the step runs");

    let seeds_after = world.destination_seeds(0).expect("plane 0 exists");
    assert!(
        !seeds_after.contains(&tile_1),
        "seeds must not contain cleared target 1; held: {seeds_after:?}"
    );
    assert!(
        seeds_after.contains(&tile_2),
        "seeds must re-aim to remaining target 2; held: {seeds_after:?}"
    );
}

/// When a faction's plan holds no remaining projects, the destination plane
/// seeds are cleared and walking units are released.
#[test]
fn when_plan_holds_no_projects_destination_seeds_are_cleared_and_units_released() {
    let (mut world, seat) = seeded_fixture(SEED);
    // Disable automatic solver so only manually zoned projects are in the plan.
    world.set_plan_rules(PlanRules::new(40, 0, 17, 17, 8));

    let target = road_tile_from(&world, Axial::new(seat.q + 4, seat.r));
    world
        .zone_project(ZERO, target, UpgradeCategory::ROAD)
        .expect("the plan takes a road");

    let unit = world
        .spawn_soldier(seat, ZERO)
        .expect("ground admits soldier");
    world.step(1).expect("the step runs");

    assert_eq!(world.soldiers().sent(unit), Some(Some(0)));
    assert!(!world.destination_seeds(0).unwrap().is_empty());

    // Clear the only project in the plan.
    assert!(world.clear_project(ZERO, target));
    assert!(world.plan_of(ZERO).is_empty());

    // Step the world once. Controller clears destination seeds.
    world.step(1).expect("the step runs");

    let seeds = world.destination_seeds(0).expect("plane 0 exists");
    assert!(
        seeds.is_empty(),
        "destination seeds must be empty when plan has no projects; held: {seeds:?}"
    );

    // Next step: movement releases sent units that no longer steer toward any seed.
    world.step(1).expect("the step runs");
    assert_eq!(
        world.soldiers().sent(unit),
        Some(None),
        "unit must be released when destination plane has no seeds"
    );
}
