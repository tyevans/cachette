//! What one faction may read, and nothing more.
//!
//! Every reader in this module takes a faction and answers for that faction.
//! **No argument asks for the truth.** The engine applies the sight rule
//! inside the reader, so a caller cannot ask past the fog.[^1] [^2]
//!
//! # The three answers
//!
//! A faction reads a tile it sees now and receives the present ground, the
//! present holder, the present upgrade and the units that stand there.
//!
//! A faction reads a tile it saw once and does not see now. The reader
//! answers with the ground of that place and with no unit. It reports no
//! holder and no upgrade, because both are facts of the present frame.[^2]
//!
//! A faction reads a tile it has never seen. The reader answers nothing at
//! all, and the caller tells that answer from a tile that holds
//! nothing.[^2]
//!
//! # What a remembered place cannot answer yet
//!
//! The remembered layer stores membership. It records that a faction saw a
//! tile, and it stores no value.[^2] The ground of a tile is a pure function
//! of the seed and the address, so a reader recovers the height, the kind
//! and the food a tile started with without a stored snapshot.[^3] [^4]
//!
//! **An upgrade is not recoverable.** An upgrade is the difference between
//! the generated world and the built world, so a faction that saw a plain,
//! marched away, and whose rival then built there has no stored answer for
//! what it last saw.[^5] This module answers no upgrade for a remembered
//! place, which understates the memory and never overstates it. The
//! decisions register holds the question.[^6]
//!
//! # The summary
//!
//! A summary reader combines only the tiles the same rule admits, so a cell
//! cannot state what its tiles hide.[^2] The fog layer and the summary level
//! divide the world over one lattice, so the tiles of one cell are the tiles
//! of one fog block.[^7]
//!
//! The reader states how many tiles of the cell it admitted and how many it
//! withheld, so a caller never mistakes a masked total for a whole one.
//!
//! # References
//!
//! [^1]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decision D3. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
//! [^2]: ADR-0059, fog storage grows with observed area, not with world area, decisions D2, D4 and D6. `docs/adrs/accepted/adr-0059-fog-storage-grows-with-observed-area.md`
//! [^3]: ADR-0068, terrain is generated from the seed and is never stored as a map, decision D1. `docs/adrs/accepted/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
//! [^4]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D1. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
//! [^5]: ADR-0090, a tile upgrade is stored sparsely, as the difference from the generated world, decision D1. `docs/adrs/draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md`
//! [^6]: Decisions register, DEC-276. `docs/DECISIONS.md`
//! [^7]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D2. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`

use crate::bridge::BridgeError;
use crate::hex::Axial;
use crate::holding::Holder;
use crate::observation::BlockForm;
use crate::pyramid::CellSummary;
use crate::resource::{Amount, ResourceKind, RESOURCE_KIND_COUNT};
use crate::sim_math;
use crate::terrain::TileKind;
use crate::types::{Accum, Entity, FactionId, Fix32, TileIdx};
use crate::upgrade::UpgradeSite;
use crate::world::World;

/// The reason that a faction reader could not answer.
///
/// **A refusal names its cause.** A reader that answered nothing at all left
/// the caller with the fact of a refusal and none of the reason, and the
/// reason is the whole of the diagnosis: a derived structure that the arena
/// has moved past says which revision it holds and which revision the arena
/// holds.[^1] [^2]
///
/// # References
///
/// [^1]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D3. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
/// [^2]: Findings register, FND-647. `docs/FINDINGS.md`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FactionViewError {
    /// The number names no faction of this world.
    NoSuchFaction(FactionId),
    /// The derived unit structure refused, and the bridge says why.
    Bridge(BridgeError),
    /// The world holds no tile at an address that a reader reached.
    NoTile(Axial),
}

impl core::fmt::Display for FactionViewError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NoSuchFaction(faction) => {
                write!(formatter, "{} names no faction of this world", faction.0)
            }
            Self::Bridge(error) => write!(
                formatter,
                "the world cannot describe its own units: {error}"
            ),
            Self::NoTile(address) => write!(
                formatter,
                "the world holds no tile at ({}, {})",
                address.q, address.r
            ),
        }
    }
}

impl std::error::Error for FactionViewError {}

impl From<BridgeError> for FactionViewError {
    fn from(error: BridgeError) -> Self {
        Self::Bridge(error)
    }
}

/// How a faction came by what it reads.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sighting {
    /// The faction has never seen the place.
    Never,
    /// The faction saw the place once and does not see it now.
    Remembered,
    /// The faction sees the place now.
    Seen,
}

/// The ground of one tile, as the generated world holds it.
///
/// Every field here is a pure function of the seed and the address, so a
/// faction that saw the tile once reads the same values a faction that sees
/// it now reads.[^1] [^2]
///
/// # References
///
/// [^1]: ADR-0068, terrain is generated from the seed and is never stored as a map, decision D1. `docs/adrs/accepted/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
/// [^2]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D1. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ground {
    /// The kind of the tile.
    pub kind: TileKind,
    /// The height, as a fraction of the full range.
    pub height: Fix32,
    /// What each resource kind started with, before anybody gathered.
    pub generated: [Amount; RESOURCE_KIND_COUNT],
}

/// What a faction reads about a tile it sees now.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SeenTile {
    /// The ground of the tile.
    pub ground: Ground,
    /// The value of the tile now.
    pub value: Fix32,
    /// How many units the tile admits.
    pub capacity: u32,
    /// What each resource kind holds now.
    pub stock: [Amount; RESOURCE_KIND_COUNT],
    /// Who holds the tile now, or `None` when nobody holds it.
    pub holder: Option<FactionId>,
    /// The upgrade that stands or grows there now.
    pub upgrade: Option<UpgradeSite>,
    /// How many units stand on the tile now.
    pub units: u32,
}

/// What one faction may read about one tile.
///
/// A caller tells the three answers apart, so a place the faction has never
/// seen is never confused with a place that holds nothing.[^1]
///
/// # References
///
/// [^1]: ADR-0059, fog storage grows with observed area, not with world area, decision D4. `docs/adrs/accepted/adr-0059-fog-storage-grows-with-observed-area.md`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FactionTile {
    /// The faction has never seen this tile. It reads nothing.
    Never,
    /// The faction saw the tile and does not see it now. It reads the
    /// ground, and it reads no unit, no holder and no upgrade.
    Remembered(Ground),
    /// The faction sees the tile now. It reads the present frame.
    Seen(SeenTile),
}

impl FactionTile {
    /// Returns how the faction came by this answer.
    #[must_use]
    pub const fn sighting(&self) -> Sighting {
        match self {
            Self::Never => Sighting::Never,
            Self::Remembered(_) => Sighting::Remembered,
            Self::Seen(_) => Sighting::Seen,
        }
    }

    /// Returns the ground of the tile, or `None` when the faction has never
    /// seen it.
    #[must_use]
    pub const fn ground(&self) -> Option<Ground> {
        match self {
            Self::Never => None,
            Self::Remembered(ground) => Some(*ground),
            Self::Seen(seen) => Some(seen.ground),
        }
    }

    /// Returns how many units the faction reads on the tile.
    ///
    /// A remembered place reports no unit, and a place the faction has never
    /// seen reports no unit either.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0059, fog storage grows with observed area, not with world area, decision D4. `docs/adrs/accepted/adr-0059-fog-storage-grows-with-observed-area.md`
    #[must_use]
    pub const fn units(&self) -> u32 {
        match self {
            Self::Never | Self::Remembered(_) => 0,
            Self::Seen(seen) => seen.units,
        }
    }

    /// Returns who the faction reads as the holder of the tile.
    #[must_use]
    pub const fn holder(&self) -> Option<FactionId> {
        match self {
            Self::Never | Self::Remembered(_) => None,
            Self::Seen(seen) => seen.holder,
        }
    }
}

/// One unit that a faction sees, with its tile and its faction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SeenUnit {
    /// The identity of the unit.
    pub unit: Entity,
    /// The tile the unit stands on.
    pub tile: TileIdx,
    /// The faction the unit belongs to.
    pub faction: FactionId,
}

/// Which tiles a summary admits.
///
/// A per-faction count names which of the two it counted, and it never
/// counts a subject the faction has not observed.[^1]
///
/// **Neither value widens the answer past the fog.** One counts what the
/// faction sees this frame, and the other counts what it has ever seen.
///
/// # References
///
/// [^1]: ADR-0059, fog storage grows with observed area, not with world area, decision D6. `docs/adrs/accepted/adr-0059-fog-storage-grows-with-observed-area.md`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Admit {
    /// Admit a tile the faction sees this frame.
    SeenNow,
    /// Admit a tile the faction has ever seen.
    SeenEver,
}

/// The two block forms of one faction over one cell of the lattice.
///
/// A reader that walks many cells reads the layer of the faction once, and
/// takes one form for each cell. A reader that looks the form up inside a
/// loop over the tiles repeats the lookup for every tile of the cell.
///
/// **The mask answers for a whole block when the faction saw no tile of
/// it.** A cell outside everything the faction ever walked therefore costs
/// no tile work at all, so the cost of a whole-lattice read follows the
/// observed area and not the world.[^1]
///
/// # References
///
/// [^1]: ADR-0059, fog storage grows with observed area, not with world area, decision D2. `docs/adrs/accepted/adr-0059-fog-storage-grows-with-observed-area.md`
pub(crate) struct BlockMask<'a> {
    visible: Option<&'a BlockForm>,
    remembered: Option<&'a BlockForm>,
}

impl<'a> BlockMask<'a> {
    /// Reads the two forms of one faction over one block.
    pub(crate) fn of(world: &'a World, faction: FactionId, block: u32) -> Self {
        Self::new(
            world
                .observation()
                .visible_layer(faction)
                .and_then(|layer| layer.block(block)),
            world
                .observation()
                .remembered_layer(faction)
                .and_then(|layer| layer.block(block)),
        )
    }

    /// Builds a mask from two forms that the caller already holds.
    pub(crate) const fn new(
        visible: Option<&'a BlockForm>,
        remembered: Option<&'a BlockForm>,
    ) -> Self {
        Self {
            visible,
            remembered,
        }
    }

    /// Reports whether the faction sees no tile of the block and saw none.
    pub(crate) fn is_empty(&self) -> bool {
        Self::vacant(self.visible) && Self::vacant(self.remembered)
    }

    /// Reports whether the faction sees the tile at one offset now.
    pub(crate) fn sees_now(&self, offset: u32) -> bool {
        self.visible.is_some_and(|form| form.holds(offset))
    }

    /// Reports whether the faction saw the tile at one offset once.
    pub(crate) fn saw_once(&self, offset: u32) -> bool {
        self.remembered.is_some_and(|form| form.holds(offset))
    }

    /// Reports whether a form names no tile at all.
    fn vacant(form: Option<&BlockForm>) -> bool {
        matches!(form, None | Some(BlockForm::None))
    }
}

/// The four counts that one cell contributes relative to one faction.
///
/// **Every count here is relative to the faction that reads, and none of
/// them is indexed by a faction.** A plane with one entry for each faction
/// multiplies the world by the faction count, and the record refuses
/// one.[^1] Two counts therefore stand for every rival together.
///
/// # References
///
/// [^1]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D3. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
#[derive(Clone, Copy, Debug, Default)]
struct FactionSplit {
    own_units: i64,
    other_units: i64,
    own_held_tiles: i64,
    other_held_tiles: i64,
}

impl FactionSplit {
    /// Adds what one tile contributes, for a tile the faction sees now.
    ///
    /// The unit walk reads the derived unit structure for the tile and the
    /// faction column of each identity on it. It is bounded by the units
    /// that stand on the observed tiles, and never by the population of the
    /// world.
    ///
    /// The walk visits the units in the order the structure holds them,
    /// which is a property of storage. Nothing here reads a thread.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    fn take(&mut self, world: &World, faction: FactionId, address: Axial, tile: TileIdx) {
        for unit in world.bridge().on_tile_unguarded(tile) {
            match world.soldiers().faction(*unit) {
                Some(owner) if owner == faction => self.own_units += 1,
                Some(_) => self.other_units += 1,
                None => {}
            }
        }
        match world.tile_holder(address).and_then(Holder::faction) {
            Some(holder) if holder == faction => self.own_held_tiles += 1,
            Some(_) => self.other_held_tiles += 1,
            None => {}
        }
    }
}

/// The summary of one cell, over the tiles one faction may read.
///
/// The summary field holds the same fields the whole-world summary holds, so
/// a caller reads one type at both boundaries.[^1] It counts only the tiles
/// the rule admitted.
///
/// A tile the faction saw once and does not see now contributes the ground
/// alone. It adds no unit, no held tile, no value and no gathering, because
/// each of those is a fact of the present frame that the faction cannot see.
///
/// # References
///
/// [^1]: ADR-0024, every summary field is declared extensive or intensive, decision D2. `docs/adrs/accepted/adr-0024-every-summary-field-is-declared-extensive-or-intensive.md`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MaskedSummary {
    admit: Admit,
    admitted: i64,
    withheld: i64,
    summary: CellSummary,
    own_units: i64,
    other_units: i64,
    own_held_tiles: i64,
    other_held_tiles: i64,
}

impl MaskedSummary {
    /// Returns which rule admitted the tiles.
    #[must_use]
    pub const fn admit(self) -> Admit {
        self.admit
    }

    /// Returns how many tiles of the cell the rule admitted.
    #[must_use]
    pub const fn admitted(self) -> i64 {
        self.admitted
    }

    /// Returns how many tiles of the cell the rule withheld.
    ///
    /// A caller that reads a zero here holds the whole cell. A caller that
    /// reads anything above zero holds a part of it, and the summary states
    /// no value for the rest.
    #[must_use]
    pub const fn withheld(self) -> i64 {
        self.withheld
    }

    /// Returns the combined summary of the admitted tiles.
    #[must_use]
    pub const fn summary(self) -> CellSummary {
        self.summary
    }

    /// Returns the units of the reading faction that stand on the admitted
    /// tiles it sees now.
    ///
    /// **The count is relative to the faction that read, and it is not one
    /// count for each faction.** A field indexed by the faction multiplies
    /// the world by the faction count, and the record refuses one.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D3. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
    #[must_use]
    pub const fn own_units(self) -> i64 {
        self.own_units
    }

    /// Returns the units of every other faction that stand on the admitted
    /// tiles the reading faction sees now.
    ///
    /// An ally and an invader both count here. The relation that separates
    /// the two is a quantity of its own, and it is not a property of a
    /// tile.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0146, a faction relation is one signed integer per ordered pair, and a pass reads a threshold, decision D1. `docs/adrs/accepted/adr-0146-a-faction-relation-is-one-signed-integer-per-ordered-pair-and-a-pass-reads-a-threshold.md`
    #[must_use]
    pub const fn other_units(self) -> i64 {
        self.other_units
    }

    /// Returns the admitted tiles the reading faction holds, over the tiles
    /// it sees now.
    #[must_use]
    pub const fn own_held_tiles(self) -> i64 {
        self.own_held_tiles
    }

    /// Returns the admitted tiles another faction holds, over the tiles the
    /// reading faction sees now.
    ///
    /// A tile that nobody holds counts in neither this nor the own count, so
    /// the two do not sum to the admitted tile count.
    #[must_use]
    pub const fn other_held_tiles(self) -> i64 {
        self.other_held_tiles
    }
}

impl World {
    /// Returns what one faction may read about one tile.
    ///
    /// The reader resolves the address against the two layers of the faction
    /// before it answers. It answers the present frame for a place the
    /// faction sees now, the ground alone for a place it saw once, and
    /// nothing for a place it has never seen.[^1]
    ///
    /// **No argument widens the answer.** A caller that wants the truth of
    /// the world calls a reader that names no faction.[^2]
    ///
    /// Returns `None` when the address lies outside the world.
    ///
    /// # References
    ///
    /// [^1]: ADR-0059, fog storage grows with observed area, not with world area, decision D4. `docs/adrs/accepted/adr-0059-fog-storage-grows-with-observed-area.md`
    /// [^2]: ADR-0059, fog storage grows with observed area, not with world area, decision D6. `docs/adrs/accepted/adr-0059-fog-storage-grows-with-observed-area.md`
    #[must_use]
    pub fn faction_tile(&self, faction: FactionId, address: Axial) -> Option<FactionTile> {
        let index = self.grid().index_of(address)?;
        let ground = self.ground_of(address)?;
        if self.observation().sees_now(faction, index) {
            return Some(FactionTile::Seen(self.seen_tile(address, ground)?));
        }
        if self.observation().has_seen(faction, index) {
            return Some(FactionTile::Remembered(ground));
        }
        Some(FactionTile::Never)
    }

    /// Returns the summary of the cell that covers one tile, over the tiles
    /// one faction may read.
    ///
    /// The cell is the cell the summary level holds, and the tiles of it are
    /// the tiles of one fog block, because the two share one lattice.[^1]
    ///
    /// The reader combines only the tiles the rule admits, so a cell cannot
    /// state what its tiles hide.[^2]
    ///
    /// # Errors
    ///
    /// Returns an error when the address lies outside the world, and when the
    /// derived unit structure does not describe the units. The error names
    /// which of the two happened.
    ///
    /// # References
    ///
    /// [^1]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D2. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
    /// [^2]: ADR-0059, fog storage grows with observed area, not with world area, decision D4. `docs/adrs/accepted/adr-0059-fog-storage-grows-with-observed-area.md`
    pub fn faction_summary_covering(
        &self,
        faction: FactionId,
        address: Axial,
        admit: Admit,
    ) -> Result<MaskedSummary, FactionViewError> {
        let layout = self.observation().layout();
        let tile = self
            .grid()
            .index_of(address)
            .ok_or(FactionViewError::NoTile(address))?;
        let key = layout
            .key_of(tile)
            .ok_or(FactionViewError::NoTile(address))?;
        let block = layout.block_of_key(key);
        self.masked_block(&BlockMask::of(self, faction, block), faction, block, admit)
    }

    /// Returns the summary of one cell of the lattice, over the tiles one
    /// faction may read.
    ///
    /// **This is the one place that applies the sight rule to a cell.** A
    /// caller that reads one cell and a caller that reads the whole lattice
    /// both come here, so the two cannot disagree about what a faction may
    /// read.[^1]
    ///
    /// The caller supplies the block forms, so a caller that walks many
    /// cells reads the layer of the faction once rather than once for each
    /// cell.
    ///
    /// A block the faction has never seen a tile of costs no tile work at
    /// all. The mask answers for the whole block, and the reader returns the
    /// identity.
    ///
    /// # Errors
    ///
    /// Returns an error when the derived unit structure does not describe the
    /// units, and when the world holds no tile at an address of the block.
    /// The error names which of the two happened, and a stale structure
    /// names both revisions.
    ///
    /// # References
    ///
    /// [^1]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
    pub(crate) fn masked_block(
        &self,
        mask: &BlockMask<'_>,
        faction: FactionId,
        block: u32,
        admit: Admit,
    ) -> Result<MaskedSummary, FactionViewError> {
        let layout = self.observation().layout();
        let grid = self.grid();
        let tiles = i64::from(self.observation().tiles_in_block(block));

        // A block the faction has never seen a tile of answers from the mask
        // alone. This is the whole of the map for a faction that has walked
        // a corner of it, so the walk below runs over the observed area and
        // never over the world.
        if mask.is_empty() {
            return Ok(MaskedSummary {
                admit,
                admitted: 0,
                withheld: tiles,
                summary: CellSummary::IDENTITY,
                own_units: 0,
                other_units: 0,
                own_held_tiles: 0,
                other_held_tiles: 0,
            });
        }

        let edge = layout.block_edge();
        let first_column = (block % layout.blocks_wide()) * edge;
        let first_row = (block / layout.blocks_wide()) * edge;
        let last_column = (first_column + edge).min(grid.width());
        let last_row = (first_row + edge).min(grid.height());

        let mut summary = CellSummary::IDENTITY;
        let mut admitted = 0i64;
        let mut withheld = 0i64;
        // **The four faction-relative counts are accumulated in this walk
        // and not in a second one.** This is the one place that applies the
        // sight rule to a cell, so a count taken anywhere else could state
        // that a faction sees ground it does not.
        let mut split = FactionSplit::default();
        for row in first_row..last_row {
            for column in first_column..last_column {
                let here = Axial::new(column as i32, row as i32);
                let Some(tile) = grid.index_of(here) else {
                    continue;
                };
                let Some(key) = layout.key_of(tile) else {
                    continue;
                };
                let offset = layout.offset_of_key(key);
                let sees_now = mask.sees_now(offset);
                let seen_ever = sees_now || mask.saw_once(offset);
                let reads = match admit {
                    Admit::SeenNow => sees_now,
                    Admit::SeenEver => seen_ever,
                };
                if !reads {
                    withheld += 1;
                    continue;
                }
                admitted += 1;
                summary = summary.combine(self.tile_summary(here, tile, sees_now)?);
                // A tile the faction only remembers contributes no unit and
                // no holder, in the way the summary of it contributes none.
                // Both are facts of the present frame.
                if sees_now {
                    split.take(self, faction, here, tile);
                }
            }
        }
        Ok(MaskedSummary {
            admit,
            admitted,
            withheld,
            summary,
            own_units: split.own_units,
            other_units: split.other_units,
            own_held_tiles: split.own_held_tiles,
            other_held_tiles: split.other_held_tiles,
        })
    }

    /// Returns every unit that one faction sees now, with its tile and its
    /// faction.
    ///
    /// A faction sees a unit when it sees the tile the unit stands on. It
    /// therefore sees its own units on the ground its own units watch, and it
    /// sees a rival that walks into that ground.[^1]
    ///
    /// **The walk runs over the factions in faction order, and over the units
    /// of each faction in slot order.** Both orders are fixed, so two runs
    /// return one answer. Nothing reads a thread.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0059, fog storage grows with observed area, not with world area, decision D4. `docs/adrs/accepted/adr-0059-fog-storage-grows-with-observed-area.md`
    /// [^2]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    #[must_use]
    pub fn units_seen_by(&self, faction: FactionId) -> Vec<SeenUnit> {
        let arena = self.soldiers();
        let mut seen = Vec::new();
        for number in 0..self.faction_count() {
            let owner = FactionId(number);
            for unit in arena.iter_faction(owner) {
                let Some(tile) = arena.tile(unit) else {
                    continue;
                };
                if self.observation().sees_now(faction, tile) {
                    seen.push(SeenUnit {
                        unit,
                        tile,
                        faction: owner,
                    });
                }
            }
        }
        seen
    }

    /// Returns the generated ground of one tile.
    fn ground_of(&self, address: Axial) -> Option<Ground> {
        let tile = self.terrain().tile(address)?;
        let mut generated = [Amount::ZERO; RESOURCE_KIND_COUNT];
        for kind in ResourceKind::ALL {
            generated[kind.to_u8() as usize] = self.resources().original(address, kind)?;
        }
        Some(Ground {
            kind: tile.kind,
            height: tile.height,
            generated,
        })
    }

    /// Returns what a faction that watches the tile now reads about it.
    fn seen_tile(&self, address: Axial, ground: Ground) -> Option<SeenTile> {
        let mut stock = [Amount::ZERO; RESOURCE_KIND_COUNT];
        for kind in ResourceKind::ALL {
            stock[kind.to_u8() as usize] = self.tile_stock(address, kind)?;
        }
        let units = self
            .bridge()
            .count_on_tile(self.soldiers(), address)
            .unwrap_or(0);
        Some(SeenTile {
            ground,
            value: self.tile_value(address)?,
            capacity: self.tile_capacity(address)?,
            stock,
            holder: self.tile_holder(address).and_then(Holder::faction),
            upgrade: self.upgrade_at(address),
            units: u32::try_from(units).unwrap_or(u32::MAX),
        })
    }

    /// Returns the contribution of one tile to a masked summary.
    ///
    /// A tile the faction sees now contributes every field. A tile it only
    /// remembers contributes the ground alone, because the units, the
    /// holders, the values and the gathering are facts of the present
    /// frame.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0059, fog storage grows with observed area, not with world area, decision D4. `docs/adrs/accepted/adr-0059-fog-storage-grows-with-observed-area.md`
    fn tile_summary(
        &self,
        address: Axial,
        tile: TileIdx,
        sees_now: bool,
    ) -> Result<CellSummary, FactionViewError> {
        let ground = self
            .terrain()
            .tile(address)
            .ok_or(FactionViewError::NoTile(address))?;
        let food = self
            .resources()
            .original(address, ResourceKind::Food)
            .ok_or(FactionViewError::NoTile(address))?;
        let mut summary = CellSummary::of_ground(ground.kind.is_passable(), ground.height, food);
        if !sees_now {
            return Ok(summary);
        }
        let taken = self
            .taken_from(address, ResourceKind::Food)
            .unwrap_or(Amount::ZERO);
        // **The refusal carries the reason.** The bridge names a stale
        // revision pair, a bridge that was never built, and a bridge built
        // from another arena. A reader that dropped that reason left the
        // caller a sentence that named none of it.[^2]
        //
        // [^2]: Findings register, FND-647. `docs/FINDINGS.md`
        let units = self.bridge().count_on_tile(self.soldiers(), address)?;
        let held = i64::from(
            self.tile_holder(address)
                .is_some_and(|holder| !holder.is_nobody()),
        );
        let value = self
            .tile_value_at(tile)
            .ok_or(FactionViewError::NoTile(address))?;
        summary = summary.combine(CellSummary::of_frame(
            units as i64,
            held,
            sim_math::accumulate(Accum(0), value),
            taken.to_accum().0,
        ));
        Ok(summary)
    }
}
