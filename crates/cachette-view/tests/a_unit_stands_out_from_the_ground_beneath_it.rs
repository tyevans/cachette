//! A unit reads against whatever lies under it.
//!
//! The product record asks that a watcher tell one faction from another and
//! read what is happening at a glance.[^1] A watcher must first find the
//! unit. A mark that carries one outline colour cannot do that, because a
//! background may be brighter than that colour or darker than it. The edge of
//! a holding draws nearly the pure colour of the faction that holds it, the
//! page behind the map is pale where a wash covers it, and open water is
//! dark.
//!
//! The viewer draws two outline bands around a unit. The halo is white and it
//! sits outside. The rim is nearly black and it sits inside the halo. These
//! tests state the bound that the pair must reach.
//!
//! # What the check measures
//!
//! The check draws the same frame twice, once with the units and once from a
//! copy of the same world that despawned them. The second picture is the
//! ground that the mark covers, read from the picture and never from a colour
//! table. A holding tint, a height shade, a resource speckle and a weather
//! wash all reach the ground colour, and no table holds the result of the
//! four together.
//!
//! The check then finds the mark, which is where the two pictures disagree,
//! and walks its boundary. At each boundary pixel it reads every pixel of the
//! mark around that one, which reaches both outline bands. One of them must
//! differ in brightness from the ground it covers by the bound below.
//!
//! **The check reads a boundary and not a radius.** A crowd draws at a radius
//! its count grows, and two crowds on neighbouring tiles draw one mark
//! between them. A check that took the radius of one bead would read the
//! faction colour of the neighbour and call it an outline.
//!
//! The measure is brightness and not hue. The two bands differ from each
//! other in brightness alone, so brightness is the axis the pair works on. A
//! faction colour carries the hue, and separate tests state that a bead keeps
//! it.[^2]
//!
//! # Why the bound holds for every background
//!
//! The halo is white and the rim is nearly black, so the two sit at the ends
//! of the brightness range. The distance from a background to the nearer end
//! is largest when the background sits midway between them. That largest
//! distance is above the bound below, so the bound holds for every possible
//! background and not only for the grounds one fixture draws. One test states
//! that over the whole range.
//!
//! # The fixture
//!
//! **A unit on plain grass is the typical case, and it decides nothing.** A
//! fixture that models the typical case supplies no extreme, so the assertion
//! never receives the input that would fail it.[^3] These fixtures supply the
//! cases that decide it: each kind of ground, ground that the unit's own
//! faction holds, water under the outline of a bead at the region zoom, the
//! page behind the map, and two crowds that touch.
//!
//! # References
//!
//! [^1]: PRD-0005, a watcher can tell what is happening and why. `docs/product/shipped/prd-0005-a-watcher-can-tell-what-is-happening-and-why.md`
//! [^2]: The unit reading tests. `crates/cachette-view/tests/a_unit_reads_at_every_zoom.rs`
//! [^3]: Testing Rules, a fixture supplies the input. `.agents/rules/testing.md`

// An integration test is its own crate, so the allowance at the viewer's
// crate root does not reach it. ADR-0067 D3 puts the float boundary at the
// viewer, and a tile width in pixels is a viewer value.
#![allow(clippy::disallowed_types)]

use cachette_core::terrain::TileKind;
use cachette_core::{Axial, Entity, FactionId, World, WorldConfig};
use cachette_view::paint::{unit_halo_colour, unit_radius, unit_rim_colour};
use cachette_view::{paint, Camera, Canvas};

/// The extent of the fixture world.
///
/// The extent is wider than the coarsest lattice spacing of the terrain
/// generator, so the world holds every kind of ground rather than one of
/// them.[^1]
///
/// # References
///
/// [^1]: Findings register, FND-054. `docs/FINDINGS.md`
const EXTENT: u32 = 64;

/// The seed of the fixture world.
const SEED: u64 = 7;

/// The factions the fixtures seat.
const FACTIONS: u16 = 2;

/// The size of every canvas these tests draw into.
const CANVAS: (usize, usize) = (420, 420);

/// The steps a fixture runs before it draws.
///
/// The holder column is rewritten from the cities on every step, so one step
/// is enough. The fixtures run more, so that the world they draw is a world
/// that has run.
const TICKS: u32 = 4;

/// The threads a fixture steps at.
const THREADS: usize = 2;

/// How far from the middle of a bead the check looks for the mark, as a
/// multiple of the tile.
///
/// The window holds the whole mark and a margin of ground around it. Its cost
/// follows the zoom and never the canvas.
const WINDOW_STEP: i32 = 3;

/// The four directions that make one pixel a neighbour of another.
const BESIDE: [(i32, i32); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];

/// The pixels around one pixel, and the pixel itself.
///
/// **The check reads the whole neighbourhood, and not the pixel behind the
/// boundary.** The bands of a bead run around a circle, so at the top of a
/// bead the pixel beside a halo pixel is another halo pixel and the rim lies
/// below it. A check that stepped against the outward direction would read
/// the halo twice there and would miss the band that answers.
const AROUND: [(i32, i32); 9] = [
    (0, 0),
    (1, 0),
    (-1, 0),
    (0, 1),
    (0, -1),
    (1, 1),
    (1, -1),
    (-1, 1),
    (-1, -1),
];

/// The smallest brightness distance the outline must reach.
///
/// Brightness runs from zero to two hundred and fifty five. The halo sits at
/// the top of that range and the rim sits near the bottom of it. A background
/// midway between the two is the furthest a background can be from both, and
/// it is still further than this from one of them.
const LEAST_DISTANCE: i32 = 120;

/// The fewest boundary pixels a case must offer the check.
///
/// A mark at the edge of the frame runs off the canvas, and the check skips a
/// boundary pixel whose ground is off the canvas. A case that offers fewer
/// than this proves nothing, so the check refuses it rather than passing on
/// one pixel.
const LEAST_BOUNDARY: usize = 8;

/// Returns the world settings the fixtures share.
const fn settings() -> WorldConfig {
    WorldConfig {
        width: EXTENT,
        height: EXTENT,
        seed: SEED,
        faction_count: FACTIONS,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    }
}

/// Returns every address of a world, in row order.
fn every_address(world: &World) -> Vec<Axial> {
    let grid = world.grid();
    (0..grid.tile_count())
        .map(|index| Axial::new((index % grid.width()) as i32, (index / grid.width()) as i32))
        .collect()
}

/// The world a case draws, and the units the case put in it.
///
/// The check needs both pictures from one world, so the fixture hands back
/// the units it spawned. The bare picture comes from a copy of this world
/// with those units removed, and every other value in it is the same value.
struct Case {
    /// The world the case draws.
    world: World,
    /// The units the case spawned, in the order it spawned them.
    units: Vec<Entity>,
}

impl Case {
    /// Builds a world with no unit in it yet.
    fn started() -> Self {
        let world = World::new(settings()).expect("the extent describes a world");
        Self {
            world,
            units: Vec::new(),
        }
    }

    /// Founds one city for each faction, near the middle of the world.
    ///
    /// A faction holds the ground within reach of a city it owns, and a unit
    /// standing on a tile gives its faction no claim on it. A fixture that
    /// took ground by standing units on it would be a second declaration of
    /// the holding rule.[^1]
    ///
    /// # References
    ///
    /// [^1]: Findings register, FND-487. `docs/FINDINGS.md`
    fn found_cities(&mut self) -> Vec<Axial> {
        let middle = (EXTENT / 2) as i32;
        let mut seats = Vec::new();
        for faction in 0..FACTIONS {
            let wanted = Axial::new(middle + i32::from(faction) * 6, middle);
            let seat = every_address(&self.world)
                .into_iter()
                .filter(|at| {
                    self.world.admits_a_unit(*at) && self.world.settlements().on_tile(*at).is_none()
                })
                .min_by_key(|at| at.distance(wanted))
                .expect("the world holds open ground");
            self.world
                .found_settlement(seat, FactionId(faction))
                .expect("the seat admits a city");
            seats.push(seat);
        }
        seats
    }

    /// Puts one unit of a faction on a tile, and remembers it.
    fn settle(&mut self, address: Axial, faction: FactionId) {
        if let Ok(unit) = self.world.spawn_soldier(address, faction) {
            self.units.push(unit);
        }
    }

    /// Steps the world, and rebuilds the structure the viewer reads.
    fn run(&mut self) {
        for _ in 0..TICKS {
            self.world.step(THREADS).expect("the step must run");
        }
        self.world
            .rebuild_bridge(THREADS)
            .expect("the bridge rebuilds");
    }

    /// Returns the same world with the units of this case removed.
    ///
    /// Every other value is the value the marked world holds, because this is
    /// a copy of that world. A second world built from the seed would hold a
    /// different tile stock, because the units of the first one ate and
    /// built.
    fn bare(&self) -> World {
        let mut bare = self.world.clone();
        for unit in &self.units {
            bare.despawn_soldier(*unit);
        }
        bare.rebuild_bridge(THREADS)
            .expect("the bridge rebuilds without the units");
        bare
    }
}

/// Returns the brightness of a packed colour.
///
/// The weights are the luminance weights of the display primaries, in
/// thousandths, so the arithmetic stays in whole numbers.
fn brightness(colour: u32) -> i32 {
    let red = ((colour >> 16) & 0xff) as i32;
    let green = ((colour >> 8) & 0xff) as i32;
    let blue = (colour & 0xff) as i32;
    (213 * red + 715 * green + 72 * blue) / 1000
}

/// Returns the colour a canvas holds at one pixel, when the pixel is on it.
fn pixel(canvas: &Canvas, x: i32, y: i32) -> Option<u32> {
    if x < 0 || y < 0 {
        return None;
    }
    let (x, y) = (x as usize, y as usize);
    if x >= canvas.width() || y >= canvas.height() {
        return None;
    }
    Some(canvas.pixels()[y * canvas.width() + x])
}

/// Draws one frame of a world.
fn shot(world: &World, camera: Camera) -> Canvas<'static> {
    let mut canvas = Canvas::new(CANVAS.0, CANVAS.1);
    paint::draw(world, camera, &mut canvas).expect("the world draws");
    canvas
}

/// The pixels of one mark, inside the window the check read them in.
///
/// The mark is where the two pictures disagree. The fill spreads from the
/// middle of the bead, so a badge above a crowd is a separate mark and stays
/// out of this one. Two crowds that touch make one mark, which is the object
/// a watcher sees at that density.
struct Mark {
    /// Whether each pixel of the window belongs to the mark.
    inside: Vec<bool>,
    /// The canvas pixel that the window starts at.
    origin: (i32, i32),
    /// How wide and how tall the window is, in pixels.
    size: (i32, i32),
}

impl Mark {
    /// Returns whether a canvas pixel belongs to the mark.
    fn holds(&self, x: i32, y: i32) -> bool {
        let (across, down) = (x - self.origin.0, y - self.origin.1);
        if across < 0 || down < 0 || across >= self.size.0 || down >= self.size.1 {
            return false;
        }
        self.inside[(down * self.size.0 + across) as usize]
    }
}

/// Finds the mark that covers the middle of a bead.
fn mark_at(marked: &Canvas, bare: &Canvas, centre: (i32, i32), reach: i32) -> Mark {
    let origin = (centre.0 - reach, centre.1 - reach);
    let size = (reach * 2 + 1, reach * 2 + 1);
    let differs = |x: i32, y: i32| match (pixel(marked, x, y), pixel(bare, x, y)) {
        (Some(over), Some(under)) => over != under,
        _ => false,
    };
    let index = |x: i32, y: i32| ((y - origin.1) * size.0 + (x - origin.0)) as usize;
    let mut inside = vec![false; (size.0 * size.1) as usize];
    let mut queue = Vec::new();
    if differs(centre.0, centre.1) {
        inside[index(centre.0, centre.1)] = true;
        queue.push(centre);
    }
    while let Some((x, y)) = queue.pop() {
        for (across, down) in BESIDE {
            let (nx, ny) = (x + across, y + down);
            if nx < origin.0 || ny < origin.1 || nx >= origin.0 + size.0 || ny >= origin.1 + size.1
            {
                continue;
            }
            if inside[index(nx, ny)] || !differs(nx, ny) {
                continue;
            }
            inside[index(nx, ny)] = true;
            queue.push((nx, ny));
        }
    }
    Mark {
        inside,
        origin,
        size,
    }
}

/// Checks that the mark over a unit contrasts the ground it covers.
///
/// The check walks the boundary of the mark. At each boundary pixel it reads
/// every pixel of the mark around that one, which reaches both outline bands,
/// and one of them must differ from the ground it covers by the bound. A mark
/// one pixel thick reaches one band, and that band answers alone.
fn outline_contrasts_the_ground(case: &Case, camera: Camera, place: Axial, what: &str) {
    let marked = shot(&case.world, camera);
    let bare = shot(&case.bare(), camera);
    assert!(
        marked.soldiers_painted() > 0,
        "{what}: the frame painted no unit, so the case supplies nothing",
    );
    assert_eq!(
        bare.soldiers_painted(),
        0,
        "{what}: the bare frame still paints a unit, so it is not the ground",
    );

    let (x, y) = camera.centre_of(place);
    let centre = (x as i32, y as i32);
    let reach = unit_radius(camera.tile_width).max(camera.tile_width as i32) * WINDOW_STEP;
    let mark = mark_at(&marked, &bare, centre, reach);
    let mut checked = 0;
    let mut worst = i32::MAX;
    for down in -reach..=reach {
        for across in -reach..=reach {
            let (px, py) = (centre.0 + across, centre.1 + down);
            if !mark.holds(px, py) {
                continue;
            }
            for (dx, dy) in BESIDE {
                let (ox, oy) = (px + dx, py + dy);
                if mark.holds(ox, oy) || pixel(&bare, ox, oy).is_none() {
                    continue;
                }
                let mut reached = 0;
                for (nx, ny) in AROUND {
                    let (ax, ay) = (px + nx, py + ny);
                    if !mark.holds(ax, ay) {
                        continue;
                    }
                    let Some(over) = pixel(&marked, ax, ay) else {
                        continue;
                    };
                    let Some(under) = pixel(&bare, ax, ay) else {
                        continue;
                    };
                    reached = reached.max((brightness(over) - brightness(under)).abs());
                }
                checked += 1;
                worst = worst.min(reached);
                assert!(
                    reached >= LEAST_DISTANCE,
                    "{what}: the outline of the mark over {place:?} reaches {reached} \
                     of {LEAST_DISTANCE} against the ground at {ox}, {oy}",
                );
            }
        }
    }
    assert!(
        checked >= LEAST_BOUNDARY,
        "{what}: the check read {checked} boundary pixels of the mark over \
         {place:?}, so it proves nothing",
    );
    println!("{what}: {checked} boundary pixels, worst distance {worst}");
}

/// Returns one tile that admits a unit, for each kind of ground.
fn one_tile_of_each_kind(world: &World) -> Vec<(TileKind, Axial)> {
    let mut found: Vec<(TileKind, Axial)> = Vec::new();
    for address in every_address(world) {
        if !world.admits_a_unit(address) {
            continue;
        }
        let Some(ground) = world.tile_terrain(address) else {
            continue;
        };
        if found.iter().any(|(kind, _)| *kind == ground.kind) {
            continue;
        }
        found.push((ground.kind, address));
    }
    found
}

/// Returns a camera over one tile, at one tile size.
fn over(place: Axial, tile: f32) -> Camera {
    Camera::at_tile_size(tile).looking_at(place, &Canvas::new(CANVAS.0, CANVAS.1))
}

#[test]
fn a_bead_outline_contrasts_every_kind_of_ground() {
    let mut case = Case::started();
    let kinds = one_tile_of_each_kind(&case.world);
    assert!(
        kinds.len() >= 3,
        "the fixture holds {} kinds of ground a unit stands on",
        kinds.len(),
    );
    for (_, address) in &kinds {
        case.settle(*address, FactionId(0));
    }
    case.run();

    for (kind, address) in &kinds {
        outline_contrasts_the_ground(&case, over(*address, 24.0), *address, &format!("{kind:?}"));
    }
}

#[test]
fn a_bead_outline_contrasts_the_ground_its_own_faction_holds() {
    // The hard case. The tint of a holding and the bead of a unit carry one
    // colour, so a faction colour cannot separate a unit from ground its own
    // faction holds. The edge of a holding is nearly the pure colour.
    let mut case = Case::started();
    let seats = case.found_cities();
    let seat = seats[0];
    let near: Vec<Axial> = every_address(&case.world)
        .into_iter()
        .filter(|at| at.distance(seat) <= 3 && case.world.admits_a_unit(*at))
        .collect();
    for address in &near {
        case.settle(*address, FactionId(0));
    }
    case.run();

    let mine: Vec<Axial> = near
        .iter()
        .copied()
        .filter(|at| case.world.holds(FactionId(0), *at) == Some(true))
        .collect();
    assert!(
        !mine.is_empty(),
        "no unit of the fixture stands on ground its own faction holds",
    );
    for address in mine.iter().take(6) {
        outline_contrasts_the_ground(&case, over(*address, 14.0), *address, "own holding");
    }
}

#[test]
fn a_bead_outline_contrasts_the_water_it_reaches_over() {
    // At the region zoom a bead is wider than a tile, so the outline of a
    // unit on the coast lands on the water beside it. Water is the darkest
    // ground the map draws, and the rim is dark, so the halo carries this
    // case.
    let mut case = Case::started();
    let coast = every_address(&case.world)
        .into_iter()
        .find(|at| {
            case.world.admits_a_unit(*at)
                && BESIDE.iter().any(|(dq, dr)| {
                    let beside = Axial::new(at.q + dq, at.r + dr);
                    case.world.tile_capacity(beside).is_some() && !case.world.admits_a_unit(beside)
                })
        })
        .expect("the fixture holds a tile beside open water");
    case.settle(coast, FactionId(0));
    case.run();

    for tile in [5.0f32, 8.0] {
        outline_contrasts_the_ground(&case, over(coast, tile), coast, "the coast");
    }
}

#[test]
fn a_bead_outline_contrasts_the_page_behind_the_map() {
    // The camera is not clamped, so the world does not fill the frame and a
    // unit at the corner of the map stands beside the page.
    let mut case = Case::started();
    let corner = every_address(&case.world)
        .into_iter()
        .filter(|at| case.world.admits_a_unit(*at))
        .min_by_key(|at| at.q + at.r)
        .expect("the fixture holds open ground");
    case.settle(corner, FactionId(0));
    case.run();

    outline_contrasts_the_ground(&case, over(corner, 12.0), corner, "the page");
}

#[test]
fn two_crowds_that_touch_keep_one_outline_against_the_ground() {
    // A crowd draws as one bead, and the bead grows with the count below the
    // badge width. Two crowds beside each other draw one mark, and that mark
    // must still carry the outline where it meets the ground. A mark that
    // works alone and turns to mush at density is not a mark.
    let mut case = Case::started();
    let place = every_address(&case.world)
        .into_iter()
        .find(|at| {
            case.world.admits_a_unit(*at)
                && case.world.admits_a_unit(Axial::new(at.q + 1, at.r))
                && case.world.tile_capacity(*at).is_some_and(|room| room >= 8)
        })
        .expect("the fixture holds a tile that admits a crowd");
    for index in 0..8u16 {
        case.settle(place, FactionId(index % FACTIONS));
        case.settle(
            Axial::new(place.q + 1, place.r),
            FactionId(index % FACTIONS),
        );
    }
    case.run();

    for tile in [10.0f32, 24.0] {
        outline_contrasts_the_ground(&case, over(place, tile), place, "two crowds");
    }
}

#[test]
fn a_bead_outline_contrasts_the_ground_at_every_zoom() {
    // The bands take two pixels of the radius at every zoom, and the radius
    // follows the tile. A test at one zoom proves nothing about another.
    let mut case = Case::started();
    let seats = case.found_cities();
    let place = every_address(&case.world)
        .into_iter()
        .filter(|at| case.world.admits_a_unit(*at))
        .min_by_key(|at| at.distance(seats[0]))
        .expect("the fixture holds open ground");
    case.settle(place, FactionId(0));
    case.run();

    for tile in [4.0f32, 6.0, 12.0, 24.0, 48.0] {
        outline_contrasts_the_ground(
            &case,
            over(place, tile),
            place,
            &format!("{tile} pixels a tile"),
        );
    }
}

#[test]
fn the_two_outline_bands_sit_at_the_ends_of_the_brightness_range() {
    // The bound of the checks above rests on this. A background midway
    // between the two bands is the furthest a background can be from both,
    // and it must still be further than the bound from one of them. A change
    // to either colour that broke the bound would otherwise be found only by
    // a case that happened to draw a ground of that brightness.
    let halo = brightness(unit_halo_colour());
    let rim = brightness(unit_rim_colour());
    assert!(
        halo > rim,
        "the halo at {halo} is not brighter than the rim at {rim}",
    );
    let worst = (0..=255)
        .map(|ground| (halo - ground).abs().min((rim - ground).abs()))
        .max()
        .expect("the range holds a value");
    assert!(
        worst >= LEAST_DISTANCE,
        "a background at the worst brightness reaches {worst} of {LEAST_DISTANCE}",
    );
}
