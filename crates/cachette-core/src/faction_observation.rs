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
//! # What a reserved field means
//!
//! **A reserved field reads zero in every position, and the schema says so.**
//! Its declared bounds are zero and zero. A field is reserved for one of two
//! reasons: the engine keeps no aggregate that answers it, or the value needs
//! a window of past frames that the engine does not carry. A reserved field
//! is not a zero that states a real quantity of zero.
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

use crate::action::{CandidateKind, Verb};
use crate::event_layout::ColumnKind;
use crate::faction_view::{BlockMask, FactionViewError};
use crate::hex::Axial;
use crate::holding::Holder;
use crate::obs_frontier::Frontier;
use crate::obs_ring_stack::RingStack;
use crate::obs_token::EntityTokens;
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
/// # References
///
/// [^1]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, the consequences. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
pub const OBSERVATION_VERSION: u32 = 5;

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
pub(crate) use crate::obs_token::TOKEN_SLOTS;

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
pub const LAYOUT_RESERVE: u32 = 29;

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

impl ValueKind {
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
    ( $( $(#[$note:meta])* $variant:ident => $name:literal, $positions:expr, $kind:ident ; )* ) => {
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
    /// The people the settlements of the faction hold.
    Population => "population", 1, Magnitude;
    /// The population over the settlement count.
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
    /// **Reserved.** The military strength of the faction.
    ///
    /// The engine keeps no military strength aggregate. The unit type table
    /// holds an attack and an armour column, and no record states how to
    /// combine them into a strength. A sum invented here would read as a real
    /// quantity.
    MilitaryStrength => "military_strength", 1, Reserved;
    /// **Reserved.** The military strength over the live unit count.
    StrengthForEachUnit => "strength_for_each_unit", 1, Reserved;
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
    /// The engine carries no window. Nothing stores the observation of a past
    /// frame, so no first difference of any quantity is available. Every
    /// field of this layout whose doc names a window is reserved for that one
    /// reason.
    PopulationChange => "population_change", 1, Reserved;
    /// **Reserved.** The change in the held tiles over a window.
    HeldTileChange => "held_tile_change", 1, Reserved;
    /// **Reserved.** The change in the live units over a window.
    UnitCountChange => "unit_count_change", 1, Reserved;
    /// **Reserved.** The change in the military strength over a window.
    StrengthChange => "strength_change", 1, Reserved;
    /// **Reserved.** The change in the store total over a window.
    StoreValueChange => "store_value_change", 1, Reserved;
    /// **Reserved.** The upgrades finished over a window.
    UpgradeCompletions => "upgrade_completions", 1, Reserved;
    /// **Reserved.** The tiles taken from a rival over a window.
    TilesTaken => "tiles_taken", 1, Reserved;
    /// **Reserved.** The tiles lost to a rival over a window.
    TilesLost => "tiles_lost", 1, Reserved;
    /// **Reserved.** The settlements lost over a window.
    SettlementsLost => "settlements_lost", 1, Reserved;
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
    HeldUnderHazardShare => "held_under_hazard_share", 1, Share;
    /// The settlements under a hazard over the settlement count.
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
    /// The value is zero because no window exists. A reader that finds zero
    /// here knows that every window field of this layout reads zero.
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

    /// Block C. The seats the faction holds over the seated factions.
    DominationProgress => "domination_progress", 1, Share;
    /// The seats of the leading faction over the seated factions.
    DominationLeader => "domination_leader", 1, Share;
    /// The seats of the faction against the seats of the strongest rival.
    DominationGap => "domination_gap", 1, Relation;
    /// The rivals the faction leads on seats, over the rival count.
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
    /// **Reserved.** The seven order statistics of the military strength.
    ///
    /// The engine keeps no military strength aggregate for any faction.
    PowerStrength => "power_strength", POWER_STATISTIC_COUNT, Reserved;
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

    /// Block I. The entity tokens.
    ///
    /// The block holds four token sets, in this order: the own settlements,
    /// the rivals, the threat clusters and the candidate sites. Each set
    /// holds a fixed number of tokens, and each token holds a fixed number
    /// of channels.
    EntityTokens => "entity_tokens", TOKEN_SLOTS, Statistic;

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
    /// **Reserved.** The units lost to a hazard over a window.
    ///
    /// The engine holds a per-frame log of the units a fire burned. A frame
    /// is not a window, and a count over one frame is a different quantity.
    UnitsLostToHazard => "units_lost_to_hazard", 1, Reserved;
    /// **Reserved.** The tiles burnt over a window.
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
    /// **Reserved.** The positions the layout holds back for a later signal.
    LayoutReserve => "layout_reserve", LAYOUT_RESERVE, Reserved;
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
            if owner != faction {
                continue;
            }
            scan.own_units_seen += 1;
            if let Some(class) = self.unit_type(*unit) {
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
            if matches!(self.tile_is_burning(address), Some(true))
                || matches!(self.ground_is_wet(address), Some(true))
            {
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

    /// Returns the seats each faction holds, by faction number.
    ///
    /// A seat is the tile of the first founding of a faction, and the
    /// domination reader compares the count against the seat count. **A seat
    /// holder carries no fog rule**, because the reader ends the game on it.
    fn seats_held_by_each(&self, seats: usize) -> Vec<i64> {
        let mut held = vec![0i64; seats];
        for seat in 0..seats {
            let Some(tile) = self.seat(FactionId(seat as u16)) else {
                continue;
            };
            let Some(address) = self.grid().address_of(tile) else {
                continue;
            };
            let Some(holder) = self.tile_holder(address).and_then(Holder::faction) else {
                continue;
            };
            if let Some(place) = held.get_mut(usize::from(holder.0)) {
                *place += 1;
            }
        }
        held
    }
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
    held_tiles: i64,
    live_units: i64,
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
    domination: Track,
    wonder: Track,
    renown: Track,
    ground_track: Track,
    power_held: PowerVector,
    power_settlements: PowerVector,
    power_units: PowerVector,
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
        let wonder_work = self
            .victory_claims()
            .into_iter()
            .map(|pair| pair.1)
            .collect::<Vec<i64>>();
        let seats_held = self.seats_held_by_each(seats);

        let world = self.pyramid().total();
        let requirement = self
            .upgrade_table()
            .row(UpgradeCategory::WONDER, upgrade::WONDER_LEVEL)
            .map_or(0, |row| i64::from(row.work));
        let renown_target = i64::from(self.balance().renown_target());
        let world_passable = world.open_tiles();

        let ring_stack = self.faction_ring_stack(faction)?;
        let frontier = self.faction_frontier(faction, &ring_stack)?;
        let tokens = self.faction_entity_tokens(faction, &ring_stack, &frontier)?;

        let power_held = PowerVector {
            values: held.clone(),
            confidence,
        };
        let leader = power_held.leader();
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
            held_tiles: self.holding_of(faction),
            live_units: i64::from(self.population_of(faction)),
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
            domination: Track::of(&seats_held, seat, seats as i64),
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
                ObsField::RingStack => span.copy_from_slice(read.ring_stack.slots()),
                ObsField::FrontierBySector => span.copy_from_slice(read.frontier.slots()),
                ObsField::EntityTokens => span.copy_from_slice(read.tokens.slots()),
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
                    span[0] = share(
                        ground.own_held_fire + ground.own_held_water,
                        ground.own_held_seen_now,
                    );
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
                ObsField::ObjectiveWeight => {
                    fill(span, |element| {
                        read.weights.get(element).copied().unwrap_or(0)
                    });
                }
                ObsField::HeldInsideReachShare
                | ObsField::MilitaryStrength
                | ObsField::StrengthForEachUnit
                | ObsField::UnitShareInsideReach
                | ObsField::UnitShareBesideRival
                | ObsField::PopulationChange
                | ObsField::HeldTileChange
                | ObsField::UnitCountChange
                | ObsField::StrengthChange
                | ObsField::StoreValueChange
                | ObsField::UpgradeCompletions
                | ObsField::TilesTaken
                | ObsField::TilesLost
                | ObsField::SettlementsLost
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
                | ObsField::PowerStrength
                | ObsField::PowerStoreValue
                | ObsField::PowerTileGain
                | ObsField::PowerReachArea
                | ObsField::PowerTradeVolume
                | ObsField::StormShareObserved
                | ObsField::StormShareHeld
                | ObsField::HazardShareChange
                | ObsField::UnitsLostToHazard
                | ObsField::TilesBurnt
                | ObsField::HazardConcentration
                | ObsField::SettleCostCoverage
                | ObsField::BestSectorAdmitsSettlement
                | ObsField::UpgradeAffordability
                | ObsField::AcceptableContracts
                | ObsField::RelationChange
                | ObsField::LayoutReserve => {}
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
