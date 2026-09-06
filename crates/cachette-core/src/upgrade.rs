//! Tile upgrades, and the table a category indexes.
//!
//! An upgrade is the mark a unit leaves on a tile. The generator made the
//! ground and the stock of the world, and neither of those records anything a
//! unit did.[^1] [^2] An upgrade is the opposite: no function of the seed can
//! produce it, so the engine stores it.
//!
//! **The store holds one entry for each tile that carries an upgrade, and
//! nothing else.** A world in which nobody built holds no entry, so the memory
//! cost follows the building and not the size of the world.[^3]
//!
//! **An upgrade is a row of a table that the world is built with.** A row is
//! one category at one level. It names the ground it fits, the work it takes
//! and the columns a pass reads. A category is an index and never a name, and
//! no pass branches on one.[^7] The table takes the form the unit type table
//! takes: one macro declares the row, and the column names and the column
//! reader derive from that declaration.[^8]
//!
//! An upgrade under construction holds a progress accumulator. Several units
//! add to it in one tick and the contributions combine exactly, because every
//! term is a whole number and the accumulator is 64 bits wide.[^4] [^5] The
//! accumulator is clamped at the work of the row above the entry. An unclamped
//! accumulator lets a builder bank surplus it can never spend, and that
//! overflow reaches the state hash.[^6]
//!
//! **A finished upgrade holds a condition, and the condition is the life of
//! it.** A level that has just been finished stands at the full condition.
//! The weather over its tile and a hostile unit on its tile take from it, a
//! worker on the tile puts it back at the price the build cost, and a site
//! that reaches nothing is removed. The tile then returns to the world the
//! generator made.
//!
//! No item in this module uses a floating-point type.[^4]
//!
//! # References
//!
//! [^1]: ADR-0068, terrain is generated from the seed and is never stored as a map, decision D1. `docs/adrs/accepted/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
//! [^2]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D1. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
//! [^3]: ADR-0090, a tile upgrade is stored sparsely, as the difference from the generated world, decision D1. `docs/adrs/draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md`
//! [^4]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
//! [^5]: ADR-0023, an aggregate combines exactly, in any order, decision D1. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
//! [^6]: Findings register, FND-011. `docs/FINDINGS.md`
//! [^7]: ADR-0151, an upgrade is a category with a ground fit and a level, decisions D1 and D4. `docs/adrs/accepted/adr-0151-an-upgrade-is-a-category-with-a-ground-fit-and-a-level.md`
//! [^8]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decision D4. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`

use bytemuck::{Pod, Zeroable};

use crate::hash::StateHash;
use crate::sim_math;
use crate::terrain::{TileKind, KIND_COUNT};
use crate::types::{Accum, TileIdx};

/// The number of categories that the upgrade table holds.
///
/// **This is the width of the table and not a budget.** The table is dense
/// and its length never changes, so a world pays for the whole table however
/// many rows it fills. The number is small because the table is read for each
/// build order, and it is fixed so that a category is a stable index that a
/// state hash and a viewer both read.
///
/// Six categories are named and one is open. A caller writes the open
/// category, and a caller may rewrite any other.
pub const UPGRADE_CATEGORY_COUNT: usize = 7;

/// The most levels that one category holds.
///
/// The table holds one row for each pair of a category and a level, so the
/// row count is the product of this number and the category count. A category
/// that holds fewer levels leaves the rows above its top empty.
pub const UPGRADE_LEVEL_COUNT: usize = 2;

/// The number of rows that the table holds.
pub const UPGRADE_ROW_COUNT: usize = UPGRADE_CATEGORY_COUNT * UPGRADE_LEVEL_COUNT;

/// The level of a tile that carries no upgrade.
pub const NO_LEVEL: u8 = 0;

/// The category of an upgrade, as an index into the shared table.
///
/// The category is one byte, because the table is small and fixed. It is a
/// newtype, so no other one-byte value substitutes for it in silence.[^1]
///
/// A category is not a type, not a trait and not a verb. Adding a category
/// adds a row and no code.[^2]
///
/// # References
///
/// [^1]: ADR-0011, every value type is a newtype with a declared size and alignment. `docs/adrs/REGISTRY.md`
/// [^2]: ADR-0151, an upgrade is a category with a ground fit and a level, decision D1. `docs/adrs/accepted/adr-0151-an-upgrade-is-a-category-with-a-ground-fit-and-a-level.md`
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Pod, Zeroable)]
pub struct UpgradeCategory(pub u8);

impl UpgradeCategory {
    /// A made way. More units cross the tile at once.
    pub const ROAD: Self = Self(0);
    /// Worked ground. A unit takes more from the tile in one tick.
    pub const TERRACE: Self = Self(1);
    /// A great work. Its completion fires the wealth-or-wonder win path for
    /// the faction that holds the ground it stands on.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0148, a game end is recorded once and stops the controllers, decision D3. `docs/adrs/accepted/adr-0148-a-game-end-is-recorded-once-and-stops-the-controllers.md`
    pub const WONDER: Self = Self(2);
    /// A storehouse. It raises the store capacity of the settlement on or
    /// beside its tile.
    pub const STORE: Self = Self(3);
    /// A defence. The default table gives it a work and no effect, because
    /// the condition it wears belongs to a later item.
    pub const WALL: Self = Self(4);
    /// A dwelling. It raises the housing of the settlement on or beside its
    /// tile, so the site holds more people.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0157, a site's free places are its built housing less the residents the engine counts, decision D1. `docs/adrs/accepted/adr-0157-a-sites-free-places-are-its-built-housing-less-the-residents-the-engine-counts.md`
    pub const LODGING: Self = Self(5);
    /// The open category. The default table holds no row for it, so a build
    /// order that names it is refused until a caller writes a row.
    pub const OPEN: Self = Self(6);

    /// Every category, in the order of the numbering.
    ///
    /// A caller that must reason over the whole table reads this rather than
    /// writing a list of its own. The length is fixed by the category count,
    /// so a category that is not here is a compile error.
    pub const ALL: [Self; UPGRADE_CATEGORY_COUNT] = [
        Self::ROAD,
        Self::TERRACE,
        Self::WONDER,
        Self::STORE,
        Self::WALL,
        Self::LODGING,
        Self::OPEN,
    ];

    /// Returns the category that a small integer names.
    ///
    /// Returns `None` when the number names no category of the table.
    #[must_use]
    pub const fn from_u8(value: u8) -> Option<Self> {
        if (value as usize) < UPGRADE_CATEGORY_COUNT {
            Some(Self(value))
        } else {
            None
        }
    }

    /// Returns the category as a small integer.
    #[must_use]
    pub const fn to_u8(self) -> u8 {
        self.0
    }

    /// Returns the position of the category in a table over the categories.
    #[must_use]
    pub const fn index(self) -> usize {
        self.0 as usize
    }
}

impl core::fmt::Display for UpgradeCategory {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "category {}", self.0)
    }
}

/// Returns the ground fit bit of one ground kind.
///
/// The fit is a set of ground kinds held as a bit for each kind. The bit
/// position is the ground number, which a state hash and a viewer already
/// read, so no second numbering exists.[^1]
///
/// # References
///
/// [^1]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
#[must_use]
pub const fn ground_bit(ground: TileKind) -> u32 {
    1u32 << ground.to_u8()
}

/// The fit of a row that fits every ground a unit stands on.
///
/// Water holds nobody, so no unit ever stands there to build.[^1]
///
/// # References
///
/// [^1]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D4. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
pub const FITS_EVERY_LAND: u32 = ground_bit(TileKind::Plain)
    | ground_bit(TileKind::Forest)
    | ground_bit(TileKind::Hill)
    | ground_bit(TileKind::Mountain);

/// A value that one column of a row holds.
///
/// The trait exists so that the column reader the macro generates can hand
/// every column to a caller as one integer type.
trait ColumnValue: Copy {
    /// Returns the raw value of the column as a wide integer.
    fn to_i64(self) -> i64;
}

impl ColumnValue for u32 {
    fn to_i64(self) -> i64 {
        i64::from(self)
    }
}

/// Declares the row struct, the column names and the column reader from one
/// list, so that the row is declared once.[^1]
///
/// # References
///
/// [^1]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decision D4. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
macro_rules! declare_upgrade_row {
    ($( $(#[$meta:meta])* $column:ident : $kind:ty ),* $(,)?) => {
        /// One row of the upgrade table: one category at one level.
        ///
        /// The row is plain data with a declared layout, so a copy of the
        /// table enters the state hash byte for byte and carries no
        /// uninitialised byte.[^1]
        ///
        /// Every column is four bytes wide at an alignment of four, so the
        /// row holds no padding at all. A test asserts the size against the
        /// column count.
        ///
        /// **Zero means does not change.** A column at zero says that the row
        /// does not change the thing the column names.[^2]
        ///
        /// # References
        ///
        /// [^1]: ADR-0006, an event is plain data and applying it is pure, decision D1. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
        /// [^2]: ADR-0151, an upgrade is a category with a ground fit and a level, decision D1. `docs/adrs/accepted/adr-0151-an-upgrade-is-a-category-with-a-ground-fit-and-a-level.md`
        #[repr(C)]
        #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Pod, Zeroable)]
        pub struct UpgradeRow {
            $( $(#[$meta])* pub $column: $kind, )*
        }

        /// The number of columns that a row holds.
        pub const UPGRADE_COLUMN_COUNT: usize = [$(stringify!($column)),*].len();

        impl UpgradeRow {
            /// The name of every column, in declaration order.
            ///
            /// The Python table and the type stub carry these names, and a
            /// test asserts that the stub agrees.
            pub const COLUMN_NAMES: [&'static str; UPGRADE_COLUMN_COUNT] =
                [$(stringify!($column)),*];

            /// Returns every column as a wide integer, in declaration order.
            ///
            /// The reader exists for the boundary that copies the table out,
            /// and no pass calls it.
            #[must_use]
            pub fn columns(&self) -> [i64; UPGRADE_COLUMN_COUNT] {
                [$(ColumnValue::to_i64(self.$column)),*]
            }
        }
    };
}

declare_upgrade_row! {
    /// The ground kinds that the row fits, as one bit for each kind.
    ///
    /// **An empty fit is not a row.** The table holds one entry for every
    /// pair of a category and a level, and the fit says which of those pairs
    /// the table holds. A row that fits no ground can never be built, so it
    /// is the absence of a row and nothing states the absence twice.[^1]
    ///
    /// # References
    ///
    /// [^1]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
    ground_fit: u32,
    /// The work that finishes this level.
    ///
    /// The value is content. It is not a cost figure: it says how much work
    /// the world asks for, not what the engine spends.[^1] It is above the
    /// work that one builder adds in one tick, so a build takes several ticks
    /// and holds state between them.
    ///
    /// # References
    ///
    /// [^1]: Blockers register, BLK-007. `docs/BLOCKERS.md`
    work: u32,
    /// How much more a unit takes from the tile in one tick.
    ///
    /// The column adds to the rate that the gather resolve grants. It does
    /// not change what the tile started with, which is generated and fixed.
    yield_change: u32,
    /// The number of units that stand on the tile once the level stands.
    ///
    /// The composition takes the larger of the ground and this column, so a
    /// column below the ground changes nothing rather than taking room away.
    /// Zero means that the row does not change how many a tile holds.
    capacity_change: u32,
    /// How much the row raises the store capacity of a settlement on or
    /// beside its tile, as a raw Q16.16 quantity.
    capacity_of_store_change: u32,
    /// How much the row raises the housing of a settlement on or beside its
    /// tile.
    ///
    /// The column is a quantity of housing and not a count of people. The
    /// people it holds is that quantity divided by the housing one person
    /// takes.[^1]
    ///
    /// **The column never takes the word capacity.** The settlement arena
    /// uses that word for the ceiling on the slots it opens, and one word
    /// with two meanings inside one shape is a defect that only a reader
    /// catches.[^2]
    ///
    /// Zero means that the row houses nobody.
    ///
    /// # References
    ///
    /// [^1]: ADR-0157, a site's free places are its built housing less the residents the engine counts, decision D1. `docs/adrs/accepted/adr-0157-a-sites-free-places-are-its-built-housing-less-the-residents-the-engine-counts.md`
    /// [^2]: Findings register, FND-539. `docs/FINDINGS.md`
    housing_change: u32,
    /// The claim toward the wealth-or-wonder end that the finished row
    /// grants the faction that holds its ground.[^1]
    ///
    /// Zero means that the row grants no claim.
    ///
    /// # References
    ///
    /// [^1]: ADR-0148, a game end is recorded once and stops the controllers, decision D3. `docs/adrs/accepted/adr-0148-a-game-end-is-recorded-once-and-stops-the-controllers.md`
    victory_claim: u32,
    /// Whether the builder must stand on ground its own faction holds.
    ///
    /// Zero means that the row is built anywhere, which is how a faction
    /// reaches ground it does not yet hold.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0150, held ground is the ground within reach of a city its faction owns, decision D4. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
    own_ground_required: u32,
}

impl UpgradeRow {
    /// The row that fits no ground. The table holds it where it holds no row.
    pub const NONE: Self = Self {
        ground_fit: 0,
        work: 0,
        yield_change: 0,
        capacity_change: 0,
        capacity_of_store_change: 0,
        housing_change: 0,
        victory_claim: 0,
        own_ground_required: 0,
    };

    /// Reports whether the table holds this row.
    ///
    /// A row that fits no ground is the absence of a row.
    #[must_use]
    pub const fn exists(self) -> bool {
        self.ground_fit != 0
    }

    /// Reports whether the row fits one ground kind.
    #[must_use]
    pub const fn fits(self, ground: TileKind) -> bool {
        self.ground_fit & ground_bit(ground) != 0
    }
}

/// The reason that the table refused a caller who wrote a row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UpgradeTableError {
    /// The number names no category of the table.
    CategoryAboveCeiling(u8),
    /// The number names no level of the table. Level zero is the tile that
    /// carries no upgrade, so it holds no row.
    LevelOutsideTable(u8),
}

impl core::fmt::Display for UpgradeTableError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::CategoryAboveCeiling(value) => write!(
                formatter,
                "the upgrade category {value} is at or above the ceiling {UPGRADE_CATEGORY_COUNT}"
            ),
            Self::LevelOutsideTable(value) => write!(
                formatter,
                "the upgrade level {value} is not between 1 and {UPGRADE_LEVEL_COUNT}"
            ),
        }
    }
}

impl std::error::Error for UpgradeTableError {}

/// The reason that the engine refused a build order.
///
/// The refusal names the category and the ground, so a caller learns which
/// of the two refused it rather than reading a bare no.[^1]
///
/// # References
///
/// [^1]: ADR-0046, every error is typed. `docs/adrs/draft/adr-0046-every-error-is-typed.md`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuildRefusal {
    /// The identity names no live soldier, or it stands nowhere.
    NoSuchBuilder,
    /// The tile carries an upgrade of another category. A tile carries one
    /// upgrade.
    TileHoldsAnother {
        /// The category that stands on the tile.
        standing: UpgradeCategory,
        /// The category the order named.
        asked: UpgradeCategory,
    },
    /// The category holds no row above the level that stands there. The
    /// category is at its top, or the table holds no row for it at all.
    CategoryAtTop {
        /// The category the order named.
        category: UpgradeCategory,
        /// The level that stands on the tile.
        level: u8,
    },
    /// The row does not fit the ground under the tile.
    GroundDoesNotFit {
        /// The category the order named.
        category: UpgradeCategory,
        /// The ground under the tile.
        ground: TileKind,
    },
    /// The builder does not stand on ground its own faction holds, and the
    /// row asks for it.
    GroundNotHeld {
        /// The category the order named.
        category: UpgradeCategory,
    },
    /// The row asks for no held ground, and no project of the builder's own
    /// faction zones the tile. A row that crosses ground nobody holds is laid
    /// only inside a project, and the plan is the bound on it.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0152, a faction plans its roads and zones with one solver, decisions D3 and D4. `docs/adrs/accepted/adr-0152-a-faction-plans-its-roads-and-zones-with-one-solver.md`
    NoProject {
        /// The category the order named.
        category: UpgradeCategory,
    },
    /// A project of the builder's own faction zones the tile for another
    /// category. The plan is the bound on what a unit builds, so an order
    /// that fights a project of its own faction is refused.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0152, a faction plans its roads and zones with one solver, decisions D3 and D4. `docs/adrs/accepted/adr-0152-a-faction-plans-its-roads-and-zones-with-one-solver.md`
    ProjectHoldsAnother {
        /// The category the project zones.
        zoned: UpgradeCategory,
        /// The category the order named.
        asked: UpgradeCategory,
    },
}

impl core::fmt::Display for BuildRefusal {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NoSuchBuilder => write!(formatter, "the identity names no live builder"),
            Self::TileHoldsAnother { standing, asked } => write!(
                formatter,
                "the tile carries {standing} and the order named {asked}"
            ),
            Self::CategoryAtTop { category, level } => write!(
                formatter,
                "the {category} holds no row above the level {level} that stands there"
            ),
            Self::GroundDoesNotFit { category, ground } => write!(
                formatter,
                "the {category} does not fit the ground {}",
                ground.to_u8()
            ),
            Self::GroundNotHeld { category } => write!(
                formatter,
                "the {category} asks for ground the builder's own faction holds"
            ),
            Self::NoProject { category } => write!(
                formatter,
                "no project of the builder's own faction zones this tile for the {category}"
            ),
            Self::ProjectHoldsAnother { zoned, asked } => write!(
                formatter,
                "a project of the builder's own faction zones this tile for {zoned} \
                 and the order named {asked}"
            ),
        }
    }
}

impl std::error::Error for BuildRefusal {}

// ---------------------------------------------------------------------------
// The default table
// ---------------------------------------------------------------------------
//
// Every value below is provisional. The balance register holds one row for
// each of them, marked unset, with the derivation that names this item.[^1]
// Do not tune a value here.
//
// [^1]: Balance register, upgrades. `docs/reference/balance.md`

/// The ground that a road fits.
///
/// A road is a made way over ground a unit walks. High ground is not it, so a
/// road stops at the mountain.[^1]
///
/// # References
///
/// [^1]: Balance register, the road ground fit. `docs/reference/balance.md`
pub const ROAD_FIT: u32 =
    ground_bit(TileKind::Plain) | ground_bit(TileKind::Forest) | ground_bit(TileKind::Hill);

/// The ground that a terrace fits.
///
/// Worked ground is ground a unit walks and works. High ground is not it, so
/// a terrace stops at the mountain.[^1]
///
/// # References
///
/// [^1]: Balance register, the terrace ground fit. `docs/reference/balance.md`
pub const TERRACE_FIT: u32 =
    ground_bit(TileKind::Plain) | ground_bit(TileKind::Forest) | ground_bit(TileKind::Hill);

/// The work that finishes the first level of a road.[^1]
///
/// # References
///
/// [^1]: Balance register, the road work by level. `docs/reference/balance.md`
pub const ROAD_LEVEL_1_WORK: u32 = 8;

/// The work that finishes the second level of a road.[^1]
///
/// # References
///
/// [^1]: Balance register, the road work by level. `docs/reference/balance.md`
pub const ROAD_LEVEL_2_WORK: u32 = 24;

/// The work that finishes the first level of a terrace.[^1]
///
/// # References
///
/// [^1]: Balance register, the terrace work by level. `docs/reference/balance.md`
pub const TERRACE_LEVEL_1_WORK: u32 = 24;

/// The work that finishes the second level of a terrace.[^1]
///
/// # References
///
/// [^1]: Balance register, the terrace work by level. `docs/reference/balance.md`
pub const TERRACE_LEVEL_2_WORK: u32 = 72;

/// The work that finishes a wonder.[^1] [^2]
///
/// A provisional value of 2400. The project owner asked for a much higher
/// bar, and the balance register holds the derivation.[^1]
///
/// # References
///
/// [^1]: Balance register, the wonder work. `docs/reference/balance.md`
/// [^2]: Blockers register, BLK-007. `docs/BLOCKERS.md`
pub const WONDER_WORK: u32 = 2400;

/// The work that finishes a store.[^1] [^2]
///
/// # References
///
/// [^1]: Balance register, the store work. `docs/reference/balance.md`
/// [^2]: Blockers register, BLK-007. `docs/BLOCKERS.md`
pub const STORE_WORK: u32 = 48;

/// The ground that a lodging fits.
///
/// A dwelling stands on ground a unit walks and settles. High ground is not
/// it, so a lodging stops at the mountain.[^1]
///
/// # References
///
/// [^1]: Balance register, the lodging ground fit. `docs/reference/balance.md`
pub const LODGING_FIT: u32 =
    ground_bit(TileKind::Plain) | ground_bit(TileKind::Forest) | ground_bit(TileKind::Hill);

/// The work that finishes the first level of a lodging.[^1] [^2]
///
/// # References
///
/// [^1]: Balance register, the lodging work by level. `docs/reference/balance.md`
/// [^2]: Blockers register, BLK-050. `docs/BLOCKERS.md`
pub const LODGING_LEVEL_1_WORK: u32 = 24;

/// The work that finishes the second level of a lodging.[^1] [^2]
///
/// # References
///
/// [^1]: Balance register, the lodging work by level. `docs/reference/balance.md`
/// [^2]: Blockers register, BLK-050. `docs/BLOCKERS.md`
pub const LODGING_LEVEL_2_WORK: u32 = 72;

/// The housing that one level of a lodging adds to the settlement on or beside
/// its tile.
///
/// The value is half the housing a founding gives, so two levels of one
/// lodging double the people a founded site holds. It is a quantity of housing
/// and not a count of people.[^1]
///
/// # References
///
/// [^1]: Balance register, the lodging housing by level. `docs/reference/balance.md`
pub const LODGING_LEVEL_HOUSING: u32 = crate::growth::FOUNDING_HOUSING_DEFAULT / 2;

/// The work that finishes a wall.[^1] [^2]
///
/// # References
///
/// [^1]: Balance register, the wall work. `docs/reference/balance.md`
/// [^2]: Blockers register, BLK-007. `docs/BLOCKERS.md`
pub const WALL_WORK: u32 = 16;

/// The units that stand on a tile that carries the first level of a road.
///
/// The project already holds the capacity of ground that a unit crosses
/// quickly. The value is not restated here: this row reads it from the
/// terrain module, so no second declaration can disagree with it.[^1]
///
/// # References
///
/// [^1]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
pub const ROAD_LEVEL_1_CAPACITY: u32 = crate::terrain::CROSSING_CAPACITY;

/// The units that stand on a tile that carries the second level of a road.
///
/// Twice the first level, so a watcher reads the second level from what the
/// tile holds.[^1]
///
/// # References
///
/// [^1]: Balance register, the road capacity by level. `docs/reference/balance.md`
pub const ROAD_LEVEL_2_CAPACITY: u32 = ROAD_LEVEL_1_CAPACITY * 2;

/// How much more a unit takes from a tile that carries the first level of a
/// terrace.[^1]
///
/// # References
///
/// [^1]: Balance register, the terrace yield by level. `docs/reference/balance.md`
pub const TERRACE_LEVEL_1_YIELD: u32 = 2;

/// How much more a unit takes from a tile that carries the second level of a
/// terrace.[^1]
///
/// # References
///
/// [^1]: Balance register, the terrace yield by level. `docs/reference/balance.md`
pub const TERRACE_LEVEL_2_YIELD: u32 = 4;

/// The store capacity that one finished store adds, as a raw Q16.16
/// quantity.[^1]
///
/// # References
///
/// [^1]: Balance register, the store capacity raise. `docs/reference/balance.md`
pub const STORE_CAPACITY_RAISE: u32 = 64 << 16;

/// The claim toward the wealth-or-wonder end that one finished wonder
/// grants.[^1]
///
/// # References
///
/// [^1]: Balance register, the wonder victory claim. `docs/reference/balance.md`
pub const WONDER_VICTORY_CLAIM: u32 = 1;

/// The value that says a row asks for the builder's own ground.
///
/// The column is a whole number and the rule it holds is a yes or a no. The
/// record asks that the rule be a column rather than a branch on a category,
/// and it also says that no column is a flag.[^1] The tension is recorded in
/// the item that wrote the table.
///
/// # References
///
/// [^1]: ADR-0151, an upgrade is a category with a ground fit and a level, decisions D1 and D4. `docs/adrs/accepted/adr-0151-an-upgrade-is-a-category-with-a-ground-fit-and-a-level.md`
pub const OWN_GROUND_REQUIRED: u32 = 1;

/// The work that one builder adds to a site in one tick.
///
/// The rate is content, and the register holds the open choice of its
/// value.[^1] It is smaller than the work of every row, so no build finishes
/// in the tick it started.
///
/// # References
///
/// [^1]: Decisions register, DEC-072. `docs/DECISIONS.md`
pub const BUILD_RATE: i64 = 1;

/// The condition of an upgrade that nothing has worn.
///
/// **The scale is fixed and it is the same for every category.** A road and a
/// wonder both start here and both reach nothing after the same amount of
/// wear. What differs between them is the price of a repair, because a repair
/// buys condition with the work that built the level. One statement of the
/// scale keeps the wear rates readable: a rate is a number of these units for
/// each tick, and no rate needs a table of its own.[^1]
///
/// The number is large so that a repair divides into it without losing much.
/// A builder that adds one work to a level of 2400 work buys 416 units, and
/// the exact share is 416.67, so a full repair of the largest row in the
/// default table costs four ticks more than the build did. A small scale
/// would round that share to zero and a repair would then never finish.[^2]
///
/// # References
///
/// [^1]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
/// [^2]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
pub const CONDITION_FULL: i64 = 1_000_000;

/// The condition that a storm takes from what stands under it, in one tick.
///
/// **This is a provisional value and not a measured one.** A tile that stands
/// under an unbroken storm loses its upgrade after 2000 ticks, which is the
/// deadline of one campaign. Weather is intermittent, so a neglected road
/// under the weather of the demonstration world lasts several times that.
///
/// The rate does not read the category. A wall and a road wear at one rate,
/// and the row holds no column that resists wear.[^1]
///
/// # References
///
/// [^1]: Blockers register, BLK-050. `docs/BLOCKERS.md`
pub const WEATHER_WEAR_FOR_EACH_TICK: i64 = 500;

/// The condition that one hostile unit takes from what it stands on, in one
/// tick.
///
/// **This is a provisional value and not a measured one.** One hostile unit
/// alone takes 500 ticks to wear an upgrade away, and it is four times as
/// quick as an unbroken storm. A cohort of four takes 125 ticks, which is
/// short enough that a raid on a road is worth ordering and long enough that
/// a unit crossing a tile does almost nothing.
///
/// # References
///
/// [^1]: Blockers register, BLK-050. `docs/BLOCKERS.md`
pub const ARMY_WEAR_FOR_EACH_UNIT: i64 = 2_000;

/// Returns the condition that one contribution of work buys.
///
/// **A repair buys condition with the work that built the level.** A full
/// repair of a level therefore costs the same worker ticks the build of that
/// level cost, whatever the category is. No second rate exists, so a category
/// cannot become cheap to keep and expensive to raise.[^1]
///
/// The share is exact integer arithmetic and it truncates towards zero.[^2]
/// Returns zero when the level asks for no work, which is a level the table
/// does not hold.
///
/// # References
///
/// [^1]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
/// [^2]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
#[must_use]
pub fn repair_gain(work: i64, level_work: i64) -> i64 {
    if work <= 0 || level_work <= 0 {
        return 0;
    }
    sim_math::share(Accum(CONDITION_FULL), Accum(work), Accum(level_work))
        .map_or(0, |gained| gained.0)
}

/// Returns the work that buys back a gap in the condition of a level.
///
/// This is the inverse of the repair gain, and it truncates towards zero in
/// the same way.[^2] The two therefore state one price, and a caller cannot
/// charge for a repair at one rate and pay for it at another.[^1]
///
/// **A gap that costs less than one unit of work costs nothing.** One unit is
/// the smallest amount a builder adds in a tick, and the wear of a tick is a
/// very small part of a level. A repair that charged a whole unit for a gap
/// worth a hundredth of one would take every unit a builder ever added, and
/// no level on ground that wears at all could ever rise. The gap then stays
/// open, it grows with the wear, and the repair takes a unit of work as soon
/// as it is worth one.
///
/// Returns zero when the gap is at or below zero, and when the level asks for
/// no work.
///
/// # References
///
/// [^1]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
/// [^2]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
#[must_use]
pub fn repair_work(missing: i64, level_work: i64) -> i64 {
    if missing <= 0 || level_work <= 0 {
        return 0;
    }
    sim_math::share(Accum(missing), Accum(level_work), Accum(CONDITION_FULL))
        .map_or(0, |asked| asked.0)
}

/// The default table that a world is built with.
///
/// It holds the road, the terrace, the wonder, the store, the wall and the
/// lodging. The road, the terrace and the lodging hold two levels each. The open
/// category holds no row, so a caller writes one.[^1]
///
/// # References
///
/// [^1]: ADR-0151, an upgrade is a category with a ground fit and a level, decisions D1 and D6. `docs/adrs/accepted/adr-0151-an-upgrade-is-a-category-with-a-ground-fit-and-a-level.md`
pub const DEFAULT_UPGRADE_TABLE: UpgradeTable = {
    let mut rows = [UpgradeRow::NONE; UPGRADE_ROW_COUNT];
    rows[row_at(UpgradeCategory::ROAD, 1)] = UpgradeRow {
        ground_fit: ROAD_FIT,
        work: ROAD_LEVEL_1_WORK,
        capacity_change: ROAD_LEVEL_1_CAPACITY,
        ..UpgradeRow::NONE
    };
    rows[row_at(UpgradeCategory::ROAD, 2)] = UpgradeRow {
        ground_fit: ROAD_FIT,
        work: ROAD_LEVEL_2_WORK,
        capacity_change: ROAD_LEVEL_2_CAPACITY,
        ..UpgradeRow::NONE
    };
    rows[row_at(UpgradeCategory::TERRACE, 1)] = UpgradeRow {
        ground_fit: TERRACE_FIT,
        work: TERRACE_LEVEL_1_WORK,
        yield_change: TERRACE_LEVEL_1_YIELD,
        own_ground_required: OWN_GROUND_REQUIRED,
        ..UpgradeRow::NONE
    };
    rows[row_at(UpgradeCategory::TERRACE, 2)] = UpgradeRow {
        ground_fit: TERRACE_FIT,
        work: TERRACE_LEVEL_2_WORK,
        yield_change: TERRACE_LEVEL_2_YIELD,
        own_ground_required: OWN_GROUND_REQUIRED,
        ..UpgradeRow::NONE
    };
    rows[row_at(UpgradeCategory::WONDER, 1)] = UpgradeRow {
        ground_fit: FITS_EVERY_LAND,
        work: WONDER_WORK,
        victory_claim: WONDER_VICTORY_CLAIM,
        own_ground_required: OWN_GROUND_REQUIRED,
        ..UpgradeRow::NONE
    };
    rows[row_at(UpgradeCategory::STORE, 1)] = UpgradeRow {
        ground_fit: FITS_EVERY_LAND,
        work: STORE_WORK,
        capacity_of_store_change: STORE_CAPACITY_RAISE,
        own_ground_required: OWN_GROUND_REQUIRED,
        ..UpgradeRow::NONE
    };
    rows[row_at(UpgradeCategory::WALL, 1)] = UpgradeRow {
        ground_fit: FITS_EVERY_LAND,
        work: WALL_WORK,
        own_ground_required: OWN_GROUND_REQUIRED,
        ..UpgradeRow::NONE
    };
    rows[row_at(UpgradeCategory::LODGING, 1)] = UpgradeRow {
        ground_fit: LODGING_FIT,
        work: LODGING_LEVEL_1_WORK,
        housing_change: LODGING_LEVEL_HOUSING,
        own_ground_required: OWN_GROUND_REQUIRED,
        ..UpgradeRow::NONE
    };
    rows[row_at(UpgradeCategory::LODGING, 2)] = UpgradeRow {
        ground_fit: LODGING_FIT,
        work: LODGING_LEVEL_2_WORK,
        housing_change: LODGING_LEVEL_HOUSING,
        own_ground_required: OWN_GROUND_REQUIRED,
        ..UpgradeRow::NONE
    };
    UpgradeTable { rows }
};

/// Returns the position of one category at one level in the row array.
///
/// The level is the level that stands on the tile, so level one is the first
/// row. Level zero holds no row and this function is never called with it.
#[must_use]
const fn row_at(category: UpgradeCategory, level: u8) -> usize {
    category.index() * UPGRADE_LEVEL_COUNT + (level as usize - 1)
}

/// The shared table that a category and a level index.
///
/// The table is dense and its length never changes. A caller fills the rows
/// it wants and leaves the rest empty.[^1]
///
/// # References
///
/// [^1]: ADR-0151, an upgrade is a category with a ground fit and a level, decision D1. `docs/adrs/accepted/adr-0151-an-upgrade-is-a-category-with-a-ground-fit-and-a-level.md`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UpgradeTable {
    rows: [UpgradeRow; UPGRADE_ROW_COUNT],
}

impl Default for UpgradeTable {
    fn default() -> Self {
        DEFAULT_UPGRADE_TABLE
    }
}

impl UpgradeTable {
    /// Builds a table that holds no row.
    ///
    /// A world built with this table refuses every build order, because no
    /// row fits any ground.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            rows: [UpgradeRow::NONE; UPGRADE_ROW_COUNT],
        }
    }

    /// Returns every row, by category and then by level.
    #[must_use]
    pub const fn rows(&self) -> &[UpgradeRow; UPGRADE_ROW_COUNT] {
        &self.rows
    }

    /// Returns the row of one category at one level.
    ///
    /// Returns `None` when the level is zero, when the level is above the
    /// table, and when the table holds no row there.
    #[must_use]
    pub const fn row(&self, category: UpgradeCategory, level: u8) -> Option<UpgradeRow> {
        if level == NO_LEVEL || level as usize > UPGRADE_LEVEL_COUNT {
            return None;
        }
        let row = self.rows[row_at(category, level)];
        if row.exists() {
            Some(row)
        } else {
            None
        }
    }

    /// Returns the work that the row above one level asks for.
    ///
    /// The value is zero at the top of a category, so a builder there adds
    /// nothing and banks nothing.[^1]
    ///
    /// # References
    ///
    /// [^1]: Findings register, FND-011. `docs/FINDINGS.md`
    #[must_use]
    pub const fn work_above(&self, category: UpgradeCategory, level: u8) -> i64 {
        match self.row(category, level + 1) {
            Some(row) => row.work as i64,
            None => 0,
        }
    }

    /// Returns the work that the row standing at one level asked for.
    ///
    /// The value is zero at level zero, because nothing stands there. A
    /// repair reads this, so the price of a repair and the price of the build
    /// come from one column.[^1]
    ///
    /// # References
    ///
    /// [^1]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
    #[must_use]
    pub const fn work_at(&self, category: UpgradeCategory, level: u8) -> i64 {
        match self.row(category, level) {
            Some(row) => row.work as i64,
            None => 0,
        }
    }

    /// Returns the highest level that one category reaches.
    ///
    /// Returns zero when the table holds no row for the category. A level
    /// above a gap is never reached, so the scan stops at the first gap.
    #[must_use]
    pub const fn top_level(&self, category: UpgradeCategory) -> u8 {
        let mut level = 0u8;
        while (level as usize) < UPGRADE_LEVEL_COUNT {
            if self.row(category, level + 1).is_none() {
                return level;
            }
            level += 1;
        }
        level
    }

    /// Returns the largest work that any row of the table asks for.
    ///
    /// The value is folded over the table rather than written down a second
    /// time. A written ceiling is one fact in two places, and nothing fails
    /// when the two disagree.[^1]
    ///
    /// # References
    ///
    /// [^1]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
    #[must_use]
    pub const fn largest_work(&self) -> i64 {
        let mut most = 0i64;
        let mut at = 0usize;
        while at < UPGRADE_ROW_COUNT {
            let work = self.rows[at].work as i64;
            if work > most {
                most = work;
            }
            at += 1;
        }
        most
    }

    /// Writes one row of the table.
    ///
    /// The caller gives the whole row. There is no partial form, because a
    /// caller that gave two columns would leave the rest at zero and would
    /// define an upgrade that changes nothing else without knowing it.[^1]
    ///
    /// A row whose ground fit is empty removes the row.
    ///
    /// # Errors
    ///
    /// Returns an error when the number names no category, and when the level
    /// is not between one and the level count.
    ///
    /// # References
    ///
    /// [^1]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decision D5. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
    pub fn define(
        &mut self,
        category: u8,
        level: u8,
        row: UpgradeRow,
    ) -> Result<(), UpgradeTableError> {
        let Some(category) = UpgradeCategory::from_u8(category) else {
            return Err(UpgradeTableError::CategoryAboveCeiling(category));
        };
        if level == NO_LEVEL || level as usize > UPGRADE_LEVEL_COUNT {
            return Err(UpgradeTableError::LevelOutsideTable(level));
        }
        self.rows[row_at(category, level)] = row;
        Ok(())
    }

    /// Absorbs the table into the state hash.
    ///
    /// The table decides what a later frame does, so the whole-world hash
    /// covers it. Two worlds built with different tables never hash the
    /// same.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
    #[must_use]
    pub fn hash_into(&self, hash: StateHash) -> StateHash {
        hash.write(bytemuck::cast_slice(&self.rows))
    }

    /// Reports whether the table holds its invariants.
    ///
    /// A row the table holds fits a ground the numbering names, and it asks
    /// for work above nothing. A row that asked for no work would finish in
    /// the tick it started, and a build would then hold no state between
    /// ticks.
    #[must_use]
    pub fn check_invariants(&self) -> bool {
        let named = (0..KIND_COUNT).fold(0u32, |bits, at| bits | (1u32 << at));
        self.rows
            .iter()
            .all(|row| !row.exists() || (row.ground_fit & !named == 0 && row.work > 0))
    }
}

/// The number of key bits that hold the category.
///
/// The width is derived from the table, so a new category widens the key
/// rather than colliding inside it.
const CATEGORY_BITS: u32 = UPGRADE_CATEGORY_COUNT.next_power_of_two().trailing_zeros();

/// Packs a tile and a category into one ordering key.
///
/// The tile is the high part, so a sort by this key gives ascending tile
/// order, and the segments of one tile stay together.[^1]
///
/// # References
///
/// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
#[must_use]
pub const fn site_key(tile: TileIdx, category: UpgradeCategory) -> u64 {
    ((tile.0 as u64) << CATEGORY_BITS) | (category.to_u8() as u64)
}

/// Returns the largest key that a world of a given tile count produces.
#[must_use]
pub const fn key_ceiling(tile_count: u32) -> u64 {
    site_key(
        TileIdx(tile_count.saturating_sub(1)),
        UpgradeCategory::ALL[UPGRADE_CATEGORY_COUNT - 1],
    )
}

/// Returns how many units may stand on a tile.
///
/// This is the one function that answers the question. The ground states the
/// capacity, the row that stands on the tile may state a larger one, and the
/// larger of the two wins. The two tables meet in one place, so no caller can
/// read one without the other.[^1]
///
/// **Ground that admits nobody stays closed.** An upgrade changes how many a
/// tile holds. It never changes whether the tile holds anybody, so every
/// caller that asks only about passability reads the ground and stays
/// correct.
///
/// The argument is the row that stands on the tile. A tile that carries
/// nothing, and a tile whose first level is still under construction, change
/// nothing.
///
/// The function reads a column and names no category.[^2]
///
/// # References
///
/// [^1]: ADR-0090, a tile upgrade is stored sparsely, as the difference from the generated world, decision D3. `docs/adrs/draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md`
/// [^2]: ADR-0151, an upgrade is a category with a ground fit and a level, decision D4. `docs/adrs/accepted/adr-0151-an-upgrade-is-a-category-with-a-ground-fit-and-a-level.md`
#[must_use]
pub const fn capacity_with(ground: u32, standing: Option<UpgradeRow>) -> u32 {
    if ground == 0 {
        return 0;
    }
    match standing {
        // An upgrade never lowers what a tile holds. The larger of the two
        // wins, so a row whose column sits below the ground it stands on
        // changes nothing rather than taking room away.
        Some(row) if row.capacity_change > ground => row.capacity_change,
        _ => ground,
    }
}

/// Returns how much one unit takes from a tile in one tick.
///
/// The base rate is what the gather resolve grants on unimproved ground, and
/// the row that stands on the tile adds its yield column.
#[must_use]
pub const fn gather_rate_with(base: u32, standing: Option<UpgradeRow>) -> u32 {
    match standing {
        Some(row) => base.saturating_add(row.yield_change),
        None => base,
    }
}

/// One upgrade: the category that stands on a tile, the level it reached, and
/// the work toward the next level.
///
/// A tile carries at most one upgrade. Two upgrades on one tile would make
/// "the tile returns to what it was" a question with more than one answer.
///
/// **A level is raised in place.** The entry never gains a sibling, and the
/// storage of an upgrade does not grow with its level.[^1]
///
/// # References
///
/// [^1]: ADR-0151, an upgrade is a category with a ground fit and a level, decision D3. `docs/adrs/accepted/adr-0151-an-upgrade-is-a-category-with-a-ground-fit-and-a-level.md`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UpgradeSite {
    /// The tile that carries the upgrade.
    pub tile: TileIdx,
    /// What is being built, or what stands there.
    pub category: UpgradeCategory,
    /// The level that stands on the tile.
    ///
    /// Zero means that the first level is still under construction and that
    /// nothing stands there yet.
    pub level: u8,
    /// The work that has gone into the next level.
    ///
    /// The accumulator is 64 bits wide and every term is a whole number, so
    /// the total is the same in any order.[^1] It never rises above the work
    /// of the row above the entry, and it returns to zero when the level
    /// rises.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0023, an aggregate combines exactly, in any order, decision D1. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
    /// [^2]: Findings register, FND-011. `docs/FINDINGS.md`
    pub progress: Accum,
    /// How much of the level that stands there is still sound.
    ///
    /// The value runs from nothing to the full condition, on a scale that
    /// every category shares. A level that has just been finished stands at
    /// the full condition, the weather and a hostile army take from it, and a
    /// worker on the tile puts it back.
    ///
    /// **A site at nothing is gone.** The wear pass removes the entry rather
    /// than storing a zero, so no reader has to ask whether a stored upgrade
    /// is really there.
    ///
    /// The value is a whole number and the wear of one tick is a sum of whole
    /// numbers, so the total is the same in any order.[^1] It is stored state
    /// that the next tick reads, so it enters the state hash.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0023, an aggregate combines exactly, in any order, decision D1. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
    /// [^2]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
    pub condition: Accum,
}

impl UpgradeSite {
    /// Reports whether a level of the upgrade stands on the tile.
    ///
    /// A site at level zero is a first build under construction, and it
    /// changes nothing about the tile.
    #[must_use]
    pub const fn is_complete(self) -> bool {
        self.level > NO_LEVEL
    }

    /// Reports whether the level that stands there has lost condition.
    ///
    /// A site under construction is never damaged. Nothing stands on the tile
    /// yet, so there is nothing for the weather or an army to take.
    #[must_use]
    pub const fn is_damaged(self) -> bool {
        self.is_complete() && self.condition.0 < CONDITION_FULL
    }

    /// Returns the work that mends what stands here back to its full
    /// condition.
    ///
    /// **This is the one statement of whether a repair is due.** The build
    /// pass spends this work before it raises anything, and the resolution
    /// that reads a row for a build order asks whether it is above zero. A
    /// second statement of the question would let a unit be ordered onto a
    /// repair that the pass then declines to do.[^1]
    ///
    /// Returns zero when nothing stands here, when the level is sound, and
    /// when the gap in the condition is worth less than one unit of work.
    ///
    /// # References
    ///
    /// [^1]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
    #[must_use]
    pub fn repair_price(self, table: &UpgradeTable) -> i64 {
        if !self.is_damaged() {
            return 0;
        }
        repair_work(
            CONDITION_FULL - self.condition.0,
            table.work_at(self.category, self.level),
        )
    }

    /// Returns the work that the next level still asks for.
    ///
    /// Returns zero at the top of the category.
    #[must_use]
    pub const fn remaining(self, table: &UpgradeTable) -> i64 {
        let work = table.work_above(self.category, self.level);
        if self.progress.0 >= work {
            0
        } else {
            work - self.progress.0
        }
    }
}

/// Every upgrade in one world.
///
/// The map holds one entry for each tile that carries an upgrade, and it
/// holds nothing else. A world in which nobody built holds no entry, so the
/// memory cost follows the building and not the size of the world.[^1]
///
/// The entries are held sorted by tile, so a lookup is a binary search and
/// the order never depends on which unit built first.[^2]
///
/// An entry is merged in ascending runs, never inserted one at a time.
/// Inserting into the middle of a vector moves every later entry, which is
/// quadratic in the number of tiles a frame touches.
///
/// # References
///
/// [^1]: ADR-0090, a tile upgrade is stored sparsely, as the difference from the generated world, decision D1. `docs/adrs/draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md`
/// [^2]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UpgradeMap {
    sites: Vec<UpgradeSite>,
    scratch: Vec<UpgradeSite>,
    visits: u64,
    collapses: u64,
}

impl UpgradeMap {
    /// Builds a map that holds no upgrade.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            sites: Vec::new(),
            scratch: Vec::new(),
            visits: 0,
            collapses: 0,
        }
    }

    /// Returns the number of tiles that carry an upgrade.
    #[must_use]
    pub fn len(&self) -> usize {
        self.sites.len()
    }

    /// Reports whether no tile carries an upgrade.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.sites.is_empty()
    }

    /// Returns every upgrade, in ascending tile order.
    #[must_use]
    pub fn sites(&self) -> &[UpgradeSite] {
        &self.sites
    }

    /// Returns the number of entries that the last advance read.
    ///
    /// The advance reads the sites and the builders. It takes no grid and no
    /// tile count, so it cannot read a tile that carries no upgrade.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0090, a tile upgrade is stored sparsely, as the difference from the generated world, decision D1. `docs/adrs/draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md`
    #[must_use]
    pub const fn last_advance_visits(&self) -> u64 {
        self.visits
    }

    /// Returns the upgrade on one tile.
    ///
    /// Returns `None` when the tile carries none.
    #[must_use]
    pub fn at(&self, tile: TileIdx) -> Option<UpgradeSite> {
        match self.sites.binary_search_by_key(&tile.0, |site| site.tile.0) {
            Ok(at) => Some(self.sites[at]),
            Err(_) => None,
        }
    }

    /// Returns the row that stands on one tile.
    ///
    /// Returns `None` when the tile carries no upgrade, and when the upgrade
    /// there has not reached its first level. A site that stands at no level
    /// changes nothing about the tile.
    #[must_use]
    pub fn standing(&self, tile: TileIdx, table: &UpgradeTable) -> Option<UpgradeRow> {
        let site = self.at(tile)?;
        table.row(site.category, site.level)
    }

    /// Removes the upgrade from one tile and returns what stood there.
    ///
    /// The tile returns to the world the generator made, at whatever level
    /// the upgrade stood. Nothing else stores a property of the tile, so
    /// removing the entry is the whole of the return, and no second copy can
    /// survive it.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0090, a tile upgrade is stored sparsely, as the difference from the generated world, decision D4. `docs/adrs/draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md`
    pub fn remove(&mut self, tile: TileIdx) -> Option<UpgradeSite> {
        match self.sites.binary_search_by_key(&tile.0, |site| site.tile.0) {
            Ok(at) => Some(self.sites.remove(at)),
            Err(_) => None,
        }
    }

    /// Adds a run of work, given in ascending tile order.
    ///
    /// Each element names a tile, the category being built there, and the
    /// work that this tick added. The caller states the order and the merge
    /// relies on it: a run out of order would silently produce an unsorted
    /// map, and every later lookup would then read the wrong tile.
    ///
    /// A tile that holds no site gains one at level zero. A tile that holds a
    /// site of the named category advances it. **A tile that holds a site of
    /// another category is left alone**, because a tile carries one upgrade
    /// and the one that is already there is the one the world holds.
    ///
    /// **A level rises in place.** When the work reaches the work of the row
    /// above the entry, the level rises by one and the work done returns to
    /// zero. No second entry is written.[^2]
    ///
    /// The work done is clamped at the work of the row above the entry. An
    /// unclamped accumulator banks surplus that nothing can spend, and that
    /// surplus reaches the state hash.[^1]
    ///
    /// # References
    ///
    /// [^1]: Findings register, FND-011. `docs/FINDINGS.md`
    /// [^2]: ADR-0151, an upgrade is a category with a ground fit and a level, decision D3. `docs/adrs/accepted/adr-0151-an-upgrade-is-a-category-with-a-ground-fit-and-a-level.md`
    pub fn merge_ascending(
        &mut self,
        run: &[(TileIdx, UpgradeCategory, i64)],
        table: &UpgradeTable,
    ) {
        debug_assert!(
            run.windows(2).all(|pair| pair[0].0 .0 < pair[1].0 .0),
            "a merged run must be sorted by tile and name each tile once"
        );
        debug_assert!(
            run.iter().all(|added| added.2 >= 0),
            "work never runs backwards"
        );
        let mut visits = 0u64;
        if run.is_empty() {
            // Nothing was built, so the merge read nothing. It did not walk
            // the sites, and it did not walk the world.
            self.visits = 0;
            return;
        }
        self.scratch.clear();
        self.scratch.reserve(self.sites.len() + run.len());
        let (mut here, mut there) = (0usize, 0usize);
        while here < self.sites.len() && there < run.len() {
            let mine = self.sites[here];
            let theirs = run[there];
            visits += 1;
            if mine.tile.0 < theirs.0 .0 {
                self.scratch.push(mine);
                here += 1;
            } else if theirs.0 .0 < mine.tile.0 {
                self.scratch.push(fresh_site(theirs, table));
                there += 1;
            } else {
                self.scratch.push(advanced(mine, theirs.1, theirs.2, table));
                here += 1;
                there += 1;
            }
        }
        visits += (self.sites.len() - here) as u64;
        self.scratch.extend_from_slice(&self.sites[here..]);
        for added in &run[there..] {
            visits += 1;
            self.scratch.push(fresh_site(*added, table));
        }
        core::mem::swap(&mut self.sites, &mut self.scratch);
        self.visits = visits;
    }

    /// Takes condition from a run of sites, given in ascending tile order.
    ///
    /// Each element names a tile and the condition that this tick took from
    /// what stands there. The caller states the order and the walk relies on
    /// it, in the same way the work merge does: a run out of order would leave
    /// later entries unworn in silence.
    ///
    /// **A site that reaches nothing is removed, and the tile returns to the
    /// world the generator made.** Nothing else stores a property of an
    /// improved tile, so dropping the entry is the whole of the collapse and
    /// no second copy can survive it.[^1] This is the only sink an upgrade has
    /// that a caller does not drive by hand.
    ///
    /// A site under construction is left alone. Nothing stands on its tile
    /// yet, so there is nothing to wear. A tile the run names that carries no
    /// site is ignored.
    ///
    /// Returns how many sites collapsed on this call.
    ///
    /// # References
    ///
    /// [^1]: ADR-0090, a tile upgrade is stored sparsely, as the difference from the generated world, decision D4. `docs/adrs/draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md`
    pub fn wear_ascending(&mut self, run: &[(TileIdx, i64)]) -> u64 {
        debug_assert!(
            run.windows(2).all(|pair| pair[0].0 .0 < pair[1].0 .0),
            "a worn run must be sorted by tile and name each tile once"
        );
        debug_assert!(
            run.iter().all(|worn| worn.1 >= 0),
            "wear never gives condition back"
        );
        let mut collapsed = 0u64;
        if run.is_empty() {
            self.collapses = 0;
            return collapsed;
        }
        // One walk over the sites and one over the run, both ascending. The
        // sites are held in tile order and the caller states the run in the
        // same order, so the two advance together and neither searches.[^1]
        //
        // [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
        let mut there = 0usize;
        self.sites.retain_mut(|site| {
            while there < run.len() && run[there].0 .0 < site.tile.0 {
                there += 1;
            }
            if there >= run.len() || run[there].0 .0 != site.tile.0 {
                return true;
            }
            let taken = run[there].1;
            there += 1;
            if !site.is_complete() || taken <= 0 {
                return true;
            }
            let left = site.condition.0.saturating_sub(taken);
            if left <= 0 {
                collapsed += 1;
                return false;
            }
            site.condition = Accum(left);
            true
        });
        self.collapses = collapsed;
        collapsed
    }

    /// Returns how many sites the last wear collapsed.
    ///
    /// The count describes one tick. It is a diagnostic and not simulated
    /// state, in the same way the visit count of the last advance is, so no
    /// pass reads it and it enters no state hash.
    #[must_use]
    pub const fn last_wear_collapses(&self) -> u64 {
        self.collapses
    }

    /// Absorbs the map into the state hash.
    ///
    /// The entries enter in tile order, which the map holds them in.[^1] An
    /// unfinished build is state that the next frame reads, so the level and
    /// the progress enter as well.[^2] The condition of a standing level is
    /// state that the next frame reads too, so it enters beside them.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    /// [^2]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
    #[must_use]
    pub fn hash_into(&self, hash: StateHash) -> StateHash {
        let mut running = hash.write_u64(self.sites.len() as u64);
        for site in &self.sites {
            running = running
                .write(&site.tile.0.to_le_bytes())
                .write(&[site.category.to_u8(), site.level])
                .write(&site.progress.0.to_le_bytes())
                .write(&site.condition.0.to_le_bytes());
        }
        running
    }

    /// Reports whether the map holds its invariants.
    ///
    /// The entries rise and name each tile once. A map that broke either
    /// would answer a lookup with the wrong tile, and nothing else would
    /// notice.
    ///
    /// The level of every site is one the table holds, or zero. The progress
    /// of every site sits between nothing and the work of the row above it. A
    /// site above that work has banked surplus, which is the defect the
    /// register names.[^1] At the top of a category the bound is zero.
    ///
    /// Every tile lies inside the world.
    ///
    /// # References
    ///
    /// [^1]: Findings register, FND-011. `docs/FINDINGS.md`
    #[must_use]
    pub fn check_invariants(&self, tile_count: u32, table: &UpgradeTable) -> bool {
        if !self
            .sites
            .windows(2)
            .all(|pair| pair[0].tile.0 < pair[1].tile.0)
        {
            return false;
        }
        self.sites.iter().all(|site| {
            site.tile.0 < tile_count
                && site.level <= table.top_level(site.category)
                && site.progress.0 >= 0
                && site.progress.0 <= table.work_above(site.category, site.level)
                // The condition sits above nothing and at or below the full
                // condition. A site at nothing collapsed and the wear pass
                // removed it, so a stored zero is a site that outlived its
                // own collapse.
                && site.condition.0 > 0
                && site.condition.0 <= CONDITION_FULL
        })
    }
}

/// Builds the site that a first contribution creates.
///
/// The site starts at no level, and the contribution may raise it to the
/// first one at once.
#[must_use]
fn fresh_site(added: (TileIdx, UpgradeCategory, i64), table: &UpgradeTable) -> UpgradeSite {
    let (tile, category, work) = added;
    let start = UpgradeSite {
        tile,
        category,
        level: NO_LEVEL,
        progress: Accum(0),
        condition: Accum(CONDITION_FULL),
    };
    advanced(start, category, work, table)
}

/// Adds work to a site that already stands on the tile.
///
/// A contribution to another category is dropped. The tile carries one
/// upgrade, and it is not the one the contributor named.
///
/// **A repair comes first, and it takes only the work it is priced at.** The
/// work buys back the condition the level lost, and the work above that price
/// goes into the level. One worker therefore mends what stands before it
/// builds on top of it, and no caller has to choose between the two. The work
/// is the same contribution the build sums, so a repair and a build run
/// through one mechanism and no second rate exists.[^1]
///
/// **A repair that took the whole tick would stop every build on ground that
/// wears.** The wear of one tick is a very small part of a level, and the
/// work of one tick is the smallest amount a builder adds. The price of the
/// repair is zero until the gap is worth one unit of work, so a level under
/// light wear still rises and a level under heavy wear does not.
///
/// The level rises in place when the work reaches the work of the row above
/// the entry, and the work done then returns to zero. A level that has just
/// risen stands at the full condition.
///
/// # References
///
/// [^1]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
#[must_use]
fn advanced(
    site: UpgradeSite,
    category: UpgradeCategory,
    work: i64,
    table: &UpgradeTable,
) -> UpgradeSite {
    if site.category != category {
        return site;
    }
    let mut site = site;
    let mut work = work.max(0);
    let price = site.repair_price(table);
    if price > 0 {
        let level_work = table.work_at(site.category, site.level);
        if work < price {
            // The work buys condition and raises no level. The repair is not
            // paid for yet, so the level stays damaged and the next tick
            // carries on with it.
            let mended = site.condition.0.saturating_add(repair_gain(work, level_work));
            return UpgradeSite {
                condition: Accum(mended.min(CONDITION_FULL)),
                ..site
            };
        }
        // The repair takes the work it is priced at, and the work above that
        // price goes into the level. A repair that took the whole tick
        // whatever the gap cost would stop every build on ground that wears.
        work -= price;
        site = UpgradeSite {
            condition: Accum(CONDITION_FULL),
            ..site
        };
    }
    let asked = table.work_above(site.category, site.level);
    if asked == 0 {
        // The category is at its top. The clamp is zero, so a builder there
        // adds nothing and banks nothing.
        return UpgradeSite {
            progress: Accum(0),
            ..site
        };
    }
    let total = site.progress.0.saturating_add(work);
    if total >= asked {
        return UpgradeSite {
            level: site.level + 1,
            progress: Accum(0),
            condition: Accum(CONDITION_FULL),
            ..site
        };
    }
    UpgradeSite {
        progress: Accum(total),
        ..site
    }
}
