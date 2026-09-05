//! The unit type, and the table it indexes.
//!
//! A unit type is an index into a shared table. The table is data that the
//! world is built with. It holds no code, and a lookup in it is not a
//! callback.[^1]
//!
//! A type parameterises a verb. It does not multiply the verbs, so no pass
//! branches on a type name and no pass holds a rule for one type.[^1]
//!
//! **A row is a set of capability columns, and a zero in a column means that
//! the type cannot do what the column names.** A pass that asks a question
//! about a type reads one column. It never reads a name and it never
//! compares a type index to a constant.[^3]
//!
//! Every value in the table is a whole number or a fixed-point number in the
//! project scale. No value here is a floating point number.[^2]
//!
//! The row is declared once, through one macro, and the column names and
//! the column reader derive from that declaration. A second list of the
//! columns would rot when nothing fails.[^4]
//!
//! # References
//!
//! [^1]: ADR-0120, a unit carries a type, and the type is an index into a table the world is built with, decision D1. `docs/adrs/draft/adr-0120-a-unit-carries-a-type-that-indexes-a-table.md`
//! [^2]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
//! [^3]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decisions D1 and D2. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
//! [^4]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`

use bytemuck::{Pod, Zeroable};

use crate::hash::StateHash;
use crate::types::Fix32;

/// The number of rows that the unit type table holds.
///
/// **This is the width of the table and not a budget.** The resolution of a
/// meeting reads the table for each ordered pair of types, so its cost follows
/// the square of this number. The number is therefore a structural property of
/// the design: it is small so that the square stays small, and it is fixed so
/// that no world pays for a table it did not fill.[^1]
///
/// A row that nobody filled holds zero in every column, and a type with zero
/// in every column can do nothing.
///
/// # References
///
/// [^1]: ADR-0121, a meeting between two factions resolves at the tile, decision D3. `docs/adrs/draft/adr-0121-a-meeting-between-two-factions-resolves-at-the-tile.md`
pub const UNIT_TYPE_COUNT: usize = 8;

/// The type that a unit carries until something gives it another.
///
/// Row zero of the default table is the worker row, so a unit that nothing
/// typed gathers, builds and carries.
pub const DEFAULT_UNIT_TYPE: UnitTypeId = WORKER;

/// The worker row of the default table. It gathers, builds and carries.
pub const WORKER: UnitTypeId = UnitTypeId(0);

/// The soldier row of the default table. It fights and does nothing else.
pub const SOLDIER: UnitTypeId = UnitTypeId(1);

/// The merchant row of the default table. It carries and does nothing else.
pub const MERCHANT: UnitTypeId = UnitTypeId(2);

/// The leader row of the default table. It holds command reach and weather
/// reach and does nothing else.
pub const LEADER: UnitTypeId = UnitTypeId(3);

/// The open row of the default table. Every column is zero, and a game fills
/// it.
pub const OPEN: UnitTypeId = UnitTypeId(4);

/// The reason that the table refused a caller.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnitTypeError {
    /// The number names no row of the table.
    TypeAboveCeiling(u8),
    /// A fixed-point column is below zero. Every column is a quantity or a
    /// scale, and a negative one has no meaning. The name is the column.
    ColumnBelowZero(&'static str, Fix32),
}

impl core::fmt::Display for UnitTypeError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::TypeAboveCeiling(value) => write!(
                formatter,
                "the unit type {value} is at or above the ceiling {UNIT_TYPE_COUNT}"
            ),
            Self::ColumnBelowZero(column, value) => {
                write!(formatter, "the {column} {} is below zero", value.0)
            }
        }
    }
}

impl std::error::Error for UnitTypeError {}

/// The type of a unit, as an index into the shared table.
///
/// The type is one byte, because the table is small and fixed. It is a
/// newtype, so no other one-byte value substitutes for it in silence.[^1]
///
/// # References
///
/// [^1]: ADR-0011, every value type is a newtype with a declared size and alignment. `docs/adrs/REGISTRY.md`
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Pod, Zeroable)]
pub struct UnitTypeId(pub u8);

impl UnitTypeId {
    /// Returns the type that a number names, or `None` when the number names
    /// no row of the table.
    #[must_use]
    pub const fn from_u8(value: u8) -> Option<Self> {
        if (value as usize) < UNIT_TYPE_COUNT {
            Some(Self(value))
        } else {
            None
        }
    }

    /// Returns the row index of the type.
    #[must_use]
    pub const fn index(self) -> usize {
        self.0 as usize
    }
}

/// A value that one column of a row holds.
///
/// The trait exists so that the column reader the macro generates can hand
/// every column to a caller as one integer type, whatever the column holds.
trait ColumnValue: Copy {
    /// Returns the raw value of the column as a wide integer.
    fn to_i64(self) -> i64;
    /// Reports whether the value is below zero. A whole-number column never
    /// is.
    fn is_below_zero(self) -> bool;
}

impl ColumnValue for Fix32 {
    fn to_i64(self) -> i64 {
        i64::from(self.0)
    }

    fn is_below_zero(self) -> bool {
        self.0 < 0
    }
}

impl ColumnValue for u32 {
    fn to_i64(self) -> i64 {
        i64::from(self)
    }

    fn is_below_zero(self) -> bool {
        false
    }
}

/// Declares the row struct, the column names and the column reader from one
/// list, so that the table is declared once.[^1]
///
/// # References
///
/// [^1]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decision D4. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
macro_rules! declare_unit_type_row {
    ($( $(#[$meta:meta])* $column:ident : $kind:ty ),* $(,)?) => {
        /// One row of the unit type table.
        ///
        /// The row is plain data with a declared layout, so a copy of the
        /// table enters the state hash byte for byte and carries no
        /// uninitialised byte.[^1]
        ///
        /// Every column is four bytes wide at an alignment of four, so the
        /// row holds no padding at all. A test asserts the size against the
        /// column count.
        ///
        /// **Zero means cannot.** A rate or a capacity at zero says that the
        /// type does not do what the column names. No column is a flag, and
        /// no column is a name.[^2]
        ///
        /// **Three columns are stored, hashed and readable, and no pass
        /// reads them yet.** The `move_cost_scale` column waits for the
        /// movement pass to scale its cost. The `command_reach` column and
        /// the `weather_reach` column wait for the relation verb and the
        /// weather verb of later passes. They are declared now so that the
        /// row is widened once, and the widening is what moves every stored
        /// hash. A capability that nothing invokes is a defect shape, and
        /// this is the one pass in which it is accepted, because the passes
        /// that read the columns are named and queued.[^3]
        ///
        /// # References
        ///
        /// [^1]: ADR-0006, an event is plain data and applying it is pure, decision D1. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
        /// [^2]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decisions D1 and D2. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
        /// [^3]: Recurring Defect Shapes, shape 3. `.agents/rules/recurring-defects.md`
        #[repr(C)]
        #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Pod, Zeroable)]
        pub struct UnitTypeRow {
            $( $(#[$meta])* pub $column: $kind, )*
        }

        /// The number of columns that a row holds.
        pub const UNIT_TYPE_COLUMN_COUNT: usize = [$(stringify!($column)),*].len();

        impl UnitTypeRow {
            /// The name of every column, in declaration order.
            ///
            /// The Python table and the type stub carry these names, and a
            /// test asserts that the stub agrees.
            pub const COLUMN_NAMES: [&'static str; UNIT_TYPE_COLUMN_COUNT] =
                [$(stringify!($column)),*];

            /// Returns every column as a wide integer, in declaration order.
            ///
            /// A fixed-point column keeps its raw scaled value. A whole-number
            /// column keeps its count. The reader exists for the boundary that
            /// copies the table out, and no pass calls it.
            #[must_use]
            pub fn columns(&self) -> [i64; UNIT_TYPE_COLUMN_COUNT] {
                [$(ColumnValue::to_i64(self.$column)),*]
            }

            /// Returns the name and the value of the first column that is
            /// below zero, or `None` when every column is at or above zero.
            fn column_below_zero(&self) -> Option<(&'static str, Fix32)> {
                $(
                    if ColumnValue::is_below_zero(self.$column) {
                        let raw = ColumnValue::to_i64(self.$column);
                        return Some((stringify!($column), Fix32(raw as i32)));
                    }
                )*
                None
            }
        }
    };
}

declare_unit_type_row! {
    /// The harm that one unit of this type delivers in one resolution.
    ///
    /// The value is a whole number of casualties for each attacker, in the
    /// project fixed-point scale, so a value of one half means that two
    /// attackers of this type end one defender.
    attack: Fix32,
    /// The attack that an attacker must exceed to reach a unit of this type.
    ///
    /// The value is in the same scale as the attack, so the comparison is
    /// exact and the two never mean different things.
    armour: Fix32,
    /// The scale on what the unit takes from a tile in one tick.
    ///
    /// The gather pass multiplies the tile rate by this scale. A scale of one
    /// takes the tile rate. A scale of zero takes nothing.
    gather_rate: Fix32,
    /// The scale on the work the unit adds to an upgrade in one tick.
    ///
    /// The build pass multiplies the builder rate by this scale. A scale of
    /// one adds the builder rate. A scale of zero adds nothing.
    build_rate: Fix32,
    /// The most the unit carries, summed over every kind.
    ///
    /// The gather pass never raises a load above this. A capacity of zero
    /// means that the unit never carries, and so never gathers.
    carry_capacity: u32,
    /// The scale on the movement cost the unit pays on a tile.
    ///
    /// No pass reads this column yet. The movement pass of a later pass
    /// reads it.
    move_cost_scale: Fix32,
    /// The reach of the unit as a leader. Nonzero means the unit may move a
    /// relation and may lead a campaign.
    ///
    /// No pass reads this column yet. The relation verb of a later pass
    /// reads it.
    command_reach: u32,
    /// The reach of the unit as a god. Nonzero means the faction may inflict
    /// weather while it holds this unit.
    ///
    /// No pass reads this column yet. The weather verb of a later pass reads
    /// it.
    weather_reach: u32,
}

impl UnitTypeRow {
    /// The row whose every column is zero. A unit of this row can do nothing.
    pub const NONE: Self = Self {
        attack: Fix32::ZERO,
        armour: Fix32::ZERO,
        gather_rate: Fix32::ZERO,
        build_rate: Fix32::ZERO,
        carry_capacity: 0,
        move_cost_scale: Fix32::ZERO,
        command_reach: 0,
        weather_reach: 0,
    };
}

// ---------------------------------------------------------------------------
// The default table
// ---------------------------------------------------------------------------
//
// Every value below is a placeholder. The balance register holds the row
// "Default table, five rows by eight columns" as unset, and the balance
// harness of pass 10 sets it. Do not tune a value here. A placeholder is
// chosen so that a world built with the table behaves as the world did
// before the row was widened: a worker takes the tile rate, adds the builder
// rate, and carries without a cap it reaches.[^1]
//
// [^1]: Balance register, unit types. `docs/reference/balance.md`

/// The placeholder scale of a rate the type performs in full. It is one, so
/// the pass takes the tile rate or the builder rate unchanged.[^1]
///
/// # References
///
/// [^1]: Balance register, unit types, the default table row. `docs/reference/balance.md`
pub const PLACEHOLDER_FULL_RATE: Fix32 = Fix32::ONE;

/// The placeholder carry capacity of a type that carries. It is the largest
/// value the column holds, so no load a gather can build reaches it.[^1]
///
/// # References
///
/// [^1]: Balance register, unit types, the default table row. `docs/reference/balance.md`
pub const PLACEHOLDER_CARRY_CAPACITY: u32 = u32::MAX;

/// The placeholder attack of the soldier row. It is one whole casualty for
/// each attacker.[^1]
///
/// # References
///
/// [^1]: Balance register, unit types, the default table row. `docs/reference/balance.md`
pub const PLACEHOLDER_SOLDIER_ATTACK: Fix32 = Fix32::ONE;

/// The placeholder reach of the leader row. It is the smallest nonzero value,
/// because nothing reads the size of the reach yet and a nonzero value is
/// what the gates of later passes read.[^1]
///
/// # References
///
/// [^1]: Balance register, unit types, the default table row. `docs/reference/balance.md`
pub const PLACEHOLDER_REACH: u32 = 1;

/// The worker row. It gathers, builds and carries, and it neither fights nor
/// leads.
pub const WORKER_ROW: UnitTypeRow = UnitTypeRow {
    attack: Fix32::ZERO,
    armour: Fix32::ZERO,
    gather_rate: PLACEHOLDER_FULL_RATE,
    build_rate: PLACEHOLDER_FULL_RATE,
    carry_capacity: PLACEHOLDER_CARRY_CAPACITY,
    move_cost_scale: PLACEHOLDER_FULL_RATE,
    command_reach: 0,
    weather_reach: 0,
};

/// The soldier row. It fights, and it does nothing else.
pub const SOLDIER_ROW: UnitTypeRow = UnitTypeRow {
    attack: PLACEHOLDER_SOLDIER_ATTACK,
    armour: Fix32::ZERO,
    gather_rate: Fix32::ZERO,
    build_rate: Fix32::ZERO,
    carry_capacity: 0,
    move_cost_scale: PLACEHOLDER_FULL_RATE,
    command_reach: 0,
    weather_reach: 0,
};

/// The merchant row. It carries, and it does nothing else.
pub const MERCHANT_ROW: UnitTypeRow = UnitTypeRow {
    attack: Fix32::ZERO,
    armour: Fix32::ZERO,
    gather_rate: Fix32::ZERO,
    build_rate: Fix32::ZERO,
    carry_capacity: PLACEHOLDER_CARRY_CAPACITY,
    move_cost_scale: PLACEHOLDER_FULL_RATE,
    command_reach: 0,
    weather_reach: 0,
};

/// The leader row. It holds command reach and weather reach, and it does
/// nothing else.
pub const LEADER_ROW: UnitTypeRow = UnitTypeRow {
    attack: Fix32::ZERO,
    armour: Fix32::ZERO,
    gather_rate: Fix32::ZERO,
    build_rate: Fix32::ZERO,
    carry_capacity: 0,
    move_cost_scale: PLACEHOLDER_FULL_RATE,
    command_reach: PLACEHOLDER_REACH,
    weather_reach: PLACEHOLDER_REACH,
};

/// The default table that a world is built with.
///
/// It holds the worker, the soldier, the merchant, the leader and one open
/// row, in that order. Every row above the open row is zero, so a caller
/// that wants a sixth type writes it.[^1]
///
/// # References
///
/// [^1]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decision D4. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
pub const DEFAULT_UNIT_TYPE_TABLE: UnitTypeTable = {
    let mut rows = [UnitTypeRow::NONE; UNIT_TYPE_COUNT];
    rows[WORKER.index()] = WORKER_ROW;
    rows[SOLDIER.index()] = SOLDIER_ROW;
    rows[MERCHANT.index()] = MERCHANT_ROW;
    rows[LEADER.index()] = LEADER_ROW;
    rows[OPEN.index()] = UnitTypeRow::NONE;
    UnitTypeTable { rows }
};

/// The shared table that a unit type indexes.
///
/// The table is dense and its length never changes. A caller fills the rows
/// it wants and leaves the rest at zero.[^1]
///
/// # References
///
/// [^1]: ADR-0120, a unit carries a type, and the type is an index into a table the world is built with, decision D2. `docs/adrs/draft/adr-0120-a-unit-carries-a-type-that-indexes-a-table.md`
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct UnitTypeTable {
    rows: [UnitTypeRow; UNIT_TYPE_COUNT],
}

impl UnitTypeTable {
    /// Builds a table whose every row holds zero.
    ///
    /// A world built with this table holds units that can do nothing: no
    /// attack exceeds any armour, no unit gathers, and no unit builds.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            rows: [UnitTypeRow::NONE; UNIT_TYPE_COUNT],
        }
    }

    /// Returns every row, in type order.
    #[must_use]
    pub const fn rows(&self) -> &[UnitTypeRow; UNIT_TYPE_COUNT] {
        &self.rows
    }

    /// Returns one row of the table.
    #[must_use]
    pub const fn row(&self, unit_type: UnitTypeId) -> UnitTypeRow {
        self.rows[unit_type.index()]
    }

    /// Writes one row of the table.
    ///
    /// The caller gives the whole row. There is no partial form, because a
    /// caller that gave two columns would leave the rest at zero and would
    /// define a unit that can do nothing else without knowing it.[^1]
    ///
    /// # Errors
    ///
    /// Returns an error when the number names no row, or when a fixed-point
    /// column is below zero.
    ///
    /// # References
    ///
    /// [^1]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decision D5. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
    pub fn define(&mut self, unit_type: u8, row: UnitTypeRow) -> Result<(), UnitTypeError> {
        let Some(index) = UnitTypeId::from_u8(unit_type) else {
            return Err(UnitTypeError::TypeAboveCeiling(unit_type));
        };
        if let Some((column, value)) = row.column_below_zero() {
            return Err(UnitTypeError::ColumnBelowZero(column, value));
        }
        self.rows[index.index()] = row;
        Ok(())
    }

    /// Reports whether an attacker of one type reaches a defender of another.
    ///
    /// **This is the one statement of the penetration threshold.** The attack
    /// of the attacker must exceed the armour of the defender. An attack that
    /// equals the armour does not exceed it and does not reach.[^1]
    ///
    /// The pass calls this once for each ordered pair of types, before it
    /// aggregates anything, so a pair that does not reach contributes exactly
    /// zero however many attackers stand on the tile.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0122, an attacker whose attack does not exceed the defender's armour contributes exactly zero, decision D1. `docs/adrs/draft/adr-0122-an-attacker-below-the-armour-contributes-exactly-zero.md`
    #[must_use]
    pub const fn penetrates(&self, attacker: UnitTypeId, defender: UnitTypeId) -> bool {
        self.rows[attacker.index()].attack.0 > self.rows[defender.index()].armour.0
    }

    /// Absorbs the table into the state hash.
    ///
    /// The table decides what a later frame does, so the whole-world hash
    /// covers it. Two worlds that hold the same units and different tables
    /// must diverge at the next meeting.[^1]
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
    /// No row holds a fixed-point column below zero. The writer refuses one,
    /// and this is what fails when a value reaches the table by another path.
    #[must_use]
    pub fn check_invariants(&self) -> bool {
        self.rows
            .iter()
            .all(|row| row.column_below_zero().is_none())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A row that fights with the given attack and armour and does nothing
    /// else.
    const fn fighter(attack: Fix32, armour: Fix32) -> UnitTypeRow {
        UnitTypeRow {
            attack,
            armour,
            ..UnitTypeRow::NONE
        }
    }

    #[test]
    fn an_unfilled_table_reaches_nothing() {
        let table = UnitTypeTable::empty();
        for attacker in 0..UNIT_TYPE_COUNT as u8 {
            for defender in 0..UNIT_TYPE_COUNT as u8 {
                let attacker = UnitTypeId::from_u8(attacker).expect("the number names a row");
                let defender = UnitTypeId::from_u8(defender).expect("the number names a row");
                assert!(!table.penetrates(attacker, defender));
            }
        }
    }

    #[test]
    fn an_attack_that_equals_the_armour_does_not_reach() {
        let mut table = UnitTypeTable::empty();
        table
            .define(0, fighter(Fix32::from_int(4), Fix32::ZERO))
            .expect("the row is inside the table");
        table
            .define(1, fighter(Fix32::ZERO, Fix32::from_int(4)))
            .expect("the row is inside the table");
        let bowman = UnitTypeId(0);
        let tank = UnitTypeId(1);
        assert!(
            !table.penetrates(bowman, tank),
            "an attack that equals the armour must not exceed it"
        );
    }

    #[test]
    fn the_table_refuses_a_row_it_does_not_hold() {
        let mut table = UnitTypeTable::empty();
        let refused = table.define(UNIT_TYPE_COUNT as u8, fighter(Fix32::ONE, Fix32::ZERO));
        assert_eq!(
            refused,
            Err(UnitTypeError::TypeAboveCeiling(UNIT_TYPE_COUNT as u8))
        );
    }

    #[test]
    fn the_table_refuses_a_negative_value_in_any_fixed_point_column() {
        let mut table = UnitTypeTable::empty();
        assert_eq!(
            table.define(0, fighter(Fix32(-1), Fix32::ZERO)),
            Err(UnitTypeError::ColumnBelowZero("attack", Fix32(-1)))
        );
        assert_eq!(
            table.define(0, fighter(Fix32::ZERO, Fix32(-1))),
            Err(UnitTypeError::ColumnBelowZero("armour", Fix32(-1)))
        );
        let negative_rate = UnitTypeRow {
            gather_rate: Fix32(-1),
            ..UnitTypeRow::NONE
        };
        assert_eq!(
            table.define(0, negative_rate),
            Err(UnitTypeError::ColumnBelowZero("gather_rate", Fix32(-1)))
        );
        let negative_build = UnitTypeRow {
            build_rate: Fix32(-1),
            ..UnitTypeRow::NONE
        };
        assert_eq!(
            table.define(0, negative_build),
            Err(UnitTypeError::ColumnBelowZero("build_rate", Fix32(-1)))
        );
        let negative_move = UnitTypeRow {
            move_cost_scale: Fix32(-1),
            ..UnitTypeRow::NONE
        };
        assert_eq!(
            table.define(0, negative_move),
            Err(UnitTypeError::ColumnBelowZero("move_cost_scale", Fix32(-1)))
        );
        assert!(table.check_invariants());
    }

    #[test]
    fn the_column_names_and_the_column_reader_walk_the_same_row() {
        // The macro declares both from one list, so the two agree by
        // construction. This is what fails if the macro is replaced by a
        // hand-written pair.
        let row = UnitTypeRow {
            attack: Fix32(1),
            armour: Fix32(2),
            gather_rate: Fix32(3),
            build_rate: Fix32(4),
            carry_capacity: 5,
            move_cost_scale: Fix32(6),
            command_reach: 7,
            weather_reach: 8,
        };
        assert_eq!(row.columns(), [1, 2, 3, 4, 5, 6, 7, 8]);
        assert_eq!(
            UnitTypeRow::COLUMN_NAMES,
            [
                "attack",
                "armour",
                "gather_rate",
                "build_rate",
                "carry_capacity",
                "move_cost_scale",
                "command_reach",
                "weather_reach",
            ]
        );
    }

    #[test]
    fn the_default_table_holds_five_rows_and_zero_above_them() {
        let table = DEFAULT_UNIT_TYPE_TABLE;
        assert_eq!(table.row(WORKER), WORKER_ROW);
        assert_eq!(table.row(SOLDIER), SOLDIER_ROW);
        assert_eq!(table.row(MERCHANT), MERCHANT_ROW);
        assert_eq!(table.row(LEADER), LEADER_ROW);
        assert_eq!(table.row(OPEN), UnitTypeRow::NONE);
        for above in (OPEN.index() + 1)..UNIT_TYPE_COUNT {
            assert_eq!(table.rows()[above], UnitTypeRow::NONE);
        }
        assert!(table.check_invariants());
    }
}
