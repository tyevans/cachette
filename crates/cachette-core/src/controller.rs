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

use crate::hash::StateHash;
use crate::resource::{ResourceKind, RESOURCE_KIND_COUNT};
use crate::rng;
use crate::types::{FactionId, Tick, TileIdx};
use crate::upgrade::{UpgradeKind, UPGRADE_KIND_COUNT};

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
pub const TICK_LIMIT_DEFAULT: u64 = 2000;

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

/// The step the controller moves a relation by when its draw says so. It is
/// one step toward war, and the drift is what brings the pair back.[^1]
///
/// # References
///
/// [^1]: ADR-0146, a faction relation is one signed integer per ordered pair, and a pass reads a threshold, decision D3. `docs/adrs/draft/adr-0146-a-faction-relation-is-one-signed-integer-per-ordered-pair-and-a-pass-reads-a-threshold.md`
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
    Build(UpgradeKind),
    /// Move the relation of the faction toward another by one step.
    Relation(FactionId),
}

impl Choice {
    /// Returns the kind number and the argument number of the choice.
    ///
    /// The argument of a relation move is the other faction. The faction
    /// ceiling is below the range of the column, so the narrowing loses
    /// nothing.
    #[must_use]
    pub const fn numbers(self) -> (u8, u8) {
        match self {
            Self::Gather(kind) => (COMMAND_GATHER, kind.to_u8()),
            Self::Build(kind) => (COMMAND_BUILD, kind.to_u8()),
            Self::Relation(other) => (COMMAND_RELATION, other.0 as u8),
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
        let index = (high % UPGRADE_KIND_COUNT as u32) as u8;
        Choice::Build(UpgradeKind::from_u8(index).expect("the index is below the count"))
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

/// The controller state the world holds.
#[derive(Clone, Debug)]
pub struct Controller {
    rows: Vec<FactionRow>,
    evaluations: u32,
    tick_limit: u64,
    game_end: GameEnd,
    log: Vec<ControllerCommand>,
    refused: u32,
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

    /// Empties the log of the last tick.
    pub fn clear_log(&mut self) {
        self.log.clear();
        self.refused = 0;
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
    /// The rivals list holds, for each faction, the faction it would move a
    /// relation against, or `None` when it holds no leader unit or no other
    /// faction exists. A faction with a rival draws once more, at the index
    /// past the evaluations, and the draw decides whether it moves.[^3]
    ///
    /// # References
    ///
    /// [^1]: ADR-0004, iteration order is explicit, decision D4. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    /// [^2]: ADR-0148, a game end is recorded once and stops the controllers, decision D4. `docs/adrs/accepted/adr-0148-a-game-end-is-recorded-once-and-stops-the-controllers.md`
    /// [^3]: ADR-0146, a faction relation is one signed integer per ordered pair, and a pass reads a threshold, decision D5. `docs/adrs/draft/adr-0146-a-faction-relation-is-one-signed-integer-per-ordered-pair-and-a-pass-reads-a-threshold.md`
    #[must_use]
    pub fn plan(
        &self,
        seed: u64,
        tick: Tick,
        rivals: &[Option<FactionId>],
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
            let Some(Some(rival)) = rivals.get(usize::from(index)).copied() else {
                continue;
            };
            let draw = self.relation_draw_index();
            if wants_relation_move(seed, tick, faction, draw, row.weights) {
                commands.push((faction, draw, Choice::Relation(rival)));
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
    }
}
