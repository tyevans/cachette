//! A wildfire starts, spreads, ends by itself, and people change what it
//! costs.
//!
//! # What this file protects
//!
//! The engine holds no convergence test. A fire must therefore end because
//! the arithmetic makes it end: every burning tile loses a fixed positive
//! quantity of fuel on every tick, and a tile that stops burning can never
//! catch again. This file drives whole worlds and asserts that the burning
//! set empties on its own, inside a bound the test states.[^1]
//!
//! It also protects the key of every draw the fire takes. A draw keyed on the
//! wrong field draws the same wrong value on every thread and on every run,
//! so both determinism tests pass while the fire is broken. The rule is to
//! test what the value depends on, and not only that it repeats.[^2]
//!
//! Every test drives the world through its public interface, and every test
//! that concerns the engine starts at the step rather than at the
//! mechanism.[^3] [^4]
//!
//! # References
//!
//! [^1]: ADR-0005, a solver runs a fixed iteration count. `docs/adrs/REGISTRY.md`
//! [^2]: Testing rules, section 2. `.agents/rules/testing.md`
//! [^3]: Testing rules, section 5. `.agents/rules/testing.md`
//! [^4]: Testing rules, section 6. `.agents/rules/testing.md`

use cachette_core::fire;
use cachette_core::{
    Axial, Entity, FactionId, TileIdx, TileKind, World, WorldConfig, END_CAUSE_DOUSED,
    START_CAUSE_LIGHTNING,
};

/// The extent of the world every test builds.
const EXTENT: u32 = 96;

/// The factions the world holds.
const FACTIONS: u16 = 2;

/// The seed of the world every test builds.
const SEED: u64 = 20_260_907;

/// The largest number of ticks a test waits for a fire to end.
///
/// **The bound is what makes the burn-to-conclusion assertion able to fail.**
/// A fire that never ended would run to this bound and the test would fail
/// there rather than hanging.
const PATIENCE: u32 = 4000;

/// The threads that the thread-count test runs at.
const THREAD_COUNTS: [usize; 3] = [1, 2, 12];

/// Builds a bare world with no faction seeded into it.
///
/// The world holds no city and no unit, so nothing but the fire changes the
/// ground. A test that wants people spawns them itself.
fn bare_world(seed: u64) -> World {
    World::new(WorldConfig {
        width: EXTENT,
        height: EXTENT,
        seed,
        faction_count: FACTIONS,
        unit_capacity: 4096,
    })
    .expect("the extent must describe a world")
}

/// Returns the address whose neighbourhood holds the most forest.
///
/// **The fixture must supply the extreme, and not the typical case.** A fire
/// lit on one lonely forest tile burns out in a few ticks and spreads to
/// nothing, so a test on such a tile would measure the fixture rather than
/// the spread.[^1] This walks the world once and takes the densest wood it
/// finds.
///
/// # References
///
/// [^1]: Testing rules, section 2a. `.agents/rules/testing.md`
fn thickest_wood(world: &World) -> Axial {
    let reach = 3i32;
    let mut best = Axial::new(0, 0);
    let mut most = -1i32;
    for r in 0..EXTENT as i32 {
        for q in 0..EXTENT as i32 {
            let here = Axial::new(q, r);
            if world.tile_kind(here) != Some(TileKind::Forest) {
                continue;
            }
            let mut count = 0i32;
            for dr in -reach..=reach {
                for dq in -reach..=reach {
                    let there = Axial::new(q + dq, r + dr);
                    if world.tile_kind(there) == Some(TileKind::Forest) {
                        count += 1;
                    }
                }
            }
            if count > most {
                most = count;
                best = here;
            }
        }
    }
    assert!(most > 8, "the world holds no wood worth burning");
    best
}

/// Returns the tile index of an address.
fn index_of(world: &World, address: Axial) -> TileIdx {
    world
        .grid()
        .index_of(address)
        .expect("the address lies inside the world")
}

/// What one whole burn came to.
struct Burn {
    /// The ticks the fire took, from the tick it was lit to the tick the last
    /// tile stopped burning.
    ticks: u32,
    /// The tiles that caught at any point.
    ///
    /// The count comes from the field and not from the log, because the step
    /// clears the log before any system runs. An ignition a caller made
    /// between two steps is therefore in the field and not in the log that
    /// the next step hands back.
    started: i64,
    /// The tiles that the spread lit, as the log reported them.
    spread: usize,
    /// The tiles that ran out of fuel.
    burnt_out: i64,
    /// The tiles that people put out.
    doused: i64,
    /// The units the fire ended.
    burned_units: i64,
    /// Whether the fire ended without the test doing anything.
    ended_by_itself: bool,
}

/// Steps the world until nothing burns, and reports what the burn came to.
fn burn_to_the_end(world: &mut World, threads: usize) -> Burn {
    let mut spread = 0usize;
    let mut ticks = 0u32;
    let mut ended_by_itself = false;
    for _ in 0..PATIENCE {
        world.step(threads).expect("a step runs");
        ticks += 1;
        spread += world
            .fires_started()
            .iter()
            .filter(|event| event.cause == fire::START_CAUSE_SPREAD)
            .count();
        if world.fire().burning_count() == 0 {
            ended_by_itself = true;
            break;
        }
    }
    Burn {
        ticks,
        started: world.fire().started_total(),
        spread,
        burnt_out: world.fire().burnt_out_total(),
        doused: world.fire().doused_total(),
        burned_units: world.fire().burned_units_total(),
        ended_by_itself,
    }
}

#[test]
fn a_fire_a_caller_starts_spreads_and_then_ends_by_itself() {
    let mut world = bare_world(SEED);
    let start = thickest_wood(&world);
    assert!(
        world.ignite(index_of(&world, start)),
        "the thickest wood in the world must catch"
    );
    assert_eq!(
        world.fire().burning_count(),
        1,
        "one tile burns before the first step"
    );

    let burn = burn_to_the_end(&mut world, 4);

    assert!(
        burn.ended_by_itself,
        "the fire still burned after {PATIENCE} ticks, so it does not end on its own"
    );
    // **A fire that fizzles on the first tick is as wrong as one that never
    // ends.** The tile that was lit is one of the tiles that caught, so more
    // than one means the fire reached its neighbours.
    assert!(
        burn.started > 1 && burn.spread > 0,
        "the fire caught {} tiles and the log reported {} spreads, so it never spread",
        burn.started,
        burn.spread
    );
    assert_eq!(
        burn.started,
        burn.burnt_out + burn.doused,
        "every tile that caught must also stop"
    );
    assert!(
        world.check_invariants(),
        "the world must hold its invariants after the fire"
    );
    // Every tile that burned is spent, and spent ground is what stops the
    // fire coming back.
    assert_eq!(
        world.tile_is_spent(start),
        Some(true),
        "the tile that was lit must be spent"
    );
    assert!(
        !world.ignite(index_of(&world, start)),
        "spent ground must refuse a second fire"
    );
    println!(
        "seed {SEED}: {} tiles caught in {} ticks, {} burnt out, {} doused",
        burn.started, burn.ticks, burn.burnt_out, burn.doused
    );
}

#[test]
fn a_fire_gives_one_answer_at_any_thread_count() {
    let mut hashes = Vec::new();
    let mut summaries = Vec::new();
    for threads in THREAD_COUNTS {
        let mut world = bare_world(SEED);
        let start = thickest_wood(&world);
        assert!(world.ignite(index_of(&world, start)));
        let burn = burn_to_the_end(&mut world, threads);
        assert!(burn.ended_by_itself);
        hashes.push(world.state_hash().finish());
        summaries.push((burn.started, burn.ticks, burn.burnt_out));
    }
    assert!(
        hashes.windows(2).all(|pair| pair[0] == pair[1]),
        "the state hash differs between thread counts: {hashes:?}"
    );
    assert!(
        summaries.windows(2).all(|pair| pair[0] == pair[1]),
        "the burn differs between thread counts: {summaries:?}"
    );
}

#[test]
fn the_spread_draw_reads_every_field_of_its_key() {
    let ground = fire::GroundReading {
        kind: TileKind::Forest,
        carries_upgrade: false,
        is_wet: false,
        cloud_share: 0,
        cloud_whole: 255,
        wind_along: [0; 6],
        wind_whole: 96,
    };
    let intensity = fire::INTENSITY_CEILING;
    // The chance must sit away from both ends, or a change to the key could
    // not change the answer and the test would measure nothing.
    let chance = fire::spread_chance(intensity, 0, ground);
    assert!(
        chance > 0 && chance < fire::CHANCE_WHOLE,
        "the fixture must give a chance the draw can fall either side of"
    );

    let base: Vec<bool> = (0..64)
        .map(|frame| fire::spreads(7, frame, TileIdx(100), 0, intensity, ground))
        .collect();

    // The frame is in the key. A fire that drew the same answer on every tick
    // would spread in a fixed pattern for ever.
    assert!(
        base.iter().any(|hit| *hit) && base.iter().any(|hit| !*hit),
        "the answer does not depend on the frame"
    );

    // The tile is in the key. A draw keyed on nothing but the frame would
    // light every neighbour of every fire on the same tick.
    let other_tile: Vec<bool> = (0..64)
        .map(|frame| fire::spreads(7, frame, TileIdx(101), 0, intensity, ground))
        .collect();
    assert_ne!(base, other_tile, "the answer does not depend on the tile");

    // The direction is in the key. A draw keyed on the tile alone would light
    // all six neighbours at once or none of them.
    let other_direction: Vec<bool> = (0..64)
        .map(|frame| fire::spreads(7, frame, TileIdx(100), 1, intensity, ground))
        .collect();
    assert_ne!(
        base, other_direction,
        "the answer does not depend on the direction"
    );

    // The seed is in the key. Two worlds of different seeds would burn the
    // same way without it.
    let other_seed: Vec<bool> = (0..64)
        .map(|frame| fire::spreads(9, frame, TileIdx(100), 0, intensity, ground))
        .collect();
    assert_ne!(base, other_seed, "the answer does not depend on the seed");
}

#[test]
fn the_casualty_draw_reads_the_whole_identity_and_the_frame() {
    let intensity = fire::INTENSITY_CEILING;
    let base: Vec<bool> = (0..256)
        .map(|frame| fire::burns_unit(7, frame, 0x0000_0001_0000_0005, intensity))
        .collect();
    assert!(
        base.iter().any(|hit| *hit) && base.iter().any(|hit| !*hit),
        "the answer does not depend on the frame"
    );
    // **The generation is in the key, not only the slot.** A unit spawned
    // into the slot a burned unit left must draw its own answer, or it dies
    // on the same ticks the unit before it died on.
    let later_generation: Vec<bool> = (0..256)
        .map(|frame| fire::burns_unit(7, frame, 0x0000_0002_0000_0005, intensity))
        .collect();
    assert_ne!(
        base, later_generation,
        "the answer does not depend on the generation of the identity"
    );
    let other_slot: Vec<bool> = (0..256)
        .map(|frame| fire::burns_unit(7, frame, 0x0000_0001_0000_0006, intensity))
        .collect();
    assert_ne!(base, other_slot, "the answer does not depend on the slot");
}

#[test]
fn water_carries_no_fuel_and_wet_ground_refuses_a_fire() {
    let dry_wood = fire::GroundReading {
        kind: TileKind::Forest,
        carries_upgrade: false,
        is_wet: false,
        cloud_share: 0,
        cloud_whole: 255,
        wind_along: [0; 6],
        wind_whole: 96,
    };
    assert!(dry_wood.admits_fire());

    let water = fire::GroundReading {
        kind: TileKind::Water,
        ..dry_wood
    };
    assert!(!water.admits_fire(), "open water must never catch");
    assert_eq!(fire::spread_chance(fire::INTENSITY_CEILING, 0, water), 0);

    let mountain = fire::GroundReading {
        kind: TileKind::Mountain,
        ..dry_wood
    };
    assert!(!mountain.admits_fire(), "bare rock must never catch");

    let rained_on = fire::GroundReading {
        is_wet: true,
        ..dry_wood
    };
    assert!(!rained_on.admits_fire(), "wet ground must never catch");
    assert_eq!(
        fire::spread_chance(fire::INTENSITY_CEILING, 0, rained_on),
        0,
        "a fire must not cross onto wet ground"
    );
}

#[test]
fn the_wind_carries_a_fire_further_downwind_than_upwind() {
    let still = fire::GroundReading {
        kind: TileKind::Forest,
        carries_upgrade: false,
        is_wet: false,
        cloud_share: 0,
        cloud_whole: 255,
        wind_along: [0; 6],
        wind_whole: 96,
    };
    let mut blowing = still;
    // The wind points along direction zero and away from direction three.
    blowing.wind_along[0] = 96;
    blowing.wind_along[3] = -96;
    let downwind = fire::spread_chance(fire::INTENSITY_CEILING, 0, blowing);
    let upwind = fire::spread_chance(fire::INTENSITY_CEILING, 3, blowing);
    let calm = fire::spread_chance(fire::INTENSITY_CEILING, 0, still);
    assert!(
        downwind > calm && calm > upwind,
        "the wind must divide the chance: downwind {downwind}, calm {calm}, upwind {upwind}"
    );
}

/// Places a crew of the given size on and around one address.
///
/// The people stand three to a tile over the neighbourhood, because that is
/// the density at which a crew beats a fire at its height. A thinner crew
/// falls behind, and a fixture that supplied one would measure itself.[^1]
///
/// # References
///
/// [^1]: Testing rules, section 2a. `.agents/rules/testing.md`
fn place_crew(world: &mut World, middle: Axial, wanted: usize) -> Vec<Entity> {
    let mut crew = Vec::new();
    'places: for r in -3i32..=3 {
        for q in -3i32..=3 {
            let here = Axial::new(middle.q + q, middle.r + r);
            if world.tile_kind(here).is_none() {
                continue;
            }
            for _ in 0..3 {
                if crew.len() >= wanted {
                    break 'places;
                }
                match world.spawn_soldier(here, FactionId(0)) {
                    Ok(unit) => crew.push(unit),
                    Err(_) => break,
                }
            }
        }
    }
    assert!(
        crew.len() >= wanted,
        "the fixture must place a whole crew, and it placed {}",
        crew.len()
    );
    crew
}

/// The people of a crew that are still alive.
fn survivors(world: &World, crew: &[Entity]) -> Vec<Entity> {
    crew.iter()
        .filter(|unit| world.douse_order(**unit).is_some())
        .copied()
        .collect()
}

#[test]
fn people_sent_to_the_fire_change_how_much_burns() {
    // Two worlds of one seed, holding one crew each. In one world the control
    // plane sends the crew at the fire on every tick. In the other the same
    // people stand there and do nothing, so the only difference between the
    // two runs is the order.
    let crew_size = 120usize;
    let mut fought_world = bare_world(SEED);
    let start = thickest_wood(&fought_world);
    let fought_crew = place_crew(&mut fought_world, start, crew_size);
    assert!(fought_world.ignite(index_of(&fought_world, start)));

    let mut free_world = bare_world(SEED);
    let free_crew = place_crew(&mut free_world, start, crew_size);
    assert_eq!(free_crew.len(), fought_crew.len());
    assert!(free_world.ignite(index_of(&free_world, start)));

    // **One command a tick, over the whole set.** The verb takes the burning
    // tiles as the seed set of one destination field, so the crew follows the
    // fire without any unit searching for it and without this test naming a
    // tile.[^1]
    //
    // [^1]: ADR-0125, the control plane names the seed set of a destination field, decision D1. `docs/adrs/draft/adr-0125-the-control-plane-names-the-seed-set-of-a-destination-field.md`
    let mut fought_ticks = 0u32;
    let mut fought_ended = false;
    for _ in 0..PATIENCE {
        let live = survivors(&fought_world, &fought_crew);
        if !live.is_empty() {
            fought_world
                .order_douse_set(&live, 0)
                .expect("every unit of the set is live and the plane exists");
        }
        fought_world.step(4).expect("a step runs");
        fought_ticks += 1;
        if fought_world.fire().burning_count() == 0 {
            fought_ended = true;
            break;
        }
    }
    let free = burn_to_the_end(&mut free_world, 4);

    let fought_started = fought_world.fire().started_total();
    let fought_doused = fought_world.fire().doused_total();
    let fought_dead = fought_world.fire().burned_units_total();

    println!(
        "with a crew: {fought_started} tiles in {fought_ticks} ticks, \
         {fought_doused} doused, {fought_dead} dead. \
         without: {} tiles in {} ticks, {} doused, {} dead",
        free.started, free.ticks, free.doused, free.burned_units
    );

    assert!(fought_ended, "the fought fire never ended");
    assert!(free.ended_by_itself, "the free fire never ended");
    // **The crew must not merely shave the total.** It must hold the fire to
    // a fraction of what it reaches when nobody fights it, or the order is
    // decoration.
    assert!(
        fought_started * 2 < free.started,
        "the crew barely changed the outcome: {fought_started} tiles caught \
         with people and {} without",
        free.started
    );
    assert!(
        fought_doused > 0,
        "no tile was put out, so the suppression reached nothing"
    );
    // **Fighting a fire costs bodies.** A crew that stands in a fire and
    // loses nobody makes the order free, and a free order is not a choice.
    assert!(
        fought_dead > 0,
        "the fire ended nobody, so fighting it costs nothing"
    );
    assert_eq!(
        free.doused, 0,
        "nobody fought the second fire, so nothing was put out"
    );
    assert_eq!(
        free.burnt_out, free.started,
        "every tile of the free fire must have burned out"
    );
    let _ = END_CAUSE_DOUSED;
}

#[test]
fn lightning_starts_a_fire_without_a_caller() {
    // **The test drives the engine and not the draw.** The lightning path
    // belongs to the step, so a test that called the draw directly would
    // prove that the draw works and not that anything reaches it.[^1]
    //
    // [^1]: Testing rules, section 5. `.agents/rules/testing.md`
    let mut world = bare_world(SEED);
    // One strike in every four ticks, so a short run reaches one.
    world.set_lightning_chance(fire::LIGHTNING_WHOLE / 4);
    let mut struck = 0usize;
    for _ in 0..200 {
        world.step(4).expect("a step runs");
        struck += world
            .fires_started()
            .iter()
            .filter(|event| event.cause == START_CAUSE_LIGHTNING)
            .count();
        if struck > 0 {
            break;
        }
    }
    assert!(
        struck > 0,
        "the engine started no fire of its own in two hundred ticks"
    );
    // A world that states no chance never catches by itself, and that is what
    // a world states until a caller says otherwise.
    let mut quiet = bare_world(SEED);
    assert_eq!(quiet.lightning_chance(), 0);
    for _ in 0..200 {
        quiet.step(4).expect("a step runs");
    }
    assert_eq!(
        quiet.fire().started_total(),
        0,
        "a world that states no chance must not catch by itself"
    );
}
