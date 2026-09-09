//! Writes the pictures that decide whether a unit stands out from the ground.
//!
//! **A unit on plain grass is the typical case, and it decides nothing.** A
//! fixture that models the typical case supplies no extreme, so a reader who
//! judges the unit mark from it judges the fixture.[^1] This example writes
//! the cases that do decide it: a unit over its own faction's holding, a unit
//! over the near-pure colour of a holding edge, a unit whose mark reaches
//! over open water, a crowd on one tile, a unit at the edge of the frame over
//! the page behind the map, and the same world at three zooms.
//!
//! Usage: `cargo run --example unit_contrast -- <out-directory>`
//!
//! The format is binary PPM, which every image tool reads and which needs no
//! dependency.
//!
//! The viewer reads the world and writes nothing to it.[^2]
//!
//! # References
//!
//! [^1]: Testing Rules, a fixture supplies the input. `.agents/rules/testing.md`
//! [^2]: ADR-0067, the viewer reads the world and never writes to it, decision D1. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`

// An example is its own crate, so the allowance at the viewer's crate root
// does not reach it. ADR-0067 D3 puts the float boundary at the viewer, and a
// tile width is a viewer value.
#![allow(clippy::disallowed_types)]

use std::path::Path;

use cachette_core::founding::FoundingOutcome;
use cachette_core::{Axial, FactionId, World, WorldConfig};
use cachette_view::picture::write_ppm;
use cachette_view::{paint, Camera, Canvas};

/// The extent of the world, in tiles a side.
const EXTENT: u32 = 128;

/// The seed of the world.
const SEED: u64 = 0x0cac_4e77_0032;

/// The factions the run seats.
const FACTIONS: u16 = 4;

/// The threads each step runs at.
const THREADS: usize = 4;

/// The tick the pictures stand at.
///
/// The factions must hold ground before a unit can stand on the colour of its
/// own faction, and a holding takes time to spread.
const TICK: u64 = 400;

/// The size of every picture, in pixels.
const SIDE: usize = 300;

/// How many units one crowd holds.
///
/// A tile admits fewer than this, so the crowd also draws the mark of an
/// over-full tile. That is the densest thing the picture can hold.
const CROWD: u32 = 12;

fn main() {
    let out = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "target/units/cases".to_string());
    std::fs::create_dir_all(&out).expect("the output directory must open");

    let mut world = World::new(WorldConfig {
        width: EXTENT,
        height: EXTENT,
        seed: SEED,
        faction_count: FACTIONS,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    })
    .expect("the extent describes a world");
    let outcomes = world.seed_world().expect("the world seeds once");
    let seat = outcomes
        .iter()
        .find_map(FoundingOutcome::founding)
        .expect("the run seated a faction")
        .place();
    while world.tick().0 < TICK {
        world.step(THREADS).expect("the step must run");
    }

    // A tile beside open water. A bead at the region zoom is wider than a
    // tile there, so part of its outline lands on the water.
    let coast = coast(&world).unwrap_or(seat);

    // The run puts its units where its own rules take them, and that is
    // neither the coast nor the corner of the map. Both cases need units
    // standing in them, so the picture puts them there.
    let corner = corner(&world).unwrap_or(seat);
    for place in [coast, corner] {
        settle(&mut world, place);
    }

    // A crowd on one tile, and a second crowd on the tile beside it, so the
    // picture holds two crowds that touch.
    let crowded = crowd_place(&world, seat);
    for index in 0..CROWD {
        let _ = world.spawn_soldier(crowded, FactionId((index % 2) as u16));
    }
    for index in 0..CROWD {
        let _ = world.spawn_soldier(
            Axial::new(crowded.q + 1, crowded.r),
            FactionId((index % 2) as u16),
        );
    }
    world.rebuild_bridge(THREADS).expect("the bridge rebuilds");

    let cases = [
        ("region-coast", coast, 5.0, true),
        ("city-holding", seat, 14.0, true),
        ("close-holding", seat, 44.0, true),
        ("crowd", crowded, 24.0, true),
        ("frame-edge", corner, 12.0, false),
        ("region-whole", seat, 4.0, true),
    ];
    for (name, over, tile, clamped) in cases {
        write(&world, over, tile, clamped, &out, name);
    }
}

/// Returns a land tile that touches open water, if the world holds one.
fn coast(world: &World) -> Option<Axial> {
    let grid = world.grid();
    (0..grid.tile_count())
        .map(|index| Axial::new((index % grid.width()) as i32, (index / grid.width()) as i32))
        .find(|address| {
            world.admits_a_unit(*address)
                && [(1, 0), (-1, 0), (0, 1), (0, -1)].iter().any(|(dq, dr)| {
                    let beside = Axial::new(address.q + dq, address.r + dr);
                    world.tile_capacity(beside).is_some() && !world.admits_a_unit(beside)
                })
        })
}

/// Returns the land tile nearest the origin of the world.
///
/// The camera looks at this tile without a clamp, so the page behind the map
/// takes part of the frame and the units there stand at the edge of it.
fn corner(world: &World) -> Option<Axial> {
    let grid = world.grid();
    (0..grid.tile_count())
        .map(|index| Axial::new((index % grid.width()) as i32, (index / grid.width()) as i32))
        .filter(|address| world.admits_a_unit(*address))
        .min_by_key(|address| address.q + address.r)
}

/// Spawns one unit of each of two factions on every tile near a place.
///
/// The two factions put a bead of one colour beside a bead of another, which
/// is the case a watcher must read at a glance.
fn settle(world: &mut World, place: Axial) {
    for down in -2i32..=2 {
        for across in -2i32..=2 {
            let address = Axial::new(place.q + across, place.r + down);
            for faction in 0..2u16 {
                if (across + down + i32::from(faction)) % 2 == 0 {
                    let _ = world.spawn_soldier(address, FactionId(faction));
                }
            }
        }
    }
}

/// Returns a tile near the seat that admits a crowd, and its neighbour too.
fn crowd_place(world: &World, seat: Axial) -> Axial {
    (0..8)
        .map(|step| Axial::new(seat.q + step, seat.r + 2))
        .find(|address| {
            world.admits_a_unit(*address)
                && world.admits_a_unit(Axial::new(address.q + 1, address.r))
        })
        .unwrap_or(seat)
}

/// Writes one picture, and returns nothing.
fn write(world: &World, over: Axial, tile: f32, clamped: bool, out: &str, name: &str) {
    let mut canvas = Canvas::new(SIDE, SIDE);
    let looking = Camera::at_tile_size(tile).looking_at(over, &canvas);
    let camera = if clamped {
        looking.clamped(world, &canvas)
    } else {
        looking
    };
    // The map pass alone, and no cards over it. A card covers most of a
    // small picture, and the question here is the ground and the units on
    // it.
    paint::draw(world, camera, &mut canvas).expect("the world draws");

    let path = Path::new(out).join(format!("{name}.ppm"));
    let mut file =
        std::io::BufWriter::new(std::fs::File::create(&path).expect("the output file must open"));
    write_ppm(&canvas, &mut file).expect("the pixels must write");
    println!(
        "{}: tick {}, {tile} pixels a tile over {over:?}, {} units drawn",
        path.display(),
        world.tick().0,
        canvas.soldiers_painted()
    );
}
