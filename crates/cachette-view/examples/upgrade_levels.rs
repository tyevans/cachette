//! Writes the three pictures that show an upgrade category and its level.
//!
//! The map draws the category as a shape over a dark square, and the level as
//! a count of pips under it.[^1] The pictures are a close-up of one road at
//! two levels, a close-up of several categories at once, and the same view
//! under the upgrade overlay.
//!
//! Usage: `cargo run --example upgrade_levels`
//!
//! The format is binary PPM, which every image tool reads and which needs no
//! dependency. The files land under a build directory and are not committed.
//!
//! The viewer reads the world and writes nothing to it.[^2]
//!
//! # References
//!
//! [^1]: ADR-0151, an upgrade is a category with a ground fit and a level, decision D5. `docs/adrs/draft/adr-0151-an-upgrade-is-a-category-with-a-ground-fit-and-a-level.md`
//! [^2]: ADR-0067, the viewer reads the world and never writes to it, decision D1. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`

// An example is its own crate, so the allowance at the viewer's crate root
// does not reach it. ADR-0067 D3 puts the float boundary at the viewer, and a
// tile width is a viewer value.
#![allow(clippy::disallowed_types)]

use std::path::Path;

use cachette_core::upgrade::UpgradeCategory;
use cachette_core::{Axial, Entity, FactionId, Holder, World, WorldConfig};
use cachette_view::picture::write_ppm;
use cachette_view::{draw_frame_paced, Camera, Canvas, Metrics, Motion, Overlay, Pace};

/// The extent of the world the pictures draw, in tiles a side.
const EXTENT: u32 = 32;

/// The seed of the world the pictures draw.
const SEED: u64 = 7;

/// The ticks the world runs before the faction holds ground.
const HOLDING_TICKS: u32 = 6;

/// The most ticks one build takes before the example gives up.
const BUILD_CEILING: u32 = 600;

/// The ticks a build under work is given, so that the entry holds work and no
/// level.
const UNDER_WORK_TICKS: u32 = 3;

/// The size of every picture, in pixels.
const WINDOW: (usize, usize) = (960, 720);

/// The directory the pictures land in.
const OUT: &str = "target/upgrade-levels";

/// One thing the example builds.
struct Plan {
    /// The category to build.
    category: UpgradeCategory,
    /// The level that must stand when the build stops. Zero leaves the site
    /// under work.
    level: u8,
}

fn main() {
    let mut world = World::new(WorldConfig {
        width: EXTENT,
        height: EXTENT,
        seed: SEED,
        faction_count: 2,
        unit_capacity: 4096,
    })
    .expect("the extent describes a world");
    let seat = Axial::new(EXTENT as i32 / 2, EXTENT as i32 / 2);
    world
        .found_settlement(seat, FactionId(0))
        .expect("the seat admits a city");
    for _ in 0..HOLDING_TICKS {
        world.step(1).expect("the step must run");
    }

    // A road at two levels, and one of each other category the table holds.
    let plans = [
        Plan {
            category: UpgradeCategory::ROAD,
            level: 1,
        },
        Plan {
            category: UpgradeCategory::ROAD,
            level: 2,
        },
        Plan {
            category: UpgradeCategory::TERRACE,
            level: 1,
        },
        Plan {
            category: UpgradeCategory::STORE,
            level: 1,
        },
        Plan {
            category: UpgradeCategory::WALL,
            level: 1,
        },
        Plan {
            category: UpgradeCategory::WONDER,
            level: 0,
        },
    ];

    let mut built: Vec<Axial> = Vec::new();
    let mut builders: Vec<Entity> = Vec::new();
    for plan in plans {
        let Some(address) = a_free_tile(&world, seat, &built, plan.category) else {
            println!("no tile fits {:?}, so it was left out", plan.category);
            continue;
        };
        builders.push(raise(&mut world, address, &plan));
        built.push(address);
    }
    // The builders leave the tiles, because the disc of a unit is wider than
    // the mark of a site and would cover it.
    let corner = (0..EXTENT as i32)
        .map(|column| Axial::new(column, 0))
        .find(|at| world.admits_a_unit(*at))
        .expect("the world holds open ground at its edge");
    for builder in builders {
        world
            .place_soldier(builder, corner)
            .expect("the corner admits the unit");
    }
    world.rebuild_bridge(1).expect("the bridge rebuilds");

    std::fs::create_dir_all(OUT).expect("the output directory must open");

    // The two roads sit beside each other, so one picture holds both.
    let roads = built[0];
    write(&world, roads, 64.0, false, "two-levels-of-one-road.ppm");
    let middle = built[built.len() / 2];
    write(&world, middle, 40.0, false, "several-categories.ppm");
    write(&world, middle, 40.0, true, "upgrade-overlay.ppm");

    for address in built {
        match world.upgrade_at(address) {
            Some(site) => println!(
                "{address:?}: category {}, level {}, work {}",
                site.category.to_u8(),
                site.level,
                site.progress.0
            ),
            None => println!("{address:?}: nothing stands"),
        }
    }
}

/// Returns a held tile that fits one category and carries no upgrade.
///
/// The search runs outward from the seat, so the sites land together and one
/// close-up holds them all.
fn a_free_tile(
    world: &World,
    seat: Axial,
    taken: &[Axial],
    category: UpgradeCategory,
) -> Option<Axial> {
    let width = world.grid().width() as i32;
    (0..width)
        .flat_map(|row| (0..width).map(move |column| Axial::new(column, row)))
        .filter(|at| *at != seat && !taken.contains(at))
        .filter(|at| world.tile_holder(*at).and_then(Holder::faction) == Some(FactionId(0)))
        .filter(|at| world.upgrade_at(*at).is_none() && world.admits_a_unit(*at))
        .filter(|at| {
            world.tile_kind(*at).is_some_and(|ground| {
                world
                    .upgrade_table()
                    .row(category, 1)
                    .is_some_and(|row| row.fits(ground))
            })
        })
        .min_by_key(|at| at.distance(seat))
}

/// Builds one thing on one tile, and returns the builder.
///
/// **The builder is put back on the tile and ordered again each tick.** A unit
/// that finished a level walks away, and a unit that stands elsewhere adds no
/// work.
fn raise(world: &mut World, address: Axial, plan: &Plan) -> Entity {
    let mut builder = world
        .spawn_soldier(address, FactionId(0))
        .expect("the tile admits a unit");
    world.rebuild_bridge(1).expect("the bridge rebuilds");
    world
        .zone_project(FactionId(0), address, plan.category)
        .expect("the plan takes the project");
    let mut ticks = 0;
    while !reached(world, address, plan, ticks) {
        assert!(
            ticks < BUILD_CEILING,
            "the build of {:?} at {address:?} never finished",
            plan.category
        );
        // A unit of this world can die of what the world does to it, and a
        // dead unit takes no order. A new one takes over the site.
        if world.soldiers().address(builder).is_none() {
            builder = world
                .spawn_soldier(address, FactionId(0))
                .expect("the tile admits a unit");
        }
        world
            .place_soldier(builder, address)
            .expect("the tile admits the builder");
        world.rebuild_bridge(1).expect("the bridge rebuilds");
        world
            .order_build(builder, plan.category)
            .expect("the engine takes the order");
        world.step(1).expect("the step must run");
        ticks += 1;
    }
    world.stop_build(builder);
    builder
}

/// Reports whether a build has reached what its plan asked for.
fn reached(world: &World, address: Axial, plan: &Plan, ticks: u32) -> bool {
    if plan.level == 0 {
        return ticks >= UNDER_WORK_TICKS;
    }
    world
        .upgrade_at(address)
        .is_some_and(|site| site.level >= plan.level)
}

/// Writes one picture, and returns nothing.
fn write(world: &World, over: Axial, tile: f32, overlay: bool, name: &str) {
    let mut canvas = Canvas::new(WINDOW.0, WINDOW.1);
    let camera = Camera::at_tile_size(tile)
        .looking_at(over, &canvas)
        .clamped(world, &canvas);
    let layer = if overlay {
        cachette_view::overlay::named("upgrade")
    } else {
        None
    };
    draw_frame_paced(
        world,
        camera,
        &Metrics::start(),
        &[],
        Overlay::Glass { reference: true },
        layer,
        Pace::STILL,
        &mut Motion::for_frame(WINDOW.0, WINDOW.1),
        &mut canvas,
    )
    .expect("the world draws");

    let path = Path::new(OUT).join(name);
    let mut file =
        std::io::BufWriter::new(std::fs::File::create(&path).expect("the output file must open"));
    write_ppm(&canvas, &mut file).expect("the pixels must write");
    println!("{}: {tile} pixels a tile over {over:?}", path.display());
}
