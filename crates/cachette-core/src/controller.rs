//! The faction controller, the weight vector it reads, and the game end.
//!
//! # Why this exists
//!
//! Nothing in a world wants anything until a controller runs. A unit gathers
//! where the choice pass sends it, no faction plans, and no run ends. The
//! controller is one system at one fixed stage of the step, the last one, and
//! it acts only through the verbs a Python caller can call.[^1]
//!
//! # What it holds
//!
//! One row for each faction: the four weights that bias its choices, and the
//! flag that says an external caller controls it. Two parameters that the
//! step reads on every tick: the evaluation count and the tick limit. The
//! game end record, empty until a reader fires. Every one of those is state
//! that a later frame reads, so every one enters the state hash.[^2]
//!
//! # Determinism
//!
//! The controller visits the factions in identifier order and makes a fixed
//! number of evaluations for each one. Each evaluation draws once, keyed on
//! this system, the tick, the faction and the draw index.[^3] The commands go
//! to a list that is sorted by faction and then by sequence before any command
//! applies, so the result never depends on the visit order.[^4] There is no
//! convergence test and no time budget.
//!
//! No item in this module uses a floating-point type.[^5]
//!
//! # References
//!
//! [^1]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
//! [^2]: ADR-0148, a game end is recorded once and stops the controllers, decision D1. `docs/adrs/accepted/adr-0148-a-game-end-is-recorded-once-and-stops-the-controllers.md`
//! [^3]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
//! [^4]: ADR-0004, iteration order is explicit, decision D4. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
//! [^5]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`

use bytemuck::{Pod, Zeroable};

use crate::campaign::wants_campaign;
use crate::hash::StateHash;
use crate::rates::RateSchedule;
use crate::resource::{ResourceKind, RESOURCE_KIND_COUNT};
use crate::rng;
use crate::trade::{Advert, ADVERT_OFFERS, ADVERT_WANTS};
use crate::types::{Entity, FactionId, Tick, TileIdx};
use crate::upgrade::{UpgradeCategory, UPGRADE_CATEGORY_COUNT};

/// The lowest weight the seeding layer draws.
///
/// **This is a provisional value and not a measured one.** The balance
/// register holds the row, marks it unset, and records how this value was
/// chosen.[^1]
///
/// # References
///
/// [^1]: Balance register, the weight vector range. `docs/reference/balance.md`
pub const WEIGHT_LOW: u8 = 1;

/// The highest weight the seeding layer draws. Provisional, as above.[^1]
///
/// # References
///
/// [^1]: Balance register, the weight vector range. `docs/reference/balance.md`
pub const WEIGHT_HIGH: u8 = 8;

/// How many evaluations the controller makes for one faction on one tick,
/// when nobody has set another count.
///
/// **This is a provisional value and not a measured one.** The balance
/// register holds the row and marks it unset.[^1]
///
/// # References
///
/// [^1]: Balance register, the controller evaluations per faction per tick. `docs/reference/balance.md`
pub const EVALUATIONS_DEFAULT: u32 = 2;

/// The tick at which the territory reader fires, when nobody has set another
/// limit.
///
/// **This is a provisional value and not a measured one.** The balance
/// register holds the row and marks it unset.[^1]
///
/// # References
///
/// [^1]: Balance register, the tick limit. `docs/reference/balance.md`
pub const TICK_LIMIT_DEFAULT: u64 = 5000;

/// The period of the advertisement schedule, when nobody has set another.
///
/// **This is a provisional value and not a measured one.** The balance
/// register holds the row and marks it unset.[^1]
///
/// # References
///
/// [^1]: Balance register, the advertisement schedule. `docs/reference/balance.md`
pub const ADVERT_PERIOD_DEFAULT: u32 = 10;

/// The phase of the advertisement schedule. Provisional, as above.[^1]
///
/// # References
///
/// [^1]: Balance register, the advertisement schedule. `docs/reference/balance.md`
pub const ADVERT_PHASE_DEFAULT: u32 = 0;

/// The store above which a faction offers a good, and below which it wants
/// one. Provisional, as above.[^1]
///
/// # References
///
/// [^1]: Balance register, the surplus mark. `docs/reference/balance.md`
pub const SURPLUS_MARK_DEFAULT: u32 = 8;

/// How many carriers one contract takes from one faction. Provisional, as
/// above.[^1]
///
/// # References
///
/// [^1]: Balance register, the carriers per contract. `docs/reference/balance.md`
pub const CONTRACT_CARRIERS_DEFAULT: u32 = 2;

/// How many ticks a contract the controller opens runs for. Provisional, as
/// above.[^1]
///
/// # References
///
/// [^1]: Balance register, the contract term. `docs/reference/balance.md`
pub const CONTRACT_TERM_DEFAULT: u32 = 200;

/// The four weights that bias the choices of one faction.
///
/// The vector is drawn from the seed when the world is built, and it is
/// simulated state. Only the build weight is read in this pass. The other
/// three exist so that a later pass reads them without a change to the
/// shape, and so that the hash already covers them.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Pod, Zeroable)]
pub struct FactionWeights {
    /// How much the faction wants a campaign.
    pub war: u8,
    /// How much the faction wants a contract.
    pub trade: u8,
    /// How much the faction wants an upgrade.
    pub build: u8,
    /// How much the faction wants a famous character.
    pub renown: u8,
}

impl FactionWeights {
    /// Draws the vector of one faction from the seed.
    ///
    /// Each weight is one draw at frame zero, keyed on the controller
    /// system, the faction and the index of the weight.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
    #[must_use]
    pub const fn from_seed(seed: u64, faction: FactionId) -> Self {
        let range = (WEIGHT_HIGH - WEIGHT_LOW) as u64 + 1;
        let mut drawn = [0u8; 4];
        let mut index = 0u32;
        while index < 4 {
            let below = rng::draw_below(
                seed,
                rng::SYSTEM_CONTROLLER,
                0,
                faction.0 as u64,
                index,
                range,
            );
            drawn[index as usize] = WEIGHT_LOW + below as u8;
            index += 1;
        }
        Self {
            war: drawn[0],
            trade: drawn[1],
            build: drawn[2],
            renown: drawn[3],
        }
    }
}

/// One row of the controller table, for one faction.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Pod, Zeroable)]
pub struct FactionRow {
    /// The seat: the tile of the first founding of the faction, or
    /// `NO_SEAT` while the faction founded nothing.
    ///
    /// The controller plans around the seat. A faction with no seat has
    /// nothing to plan around and receives no evaluation, so a world that
    /// spawned units by hand and founded nothing runs no controller.
    pub seat: u32,
    /// The weight vector.
    pub weights: FactionWeights,
    /// One when an external caller controls the faction, zero otherwise.
    ///
    /// A faction under external control receives no evaluation. Nothing in
    /// the engine sets this. It exists so that a later player hook has a
    /// place to stand.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decision D6. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
    pub externally_controlled: u8,
    /// Declared padding, always zero.
    pub padding: [u8; 3],
}

/// The seat value of a faction that founded nothing.
pub const NO_SEAT: u32 = u32::MAX;

impl FactionRow {
    /// Returns the seat, or `None` while the faction founded nothing.
    #[must_use]
    pub const fn seat(self) -> Option<TileIdx> {
        if self.seat == NO_SEAT {
            None
        } else {
            Some(TileIdx(self.seat))
        }
    }
}

/// The path by which a game ended.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WinPath {
    /// One faction holds every seat, or every other faction has no units.
    Domination = 0,
    /// At the tick limit, the faction with the most held tiles.
    Territory = 1,
    /// A stock total reaches a target, or a wonder completes.
    WealthOrWonder = 2,
    /// A character reaches a renown target.
    Renown = 3,
}

impl WinPath {
    /// Returns the path for its number, or `None` when the number names none.
    #[must_use]
    pub const fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Domination),
            1 => Some(Self::Territory),
            2 => Some(Self::WealthOrWonder),
            3 => Some(Self::Renown),
            _ => None,
        }
    }

    /// Returns the number of the path.
    #[must_use]
    pub const fn to_u8(self) -> u8 {
        self as u8
    }

    /// Returns the name of the path, as a reader prints it.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Domination => "domination",
            Self::Territory => "territory",
            Self::WealthOrWonder => "wealth_or_wonder",
            Self::Renown => "renown",
        }
    }
}

/// The record of how a game ended.
///
/// A world with no game end holds an empty record, and the hash covers the
/// empty record the same way. The record is written once, and nothing
/// rewrites it.[^1]
///
/// # References
///
/// [^1]: ADR-0148, a game end is recorded once and stops the controllers, decisions D1 and D2. `docs/adrs/accepted/adr-0148-a-game-end-is-recorded-once-and-stops-the-controllers.md`
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Pod, Zeroable)]
pub struct GameEnd {
    /// The tick at which the reader fired.
    pub tick: Tick,
    /// The faction that won.
    pub winner: FactionId,
    /// The number of the path, as `WinPath` numbers them.
    pub path: u8,
    /// One when the record is written, zero while it is empty.
    pub set: u8,
    /// Declared padding, always zero.
    pub padding: [u8; 4],
}

impl GameEnd {
    /// The empty record.
    pub const EMPTY: Self = Self {
        tick: Tick(0),
        winner: FactionId(0),
        path: 0,
        set: 0,
        padding: [0; 4],
    };

    /// Returns whether the record is written.
    #[must_use]
    pub const fn is_set(&self) -> bool {
        self.set != 0
    }

    /// Returns the path, or `None` while the record is empty.
    #[must_use]
    pub const fn win_path(&self) -> Option<WinPath> {
        if self.set == 0 {
            return None;
        }
        WinPath::from_u8(self.path)
    }
}

/// The kind of command the controller emitted: a gather order.
pub const COMMAND_GATHER: u8 = 0;

/// The kind of command the controller emitted: a build order.
pub const COMMAND_BUILD: u8 = 1;

/// The kind of command the controller emitted: a relation move against
/// another faction.
pub const COMMAND_RELATION: u8 = 2;

/// The kind of command the controller emitted: a campaign raised against a
/// faction at war. The argument is the objective kind.
pub const COMMAND_CAMPAIGN: u8 = 3;

/// The kind of command the controller emitted: a whole board written from the
/// site economies of the faction. The argument is how many rows it wrote.
pub const COMMAND_ADVERTISE: u8 = 4;

/// The kind of command the controller emitted: one negotiation step against
/// another faction. The argument is that faction.
pub const COMMAND_TRADE: u8 = 5;

/// The kind of command the controller emitted: the carriers of every contract
/// the faction owes a quantity on. The argument is how many units it assigned.
pub const COMMAND_CARRY: u8 = 6;

/// The step the controller moves a relation by when its draw says so. It is
/// one step toward war, and the drift is what brings the pair back.[^1]
///
/// # References
///
/// [^1]: ADR-0146, a faction relation is one signed integer per ordered pair, and a pass reads a threshold, decision D3. `docs/adrs/accepted/adr-0146-a-faction-relation-is-one-signed-integer-per-ordered-pair-and-a-pass-reads-a-threshold.md`
pub const RELATION_STEP: i32 = -1;

/// One command the controller emitted on the last tick.
///
/// The log is the record of what the controller asked for and whether the
/// verb took it. A caller reads it to see the controller act. It is a log
/// of one tick, and the step empties it before the controller runs.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Pod, Zeroable)]
pub struct ControllerCommand {
    /// The tick the controller emitted it on.
    pub tick: Tick,
    /// The faction it was emitted for.
    pub faction: FactionId,
    /// The kind of command: a gather order or a build order.
    pub kind: u8,
    /// The resource kind of a gather order, or the upgrade kind of a build
    /// order.
    pub argument: u8,
    /// The draw index that produced it. Commands apply in the order of
    /// faction and then sequence.
    pub sequence: u32,
    /// One when the verb took the command for at least one unit, zero when
    /// it refused every unit.
    pub applied: u8,
    /// Declared padding, always zero.
    pub padding: [u8; 7],
}

/// What one evaluation chose.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Choice {
    /// Order the units of the faction to gather one kind.
    Gather(ResourceKind),
    /// Order the units of the faction to build one kind.
    Build(UpgradeCategory),
    /// Move the relation of the faction toward another by one step.
    Relation(FactionId),
    /// Raise a campaign of one objective kind against one tile.
    Campaign {
        /// The objective kind, as the campaign module numbers it.
        kind: u8,
        /// The objective tile.
        tile: TileIdx,
    },
    /// Rewrite the whole board of the faction from its site economies.
    ///
    /// The rows are built when the command applies, so the write reads the
    /// stores of this tick and not those of the tick the plan was made
    /// on.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0149, a faction's trade board is simulated state that any faction may read, decision D3. `docs/adrs/accepted/adr-0149-a-factions-trade-board-is-simulated-state-that-any-faction-may-read.md`
    Advertise,
    /// Take one negotiation step against one other faction.
    ///
    /// The step is chosen when the command applies, for the reason the board
    /// write is. A faction that finds no step is refused, and the refusal
    /// counts.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decision D3. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
    Trade,
    /// Assign the carriers of every contract the faction owes a quantity on,
    /// and release the carriers of every contract that ended.
    Carry,
}

impl Choice {
    /// Returns the kind number and the argument number of the choice.
    ///
    /// The argument of a relation move is the other faction. The faction
    /// ceiling is below the range of the column, so the narrowing loses
    /// nothing. The argument of a campaign is the objective kind, and the
    /// campaign log holds the tile.
    #[must_use]
    pub const fn numbers(self) -> (u8, u8) {
        match self {
            Self::Gather(kind) => (COMMAND_GATHER, kind.to_u8()),
            Self::Build(kind) => (COMMAND_BUILD, kind.to_u8()),
            Self::Relation(other) => (COMMAND_RELATION, other.0 as u8),
            Self::Campaign { kind, .. } => (COMMAND_CAMPAIGN, kind),
            Self::Advertise => (COMMAND_ADVERTISE, 0),
            Self::Trade => (COMMAND_TRADE, 0),
            Self::Carry => (COMMAND_CARRY, 0),
        }
    }
}

/// Decides whether a faction moves its relation toward its rival this tick.
///
/// **This draws exactly once.** The key is the controller system, the tick,
/// the faction and the draw index, and the index is one past the evaluation
/// indexes so it collides with none of them.[^1] The war weight biases the
/// draw: the answer is yes with probability `war / (WEIGHT_HIGH + war)`.
///
/// # References
///
/// [^1]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
#[must_use]
pub fn wants_relation_move(
    seed: u64,
    tick: Tick,
    faction: FactionId,
    draw: u32,
    weights: FactionWeights,
) -> bool {
    let raw = rng::draw(seed, rng::SYSTEM_CONTROLLER, tick.0, faction.0 as u64, draw);
    let bound = u64::from(WEIGHT_HIGH) + u64::from(weights.war);
    let roll = ((u128::from(raw) * u128::from(bound)) >> 64) as u64;
    roll < u64::from(weights.war)
}

/// Picks the rival of a faction: the other faction with the most held tiles.
///
/// **A tie resolves by the lowest faction identifier**, in the way the
/// territory winner does. Returns `None` when no other faction exists.
#[must_use]
pub fn rival_of(
    faction: FactionId,
    held: impl Iterator<Item = (FactionId, i64)>,
) -> Option<FactionId> {
    territory_winner(held.filter(|(other, _)| *other != faction))
}

/// Makes one evaluation for one faction.
///
/// **This draws exactly once.** The key is the controller system, the tick,
/// the faction and the draw index.[^1] The build weight biases the draw: the
/// choice is a build order with probability `build / (WEIGHT_HIGH + build)`,
/// and a gather order otherwise. The kind comes from the high bits of the
/// same draw, so a second draw is never needed.
///
/// # References
///
/// [^1]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
#[must_use]
pub fn evaluate(
    seed: u64,
    tick: Tick,
    faction: FactionId,
    draw: u32,
    weights: FactionWeights,
) -> Choice {
    let raw = rng::draw(seed, rng::SYSTEM_CONTROLLER, tick.0, faction.0 as u64, draw);
    let bound = u64::from(WEIGHT_HIGH) + u64::from(weights.build);
    let roll = ((u128::from(raw) * u128::from(bound)) >> 64) as u64;
    let high = (raw >> 32) as u32;
    if roll < u64::from(weights.build) {
        let index = (high % UPGRADE_CATEGORY_COUNT as u32) as u8;
        Choice::Build(UpgradeCategory::from_u8(index).expect("the index is below the count"))
    } else {
        let index = (high % RESOURCE_KIND_COUNT as u32) as u8;
        Choice::Gather(ResourceKind::from_u8(index).expect("the index is below the count"))
    }
}

/// Picks the territory winner: the faction with the most held tiles.
///
/// **A tie resolves by the lowest faction identifier.** The scan visits the
/// factions in ascending identifier order and replaces the leader only on a
/// strictly greater count, so the first faction to reach the highest count
/// keeps it.[^1] A world with no faction has no winner.
///
/// # References
///
/// [^1]: ADR-0148, a game end is recorded once and stops the controllers, decision D3. `docs/adrs/accepted/adr-0148-a-game-end-is-recorded-once-and-stops-the-controllers.md`
#[must_use]
pub fn territory_winner(held: impl Iterator<Item = (FactionId, i64)>) -> Option<FactionId> {
    let mut leader: Option<(FactionId, i64)> = None;
    for (faction, count) in held {
        match leader {
            Some((_, best)) if count <= best => {}
            _ => leader = Some((faction, count)),
        }
    }
    leader.map(|(faction, _)| faction)
}

/// One carrier the controller assigned to one contract.
///
/// The row is simulated state. A later frame reads it to release the unit
/// when the contract ends, so every byte of it enters the state hash.[^1]
///
/// The layout is 8 + 4 + 2 + 2 bytes, which is 16 bytes at an alignment of 8.
/// The trailing array declares every padding byte.[^2]
///
/// # References
///
/// [^1]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
/// [^2]: ADR-0006, an event is plain data and applying it is pure, decision D1. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Pod, Zeroable)]
pub struct CarrierAssignment {
    /// The identity of the unit, as one integer.
    pub unit: u64,
    /// The index of the contract row in the negotiation plane.
    pub row: u32,
    /// The faction that assigned the unit.
    pub faction: FactionId,
    /// Declared padding, always zero.
    pub padding: [u8; 2],
}

/// The size of one carrier row, in bytes.
pub const CARRIER_ASSIGNMENT_BYTES: usize = 16;

const _: () = assert!(core::mem::size_of::<CarrierAssignment>() == CARRIER_ASSIGNMENT_BYTES);

impl CarrierAssignment {
    /// Builds a row with zero padding.
    #[must_use]
    pub const fn new(unit: Entity, row: u32, faction: FactionId) -> Self {
        Self {
            unit: unit.to_bits(),
            row,
            faction,
            padding: [0; 2],
        }
    }
}

/// Decides whether a faction takes its one negotiation step this tick.
///
/// **This draws exactly once.** The key is the controller system, the tick,
/// the faction and the draw index, and the index is past the campaign
/// draw.[^1] The trade weight biases the draw: the answer is yes with
/// probability `trade / (WEIGHT_HIGH + trade)`.
///
/// # References
///
/// [^1]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
#[must_use]
pub fn wants_trade_step(
    seed: u64,
    tick: Tick,
    faction: FactionId,
    draw: u32,
    weights: FactionWeights,
) -> bool {
    let raw = rng::draw(seed, rng::SYSTEM_CONTROLLER, tick.0, faction.0 as u64, draw);
    let bound = u64::from(WEIGHT_HIGH) + u64::from(weights.trade);
    let roll = ((u128::from(raw) * u128::from(bound)) >> 64) as u64;
    roll < u64::from(weights.trade)
}

/// Picks the good that a faction asks for in return for one good.
///
/// The answer is the good the faction holds least of, and never the good the
/// row is about. **The scan draws nothing while one good is lowest.** Two
/// goods that tie are separated by one keyed draw, because a tie broken by
/// the index would always name the same good and the board would never ask
/// for the other.[^1]
///
/// # References
///
/// [^1]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
#[must_use]
pub fn asking_good_of(
    seed: u64,
    tick: Tick,
    faction: FactionId,
    draw: u32,
    stores: &[i64; RESOURCE_KIND_COUNT],
    good: u8,
) -> u8 {
    let mut lowest = i64::MAX;
    let mut tied = [0u8; RESOURCE_KIND_COUNT];
    let mut count = 0usize;
    for (index, held) in stores.iter().enumerate() {
        if index as u8 == good {
            continue;
        }
        if *held < lowest {
            lowest = *held;
            count = 0;
        }
        if *held == lowest {
            tied[count] = index as u8;
            count += 1;
        }
    }
    if count == 0 {
        return good;
    }
    if count == 1 {
        return tied[0];
    }
    let raw = rng::draw(seed, rng::SYSTEM_CONTROLLER, tick.0, faction.0 as u64, draw);
    let picked = ((u128::from(raw) * count as u128) >> 64) as usize;
    tied[picked.min(count - 1)]
}

/// Builds the whole board of one faction from what its sites hold.
///
/// The goods are visited in index order. A good above the mark becomes an
/// offer row of the surplus. A good below the mark becomes a want row of the
/// shortfall. A good at the mark writes no row. The asking quantity equals
/// the quantity, which is an even swap and not a price.[^1]
///
/// The list is truncated to the board bound, because a write past the bound
/// is refused and changes nothing.[^2]
///
/// # References
///
/// [^1]: Balance register, the surplus mark. `docs/reference/balance.md`
/// [^2]: ADR-0149, a faction's trade board is simulated state that any faction may read, decision D2. `docs/adrs/accepted/adr-0149-a-factions-trade-board-is-simulated-state-that-any-faction-may-read.md`
#[must_use]
pub fn board_of(
    seed: u64,
    tick: Tick,
    faction: FactionId,
    draw: u32,
    stores: &[i64; RESOURCE_KIND_COUNT],
    mark: i64,
    bound: usize,
) -> Vec<Advert> {
    let mut rows = Vec::with_capacity(RESOURCE_KIND_COUNT);
    for (index, held) in stores.iter().enumerate() {
        let good = index as u8;
        let (side, quantity) = if *held > mark {
            (ADVERT_OFFERS, held.saturating_sub(mark))
        } else if *held < mark {
            (ADVERT_WANTS, mark.saturating_sub(*held))
        } else {
            continue;
        };
        let quantity = u32::try_from(quantity).unwrap_or(u32::MAX);
        let asking = asking_good_of(seed, tick, faction, draw, stores, good);
        rows.push(Advert::new(good, quantity, side, asking, quantity));
    }
    rows.truncate(bound);
    rows
}

/// The terms of the offer that one faction opens against another.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Terms {
    /// The good the faction that opens the pair owes.
    pub give_kind: u8,
    /// How much of it.
    pub give_amount: u32,
    /// The good the other party owes.
    pub take_kind: u8,
    /// How much of it.
    pub take_amount: u32,
}

/// Matches the wants of one board against the offers of another.
///
/// The rows of each board are read in slot order, and the first pair that
/// names one good is the answer. **The quantity is the smaller of the two**,
/// so neither side promises more than it posted.
///
/// **The buyer names the good it pays with.** The seller states a preference
/// on its own row, and the two preferences may differ, because each is drawn
/// against what that faction lacks. A match that asked the two to agree would
/// wait for two draws to meet.
///
/// Returns `None` when no row of the first board meets a row of the second.
#[must_use]
pub fn match_boards(mine: &[Advert], theirs: &[Advert]) -> Option<Terms> {
    for want in mine {
        if want.is_empty() || want.wants != ADVERT_WANTS {
            continue;
        }
        for offer in theirs {
            if offer.is_empty() || offer.wants != ADVERT_OFFERS {
                continue;
            }
            if offer.good != want.good || want.asking_good == want.good {
                continue;
            }
            return Some(Terms {
                give_kind: want.asking_good,
                give_amount: want.asking_quantity,
                take_kind: want.good,
                take_amount: offer.quantity.min(want.quantity),
            });
        }
    }
    None
}

/// Returns the quantity a faction asks for one good on its own board, or
/// `None` when its board says nothing about that good.
#[must_use]
pub fn asking_quantity_of(board: &[Advert], good: u8) -> Option<u32> {
    board
        .iter()
        .find(|row| !row.is_empty() && row.good == good)
        .map(|row| row.asking_quantity)
}

/// Returns the integer midpoint of two asking quantities.
///
/// The sum is taken as a wide integer, so two quantities near the ceiling of
/// the column give the midpoint and never an overflow. **No draw decides a
/// price.**
#[must_use]
pub const fn midpoint(one: u32, other: u32) -> u32 {
    ((one as u64 + other as u64) / 2) as u32
}

/// Returns whether a faction accepts a counteroffer.
///
/// The answer is yes when the counter asks no more than the faction posted.
#[must_use]
pub const fn accepts(counter: u32, own_ask: u32) -> bool {
    counter <= own_ask
}

/// What the world offers one faction when the stage plans its tick.
///
/// The stage reads the world once for each faction and hands the answers to
/// the plan. The plan then draws and orders, and it reads no world of its
/// own. The rows are in faction order, one for each faction.[^1]
///
/// # References
///
/// [^1]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decision D1. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FactionState {
    /// The faction it would move a relation against, or `None` when it holds
    /// no leader unit and when no other faction exists.
    pub rival: Option<FactionId>,
    /// The objective kind and the tile it would march on, or `None` when no
    /// pair it belongs to is at war, when no enemy site exists, when it holds
    /// a live campaign, and when it holds a carrier.
    pub objective: Option<(u8, TileIdx)>,
    /// Whether the advertisement schedule falls due for it on this tick.
    pub board_due: bool,
    /// Whether it holds a negotiation step to take.
    pub trade_due: bool,
    /// Whether it holds a carrier to assign or to release.
    pub carry_due: bool,
}

/// The controller state the world holds.
#[derive(Clone, Debug)]
pub struct Controller {
    rows: Vec<FactionRow>,
    evaluations: u32,
    tick_limit: u64,
    game_end: GameEnd,
    log: Vec<ControllerCommand>,
    refused: u32,
    advert: RateSchedule,
    surplus_mark: u32,
    contract_carriers: u32,
    contract_term: u32,
    carriers: Vec<CarrierAssignment>,
    boards_written: u32,
    offers_made: u32,
    contracts_bound: u32,
    carriers_assigned: u32,
}

impl Controller {
    /// Builds the table for a world, drawing every weight vector from the
    /// seed.
    #[must_use]
    pub fn new(seed: u64, faction_count: u16) -> Self {
        let rows = (0..faction_count)
            .map(|index| FactionRow {
                seat: NO_SEAT,
                weights: FactionWeights::from_seed(seed, FactionId(index)),
                externally_controlled: 0,
                padding: [0; 3],
            })
            .collect();
        Self {
            rows,
            evaluations: EVALUATIONS_DEFAULT,
            tick_limit: TICK_LIMIT_DEFAULT,
            game_end: GameEnd::EMPTY,
            log: Vec::new(),
            refused: 0,
            advert: RateSchedule::new(ADVERT_PERIOD_DEFAULT, ADVERT_PHASE_DEFAULT)
                .expect("the default period is inside the range"),
            surplus_mark: SURPLUS_MARK_DEFAULT,
            contract_carriers: CONTRACT_CARRIERS_DEFAULT,
            contract_term: CONTRACT_TERM_DEFAULT,
            carriers: Vec::new(),
            boards_written: 0,
            offers_made: 0,
            contracts_bound: 0,
            carriers_assigned: 0,
        }
    }

    /// Returns the row of one faction, or `None` when the world has no such
    /// faction.
    #[must_use]
    pub fn row(&self, faction: FactionId) -> Option<FactionRow> {
        self.rows.get(usize::from(faction.0)).copied()
    }

    /// Returns every row, in faction order.
    #[must_use]
    pub fn rows(&self) -> &[FactionRow] {
        &self.rows
    }

    /// Records the seat of a faction, once. A later call leaves the first
    /// seat where it is.
    pub fn set_seat(&mut self, faction: FactionId, tile: TileIdx) {
        if let Some(row) = self.rows.get_mut(usize::from(faction.0)) {
            if row.seat == NO_SEAT {
                row.seat = tile.0;
            }
        }
    }

    /// Sets the flag that says an external caller controls a faction.
    ///
    /// Returns `false` when the world has no such faction.
    pub fn set_externally_controlled(&mut self, faction: FactionId, controlled: bool) -> bool {
        let Some(row) = self.rows.get_mut(usize::from(faction.0)) else {
            return false;
        };
        row.externally_controlled = u8::from(controlled);
        true
    }

    /// Returns how many evaluations the controller makes for one faction on
    /// one tick.
    #[must_use]
    pub const fn evaluations(&self) -> u32 {
        self.evaluations
    }

    /// Sets how many evaluations the controller makes for one faction on one
    /// tick.
    pub const fn set_evaluations(&mut self, evaluations: u32) {
        self.evaluations = evaluations;
    }

    /// Returns the tick at which the territory reader fires.
    #[must_use]
    pub const fn tick_limit(&self) -> u64 {
        self.tick_limit
    }

    /// Sets the tick at which the territory reader fires.
    pub const fn set_tick_limit(&mut self, tick_limit: u64) {
        self.tick_limit = tick_limit;
    }

    /// Returns the game end record. It is empty until a reader fires.
    #[must_use]
    pub const fn game_end(&self) -> GameEnd {
        self.game_end
    }

    /// Writes the game end record, once.
    ///
    /// Returns `false` and changes nothing when the record is already
    /// written.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0148, a game end is recorded once and stops the controllers, decision D2. `docs/adrs/accepted/adr-0148-a-game-end-is-recorded-once-and-stops-the-controllers.md`
    pub fn record_end(&mut self, tick: Tick, winner: FactionId, path: WinPath) -> bool {
        if self.game_end.is_set() {
            return false;
        }
        self.game_end = GameEnd {
            tick,
            winner,
            path: path.to_u8(),
            set: 1,
            padding: [0; 4],
        };
        true
    }

    /// Returns the commands of the last tick, in the order they applied.
    #[must_use]
    pub fn log(&self) -> &[ControllerCommand] {
        &self.log
    }

    /// Returns how many commands the verbs refused outright on the last
    /// tick.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decision D3. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
    #[must_use]
    pub const fn refused(&self) -> u32 {
        self.refused
    }

    /// Returns how many commands a verb took on the last tick.
    #[must_use]
    pub fn applied(&self) -> u32 {
        self.log.iter().filter(|entry| entry.applied != 0).count() as u32
    }

    /// Empties the log of the last tick, and the counts of the last tick.
    pub fn clear_log(&mut self) {
        self.log.clear();
        self.refused = 0;
        self.boards_written = 0;
        self.offers_made = 0;
        self.contracts_bound = 0;
        self.carriers_assigned = 0;
    }

    /// Returns the advertisement schedule: how often a faction rewrites its
    /// board, and the offset inside the period.
    #[must_use]
    pub const fn advert_schedule(&self) -> RateSchedule {
        self.advert
    }

    /// Sets the advertisement schedule.
    pub const fn set_advert_schedule(&mut self, schedule: RateSchedule) {
        self.advert = schedule;
    }

    /// Returns the store above which a faction offers a good, and below which
    /// it wants one.
    #[must_use]
    pub const fn surplus_mark(&self) -> u32 {
        self.surplus_mark
    }

    /// Sets the surplus mark.
    pub const fn set_surplus_mark(&mut self, mark: u32) {
        self.surplus_mark = mark;
    }

    /// Returns how many carriers one faction assigns to one contract.
    #[must_use]
    pub const fn contract_carriers(&self) -> u32 {
        self.contract_carriers
    }

    /// Sets how many carriers one faction assigns to one contract.
    pub const fn set_contract_carriers(&mut self, carriers: u32) {
        self.contract_carriers = carriers;
    }

    /// Returns how many ticks a contract that the controller opens runs for.
    #[must_use]
    pub const fn contract_term(&self) -> u32 {
        self.contract_term
    }

    /// Sets how many ticks a contract that the controller opens runs for.
    pub const fn set_contract_term(&mut self, term: u32) {
        self.contract_term = term;
    }

    /// Returns every carrier the controller has assigned, in faction order
    /// and then in contract order and then in identity order.
    #[must_use]
    pub fn carriers(&self) -> &[CarrierAssignment] {
        &self.carriers
    }

    /// Replaces the carrier list, and puts it in the one order a reader sees.
    ///
    /// The key is the faction, the contract row and the identity of the unit.
    /// The key is unique, because one unit carries for one contract, so a
    /// stable sort and an unstable sort give one answer here.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0004, iteration order is explicit, decision D4. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    pub fn set_carriers(&mut self, mut carriers: Vec<CarrierAssignment>) {
        carriers.sort_unstable_by_key(|entry| (entry.faction, entry.row, entry.unit));
        self.carriers = carriers;
    }

    /// Counts one board the stage wrote on this tick.
    pub const fn count_board(&mut self) {
        self.boards_written = self.boards_written.saturating_add(1);
    }

    /// Counts one offer the stage opened on this tick.
    pub const fn count_offer(&mut self) {
        self.offers_made = self.offers_made.saturating_add(1);
    }

    /// Counts one contract the stage bound on this tick.
    pub const fn count_bound(&mut self) {
        self.contracts_bound = self.contracts_bound.saturating_add(1);
    }

    /// Counts the carriers the stage assigned on this tick.
    pub const fn count_carriers(&mut self, count: u32) {
        self.carriers_assigned = self.carriers_assigned.saturating_add(count);
    }

    /// Returns how many boards the stage wrote on the last tick.
    #[must_use]
    pub const fn boards_written(&self) -> u32 {
        self.boards_written
    }

    /// Returns how many offers the stage opened on the last tick.
    #[must_use]
    pub const fn offers_made(&self) -> u32 {
        self.offers_made
    }

    /// Returns how many contracts the stage bound on the last tick.
    #[must_use]
    pub const fn contracts_bound(&self) -> u32 {
        self.contracts_bound
    }

    /// Returns how many carriers the stage assigned on the last tick.
    #[must_use]
    pub const fn carriers_assigned(&self) -> u32 {
        self.carriers_assigned
    }

    /// Records one command and how the verb answered it.
    pub fn push(&mut self, command: ControllerCommand) {
        if command.applied == 0 {
            self.refused = self.refused.wrapping_add(1);
        }
        self.log.push(command);
    }

    /// Plans the commands of one tick, in the order they must apply.
    ///
    /// The factions are visited in identifier order, and each faction under
    /// external control is skipped. The list is sorted by faction and then by
    /// sequence before it is returned, so the caller applies the commands in
    /// an order the data fixes.[^1]
    ///
    /// Returns an empty list when the game end record is written.[^2]
    ///
    /// The states list holds one row for each faction, in faction order. The
    /// row says what the world offers that faction this tick: the rival it
    /// would move a relation against, the objective it would march on, and
    /// whether a board write, a negotiation step or a carrier move is due.
    /// A faction with a rival draws once more, at the index past the
    /// evaluations, and the draw decides whether it moves.[^3] A faction with
    /// an objective draws once more, at the index past that, and the draw
    /// decides whether it raises.[^4] A faction with a negotiation step due
    /// draws once more, at the index past the board write, and the draw
    /// decides whether it speaks.
    ///
    /// # References
    ///
    /// [^1]: ADR-0004, iteration order is explicit, decision D4. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    /// [^2]: ADR-0148, a game end is recorded once and stops the controllers, decision D4. `docs/adrs/accepted/adr-0148-a-game-end-is-recorded-once-and-stops-the-controllers.md`
    /// [^3]: ADR-0146, a faction relation is one signed integer per ordered pair, and a pass reads a threshold, decision D5. `docs/adrs/accepted/adr-0146-a-faction-relation-is-one-signed-integer-per-ordered-pair-and-a-pass-reads-a-threshold.md`
    /// [^4]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decision D4. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
    #[must_use]
    pub fn plan(
        &self,
        seed: u64,
        tick: Tick,
        states: &[FactionState],
    ) -> Vec<(FactionId, u32, Choice)> {
        let mut commands = Vec::new();
        if self.game_end.is_set() {
            return commands;
        }
        for (index, row) in self.visit_order() {
            if row.externally_controlled != 0 || row.seat == NO_SEAT {
                continue;
            }
            let faction = FactionId(index);
            for draw in self.draw_order() {
                let choice = evaluate(seed, tick, faction, draw, row.weights);
                commands.push((faction, draw, choice));
            }
            let state = states.get(usize::from(index)).copied().unwrap_or_default();
            if let Some(rival) = state.rival {
                let draw = self.relation_draw_index();
                if wants_relation_move(seed, tick, faction, draw, row.weights) {
                    commands.push((faction, draw, Choice::Relation(rival)));
                }
            }
            if let Some((kind, tile)) = state.objective {
                let draw = self.campaign_draw_index();
                if wants_campaign(seed, tick, faction, draw, row.weights) {
                    commands.push((faction, draw, Choice::Campaign { kind, tile }));
                }
            }
            if state.board_due {
                commands.push((faction, self.board_draw_index(), Choice::Advertise));
            }
            if state.trade_due {
                let draw = self.trade_draw_index();
                if wants_trade_step(seed, tick, faction, draw, row.weights) {
                    commands.push((faction, draw, Choice::Trade));
                }
            }
            if state.carry_due {
                commands.push((faction, self.carry_draw_index(), Choice::Carry));
            }
        }
        // The visit order above is fixed, and the sort is what makes the
        // applied order independent of it. The key is unique, because one
        // faction makes each draw once, so a stable sort and an unstable sort
        // give one answer here.
        commands.sort_by_key(|(faction, draw, _)| (*faction, *draw));
        commands
    }

    /// Returns the draw index of the relation move: one past the evaluation
    /// indexes, so it collides with none of them.
    #[must_use]
    pub const fn relation_draw_index(&self) -> u32 {
        self.evaluations
    }

    /// Returns the draw index of the campaign draw: one past the relation
    /// draw, so it collides with no other.
    #[must_use]
    pub const fn campaign_draw_index(&self) -> u32 {
        self.evaluations + 1
    }

    /// Returns the draw index of the board write: one past the campaign draw.
    ///
    /// The board write draws only to break a tie between two equally lacked
    /// goods. The index is reserved whether it draws or not, so no other
    /// draw of this stage ever takes it.
    #[must_use]
    pub const fn board_draw_index(&self) -> u32 {
        self.evaluations + 2
    }

    /// Returns the draw index of the negotiation step: one past the board
    /// write, so it collides with no other.
    #[must_use]
    pub const fn trade_draw_index(&self) -> u32 {
        self.evaluations + 3
    }

    /// Returns the draw index of the carrier command: one past the
    /// negotiation step. The carrier command draws nothing, and the index
    /// puts it last in the order the commands apply.
    #[must_use]
    pub const fn carry_draw_index(&self) -> u32 {
        self.evaluations + 4
    }

    /// Reports whether a faction rewrites its board on this tick.
    #[must_use]
    pub const fn board_due(&self, tick: Tick) -> bool {
        self.advert.due(tick)
    }

    /// Returns the rows in ascending faction order.
    #[cfg(not(feature = "probe-nondeterminism"))]
    fn visit_order(&self) -> impl Iterator<Item = (u16, FactionRow)> + '_ {
        self.rows
            .iter()
            .enumerate()
            .map(|(index, row)| (index as u16, *row))
    }

    /// Returns the rows in descending faction order.
    ///
    /// This is the test-only switch. It visits the factions backwards, so
    /// that a test can prove the sort restores the order. Never build a
    /// release with it.
    #[cfg(feature = "probe-nondeterminism")]
    fn visit_order(&self) -> impl Iterator<Item = (u16, FactionRow)> + '_ {
        self.rows
            .iter()
            .enumerate()
            .rev()
            .map(|(index, row)| (index as u16, *row))
    }

    /// Returns the draw indexes in ascending order.
    #[cfg(not(feature = "probe-nondeterminism"))]
    fn draw_order(&self) -> impl Iterator<Item = u32> {
        0..self.evaluations
    }

    /// Returns the draw indexes in descending order. The test-only switch.
    #[cfg(feature = "probe-nondeterminism")]
    fn draw_order(&self) -> impl Iterator<Item = u32> {
        (0..self.evaluations).rev()
    }

    /// Folds every value that a later frame reads into the hash.
    ///
    /// The rows, the two parameters and the game end record all enter. The
    /// log does not, because it is a log of one tick and the next step
    /// empties it.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
    #[must_use]
    pub fn hash_into(&self, hash: StateHash) -> StateHash {
        hash.write_u64(self.rows.len() as u64)
            .write(bytemuck::cast_slice(&self.rows))
            .write_u64(u64::from(self.evaluations))
            .write_u64(self.tick_limit)
            .write(bytemuck::bytes_of(&self.game_end))
            .write_u64(u64::from(self.advert.period()))
            .write_u64(u64::from(self.advert.phase()))
            .write_u64(u64::from(self.surplus_mark))
            .write_u64(u64::from(self.contract_carriers))
            .write_u64(u64::from(self.contract_term))
            .write_u64(self.carriers.len() as u64)
            .write(bytemuck::cast_slice(&self.carriers))
    }
}
