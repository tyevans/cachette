//! The founding of a run.
//!
//! A run begins with a small group of people in one place. This module
//! chooses the place and says why it chose it.
//!
//! The choice reads a bounded sample of the world. It draws a fixed number of
//! candidate places, and it reads a fixed number of tiles around each one.
//! Neither number is a function of the world extent, so a world of a hundred
//! tiles and a world of sixteen million tiles cost the same choice.[^1] A pass
//! over every tile is the simple way to choose, and it is refused here,
//! because the whole of that cost falls before the first frame.[^2]
//!
//! Every candidate address comes from the counter-based generator, keyed on
//! the tuple of system, frame, entity and draw index.[^3] The frame slot holds
//! the faction that founds, because a founding happens before the first frame
//! and the slot is otherwise a constant.[^7] The entity slot holds the
//! candidate ordinal. The draw slot holds the axis, so the column of a
//! candidate and its row never correlate.
//!
//! A run founds one group for each faction, and each founding keeps a minimum
//! distance from every place a founding before it took.[^7] A founding that
//! finds no admissible place in its sample fails. It does not draw again and
//! it does not widen the sample.
//!
//! Every score is an exact integer or a Q16.16 value, and every arithmetic
//! step goes through the arithmetic module.[^4] [^5] No item in this module
//! uses a floating-point type.
//!
//! Two candidates that score the same resolve by the tile index, which is
//! unique inside the sample.[^6] The order is therefore total and it does not
//! depend on the order the candidates were drawn in.
//!
//! # References
//!
//! [^1]: ADR-0075, the founding choice reads a bounded sample of the world, decision D1. `docs/adrs/accepted/adr-0075-the-founding-choice-reads-a-bounded-sample-of-the-world.md`
//! [^2]: PRD-0012, a world starts small and grows. `docs/product/accepted/prd-0012-a-world-starts-small-and-grows.md`
//! [^3]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
//! [^4]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
//! [^5]: ADR-0002, simulated and aggregated state holds no floating point number, decision D2. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
//! [^6]: ADR-0004, iteration order is explicit, decision D4. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
//! [^7]: ADR-0076, a founding keeps a fixed distance from the foundings before it. `docs/adrs/draft/adr-0076-a-founding-keeps-a-fixed-distance-from-the-foundings-before-it.md`

use crate::hex::{Axial, Grid};
use crate::resource::{Amount, ResourceField, ResourceKind};
use crate::rng;
use crate::sim_math;
use crate::site::SettlementError;
use crate::soldier::SoldierError;
use crate::sort::{self, SortError, SortKey};
use crate::terrain::TileKind;
use crate::types::{Accum, Entity, FactionId, Fix32, TileIdx};

/// Returns the frame slot that the draws of one founding are keyed on.
///
/// A founding happens before the first frame, so the frame slot carries no
/// frame. It carries the faction that founds.[^1] Two factions then read two
/// samples. A key without the faction gives every faction one sample, which
/// narrows the pool that every founding after the first draws from. It does
/// not empty it, and a run with a shared sample still seats every faction, so
/// no test of the outcome sees the defect.[^2]
///
/// The slot stays in the key because the key shape is fixed by the
/// record.[^3]
///
/// # References
///
/// [^1]: ADR-0076, a founding keeps a fixed distance from the foundings before it, decision D3. `docs/adrs/draft/adr-0076-a-founding-keeps-a-fixed-distance-from-the-foundings-before-it.md`
/// [^2]: Testing rules, section 2. `.claude/rules/testing.md`
/// [^3]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
#[must_use]
pub const fn founding_frame(faction: FactionId) -> u64 {
    faction.0 as u64
}

/// The draw index of the column of a candidate.
const DRAW_COLUMN: u32 = 0;

/// The draw index of the row of a candidate.
///
/// The two indices differ, so the column of a candidate and its row are two
/// draws and not one. A key that shared the index would give a sample on the
/// diagonal of the world, and every determinism test would still pass,
/// because the defect repeats.[^1]
///
/// The perturbed build drops the row draw and holds one row, so nothing reads
/// this constant there. The allowance is narrowed to that build, so the
/// ordinary build still reports the constant if its last reader goes.
///
/// # References
///
/// [^1]: Testing rules, section 2. `.claude/rules/testing.md`
#[cfg_attr(feature = "probe-nondeterminism", allow(dead_code))]
const DRAW_ROW: u32 = 8;

/// The number of candidate places the founding draws.
///
/// This is a property of the choosing rule. It is not a function of the world
/// extent, and it may not become one without superseding the record.[^1]
///
/// # References
///
/// [^1]: ADR-0075, the founding choice reads a bounded sample of the world, decision D1. `docs/adrs/accepted/adr-0075-the-founding-choice-reads-a-bounded-sample-of-the-world.md`
pub const SAMPLE_SIZE: u32 = 64;

/// The number of rings the founding reads around a candidate.
///
/// This is a property of the choosing rule, in the same way the sample size
/// is. The group settles inside this radius, so the radius is also what the
/// group can reach on the tick it arrives.
pub const SURVEY_RADIUS: u32 = 3;

/// The number of tiles in the disc that one survey reads.
///
/// A hex disc of a given radius holds one tile at the centre and six for each
/// ring, so the count is one plus three times the radius times the radius plus
/// one. The value holds at any world extent.
pub const SURVEY_TILES: u32 = 1 + 3 * SURVEY_RADIUS * (SURVEY_RADIUS + 1);

/// The largest number of tiles that one survey of a world reads.
///
/// No world extent enters this value. A test compares the tiles a survey
/// actually read against this ceiling, over worlds that differ in extent by a
/// large factor, and the ceiling does not move.[^1] That is how the cost
/// constraint is checked without a timing assertion.[^2]
///
/// # References
///
/// [^1]: ADR-0075, the founding choice reads a bounded sample of the world, decision D1. `docs/adrs/accepted/adr-0075-the-founding-choice-reads-a-bounded-sample-of-the-world.md`
/// [^2]: Testing rules, section 3. `.claude/rules/testing.md`
pub const SURVEY_CEILING: u64 = SAMPLE_SIZE as u64 * SURVEY_TILES as u64;

/// The smallest distance that two foundings of one run keep between them.
///
/// The value is a tuning knob of the founding rule, in the way the sample
/// size is one. The record states the constraint and not the value.[^1]
///
/// The floor is not a knob. Two foundings closer than twice the survey radius
/// settle their groups over one piece of ground, so the distance must exceed
/// that. The assertion below fails to compile when the two disagree, so the
/// floor is checked and not commented.[^2]
///
/// # References
///
/// [^1]: ADR-0076, a founding keeps a fixed distance from the foundings before it, decision D1. `docs/adrs/draft/adr-0076-a-founding-keeps-a-fixed-distance-from-the-foundings-before-it.md`
/// [^2]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
pub const MINIMUM_FOUNDING_DISTANCE: u32 = 16;

const _: () = assert!(
    MINIMUM_FOUNDING_DISTANCE > 2 * SURVEY_RADIUS,
    "two foundings at the minimum distance would settle over one disc"
);

/// The weight of the food a place can reach.
const WEIGHT_FOOD: Fix32 = Fix32::from_int(4);

/// The weight of the wood a place can reach.
const WEIGHT_WOOD: Fix32 = Fix32::from_int(2);

/// The weight of the stone a place can reach.
const WEIGHT_STONE: Fix32 = Fix32::from_int(1);

/// The weight of the open ground a place can reach.
const WEIGHT_OPEN: Fix32 = Fix32::from_int(2);

/// The weight of one open water tile that touches a place.
///
/// Open water beside a place is worth more than one tile of anything else,
/// because a settlement wants water it can reach on foot every tick.
const WEIGHT_WATER_EDGE: Fix32 = Fix32::from_int(8);

/// What a place can reach.
///
/// Every field is an exact count. The score is derived from these, and a
/// watcher reads them to learn why a place was chosen.[^1]
///
/// The set of properties is recorded, because the product record declines to
/// choose it and a set nobody wrote down is a set nobody can change
/// deliberately.[^2]
///
/// # References
///
/// [^1]: ADR-0075, the founding choice reads a bounded sample of the world, decision D5. `docs/adrs/accepted/adr-0075-the-founding-choice-reads-a-bounded-sample-of-the-world.md`
/// [^2]: Open decisions register, DEC-031. `docs/DECISIONS.md`
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Provision {
    /// The food in the disc that the survey read.
    pub food: Amount,
    /// The wood in the disc that the survey read.
    pub wood: Amount,
    /// The stone in the disc that the survey read.
    pub stone: Amount,
    /// The number of tiles in the disc that admit a unit.
    pub open_ground: u32,
    /// The number of units that the open tiles of the disc hold together.
    pub room: u32,
    /// The number of the six neighbours of the centre that hold open water.
    pub water_edge: u32,
}

impl Provision {
    /// Returns the score of a place.
    ///
    /// The score is a weighted sum of exact counts. Every step goes through
    /// the arithmetic module and no step uses a floating-point type, so the
    /// comparison gives one answer whatever order the work ran in.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0075, the founding choice reads a bounded sample of the world, decision D3. `docs/adrs/accepted/adr-0075-the-founding-choice-reads-a-bounded-sample-of-the-world.md`
    #[must_use]
    pub fn score(self) -> Accum {
        let mut total = Accum(0);
        total = sim_math::accumulate(total, weigh(self.food.0, WEIGHT_FOOD));
        total = sim_math::accumulate(total, weigh(self.wood.0, WEIGHT_WOOD));
        total = sim_math::accumulate(total, weigh(self.stone.0, WEIGHT_STONE));
        total = sim_math::accumulate(total, weigh(self.open_ground, WEIGHT_OPEN));
        total = sim_math::accumulate(total, weigh(self.water_edge, WEIGHT_WATER_EDGE));
        total
    }
}

/// Multiplies an exact count by a weight.
///
/// The count saturates into the fixed-point range rather than wrapping. A
/// wrap would turn a large count into a large negative one and would hide the
/// defect.
fn weigh(count: u32, weight: Fix32) -> Fix32 {
    let bounded = i16::try_from(count).unwrap_or(i16::MAX);
    sim_math::mul(Fix32::from_int(bounded), weight)
}

/// One candidate place, and what the survey read about it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Candidate {
    address: Axial,
    tile: TileIdx,
    provision: Provision,
    score: Accum,
    separated: bool,
    eligible: bool,
}

impl Candidate {
    /// Returns the address of the place.
    #[must_use]
    pub const fn address(self) -> Axial {
        self.address
    }

    /// Returns the tile index of the place, which is the stable key.
    #[must_use]
    pub const fn tile(self) -> TileIdx {
        self.tile
    }

    /// Returns what the place can reach.
    #[must_use]
    pub const fn provision(self) -> Provision {
        self.provision
    }

    /// Returns the score of the place.
    #[must_use]
    pub const fn score(self) -> Accum {
        self.score
    }

    /// Reports whether the place keeps its distance from the places taken.
    ///
    /// A watcher reads this to learn that a place was refused for the company
    /// it keeps and not for what the ground holds.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0075, the founding choice reads a bounded sample of the world, decision D5. `docs/adrs/accepted/adr-0075-the-founding-choice-reads-a-bounded-sample-of-the-world.md`
    #[must_use]
    pub const fn is_separated(self) -> bool {
        self.separated
    }

    /// Reports whether the group could settle here.
    ///
    /// A place is eligible when the ground at its centre admits a unit, when
    /// the open tiles of its disc hold the whole group, and when the place
    /// keeps the minimum distance from every place a founding before it
    /// took.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0076, a founding keeps a fixed distance from the foundings before it, decision D1. `docs/adrs/draft/adr-0076-a-founding-keeps-a-fixed-distance-from-the-foundings-before-it.md`
    #[must_use]
    pub const fn is_eligible(self) -> bool {
        self.eligible
    }
}

/// What the founding read, and what it chose.
///
/// The report is the output of the choice. Nothing recomputes a score to
/// answer a question about it, so no copy can disagree with the choice that
/// was made.[^1]
///
/// # References
///
/// [^1]: ADR-0075, the founding choice reads a bounded sample of the world, decision D5. `docs/adrs/accepted/adr-0075-the-founding-choice-reads-a-bounded-sample-of-the-world.md`
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Survey {
    ordered: Vec<Candidate>,
    drawn: u32,
    tiles_read: u64,
}

impl Survey {
    /// Returns the candidates, best first.
    ///
    /// The order puts every eligible place before every place that refuses
    /// the group, then the higher score before the lower, then the lower tile
    /// index before the higher.
    #[must_use]
    pub fn candidates(&self) -> &[Candidate] {
        &self.ordered
    }

    /// Returns the place the founding chose.
    ///
    /// Returns `None` when no candidate in the sample admits the group.
    #[must_use]
    pub fn chosen(&self) -> Option<Candidate> {
        self.ordered
            .first()
            .copied()
            .filter(|candidate| candidate.is_eligible())
    }

    /// Returns the places the founding did not choose, best first.
    #[must_use]
    pub fn rejected(&self) -> &[Candidate] {
        self.ordered.get(1..).unwrap_or(&[])
    }

    /// Returns the number of candidate places the founding drew.
    #[must_use]
    pub const fn drawn(&self) -> u32 {
        self.drawn
    }

    /// Returns the number of distinct places the founding surveyed.
    ///
    /// Two draws may name one tile. The survey reads such a tile once, and
    /// the sort refuses a repeated stable key.
    #[must_use]
    pub fn considered(&self) -> usize {
        self.ordered.len()
    }

    /// Returns the number of tiles the founding read.
    ///
    /// The count rises as the survey reads, so it is a measurement and not a
    /// second declaration of the sample size.[^1]
    ///
    /// # References
    ///
    /// [^1]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
    #[must_use]
    pub const fn tiles_read(&self) -> u64 {
        self.tiles_read
    }
}

/// A founded run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Founding {
    place: Axial,
    settlement: Entity,
    people: Vec<Entity>,
    survey: Survey,
}

impl Founding {
    /// Builds the record of a founding. The world is the only caller.
    pub(crate) const fn new(
        place: Axial,
        settlement: Entity,
        people: Vec<Entity>,
        survey: Survey,
    ) -> Self {
        Self {
            place,
            settlement,
            people,
            survey,
        }
    }

    /// Returns the place the group settled.
    #[must_use]
    pub const fn place(&self) -> Axial {
        self.place
    }

    /// Returns the settlement that stands at the place.
    #[must_use]
    pub const fn settlement(&self) -> Entity {
        self.settlement
    }

    /// Returns the people of the founding group.
    #[must_use]
    pub fn people(&self) -> &[Entity] {
        &self.people
    }

    /// Returns what the founding read, and what it chose.
    #[must_use]
    pub const fn survey(&self) -> &Survey {
        &self.survey
    }
}

/// What one faction got when a run founded.
///
/// A run of several foundings can seat some factions and refuse another. One
/// result for the whole run would hide one or the other, so the run reports
/// one outcome for each faction.[^1]
///
/// # References
///
/// [^1]: ADR-0076, a founding keeps a fixed distance from the foundings before it, decision D2. `docs/adrs/draft/adr-0076-a-founding-keeps-a-fixed-distance-from-the-foundings-before-it.md`
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FoundingOutcome {
    faction: FactionId,
    result: Result<Founding, FoundingError>,
}

impl FoundingOutcome {
    /// Builds one outcome. The world is the only caller.
    pub(crate) const fn new(faction: FactionId, result: Result<Founding, FoundingError>) -> Self {
        Self { faction, result }
    }

    /// Returns the faction this outcome belongs to.
    #[must_use]
    pub const fn faction(&self) -> FactionId {
        self.faction
    }

    /// Returns the founding, or the reason the faction was refused.
    pub const fn result(&self) -> &Result<Founding, FoundingError> {
        &self.result
    }

    /// Returns the founding, or `None` when the faction was refused.
    #[must_use]
    pub const fn founding(&self) -> Option<&Founding> {
        match &self.result {
            Ok(founding) => Some(founding),
            Err(_) => None,
        }
    }

    /// Reports whether the faction was seated.
    #[must_use]
    pub const fn is_seated(&self) -> bool {
        self.result.is_ok()
    }
}

/// The reason a founding did not happen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FoundingError {
    /// The caller asked for a group of nobody.
    EmptyGroup,
    /// No place in the sample admits the group.
    ///
    /// The founding reports the refusal rather than widening the sample. A
    /// sample that widens until it succeeds is a pass over every tile with
    /// extra steps.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0075, the founding choice reads a bounded sample of the world. `docs/adrs/accepted/adr-0075-the-founding-choice-reads-a-bounded-sample-of-the-world.md`
    NoPlaceFound(u32),
    /// The address the caller named lies outside the world.
    OutsideWorld(Axial),
    /// The ordering of the candidates refused to run.
    Order(SortError),
    /// A member of the group refused to spawn.
    Person(SoldierError),
    /// The settlement refused to stand at the place.
    Seat(SettlementError),
}

impl core::fmt::Display for FoundingError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::EmptyGroup => write!(formatter, "a founding group holds at least one person"),
            Self::NoPlaceFound(drawn) => write!(
                formatter,
                "none of the {drawn} places drawn admits the whole group"
            ),
            Self::OutsideWorld(address) => write!(
                formatter,
                "the address ({}, {}) lies outside the world",
                address.q, address.r
            ),
            Self::Order(error) => write!(formatter, "the candidates would not order: {error}"),
            Self::Person(error) => write!(formatter, "a person refused to arrive: {error}"),
            Self::Seat(error) => write!(formatter, "the settlement refused to stand: {error}"),
        }
    }
}

impl std::error::Error for FoundingError {}

impl From<SortError> for FoundingError {
    fn from(error: SortError) -> Self {
        Self::Order(error)
    }
}

impl From<SoldierError> for FoundingError {
    fn from(error: SoldierError) -> Self {
        Self::Person(error)
    }
}

impl From<SettlementError> for FoundingError {
    fn from(error: SettlementError) -> Self {
        Self::Seat(error)
    }
}

/// Returns the addresses of the disc around a place, in a fixed order.
///
/// The order is the column offset, then the row offset, both ascending. It
/// does not depend on the world, on the tick, or on the caller.[^1] An address
/// outside the world is dropped, because the world does not wrap.
///
/// # References
///
/// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
#[must_use]
pub fn disc(grid: Grid, centre: Axial, radius: u32) -> Vec<Axial> {
    let reach = radius as i32;
    let mut tiles = Vec::with_capacity(SURVEY_TILES as usize);
    let mut dq = -reach;
    while dq <= reach {
        let low = (-reach).max(-reach - dq);
        let high = reach.min(reach - dq);
        let mut dr = low;
        while dr <= high {
            let address = Axial::new(centre.q + dq, centre.r + dr);
            if grid.contains(address) {
                tiles.push(address);
            }
            dr += 1;
        }
        dq += 1;
    }
    tiles
}

/// Reads what one place can reach, and returns the tiles it read.
///
/// Returns `None` when the address lies outside the world.
fn read_place(field: ResourceField, centre: Axial) -> Option<(Provision, u64)> {
    let grid = field.grid();
    if !grid.contains(centre) {
        return None;
    }
    let mut provision = Provision::default();
    let mut read = 0u64;
    for address in disc(grid, centre, SURVEY_RADIUS) {
        let Some(ground) = field.terrain().kind(address) else {
            continue;
        };
        read += 1;
        if ground.is_passable() {
            provision.open_ground += 1;
            provision.room += ground.capacity();
        }
        for kind in ResourceKind::ALL {
            let amount = field.original(address, kind).unwrap_or(Amount::ZERO);
            let slot = match kind {
                ResourceKind::Food => &mut provision.food,
                ResourceKind::Wood => &mut provision.wood,
                ResourceKind::Stone => &mut provision.stone,
            };
            slot.0 = slot.0.saturating_add(amount.0);
        }
    }
    // The neighbours of the centre lie inside the disc, so this reads no tile
    // that the loop above did not already read.
    for neighbour in grid.neighbours(centre).into_iter().flatten() {
        if field.terrain().kind(neighbour) == Some(TileKind::Water) {
            provision.water_edge += 1;
        }
    }
    Some((provision, read))
}

/// Returns the column of one candidate.
#[cfg(not(feature = "probe-nondeterminism"))]
fn candidate_column(seed: u64, faction: FactionId, ordinal: u32, width: u32) -> i32 {
    rng::draw_below(
        seed,
        rng::SYSTEM_FOUNDING,
        founding_frame(faction),
        u64::from(ordinal),
        DRAW_COLUMN,
        u64::from(width),
    ) as i32
}

/// Returns the row of one candidate.
///
/// The draw index differs from the column draw index, so the two coordinates
/// of one candidate are two draws.
#[cfg(not(feature = "probe-nondeterminism"))]
fn candidate_row(seed: u64, faction: FactionId, ordinal: u32, height: u32) -> i32 {
    rng::draw_below(
        seed,
        rng::SYSTEM_FOUNDING,
        founding_frame(faction),
        u64::from(ordinal),
        DRAW_ROW,
        u64::from(height),
    ) as i32
}

/// The perturbed column. It is unchanged.
#[cfg(feature = "probe-nondeterminism")]
fn candidate_column(seed: u64, faction: FactionId, ordinal: u32, width: u32) -> i32 {
    rng::draw_below(
        seed,
        rng::SYSTEM_FOUNDING,
        founding_frame(faction),
        u64::from(ordinal),
        DRAW_COLUMN,
        u64::from(width),
    ) as i32
}

/// The perturbed row. It drops the draw and holds one row.
///
/// This is the defect that the testing rule warns about: the sample it draws
/// is identical on every run and at every thread count, so both determinism
/// tests pass over it. Only a test of the key itself sees that the sample
/// covers one row of the world.[^1]
///
/// # References
///
/// [^1]: Testing rules, section 2. `.claude/rules/testing.md`
#[cfg(feature = "probe-nondeterminism")]
fn candidate_row(_seed: u64, _faction: FactionId, _ordinal: u32, _height: u32) -> i32 {
    0
}

/// Surveys a bounded sample of a world and orders the places it read.
///
/// The sample size and the survey radius are properties of this rule. Neither
/// is a function of the world extent, so the cost of the call does not grow
/// with the world.[^1]
///
/// # Errors
///
/// Returns an error when the group holds nobody, or when the ordering refuses
/// to run.
///
/// # References
///
/// [^1]: ADR-0075, the founding choice reads a bounded sample of the world, decision D1. `docs/adrs/accepted/adr-0075-the-founding-choice-reads-a-bounded-sample-of-the-world.md`
pub fn survey(
    field: ResourceField,
    group: u32,
    faction: FactionId,
    taken: &[Axial],
) -> Result<Survey, FoundingError> {
    if group == 0 {
        return Err(FoundingError::EmptyGroup);
    }
    let grid = field.grid();
    let seed = field.terrain().seed();
    let mut addresses = Vec::with_capacity(SAMPLE_SIZE as usize);
    for ordinal in 0..SAMPLE_SIZE {
        addresses.push(Axial::new(
            candidate_column(seed, faction, ordinal, grid.width()),
            candidate_row(seed, faction, ordinal, grid.height()),
        ));
    }
    let mut ordered = survey_addresses(field, &addresses, group, taken)?;
    ordered.drawn = SAMPLE_SIZE;
    Ok(ordered)
}

/// Surveys the places a caller names, and orders them.
///
/// The engine calls this with the sample it drew. A caller that wants to
/// compare two places of its own choosing calls it directly.
///
/// # Errors
///
/// Returns an error when the group holds nobody, or when the ordering refuses
/// to run.
pub fn survey_addresses(
    field: ResourceField,
    addresses: &[Axial],
    group: u32,
    taken: &[Axial],
) -> Result<Survey, FoundingError> {
    if group == 0 {
        return Err(FoundingError::EmptyGroup);
    }
    let grid = field.grid();
    let mut seen: Vec<TileIdx> = Vec::with_capacity(addresses.len());
    let mut candidates: Vec<Candidate> = Vec::with_capacity(addresses.len());
    let mut tiles_read = 0u64;

    for address in addresses {
        let Some(tile) = grid.index_of(*address) else {
            continue;
        };
        // Two draws may name one tile. The stable key must be unique, so the
        // repeat is dropped rather than read twice.
        if seen.contains(&tile) {
            continue;
        }
        seen.push(tile);
        let Some((provision, read)) = read_place(field, *address) else {
            continue;
        };
        tiles_read += read;
        let admits = field
            .terrain()
            .kind(*address)
            .is_some_and(TileKind::is_passable);
        // The place keeps the minimum distance from every place a founding
        // before it took. The comparison grows with the number of foundings
        // and not with the world extent, so the cost constraint holds.[^2]
        //
        // [^2]: ADR-0075, the founding choice reads a bounded sample of the world, decision D1. `docs/adrs/accepted/adr-0075-the-founding-choice-reads-a-bounded-sample-of-the-world.md`
        let separated = taken
            .iter()
            .all(|place| place.distance(*address) >= MINIMUM_FOUNDING_DISTANCE);
        candidates.push(Candidate {
            address: *address,
            tile,
            provision,
            score: provision.score(),
            separated,
            eligible: admits && separated && provision.room >= group,
        });
    }

    // The key vector carries the eligibility, then the score from high to
    // low, then the tile index. The tile index is unique inside the sample,
    // so no two keys tie and the order does not depend on the draw order.[^1]
    //
    // [^1]: ADR-0004, iteration order is explicit, decision D4. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    let keys: Vec<SortKey<3>> = candidates
        .iter()
        .map(|candidate| {
            SortKey::new([
                u64::from(!candidate.eligible),
                sort::descending(sort::from_signed(candidate.score.0)),
                u64::from(candidate.tile.0),
            ])
        })
        .collect();
    let order = sort::order(&keys)?;
    let ordered = order
        .into_iter()
        .map(|position| candidates[position as usize])
        .collect();

    Ok(Survey {
        ordered,
        drawn: addresses.len() as u32,
        tiles_read,
    })
}
