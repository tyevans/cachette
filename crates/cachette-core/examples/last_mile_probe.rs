//! Reads how long a laden carrier takes to reach the tile of its home site.
//!
//! The destination field and the return field are both at the pitch of one
//! level 1 cell. A unit that climbs either of them arrives at the cell that
//! holds its target and not at the tile. The delivery reads the tile the unit
//! stands on, so a carrier that reached the cell has not delivered anything.
//!
//! The probe seats a site, homes a unit to it, loads the unit, walks it a few
//! cells away and then lets the engine bring it home. It reports the tick the
//! unit entered the home cell and the tick it stood on the home tile. The
//! difference between the two is the last mile.
//!
//! The probe drives the step. It calls no pass of its own.[^1]
//!
//! # References
//!
//! [^1]: Testing rules, section 5. `.agents/rules/testing.md`

use cachette_core::cohort::NeedRule;
use cachette_core::resource::ResourceKind;
use cachette_core::{Axial, Entity, FactionId, Fix32, World, WorldConfig};

/// The extent of the probe world.
const EXTENT: u32 = 256;

/// The ticks the probe gives the unit to fill its carry.
const GATHER_TICKS: u32 = 4096;

/// Returns the value of a named argument, or the fallback.
fn number(name: &str, fallback: u64) -> u64 {
    let mut args = std::env::args().skip(1);
    while let Some(argument) = args.next() {
        if argument == name {
            return args
                .next()
                .and_then(|value| value.parse().ok())
                .unwrap_or(fallback);
        }
    }
    fallback
}

/// Returns the first open address that carries food.
fn a_tile_that_carries_food(world: &World) -> Axial {
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
            return address;
        }
    }
    panic!("the probe world holds no open tile that carries food");
}

/// Returns the level 1 cell that covers an address.
fn cell_of(world: &World, address: Axial) -> Option<u32> {
    let layout = world.pyramid().layout();
    let tile = world.grid().index_of(address)?;
    Some(layout.block_of_key(layout.key_of(tile)?))
}

/// Returns an open address that lies a given walk from another.
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

fn main() {
    let seed = number("--seed", 7);
    let ticks = number("--ticks", 600) as u32;
    let span = number("--span", 40) as u32;

    let mut world = World::new(WorldConfig {
        width: EXTENT,
        height: EXTENT,
        seed,
        faction_count: 2,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    })
    .expect("the extent must describe a world");
    // Nothing starves and nothing eats, so the probe measures the walk.
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
    let home = a_tile_that_carries_food(&world);
    let site = world
        .found_settlement(home, FactionId(0))
        .expect("the ground admits a settlement");
    let away = ground_at(&world, home, span);

    // The unit gathers away from home with no home, so it holds a load when
    // it is given one. The gather resolve runs before the delivery in a
    // frame, so a unit that had a home would end each frame empty.
    let unit: Entity = world
        .spawn_soldier(away, FactionId(0))
        .expect("the ground admits the unit");
    // **The unit gathers until it is laden, and the probe asserts that it
    // is.** The deliver option is worth nothing to a unit below the carry
    // mark, so a unit that gathered a little roams instead of going home,
    // and the probe then measures the roam. The first draft of this probe
    // gathered a fixed count, reached twenty against a mark of thirty-two,
    // and measured nothing.[^2]
    //
    // [^2]: Testing rules, section 2a. `.agents/rules/testing.md`
    let mark = world.carry_mark().0;
    for _ in 0..GATHER_TICKS {
        if world
            .soldiers()
            .carry(unit)
            .is_some_and(|load| load.of(ResourceKind::Food).0 >= mark)
        {
            break;
        }
        world.order_gather(unit, ResourceKind::Food);
        world.step(1).expect("the step must run");
    }
    let at = world
        .soldiers()
        .address(unit)
        .expect("the unit is still alive");
    assert!(
        world.set_home_site(unit, Some(site)),
        "the unit takes its home"
    );
    let carried = world
        .soldiers()
        .carry(unit)
        .expect("the unit is live")
        .of(ResourceKind::Food)
        .0;
    let home_cell = cell_of(&world, home).expect("the home names a cell");
    println!(
        "seed {seed}: home {home:?} in cell {home_cell}, the unit is at {at:?}, \
         {} tiles away, carrying {carried}",
        home.distance(at)
    );
    if carried < mark {
        println!(
            "  the unit carries {carried} against a mark of {mark}, so it is not laden \
             and the probe measured nothing"
        );
        return;
    }

    let mut entered = None;
    let mut stood = None;
    let mut delivered = None;
    for tick in 0..ticks {
        world.step(1).expect("the step must run");
        let Some(at) = world.soldiers().address(unit) else {
            println!("  tick {tick}: the unit is gone");
            break;
        };
        if cell_of(&world, at) == Some(home_cell) && entered.is_none() {
            entered = Some(tick);
        }
        if at == home && stood.is_none() {
            stood = Some(tick);
        }
        let held = world
            .soldiers()
            .carry(unit)
            .map_or(0, |load| load.of(ResourceKind::Food).0);
        if held == 0 && delivered.is_none() {
            delivered = Some(tick);
            break;
        }
    }
    println!(
        "  it entered the home cell at {entered:?}, stood on the home tile at {stood:?}, \
         and delivered at {delivered:?}"
    );
    match (entered, delivered) {
        (Some(entered), Some(delivered)) => {
            println!("  the last mile cost {} ticks", delivered - entered);
        }
        (Some(_), None) => println!("  it never delivered inside {ticks} ticks"),
        _ => println!("  it never reached the home cell inside {ticks} ticks"),
    }
}
