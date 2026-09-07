//! Where a road runs, as a way and not as a tile.
//!
//! A road is not a property of one tile. It runs from somewhere to somewhere,
//! it joins another road at a junction, it bends, and it ends. A tile that
//! carries a road therefore draws a ribbon through its middle and out through
//! the edge it shares with each road beside it, and it does not draw a filled
//! cell.
//!
//! **This module is the one derivation, and every renderer reads it.** The
//! engine renderer draws the ribbon with pixels. The sketchbook renderer draws
//! it on the processor and on the graphics device. A renderer that worked out
//! its own joins would be one fact in three places, and nothing would fail
//! when the three disagreed.[^1]
//!
//! **The read follows the roads and not the world.** The engine stores an
//! upgrade sparsely, so the whole road set is one slice however large the
//! world is.[^2] A join is one search of that slice.
//!
//! This module reads the world. It writes nothing to it.[^3]
//!
//! # References
//!
//! [^1]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
//! [^2]: ADR-0090, a tile upgrade is stored sparsely, as the difference from the generated world, decision D1. `docs/adrs/draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md`
//! [^3]: ADR-0067, the viewer reads the world and never writes to it, decision D3. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`

use cachette_core::hex::{Axial, NEIGHBOURS, NEIGHBOUR_COUNT};
use cachette_core::upgrade::{UpgradeCategory, UPGRADE_CATEGORY_COUNT};
use cachette_core::World;

/// One tile of a way, and the neighbours the way runs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Way {
    /// The tile the way runs over.
    pub address: Axial,
    /// The level that stands there.
    ///
    /// Zero means that the first level is still under construction, so
    /// nothing stands there yet and the ground is only marked out.
    pub level: u8,
    /// Which of the six neighbours carry a road, as one bit for each.
    ///
    /// Bit `i` is set when the neighbour in direction `i` carries a road. The
    /// directions are the engine's own, so a caller reads a direction by its
    /// index and this module states no order of its own.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    pub joins: u8,
}

impl Way {
    /// Returns how many neighbours the way runs to.
    #[must_use]
    pub const fn join_count(self) -> u32 {
        self.joins.count_ones()
    }

    /// Reports whether the way runs to the neighbour in one direction.
    #[must_use]
    pub const fn joins_direction(self, direction: usize) -> bool {
        direction < NEIGHBOUR_COUNT && (self.joins >> direction) & 1 == 1
    }

    /// Returns what the tile is: an end, a through-way, a junction or a mark.
    #[must_use]
    pub const fn shape(self) -> WayShape {
        match self.joins.count_ones() {
            0 => WayShape::Alone,
            1 => WayShape::End,
            2 => WayShape::Through,
            _ => WayShape::Junction,
        }
    }
}

/// What one tile of a way is, by how many neighbours it runs to.
///
/// The shape is a reading of the joins and not a second store of them. A
/// caller draws from the joins; this names what the joins say, so two
/// renderers describe one picture with one set of words.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WayShape {
    /// No neighbour carries a road. The tile draws a mark and no ribbon.
    Alone,
    /// One neighbour carries a road. The way ends here.
    End,
    /// Two neighbours carry a road. The way passes through and bends.
    Through,
    /// Three or more neighbours carry a road. The ways meet here.
    Junction,
}

/// Returns every road in the world, with the neighbours each one runs to.
///
/// The answer is in ascending tile order, because the engine holds the
/// upgrades in that order and this keeps it. The order is explicit and it
/// does not follow any completion order.[^1]
///
/// A world in which nobody built a road returns an empty vector. A tile whose
/// road is still under construction is in the answer at level zero, because a
/// way under construction is still a way and a watcher reads where it will
/// run.
///
/// # References
///
/// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
#[must_use]
pub fn road_ways(world: &World) -> Vec<Way> {
    let grid = world.grid();
    // The tiles that carry a road, in ascending tile order. The engine hands
    // the upgrades over in that order, so the filter keeps it and the search
    // below is a binary search.
    let carried: Vec<u32> = world
        .upgrade_sites()
        .iter()
        .filter(|site| site.category == UpgradeCategory::ROAD)
        .map(|site| site.tile.0)
        .collect();
    world
        .upgrade_sites()
        .iter()
        .filter(|site| site.category == UpgradeCategory::ROAD)
        .filter_map(|site| {
            let address = grid.address_of(site.tile)?;
            let mut joins = 0u8;
            for (direction, step) in NEIGHBOURS.iter().enumerate() {
                let beside = address.add(*step);
                let Some(index) = grid.index_of(beside) else {
                    continue;
                };
                if carried.binary_search(&index.0).is_ok() {
                    joins |= 1 << direction;
                }
            }
            Some(Way {
                address,
                level: site.level,
                joins,
            })
        })
        .collect()
}

/// Which upgrade categories draw as a way rather than as a thing on a tile.
///
/// **A way runs, and a thing stands.** A road runs from somewhere to
/// somewhere and joins another road, so it draws a ribbon through the ground
/// it crosses. A store, a wall or a lodging stands on one tile, so it draws a
/// glyph in the middle of that tile. The table is indexed by the number the
/// core gives each category, so a category that joins the core without a row
/// here fails to compile rather than taking a shape nobody chose.[^1]
///
/// # References
///
/// [^1]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
const DRAWS_AS_A_WAY: [bool; UPGRADE_CATEGORY_COUNT] = [
    // A road is a way.
    true,
    // A terrace, a wonder, a store, a wall, a lodging and the open category
    // all stand on the tile they are built on.
    false, false, false, false, false, false,
];

/// Reports whether a category draws as a way rather than as a thing on a tile.
///
/// A caller asks this rather than naming a category, so the set of ways is
/// declared once and every renderer and every test reads that one
/// declaration.[^1]
///
/// # References
///
/// [^1]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
#[must_use]
pub fn draws_as_a_way(category: UpgradeCategory) -> bool {
    DRAWS_AS_A_WAY[category.index()]
}
