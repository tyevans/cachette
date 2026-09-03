//! A laden unit goes home, and the delivery it reaches is not a coincidence.
//!
//! The delivery of a carried load worked and never ran. A unit gave its load
//! to the store of its site only while it stood on the tile of that site, and
//! no rule ever put it there on purpose. The findings register holds the
//! measurement.[^1]
//!
//! The engine now holds a fifth option. A unit that carries a full load and
//! holds a home site takes it, and a field over the level 1 cells steers the
//! step.[^2] [^3]
//!
//! **Each test below was watched with its defect put back.** The commit body
//! names which defect each one caught. A test with no proven failure mode is
//! decoration.[^4]
//!
//! # References
//!
//! [^1]: Findings register, FND-317. `docs/FINDINGS.md`
//! [^2]: ADR-0107, the choice key holds a bounded class of the unit's own state. `docs/adrs/draft/adr-0107-the-choice-key-holds-a-bounded-class-of-the-unit-state.md`
//! [^3]: ADR-0108, a unit returns by climbing a reach field seeded at every site of its faction. `docs/adrs/draft/adr-0108-a-unit-returns-by-climbing-a-reach-field.md`
//! [^4]: Testing rules, sections 1 and 2a. `.claude/rules/testing.md`

use cachette_core::choose::{self, CarryClass};
use cachette_core::hex::Axial;
use cachette_core::resource::Amount;
use cachette_core::types::{FactionId, Fix32, TileIdx};
use cachette_core::world::{World, WorldConfig};

/// The option index of the row that carries a load home.
///
/// The row takes the lowest index, so it wins a tie against a row that ranks
/// the ground.
const DELIVER: u8 = 0;

/// The world that the demonstration runs, and that the finding measured.
fn demonstration() -> World {
    World::new(WorldConfig {
        width: 256,
        height: 256,
        seed: 0x0123_4567_89ab_cdef,
        faction_count: 4,
        ..Default::default()
    })
    .expect("the world builds")
}

/// The ticks that the demonstration runs before it delivers.
///
/// The first delivery of the run lands well inside this, and the margin is
/// there so that a change which slows the loop still fails rather than flaps.
const TICKS: u64 = 300;

/// **The engine reaches the delivery, and this is the test that says so.**
///
/// The pass had tests of its own and they passed, because they built the case
/// by hand: a site and a unit seated on one tile. The engine never produced
/// that case. Driving the real caller and reaching the real case are two
/// requirements, so this test drives the world the demonstration runs and
/// asserts on what the engine did with no help.[^1]
///
/// # References
///
/// [^1]: Findings register, FND-317. `docs/FINDINGS.md`
#[test]
fn the_demonstration_world_delivers_a_carried_load() {
    let mut world = demonstration();
    let outcomes = world.found_run_for_every_faction(64);
    assert_eq!(
        outcomes.iter().filter(|it| it.founding().is_some()).count(),
        4,
        "the fixture needs every faction to found, or no unit has a home"
    );
    for _ in 0..TICKS {
        world.step(2).expect("the step runs");
    }
    let delivered: u64 = world.delivered_carry().iter().sum();
    assert!(
        delivered > 0,
        "the engine delivered nothing in {TICKS} ticks, so the pass is still unreachable"
    );
    // The witness that the behaviour ran, and not only that a total moved. A
    // unit reaches the tile of its site by a keyed draw once in a long while,
    // so a total above zero alone does not separate the option from luck.
    let delivering = world
        .soldiers()
        .iter()
        .filter(|unit| world.soldier_intent(*unit) == Some(Some(DELIVER)))
        .count();
    assert!(
        delivering > 0,
        "no unit holds the option that carries a load home"
    );
}

/// A unit that carries nothing never takes the option.
#[test]
fn an_empty_unit_is_free_and_takes_no_delivery() {
    let mut world = demonstration();
    let _ = world.found_run_for_every_faction(64);
    let unit = world
        .soldiers()
        .iter()
        .next()
        .expect("the run spawned units");
    assert_eq!(world.carry_class(unit), Some(CarryClass::Free));
    let summary = world
        .summary_covering(
            world
                .soldiers()
                .address(unit)
                .expect("a live unit stands somewhere"),
        )
        .expect("the unit stands inside the lattice");
    let profile = choose::WeightProfile::EVEN;
    assert_ne!(
        choose::best_option(
            cachette_core::cohort::NEED_FULL,
            CarryClass::Free,
            summary,
            &profile
        ),
        DELIVER,
        "a unit that carries nothing must never take the option"
    );
}

/// **A unit with no home is never laden, whatever it carries.**
///
/// The delivery moves a load into the store of a home site, so an option that
/// sent a homeless unit home would be a capability nothing can act on.[^1]
///
/// # References
///
/// [^1]: Recurring defect shapes, shape 3. `.claude/rules/recurring-defects.md`
#[test]
fn a_unit_with_no_home_is_never_laden() {
    let mut world = demonstration();
    let _ = world.found_run_for_every_faction(64);
    let unit = world
        .soldiers()
        .iter()
        .next()
        .expect("the run spawned units");
    assert!(world.set_home_site(unit, None));
    world.set_carry_mark(Amount::ZERO);
    assert_eq!(
        world.carry_class(unit),
        Some(CarryClass::Free),
        "a unit with no home must be free at a mark of zero, which makes every homed unit laden"
    );
}

/// **A tie between the ground and the load goes to the load.**
///
/// The open share of a cell reaches one whole unit wherever a whole block
/// admits a unit, and the laden value is one whole unit as well. The two
/// scores are then equal, and the tie-break decides. This test names the
/// winner, because a tie that the ground wins leaves a laden unit walking in
/// a world with no water in it.
#[test]
fn a_laden_unit_wins_a_tie_against_the_ground() {
    let profile = choose::WeightProfile::EVEN;
    // **The fixture is a cell of the demonstration world whose ground is
    // entirely open.** A summary cannot be built by hand, and a cell chosen
    // for looking ordinary would not produce the tie this test is about, so
    // the world is searched for the cell that does.[^1]
    //
    // [^1]: Testing rules, section 2a. `.claude/rules/testing.md`
    let world = demonstration();
    let summary = (0..world.pyramid().len())
        .filter_map(|cell| world.pyramid().cell(cell as u32))
        .find(|summary| summary.open_share() == Some(Fix32::ONE))
        .expect("the demonstration world holds a cell of entirely open ground");
    let need = cachette_core::cohort::NEED_FULL;
    let scores: Vec<Fix32> = choose::OPTIONS
        .iter()
        .enumerate()
        .map(|(index, row)| {
            choose::score(
                need,
                CarryClass::Laden,
                profile.weight(index as u8).expect("inside the set"),
                summary,
                *row,
            )
        })
        .collect();
    assert_eq!(
        scores[DELIVER as usize], scores[1],
        "the fixture must produce the tie that this test is about"
    );
    assert_eq!(
        choose::best_option(need, CarryClass::Laden, summary, &profile),
        DELIVER,
        "the tie must go to the option that reads the state of the unit"
    );
}

/// **The return field points at the site, and it points at it from far away.**
///
/// Each step down the field must reach a cell that is nearer to the seed. A
/// field that pointed anywhere else would still be a field, and a unit
/// following it would still move, so a test that only asked for a direction
/// would pass on a field that means nothing.
#[test]
fn the_return_field_leads_to_the_site() {
    let mut world = demonstration();
    let outcomes = world.found_run_for_every_faction(64);
    let founded = outcomes
        .iter()
        .find_map(|outcome| outcome.founding().map(|it| (outcome.faction(), it.place())))
        .expect("a faction founded");
    // The field is derived at the barrier of a step, so a world that founded
    // and never stepped holds the field it had before the site existed.
    world.step(1).expect("the step runs");
    let (faction, place) = founded;
    let field = world.return_field();
    let cells = field.cells();
    let seat = cells
        .index_of(cell_of(&world, place))
        .expect("the site sits inside the lattice");
    let mut walked = 0;
    let mut furthest = 0;
    for cell in 0..cells.tile_count() {
        let Some(Some(direction)) = field.direction(faction, cell) else {
            continue;
        };
        let here = cells.address_of(TileIdx(cell)).expect("inside the lattice");
        let there = cells
            .neighbour(here, direction as usize)
            .expect("the field never names a neighbour outside the lattice");
        let before = here.distance(cells.address_of(seat).expect("inside"));
        let after = there.distance(cells.address_of(seat).expect("inside"));
        assert!(
            after < before,
            "the field sent a unit from a cell {before} away to one {after} away"
        );
        walked += 1;
        furthest = furthest.max(before);
    }
    assert!(walked > 0, "no cell of the world holds a return direction");
    assert!(
        furthest > 1,
        "every cell that holds a direction touches the site, so the fixture never left the block"
    );
}

/// Returns the level 1 cell address that covers one tile address.
fn cell_of(world: &World, address: Axial) -> Axial {
    let layout = world.pyramid().layout();
    let tile = world.grid().index_of(address).expect("inside the world");
    let block = layout.block_of_key(layout.key_of(tile).expect("inside the world"));
    world
        .return_field()
        .cells()
        .address_of(TileIdx(block))
        .expect("inside the lattice")
}

/// A faction the world does not hold has no plane.
#[test]
fn a_faction_outside_the_world_reads_no_direction() {
    let world = demonstration();
    assert_eq!(world.return_field().direction(FactionId(9), 0), None);
}

/// **The step of a laden unit follows the return field, and not a draw.**
///
/// A unit that only stopped gathering would still walk, and it would still
/// reach its site once in a while, so a test that asked whether anything was
/// delivered passes on an engine that steers nothing. That was measured
/// rather than assumed: the delivery total of the demonstration world stays
/// above zero when the option takes no direction from the field at all.[^1]
///
/// This test names the tile. For every laden unit that holds the option, in a
/// cell that holds a direction, over ground that admits a step, the unit is
/// either on the tile the field named or on the tile it started from. The
/// second case is the capacity of the target refusing it, which admission
/// owns.[^2]
///
/// # References
///
/// [^1]: Findings register, FND-319. `docs/FINDINGS.md`
/// [^2]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D2. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
#[test]
fn the_step_of_a_laden_unit_follows_the_return_field() {
    let mut world = demonstration();
    let _ = world.found_run_for_every_faction(64);
    for _ in 0..TICKS {
        world.step(2).expect("the step runs");
    }
    let grid = world.grid();
    // The units this test is about, with the tile each of them must reach.
    let mut expected: Vec<(cachette_core::types::Entity, Axial, Axial)> = Vec::new();
    for unit in world.soldiers().iter() {
        if world.soldier_intent(unit) != Some(Some(DELIVER)) {
            continue;
        }
        let Some(here) = world.soldiers().address(unit) else {
            continue;
        };
        let Some(faction) = world.soldiers().faction(unit) else {
            continue;
        };
        let Some(Some(direction)) = world.return_direction(faction, here) else {
            continue;
        };
        let Some(there) = grid.neighbour(here, direction as usize) else {
            continue;
        };
        if !world.admits_a_unit(there) {
            continue;
        }
        expected.push((unit, here, there));
    }
    assert!(
        !expected.is_empty(),
        "the fixture found no laden unit that the field steers, so it measures nothing"
    );
    world.step(2).expect("the step runs");
    for (unit, here, there) in expected {
        let now = world.soldiers().address(unit).expect("the unit is alive");
        assert!(
            now == there || now == here,
            "a laden unit was sent to {there:?} from {here:?} and it is at {now:?}"
        );
    }
}
