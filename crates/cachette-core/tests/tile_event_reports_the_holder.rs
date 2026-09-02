//! The tile event reports who holds the tile.
//!
//! A tile carries one value that names a faction, and that value is the
//! holder. The holder names a faction, or nobody, and no rule derives it from
//! the tile index.[^1] The event log crosses to the control plane, so the
//! event must carry that value and no other.[^2]
//!
//! The tests drive the world step, because the step is what must stamp the
//! event. A test that stamped an event itself would prove that the stamp
//! works and not that anything reaches it.[^3]
//!
//! The fixture proves that it produced the three cases the tests need. It
//! proves it by reading the holders back after the step, and not by asserting
//! over the settings it passed in.[^4] A world that held no ground, or that
//! took no tile on the tick under test, would pass every assertion below
//! without exercising one of them.[^5]
//!
//! # References
//!
//! [^1]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D2. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
//! [^2]: Findings register, FND-079. `docs/FINDINGS.md`
//! [^3]: Testing rules, drive the real caller. `.claude/rules/testing.md`
//! [^4]: Findings register, FND-061. `docs/FINDINGS.md`
//! [^5]: Findings register, FND-051. `docs/FINDINGS.md`

use cachette_core::{Axial, FactionId, Holder, TileChanged, TileIdx, World, WorldConfig};

/// The extent of a world that holds ground of more than one kind.
///
/// The terrain generator lays its coarsest lattice over the world, and a
/// world narrower than that spacing holds one kind of ground everywhere. The
/// fixture below needs open ground for a holding to spread over, and it
/// asserts what it found rather than assuming it.[^1]
///
/// # References
///
/// [^1]: Findings register, FND-054. `docs/FINDINGS.md`
const VARIED: WorldConfig = WorldConfig {
    width: 96,
    height: 96,
    seed: 0x00c0_ffee_0123_4567,
    faction_count: 3,
};

/// The number of threads that the tick under test runs on.
///
/// The stamp reads one column and writes one event, so it runs after the
/// parallel work joins. The fixture still steps on more than one thread, so
/// that the pass under test sees the log the joined slots produced.
const THREADS: usize = 2;

/// What one step of the fixture produced.
struct Stepped {
    world: World,
    events: Vec<TileChanged>,
    before: Vec<Holder>,
    after: Vec<Holder>,
}

impl Stepped {
    /// Returns the holder of a tile before the tick under test.
    fn before(&self, tile: TileIdx) -> Holder {
        self.before[tile.0 as usize]
    }

    /// Returns the holder of a tile after the tick under test.
    fn after(&self, tile: TileIdx) -> Holder {
        self.after[tile.0 as usize]
    }

    /// Returns the events whose tile nobody held after the step.
    fn unheld(&self) -> Vec<TileChanged> {
        self.select(|holder, _| holder.is_nobody())
    }

    /// Returns the events whose tile a faction held after the step.
    fn held(&self) -> Vec<TileChanged> {
        self.select(|holder, _| !holder.is_nobody())
    }

    /// Returns the events whose tile took a different holder on this step.
    fn changed(&self) -> Vec<TileChanged> {
        self.select(|after, before| after != before)
    }

    /// Returns the events whose holders satisfy the test.
    fn select(&self, test: impl Fn(Holder, Holder) -> bool) -> Vec<TileChanged> {
        self.events
            .iter()
            .copied()
            .filter(|event| test(self.after(event.tile), self.before(event.tile)))
            .collect()
    }
}

/// Fills a patch of open ground with soldiers of one faction.
fn garrison(world: &mut World, faction: FactionId, first: Axial, edge: i32) {
    for row in 0..edge {
        for column in 0..edge {
            let address = Axial::new(first.q + column, first.r + row);
            if !world.admits_a_unit(address) {
                continue;
            }
            let _ = world.spawn_soldier(address, faction);
        }
    }
}

/// Runs a world up to the tick under test, then steps it once more.
///
/// The world runs several frames first, so that a holding exists and is still
/// growing when the tick under test starts. A holding that had settled would
/// change no tile on that tick, and the test of a changed holder would then
/// have nothing to read.
fn step_once() -> Stepped {
    let mut world = World::new(VARIED).expect("the extent must describe a world");
    // A unit takes an intent at the interval its cell schedules, and it does
    // not move before it has one. This file is not about the interval, so
    // every unit chooses on every tick.
    world
        .set_choice_schedule(0)
        .expect("the exponent is inside the range");
    garrison(&mut world, FactionId(0), Axial::new(4, 4), 8);
    garrison(&mut world, FactionId(1), Axial::new(70, 70), 8);
    for _ in 0..4 {
        world.step(THREADS).expect("the step must run");
    }

    let before = world.holding().holders().to_vec();
    let events = world.step(THREADS).expect("the step must run").to_vec();
    let after = world.holding().holders().to_vec();
    assert!(world.check_invariants(), "the world broke an invariant");
    assert!(!events.is_empty(), "the step must report a change");
    Stepped {
        world,
        events,
        before,
        after,
    }
}

#[test]
fn the_fixture_reaches_a_held_tile_an_unheld_tile_and_a_tile_that_changed() {
    let stepped = step_once();
    // Each count comes from the holder column read back after the step. None
    // of them comes from a setting the fixture passed in.
    assert!(
        !stepped.unheld().is_empty(),
        "the fixture must report a tile that nobody holds"
    );
    assert!(
        !stepped.held().is_empty(),
        "the fixture must report a tile that a faction holds"
    );
    assert!(
        !stepped.changed().is_empty(),
        "the fixture must report a tile that took a new holder on this tick"
    );
}

#[test]
fn every_event_reports_the_holder_that_the_world_reports() {
    let stepped = step_once();
    let grid = stepped.world.grid();
    for event in &stepped.events {
        let address = grid
            .address_of(event.tile)
            .expect("the event names a tile of this world");
        let holder = stepped
            .world
            .tile_holder(address)
            .expect("the address is inside the world");
        assert_eq!(
            event.holder, holder,
            "the event for tile {} must report the holder of that tile",
            event.tile.0
        );
    }
}

#[test]
fn an_event_for_an_unheld_tile_names_nobody() {
    let stepped = step_once();
    let unheld = stepped.unheld();
    assert!(!unheld.is_empty(), "the fixture must reach an unheld tile");
    let ceiling = VARIED.faction_count;
    for event in &unheld {
        assert!(
            event.holder.is_nobody(),
            "the event for unheld tile {} must name nobody",
            event.tile.0
        );
        assert_eq!(event.holder.faction(), None);
        // The value that names nobody is no faction of this world. It sits
        // above the faction ceiling, so nothing can read it as a faction.
        for index in 0..ceiling {
            assert_ne!(
                event.holder,
                Holder::of(FactionId(index)),
                "the value for nobody must name no faction the world holds"
            );
        }
    }
}

#[test]
fn an_event_for_a_held_tile_names_the_faction_that_holds_it() {
    let stepped = step_once();
    let held = stepped.held();
    assert!(!held.is_empty(), "the fixture must reach a held tile");
    for event in &held {
        let faction = event
            .holder
            .faction()
            .expect("the event for a held tile must name a faction");
        assert!(faction.0 < VARIED.faction_count);
        assert_eq!(event.holder, stepped.after(event.tile));
    }
}

#[test]
fn an_event_for_a_tile_that_changed_holder_reports_the_new_holder() {
    let stepped = step_once();
    let changed = stepped.changed();
    assert!(
        !changed.is_empty(),
        "the fixture must reach a tile that changed holder on this tick"
    );
    for event in &changed {
        assert_eq!(
            event.holder,
            stepped.after(event.tile),
            "the event for tile {} must report the holder this tick left",
            event.tile.0
        );
        assert_ne!(
            event.holder,
            stepped.before(event.tile),
            "the event for tile {} must not report the holder of the frame before",
            event.tile.0
        );
    }
}
