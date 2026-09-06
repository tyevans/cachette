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
//! # References
//!
//! [^1]: Backlog item 0136, provision a founded site from the ground it reaches. `docs/backlog/complete/0136-provision-a-founded-site-from-the-ground-it-reaches.md`
//! [^2]: Testing rules, section 5. `.agents/rules/testing.md`
//! [^3]: Testing rules, section 2a. `.agents/rules/testing.md`
//! [^4]: Backlog item 0505, keep a builder on the tile it builds until the work is done. `docs/backlog/complete/0505-keep-a-builder-on-the-tile-it-builds-until-the-work-is-done.md`

use cachette_core::effective::{SCALE_CEILING, SCALE_FLOOR, WET_WEIGHT};
use cachette_core::founding::{disc, SURVEY_RADIUS};
use cachette_core::hex::Axial;
use cachette_core::resource::ResourceKind;
use cachette_core::upgrade::UpgradeCategory;
use cachette_core::{sim_math, CommodityId, Entity, Fix32, World, WorldConfig};

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
    // The engine's own weather solve wets and dries the ground of the site
    // over a run. The test finds two ticks that agree in every other input
    // and differ only in the weather, so the weather is the one term left
    // that can move. That holds the other three still without a verb.
    let samples = sample_a_run(0x0cac_4e77_5104_0002, 1600);
    let wet: Vec<&Sample> = samples.iter().filter(|sample| sample.wet).collect();
    let dry: Vec<&Sample> = samples.iter().filter(|sample| !sample.wet).collect();
    assert!(
        !wet.is_empty() && !dry.is_empty(),
        "the fixture must hold both a wet tick and a dry tick, or the \
         assertion below measures nothing. It held {} wet and {} dry",
        wet.len(),
        dry.len()
    );

    let mut pairs = 0usize;
    for soaked in &wet {
        for parched in &dry {
            if soaked.taken != parched.taken || soaked.terraces != parched.terraces {
                continue;
            }
            pairs += 1;
            assert_eq!(
                soaked.scale,
                sim_math::add(parched.scale, WET_WEIGHT),
                "wet ground must add exactly the wet weight to the scale"
            );
            assert!(
                soaked.scale > parched.scale,
                "wet ground must give a higher scale than dry ground"
            );
        }
    }
    assert!(
        pairs > 0,
        "the fixture must hold a wet tick and a dry tick that agree in every \
         other input, or the assertion above measures nothing"
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
    assert_eq!(
        terraces_over_the_disc(&world, address),
        0,
        "the fixture must start with no terrace, or the assertion below \
         measures nothing"
    );

    // The order is placed again on every tick. A builder does not stay on the
    // tile it builds, so one order at one tick finishes nothing, and a
    // separate item holds that defect.[^4]
    let units: Vec<Entity> = world.soldiers().iter().collect();
    let (refused, reason) = world.order_build_set_reporting(&units, UpgradeCategory::TERRACE);
    assert!(
        refused < units.len(),
        "the fixture must place at least one build order, but every one of {} \
         was refused for {reason:?}",
        units.len()
    );

    let mut waited = 0u32;
    while terraces_over_the_disc(&world, address) == 0 && waited < 3000 {
        world.step(THREADS).expect("the fixture steps");
        waited += 1;
        let live: Vec<Entity> = world.soldiers().iter().collect();
        world.order_build_set(&live, UpgradeCategory::TERRACE);
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
