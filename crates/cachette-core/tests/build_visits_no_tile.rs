//! The proof that building a world visits no tile of the value field.
//!
//! The product record for the ground states that building a world must not
//! cost a pass over every tile before the first frame.[^1] A pass is a
//! visit to each tile, and the visit is what this file counts.
//!
//! The count comes from a test-only switch, in the same way the
//! nondeterminism probe does.[^2] The switch makes the tile value field
//! count the tiles it generates on the calling thread. The whole file
//! compiles to nothing when the switch is off. Run it with the switch on:
//!
//! ```text
//! cargo test --package cachette-core --features census-generated-tiles \
//!     --test build_visits_no_tile -- --test-threads=1
//! ```
//!
//! **The value field contributes no visit, and one visitor remains.** The
//! build closes by filling the first level of the pyramid, and that fill
//! sums the value of every tile. So the build visits each tile once, and the
//! assertion says once rather than none. An item removes the remaining
//! visitor, and when it lands this test asserts none.[^3]
//!
//! The assertion still catches what it is for. Before this test existed the
//! build visited each tile twice: once to fill an eager column, and once for
//! the pyramid. A reinstated eager column doubles the count and fails here.
//!
//! **The counter must be shown to count, or an assertion on it proves
//! nothing.** A counter wired to nothing reads a constant for ever. Each
//! test below therefore reads the counter over a second call whose visit
//! count is known independently.
//!
//! The counter is one counter for the whole process, because a build starts
//! threads and a count held per thread would miss the work they do. Cargo
//! runs the tests of one binary on several threads, so this binary runs on
//! one thread. The recipe that runs it says so.
//!
//! # References
//!
//! [^1]: PRD-0003, a developer sees a world worth looking at, what it costs at the target scale. `docs/product/accepted/prd-0003-a-developer-sees-a-world-worth-looking-at.md`
//! [^2]: Testing rules, section 1. `.claude/rules/testing.md`
//! [^3]: Backlog item 0171. `docs/backlog/proposed/0171-build-the-first-level-without-a-pass-over-every-tile.md`
#![cfg(feature = "census-generated-tiles")]

use cachette_core::tile_value::census;
use cachette_core::{TileIdx, World, WorldConfig};

/// Builds a square world of the given edge.
fn world_of(edge: u32) -> World {
    World::new(WorldConfig {
        width: edge,
        height: edge,
        ..WorldConfig::default()
    })
    .expect("the extent must describe a world")
}

#[test]
fn building_a_world_visits_each_tile_once_and_the_value_field_adds_none() {
    for edge in [16u32, 64, 256] {
        census::reset();
        let world = world_of(edge);
        let visits = census::generated();
        let tiles = world.tile_count() as u64;
        assert_eq!(
            visits, tiles,
            "building a world of {tiles} tiles made {visits} visits, and the \
             one pass that remains makes {tiles}"
        );

        // The counter counts. A copy of the whole column visits every tile,
        // and the same counter reports every one of them. Without this, the
        // count above would also be what a counter wired to nothing reports.
        census::reset();
        let _ = world.copy_tile_values();
        assert_eq!(census::generated(), tiles);
    }
}

#[test]
fn reading_one_tile_visits_one_tile() {
    let world = world_of(64);
    census::reset();
    let _ = world.tile_value_at(TileIdx(7));
    assert_eq!(census::generated(), 1);
}
