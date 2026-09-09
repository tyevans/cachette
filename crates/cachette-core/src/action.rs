//! One integer for one action, and the schema that declares how to read it.
//!
//! A learner plays one faction, and it needs the set of actions it may take
//! in a form a network can index.[^1] The engine therefore declares one
//! bounded table of actions for one world, and an action is one index into
//! that table.[^2]
//!
//! # The integer is a mixed radix over the positions a verb declares
//!
//! A verb declares its own argument positions, in order. The action integer
//! is a mixed radix over the positions of the verb it selects, and a verb
//! declares as many positions as it needs, including none.[^3] A caller
//! decodes an integer by arithmetic over the schema, and never by a table it
//! holds.
//!
//! The table lays the verbs out in the order the verb list states. The rows
//! of one verb are contiguous, and the first position of that verb is the
//! most significant digit of its index. A verb with no position holds one
//! row.
//!
//! # A verb whose content the engine resolves declares no position
//!
//! Five verbs of the enumeration name nothing at all: a board rewrite, a
//! negotiation step, a carrier assignment, a project send and a
//! founding.[^4] A crossing names something that the engine itself resolves
//! at the tick the action applies: it takes the tile the engine surveys for
//! that faction. **The schema gives it no position.** A position that
//! carried the tile would state a second time what the engine already
//! answers, and the engine's answer is the one that acts.[^4] [^5]
//!
//! Four verbs take an argument that no engine rule supplies. A gather order
//! names a resource kind, a build order names an upgrade category, a
//! relation move names another faction, and a queue order names a unit type.
//! Each of those is one candidate position.
//!
//! # One verb names a place
//!
//! A campaign declares one place position. The position carries a cell of the
//! egocentric frame that the observation reads, and it names the region the
//! engine resolves the objective within.[^10] It never carries the objective
//! itself, so the rule that forbids a second declaration of the engine's own
//! answer holds.[^4]
//!
//! **The first value of the place enumeration names the whole frame.** It
//! says that the engine resolves over every cell, which is the answer the
//! verb gave before the position existed. The built-in controller's own
//! choice therefore lands on that row, and the verb set does not move.[^10]
//!
//! A crossing and a settling also act on a place, and neither declares a
//! position. Both take their target from a keyed sample of the world, and a
//! place position may not narrow a draw.[^10]
//!
//! **No verb of the enumeration carries a quantity, so no verb declares a
//! bucket position.**[^6] The balance register therefore holds no bucket
//! edge for this table, and this module states none.
//!
//! # Every bound comes from the world parameters
//!
//! The bound of a position is a fixed enumeration count, the cell count of
//! the egocentric frame, or the faction count of the world. **No bound
//! follows the population, and no bound follows the world extent.**[^3]
//! [^10] A faction that holds a million units reads a table of the same
//! length as a faction that holds none, and a world of four thousand tiles
//! declares the same table as a world of sixteen million. The table therefore stays small, and the answer that says
//! which of its rows are legal stays small with it.
//!
//! # The no-op row
//!
//! Row zero is the no-op. It takes no position, it changes nothing, and it
//! is always legal, so the legality answer is never empty and a learner
//! never learns legality by trial.[^7]
//!
//! # Determinism
//!
//! The schema is a pure function of the world parameters. The encoding and
//! the decoding are integer arithmetic. Nothing in this module draws, reads
//! a thread, or reads a completion order.[^8] No item in this module uses a
//! floating-point type.[^9]
//!
//! # References
//!
//! [^1]: PRD-0056, a learner plays one faction against the controllers. `docs/product/accepted/prd-0056-a-learner-plays-one-faction-against-the-controllers.md`
//! [^2]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decision D4. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
//! [^3]: ADR-0176, an action integer is a mixed radix over the argument positions each verb declares, decision D1. `docs/adrs/accepted/adr-0176-an-action-integer-is-a-mixed-radix-over-the-positions-a-verb-declares.md`
//! [^4]: ADR-0176, an action integer is a mixed radix over the argument positions each verb declares, decision D2. `docs/adrs/accepted/adr-0176-an-action-integer-is-a-mixed-radix-over-the-positions-a-verb-declares.md`
//! [^5]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
//! [^6]: ADR-0176, an action integer is a mixed radix over the argument positions each verb declares, decision D3. `docs/adrs/accepted/adr-0176-an-action-integer-is-a-mixed-radix-over-the-positions-a-verb-declares.md`
//! [^7]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decision D5. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
//! [^8]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
//! [^9]: ADR-0002, state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
//! [^10]: ADR-0199, a verb names a place by a cell of the egocentric frame the observation publishes, decisions D1, D2, D4 and D5. `docs/adrs/draft/adr-0199-a-verb-names-a-place-by-a-cell-of-the-egocentric-frame.md`

use crate::obs_ring::RING_STACK_CELLS;
use crate::resource::RESOURCE_KIND_COUNT;
use crate::unit_type::UNIT_TYPE_COUNT;
use crate::upgrade::UPGRADE_CATEGORY_COUNT;

/// The place value that names the whole egocentric frame.
///
/// The first value of a place enumeration names no cell. It says that the
/// engine resolves the place over every cell, which is the answer the verb
/// gave before the position existed. The built-in controller's own choice
/// therefore lands on this row.[^1]
///
/// # References
///
/// [^1]: ADR-0199, a verb names a place by a cell of the egocentric frame the observation publishes, decision D2. `docs/adrs/draft/adr-0199-a-verb-names-a-place-by-a-cell-of-the-egocentric-frame.md`
pub const PLACE_ANYWHERE: u32 = 0;

/// The values that a place position holds.
///
/// The count is the cells of the egocentric frame, plus the whole-frame
/// value. **The ring module derives the cell count from the sector rule of
/// the frame, so this bound follows nothing about the world.**[^1] A place
/// value above the whole-frame value names cell `value - 1` of that frame.
///
/// # References
///
/// [^1]: ADR-0199, a verb names a place by a cell of the egocentric frame the observation publishes, decision D4. `docs/adrs/draft/adr-0199-a-verb-names-a-place-by-a-cell-of-the-egocentric-frame.md`
pub const PLACE_COUNT: u32 = RING_STACK_CELLS + 1;

/// Returns the cell of the egocentric frame that one place value names, and
/// nothing when the value names the whole frame.
///
/// **This is the one statement of the mapping.** The legality answer and the
/// verb both read it, so neither holds a second copy of the offset.[^1]
///
/// # References
///
/// [^1]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
#[must_use]
pub const fn place_cell(place: u32) -> Option<u32> {
    if place == PLACE_ANYWHERE || place >= PLACE_COUNT {
        None
    } else {
        Some(place - 1)
    }
}

/// The version of the action table layout.
///
/// The table is a mixed radix, so a new verb or a new bound moves every row
/// above it. That renumbering changes the meaning of a stored weight file,
/// so the engine carries this integer beside the schema.[^1]
///
/// Raise this number whenever the verb list, a position list or a bound rule
/// changes.
///
/// # References
///
/// [^1]: ADR-0176, an action integer is a mixed radix over the argument positions each verb declares, decision D4. `docs/adrs/accepted/adr-0176-an-action-integer-is-a-mixed-radix-over-the-positions-a-verb-declares.md`
pub const ACTION_VERSION: u32 = 2;

/// One verb of the action table.
///
/// **The verb set is the set the built-in controller's choice enumeration
/// holds, plus the no-op.**[^1] A choice added to that enumeration adds a
/// verb here, and it renumbers every row above the verb it joins.
///
/// # References
///
/// [^1]: ADR-0176, an action integer is a mixed radix over the argument positions each verb declares, decision D4. `docs/adrs/accepted/adr-0176-an-action-integer-is-a-mixed-radix-over-the-positions-a-verb-declares.md`
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Verb {
    /// Do nothing this tick. This verb is always legal.
    NoOp,
    /// Order the units of the faction to gather one resource kind.
    Gather,
    /// Order the units of the faction to build one upgrade category.
    Build,
    /// Move the relation of the faction toward another faction by one step.
    Relation,
    /// Raise a campaign against the objective the engine resolves.
    Campaign,
    /// Rewrite the whole board of the faction from its site economies.
    Advertise,
    /// Take one negotiation step against the faction the engine resolves.
    Trade,
    /// Assign and release the carriers of the contracts of the faction.
    Carry,
    /// Send the idle units of the faction to the projects its plan zones.
    Project,
    /// Put one entry of one unit type into the queue of a site.
    Queue,
    /// Send the water-crossing units at the tile the engine surveys.
    Cross,
    /// Found a city from every settler that stands on ground a city may take.
    Settle,
}

/// What one argument position of a verb names.
///
/// Every position of this table is a candidate position. It names the kind
/// of thing its list holds and it carries a bound.[^1] **No verb of the
/// enumeration takes a quantity, so no bucket position exists.**[^2]
///
/// # References
///
/// [^1]: ADR-0176, an action integer is a mixed radix over the argument positions each verb declares, decision D1. `docs/adrs/accepted/adr-0176-an-action-integer-is-a-mixed-radix-over-the-positions-a-verb-declares.md`
/// [^2]: ADR-0176, an action integer is a mixed radix over the argument positions each verb declares, decision D3. `docs/adrs/accepted/adr-0176-an-action-integer-is-a-mixed-radix-over-the-positions-a-verb-declares.md`
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CandidateKind {
    /// A resource kind, as the resource module numbers them.
    Resource,
    /// An upgrade category, as the upgrade module numbers them.
    Category,
    /// A faction of the world, by faction number.
    Faction,
    /// A row of the unit type table.
    UnitType,
    /// A cell of the egocentric frame the observation publishes, offset by
    /// the whole-frame value.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0199, a verb names a place by a cell of the egocentric frame the observation publishes, decisions D1 and D2. `docs/adrs/draft/adr-0199-a-verb-names-a-place-by-a-cell-of-the-egocentric-frame.md`
    Place,
}

impl CandidateKind {
    /// Returns the name of the candidate kind.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Resource => "resource",
            Self::Category => "category",
            Self::Faction => "faction",
            Self::UnitType => "unit_type",
            Self::Place => "place",
        }
    }

    /// Returns the ceiling on the candidate list, for one world shape.
    ///
    /// Three of the five are fixed enumeration counts. One is the cell count
    /// of the egocentric frame, which the ring module derives from the
    /// sector rule of that frame. The last is the faction count of the
    /// world, which the world fixes at construction.
    ///
    /// **Not one of them follows the population, and not one of them follows
    /// the world extent.**[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0199, a verb names a place by a cell of the egocentric frame the observation publishes, decision D4. `docs/adrs/draft/adr-0199-a-verb-names-a-place-by-a-cell-of-the-egocentric-frame.md`
    const fn bound(self, shape: ActionShape) -> u32 {
        match self {
            Self::Resource => RESOURCE_KIND_COUNT as u32,
            Self::Category => UPGRADE_CATEGORY_COUNT as u32,
            Self::Faction => shape.faction_count,
            Self::UnitType => UNIT_TYPE_COUNT as u32,
            Self::Place => PLACE_COUNT,
        }
    }
}

impl Verb {
    /// Every verb, in the order the table holds them.
    ///
    /// **This list is the whole verb set.** The schema derives every row
    /// from it, and the engine dispatches through a match that the compiler
    /// checks for coverage. A verb added here reaches both without a second
    /// edit.[^1]
    ///
    /// # References
    ///
    /// [^1]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
    pub const ALL: [Self; 12] = [
        Self::NoOp,
        Self::Gather,
        Self::Build,
        Self::Relation,
        Self::Campaign,
        Self::Advertise,
        Self::Trade,
        Self::Carry,
        Self::Project,
        Self::Queue,
        Self::Cross,
        Self::Settle,
    ];

    /// Returns the name of the verb.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::NoOp => "no_op",
            Self::Gather => "gather",
            Self::Build => "build",
            Self::Relation => "relation",
            Self::Campaign => "campaign",
            Self::Advertise => "advertise",
            Self::Trade => "trade",
            Self::Carry => "carry",
            Self::Project => "project",
            Self::Queue => "queue",
            Self::Cross => "cross",
            Self::Settle => "settle",
        }
    }

    /// Returns the argument positions the verb declares, in order.
    ///
    /// A verb whose content the engine resolves at the tick the action
    /// applies declares none.[^1]
    ///
    /// A campaign declares one place position. The position names the region
    /// the engine resolves the objective within, and never the objective
    /// itself, so the engine still resolves the content.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0176, an action integer is a mixed radix over the argument positions each verb declares, decision D2. `docs/adrs/accepted/adr-0176-an-action-integer-is-a-mixed-radix-over-the-positions-a-verb-declares.md`
    /// [^2]: ADR-0199, a verb names a place by a cell of the egocentric frame the observation publishes, decisions D1 and D2. `docs/adrs/draft/adr-0199-a-verb-names-a-place-by-a-cell-of-the-egocentric-frame.md`
    #[must_use]
    pub const fn positions(self) -> &'static [CandidateKind] {
        match self {
            Self::Gather => &[CandidateKind::Resource],
            Self::Build => &[CandidateKind::Category],
            Self::Relation => &[CandidateKind::Faction],
            Self::Queue => &[CandidateKind::UnitType],
            Self::Campaign => &[CandidateKind::Place],
            Self::NoOp
            | Self::Advertise
            | Self::Trade
            | Self::Carry
            | Self::Project
            | Self::Cross
            | Self::Settle => &[],
        }
    }
}

/// The world parameters that fix every bound of the table.
///
/// **The faction count is the count the world fixes at construction.** It
/// does not change when a faction gains or loses a unit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ActionShape {
    /// The factions the world holds.
    pub faction_count: u32,
}

/// One argument position of one verb, as the schema states it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PositionRow {
    /// The kind of thing the candidate list of the position holds.
    pub candidate: CandidateKind,
    /// The ceiling on that candidate list.
    pub bound: u32,
    /// How far one step of this position moves the action integer.
    ///
    /// The stride is the product of the bounds of the positions after this
    /// one, so the first position of a verb is its most significant digit.
    pub stride: u32,
}

/// One verb of the action schema.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerbRow {
    /// The verb the row describes.
    pub verb: Verb,
    /// The action integer of the first row of this verb.
    pub first: u32,
    /// How many rows of the table this verb holds.
    ///
    /// The count is the product of the bounds of its positions. A verb with
    /// no position holds one row.
    pub rows: u32,
    /// The argument positions the verb declares, in order.
    pub positions: Vec<PositionRow>,
}

impl VerbRow {
    /// Returns the name of the verb.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        self.verb.name()
    }
}

/// The declared layout of the action table of one world.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActionSchema {
    version: u32,
    length: u32,
    rows: Vec<VerbRow>,
}

impl ActionSchema {
    /// Builds the schema of one world shape.
    #[must_use]
    pub fn of(shape: ActionShape) -> Self {
        let mut rows = Vec::with_capacity(Verb::ALL.len());
        let mut first = 0u32;
        for verb in Verb::ALL {
            let declared = verb.positions();
            // The stride of a position is the product of the bounds after
            // it, so the list is built from the last position backwards.
            let mut positions: Vec<PositionRow> = Vec::with_capacity(declared.len());
            let mut stride = 1u32;
            for candidate in declared.iter().rev() {
                let bound = candidate.bound(shape);
                positions.push(PositionRow {
                    candidate: *candidate,
                    bound,
                    stride,
                });
                stride = stride.saturating_mul(bound);
            }
            positions.reverse();
            rows.push(VerbRow {
                verb,
                first,
                rows: stride,
                positions,
            });
            first = first.saturating_add(stride);
        }
        Self {
            version: ACTION_VERSION,
            length: first,
            rows,
        }
    }

    /// Returns the version of the layout.
    #[must_use]
    pub const fn version(&self) -> u32 {
        self.version
    }

    /// Returns how many rows the whole table holds.
    #[must_use]
    pub const fn length(&self) -> u32 {
        self.length
    }

    /// Returns one row for each verb, in the order the table holds them.
    #[must_use]
    pub fn rows(&self) -> &[VerbRow] {
        &self.rows
    }

    /// Returns the row of one verb.
    #[must_use]
    pub fn row(&self, verb: Verb) -> Option<&VerbRow> {
        self.rows.iter().find(|row| row.verb == verb)
    }

    /// Returns the row of one verb, by name.
    #[must_use]
    pub fn row_named(&self, name: &str) -> Option<&VerbRow> {
        self.rows.iter().find(|row| row.name() == name)
    }

    /// Returns the action integer of one verb and its arguments.
    ///
    /// The arguments are the values of the positions the verb declares, in
    /// the order it declares them. Returns `None` when the count of the
    /// arguments is not the count of the positions, or when an argument is
    /// at or above the bound of its position.
    #[must_use]
    pub fn encode(&self, verb: Verb, arguments: &[u32]) -> Option<u32> {
        let row = self.row(verb)?;
        if arguments.len() != row.positions.len() {
            return None;
        }
        let mut action = row.first;
        for (position, argument) in row.positions.iter().zip(arguments) {
            if *argument >= position.bound {
                return None;
            }
            action = action.checked_add(argument.checked_mul(position.stride)?)?;
        }
        Some(action)
    }

    /// Returns the verb of one action integer, and nothing when the
    /// integer is at or above the length of the table.
    #[must_use]
    pub fn verb_of(&self, action: u32) -> Option<Verb> {
        self.rows
            .iter()
            .find(|row| action >= row.first && action - row.first < row.rows)
            .map(|row| row.verb)
    }

    /// Returns the verb and the arguments of one action integer.
    ///
    /// The decoding is arithmetic over this schema alone. Returns `None`
    /// when the integer is at or above the length of the table.
    #[must_use]
    pub fn decode(&self, action: u32) -> Option<(Verb, Vec<u32>)> {
        let row = self
            .rows
            .iter()
            .find(|row| action >= row.first && action - row.first < row.rows)?;
        let mut rest = action - row.first;
        let mut arguments = Vec::with_capacity(row.positions.len());
        for position in &row.positions {
            arguments.push(rest / position.stride);
            rest %= position.stride;
        }
        Some((row.verb, arguments))
    }
}
