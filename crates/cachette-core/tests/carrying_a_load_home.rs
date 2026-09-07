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
//! **Two tests here read the demonstration world once, and they measured that
//! world.** They went red when two correct changes landed, because the world
//! they copied stopped producing a laden unit.[^5] [^6] Both now build the
//! world that produces the case, and the builder states every value that
//! decides it.
//!
//! # References
//!
//! [^1]: Findings register, FND-317. `docs/FINDINGS.md`
//! [^2]: ADR-0109, the choice key holds a bounded class of the unit's own state. `docs/adrs/draft/adr-0109-the-choice-key-holds-a-bounded-class-of-the-unit-state.md`
//! [^3]: ADR-0110, a unit returns by climbing a reach field seeded at every site of its faction. `docs/adrs/draft/adr-0110-a-unit-returns-by-climbing-a-reach-field.md`
//! [^4]: Testing rules, sections 1 and 2a. `.agents/rules/testing.md`
//! [^5]: Findings register, FND-572. `docs/FINDINGS.md`
//! [^6]: Findings register, FND-574. `docs/FINDINGS.md`

use cachette_core::choose::{self, CarryClass};
use cachette_core::cohort::NeedRule;
use cachette_core::hex::Axial;
use cachette_core::position::WORK_COMMODITY;
use cachette_core::resource::{Amount, ResourceKind};
use cachette_core::types::{Entity, FactionId, Fix32, TileIdx};
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

/// The number of carriers that the fixture builds.
///
/// The count is above one, so an assertion that reads a laden unit reads a
/// set of them. It bounds no measurement.
const CARRIERS: usize = 4;

/// The walk between the site and the ground that the carriers load on.
///
/// The span is longer than the pitch of one level 1 block. The return field
/// holds one direction for each block, so a shorter walk would never leave
/// the block that holds the site.
const CARRY_SPAN: u32 = 40;

/// The load that makes a carrier of this fixture laden.
///
/// **The fixture states the mark, so no constant of the world decides what
/// these tests measure.** A fixture that took the mark of the world would
/// measure whichever value the world held on the day it ran.[^1]
///
/// # References
///
/// [^1]: Findings register, FND-573. `docs/FINDINGS.md`
const CARRY_MARK: Amount = Amount(24);

/// The food that the fixture puts in the store of the site.
///
/// The quantity feeds every carrier and every resident for the whole run,
/// and it leaves room for the load that a carrier brings home. A full store
/// refuses a delivery.
const STORE_FOOD: i16 = 32000;

/// The seed of the world that the fixture builds.
///
/// One seed is enough. The engine gives one answer for one binary, so a
/// world that produces this case once produces it every time.[^1]
///
/// # References
///
/// [^1]: ADR-0001, one binary gives one answer at any thread count, decision D1. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
const CARRY_SEED: u64 = 7;

/// The ticks that the fixture gives a carrier to reach the ground it was
/// sent to.
const ARRIVE_TICKS: u32 = 400;

/// The ticks that the fixture gives a carrier to fill its carry.
const LOAD_TICKS: u32 = 2000;

/// The ticks that a test gives a laden carrier to walk home and deliver.
const HOME_TICKS: u32 = 400;

/// A world that the fixture built, and the carriers in it.
struct Carriers {
    /// The world, stepped to the state that a test reads.
    world: World,
    /// The tile of the site that every carrier calls home.
    home: Axial,
    /// The carriers, in the order that the fixture spawned them.
    units: Vec<Entity>,
}

/// Builds the world that the two carrier tests measure.
///
/// **These two tests read the demonstration world once, and they measured
/// that world rather than the engine.** The demonstration world is chosen to
/// look right. It produced a laden unit through a random walk, and it stopped
/// producing one when two correct changes landed: a sent unit began to reach
/// the tile it was sent to, and a founding stopped seating more people than
/// its ground feeds.[^1] [^2] [^3] A fixture built by copying that world
/// supplies no extreme, so the assertion never receives the input that would
/// fail it.[^4]
///
/// This fixture states every value that decides the case, and it asserts each
/// stage. The stages are the carry path itself.
///
/// 1. The fixture founds one site on ground that carries food, and it stops
///    the production of that site. The store then holds nothing until the
///    fixture fills it.
/// 2. The fixture spawns the carriers beside the site, gives each of them
///    that site as a home, and sends the set to a patch of food a long walk
///    away. **The engine releases each unit that reaches the patch.** A unit
///    that stayed sent would climb its destination plane for ever, and it
///    would read no option row.[^2]
/// 3. The need of a carrier falls, because the store of the site is empty.
///    A hungry unit forages, so each carrier fills its carry from the patch.
/// 4. The fixture then fills the store and states a need rule that feeds a
///    unit to the full. A fed unit that carries the mark takes the row that
///    goes home, because the row that forages is worth nothing to a unit that
///    lacks nothing.
///
/// The world returns with every carrier laden, free of any send, and a long
/// walk from its home. No carrier holds the delivery option yet, because the
/// choice pass reads one cell on one tick in each interval. Each test steps
/// the world itself and asserts what the engine does.
///
/// # References
///
/// [^1]: Findings register, FND-572. `docs/FINDINGS.md`
/// [^2]: Findings register, FND-576. `docs/FINDINGS.md`
/// [^3]: Findings register, FND-574. `docs/FINDINGS.md`
/// [^4]: Testing rules, section 2a. `.agents/rules/testing.md`
fn a_world_built_for_the_carry() -> Carriers {
    let mut world = World::new(WorldConfig {
        width: 256,
        height: 256,
        seed: CARRY_SEED,
        faction_count: 2,
        ..Default::default()
    })
    .expect("the world builds");
    // **Every faction is under external control, so the built-in controller
    // sends nobody.** The controller is the one other writer of the send
    // column, and a fixture that left it running would measure whichever
    // command the controller drew on the tick the test read.[^1]
    //
    // [^1]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decision D6. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
    for index in 0..world.config().faction_count {
        world.set_externally_controlled(FactionId(index), true);
    }
    world.set_carry_mark(CARRY_MARK);
    // The threshold is zero, so no unit of this fixture is ever short and
    // nothing ends it. The decay and the ration keep the rule the world
    // holds, because the hunger of a carrier is what drives it to forage.
    let rule = NeedRule::DEFAULT;
    world.set_need_rule(
        NeedRule::new(
            rule.decay(),
            rule.ration(),
            Fix32::ZERO,
            rule.recovery(),
            Fix32::MAX,
        )
        .expect("no rate is below zero"),
    );

    let home = a_tile_that_carries_food(&world).expect("the world holds ground that carries food");
    let site = world
        .found_settlement(home, FactionId(0))
        .expect("the ground admits a settlement");
    let ration = WORK_COMMODITY[ResourceKind::Food.index()];
    // The site produces nothing, so the fixture alone decides the tick on
    // which a carrier is fed.
    world
        .set_production_rate(site, ration, Fix32::ZERO)
        .expect("the site resolves");

    let patch = food_ground_at(&world, home, CARRY_SPAN);
    let seeds: Vec<Axial> = every_address(&world)
        .into_iter()
        .filter(|at| {
            patch.distance(*at) <= 2 && world.admits_a_unit(*at) && food_on(&world, *at) >= 8
        })
        .collect();
    assert!(
        !seeds.is_empty(),
        "the fixture found no ground that carries food a walk of {CARRY_SPAN} from the site"
    );
    let spots: Vec<Axial> = every_address(&world)
        .into_iter()
        .filter(|at| *at != home && home.distance(*at) <= 3 && world.admits_a_unit(*at))
        .take(CARRIERS)
        .collect();
    assert_eq!(
        spots.len(),
        CARRIERS,
        "the fixture found no room for the carriers beside the site"
    );
    let units: Vec<Entity> = spots
        .iter()
        .map(|at| {
            world
                .spawn_soldier(*at, FactionId(0))
                .expect("the ground admits the unit")
        })
        .collect();
    for unit in &units {
        assert!(
            world.set_home_site(*unit, Some(site)),
            "the unit takes home"
        );
    }
    world
        .send_units_to(&units, &seeds, 0)
        .expect("every identity is live and every seed is inside the world");

    // **The engine frees a carrier that arrives, and nothing else frees
    // one.** A carrier that stayed sent would read its destination plane for
    // ever, so it would take no option row and it would deliver nothing.[^1]
    //
    // [^1]: Findings register, FND-576. `docs/FINDINGS.md`
    let mut arrived = false;
    for _ in 0..ARRIVE_TICKS {
        world.step(2).expect("the step runs");
        if units
            .iter()
            .all(|unit| world.soldiers().sent(*unit) == Some(None))
        {
            arrived = true;
            break;
        }
    }
    assert!(
        arrived,
        "the engine held a carrier on its destination plane for {ARRIVE_TICKS} ticks, \
         so no carrier ever reads its option row"
    );

    // The store of the site is empty, so the need of each carrier falls and
    // the carrier forages. The patch under it carries the food.
    let mut loaded = false;
    for _ in 0..LOAD_TICKS {
        world.step(2).expect("the step runs");
        if units
            .iter()
            .all(|unit| world.carry_class(*unit) == Some(CarryClass::Laden))
        {
            loaded = true;
            break;
        }
    }
    assert!(
        loaded,
        "a carrier stayed below the carry mark for {LOAD_TICKS} ticks, \
         so the fixture holds no laden unit"
    );

    // **The fixture now feeds every carrier to the full.** The row that
    // forages is driven by what a unit lacks, and the row that goes home is
    // driven by what it holds. A hungry unit therefore forages beside a full
    // carry, and only a fed unit takes the load home.
    //
    // The decay is zero and the ration fills a unit in one application, so
    // the need of a fed carrier reaches the full value and stays there.
    world.set_need_rule(
        NeedRule::new(
            Fix32::ZERO,
            cachette_core::cohort::NEED_FULL,
            Fix32::ZERO,
            Fix32::ZERO,
            Fix32::MAX,
        )
        .expect("no rate is below zero"),
    );
    assert!(
        world
            .set_settlement_store(site, ration, Fix32::from_int(STORE_FOOD))
            .expect("the site resolves"),
        "the store takes the food"
    );

    Carriers { world, home, units }
}

/// Returns every address of the world, in tile index order.
fn every_address(world: &World) -> Vec<Axial> {
    let grid = world.grid();
    (0..grid.tile_count())
        .map(|index| Axial::new((index % grid.width()) as i32, (index / grid.width()) as i32))
        .collect()
}

/// Returns the food that one tile still holds.
fn food_on(world: &World, at: Axial) -> u32 {
    world
        .tile_stock(at, ResourceKind::Food)
        .map_or(0, |stock| stock.0)
}

/// Returns the open address nearest to a walk of one span that carries food.
///
/// The ground must carry enough food to fill a carry in a few gathers. A tile
/// that carries one unit of food would make the walk of the carrier a
/// property of the recovery rate.
fn food_ground_at(world: &World, from: Axial, span: u32) -> Axial {
    every_address(world)
        .into_iter()
        .filter(|at| world.admits_a_unit(*at) && food_on(world, *at) >= 10)
        .min_by_key(|at| (i64::from(from.distance(*at)) - i64::from(span)).abs())
        .expect("the world holds ground that carries food")
}

/// **The engine carries a load home and delivers it, with no help.**
///
/// The delivery pass had tests of its own and they passed, because they built
/// the case by hand: a site and a unit seated on one tile. The engine never
/// produced that case.[^1] Driving the real caller and reaching the real case
/// are two requirements, so this test builds a world that produces the case
/// and then asserts only on what the engine does inside it.
///
/// The fixture stops when every carrier is laden, fed and a long walk from
/// its home. The engine alone chooses the option, steers each step and moves
/// the load into the store.
///
/// # References
///
/// [^1]: Findings register, FND-317. `docs/FINDINGS.md`
#[test]
fn the_engine_carries_a_load_home_and_delivers_it() {
    let Carriers {
        mut world,
        home,
        units,
    } = a_world_built_for_the_carry();
    // The witness that the behaviour ran, and not only that a total moved. A
    // unit reaches the tile of its site by a keyed draw once in a long while,
    // so a total above zero alone does not separate the option from luck.
    let mut delivering = 0;
    let mut emptied = 0;
    let mut delivered = 0;
    for _ in 0..HOME_TICKS {
        world.step(2).expect("the step runs");
        delivering = delivering.max(
            units
                .iter()
                .filter(|unit| world.soldier_intent(**unit) == Some(Some(DELIVER)))
                .count(),
        );
        // A carrier that stands on the tile of its site with an empty carry
        // gave its load to the store. The reading names the unit, so a
        // delivery by anything else cannot answer for it.
        emptied = units
            .iter()
            .filter(|unit| {
                world.soldiers().address(**unit) == Some(home) && carried_food(&world, **unit) == 0
            })
            .count();
        delivered = world.delivered_carry().iter().sum::<u64>();
        if emptied > 0 {
            break;
        }
    }
    assert!(
        delivering > 0,
        "no unit holds the option that carries a load home"
    );
    assert!(
        emptied > 0,
        "no carrier reached the tile of its site and gave up its load in {HOME_TICKS} ticks"
    );
    assert!(
        delivered > 0,
        "a carrier emptied its carry and the engine counted no delivery"
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
/// rather than assumed: the delivery total of a world stays above zero when
/// the option takes no direction from the field at all.[^1]
///
/// This test names the tile. For every laden unit that holds the option, in a
/// cell that holds a direction, over ground that admits a step, the unit is
/// either on the tile the field named or on the tile it started from. The
/// second case is the capacity of the target refusing it, which admission
/// owns.[^2]
///
/// **The test reads only a unit that the engine freed.** A sent unit climbs
/// its destination plane and reads no option row, so it is no evidence about
/// the return field.[^3] [^4] The fixture frees nobody. It sends the carriers
/// out, and the engine releases each one that arrives, so a reading here is a
/// reading of the engine.
///
/// # References
///
/// [^1]: Findings register, FND-319. `docs/FINDINGS.md`
/// [^2]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D2. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
/// [^3]: ADR-0125, the control plane names the seed set of a destination field, decision D3. `docs/adrs/draft/adr-0125-the-control-plane-names-the-seed-set-of-a-destination-field.md`
/// [^4]: Findings register, FND-496. `docs/FINDINGS.md`
#[test]
fn the_step_of_a_laden_unit_follows_the_return_field() {
    let Carriers {
        mut world,
        home: _,
        units,
    } = a_world_built_for_the_carry();
    let mut measured = false;
    for _ in 0..HOME_TICKS {
        // The units this test is about, with the tile each of them must
        // reach.
        let grid = world.grid();
        let mut expected: Vec<(Entity, Axial, Axial)> = Vec::new();
        for unit in units.iter().copied() {
            if world.soldier_intent(unit) != Some(Some(DELIVER)) {
                continue;
            }
            if world.soldiers().sent(unit) != Some(None) {
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
        world.step(2).expect("the step runs");
        for (unit, here, there) in expected {
            measured = true;
            let now = world.soldiers().address(unit).expect("the unit is alive");
            assert!(
                now == there || now == here,
                "a laden unit was sent to {there:?} from {here:?} and it is at {now:?}"
            );
        }
        if measured {
            break;
        }
    }
    assert!(
        measured,
        "the fixture found no laden unit that the field steers, so it measures nothing"
    );
}

/// The seeds the last-mile test runs.
///
/// The relaxation of the fine field stays inside one block. A world where the
/// ground between the unit and its home leaves that block holds no offset for
/// the unit, and the unit there reads the coarse field, which is the answer it
/// read before the fine field existed. Several seeds keep one such world from
/// deciding the test.
const LAST_MILE_SEEDS: [u64; 4] = [1, 2, 5, 7];

/// The ticks the last-mile test gives the unit to fill its carry.
const LAST_MILE_GATHER: u32 = 4096;

/// The ticks the last-mile test gives the unit to walk home.
const LAST_MILE_WALK: u32 = 2000;

/// The walk between the unit and its home, in tiles.
const LAST_MILE_SPAN: u32 = 40;

#[test]
fn a_laden_unit_reaches_the_tile_of_its_home_and_not_only_the_cell() {
    // **The delivery reads the tile the unit stands on.** The return field
    // holds one direction for a block of tiles, so it steers a laden unit to
    // the cell that holds a site and says nothing inside it. The unit then
    // drew a direction at random and never stood on the tile, so it never
    // delivered.[^1]
    //
    // The test asserts the delivery, which is the thing the carrier is for.
    // An assertion on the cell held through the whole defect.[^2]
    //
    // [^1]: Findings register, FND-315. `docs/FINDINGS.md`
    // [^2]: Testing rules, section 2. `.claude/rules/testing.md`
    let mut delivered = 0;
    let mut laden = 0;
    for seed in LAST_MILE_SEEDS {
        let mut world = World::new(WorldConfig {
            width: 256,
            height: 256,
            seed,
            faction_count: 2,
            ..Default::default()
        })
        .expect("the world builds");
        // Nothing starves and nothing eats, so the test measures the walk.
        world.set_need_rule(
            NeedRule::new(
                Fix32::ZERO,
                Fix32::ZERO,
                Fix32::ZERO,
                Fix32::ZERO,
                Fix32::MAX,
            )
            .expect("no rate is below zero"),
        );
        let Some(home) = a_tile_that_carries_food(&world) else {
            continue;
        };
        let site = world
            .found_settlement(home, FactionId(0))
            .expect("the ground admits a settlement");
        let away = ground_at(&world, home, LAST_MILE_SPAN);
        let unit = world
            .spawn_soldier(away, FactionId(0))
            .expect("the ground admits the unit");
        // The unit gathers with no home, so it holds a load when it is given
        // one. The gather resolve runs before the delivery in a frame.
        let mark = world.carry_mark().0;
        for _ in 0..LAST_MILE_GATHER {
            if carried_food(&world, unit) >= mark {
                break;
            }
            world.order_gather(unit, ResourceKind::Food);
            world.step(1).expect("the step runs");
        }
        // **A unit below the carry mark is free, and the deliver row is worth
        // nothing to it.** Such a unit roams, and a test that counted it
        // would measure the roam.[^3]
        //
        // [^3]: Testing rules, section 2a. `.claude/rules/testing.md`
        if carried_food(&world, unit) < mark {
            continue;
        }
        assert!(world.set_home_site(unit, Some(site)), "the unit takes home");
        laden += 1;
        for _ in 0..LAST_MILE_WALK {
            world.step(1).expect("the step runs");
            if world.soldiers().address(unit).is_none() {
                break;
            }
            if carried_food(&world, unit) == 0 {
                delivered += 1;
                break;
            }
        }
    }
    assert!(
        laden > 0,
        "the fixture produced no laden unit, so it measures nothing"
    );
    assert_eq!(
        delivered, laden,
        "every laden unit must reach the tile of its home and deliver there"
    );
}

/// Returns the food that one unit carries.
fn carried_food(world: &World, unit: Entity) -> u32 {
    world
        .soldiers()
        .carry(unit)
        .map_or(0, |load| load.of(ResourceKind::Food).0)
}

/// Returns the first open address that carries food.
fn a_tile_that_carries_food(world: &World) -> Option<Axial> {
    let grid = world.grid();
    for index in 0..grid.tile_count() {
        let address = Axial::new((index % grid.width()) as i32, (index / grid.width()) as i32);
        if !world.admits_a_unit(address) {
            continue;
        }
        if world
            .tile_stock(address, ResourceKind::Food)
            .is_some_and(|stock| stock.0 > 0)
        {
            return Some(address);
        }
    }
    None
}

/// Returns the nearest open address at or beyond a walk from another.
fn ground_at(world: &World, from: Axial, span: u32) -> Axial {
    let grid = world.grid();
    let mut best: Option<(u32, Axial)> = None;
    for q in 0..grid.width() as i32 {
        for r in 0..grid.height() as i32 {
            let at = Axial::new(q, r);
            if !world.admits_a_unit(at) {
                continue;
            }
            let reach = from.distance(at);
            if reach >= span && best.is_none_or(|(held, _)| reach < held) {
                best = Some((reach, at));
            }
        }
    }
    best.expect("the world holds ground that far away").1
}
