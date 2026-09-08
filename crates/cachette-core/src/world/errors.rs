//! The reasons that a call on the world refused.
//!
//! Every error type of the world module lives here, with the conversion that
//! carries one error into another. The grouping is deliberate. An error type
//! holds no world state and no rule, so it reads and changes on its own, and
//! a reader who wants the rule does not walk through the plumbing to find it.

use crate::bridge::BridgeError;
use crate::choose::ChoiceError;
use crate::cohort::CohortError;
use crate::contest::ContestError;
use crate::conversion::ConversionError;
use crate::hex::{Axial, GridError};
use crate::influence::InfluenceError;
use crate::luxury::LuxuryError;
use crate::position::PositionError;
use crate::promotion::PromotionError;
use crate::rates::RateError;
use crate::relation::RelationError;
use crate::sort::SortError;
use crate::types::{Entity, FACTION_CEILING};
use crate::weather::WeatherError;

/// The reason that a value did not name a live entity.
///
/// A caller outside this crate cannot build an identity, so it names one by
/// the value the engine gave it. That value can be stale, and it can be
/// nothing the engine ever gave. The variants below tell the two apart, so
/// that a boundary can report which one happened.[^1]
///
/// # References
///
/// [^1]: ADR-0085, an entity crosses to Python as one opaque identity that the engine resolves, decision D3. `docs/adrs/accepted/adr-0085-an-entity-crosses-to-python-as-one-opaque-identity.md`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IdentityError {
    /// The value is not an identity at all. The engine never gives out zero.
    NotAnIdentity,
    /// The value names a slot that the arena does not hold.
    NoSuchSlot {
        /// The slot that the value named.
        slot: u32,
    },
    /// The slot exists and holds a later generation, so the entity is dead.
    ///
    /// The arena may have given the slot to another entity. Resolution
    /// refuses rather than return that entity.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0014, entity identity is an index plus a generation, decision D2. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
    Stale {
        /// The slot that the value named.
        slot: u32,
        /// The generation that the value carried.
        given: u32,
        /// The generation that the arena holds for the slot.
        held: u32,
    },
}

impl core::fmt::Display for IdentityError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NotAnIdentity => write!(formatter, "the value is not an identity"),
            Self::NoSuchSlot { slot } => {
                write!(formatter, "the arena holds no slot {slot}")
            }
            Self::Stale { slot, given, held } => write!(
                formatter,
                "the identity names slot {slot} at generation {given}, \
                 and the arena holds generation {held} there"
            ),
        }
    }
}

impl std::error::Error for IdentityError {}

/// The reason that the world refused an order to raze a site.
///
/// Each value names the thing that refused, so a caller repairs the call
/// without guessing which part of it was wrong.[^1]
///
/// # References
///
/// [^1]: ADR-0085, an entity crosses to Python as one opaque identity that the engine resolves, decision D3. `docs/adrs/accepted/adr-0085-an-entity-crosses-to-python-as-one-opaque-identity.md`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RazeError {
    /// The identity names no live site.
    NoSuchSite,
    /// The razer owns the site. A faction does not raze its own city.
    OwnSite,
    /// No siege of the razer stands against the site.
    ///
    /// A raze is an order against a siege, and a siege stands only while the
    /// razer holds the site tile against no defender.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0180, a site changes hands or the taker destroys it, decisions D8 and D11. `docs/adrs/draft/adr-0180-a-site-changes-hands-or-the-taker-destroys-it.md`
    NotBesieging,
}

impl core::fmt::Display for RazeError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NoSuchSite => write!(formatter, "the identity names no live site"),
            Self::OwnSite => write!(formatter, "a faction does not raze its own site"),
            Self::NotBesieging => {
                write!(formatter, "no siege of the razer stands against the site")
            }
        }
    }
}

impl std::error::Error for RazeError {}

/// The reason that the world refused to send a set of units somewhere.
///
/// Each value names the thing that refused, so a caller repairs the call
/// without guessing which part of the set was wrong.[^1]
///
/// # References
///
/// [^1]: ADR-0085, an entity crosses to Python as one opaque identity that the engine resolves, decision D3. `docs/adrs/accepted/adr-0085-an-entity-crosses-to-python-as-one-opaque-identity.md`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SendError {
    /// The number names no destination plane of this world.
    NoSuchDestination(u16),
    /// An identity names no live soldier.
    DeadUnit(Entity),
    /// A seed address is outside the world.
    AddressOutsideWorld(Axial),
}

impl core::fmt::Display for SendError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NoSuchDestination(destination) => {
                write!(formatter, "{destination} names no destination")
            }
            Self::DeadUnit(unit) => {
                write!(formatter, "the identity {unit:?} names no live soldier")
            }
            Self::AddressOutsideWorld(address) => {
                write!(formatter, "the address {address:?} is outside the world")
            }
        }
    }
}

impl std::error::Error for SendError {}

/// The reason that a conversion verb refused a caller.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConvertError {
    /// The number names no faction of this world.
    NoSuchFaction(u16),
    /// An identity names no live soldier.
    DeadUnit(Entity),
}

/// The reason that the relation verb refused a caller.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MoveRelationError {
    /// The speaker identity names no live soldier.
    DeadUnit(Entity),
    /// The relation refused the move.
    Relation(RelationError),
}

impl From<RelationError> for MoveRelationError {
    fn from(error: RelationError) -> Self {
        Self::Relation(error)
    }
}

impl core::fmt::Display for MoveRelationError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::DeadUnit(unit) => write!(formatter, "{unit:?} names no live soldier"),
            Self::Relation(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for MoveRelationError {}

/// Why a campaign was not raised.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CampaignError {
    /// The number names no faction of this world.
    NoSuchFaction(u16),
    /// The objective address is outside the world.
    OutsideWorld(Axial),
    /// The cohort size is zero.
    EmptyCohort,
    /// The faction holds a live campaign.
    LiveCampaign,
    /// The faction has no idle unit to take.
    NoIdleUnit,
    /// The world holds no destination plane for the faction.
    NoPlane(u16),
    /// The send verb refused.
    Send(SendError),
}

impl core::fmt::Display for CampaignError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NoSuchFaction(faction) => {
                write!(formatter, "{faction} names no faction of this world")
            }
            Self::OutsideWorld(address) => {
                write!(formatter, "the objective {address:?} is outside the world")
            }
            Self::EmptyCohort => write!(formatter, "a cohort of zero raises nothing"),
            Self::LiveCampaign => write!(formatter, "the faction holds a live campaign"),
            Self::NoIdleUnit => write!(formatter, "the faction has no idle unit"),
            Self::NoPlane(plane) => write!(
                formatter,
                "the world holds no destination plane {plane} for the faction"
            ),
            Self::Send(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for CampaignError {}

impl From<SendError> for CampaignError {
    fn from(error: SendError) -> Self {
        Self::Send(error)
    }
}

impl core::fmt::Display for ConvertError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NoSuchFaction(faction) => {
                write!(formatter, "{faction} names no faction of this world")
            }
            Self::DeadUnit(unit) => {
                write!(formatter, "the identity {unit:?} names no live soldier")
            }
        }
    }
}

impl std::error::Error for ConvertError {}

/// The reason that a step refused to run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StepError {
    /// The caller asked for zero threads. A step needs at least one.
    ZeroThreads,
    /// The rebuild of the unit-to-tile bridge refused to run.
    Bridge(BridgeError),
    /// The admission sort refused the intent keys.
    Sort(SortError),
    /// An intent named a tile outside the world.
    ///
    /// The intent half already drops a target outside the extent, so this
    /// says that the two halves disagree rather than that a caller erred.
    TargetOutsideWorld,
    /// The site rate pass refused to run.
    Rates(RateError),
    /// The consumption pass refused to run.
    Consumption(CohortError),
    /// The choice pass refused to run.
    Choice(ChoiceError),
    /// The influence solve refused to run.
    Influence(InfluenceError),
    /// A pass over the positions of the sites refused to run.
    Positions(PositionError),
    /// The promotion pass refused to run.
    Promotion(PromotionError),
    /// The resolution of a meeting refused to run.
    Contest(ContestError),
    /// The conversion pass refused to run.
    Conversion(ConversionError),
    /// The weather solve refused to run.
    Weather(WeatherError),
}

impl From<ConversionError> for StepError {
    fn from(error: ConversionError) -> Self {
        match error {
            ConversionError::ZeroThreads => Self::ZeroThreads,
            other => Self::Conversion(other),
        }
    }
}

impl From<PromotionError> for StepError {
    fn from(error: PromotionError) -> Self {
        match error {
            PromotionError::ZeroThreads => Self::ZeroThreads,
            other => Self::Promotion(other),
        }
    }
}

impl From<PositionError> for StepError {
    fn from(error: PositionError) -> Self {
        match error {
            PositionError::ZeroThreads => Self::ZeroThreads,
            other => Self::Positions(other),
        }
    }
}

impl From<InfluenceError> for StepError {
    fn from(error: InfluenceError) -> Self {
        match error {
            InfluenceError::ZeroThreads => Self::ZeroThreads,
            other => Self::Influence(other),
        }
    }
}

impl From<ChoiceError> for StepError {
    fn from(error: ChoiceError) -> Self {
        Self::Choice(error)
    }
}

impl From<CohortError> for StepError {
    fn from(error: CohortError) -> Self {
        match error {
            CohortError::ZeroThreads => Self::ZeroThreads,
            other => Self::Consumption(other),
        }
    }
}

impl From<RateError> for StepError {
    fn from(error: RateError) -> Self {
        match error {
            RateError::ZeroThreads => Self::ZeroThreads,
            other => Self::Rates(other),
        }
    }
}

impl From<SortError> for StepError {
    fn from(error: SortError) -> Self {
        Self::Sort(error)
    }
}

impl From<WeatherError> for StepError {
    fn from(error: WeatherError) -> Self {
        match error {
            WeatherError::ZeroThreads => Self::ZeroThreads,
            other => Self::Weather(other),
        }
    }
}

impl From<ContestError> for StepError {
    fn from(error: ContestError) -> Self {
        match error {
            ContestError::ZeroThreads => Self::ZeroThreads,
            other => Self::Contest(other),
        }
    }
}

impl From<BridgeError> for StepError {
    fn from(error: BridgeError) -> Self {
        Self::Bridge(error)
    }
}

/// The reason that a world refused to build.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorldError {
    /// The configured extent does not describe a grid.
    Grid(GridError),
    /// The block partition of the world refused to build.
    Bridge(BridgeError),
    /// The configured faction count is above the storage ceiling.
    ///
    /// A faction is one bit in a 64-bit mask, so the project holds a fixed
    /// ceiling. A world may hold fewer factions than the ceiling. It may
    /// never hold more.[^1]
    ///
    /// # References
    ///
    /// [^1]: Budgets and costs, the scale constants. `docs/reference/budgets.md`
    FactionCountAboveCeiling(u16),
    /// The influence field refused to build.
    Influence(InfluenceError),
    /// The weather field refused to build.
    Weather(WeatherError),
}

impl From<WeatherError> for WorldError {
    fn from(error: WeatherError) -> Self {
        Self::Weather(error)
    }
}

impl From<InfluenceError> for WorldError {
    fn from(error: InfluenceError) -> Self {
        Self::Influence(error)
    }
}

impl From<BridgeError> for WorldError {
    fn from(error: BridgeError) -> Self {
        Self::Bridge(error)
    }
}

impl core::fmt::Display for WorldError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Grid(error) => write!(formatter, "the world extent is not a grid: {error}"),
            Self::Bridge(error) => write!(formatter, "the world has no block partition: {error}"),
            Self::FactionCountAboveCeiling(count) => write!(
                formatter,
                "the world asks for {count} factions, and the ceiling is {FACTION_CEILING}"
            ),
            Self::Influence(error) => {
                write!(formatter, "the world has no influence field: {error:?}")
            }
            Self::Weather(error) => {
                write!(formatter, "the world has no weather field: {error}")
            }
        }
    }
}

impl std::error::Error for WorldError {}

/// Why the seeding of a world refused.
///
/// The seeding founds a run for every faction, and it places the luxuries.
/// It then leaves the world readable, and that needs a rebuild of the
/// derived unit structure. Either half can refuse, so the error says which
/// one did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SeedError {
    /// The luxury field refused the placements.
    Luxury(LuxuryError),
    /// The rebuild of the derived unit structure refused to run.
    Bridge(BridgeError),
}

impl From<LuxuryError> for SeedError {
    fn from(error: LuxuryError) -> Self {
        Self::Luxury(error)
    }
}

impl From<BridgeError> for SeedError {
    fn from(error: BridgeError) -> Self {
        Self::Bridge(error)
    }
}

impl core::fmt::Display for SeedError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Luxury(error) => write!(formatter, "the world took no luxuries: {error}"),
            Self::Bridge(error) => write!(
                formatter,
                "the seeded world holds no readable unit structure: {error}"
            ),
        }
    }
}

impl std::error::Error for SeedError {}

impl From<GridError> for WorldError {
    fn from(error: GridError) -> Self {
        Self::Grid(error)
    }
}

impl core::fmt::Display for StepError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::ZeroThreads => write!(formatter, "a step needs at least one thread"),
            Self::Bridge(error) => write!(formatter, "the bridge rebuild refused: {error}"),
            Self::Sort(error) => write!(formatter, "the admission sort refused: {error}"),
            Self::TargetOutsideWorld => {
                write!(formatter, "an intent named a tile outside the world")
            }
            Self::Rates(error) => write!(formatter, "the site rate pass refused: {error}"),
            Self::Consumption(error) => {
                write!(formatter, "the consumption pass refused: {error}")
            }
            Self::Choice(error) => write!(formatter, "the choice pass refused: {error}"),
            Self::Influence(error) => {
                write!(formatter, "the influence solve refused: {error:?}")
            }
            Self::Positions(error) => {
                write!(formatter, "the position pass refused: {error}")
            }
            Self::Promotion(error) => {
                write!(formatter, "the promotion pass refused: {error}")
            }
            Self::Contest(error) => {
                write!(formatter, "the resolution of a meeting refused: {error}")
            }
            Self::Conversion(error) => {
                write!(formatter, "the conversion pass refused: {error}")
            }
            Self::Weather(error) => {
                write!(formatter, "the weather solve refused: {error}")
            }
        }
    }
}

impl std::error::Error for StepError {}
