//! A site's production follows the world, and it does not stand still.
//!
//! A site set its production rate once, at its founding, from the food the
//! survey measured. Nothing read the ground again, so a site produced the
//! same amount on the last tick of a run as on the first.[^1]
//!
//! These tests assert that each of the four inputs of the pipeline reaches
//! the rate. Each drives the engine rather than the pipeline, because the
//! engine is the caller that is obligated to invoke it.[^2]
//!
//! **Each fixture is built for its extreme and is not a copy of the
//! demonstration world.** The founding group is fifteen times the default, so
//! the units strip the disc of a site rather than sample it. The ground test
//! reads the whole run and compares the emptiest tick against the fullest,
//! rather than two ticks chosen in advance. A fixture that modelled the
//! typical case would supply no extreme, and the assertion would then measure
//! the fixture.[^3]
//!
//! **Each fixture holds the other three inputs still by construction, and
//! then checks that it did.** The weather test seats sites across the world
//! and spawns no unit, so nothing draws a disc down and nothing builds. The
//! terrace test names one tile, zones it for a terrace and puts a builder on
//! it, so the one terrace of the run stands where the test chose. A fixture
//! that searched one run for a matching pair would depend on the seed, and a
//! seed is not a fixture.
//!
//! # References
//!
//! [^1]: Backlog item 0136, provision a founded site from the ground it reaches. `docs/backlog/complete/0136-provision-a-founded-site-from-the-ground-it-reaches.md`
//! [^2]: Testing rules, section 5. `.agents/rules/testing.md`
//! [^3]: Testing rules, section 2a. `.agents/rules/testing.md`
//! [^4]: Backlog item 0505, keep a builder on the tile it builds until the work is done. `docs/backlog/complete/0505-keep-a-builder-on-the-tile-it-builds-until-the-work-is-done.md`

use cachette_core::cohort::NeedRule;
use cachette_core::effective::{SCALE_CEILING, SCALE_FLOOR, WET_WEIGHT};
use cachette_core::founding::{disc, SURVEY_RADIUS};
use cachette_core::hex::Axial;
use cachette_core::resource::ResourceKind;
use cachette_core::upgrade::UpgradeCategory;
use cachette_core::{sim_math, CommodityId, Entity, FactionId, Fix32, World, WorldConfig};

/// The extent of every fixture.
///
/// The world is wider than the coarsest lattice spacing of the terrain
/// generator, so it holds more than one kind of ground.
const EXTENT: u32 = 192;

/// The number of factions every fixture holds.
const FACTIONS: u16 = 4;

/// The size of the founding group.
///
/// This is fifteen times the default. A large group strips the disc of a site
/// in a few tens of ticks, which is the extreme the ground term needs.
const GROUP: u32 = 30;

/// The number of threads every test steps at.
const THREADS: usize = 4;

/// The ticks a fixture runs before it reads anything.
///
/// The residents of a site are derived from the home column of the units, and
/// the pass that derives them runs inside a step. A site therefore reads no
/// resident at tick zero, whatever the founding seated.
const SETTLE: u32 = 30;

/// The one commodity the world holds.
const FOOD: CommodityId = CommodityId(0);

/// The spacing of the seats the weather fixture founds.
///
/// The weather field answers for a level 1 cell. The spacing is wider than
/// one cell, so two seats read two cells and the sample spans the world.
const SEAT_SPACING: usize = 24;

/// The ticks the weather fixture runs.
///
/// The field starts dry and fills over the first ticks of a run. The count is
/// far above that start, so the sample reads the weather the solve settles on
/// rather than the start it began from.
const WET_TICKS: u32 = 900;

/// Builds a fixture world.
fn world(seed: u64) -> World {
    World::new(WorldConfig {
        width: EXTENT,
        height: EXTENT,
        seed,
        faction_count: FACTIONS,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    })
    .expect("the settings describe a world")
}

/// Founds every faction and returns the site of the first one that seated.
fn seated(world: &mut World) -> Entity {
    let outcomes = world.found_run_for_every_faction(GROUP);
    outcomes
        .iter()
        .filter_map(|outcome| outcome.founding())
        .map(|founding| founding.settlement())
        .next()
        .expect("the fixture world seats at least one group")
}

/// Seats a site on a lattice across the world, and returns each seat.
///
/// The lattice spans the world, so the seats read many weather cells rather
/// than one. A tile that refuses a seat is skipped.
fn wet_seats(world: &mut World) -> Vec<(Entity, Axial)> {
    let mut seats = Vec::new();
    for r in (0..EXTENT as usize).step_by(SEAT_SPACING) {
        for q in (0..EXTENT as usize).step_by(SEAT_SPACING) {
            let address = Axial::new(q as i32, r as i32);
            if let Ok(site) = world.found_settlement(address, FactionId(0)) {
                seats.push((site, address));
            }
        }
    }
    seats
}

/// Orders every live unit to gather food, so the discs are stripped.
fn everybody_gathers(world: &mut World) {
    let units: Vec<Entity> = world.soldiers().iter().collect();
    world.order_gather_set(&units, ResourceKind::Food);
}

/// Steps the world a number of ticks.
fn run(world: &mut World, ticks: u32) {
    for _ in 0..ticks {
        world.step(THREADS).expect("the fixture steps");
    }
}

/// Returns the food that somebody has taken from the disc of one place.
fn taken_over_the_disc(world: &World, address: Axial) -> u32 {
    disc(world.grid(), address, SURVEY_RADIUS)
        .into_iter()
        .filter_map(|place| world.taken_from(place, ResourceKind::Food))
        .map(|amount| amount.0)
        .sum()
}

/// Returns the standing terraces of the disc of one place.
fn terraces_over_the_disc(world: &World, address: Axial) -> usize {
    disc(world.grid(), address, SURVEY_RADIUS)
        .into_iter()
        .filter(|place| world.finished_upgrade(*place) == Some(UpgradeCategory::TERRACE))
        .count()
}

/// One sample of the pipeline, and of the inputs that could move it.
struct Sample {
    taken: u32,
    scale: Fix32,
    wet: bool,
    terraces: usize,
}

/// Runs a fixture and samples the pipeline on every tick.
fn sample_a_run(seed: u64, ticks: u32) -> Vec<Sample> {
    let mut world = world(seed);
    let site = seated(&mut world);
    everybody_gathers(&mut world);
    run(&mut world, SETTLE);
    let address = world
        .settlements()
        .address(site)
        .expect("the founding seated the site");
    let mut samples = Vec::new();
    for _ in 0..ticks {
        world.step(THREADS).expect("the fixture steps");
        samples.push(Sample {
            taken: taken_over_the_disc(&world, address),
            scale: world.production_scale(site).expect("the site is live"),
            wet: world.ground_is_wet(address) == Some(true),
            terraces: terraces_over_the_disc(&world, address),
        });
    }
    samples
}

#[test]
fn production_falls_as_the_ground_is_drawn_down_and_recovers_when_it_does() {
    let samples = sample_a_run(0x0cac_4e77_5104_0001, 1200);

    // Hold every other input still. Compare only the samples that agree on
    // the weather and on the terraces, so the ground is the one term left
    // that can move.
    let last = samples.last().expect("the run has samples");
    let held: Vec<&Sample> = samples
        .iter()
        .filter(|sample| sample.wet == last.wet && sample.terraces == last.terraces)
        .collect();
    assert!(
        held.len() > 100,
        "the fixture must hold the other terms still for a long span, but it \
         held them for {} ticks",
        held.len()
    );

    let fullest = held
        .iter()
        .min_by_key(|sample| sample.taken)
        .expect("the span is not empty");
    let emptiest = held
        .iter()
        .max_by_key(|sample| sample.taken)
        .expect("the span is not empty");
    assert!(
        emptiest.taken > fullest.taken,
        "the fixture must draw the ground down, or the assertion below \
         measures nothing"
    );
    assert!(
        emptiest.scale < fullest.scale,
        "an emptier disc must give a lower scale, but {} taken gives {:?} and \
         {} taken gives {:?}",
        emptiest.taken,
        emptiest.scale,
        fullest.taken,
        fullest.scale
    );

    // And the ground recovers, so the scale must both fall and rise over the
    // run. A rate that only falls is a sink with no source, which is the same
    // defect as a source with no sink.
    let fell = samples.windows(2).any(|pair| pair[1].scale < pair[0].scale);
    let rose = samples.windows(2).any(|pair| pair[1].scale > pair[0].scale);
    assert!(fell, "the scale never fell over the run");
    assert!(rose, "the scale never rose over the run");
}

#[test]
fn production_rises_when_the_ground_is_wet_and_falls_when_it_dries() {
    // The weather field answers for a level 1 cell, so the wet term reads the
    // cell of the site and nothing else. One site therefore samples one cell,
    // and a run over one site measures whatever that cell happens to do. This
    // fixture spans the world instead. It seats a site in each of many cells,
    // so the sample holds the cells that dry as well as the cells that stay
    // wet.
    let mut world = world(0x0cac_4e77_5104_0002);
    let seats: Vec<(Entity, Axial)> = wet_seats(&mut world);
    assert!(
        seats.len() > 8,
        "the fixture must seat a site in many cells, but it seated {}",
        seats.len()
    );

    // No unit stands in this world. Nobody gathers, so the disc of every site
    // keeps its whole store, and nobody builds, so no terrace stands. The
    // ground term and the terrace term are therefore held still by
    // construction, and the weather is the one term left that can move. The
    // assertions below check that, rather than assume it.
    let mut wet: Vec<Vec<Fix32>> = vec![Vec::new(); seats.len()];
    let mut dry: Vec<Vec<Fix32>> = vec![Vec::new(); seats.len()];
    for _ in 0..WET_TICKS {
        world.step(THREADS).expect("the fixture steps");
        for (index, (site, address)) in seats.iter().enumerate() {
            let Some(scale) = world.production_scale(*site) else {
                continue;
            };
            assert_eq!(
                taken_over_the_disc(&world, *address),
                0,
                "no unit stands in this world, so nothing may draw the ground \
                 down"
            );
            assert_eq!(
                terraces_over_the_disc(&world, *address),
                0,
                "no unit stands in this world, so no terrace may stand"
            );
            if world.ground_is_wet(*address) == Some(true) {
                wet[index].push(scale);
            } else {
                dry[index].push(scale);
            }
        }
    }

    // A cell that holds both a wet tick and a dry tick is the case this test
    // needs. The count is an assertion about the weather: a field that never
    // dried anywhere would leave the assertion below with nothing to compare.
    let mut both = 0usize;
    for index in 0..seats.len() {
        if wet[index].is_empty() || dry[index].is_empty() {
            continue;
        }
        both += 1;
        for soaked in &wet[index] {
            for parched in &dry[index] {
                assert_eq!(
                    *soaked,
                    sim_math::add(*parched, WET_WEIGHT),
                    "wet ground must add exactly the wet weight to the scale"
                );
                assert!(
                    *soaked > *parched,
                    "wet ground must give a higher scale than dry ground"
                );
            }
        }
    }
    assert!(
        both > 0,
        "the weather must both wet and dry the cell of at least one site, or \
         the assertion above measures nothing. None of {} sites saw both",
        seats.len()
    );
}

#[test]
fn production_rises_when_a_terrace_completes() {
    let mut world = world(0x0cac_4e77_5104_0003);
    let site = seated(&mut world);
    // A terrace stands only on ground the faction holds, and the holding pass
    // must run before the order.
    run(&mut world, SETTLE);

    let address = world.settlements().address(site).expect("the site is live");
    let faction = world.settlements().faction(site).expect("the site is live");
    assert_eq!(
        terraces_over_the_disc(&world, address),
        0,
        "the fixture must start with no terrace, or the assertion below \
         measures nothing"
    );

    // The fixture names the tile it terraces. A set order over the units of a
    // founding reaches whatever tiles those units stand on, and the founding
    // zones its own disc, so every such order is refused for a project the
    // plan already holds. This picks a tile that is free of an upgrade and
    // held by the faction, then zones that tile for a terrace, so the plan
    // names what the builder is about to build.
    let chosen = disc(world.grid(), address, SURVEY_RADIUS)
        .into_iter()
        .find(|place| {
            world.upgrade_at(*place).is_none() && world.holds(faction, *place) == Some(true)
        })
        .expect("the disc of a seated site holds a free tile its faction holds");
    world
        .zone_project(faction, chosen, UpgradeCategory::TERRACE)
        .expect("a terrace fits a tile the faction holds");

    // **The builder is fed for as long as the terrace takes.** A unit this
    // test places has no home, so nothing feeds it. Its need falls on every
    // tick, its deficit reaches the bound, and it ends after a fixed number of
    // ticks. The work of a terrace is a balance value, and it now asks for
    // more ticks than an unfed unit lives for.[^5] The builder died partway,
    // the loop below re-ordered a dead unit for the whole of its patience, and
    // the assertion after it measured the hunger and not the terrace.
    //
    // The fixture answers by holding the need where it is, through the verb a
    // caller has. No assertion below is weaker for it.
    //
    // [^5]: Balance register, the work of an upgrade level. `docs/reference/balance.md`
    let rule = world.need_rule();
    world.set_need_rule(
        NeedRule::new(
            Fix32::ZERO,
            rule.ration(),
            rule.threshold(),
            rule.recovery(),
            rule.bound(),
        )
        .expect("a rule of no decay is legal"),
    );

    // A soldier builds the tile it stands on, so the fixture puts one there.
    let builder = world
        .spawn_soldier(chosen, faction)
        .expect("the chosen tile takes a unit");
    world
        .order_build(builder, UpgradeCategory::TERRACE)
        .expect("the plan names a terrace on the tile the builder stands on");

    // The order is placed again on every tick. A builder does not stay on the
    // tile it builds, so one order at one tick finishes nothing, and a
    // separate item holds that defect.[^4]
    let mut waited = 0u32;
    while terraces_over_the_disc(&world, address) == 0 && waited < 3000 {
        world.step(THREADS).expect("the fixture steps");
        waited += 1;
        let _ = world.order_build(builder, UpgradeCategory::TERRACE);
    }
    assert!(
        terraces_over_the_disc(&world, address) > 0,
        "the fixture must finish a terrace inside the disc, or the assertion \
         below measures nothing"
    );

    // The comparison destroys the terraces and reads the scale again. That
    // holds every other input still by construction, which a comparison
    // against an earlier tick could not do.
    let terraced = world.production_scale(site).expect("the site is live");
    let places: Vec<Axial> = disc(world.grid(), address, SURVEY_RADIUS)
        .into_iter()
        .filter(|place| world.finished_upgrade(*place) == Some(UpgradeCategory::TERRACE))
        .collect();
    for place in &places {
        assert!(world.destroy_upgrade(*place), "the terrace stood there");
    }
    let bare = world.production_scale(site).expect("the site is live");
    assert!(
        terraced > bare,
        "a standing terrace must raise the scale, but it reads {terraced:?} \
         with the terraces and {bare:?} without them"
    );
}

#[test]
fn a_site_on_poor_ground_never_matches_a_site_on_good_ground() {
    // The pipeline spans a factor of eight, from its floor to its ceiling.
    // Two sites whose base rates differ by more than that span keep their
    // order whatever the world does to either of them. This is the property,
    // and it does not depend on a fixture.
    let span = SCALE_CEILING.0 / SCALE_FLOOR.0;
    assert_eq!(span, 8, "the pipeline spans a factor of eight");

    let good = Fix32::from_int(80);
    let poor = Fix32::from_int((80 / span - 1) as i16);
    let good_at_worst = sim_math::mul(good, SCALE_FLOOR);
    let poor_at_best = sim_math::mul(poor, SCALE_CEILING);
    assert!(
        good_at_worst > poor_at_best,
        "the pipeline must not invert two sites whose ground differs by more \
         than its span, but {good_at_worst:?} is not above {poor_at_best:?}"
    );

    // And the engine agrees: the scale of a live site stays inside the range
    // over a whole run, so the property above governs the engine and not only
    // the arithmetic.
    let samples = sample_a_run(0x0cac_4e77_5104_0004, 600);
    for sample in &samples {
        assert!(
            sample.scale >= SCALE_FLOOR && sample.scale <= SCALE_CEILING,
            "the scale {:?} is outside the range the pipeline states",
            sample.scale
        );
    }
}

#[test]
fn the_pass_reads_the_stored_rate_and_never_writes_it() {
    // The stored rate is the base rate. The pass reads it and never writes
    // it, so a run of any length leaves it where the founding put it.
    let mut world = world(0x0cac_4e77_5104_0005);
    let site = seated(&mut world);
    let founded = world
        .production_rate(site, FOOD)
        .expect("the founding seated the site");
    everybody_gathers(&mut world);
    run(&mut world, 400);
    let after = world
        .production_rate(site, FOOD)
        .expect("the site is still live");
    assert_eq!(
        founded, after,
        "the rate pass must not write the stored base rate"
    );

    // And the rate the next application earns is not the stored one, because
    // the world has moved under the site.
    let effective = world
        .effective_production_rate(site, FOOD)
        .expect("the site is still live");
    assert_ne!(
        effective, founded,
        "a run of four hundred ticks must move the effective rate off the \
         base rate"
    );
}
