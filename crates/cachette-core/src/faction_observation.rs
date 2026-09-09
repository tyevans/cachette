//! One flat array for one faction, and the schema that declares its layout.
//!
//! A learner reads one array on every decision, and it trains a function of
//! that array's layout. The engine therefore returns one flat array of signed
//! integers for one faction, and one schema that says where every field of it
//! starts.[^1] [^2]
//!
//! # The width follows nothing in the world
//!
//! **Every field of this layout holds a fixed number of positions.** The
//! width is one number for every world shape and every faction count, so a
//! policy trained on a small world reads a large one.
//!
//! The layout it replaces did not have that property. A field with one
//! position for each faction multiplied by the faction count, and a field
//! with one position for each cell of the block lattice multiplied by the
//! cell count. The width was a function of the width, the height and the
//! faction count, and a policy trained against one triple could not read
//! another. On the world the project trains on most, the lattice held one
//! cell, so the policy read no spatial information at all.[^3]
//!
//! # No slot names a seat
//!
//! **Nothing here is indexed by a faction.** A block indexed by seat teaches
//! a policy a seat number, and a league seats one policy in one seat for one
//! game and in another seat for the next.[^4] The standing of the rivals
//! arrives as order statistics over the whole field: a share of the total, a
//! rank, a gap to the strongest rival, a gap to the median rival, the leader
//! share, a concentration, and a confidence.[^5]
//!
//! # Every value is a share, a signed relation, or a compressed magnitude
//!
//! The array publishes no raw count and no unbounded fixed-point total. A
//! share divides by a denominator that the field doc names. A signed relation
//! divides a difference by the sum of the two magnitudes. A compressed
//! magnitude maps a quantity through an integer base-two logarithm against a
//! fixed bit cap.[^6]
//!
//! The four functions live in the arithmetic module, so no block of this file
//! states arithmetic of its own.[^7] Every division truncates toward zero.
//!
//! **The schema names the form of each field, and it publishes what an
//! inversion of that form needs.** A start, a width and a pair of bounds say
//! where a value sits and how far it reaches. They do not say what the value
//! means. A reader that wanted the population count behind a compressed
//! magnitude therefore had to restate the compression, and a rule stated
//! twice fails silently when one copy moves.[^15]
//!
//! The form travels with the field. The field list gives one kind for each
//! field, the writer takes its arithmetic from that kind, and the schema
//! publishes that same kind. A compressed magnitude publishes its base, its
//! offset and its divisor, so a reader recovers the quantity. A share and a
//! signed relation publish the denominator convention and say that they are
//! not invertible, because neither value carries the denominator it divided
//! by.[^6]
//!
//! # What a reserved field means
//!
//! **A reserved field reads zero in every position, and the schema says so.**
//! Its declared bounds are zero and zero. A field is reserved for one of two
//! reasons: the engine keeps no aggregate that answers it, or the value needs
//! a reading of a past frame that the engine does not store. A reserved field
//! is not a zero that states a real quantity of zero.
//!
//! **The engine carries a window of events and no window of readings.** A
//! decayed history counts the events of each kind as they arrive, so a field
//! that names a count over a window is answerable and a field that names the
//! change of a snapshot quantity is not. The doc of each reserved field
//! states which case it is, and it names the history where the history
//! answers.
//!
//! A reserved field holds its declared positions, so every later field starts
//! where the design puts it and a builder fills the reserve without moving
//! anything.[^5]
//!
//! The three spatial blocks are built. The egocentric ring stack, the
//! frontier by sector and the entity tokens each take their width from the
//! module that fills them.
//!
//! # What the array says about a place the faction has never seen
//!
//! The cost of a read follows what the faction observed and never the
//! world.[^8] Every fog-dependent quantity comes from one pass over the
//! ground the faction has seen, and a block the faction has never seen a tile
//! of answers from the block form of the fog layer at no tile cost.
//!
//! An estimate of a rival quantity therefore states less than the truth. The
//! confidence statistic of each power quantity says how much of the estimate
//! the faction sees this frame, so a reader tells an unobserved estimate from
//! a real zero.[^5]
//!
//! **A faction reads its own held tile count from the running total and not
//! through the fog of this frame.** The count the holding keeps is the whole
//! count, and a fog-scoped count of the same thing flickers with sight.[^10]
//!
//! # The schema states the structure, and not only the position
//!
//! A policy that exploits the shape of the observation must know that shape.
//! A start and a width do not carry it. One block of channels over cells
//! holds the same positions as another block of cells over channels, and
//! nothing in the width separates the two.[^11]
//!
//! The schema therefore states more than the place of each field, and it
//! derives every entry from the modules that fill the blocks. No file outside
//! this module states a position, a width, a count or an order.[^12]
//!
//! **A spatial field names its space.** The ring stack marks its field, so a
//! reader tells it from a scalar field by the mark and not by the name.
//!
//! **A field of channels names its channels.** The module that fills a block
//! owns the channel list, and the schema publishes that list.
//!
//! **The schema states the cells of each ring.** A cell index says nothing
//! about its ring and its sector on its own.
//!
//! **The schema states which axis runs first.** Every block of this layout
//! stores the channels of one place next to each other.
//!
//! **The schema names the channel that gates a cell.** A cell of the frame
//! that lies outside the world reads zero in every channel. That zero is an
//! absent value and not a quantity of zero, and the gate channel states the
//! difference.[^13]
//!
//! **Each token set is one field.** The sets hold different channel counts,
//! so one field cannot state the shape of every set.[^14]
//!
//! # Determinism
//!
//! One pass builds the array, on the calling thread. It visits the observed
//! cells in ascending cell order, the seated factions in ascending seat
//! order, the settlements in ascending slot order, and the board rows in
//! ascending row order. Nothing reads a thread and nothing reads a completion
//! order.[^9]
//!
//! # References
//!
//! [^1]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decision D1. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
//! [^2]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decision D2. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
//! [^3]: Findings register, FND-670. `docs/FINDINGS.md`
//! [^4]: Findings register, FND-647. `docs/FINDINGS.md`
//! [^5]: Research report 42, what a policy should be able to see, sections 6 and 9. `docs/research/reports/42-what-a-policy-should-be-able-to-see.md`
//! [^6]: Research report 42, what a policy should be able to see, sections 4 and 8. `docs/research/reports/42-what-a-policy-should-be-able-to-see.md`
//! [^7]: ADR-0002, simulated and aggregated state holds no floating point number, decision D2. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
//! [^8]: ADR-0059, fog storage grows with observed area, not with world area, decision D2. `docs/adrs/accepted/adr-0059-fog-storage-grows-with-observed-area.md`
//! [^9]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
//! [^10]: Findings register, FND-671. `docs/FINDINGS.md`
//! [^11]: The layout reader of the control plane. `python/cachette/learn/layout.py`
//! [^12]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decision D1. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
//! [^13]: ADR-0195, the observation of a faction is a fixed-width scale-free table, decision D8. `docs/adrs/draft/adr-0195-the-observation-of-a-faction-is-a-fixed-width-scale-free-table.md`
//! [^14]: ADR-0195, the observation of a faction is a fixed-width scale-free table, decision D4. `docs/adrs/draft/adr-0195-the-observation-of-a-faction-is-a-fixed-width-scale-free-table.md`
//! [^15]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`

use crate::action::{CandidateKind, Verb};
use crate::event_layout::ColumnKind;
use crate::event_memory::{Decay, MemoryKind, BLAMED_KIND_COUNT, KIND_COUNT};
use crate::faction_memory_observation as memory;
use crate::faction_view::{BlockMask, FactionViewError};
use crate::hex::Axial;
use crate::holding::Holder;
use crate::obs_frontier::Frontier;
use crate::obs_ring::ring_cell_counts;
use crate::obs_ring_stack::RingStack;
use crate::obs_ring_stack::RING_SPACE;
use crate::obs_token::{distance_share, EntityTokens, RivalPowers, TokenSet, TOKEN_SPACE};
use crate::position::WORK_COMMODITY;
use crate::resource::ResourceKind;
use crate::sim_math;
use crate::site::{CommodityId, COMMODITY_COUNT};
use crate::trade::TRADE_BOUND;
use crate::types::{Accum, Entity, FactionId, Fix32, TileIdx};
use crate::unit_type::UNIT_TYPE_COUNT;
use crate::upgrade::{self, UpgradeCategory};
use crate::weather::SEASON_PERIOD_TICKS;
use crate::world::World;

/// The version of the observation layout.
///
/// A field added, removed, relengthened or rebounded changes the meaning of a
/// stored weight file, so the engine carries this integer beside the schema.
/// A learner that loads a policy under another version must stop.[^1]
///
/// Raise this number whenever the field list, a length rule or a bound rule
/// changes.
///
/// **Do not raise it for an addition that describes the layout it already
/// had.** A stored weight file names a position by its index, so a weight is
/// wrong only when a position moves or a value changes. A revision that adds
/// a descriptive entry to the schema moves nothing, and a raise refuses every
/// stored file for nothing. This has cost a set of weights once.[^2]
///
/// # References
///
/// [^1]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, the consequences. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
/// [^2]: Findings register, FND-689. `docs/FINDINGS.md`
pub const OBSERVATION_VERSION: u32 = 7;

/// The good classes that the layout carries.
///
/// The taxonomy is fixed, so a new good does not change the width. The engine
/// maps a resource kind onto a class through the work commodity table, which
/// is the one declaration of that mapping. A class the engine holds no
/// commodity for reads zero.
pub const GOOD_CLASS_COUNT: u32 = 8;

/// The unit type classes that the layout carries.
///
/// The unit type table holds this many rows, so a class is a row of it and
/// the mapping needs no table of its own.
pub const UNIT_CLASS_COUNT: u32 = UNIT_TYPE_COUNT as u32;

/// The upgrade classes that the layout carries.
///
/// The taxonomy is fixed at eight. The upgrade table holds fewer categories
/// than that today, and a class above the last category reads zero.
pub const UPGRADE_CLASS_COUNT: u32 = 8;

/// The order statistics that the layout publishes for one power quantity.
///
/// They are the own share of the total, the own rank as a share, the gap to
/// the strongest rival, the gap to the median rival, the leader share of the
/// total, the concentration, and the observation confidence.[^1]
///
/// # References
///
/// [^1]: Research report 42, what a policy should be able to see, section 6.3. `docs/research/reports/42-what-a-policy-should-be-able-to-see.md`
pub const POWER_STATISTIC_COUNT: u32 = 7;

/// The statistics that the layout publishes for one good class of the board.
///
/// They are the best price to buy, the best price to sell, the spread between
/// the two, the depth offered, and the price change over a window. The last
/// one reads zero, because the engine carries no window.
pub const BOARD_STATISTIC_COUNT: u32 = 5;

/// The widths of the three spatial blocks, taken from the modules that build
/// them.
///
/// The field list needs each width to derive the start of every later block.
/// The module that fills a block is the one place that decides how wide it
/// is, so this file takes the width rather than states it. A width stated
/// twice is the defect shape this project names first, and neither copy
/// would fail when they disagreed.[^1]
///
/// # References
///
/// [^1]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
pub(crate) use crate::obs_frontier::FRONTIER_SLOTS;
pub(crate) use crate::obs_ring::{RING_STACK_CELLS, RING_STACK_CHANNELS};

/// The elements of the objective weight vector that the layout carries.
///
/// A policy reads the weight vector as an input, so one policy serves every
/// play style and the style becomes an input rather than a second set of
/// weights.[^1]
///
/// # References
///
/// [^1]: Research report 42, what a policy should be able to see, section 10.3. `docs/research/reports/42-what-a-policy-should-be-able-to-see.md`
pub const OBJECTIVE_WEIGHT_COUNT: u32 = 12;

/// The positions the layout holds back for a later signal.
///
/// A width change invalidates every trained policy and every stored
/// checkpoint, so the layout carries spare positions that read zero until a
/// revision claims them.
///
/// **A revision that claims a position takes it from here and holds the
/// length.** A field added anywhere else moves the start of every field after
/// it, and that refuses every stored weight file.
///
/// Four revisions claimed positions from it. The event history gained two
/// kinds, which widened five fields of the memory block. The domination path
/// gained two counts at the end of the layout. The founding chain gained the
/// settler count. The campaign path gained the objective it would march on.
/// The commit message of each states what it took.
pub const LAYOUT_RESERVE: u32 = 13;

/// How a reader reads every position of one field.
///
/// **The kind fixes the bounds, and no field states a bound of its own.** A
/// share lies between zero and one. A signed relation and a compressed
/// magnitude lie between minus one and one. A statistic is a group of shares
/// and signed relations over one quantity, and it lies between minus one and
/// one. A reserved field reads zero.
///
/// One unit is 65536, because the fixed-point scale of this project is
/// Q16.16.[^1]
///
/// # References
///
/// [^1]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ValueKind {
    /// A part of a named whole, from zero to one.
    Share,
    /// A signed comparison of two magnitudes, from minus one to one.
    Relation,
    /// A quantity compressed through a logarithm, from minus one to one.
    Magnitude,
    /// A group of shares and signed relations over one quantity.
    Statistic,
    /// A field that reads zero until a revision claims it.
    Reserved,
}

/// The value form of one kind, and the parameters an inversion of it needs.
///
/// **A reader outside the engine inverts a published value through this row
/// and never through a rule of its own.** A compressed magnitude hides the
/// quantity it came from, so a reader that wanted the count had to restate
/// the compression. That second statement of an engine rule is the defect
/// shape this project names first, and nothing fails when the two copies
/// disagree.[^1]
///
/// The row carries numbers and never a formula, so a caller acts on it
/// without parsing text.
///
/// # References
///
/// [^1]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ValueForm {
    /// The name a reader acts on.
    pub name: &'static str,
    /// The lowest value a position of this form may hold.
    pub low: i64,
    /// The highest value a position of this form may hold.
    pub high: i64,
    /// The integer that stands for one unit of the fixed-point scale.
    pub unit: i64,
    /// Whether every position of a field of this form holds this one form.
    ///
    /// A form that groups several forms over one quantity answers false. A
    /// reader that meets a false answer must read the channel and not the
    /// field.
    pub uniform: bool,
    /// Whether a reader recovers the quantity from the value and this row
    /// alone.
    pub invertible: bool,
    /// What the form divides by, or nothing where it divides by nothing.
    ///
    /// A share divides by a denominator that the doc of each field names, and
    /// the schema does not carry that denominator. A signed relation divides
    /// by the sum of the two magnitudes, which the value itself does not
    /// carry either. Neither form is therefore invertible.
    pub denominator: Option<&'static str>,
    /// The base of the logarithm the form takes, or nothing.
    pub log_base: Option<u32>,
    /// The amount the form adds before it takes the logarithm, or nothing.
    pub log_offset: Option<i64>,
    /// The bit width the form divides the logarithm by, or nothing.
    pub divisor_bits: Option<u32>,
}

/// The denominator a share divides by.
///
/// The doc of each field names it, and the schema does not carry it. A reader
/// that meets this answer knows the value is a part of a whole and that the
/// whole is not published.
const DENOMINATOR_PER_FIELD: &str = "per_field";

/// The denominator a signed relation divides by.
const DENOMINATOR_SUM_OF_MAGNITUDES: &str = "sum_of_magnitudes";

impl ValueKind {
    /// Every value form, in one order.
    pub const ALL: &'static [Self] = &[
        Self::Share,
        Self::Relation,
        Self::Magnitude,
        Self::Statistic,
        Self::Reserved,
    ];

    /// Returns the name a reader outside the engine acts on.
    #[must_use]
    pub const fn form_name(self) -> &'static str {
        match self {
            Self::Share => "share",
            Self::Relation => "relation",
            Self::Magnitude => "magnitude",
            Self::Statistic => "statistic",
            Self::Reserved => "reserved",
        }
    }

    /// Returns the form of this kind, with the parameters an inversion needs.
    ///
    /// **Every number here comes from the arithmetic module that writes the
    /// value.** This file states no base, no offset and no divisor of its
    /// own, so a change to the compression reaches the published form without
    /// a second edit.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0002, simulated and aggregated state holds no floating point number, decision D2. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
    #[must_use]
    pub const fn form(self) -> ValueForm {
        let (low, high) = self.bounds();
        let unit = Fix32::ONE.0 as i64;
        let plain = ValueForm {
            name: self.form_name(),
            low,
            high,
            unit,
            uniform: true,
            invertible: false,
            denominator: None,
            log_base: None,
            log_offset: None,
            divisor_bits: None,
        };
        match self {
            Self::Share => ValueForm {
                denominator: Some(DENOMINATOR_PER_FIELD),
                ..plain
            },
            Self::Relation => ValueForm {
                denominator: Some(DENOMINATOR_SUM_OF_MAGNITUDES),
                ..plain
            },
            Self::Magnitude => ValueForm {
                invertible: true,
                log_base: Some(sim_math::MAGNITUDE_LOG_BASE),
                log_offset: Some(sim_math::MAGNITUDE_LOG_OFFSET as i64),
                divisor_bits: Some(sim_math::MAGNITUDE_CAP_BITS),
                ..plain
            },
            Self::Statistic => ValueForm {
                uniform: false,
                ..plain
            },
            Self::Reserved => plain,
        }
    }

    /// Returns the lowest and the highest value a position of this kind may
    /// hold.
    #[must_use]
    pub const fn bounds(self) -> (i64, i64) {
        let one = Fix32::ONE.0 as i64;
        match self {
            Self::Share => (0, one),
            Self::Relation | Self::Magnitude | Self::Statistic => (-one, one),
            Self::Reserved => (0, 0),
        }
    }

    /// Reports whether a field of this kind reads zero everywhere.
    #[must_use]
    pub const fn is_reserved(self) -> bool {
        matches!(self, Self::Reserved)
    }
}

/// Declares the whole field list once.
///
/// The macro derives the enumeration, the ordered list, the name of each
/// field, the positions it holds and the kind of every position from one
/// table. A field added to the table reaches all four without a second
/// edit.[^1]
///
/// # References
///
/// [^1]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
macro_rules! declare_observation_fields {
    ( $( $(#[$note:meta])* $variant:ident => $name:expr, $positions:expr, $kind:ident ; )* ) => {
        /// One field of the observation array.
        ///
        /// **This list is the whole layout.** The schema derives every start
        /// from it, and the writer fills the array through a match that the
        /// compiler checks for coverage.
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub enum ObsField {
            $( $(#[$note])* $variant, )*
        }

        impl ObsField {
            /// Every field, in the order the array holds them.
            pub const ALL: &'static [Self] = &[ $( Self::$variant, )* ];

            /// Returns the name of the field.
            #[must_use]
            pub const fn name(self) -> &'static str {
                match self { $( Self::$variant => $name, )* }
            }

            /// Returns how many positions the field holds.
            ///
            /// **No answer here follows the world shape, the faction count or
            /// the population.** Every answer is a compile-time constant.
            #[must_use]
            pub const fn positions(self) -> u32 {
                match self { $( Self::$variant => $positions, )* }
            }

            /// Returns how a reader reads every position of the field.
            #[must_use]
            pub const fn value_kind(self) -> ValueKind {
                match self { $( Self::$variant => ValueKind::$kind, )* }
            }
        }
    };
}

declare_observation_fields! {
    /// Block A. The tiles the faction holds, from the running total the
    /// holding keeps.
    HeldTiles => "held_tiles", 1, Magnitude;
    /// The held tiles over the observed tiles whose ground admits a unit.
    HeldShareObserved => "held_share_observed", 1, Share;
    /// The held tiles over the world tiles whose ground admits a unit.
    HeldShareWorld => "held_share_world", 1, Share;
    /// The units of the faction that are alive.
    LiveUnits => "live_units", 1, Magnitude;
    /// The live units over eight times the held tiles.
    UnitsForEachHeldTile => "units_for_each_held_tile", 1, Share;
    /// The live settlements of the faction.
    Settlements => "settlements", 1, Magnitude;
    /// The residents of the settlements of the faction.
    ///
    /// **A resident is a unit that a settlement is the home of.** The engine
    /// holds no person apart from a unit. A settlement counts the units whose
    /// home column names it, and this field sums that count over the
    /// settlements of the faction.[^1]
    ///
    /// **This field therefore equals the live unit count whenever every unit
    /// of the faction is homed at a settlement it holds**, which is the
    /// ordinary case. It falls below the live unit count for a unit that is
    /// homed nowhere, and for a unit whose home the faction has lost. A
    /// reader that wants a quantity of people distinct from an army reads
    /// nothing of the sort here, and a blocker holds that question.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0157, a site's free places are its built housing less the residents the engine counts, decision D2. `docs/adrs/accepted/adr-0157-a-sites-free-places-are-its-built-housing-less-the-residents-the-engine-counts.md`
    /// [^2]: Blockers register, BLK-159. `docs/BLOCKERS.md`
    Population => "population", 1, Magnitude;
    /// The residents over the settlement count.
    ///
    /// **A faction of one settlement reads the same value here as it reads
    /// for the residents above.** The division is by the settlement count,
    /// and the count is one. A reader that meets three equal fields is
    /// meeting that case and the case the residents doc names.[^1]
    ///
    /// # References
    ///
    /// [^1]: Blockers register, BLK-159. `docs/BLOCKERS.md`
    PopulationForEachSettlement => "population_for_each_settlement", 1, Magnitude;
    /// The upgrades that stand on ground the faction holds.
    FinishedUpgrades => "finished_upgrades", 1, Magnitude;
    /// The upgrades under construction on ground the faction holds.
    UpgradesUnderway => "upgrades_underway", 1, Magnitude;
    /// The finished upgrades over the settlement count.
    UpgradesForEachSettlement => "upgrades_for_each_settlement", 1, Magnitude;
    /// The furthest a city of the faction reaches, in hex steps.
    ReachRadius => "reach_radius", 1, Magnitude;
    /// The reach radius over the hex radius of the world.
    ReachRadiusShare => "reach_radius_share", 1, Share;
    /// The hexes inside the reach radius over the world passable tiles.
    ReachAreaShare => "reach_area_share", 1, Share;
    /// **Reserved.** The held tiles inside reach over the held tiles.
    ///
    /// The holding assigns a tile to the nearest city that reaches it, so
    /// every held tile lies inside reach by construction and the value is
    /// identically one. A slot that holds one value in every state is either
    /// inert or mis-wired, and a policy spends capacity on it.[^1]
    ///
    /// # References
    ///
    /// [^1]: Research report 42, what a policy should be able to see, section 12.1. `docs/research/reports/42-what-a-policy-should-be-able-to-see.md`
    HeldInsideReachShare => "held_inside_reach_share", 1, Reserved;
    /// The stock of each good class, as a compressed magnitude.
    StockOfClass => "stock_of_class", GOOD_CLASS_COUNT, Magnitude;
    /// The stock of each good class over the whole stock of the faction.
    StockShareOfClass => "stock_share_of_class", GOOD_CLASS_COUNT, Share;
    /// The production less the consumption of each good class, over the sum
    /// of the two.
    NetFlowOfClass => "net_flow_of_class", GOOD_CLASS_COUNT, Relation;
    /// The production rate of each good class, over one tick.
    ProductionOfClass => "production_of_class", GOOD_CLASS_COUNT, Magnitude;
    /// The consumption rate of each good class, over one tick.
    ConsumptionOfClass => "consumption_of_class", GOOD_CLASS_COUNT, Magnitude;
    /// The stores of every settlement of the faction, summed.
    StoreTotal => "store_total", 1, Magnitude;
    /// The military strength of the faction, as a compressed magnitude.
    ///
    /// **This definition is provisional, and a blocker holds the open
    /// question.**[^1] The strength is the sum over the live units of the
    /// faction of the attack column plus the armour column of the unit type
    /// of each one.
    ///
    /// **The engine states that sum in one place, and this field reads it.**
    /// The unit type table answers the strength of one type, the world folds
    /// it over the per-type headcount the arena keeps, and this position
    /// publishes that one number. No arithmetic of the strength lives
    /// here.[^2]
    ///
    /// The sum is monotone in both columns the table holds, so it cannot rank
    /// a stronger army below a weaker one on either axis. A product of the
    /// two columns is the more tempting shape and it is not used: it is
    /// nonlinear, it reaches the range limit sooner, and it makes a unit of
    /// no attack worth nothing when a wall of armour holds ground.
    ///
    /// The value carries the fixed-point scale of the two columns, in the way
    /// the store total does, so a reader that inverts the compression reads a
    /// Q16.16 total and not a headcount.
    ///
    /// **A provisional definition is published on purpose.** A field that
    /// reads a constant zero states a false quantity to every policy that
    /// reads it, and nothing fails. A definition that is wrong is revisable,
    /// and the blocker is where the revision is argued.[^1]
    ///
    /// # References
    ///
    /// [^1]: Blockers register, BLK-158. `docs/BLOCKERS.md`
    /// [^2]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decision D1. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
    MilitaryStrength => "military_strength", 1, Magnitude;
    /// The military strength over the live unit count, as a compressed
    /// magnitude.
    ///
    /// The value says how strong the average unit of the faction is, so a
    /// reader tells a large weak army from a small strong one. It carries the
    /// provisional definition of the strength above and its blocker.[^1]
    ///
    /// # References
    ///
    /// [^1]: Blockers register, BLK-158. `docs/BLOCKERS.md`
    StrengthForEachUnit => "strength_for_each_unit", 1, Magnitude;
    /// The units of each type class over the live units the faction sees.
    UnitShareOfClass => "unit_share_of_class", UNIT_CLASS_COUNT, Share;
    /// The units under no order over the live units the faction sees.
    IdleUnitShare => "idle_unit_share", 1, Share;
    /// **Reserved.** The units inside reach over the live units.
    ///
    /// The engine holds no reach test for a place, only the reach radius of
    /// a city, so the pass cannot say whether a unit stands inside reach
    /// without a distance search over the settlements of the faction.
    UnitShareInsideReach => "unit_share_inside_reach", 1, Reserved;
    /// **Reserved.** The units beside a rival unit over the live units.
    UnitShareBesideRival => "unit_share_beside_rival", 1, Reserved;
    /// The food stock over the food consumption rate, in ticks.
    FoodCoverageTicks => "food_coverage_ticks", 1, Magnitude;
    /// **Reserved.** The change in the population over a window.
    ///
    /// Nothing stores the observation of a past frame, so no first difference
    /// of a snapshot quantity is available. **The decayed event history is
    /// not a substitute for it.** That history counts the events of a kind as
    /// they arrive, and a population that grew carries no event, so no
    /// counter of it rises.[^1]
    ///
    /// Every field of this layout whose doc names a change over a window is
    /// reserved for that one reason.
    ///
    /// # References
    ///
    /// [^1]: The event history. [`crate::event_memory`]
    PopulationChange => "population_change", 1, Reserved;
    /// **Reserved.** The change in the held tiles over a window.
    HeldTileChange => "held_tile_change", 1, Reserved;
    /// **Reserved.** The change in the live units over a window.
    UnitCountChange => "unit_count_change", 1, Reserved;
    /// **Reserved.** The change in the military strength over a window.
    ///
    /// The strength is a snapshot quantity, and nothing stores the reading of
    /// a past frame. The decayed event history counts events and no event
    /// names a change of strength, so no counter of it rises.
    StrengthChange => "strength_change", 1, Reserved;
    /// **Reserved.** The change in the store total over a window.
    StoreValueChange => "store_value_change", 1, Reserved;
    /// **Reserved.** The upgrades finished over a window.
    UpgradeCompletions => "upgrade_completions", 1, Reserved;
    /// **Reserved.** The tiles taken from a rival over a window.
    ///
    /// The engine counts the ground each faction gained, and gained ground is
    /// not taken ground. A tile changes hands because the reach of a city
    /// moved, and the holder column carries the holder of this step and not
    /// the holder of the last one, so nothing says whether a tile the faction
    /// gained came from a rival or from nobody.[^1]
    ///
    /// A count of ground newly held would answer under this name and would
    /// state a different quantity, and a policy rewarded on it would score
    /// for settling an empty plain.
    ///
    /// # References
    ///
    /// [^1]: The event history. [`crate::event_memory`]
    TilesTaken => "tiles_taken", 1, Reserved;
    /// The tiles the faction stopped holding over a window, as a share of the
    /// tiles it holds.
    ///
    /// **The memory block publishes the same value.** The decayed event
    /// history counts the ground each faction lost, one function reads that
    /// counter, and both positions call it.[^1]
    ///
    /// Held ground carries no event of its own. A tile changes hands because
    /// the reach of a city moved, so the count is the fall of the held total
    /// between two steps and it names no taker.[^1]
    ///
    /// # References
    ///
    /// [^1]: The event history. [`crate::event_memory`]
    TilesLost => "tiles_lost", 1, Share;
    /// The settlements the faction lost over a window, as a share of the
    /// settlements it holds.
    ///
    /// The count holds a settlement that a rival kept and a settlement that a
    /// rival destroyed, because both end the seat of the reader on that
    /// ground. **The memory block publishes the same value from the same
    /// counter.**[^1]
    ///
    /// # References
    ///
    /// [^1]: The event history. [`crate::event_memory`]
    SettlementsLost => "settlements_lost", 1, Share;
    /// The highest renown of a live character of the faction.
    BestRenown => "best_renown", 1, Magnitude;
    /// The best renown of the faction over the best renown of every faction.
    BestRenownShare => "best_renown_share", 1, Share;
    /// The live characters of the faction.
    LiveCharacters => "live_characters", 1, Magnitude;
    /// The tiles the faction has ever seen over the world tiles.
    ObservedShareWorld => "observed_share_world", 1, Share;
    /// The observed tiles whose ground admits a unit, over the observed
    /// tiles.
    ObservedPassableShare => "observed_passable_share", 1, Share;
    /// **Reserved.** The tiles first seen over a window.
    NewlyObserved => "newly_observed", 1, Reserved;
    /// **Reserved.** The mean ticks since the faction last saw a remembered
    /// tile.
    ///
    /// The fog layer records whether a faction saw a tile and never when. A
    /// staleness needs a last-seen tick for each observed tile, which is
    /// per-faction state the engine does not carry.
    MeanStaleness => "mean_staleness", 1, Reserved;
    /// The held tiles under a hazard over the held tiles the faction sees.
    ///
    /// **A hazard is a ground condition that harms.** Fire is the one such
    /// condition this engine holds: it takes units, and the memory block
    /// counts the units it took. Wet ground is not a hazard. A gatherer takes
    /// more from wet ground, so wet ground is a benefit, and this field
    /// counted it as a harm.[^1] [^2]
    ///
    /// The fire share of the held ground publishes the same count over the
    /// same denominator, because fire is the whole of the hazard today. **The
    /// two positions separate on the day the engine gains a second harm**,
    /// and this one is the aggregate.
    ///
    /// # References
    ///
    /// [^1]: ADR-0143, wet ground yields more to a gatherer, decision D1. `docs/adrs/draft/adr-0143-wet-ground-yields-more-to-a-gatherer.md`
    /// [^2]: The audit of the observation, section 2.8. `docs/research/what-a-policy-cannot-see.md`
    HeldUnderHazardShare => "held_under_hazard_share", 1, Share;
    /// The settlements under a hazard over the settlement count.
    ///
    /// A settlement counts when the tile it stands on burns. The rule of the
    /// hazard is the one stated at the field above.
    SettlementsUnderHazardShare => "settlements_under_hazard_share", 1, Share;
    /// The tiles of the world.
    WorldTiles => "world_tiles", 1, Magnitude;
    /// The tiles of the world whose ground admits a unit.
    WorldPassableTiles => "world_passable_tiles", 1, Magnitude;
    /// The seated factions over the faction ceiling of sixteen.
    SeatedFactionShare => "seated_faction_share", 1, Share;
    /// The hex distance from the centre of the held ground to the centre of
    /// the world, over the hex radius of the world.
    CentroidOffset => "centroid_offset", 1, Share;
    /// **Reserved.** The movement of that centre over a window.
    CentroidMovement => "centroid_movement", 1, Reserved;
    /// The held tiles with a neighbour the faction does not hold.
    BorderLength => "border_length", 1, Magnitude;
    /// The border tiles beside ground a rival holds, over the border length.
    ContestedBorderShare => "contested_border_share", 1, Share;
    /// The work toward the furthest wonder of the faction, over the work the
    /// wonder row asks for.
    ///
    /// **The wonder track of block C publishes the same value.** The design
    /// places the signal in both blocks, one pass produces it, and both
    /// positions read that one accumulator.
    WonderProgress => "wonder_progress", 1, Share;

    /// Block B. The tick over the tick limit.
    ///
    /// A world with no limit reads zero, because a run with no end has no
    /// position inside an episode.
    TickShare => "tick_share", 1, Share;
    /// The ticks left before the limit fires.
    RemainingTicks => "remaining_ticks", 1, Magnitude;
    /// **Reserved.** The decisions the faction has taken.
    ///
    /// The engine counts ticks and does not count decisions. A decision
    /// interval is a property of the trainer and not of the world.
    DecisionsTaken => "decisions_taken", 1, Reserved;
    /// **Reserved.** The length of the window, in ticks.
    ///
    /// **The layout carries two window lengths and not one.** The decayed
    /// event history keeps a short memory and a long memory of every kind,
    /// because one counter cannot separate a spike from a trend, and one
    /// position cannot state both lengths.[^1]
    ///
    /// The two lengths are structural, so a reader that needs them reads the
    /// history and never this position.
    ///
    /// # References
    ///
    /// [^1]: The event history. [`crate::event_memory`]
    WindowTicks => "window_ticks", 1, Reserved;
    /// The season phase, as a triangle wave and a quarter-shifted triangle
    /// wave.
    ///
    /// The pair replaces a sine and a cosine. A single phase value carries a
    /// step at the wrap point, and a network reads that step as a large
    /// change in the world.
    WeatherPhase => "weather_phase", 2, Relation;
    /// **Reserved.** A longer world cycle, in the same form.
    ///
    /// The weather carries one cycle, which is the season. The engine holds
    /// no second and longer cycle.
    WorldCyclePhase => "world_cycle_phase", 2, Reserved;

    /// Block C. The seats of live factions the faction holds, over the seats
    /// the domination reader asks for.
    ///
    /// **The denominator is the requirement of the reader that ends the
    /// game, and that requirement shrinks.** The seat clause asks the
    /// candidate for the seat of every faction that is still in the game, its
    /// own seat included, and a faction that has left the game takes its seat
    /// out of the ask. A share over the seated factions therefore tops out
    /// below one at the moment the reader fires, and a policy that climbed it
    /// would climb toward a value it cannot reach.[^1] [^2]
    ///
    /// This position reads one exactly when the seat clause holds.
    ///
    /// **The unit clause of the same reader stays unpublished, and the fog is
    /// the reason.** That clause ends the game for a faction that holds a
    /// unit while every rival holds none, and a share of it would state the
    /// unit count of a rival the reader has never observed. The layout
    /// publishes the units the reader has seen and no other unit count, and a
    /// second, unfogged count of the same quantity would contradict it.[^3]
    /// A reader that wants the annihilation branch reads the unit order
    /// statistics, which are weaker on purpose.
    ///
    /// # References
    ///
    /// [^1]: ADR-0148, a game end is recorded once and stops the controllers, decision D3. `docs/adrs/accepted/adr-0148-a-game-end-is-recorded-once-and-stops-the-controllers.md`
    /// [^2]: ADR-0181, a faction that holds no site and no unit leaves the game, decision D5. `docs/adrs/draft/adr-0181-a-faction-that-holds-no-site-and-no-unit-leaves-the-game.md`
    /// [^3]: PRD-0001, a faction sees only what it observes. `docs/product/accepted/prd-0001-a-faction-sees-only-what-it-observes.md`
    DominationProgress => "domination_progress", 1, Share;
    /// The seats of live factions the leading faction holds, over the same
    /// requirement.
    DominationLeader => "domination_leader", 1, Share;
    /// The seats of live factions the faction holds, against the count of the
    /// strongest rival.
    DominationGap => "domination_gap", 1, Relation;
    /// The rivals the faction leads on those seats, over the rival count.
    DominationRank => "domination_rank", 1, Share;
    /// **Reserved.** The estimated ticks to a domination win, for the faction
    /// and for the leader.
    ///
    /// An estimate divides the gap by the rate over a window, and no window
    /// exists.
    DominationTicks => "domination_ticks", 2, Reserved;
    /// The work toward a wonder over the work the wonder row asks for.
    WonderTrackProgress => "wonder_track_progress", 1, Share;
    /// The same share for the leading faction.
    WonderTrackLeader => "wonder_track_leader", 1, Share;
    /// The wonder work of the faction against the strongest rival.
    WonderTrackGap => "wonder_track_gap", 1, Relation;
    /// The rivals the faction leads on wonder work, over the rival count.
    WonderTrackRank => "wonder_track_rank", 1, Share;
    /// **Reserved.** The estimated ticks to a wonder win.
    WonderTrackTicks => "wonder_track_ticks", 2, Reserved;
    /// The best renown over the renown target the balance holds.
    RenownProgress => "renown_progress", 1, Share;
    /// The same share for the leading faction.
    RenownLeader => "renown_leader", 1, Share;
    /// The best renown of the faction against the strongest rival.
    RenownGap => "renown_gap", 1, Relation;
    /// The rivals the faction leads on renown, over the rival count.
    RenownRank => "renown_rank", 1, Share;
    /// **Reserved.** The estimated ticks to a renown win.
    RenownTicks => "renown_ticks", 2, Reserved;
    /// The held tiles over the world passable tiles.
    GroundProgress => "ground_progress", 1, Share;
    /// The same share for the leading faction.
    GroundLeader => "ground_leader", 1, Share;
    /// The held tiles of the faction against the strongest rival.
    GroundGap => "ground_gap", 1, Relation;
    /// The rivals the faction leads on held tiles, over the rival count.
    GroundRank => "ground_rank", 1, Share;
    /// **Reserved.** The estimated ticks to a held-ground win.
    GroundTicks => "ground_ticks", 2, Reserved;

    /// Block D. The seven order statistics of the held tile count.
    ///
    /// The own value is the running total the holding keeps. The value of a
    /// rival is the ground the faction sees the rival hold this frame, which
    /// is an estimate from below. The confidence statistic states how much of
    /// the ground the faction sees now.
    PowerHeldTiles => "power_held_tiles", POWER_STATISTIC_COUNT, Statistic;
    /// The seven order statistics of the settlement count.
    ///
    /// The value of a rival counts the settlements standing on ground the
    /// faction has ever seen.
    PowerSettlements => "power_settlements", POWER_STATISTIC_COUNT, Statistic;
    /// **Reserved.** The seven order statistics of the population total.
    ///
    /// A faction reads the people of its own settlements. The people of a
    /// rival settlement are not a fact of the ground, so a faction that sees
    /// the settlement still cannot count them, and the engine holds no
    /// observable estimate of them.
    PowerPopulation => "power_population", POWER_STATISTIC_COUNT, Reserved;
    /// The seven order statistics of the unit count.
    ///
    /// The value of a rival counts the units standing on ground the faction
    /// sees this frame.
    PowerUnits => "power_units", POWER_STATISTIC_COUNT, Statistic;
    /// The seven order statistics of the military strength.
    ///
    /// The own value is exact, and it is the strength the world folds over
    /// the whole army of the faction. The value of a rival is the strength of
    /// the rival units that stand on ground the faction sees this frame,
    /// which is an estimate from below. The confidence statistic states how
    /// much of the ground the faction sees now.
    ///
    /// **The strength carries a provisional definition, and a blocker holds
    /// the open question.**[^1]
    ///
    /// # References
    ///
    /// [^1]: Blockers register, BLK-158. `docs/BLOCKERS.md`
    PowerStrength => "power_strength", POWER_STATISTIC_COUNT, Statistic;
    /// The seven order statistics of the finished upgrade count.
    ///
    /// The value of a rival counts the upgrades standing on ground the
    /// faction has ever seen.
    PowerUpgrades => "power_upgrades", POWER_STATISTIC_COUNT, Statistic;
    /// The seven order statistics of the highest live renown.
    ///
    /// **Renown carries no fog rule.** A character is not a fact of a tile,
    /// and the renown reader ends the game on the value, so the layout treats
    /// it as public and the confidence statistic reads one.
    PowerRenown => "power_renown", POWER_STATISTIC_COUNT, Statistic;
    /// The seven order statistics of the wonder progress.
    ///
    /// **A victory claim carries no fog rule.** The wonder reader ends the
    /// game on it, so the layout treats it as public and the confidence
    /// statistic reads one.
    PowerWonder => "power_wonder", POWER_STATISTIC_COUNT, Statistic;
    /// **Reserved.** The seven order statistics of the store total.
    ///
    /// A store is not a fact of the ground, so a faction cannot observe the
    /// store of a rival at all.
    PowerStoreValue => "power_store_value", POWER_STATISTIC_COUNT, Reserved;
    /// **Reserved.** The seven order statistics of the held tiles gained over
    /// a window.
    PowerTileGain => "power_tile_gain", POWER_STATISTIC_COUNT, Reserved;
    /// **Reserved.** The seven order statistics of the reach area.
    ///
    /// The reach of a city grows with the finished upgrades of its faction.
    /// A faction observes an upgrade only where it has walked, so an
    /// observed reach is a lower bound of unknown tightness.
    PowerReachArea => "power_reach_area", POWER_STATISTIC_COUNT, Reserved;
    /// **Reserved.** The seven order statistics of the trade volume over a
    /// window.
    ///
    /// The engine keeps no volume account, and no window exists.
    PowerTradeVolume => "power_trade_volume", POWER_STATISTIC_COUNT, Reserved;

    /// Block E. The egocentric multi-resolution ring stack.
    ///
    /// The block holds one group of channels for each ring-sector cell, in
    /// ascending cell order. The ring module derives the cell count from the
    /// ring cap, so the width of the block follows the cap and no number
    /// here.
    RingStack => "ring_stack", RING_STACK_CHANNELS * RING_STACK_CELLS, Statistic;

    /// Block F. The frontier and the pressure by sector.
    ///
    /// The block samples the far sectors alone, so its width does not follow
    /// the ring count.
    FrontierBySector => "frontier_by_sector", FRONTIER_SLOTS, Statistic;

    /// Block G. The public trade board, by good class.
    ///
    /// The block holds one group for each good class in ascending class
    /// order. A group holds the best price to buy, the best price to sell,
    /// the spread between the two, the depth offered, and a reserved position
    /// for the price change over a window.
    ///
    /// A price is the asking quantity over the offered quantity, held in
    /// Q16.16 and then compressed. The board is public, so no fog applies.
    TradeBoard => "trade_board", GOOD_CLASS_COUNT * BOARD_STATISTIC_COUNT, Statistic;

    /// Block H. The mean relation the faction holds toward its rivals.
    RelationMeanOut => "relation_mean_out", 1, Relation;
    /// The lowest relation the faction holds toward a rival.
    RelationMinOut => "relation_min_out", 1, Relation;
    /// The highest relation the faction holds toward a rival.
    RelationMaxOut => "relation_max_out", 1, Relation;
    /// The mean relation the rivals hold toward the faction.
    RelationMeanIn => "relation_mean_in", 1, Relation;
    /// The lowest relation a rival holds toward the faction.
    RelationMinIn => "relation_min_in", 1, Relation;
    /// The mean difference between the two directions of each pair.
    RelationAsymmetry => "relation_asymmetry", 1, Relation;
    /// The rivals at war with the faction, over the rival count.
    WarShare => "war_share", 1, Share;
    /// The rivals at peace with the faction, over the rival count.
    PeaceShare => "peace_share", 1, Share;
    /// The rivals above the alliance edge, over the rival count.
    AllianceShare => "alliance_share", 1, Share;
    /// The mean relation among the rivals, with the faction excluded.
    ///
    /// The value tells the faction whether the others are aligned against
    /// it. A faction that reads only its own relations cannot see that.
    RivalPairRelationMean => "rival_pair_relation_mean", 1, Relation;
    /// The rival pairs at war with each other, over the rival pairs.
    RivalPairWarShare => "rival_pair_war_share", 1, Share;
    /// The relation the faction holds toward the leader on held ground.
    RelationToLeader => "relation_to_leader", 1, Relation;
    /// The relation the leader on held ground holds toward the faction.
    RelationFromLeader => "relation_from_leader", 1, Relation;
    /// **Reserved.** The mean relation change over a window, and the rivals
    /// whose relation to the faction fell.
    RelationChange => "relation_change", 2, Reserved;
    /// The rivals holding a bound contract with the faction, over the rival
    /// count.
    ContractShare => "contract_share", 1, Share;

    /// Block I. The settlements the reader holds, as tokens.
    ///
    /// **One field states one shape.** The token sets hold different channel
    /// counts, so one field cannot say how wide a token of each set is. The
    /// layout therefore publishes one field for each set, and each one names
    /// its own channels.
    ///
    /// The set list is the one declaration of the order of the sets, and each
    /// field takes its name and its width from that list.
    TokenOwnSettlements => TokenSet::Settlements.name(), TokenSet::Settlements.slots(), Statistic;
    /// The rivals the reader ranks by threat, as tokens.
    TokenRivals => TokenSet::Rivals.name(), TokenSet::Rivals.slots(), Statistic;
    /// The clusters of rival units the reader remembers, as tokens.
    TokenThreatClusters => TokenSet::Threats.name(), TokenSet::Threats.slots(), Statistic;
    /// The places a founding survey scored, as tokens.
    TokenCandidateSites => TokenSet::Sites.name(), TokenSet::Sites.slots(), Statistic;

    /// **Reserved.** Block J. The observed ground under a storm.
    ///
    /// The engine holds a cyclone list in weather lattice coordinates and no
    /// reader that says whether a storm stands over a tile. A threshold on
    /// the cloud share would be a number this file invented.
    StormShareObserved => "storm_share_observed", 1, Reserved;
    /// The observed ground with wet ground, over the observed tiles.
    WaterShareObserved => "water_share_observed", 1, Share;
    /// The observed ground on fire, over the observed tiles.
    FireShareObserved => "fire_share_observed", 1, Share;
    /// **Reserved.** The held ground under a storm.
    StormShareHeld => "storm_share_held", 1, Reserved;
    /// The held ground with wet ground, over the held tiles the faction sees.
    WaterShareHeld => "water_share_held", 1, Share;
    /// The held ground on fire, over the held tiles the faction sees.
    FireShareHeld => "fire_share_held", 1, Share;
    /// **Reserved.** The change in each held-ground hazard share over a
    /// window.
    HazardShareChange => "hazard_share_change", 3, Reserved;
    /// The units a hazard took over a window, as a share of the live units.
    ///
    /// A fire has no aggressor. A policy that read this loss under a loss to
    /// a rival would learn to fear ground that nobody threatens, so the two
    /// counters are separate. **The memory block publishes the same value
    /// from the same counter.**[^1]
    ///
    /// # References
    ///
    /// [^1]: The event history. [`crate::event_memory`]
    UnitsLostToHazard => "units_lost_to_hazard", 1, Share;
    /// **Reserved.** The tiles burnt over a window.
    ///
    /// The engine counts the units a fire took and counts no ground it
    /// burned. The fire logs name the tile a fire started on and the tile it
    /// ended on, and neither says that the ground of the reader changed.
    TilesBurnt => "tiles_burnt", 1, Reserved;
    /// **Reserved.** The concentration of the hazard across the twelve
    /// sectors.
    ///
    /// The sector frame arrives with the ring stack. **The spatial layout
    /// revision owns this position.**
    HazardConcentration => "hazard_concentration", 1, Reserved;

    /// Block K. One when the settle verb is legal for the faction now.
    ///
    /// A flag is a share that holds zero or one. No position of this layout
    /// holds a Rust boolean, because an observation must be plain data with
    /// declared padding.
    MayFound => "may_found", 1, Share;
    /// **Reserved.** The stores the faction holds over the stores a founding
    /// asks for.
    ///
    /// A founding in this engine spends no store. It asks for a settler unit
    /// on ground a city may take, and the legality flag already states that.
    SettleCostCoverage => "settle_cost_coverage", 1, Reserved;
    /// **Reserved.** One when the best sector admits a settlement.
    ///
    /// The sector frame arrives with the ring stack.
    BestSectorAdmitsSettlement => "best_sector_admits_settlement", 1, Reserved;
    /// **Reserved.** The stores the faction holds over the stores each
    /// upgrade class asks for.
    ///
    /// An upgrade in this engine costs work and not stores, so no store cost
    /// exists to divide by.
    UpgradeAffordability => "upgrade_affordability", UPGRADE_CLASS_COUNT, Reserved;
    /// One for each upgrade class the build verb accepts from the faction
    /// now.
    UpgradeLegality => "upgrade_legality", UPGRADE_CLASS_COUNT, Share;
    /// The units under no order over the live units the faction sees.
    ///
    /// The value is the same accumulator the idle unit share of block A
    /// reads. The layout publishes it twice because the design places the
    /// signal in both blocks, and one pass produces it.
    FreeUnitShare => "free_unit_share", 1, Share;
    /// The stores a bound contract owes, over the whole stock of the faction.
    CommittedStockShare => "committed_stock_share", 1, Share;
    /// The empty rows of the board of the faction, over the row limit.
    BoardRowHeadroom => "board_row_headroom", 1, Share;
    /// **Reserved.** The contracts the faction would accept.
    ///
    /// An acceptance test needs a valuation rule, and no record states one.
    AcceptableContracts => "acceptable_contracts", 1, Reserved;
    /// The reach the cap allows above the current reach, over the cap.
    ReachHeadroomShare => "reach_headroom_share", 1, Share;

    /// Block L. The weight vector the faction plays under.
    ///
    /// The engine holds five controller weights, and they fill the first five
    /// positions in the order the weight vector declares. The remaining
    /// positions read zero, because the twelve-element objective vector of
    /// the reward design does not exist in the engine yet.
    ObjectiveWeight => "objective_weight", OBJECTIVE_WEIGHT_COUNT, Relation;
    /// The share of its own stock that each kind of event moved lately, over
    /// the short memory.
    ///
    /// **The array is a snapshot, and this is the memory beside it.** A
    /// snapshot cannot tell a faction that is gaining ground from one that is
    /// losing it, and it cannot say that a rival is taking a city now.[^1]
    ///
    /// Each position holds one kind of event, in the order the kind list
    /// declares. The value is the part of the stock of the reader that the
    /// events of one step move, so a small world and a large one with the
    /// same event rate read the same value.[^2]
    ///
    /// # References
    ///
    /// [^1]: The event history. [`crate::event_memory`]
    /// [^2]: Research report 42, what a policy should be able to see, section 8.1. `docs/research/reports/42-what-a-policy-should-be-able-to-see.md`
    MemoryRecent => "memory_recent", KIND_COUNT as u32, Share;
    /// The same share of each kind of event, over the long memory.
    MemoryLasting => "memory_lasting", KIND_COUNT as u32, Share;
    /// The signed relation between the short memory and the long memory of
    /// each kind.
    ///
    /// **This is the position that separates a spike from a trend.** Both
    /// memories reach the same share for the same constant arrival rate, so
    /// this reads zero while the rate holds, positive while it rises and
    /// negative while it falls. A rival that takes a city this minute raises
    /// the short memory alone. A rival that keeps killing the people of the
    /// reader raises both.
    MemoryTrend => "memory_trend", KIND_COUNT as u32, Relation;
    /// The share of each kind of event that the single worst rival caused,
    /// over the long memory.
    ///
    /// The field holds one position for each kind that names a faction as its
    /// cause. **No position names a seat.** A league seats one policy in
    /// different seats between games, so a policy that learned a seat number
    /// would read another faction's quantities under the same weight.[^1]
    ///
    /// # References
    ///
    /// [^1]: Findings register, FND-647. `docs/FINDINGS.md`
    MemoryWorstRival => "memory_worst_rival", BLAMED_KIND_COUNT as u32, Share;
    /// How concentrated the cause of each kind of event is over the rivals,
    /// over the long memory.
    ///
    /// The value is the sum of the squared rival shares. Two rivals in equal
    /// measure give one half, and one rival that does everything gives one. A
    /// reader tells one enemy from a field of them by this position alone.
    MemoryConcentration => "memory_concentration", BLAMED_KIND_COUNT as u32, Share;

    /// The seats of rivals that the faction holds, as a compressed magnitude.
    ///
    /// **A domination win asks for every rival seat, so the win condition is
    /// a count.** The domination block of this layout publishes the progress
    /// as a share of the seated factions, and a share cannot say that one
    /// seat is left. A policy that chases the domination path reads this
    /// position and the one below it, and it knows how many seats remain.[^1]
    ///
    /// The seat of the faction itself is not counted here. A faction holds
    /// its own seat in the ordinary case, so a count that held it would never
    /// reach zero and would mean a different thing on a lost seat.
    ///
    /// **This field and the one below sit at the end of the layout on
    /// purpose.** They take their positions from the reserve, so no earlier
    /// field moves.[^2]
    ///
    /// # References
    ///
    /// [^1]: Research report 42, what a policy should be able to see, section 9. `docs/research/reports/42-what-a-policy-should-be-able-to-see.md`
    /// [^2]: ADR-0195, the observation of a faction is a fixed-width scale-free table, the reserve. `docs/adrs/draft/adr-0195-the-observation-of-a-faction-is-a-fixed-width-scale-free-table.md`
    RivalSeatsHeld => "rival_seats_held", 1, Magnitude;
    /// The seats of rivals that exist, as a compressed magnitude.
    ///
    /// The value is the seated factions less one, so a reader divides the
    /// count above by it and gets the part of the win path the faction has
    /// walked. It holds every rival seat, whether the rival that started on
    /// it is still seated or not, because a seat that changed hands is still
    /// a seat a winner must hold.
    RivalSeats => "rival_seats", 1, Magnitude;
    /// The settlers the faction holds.
    ///
    /// A settler is a unit whose type row holds a settle column above zero,
    /// and this reads the reader the settle verb reads.[^1]
    ///
    /// **The founding flag says whether the verb would found now, and this
    /// says whether the faction owns the unit the verb needs.** The two
    /// answer different questions, and a faction with no settler reads zero
    /// for both. A reader of the flag alone cannot tell a faction that holds
    /// no settler from a faction whose settler stands on ground a city may
    /// not take, and those two states ask for different actions.
    ///
    /// **The position comes from the reserve, so no other field moved.** The
    /// reserve exists for this, and a claim on it keeps every stored weight
    /// file placeable.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decision D2. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
    /// [^2]: The reserve. [`LAYOUT_RESERVE`]
    Settlers => "settlers", 1, Magnitude;
    /// The hex distance from the seat of the faction to the campaign
    /// objective, over the widest distance the world holds.
    ///
    /// **The objective is the settlement the campaign verb would march on,
    /// and the faction reads it through its own fog.** The engine resolves
    /// one objective for each faction: an own settlement on ground a faction
    /// at war holds is a relief, and failing that the nearest settlement of a
    /// faction at war is a take. The nearest wins and a relief comes
    /// first.[^1]
    ///
    /// **This position reads the fogged reader, and not the search the
    /// built-in controller reads.** That search walks the whole board, and a
    /// faction may not read a settlement it has never observed.[^2] A faction
    /// that has seen no enemy settlement therefore reads zero here and zero
    /// in the flag below, whatever the board holds.
    ///
    /// # References
    ///
    /// [^1]: ADR-0146, a faction relation is one signed integer per ordered pair, and a pass reads a threshold, decision D2. `docs/adrs/accepted/adr-0146-a-faction-relation-is-one-signed-integer-per-ordered-pair-and-a-pass-reads-a-threshold.md`
    /// [^2]: PRD-0001, a faction sees only what it observes. `docs/product/accepted/prd-0001-a-faction-sees-only-what-it-observes.md`
    CampaignObjectiveDistance => "campaign_objective_distance", 1, Share;
    /// One when the faction has an observed campaign objective now.
    ///
    /// A flag is a share that holds zero or one. The distance above means
    /// nothing while this position reads zero, in the way every distance of
    /// this layout carries a flag beside it.
    CampaignObjectiveFlag => "campaign_objective_flag", 1, Share;
    /// One when the campaign objective is a relief of the reader's own
    /// settlement.
    ///
    /// A relief and a take ask for different action: a relief defends ground
    /// the faction already holds, and a take crosses a border. The engine
    /// ranks the two, and this position names which one the rank chose.
    CampaignObjectiveRelief => "campaign_objective_relief", 1, Share;

    /// **Reserved.** The positions the layout holds back for a later signal.
    LayoutReserve => "layout_reserve", LAYOUT_RESERVE, Reserved;
}

/// The axis that runs first where a block holds channels over places.
///
/// Every block of this layout stores the channels of one place next to each
/// other, so a reader of one place gathers one unbroken run. The vocabulary
/// is the one the drawing tool of the project established.[^1]
///
/// # References
///
/// [^1]: The drawing tool of the observation. `python/cachette/learn/picture.py`
pub const CHANNEL_ORDER: &str = "cell_major";

impl ObsField {
    /// Returns the space the positions of the field lay out in.
    ///
    /// A ring field holds one group of channels for each cell of the
    /// egocentric frame. A token field holds one group of channels for each
    /// token of a set, and a token position names no subject. Every other
    /// field holds separate quantities and lays out in no space.
    #[must_use]
    pub const fn space(self) -> Option<&'static str> {
        if self.token_set().is_some() {
            return Some(TOKEN_SPACE);
        }
        match self {
            Self::RingStack => Some(RING_SPACE),
            _ => None,
        }
    }

    /// Returns the channels of one place of the field, in store order.
    ///
    /// A field that holds one quantity for each position names no channel and
    /// answers an empty list. The module that fills a block owns the list, so
    /// this file states no name of its own.
    #[must_use]
    pub const fn channels(self) -> &'static [&'static str] {
        if let Some(set) = self.token_set() {
            return set.channel_names();
        }
        match self {
            Self::RingStack => &crate::obs_ring_stack::RING_STACK_CHANNEL_NAMES,
            _ => &[],
        }
    }

    /// Returns the token set the field publishes, or nothing.
    #[must_use]
    pub const fn token_set(self) -> Option<TokenSet> {
        match self {
            Self::TokenOwnSettlements => Some(TokenSet::Settlements),
            Self::TokenRivals => Some(TokenSet::Rivals),
            Self::TokenThreatClusters => Some(TokenSet::Threats),
            Self::TokenCandidateSites => Some(TokenSet::Sites),
            _ => None,
        }
    }
}

/// One row of the observation schema.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FieldRow {
    /// The field the row describes.
    pub field: ObsField,
    /// The position the field starts at.
    pub start: u32,
    /// The positions the field holds.
    pub positions: u32,
    /// The lowest value any position of the field may hold.
    pub low: i64,
    /// The highest value any position of the field may hold.
    pub high: i64,
}

impl FieldRow {
    /// Returns the name of the field.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        self.field.name()
    }

    /// Returns the space the positions of the field lay out in.
    #[must_use]
    pub const fn space(&self) -> Option<&'static str> {
        self.field.space()
    }

    /// Returns the channels of one place of the field, in store order.
    #[must_use]
    pub const fn channels(&self) -> &'static [&'static str] {
        self.field.channels()
    }

    /// Returns the name of the value form of the field.
    ///
    /// **The form comes from the same declaration that decides how the field
    /// is written.** The field list gives one kind for each field, the writer
    /// takes its arithmetic from that kind, and this answer reads the same
    /// kind. A form declared beside the field could disagree with the writer,
    /// and nothing would fail.[^1]
    ///
    /// # References
    ///
    /// [^1]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
    #[must_use]
    pub const fn form_name(&self) -> &'static str {
        self.field.value_kind().form_name()
    }

    /// Returns the value form of the field, with its inversion parameters.
    #[must_use]
    pub const fn form(&self) -> ValueForm {
        self.field.value_kind().form()
    }

    /// Returns how a reader reads each position of the field.
    ///
    /// Every position of the array is a signed eight-byte integer that holds
    /// a Q16.16 value. The array holds no floating point number, because the
    /// simulation holds none.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0002, state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
    #[must_use]
    pub const fn kind(&self) -> ColumnKind {
        ColumnKind::I64
    }

    /// Returns the width of one position, in bytes.
    #[must_use]
    pub const fn width(&self) -> usize {
        self.kind().width()
    }
}

/// The declared layout of the observation array.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObservationSchema {
    version: u32,
    length: u32,
    rows: Vec<FieldRow>,
    ring_cells: Vec<u32>,
}

impl ObservationSchema {
    /// Returns the version of the layout.
    #[must_use]
    pub const fn version(&self) -> u32 {
        self.version
    }

    /// Returns how many positions the whole array holds.
    #[must_use]
    pub const fn length(&self) -> u32 {
        self.length
    }

    /// Returns one row for each field, in the order the array holds them.
    #[must_use]
    pub fn rows(&self) -> &[FieldRow] {
        &self.rows
    }

    /// Returns the row of one field, by name.
    #[must_use]
    pub fn row(&self, name: &str) -> Option<FieldRow> {
        self.rows.iter().copied().find(|row| row.name() == name)
    }

    /// Returns the cells of each ring of the egocentric frame, in ring order.
    ///
    /// A cell index says nothing about its ring and its sector on its own, so
    /// a reader of a ring field needs this list. The ring module derives it
    /// from the sector rule of the frame.
    #[must_use]
    pub fn ring_cells(&self) -> &[u32] {
        &self.ring_cells
    }

    /// Returns every value form the layout uses, with its parameters.
    ///
    /// A field names its form, and this table says what the name means. The
    /// table sits beside the field list rather than inside each row, because
    /// one form serves many fields and a parameter repeated for each field is
    /// a parameter stated many times.
    ///
    /// **A reader inverts a published value through this table alone.** A
    /// count crosses the boundary as a compressed magnitude, so a reader that
    /// wanted the count had to hold the compression itself, and that second
    /// copy of an engine rule fails silently when the engine moves.[^1]
    ///
    /// # References
    ///
    /// [^1]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
    #[must_use]
    pub fn value_forms(&self) -> Vec<ValueForm> {
        ValueKind::ALL.iter().map(|kind| kind.form()).collect()
    }

    /// Returns the axis that runs first where a block holds channels over
    /// places.
    #[must_use]
    pub const fn channel_order(&self) -> &'static str {
        CHANNEL_ORDER
    }

    /// Returns the name of the channel that says whether a cell holds a value.
    ///
    /// A cell of the frame that lies outside the world reads zero in every
    /// channel. A reader that took that zero for a quantity would read a fact
    /// the engine never published, so the gate channel states the
    /// difference.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0195, the observation of a faction is a fixed-width scale-free table, decision D8. `docs/adrs/draft/adr-0195-the-observation-of-a-faction-is-a-fixed-width-scale-free-table.md`
    #[must_use]
    pub const fn spatial_gate(&self) -> &'static str {
        crate::obs_ring_stack::AREA_CHANNEL_NAME
    }
}

/// Builds the one schema of this layout.
///
/// The schema is a property of the layout and not of a world, because no
/// length and no bound follows the world shape or the faction count.
#[must_use]
pub fn observation_schema() -> ObservationSchema {
    let mut rows = Vec::with_capacity(ObsField::ALL.len());
    let mut start = 0u32;
    for field in ObsField::ALL {
        let positions = field.positions();
        let (low, high) = field.value_kind().bounds();
        rows.push(FieldRow {
            field: *field,
            start,
            positions,
            low,
            high,
        });
        start = start.saturating_add(positions);
    }
    ObservationSchema {
        version: OBSERVATION_VERSION,
        length: start,
        rows,
        ring_cells: ring_cell_counts().to_vec(),
    }
}

/// The faction ceiling that the seated faction share divides by.
const FACTION_CEILING: i64 = 16;

/// The units one tile admits before the layout calls it full.
///
/// The value scales the units-for-each-held-tile share into the unit range.
/// It is a structural choice of the layout and not a rule of the world.
const UNITS_FOR_EACH_TILE_SCALE: i64 = 8;

/// One power quantity over every seated faction, and how much of it the
/// reading faction observed.
///
/// The vector is indexed by seat, and **no position of the array publishes
/// it that way**. The vector exists so that the order statistics can be
/// taken, and the statistics are order independent.[^1]
///
/// # References
///
/// [^1]: Research report 42, what a policy should be able to see, section 6.3. `docs/research/reports/42-what-a-policy-should-be-able-to-see.md`
struct PowerVector {
    values: Vec<i64>,
    confidence: Fix32,
}

impl PowerVector {
    /// Returns the seven order statistics of the quantity, for one reader.
    ///
    /// The statistics are the own share of the total, the own rank as a
    /// share, the gap to the strongest rival, the gap to the median rival,
    /// the leader share of the total, the concentration, and the observation
    /// confidence.
    ///
    /// The median of an even rival count is the lower of the two middle
    /// values, because every division of this layout truncates.
    fn statistics(&self, seat: usize) -> [i64; POWER_STATISTIC_COUNT as usize] {
        let own = self.values.get(seat).copied().unwrap_or(0);
        let total = self
            .values
            .iter()
            .fold(0i64, |sum, value| sum.saturating_add(*value));
        let mut rivals = Vec::with_capacity(self.values.len());
        for (at, value) in self.values.iter().enumerate() {
            if at != seat {
                rivals.push(*value);
            }
        }
        rivals.sort_unstable();
        let rival_count = rivals.len() as i64;
        let strongest = rivals.last().copied().unwrap_or(0);
        let median = if rivals.is_empty() {
            0
        } else {
            rivals[(rivals.len() - 1) / 2]
        };
        let ahead = rivals.iter().filter(|value| **value < own).count() as i64;
        let leader = self.values.iter().copied().fold(0i64, i64::max);
        let mut concentration = 0i64;
        for value in &self.values {
            let part = sim_math::bounded_share(*value, total);
            concentration = concentration.saturating_add(i64::from(sim_math::mul(part, part).0));
        }
        [
            i64::from(sim_math::bounded_share(own, total).0),
            i64::from(sim_math::bounded_share(ahead, rival_count).0),
            i64::from(sim_math::signed_relation(own, strongest).0),
            i64::from(sim_math::signed_relation(own, median).0),
            i64::from(sim_math::bounded_share(leader, total).0),
            i64::from(sim_math::bounded_share(concentration, i64::from(Fix32::ONE.0)).0),
            i64::from(self.confidence.0),
        ]
    }

    /// Returns the seat that holds the largest value, or the reader when
    /// every value is equal.
    ///
    /// The scan runs over the seats in ascending order and keeps the first
    /// largest, so two runs name one seat.
    fn leader(&self) -> usize {
        let mut best = 0usize;
        for (at, value) in self.values.iter().enumerate() {
            if *value > self.values[best] {
                best = at;
            }
        }
        best
    }
}

/// What one pass over the ground the faction observed accumulated.
///
/// **The cost of this pass follows the observed area and never the world.** A
/// block the faction has never seen a tile of answers from the block form of
/// its fog layer, so the walk visits the ground the faction has walked.[^1]
///
/// # References
///
/// [^1]: ADR-0059, fog storage grows with observed area, not with world area, decision D2. `docs/adrs/accepted/adr-0059-fog-storage-grows-with-observed-area.md`
#[derive(Clone, Debug, Default)]
struct GroundScan {
    observed_tiles: i64,
    observed_passable: i64,
    seen_now_tiles: i64,
    observed_fire: i64,
    observed_water: i64,
    held_seen_now: Vec<i64>,
    units_seen_now: Vec<i64>,
    strength_seen_now: Vec<i64>,
    own_units_seen: i64,
    own_idle_units: i64,
    own_type_counts: [i64; UNIT_TYPE_COUNT],
    own_held_seen_now: i64,
    own_held_fire: i64,
    own_held_water: i64,
    centre_q: i64,
    centre_r: i64,
    border_tiles: i64,
    contested_border_tiles: i64,
}

/// What one pass over the settlements of the world accumulated.
///
/// The walk runs over the settlement arena in ascending slot order. Its cost
/// follows the settlement count and never the world area. A settlement of a
/// rival counts for the reader only where the reader has walked.
#[derive(Clone, Debug, Default)]
struct SettlementScan {
    own_settlements: i64,
    own_population: i64,
    own_reach: i64,
    own_hazard_settlements: i64,
    own_stock: [i64; COMMODITY_COUNT],
    own_production: [i64; COMMODITY_COUNT],
    own_consumption: [i64; COMMODITY_COUNT],
    settlements_seen: Vec<i64>,
}

/// What one pass over the sparse upgrade map accumulated.
///
/// The map holds one entry for each improved tile and nothing else, so the
/// walk is not a walk over the tiles.
#[derive(Clone, Debug, Default)]
struct UpgradeScan {
    own_finished: i64,
    own_underway: i64,
    finished_seen: Vec<i64>,
}

/// What one pass over the character arena accumulated.
///
/// **Renown carries no fog rule**, so the best renown of every faction is
/// read as a public quantity.
#[derive(Clone, Debug, Default)]
struct CharacterScan {
    own_live: i64,
    best_renown: Vec<i64>,
}

impl World {
    /// Walks the ground one faction observed, and accumulates every
    /// fog-dependent quantity of the observation in one pass.
    ///
    /// **The walk visits the ground the faction has walked and never the
    /// world.** It reads the two fog layers of the faction once, then visits
    /// the cells that at least one layer names, in ascending cell order. A
    /// cell that neither layer names costs no tile work.[^1]
    ///
    /// A tile the faction only remembers contributes the ground alone. It
    /// adds no unit and no holder, because each of those is a fact of the
    /// present frame.[^2]
    ///
    /// # Errors
    ///
    /// Returns an error when the derived unit structure does not describe the
    /// units.
    ///
    /// # References
    ///
    /// [^1]: ADR-0059, fog storage grows with observed area, not with world area, decision D2. `docs/adrs/accepted/adr-0059-fog-storage-grows-with-observed-area.md`
    /// [^2]: ADR-0059, fog storage grows with observed area, not with world area, decision D4. `docs/adrs/accepted/adr-0059-fog-storage-grows-with-observed-area.md`
    fn scan_ground(
        &self,
        faction: FactionId,
        seats: usize,
    ) -> Result<GroundScan, FactionViewError> {
        let observation = self.observation();
        let layout = observation.layout();
        let grid = self.grid();
        let mut scan = GroundScan {
            held_seen_now: vec![0; seats],
            units_seen_now: vec![0; seats],
            strength_seen_now: vec![0; seats],
            ..GroundScan::default()
        };
        let visible = observation.visible_layer(faction);
        let remembered = observation.remembered_layer(faction);
        let observed = merge(
            visible.map_or(&[][..], |layer| layer.populated_blocks()),
            remembered.map_or(&[][..], |layer| layer.populated_blocks()),
        );
        let edge = layout.block_edge();
        for block in observed {
            let mask = BlockMask::new(
                visible.and_then(|layer| layer.block(block)),
                remembered.and_then(|layer| layer.block(block)),
            );
            if mask.is_empty() {
                continue;
            }
            let first_column = (block % layout.blocks_wide()) * edge;
            let first_row = (block / layout.blocks_wide()) * edge;
            let last_column = (first_column + edge).min(grid.width());
            let last_row = (first_row + edge).min(grid.height());
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
                    if !sees_now && !mask.saw_once(offset) {
                        continue;
                    }
                    scan.observed_tiles += 1;
                    if self.admits_a_unit(here) {
                        scan.observed_passable += 1;
                    }
                    if !sees_now {
                        continue;
                    }
                    scan.seen_now_tiles += 1;
                    let burning = matches!(self.tile_is_burning(here), Some(true));
                    let wet = matches!(self.ground_is_wet(here), Some(true));
                    if burning {
                        scan.observed_fire += 1;
                    }
                    if wet {
                        scan.observed_water += 1;
                    }
                    if let Some(holder) = self.tile_holder(here).and_then(Holder::faction) {
                        if let Some(place) = scan.held_seen_now.get_mut(usize::from(holder.0)) {
                            *place += 1;
                        }
                        if holder == faction {
                            self.take_own_tile(&mut scan, faction, here, burning, wet);
                        }
                    }
                    self.take_units_on(&mut scan, faction, tile);
                }
            }
        }
        Ok(scan)
    }
}

impl World {
    /// Accumulates the units that stand on one tile the faction sees.
    ///
    /// The walk reads the derived unit structure for the tile and the faction
    /// column of each identity on it. It is bounded by the units that stand
    /// on the observed tiles, and never by the population of the world.
    fn take_units_on(&self, scan: &mut GroundScan, faction: FactionId, tile: TileIdx) {
        for unit in self.bridge().on_tile_unguarded(tile) {
            let Some(owner) = self.soldiers().faction(*unit) else {
                continue;
            };
            if let Some(place) = scan.units_seen_now.get_mut(usize::from(owner.0)) {
                *place += 1;
            }
            let class = self.unit_type(*unit);
            if let Some(class) = class {
                if let Some(place) = scan.strength_seen_now.get_mut(usize::from(owner.0)) {
                    let one = self.unit_types().strength(class);
                    *place = sim_math::combine(Accum(*place), Accum(i64::from(one.0))).0;
                }
            }
            if owner != faction {
                continue;
            }
            scan.own_units_seen += 1;
            if let Some(class) = class {
                if let Some(place) = scan.own_type_counts.get_mut(class.index()) {
                    *place += 1;
                }
            }
            if self.unit_is_idle(*unit) {
                scan.own_idle_units += 1;
            }
        }
    }

    /// Accumulates what one tile of the faction's own ground contributes.
    ///
    /// **The border test asks whether the faction holds the neighbour, and
    /// never who else does.** Whether the faction holds a place is its own
    /// fact, so the test leaks nothing. The contested test asks who holds the
    /// neighbour, and it therefore runs only where the faction sees the
    /// neighbour this frame.
    ///
    /// A neighbour outside the world counts as ground the faction does not
    /// hold, so the edge of the world is a border.
    fn take_own_tile(
        &self,
        scan: &mut GroundScan,
        faction: FactionId,
        here: Axial,
        burning: bool,
        wet: bool,
    ) {
        scan.own_held_seen_now += 1;
        scan.centre_q += i64::from(here.q);
        scan.centre_r += i64::from(here.r);
        if burning {
            scan.own_held_fire += 1;
        }
        if wet {
            scan.own_held_water += 1;
        }
        let mut border = false;
        let mut contested = false;
        for step in self.grid().neighbours(here) {
            let Some(there) = step else {
                border = true;
                continue;
            };
            if self.holds(faction, there) == Some(true) {
                continue;
            }
            border = true;
            if !self.faction_sees_now(faction, there) {
                continue;
            }
            if self.tile_holder(there).and_then(Holder::faction).is_some() {
                contested = true;
            }
        }
        if border {
            scan.border_tiles += 1;
        }
        if contested {
            scan.contested_border_tiles += 1;
        }
    }

    /// Reports whether one unit stands under no order.
    ///
    /// A unit is idle when it gathers nothing, builds nothing and travels to
    /// no destination. **This is the one statement of that rule**, and both
    /// the idle share of block A and the free share of block K read it.
    fn unit_is_idle(&self, unit: Entity) -> bool {
        matches!(self.gather_order(unit), Some(None))
            && matches!(self.build_order(unit), Some(None))
            && matches!(self.sent_to(unit), Some(None))
    }
}

impl World {
    /// Walks the settlement arena once, in ascending slot order.
    ///
    /// The cost follows the settlement count and never the world area. A
    /// settlement of a rival counts for the reader only where the reader has
    /// walked, so the count is an estimate from below.
    fn scan_settlements(&self, faction: FactionId, seats: usize) -> SettlementScan {
        let mut scan = SettlementScan {
            settlements_seen: vec![0; seats],
            ..SettlementScan::default()
        };
        let settlements = self.settlements();
        for site in settlements.iter() {
            let Some(owner) = settlements.faction(site) else {
                continue;
            };
            let Some(address) = settlements.address(site) else {
                continue;
            };
            if owner != faction {
                let seen = self.faction_has_seen(faction, address);
                if let Some(place) = scan
                    .settlements_seen
                    .get_mut(usize::from(owner.0))
                    .filter(|_| seen)
                {
                    *place += 1;
                }
                continue;
            }
            scan.own_settlements += 1;
            scan.own_population += i64::from(self.site_residents(site).unwrap_or(0));
            scan.own_reach = scan
                .own_reach
                .max(i64::from(self.city_reach(site).unwrap_or(0)));
            if matches!(self.tile_is_burning(address), Some(true)) {
                scan.own_hazard_settlements += 1;
            }
            for commodity in 0..COMMODITY_COUNT {
                let id = CommodityId(commodity as u16);
                if let Some(held) = self.settlement_store(site, id) {
                    scan.own_stock[commodity] += i64::from(held.0);
                }
                if let Some(rate) = self.production_rate(site, id) {
                    scan.own_production[commodity] += i64::from(rate.0);
                }
                if let Some(rate) = self.upkeep_rate(site, id) {
                    scan.own_consumption[commodity] += i64::from(rate.0);
                }
            }
        }
        if let Some(place) = scan.settlements_seen.get_mut(usize::from(faction.0)) {
            *place = scan.own_settlements;
        }
        scan
    }

    /// Walks the sparse upgrade map once.
    ///
    /// The map holds one entry for each improved tile, so the walk is not a
    /// walk over the tiles. An upgrade of a rival counts for the reader only
    /// where the reader has walked.
    fn scan_upgrades(&self, faction: FactionId, seats: usize) -> UpgradeScan {
        let mut scan = UpgradeScan {
            finished_seen: vec![0; seats],
            ..UpgradeScan::default()
        };
        let grid = self.grid();
        for site in self.upgrade_sites() {
            let Some(address) = grid.address_of(site.tile) else {
                continue;
            };
            let Some(owner) = self.tile_holder(address).and_then(Holder::faction) else {
                continue;
            };
            let finished = site.level > 0;
            if owner == faction {
                if finished {
                    scan.own_finished += 1;
                }
                if site.progress.0 > 0 {
                    scan.own_underway += 1;
                }
                continue;
            }
            let seen = finished && self.faction_has_seen(faction, address);
            if let Some(place) = scan
                .finished_seen
                .get_mut(usize::from(owner.0))
                .filter(|_| seen)
            {
                *place += 1;
            }
        }
        if let Some(place) = scan.finished_seen.get_mut(usize::from(faction.0)) {
            *place = scan.own_finished;
        }
        scan
    }

    /// Walks the character arena once, in ascending slot order.
    ///
    /// **Renown carries no fog rule.** A character is not a fact of a tile,
    /// and the renown reader ends the game on the highest live renown, so the
    /// layout treats the value as public.
    fn scan_characters(&self, faction: FactionId, seats: usize) -> CharacterScan {
        let mut scan = CharacterScan {
            best_renown: vec![0; seats],
            ..CharacterScan::default()
        };
        let characters = self.characters();
        for entity in characters.iter() {
            let Some(owner) = characters.faction(entity) else {
                continue;
            };
            if owner == faction {
                scan.own_live += 1;
            }
            let renown = characters.renown(entity).unwrap_or(Fix32::ZERO);
            if let Some(place) = scan.best_renown.get_mut(usize::from(owner.0)) {
                *place = (*place).max(i64::from(renown.0));
            }
        }
        scan
    }

    /// Reports whether one faction holds the seat it started on.
    ///
    /// **A seat holder carries no fog rule**, because the reader that ends
    /// the game reads the same column.
    fn faction_holds_its_own_seat(&self, faction: FactionId) -> bool {
        let Some(tile) = self.seat(faction) else {
            return false;
        };
        let Some(address) = self.grid().address_of(tile) else {
            return false;
        };
        self.tile_holder(address).and_then(Holder::faction) == Some(faction)
    }

    /// Returns who holds the seats, in one walk over the seats.
    ///
    /// A seat is the tile of the first founding of a faction. **A seat holder
    /// carries no fog rule**, because the reader that ends the game reads the
    /// same column.
    ///
    /// **The walk answers two questions at once, so the layout states the
    /// seat rule once.** The whole count feeds the rival seat counts, which
    /// hold a seat whose faction has left the game. The live count feeds the
    /// domination track, whose denominator is the ask of the reader.
    fn seat_holding(&self, seats: usize) -> SeatHolding {
        let mut holding = SeatHolding {
            held: vec![0i64; seats],
            live_held: vec![0i64; seats],
            live_seats: 0,
        };
        for seat in 0..seats {
            let subject = FactionId(seat as u16);
            let Some(tile) = self.seat(subject) else {
                continue;
            };
            let live = !self.is_eliminated(subject);
            if live {
                holding.live_seats += 1;
            }
            let Some(address) = self.grid().address_of(tile) else {
                continue;
            };
            let Some(holder) = self.tile_holder(address).and_then(Holder::faction) else {
                continue;
            };
            let at = usize::from(holder.0);
            if let Some(place) = holding.held.get_mut(at) {
                *place += 1;
            }
            if live {
                if let Some(place) = holding.live_held.get_mut(at) {
                    *place += 1;
                }
            }
        }
        holding
    }

    /// Returns the campaign objective of one faction, as the fog admits it.
    ///
    /// **The engine holds one reader of this, and it is the reader the
    /// legality answer of the campaign verb reads.** That reader admits a
    /// settlement the faction has seen, ranks a relief before a take, and
    /// takes the nearest from the seat of the faction. A second walk here
    /// would state the same rank rule twice.[^1] [^2]
    ///
    /// The first entry of that answer is the objective over the whole frame,
    /// and this reads it. A relief is an objective whose settlement belongs
    /// to the reader.
    ///
    /// The walk is over the settlement arena and over the cells of the frame,
    /// so its cost follows the settlement count and a fixed cell count. It
    /// walks no tile and no unit.
    ///
    /// # References
    ///
    /// [^1]: ADR-0199, a verb names a place by a cell of the egocentric frame the observation publishes, decision D3. `docs/adrs/draft/adr-0199-a-verb-names-a-place-by-a-cell-of-the-egocentric-frame.md`
    /// [^2]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
    fn observed_march_target(&self, faction: FactionId) -> Option<MarchTarget> {
        let tile = self
            .observed_campaign_objectives(faction)
            .first()
            .copied()
            .flatten()?;
        let address = self.grid().address_of(tile)?;
        let seat = self
            .seat(faction)
            .and_then(|at| self.grid().address_of(at))?;
        let owner = self
            .settlement_on(address)
            .and_then(|site| self.settlement_faction(site));
        Some(MarchTarget {
            distance: seat.distance(address),
            relief: owner == Some(faction),
        })
    }
}

/// Who holds the seats of the world, from one walk over them.
struct SeatHolding {
    /// The seats each faction holds, by faction number, whether the faction
    /// that started on a seat is still in the game or not.
    held: Vec<i64>,
    /// The seats of factions that are still in the game, by the faction
    /// number of the holder.
    live_held: Vec<i64>,
    /// The seats the domination reader asks a candidate to hold.
    live_seats: i64,
}

/// The campaign objective one faction has observed.
struct MarchTarget {
    /// The hex distance from the seat of the faction to the objective.
    distance: u32,
    /// Whether the objective is a settlement of the faction itself.
    relief: bool,
}

/// Returns the good class of one resource kind.
///
/// **The work commodity table is the one declaration of this mapping.** It
/// says which commodity a site holds more of when one kind of work is done,
/// and the layout groups a good by that commodity. A second mapping here
/// would be a redundant declaration site.
fn good_class_of(kind: ResourceKind) -> usize {
    WORK_COMMODITY[kind.index()].0 as usize
}

/// Returns one relation value as a signed share of the alliance edge.
///
/// A relation is a whole signed integer, and the rules of the world name the
/// edge at which it becomes an alliance. The layout divides by that edge, so
/// a relation at the alliance edge reads one and a relation at the negative
/// of it reads minus one. **This is the one statement of that scale**, and
/// every relation slot of the layout reads it.
fn relation_share(value: i32, edge: i64) -> i64 {
    let magnitude = sim_math::bounded_share(i64::from(value).abs(), edge);
    if value < 0 {
        -i64::from(magnitude.0)
    } else {
        i64::from(magnitude.0)
    }
}

/// Returns the mean of a sum of bounded values.
///
/// The mean of values that each lie in the unit range lies in the unit range,
/// so the result needs no clamp. A count of zero gives zero, because a mean
/// over nothing states nothing.
fn mean_of(sum: i64, count: i64) -> i64 {
    if count <= 0 {
        return 0;
    }
    sim_math::divide_by_count(Accum(sum), count as u32).map_or(0, |value| i64::from(value.0))
}

/// The summary of the diplomacy of one faction, in the order block H holds
/// it.
///
/// **No member of this struct names a seat.** Each one is a mean, a minimum,
/// a maximum or a share over the rivals of the reader.
#[derive(Clone, Copy, Debug, Default)]
struct Diplomacy {
    mean_out: i64,
    min_out: i64,
    max_out: i64,
    mean_in: i64,
    min_in: i64,
    asymmetry: i64,
    war_share: i64,
    peace_share: i64,
    alliance_share: i64,
    rival_pair_mean: i64,
    rival_pair_war_share: i64,
    to_leader: i64,
    from_leader: i64,
    contract_share: i64,
    committed_stock: i64,
}

impl World {
    /// Returns the hex radius of the world.
    ///
    /// The world is a rhombus of tiles. The radius is half of the longer
    /// axis, which is the steps a unit at the centre takes to reach the far
    /// edge along that axis. It is a structural property of the grid and it
    /// is not a measured figure.
    fn world_radius(&self) -> i64 {
        let grid = self.grid();
        (i64::from(grid.width().max(grid.height())) / 2).max(1)
    }

    /// Returns the diplomacy summary of one faction.
    ///
    /// The pass visits the rivals in ascending seat order, and it visits the
    /// ordered rival pairs in ascending seat order of each member. Every
    /// answer is a mean, a minimum, a maximum or a share, so the order does
    /// not change one of them, and the order is stated for determinism.
    ///
    /// **Diplomacy carries no fog rule in this engine.** The relation plane
    /// holds one value for each ordered pair, and no reader masks it, so the
    /// layout treats every relation as public.
    fn diplomacy(&self, faction: FactionId, seats: usize, leader: usize) -> Diplomacy {
        let edge = i64::from(self.relation_rules().alliance_edge).abs().max(1);
        let alliance_band = crate::relation::BAND_COUNT - 1;
        let mut out = Diplomacy::default();
        let mut sum_out = 0i64;
        let mut sum_in = 0i64;
        let mut sum_gap = 0i64;
        let mut lowest_out = i64::MAX;
        let mut highest_out = i64::MIN;
        let mut lowest_in = i64::MAX;
        let mut rivals = 0i64;
        let mut at_war = 0i64;
        let mut allied = 0i64;
        let mut contracted = 0i64;
        for seat in 0..seats {
            let rival = FactionId(seat as u16);
            if rival == faction {
                continue;
            }
            rivals += 1;
            let toward = relation_share(self.relation(faction, rival).unwrap_or(0), edge);
            let from = relation_share(self.relation(rival, faction).unwrap_or(0), edge);
            sum_out += toward;
            sum_in += from;
            sum_gap += toward - from;
            lowest_out = lowest_out.min(toward);
            highest_out = highest_out.max(toward);
            lowest_in = lowest_in.min(from);
            if self.at_war(faction, rival) {
                at_war += 1;
            }
            if self.relation_band(faction, rival) == Some(alliance_band) {
                allied += 1;
            }
            let owed = self.contract_debt(faction, rival);
            if owed.is_some() {
                contracted += 1;
            }
            out.committed_stock += owed.unwrap_or(0);
        }
        let mut pair_sum = 0i64;
        let mut pairs = 0i64;
        let mut pair_wars = 0i64;
        for first in 0..seats {
            for second in 0..seats {
                let (a, b) = (FactionId(first as u16), FactionId(second as u16));
                if a == faction || b == faction || a == b {
                    continue;
                }
                pairs += 1;
                pair_sum += relation_share(self.relation(a, b).unwrap_or(0), edge);
                if first < second && self.at_war(a, b) {
                    pair_wars += 1;
                }
            }
        }
        let leading = FactionId(leader as u16);
        out.mean_out = mean_of(sum_out, rivals);
        out.min_out = if lowest_out == i64::MAX {
            0
        } else {
            lowest_out
        };
        out.max_out = if highest_out == i64::MIN {
            0
        } else {
            highest_out
        };
        out.mean_in = mean_of(sum_in, rivals);
        out.min_in = if lowest_in == i64::MAX { 0 } else { lowest_in };
        out.asymmetry = mean_of(sum_gap, rivals);
        out.war_share = i64::from(sim_math::bounded_share(at_war, rivals).0);
        out.peace_share = i64::from(sim_math::bounded_share(rivals - at_war, rivals).0);
        out.alliance_share = i64::from(sim_math::bounded_share(allied, rivals).0);
        out.rival_pair_mean = mean_of(pair_sum, pairs);
        out.rival_pair_war_share = i64::from(sim_math::bounded_share(pair_wars, pairs / 2).0);
        out.to_leader = relation_share(self.relation(faction, leading).unwrap_or(0), edge);
        out.from_leader = relation_share(self.relation(leading, faction).unwrap_or(0), edge);
        out.contract_share = i64::from(sim_math::bounded_share(contracted, rivals).0);
        out
    }

    /// Returns what one faction still owes a rival on a bound contract, as a
    /// Q16.16 quantity, or `None` when the pair holds no bound contract.
    ///
    /// The board holds one row for each ordered pair. The faction that opened
    /// the pair owes the give amount, and the other party owes the take
    /// amount, so the reader picks the term its own side owes.
    fn contract_debt(&self, faction: FactionId, rival: FactionId) -> Option<i64> {
        if let Some(row) = self.trade_row(faction, rival) {
            if row.status == TRADE_BOUND {
                let owed = row.give_amount.saturating_sub(row.given);
                return Some(sim_math::scale_by_count(Fix32::ONE, owed).0);
            }
        }
        let row = self.trade_row(rival, faction)?;
        if row.status != TRADE_BOUND {
            return None;
        }
        let owed = row.take_amount.saturating_sub(row.taken);
        Some(sim_math::scale_by_count(Fix32::ONE, owed).0)
    }
}

/// The four statistics of one victory track that the engine can answer.
///
/// The two estimated tick counts of the track are reserved, because an
/// estimate divides the gap by a rate over a window and the engine carries
/// no window.
#[derive(Clone, Copy, Debug, Default)]
struct Track {
    progress: i64,
    leader: i64,
    gap: i64,
    rank: i64,
}

impl Track {
    /// Returns the statistics of one track from the value of every faction.
    ///
    /// The requirement is the value at which the reader of the track fires.
    /// A requirement of zero or below reads as one, so the progress share is
    /// total and it never divides by zero.
    fn of(values: &[i64], seat: usize, requirement: i64) -> Self {
        let own = values.get(seat).copied().unwrap_or(0);
        let mut strongest = 0i64;
        let mut leader = 0i64;
        let mut behind = 0i64;
        let mut rivals = 0i64;
        for (at, value) in values.iter().enumerate() {
            leader = leader.max(*value);
            if at == seat {
                continue;
            }
            rivals += 1;
            strongest = strongest.max(*value);
            if *value < own {
                behind += 1;
            }
        }
        Self {
            progress: i64::from(sim_math::bounded_share(own, requirement).0),
            leader: i64::from(sim_math::bounded_share(leader, requirement).0),
            gap: i64::from(sim_math::signed_relation(own, strongest).0),
            rank: i64::from(sim_math::bounded_share(behind, rivals).0),
        }
    }
}

impl World {
    /// Returns the board statistics of every good class, in class order.
    ///
    /// **The board is public, so no fog applies.** The pass visits the
    /// factions in ascending seat order and the rows of each board in
    /// ascending row order. Its cost follows the row count, which does not
    /// follow the world size.
    ///
    /// A price is the asking quantity over the offered quantity, held in
    /// Q16.16 and then compressed. The best price to buy a class is the
    /// lowest price among the rows that offer it. The best price to sell a
    /// class is the highest price among the rows that want it. The depth is
    /// the whole quantity the rows that offer it declare.
    ///
    /// The fifth statistic of each class is the price change over a window,
    /// and it reads zero.
    fn board_statistics(&self, seats: usize) -> Vec<i64> {
        let classes = GOOD_CLASS_COUNT as usize;
        let mut buy: Vec<Option<i64>> = vec![None; classes];
        let mut sell: Vec<Option<i64>> = vec![None; classes];
        let mut depth = vec![0i64; classes];
        for seat in 0..seats {
            for advert in self.market(FactionId(seat as u16)) {
                if advert.quantity == 0 {
                    continue;
                }
                let Some(kind) = ResourceKind::ALL.get(usize::from(advert.good)) else {
                    continue;
                };
                let class = good_class_of(*kind);
                let Some(price) = sim_math::share(
                    Accum(i64::from(Fix32::ONE.0)),
                    Accum(i64::from(advert.asking_quantity)),
                    Accum(i64::from(advert.quantity)),
                ) else {
                    continue;
                };
                if advert.wants == 0 {
                    if let Some(place) = depth.get_mut(class) {
                        *place += i64::from(advert.quantity);
                    }
                    if let Some(place) = buy.get_mut(class) {
                        *place = Some(place.map_or(price.0, |held| held.min(price.0)));
                    }
                } else if let Some(place) = sell.get_mut(class) {
                    *place = Some(place.map_or(price.0, |held| held.max(price.0)));
                }
            }
        }
        let mut out = vec![0i64; classes * BOARD_STATISTIC_COUNT as usize];
        for class in 0..classes {
            let best_buy = buy[class].unwrap_or(0);
            let best_sell = sell[class].unwrap_or(0);
            let base = class * BOARD_STATISTIC_COUNT as usize;
            out[base] = i64::from(sim_math::compressed_magnitude(best_buy).0);
            out[base + 1] = i64::from(sim_math::compressed_magnitude(best_sell).0);
            out[base + 2] = i64::from(sim_math::signed_relation(best_sell, best_buy).0);
            out[base + 3] = i64::from(sim_math::compressed_magnitude(depth[class]).0);
        }
        out
    }

    /// Returns one flag for each upgrade class the build verb accepts now.
    ///
    /// The pass reads the legality answer the engine already gives, and it
    /// finds the class of each action through the argument position whose
    /// candidate kind is a category. **It states no part of the action
    /// encoding of its own.**
    fn upgrade_legality(&self, legal: &[u8]) -> Vec<i64> {
        let mut flags = vec![0i64; UPGRADE_CLASS_COUNT as usize];
        let schema = self.action_schema();
        for row in schema.rows() {
            if row.verb != Verb::Build {
                continue;
            }
            for offset in 0..row.rows {
                let action = row.first + offset;
                if legal.get(action as usize).copied().unwrap_or(0) == 0 {
                    continue;
                }
                let Some((_, arguments)) = schema.decode(action) else {
                    continue;
                };
                for (position, value) in row.positions.iter().zip(&arguments) {
                    if position.candidate != CandidateKind::Category {
                        continue;
                    }
                    if let Some(place) = flags.get_mut(*value as usize) {
                        *place = i64::from(Fix32::ONE.0);
                    }
                }
            }
        }
        flags
    }

    /// Returns one when the legality answer admits one verb anywhere.
    ///
    /// A flag is a share that holds zero or one. The layout holds no Rust
    /// boolean, because an observation must be plain data with declared
    /// padding.
    fn verb_flag(&self, legal: &[u8], verb: Verb) -> i64 {
        let schema = self.action_schema();
        for row in schema.rows() {
            if row.verb != verb {
                continue;
            }
            for offset in 0..row.rows {
                if legal
                    .get((row.first + offset) as usize)
                    .copied()
                    .unwrap_or(0)
                    != 0
                {
                    return i64::from(Fix32::ONE.0);
                }
            }
        }
        0
    }
}

/// Every quantity one observation publishes, gathered before the array is
/// filled.
///
/// The passes that build this run first, and the writer then indexes the
/// array through the schema. **The writer computes nothing that a second
/// field could compute differently.**
struct Reading {
    seat: usize,
    ground: GroundScan,
    settlement: SettlementScan,
    upgrade: UpgradeScan,
    character: CharacterScan,
    diplomacy: Diplomacy,
    board: Vec<i64>,
    legality: Vec<i64>,
    settle_flag: i64,
    settlers: i64,
    held_tiles: i64,
    live_units: i64,
    strength: i64,
    store_total: i64,
    renown_total: i64,
    world_tiles: i64,
    world_passable: i64,
    world_radius: i64,
    reach_cap: i64,
    seated: i64,
    tick: i64,
    tick_limit: i64,
    board_rows: i64,
    board_free: i64,
    weights: Vec<i64>,
    rival_seats_held: i64,
    rival_seats: i64,
    march: Option<MarchTarget>,
    domination: Track,
    wonder: Track,
    renown: Track,
    ground_track: Track,
    power_held: PowerVector,
    power_settlements: PowerVector,
    power_units: PowerVector,
    power_strength: PowerVector,
    power_upgrades: PowerVector,
    power_renown: PowerVector,
    power_wonder: PowerVector,
    ring_stack: RingStack,
    frontier: Frontier,
    tokens: EntityTokens,
}

impl World {
    /// Gathers every quantity one observation publishes.
    ///
    /// **The own value of a power quantity is exact, and the value of a rival
    /// is what the faction observed.** The confidence of each fog-derived
    /// quantity is the ground the faction sees this frame over the ground it
    /// has ever seen, so a reader tells an unobserved estimate from a real
    /// zero.
    ///
    /// # Errors
    ///
    /// Returns an error when the derived unit structure does not describe the
    /// units.
    fn reading_of(&self, faction: FactionId) -> Result<Reading, FactionViewError> {
        let seats = usize::from(self.faction_count().max(1));
        let seat = usize::from(faction.0);
        let ground = self.scan_ground(faction, seats)?;
        let settlement = self.scan_settlements(faction, seats);
        let upgrade = self.scan_upgrades(faction, seats);
        let character = self.scan_characters(faction, seats);
        let confidence = sim_math::bounded_share(ground.seen_now_tiles, ground.observed_tiles);
        let exact = Fix32::ONE;

        let mut held = ground.held_seen_now.clone();
        if let Some(place) = held.get_mut(seat) {
            *place = self.holding_of(faction);
        }
        let mut units = ground.units_seen_now.clone();
        if let Some(place) = units.get_mut(seat) {
            *place = i64::from(self.population_of(faction));
        }
        let mut strengths = ground.strength_seen_now.clone();
        if let Some(place) = strengths.get_mut(seat) {
            *place = self.faction_strength(faction).0;
        }
        let wonder_work = self
            .victory_claims()
            .into_iter()
            .map(|pair| pair.1)
            .collect::<Vec<i64>>();
        let seat_holding = self.seat_holding(seats);
        let own_seat_held = i64::from(self.faction_holds_its_own_seat(faction));

        let world = self.pyramid().total();
        let requirement = self
            .upgrade_table()
            .row(UpgradeCategory::WONDER, upgrade::WONDER_LEVEL)
            .map_or(0, |row| i64::from(row.work));
        let renown_target = i64::from(self.balance().renown_target());
        let world_passable = world.open_tiles();

        let power_held = PowerVector {
            values: held.clone(),
            confidence,
        };
        let leader = power_held.leader();
        let powers = RivalPowers {
            held_tiles: &held,
            units: &units,
            strength: &strengths,
            upgrades: &upgrade.finished_seen,
            renown: &character.best_renown,
            wonder: &wonder_work,
            leader,
            confidence,
        };

        let ring_stack = self.faction_ring_stack(faction)?;
        let frontier = self.faction_frontier(faction, &ring_stack)?;
        let tokens = self.faction_entity_tokens(faction, &ring_stack, &frontier, &powers)?;
        let legal = self.legal_actions(faction).unwrap_or_default();
        let weights = self.faction_weights(faction).map_or_else(Vec::new, |set| {
            [set.war, set.trade, set.build, set.renown, set.settle]
                .into_iter()
                .map(|weight| {
                    i64::from(sim_math::bounded_share(i64::from(weight), i64::from(u8::MAX)).0)
                })
                .collect()
        });
        let board_rows = i64::from(self.board_rows());
        let board_used = self
            .market(faction)
            .iter()
            .filter(|advert| advert.quantity > 0)
            .count() as i64;

        Ok(Reading {
            seat,
            diplomacy: self.diplomacy(faction, seats, leader),
            board: self.board_statistics(seats),
            legality: self.upgrade_legality(&legal),
            settle_flag: self.verb_flag(&legal, Verb::Settle),
            settlers: i64::from(self.settler_count(faction).unwrap_or(0)),
            held_tiles: self.holding_of(faction),
            live_units: i64::from(self.population_of(faction)),
            strength: self.faction_strength(faction).0,
            store_total: settlement.own_stock.iter().sum(),
            renown_total: character.best_renown.iter().sum(),
            world_tiles: i64::from(self.grid().tile_count()),
            world_passable,
            world_radius: self.world_radius(),
            reach_cap: i64::from(self.reach_rules().cap()),
            seated: seats as i64,
            tick: self.tick().0 as i64,
            tick_limit: self.tick_limit() as i64,
            board_rows,
            board_free: board_rows - board_used,
            weights,
            rival_seats_held: seat_holding
                .held
                .get(seat)
                .copied()
                .unwrap_or(0)
                .saturating_sub(own_seat_held),
            rival_seats: (seats as i64 - 1).max(0),
            march: self.observed_march_target(faction),
            domination: Track::of(&seat_holding.live_held, seat, seat_holding.live_seats),
            wonder: Track::of(&wonder_work, seat, requirement),
            renown: Track::of(&character.best_renown, seat, renown_target),
            ground_track: Track::of(&held, seat, world_passable),
            power_held,
            power_settlements: PowerVector {
                values: settlement.settlements_seen.clone(),
                confidence,
            },
            power_units: PowerVector {
                values: units,
                confidence,
            },
            power_strength: PowerVector {
                values: strengths,
                confidence,
            },
            power_upgrades: PowerVector {
                values: upgrade.finished_seen.clone(),
                confidence,
            },
            power_renown: PowerVector {
                values: character.best_renown.clone(),
                confidence: exact,
            },
            power_wonder: PowerVector {
                values: wonder_work,
                confidence: exact,
            },
            ground,
            settlement,
            upgrade,
            character,
            ring_stack,
            frontier,
            tokens,
        })
    }
}

/// Widens a bounded share into the width the array holds.
fn share(part: i64, whole: i64) -> i64 {
    i64::from(sim_math::bounded_share(part, whole).0)
}

/// Widens a compressed magnitude into the width the array holds.
fn magnitude(value: i64) -> i64 {
    i64::from(sim_math::compressed_magnitude(value).0)
}

/// Widens a signed relation into the width the array holds.
fn relation(a: i64, b: i64) -> i64 {
    i64::from(sim_math::signed_relation(a, b).0)
}

/// Widens a flag into the width the array holds.
///
/// A flag is a share that holds zero or one, and no position of this layout
/// holds any other kind of flag.
fn flag(set: bool) -> i64 {
    if set {
        i64::from(Fix32::ONE.0)
    } else {
        0
    }
}

/// Widens one quadrature of a cyclic phase into the width the array holds.
fn triangle(phase: i64, period: i64) -> i64 {
    i64::from(sim_math::phase_triangle(phase, period).0)
}

/// Writes a value into every position of a span, by class index.
fn fill(span: &mut [i64], read: impl Fn(usize) -> i64) {
    for (class, place) in span.iter_mut().enumerate() {
        *place = read(class);
    }
}

/// Merges two ascending cell lists into one ascending list without repeats.
fn merge(first: &[u32], second: &[u32]) -> Vec<u32> {
    let mut out = Vec::with_capacity(first.len() + second.len());
    let (mut left, mut right) = (0usize, 0usize);
    while left < first.len() && right < second.len() {
        match first[left].cmp(&second[right]) {
            std::cmp::Ordering::Less => {
                out.push(first[left]);
                left += 1;
            }
            std::cmp::Ordering::Greater => {
                out.push(second[right]);
                right += 1;
            }
            std::cmp::Ordering::Equal => {
                out.push(first[left]);
                left += 1;
                right += 1;
            }
        }
    }
    out.extend_from_slice(&first[left..]);
    out.extend_from_slice(&second[right..]);
    out
}

impl World {
    /// Returns the observation of one faction, as one flat array.
    ///
    /// The array holds what that faction observes, and nothing else. **No
    /// argument widens the answer.** A caller that wants the truth of the
    /// world calls a reader that names no faction.[^1]
    ///
    /// The schema of this layout declares where every field of the array
    /// sits.[^2] **The length is one number for every world**, because no
    /// field of the layout follows the world shape, the faction count or the
    /// population.[^3]
    ///
    /// Every position holds a Q16.16 value between minus one and one. A field
    /// the engine cannot answer reads zero, and the schema declares its
    /// bounds as zero and zero.
    ///
    /// # Errors
    ///
    /// Returns an error when the number names no faction of this world, and
    /// when the derived unit structure does not describe the units. **The
    /// error names the cause.**
    ///
    /// # References
    ///
    /// [^1]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decision D3. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
    /// [^2]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decision D1. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
    /// [^3]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decision D2. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
    pub fn faction_observation(&self, faction: FactionId) -> Result<Vec<i64>, FactionViewError> {
        if faction.0 >= self.faction_count().max(1) {
            return Err(FactionViewError::NoSuchFaction(faction));
        }
        let schema = observation_schema();
        let read = self.reading_of(faction)?;
        let ground = &read.ground;
        let site = &read.settlement;
        let radius = site.own_reach;
        let reach_area = 3 * radius * (radius + 1) + 1;
        let own_renown = read
            .character
            .best_renown
            .get(read.seat)
            .copied()
            .unwrap_or(0);
        let food = good_class_of(ResourceKind::Food);
        let coverage = food_coverage(site, food);
        let centre = self.centroid_offset(ground);

        let mut out = vec![0i64; schema.length() as usize];
        for row in schema.rows() {
            let first = row.start as usize;
            let span = &mut out[first..first + row.positions as usize];
            match row.field {
                ObsField::MemoryRecent => {
                    memory::write_shares(self, faction, Decay::Recent, span);
                }
                ObsField::MemoryLasting => {
                    memory::write_shares(self, faction, Decay::Lasting, span);
                }
                ObsField::MemoryTrend => memory::write_trends(self, faction, span),
                ObsField::MemoryWorstRival => memory::write_worst_rival(self, faction, span),
                ObsField::MemoryConcentration => {
                    memory::write_concentration(self, faction, span);
                }
                ObsField::MilitaryStrength => span[0] = magnitude(read.strength),
                ObsField::StrengthForEachUnit => {
                    span[0] = magnitude(read.strength / read.live_units.max(1));
                }
                ObsField::TilesLost => {
                    span[0] =
                        memory::kind_share(self, faction, MemoryKind::OwnGroundLost, Decay::Recent);
                }
                ObsField::SettlementsLost => {
                    span[0] =
                        memory::kind_share(self, faction, MemoryKind::OwnSitesLost, Decay::Recent);
                }
                ObsField::UnitsLostToHazard => {
                    span[0] = memory::kind_share(
                        self,
                        faction,
                        MemoryKind::OwnUnitsBurned,
                        Decay::Recent,
                    );
                }
                ObsField::RingStack => span.copy_from_slice(read.ring_stack.slots()),
                ObsField::FrontierBySector => span.copy_from_slice(read.frontier.slots()),
                ObsField::TokenOwnSettlements => {
                    span.copy_from_slice(read.tokens.set_slots(TokenSet::Settlements));
                }
                ObsField::TokenRivals => {
                    span.copy_from_slice(read.tokens.set_slots(TokenSet::Rivals));
                }
                ObsField::TokenThreatClusters => {
                    span.copy_from_slice(read.tokens.set_slots(TokenSet::Threats));
                }
                ObsField::TokenCandidateSites => {
                    span.copy_from_slice(read.tokens.set_slots(TokenSet::Sites));
                }
                ObsField::HeldTiles => span[0] = magnitude(read.held_tiles),
                ObsField::HeldShareObserved => {
                    span[0] = share(read.held_tiles, ground.observed_passable);
                }
                ObsField::HeldShareWorld => span[0] = share(read.held_tiles, read.world_passable),
                ObsField::LiveUnits => span[0] = magnitude(read.live_units),
                ObsField::UnitsForEachHeldTile => {
                    span[0] = share(read.live_units, read.held_tiles * UNITS_FOR_EACH_TILE_SCALE);
                }
                ObsField::Settlements => span[0] = magnitude(site.own_settlements),
                ObsField::Population => span[0] = magnitude(site.own_population),
                ObsField::PopulationForEachSettlement => {
                    span[0] = magnitude(site.own_population / site.own_settlements.max(1));
                }
                ObsField::FinishedUpgrades => span[0] = magnitude(read.upgrade.own_finished),
                ObsField::UpgradesUnderway => span[0] = magnitude(read.upgrade.own_underway),
                ObsField::UpgradesForEachSettlement => {
                    span[0] = magnitude(read.upgrade.own_finished / site.own_settlements.max(1));
                }
                ObsField::ReachRadius => span[0] = magnitude(radius),
                ObsField::ReachRadiusShare => span[0] = share(radius, read.world_radius),
                ObsField::ReachAreaShare => span[0] = share(reach_area, read.world_passable),
                ObsField::StockOfClass => fill(span, |class| magnitude(stock_of(site, class))),
                ObsField::StockShareOfClass => {
                    fill(span, |class| share(stock_of(site, class), read.store_total));
                }
                ObsField::NetFlowOfClass => fill(span, |class| {
                    relation(production_of(site, class), consumption_of(site, class))
                }),
                ObsField::ProductionOfClass => {
                    fill(span, |class| magnitude(production_of(site, class)));
                }
                ObsField::ConsumptionOfClass => {
                    fill(span, |class| magnitude(consumption_of(site, class)));
                }
                ObsField::StoreTotal => span[0] = magnitude(read.store_total),
                ObsField::UnitShareOfClass => fill(span, |class| {
                    let held = ground.own_type_counts.get(class).copied().unwrap_or(0);
                    share(held, ground.own_units_seen)
                }),
                ObsField::IdleUnitShare | ObsField::FreeUnitShare => {
                    span[0] = share(ground.own_idle_units, ground.own_units_seen);
                }
                ObsField::FoodCoverageTicks => span[0] = magnitude(coverage),
                ObsField::BestRenown => span[0] = magnitude(own_renown),
                ObsField::BestRenownShare => span[0] = share(own_renown, read.renown_total),
                ObsField::LiveCharacters => span[0] = magnitude(read.character.own_live),
                ObsField::ObservedShareWorld => {
                    span[0] = share(ground.observed_tiles, read.world_tiles);
                }
                ObsField::ObservedPassableShare => {
                    span[0] = share(ground.observed_passable, ground.observed_tiles);
                }
                ObsField::HeldUnderHazardShare => {
                    span[0] = share(ground.own_held_fire, ground.own_held_seen_now);
                }
                ObsField::SettlementsUnderHazardShare => {
                    span[0] = share(site.own_hazard_settlements, site.own_settlements);
                }
                ObsField::WorldTiles => span[0] = magnitude(read.world_tiles),
                ObsField::WorldPassableTiles => span[0] = magnitude(read.world_passable),
                ObsField::SeatedFactionShare => span[0] = share(read.seated, FACTION_CEILING),
                ObsField::CentroidOffset => span[0] = share(centre, read.world_radius),
                ObsField::BorderLength => span[0] = magnitude(ground.border_tiles),
                ObsField::ContestedBorderShare => {
                    span[0] = share(ground.contested_border_tiles, ground.border_tiles);
                }
                ObsField::WonderProgress => span[0] = read.wonder.progress,
                ObsField::TickShare => {
                    span[0] = if read.tick_limit > 0 {
                        share(read.tick, read.tick_limit)
                    } else {
                        0
                    };
                }
                ObsField::RemainingTicks => {
                    span[0] = magnitude((read.tick_limit - read.tick).max(0));
                }
                ObsField::WeatherPhase => {
                    span[0] = triangle(read.tick, SEASON_PERIOD_TICKS);
                    span[1] = triangle(read.tick + SEASON_PERIOD_TICKS / 4, SEASON_PERIOD_TICKS);
                }
                ObsField::DominationProgress => span[0] = read.domination.progress,
                ObsField::DominationLeader => span[0] = read.domination.leader,
                ObsField::DominationGap => span[0] = read.domination.gap,
                ObsField::DominationRank => span[0] = read.domination.rank,
                ObsField::WonderTrackProgress => span[0] = read.wonder.progress,
                ObsField::WonderTrackLeader => span[0] = read.wonder.leader,
                ObsField::WonderTrackGap => span[0] = read.wonder.gap,
                ObsField::WonderTrackRank => span[0] = read.wonder.rank,
                ObsField::RenownProgress => span[0] = read.renown.progress,
                ObsField::RenownLeader => span[0] = read.renown.leader,
                ObsField::RenownGap => span[0] = read.renown.gap,
                ObsField::RenownRank => span[0] = read.renown.rank,
                ObsField::GroundProgress => span[0] = read.ground_track.progress,
                ObsField::GroundLeader => span[0] = read.ground_track.leader,
                ObsField::GroundGap => span[0] = read.ground_track.gap,
                ObsField::GroundRank => span[0] = read.ground_track.rank,
                ObsField::PowerHeldTiles => {
                    span.copy_from_slice(&read.power_held.statistics(read.seat));
                }
                ObsField::PowerSettlements => {
                    span.copy_from_slice(&read.power_settlements.statistics(read.seat));
                }
                ObsField::PowerUnits => {
                    span.copy_from_slice(&read.power_units.statistics(read.seat));
                }
                ObsField::PowerStrength => {
                    span.copy_from_slice(&read.power_strength.statistics(read.seat));
                }
                ObsField::PowerUpgrades => {
                    span.copy_from_slice(&read.power_upgrades.statistics(read.seat));
                }
                ObsField::PowerRenown => {
                    span.copy_from_slice(&read.power_renown.statistics(read.seat));
                }
                ObsField::PowerWonder => {
                    span.copy_from_slice(&read.power_wonder.statistics(read.seat));
                }
                ObsField::TradeBoard => span.copy_from_slice(&read.board),
                ObsField::RelationMeanOut => span[0] = read.diplomacy.mean_out,
                ObsField::RelationMinOut => span[0] = read.diplomacy.min_out,
                ObsField::RelationMaxOut => span[0] = read.diplomacy.max_out,
                ObsField::RelationMeanIn => span[0] = read.diplomacy.mean_in,
                ObsField::RelationMinIn => span[0] = read.diplomacy.min_in,
                ObsField::RelationAsymmetry => span[0] = read.diplomacy.asymmetry,
                ObsField::WarShare => span[0] = read.diplomacy.war_share,
                ObsField::PeaceShare => span[0] = read.diplomacy.peace_share,
                ObsField::AllianceShare => span[0] = read.diplomacy.alliance_share,
                ObsField::RivalPairRelationMean => span[0] = read.diplomacy.rival_pair_mean,
                ObsField::RivalPairWarShare => span[0] = read.diplomacy.rival_pair_war_share,
                ObsField::RelationToLeader => span[0] = read.diplomacy.to_leader,
                ObsField::RelationFromLeader => span[0] = read.diplomacy.from_leader,
                ObsField::ContractShare => span[0] = read.diplomacy.contract_share,
                ObsField::WaterShareObserved => {
                    span[0] = share(ground.observed_water, ground.observed_tiles);
                }
                ObsField::FireShareObserved => {
                    span[0] = share(ground.observed_fire, ground.observed_tiles);
                }
                ObsField::WaterShareHeld => {
                    span[0] = share(ground.own_held_water, ground.own_held_seen_now);
                }
                ObsField::FireShareHeld => {
                    span[0] = share(ground.own_held_fire, ground.own_held_seen_now);
                }
                ObsField::MayFound => span[0] = read.settle_flag,
                ObsField::Settlers => span[0] = magnitude(read.settlers),
                ObsField::UpgradeLegality => {
                    fill(span, |class| read.legality.get(class).copied().unwrap_or(0));
                }
                ObsField::CommittedStockShare => {
                    span[0] = share(read.diplomacy.committed_stock, read.store_total);
                }
                ObsField::BoardRowHeadroom => span[0] = share(read.board_free, read.board_rows),
                ObsField::ReachHeadroomShare => {
                    span[0] = share((read.reach_cap - radius).max(0), read.reach_cap);
                }
                ObsField::RivalSeatsHeld => span[0] = magnitude(read.rival_seats_held),
                ObsField::RivalSeats => span[0] = magnitude(read.rival_seats),
                ObsField::CampaignObjectiveDistance => {
                    span[0] = read.march.as_ref().map_or(0, |march| {
                        i64::from(distance_share(self.grid(), march.distance).0)
                    });
                }
                ObsField::CampaignObjectiveFlag => {
                    span[0] = flag(read.march.is_some());
                }
                ObsField::CampaignObjectiveRelief => {
                    span[0] = flag(read.march.as_ref().is_some_and(|march| march.relief));
                }
                ObsField::ObjectiveWeight => {
                    fill(span, |element| {
                        read.weights.get(element).copied().unwrap_or(0)
                    });
                }
                ObsField::HeldInsideReachShare
                | ObsField::UnitShareInsideReach
                | ObsField::UnitShareBesideRival
                | ObsField::PopulationChange
                | ObsField::HeldTileChange
                | ObsField::UnitCountChange
                | ObsField::StrengthChange
                | ObsField::StoreValueChange
                | ObsField::UpgradeCompletions
                | ObsField::TilesTaken
                | ObsField::NewlyObserved
                | ObsField::MeanStaleness
                | ObsField::CentroidMovement
                | ObsField::DecisionsTaken
                | ObsField::WindowTicks
                | ObsField::WorldCyclePhase
                | ObsField::DominationTicks
                | ObsField::WonderTrackTicks
                | ObsField::RenownTicks
                | ObsField::GroundTicks
                | ObsField::PowerPopulation
                | ObsField::PowerStoreValue
                | ObsField::PowerTileGain
                | ObsField::PowerReachArea
                | ObsField::PowerTradeVolume
                | ObsField::StormShareObserved
                | ObsField::StormShareHeld
                | ObsField::HazardShareChange
                | ObsField::TilesBurnt
                | ObsField::HazardConcentration
                | ObsField::SettleCostCoverage
                | ObsField::BestSectorAdmitsSettlement
                | ObsField::UpgradeAffordability
                | ObsField::AcceptableContracts
                | ObsField::RelationChange
                | ObsField::LayoutReserve => {
                    debug_assert!(
                        row.field.value_kind().is_reserved(),
                        "a field this arm leaves unwritten must declare the reserved form"
                    );
                }
            }
        }
        Ok(out)
    }
}

/// Returns the stock of one good class, or zero when the engine holds no
/// commodity for that class.
fn stock_of(site: &SettlementScan, class: usize) -> i64 {
    site.own_stock.get(class).copied().unwrap_or(0)
}

/// Returns the production rate of one good class, over one tick.
fn production_of(site: &SettlementScan, class: usize) -> i64 {
    site.own_production.get(class).copied().unwrap_or(0)
}

/// Returns the consumption rate of one good class, over one tick.
fn consumption_of(site: &SettlementScan, class: usize) -> i64 {
    site.own_consumption.get(class).copied().unwrap_or(0)
}

/// Returns how many ticks the food stock of one faction lasts.
///
/// The stock and the rate are both Q16.16 quantities, so the quotient is a
/// whole number of ticks. A faction that consumes nothing and holds
/// something reads the widest count, which the compressed magnitude then caps
/// at one. A faction that consumes nothing and holds nothing reads zero.
fn food_coverage(site: &SettlementScan, food: usize) -> i64 {
    let stock = stock_of(site, food);
    let rate = consumption_of(site, food);
    if rate > 0 {
        return stock / rate;
    }
    if stock > 0 {
        i64::MAX
    } else {
        0
    }
}

impl World {
    /// Returns the hex distance from the centre of the held ground of one
    /// faction to the centre of the world.
    ///
    /// The centre of the held ground is the integer centroid of the held
    /// tiles the faction sees, which is every tile it holds. Each axial
    /// coordinate is a sum divided by the count, and the division truncates.
    /// A faction that holds nothing reads zero.
    fn centroid_offset(&self, ground: &GroundScan) -> i64 {
        let count = ground.own_held_seen_now;
        if count <= 0 {
            return 0;
        }
        let grid = self.grid();
        let centre = Axial::new(
            (ground.centre_q / count) as i32,
            (ground.centre_r / count) as i32,
        );
        let middle = Axial::new((grid.width() / 2) as i32, (grid.height() / 2) as i32);
        i64::from(centre.distance(middle))
    }
}
