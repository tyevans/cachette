//! The campaign register: what each faction marches on, and with how many.
//!
//! # Why this exists
//!
//! A faction at war does nothing about it until a campaign exists. A campaign
//! is one row of (faction, objective, cohort). Raising one is two verbs a
//! caller already has: the set form of the type verb, to the soldier row, and
//! the send verb, to the objective tile. The controller raises a campaign
//! through the same core function a Python caller uses, so the engine holds
//! one rule set.[^1]
//!
//! # What it holds
//!
//! A bounded register of rows, one block of rows for each faction. The block
//! size is a balance value.[^2] A faction holds at most one live campaign. A
//! closed row stays in the register until the next raise of the same faction
//! reuses it, so a reader sees the last outcome.
//!
//! The register is simulated state. A later frame reads it: the stage closes
//! a campaign by comparing the holder of the objective tile against the holder
//! it had at the raise, and the raise refuses while a campaign is live. It
//! therefore enters the state hash.[^3]
//!
//! The log of one tick is not state. The step empties it before the controller
//! runs, and the census and the demonstration read it afterwards.
//!
//! # Determinism
//!
//! The controller decides whether to raise a campaign by one keyed draw for
//! each faction on each tick, at the index past the relation draw.[^4] The
//! objective is chosen with no draw. The cohort is the lowest identities among
//! the idle units, so two runs that hold one arena choose one cohort.
//!
//! No item in this module uses a floating-point type.[^5]
//!
//! # References
//!
//! [^1]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decision D2. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
//! [^2]: Balance register, the campaign register size and the cohort size. `docs/reference/balance.md`
//! [^3]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
//! [^4]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
//! [^5]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`

use bytemuck::{Pod, Zeroable};

use crate::controller::{FactionWeights, WEIGHT_HIGH};
use crate::hash::StateHash;
use crate::rng;
use crate::types::{FactionId, Tick, TileIdx};

/// How many campaign rows each faction holds, when nobody has set another
/// size.
///
/// **This is a provisional value and not a measured one.** The balance
/// register holds the row, marks it unset, and records how this value was
/// chosen.[^1]
///
/// # References
///
/// [^1]: Balance register, the campaign register size. `docs/reference/balance.md`
pub const REGISTER_SIZE_DEFAULT: u32 = 2;

/// How many units a campaign takes, when nobody has set another size.
///
/// **This is a provisional value and not a measured one.** The rules of the
/// downstream game are not written down, so a blocker governs it.[^1] The
/// balance register holds the row and records how this value was chosen.[^2]
///
/// # References
///
/// [^1]: Blockers register, BLK-050. `docs/BLOCKERS.md`
/// [^2]: Balance register, the campaign cohort size. `docs/reference/balance.md`
pub const COHORT_SIZE_DEFAULT: u32 = 4;

/// The objective kind: take a settlement tile of a faction at war.
pub const OBJECTIVE_TAKE_SITE: u8 = 0;

/// The objective kind: relieve an own settlement whose ground a faction at
/// war holds.
pub const OBJECTIVE_RELIEVE_SITE: u8 = 1;

/// The objective kind: wear an upgrade of a faction at war.
///
/// **Nothing raises this kind yet.** It waits for the upgrade pass to give an
/// upgrade a condition that wears. It is declared here so that the number is
/// allocated and a reader knows what the column can hold.
pub const OBJECTIVE_WEAR_UPGRADE: u8 = 2;

/// The row state: the row holds no campaign.
pub const STATE_EMPTY: u8 = 0;

/// The row state: the campaign is live.
pub const STATE_LIVE: u8 = 1;

/// The row state: the objective tile passed to the campaigner.
pub const STATE_WON: u8 = 2;

/// The row state: every unit of the cohort fell.
pub const STATE_LOST: u8 = 3;

/// The row state: the objective tile changed holder, and not to the
/// campaigner.
pub const STATE_ENDED: u8 = 4;

/// The row state: the campaign passed its deadline and reached nothing.
pub const STATE_EXPIRED: u8 = 5;

/// How many ticks a campaign runs before it expires, when nobody has set
/// another deadline.
///
/// **A campaign that reaches nothing must close, because a faction with a
/// live campaign raises no other one.** A campaign with no deadline therefore
/// takes the whole run, and a faction raises about one campaign for the whole
/// run.[^1]
///
/// **The deadline governs the stuck campaign alone.** A campaign that takes
/// its objective closes on the tick the holder changes, and a campaign whose
/// cohort falls closes on the tick the last unit dies. Neither waits for the
/// deadline. What the deadline ends is a cohort that is walking and arriving
/// nowhere.
///
/// **This is a provisional value and not a measured one.** It is 500 ticks.
/// The movement pass admits one tile step for each unit on each tick, and the
/// greatest hex distance across the balance harness world at extent 256 is
/// below 256 tiles, so 500 ticks is about twice the worst crossing of the
/// whole world. A cohort that has not reached its objective in twice that
/// time is stuck and not slow. At 500 ticks the tick limit of the harness
/// leaves room for about forty raises for each faction, where 2000 left room
/// for ten.[^2] [^3]
///
/// # References
///
/// [^1]: Findings register, FND-542. `docs/FINDINGS.md`
/// [^2]: Blockers register, BLK-050. `docs/BLOCKERS.md`
/// [^3]: Balance register, the campaign deadline. `docs/reference/balance.md`
pub const DEADLINE_DEFAULT: Tick = Tick(500);

/// The holder column value that names no faction.
pub const NO_HOLDER: u16 = u16::MAX;

/// One row of the campaign register.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Pod, Zeroable)]
pub struct CampaignRow {
    /// The tick the campaign was raised on.
    pub raised_at: Tick,
    /// The objective tile.
    pub objective_tile: u32,
    /// How many units the raise took.
    pub cohort_size: u32,
    /// The faction that raised it.
    pub faction: FactionId,
    /// The faction that held the objective tile at the raise, or `NO_HOLDER`.
    pub holder_at_raise: u16,
    /// The objective kind, as the `OBJECTIVE_` constants number it.
    pub objective_kind: u8,
    /// The state, as the `STATE_` constants number it.
    pub state: u8,
    /// Declared padding, always zero.
    pub padding: [u8; 2],
}

impl CampaignRow {
    /// Returns whether the row holds a live campaign.
    #[must_use]
    pub const fn is_live(&self) -> bool {
        self.state == STATE_LIVE
    }

    /// Returns whether the row holds a campaign, live or closed.
    #[must_use]
    pub const fn is_set(&self) -> bool {
        self.state != STATE_EMPTY
    }
}

/// The event kind: a campaign was raised.
pub const EVENT_RAISED: u8 = 0;

/// The event kind: a campaign took its objective.
pub const EVENT_WON: u8 = 1;

/// The event kind: every unit of the cohort fell.
pub const EVENT_LOST: u8 = 2;

/// The event kind: the objective changed holder, and not to the campaigner.
pub const EVENT_ENDED: u8 = 3;

/// The event kind: the campaign passed its deadline and reached nothing.
pub const EVENT_EXPIRED: u8 = 4;

/// One thing that happened to a campaign on the last tick.
///
/// The log is plain data with declared padding and no boolean.[^1] It covers
/// one tick, and the step empties it before the controller runs.
///
/// # References
///
/// [^1]: ADR-0006, an event is plain data and applying it is pure, decision D1. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Pod, Zeroable)]
pub struct CampaignEvent {
    /// The tick it happened on.
    pub tick: Tick,
    /// The objective tile.
    pub objective_tile: u32,
    /// How many units the cohort held at the raise.
    pub cohort_size: u32,
    /// The faction of the campaign.
    pub faction: FactionId,
    /// The event kind, as the `EVENT_` constants number it.
    pub kind: u8,
    /// The objective kind, as the `OBJECTIVE_` constants number it.
    pub objective_kind: u8,
    /// Declared padding, always zero.
    pub padding: [u8; 4],
}

/// Decides whether a faction raises a campaign this tick.
///
/// **This draws exactly once.** The key is the controller system, the tick,
/// the faction and the draw index, and the index is the one past the relation
/// draw so it collides with no other.[^1] The war weight biases the draw: the
/// answer is yes with probability `war / (WEIGHT_HIGH + war)`.
///
/// # References
///
/// [^1]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
#[must_use]
pub fn wants_campaign(
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

/// Picks the objective among candidate sites: the nearest by distance, and a
/// tie goes to the lowest slot.
///
/// The candidates are `(distance, slot, tile)`. The scan replaces the leader
/// only on a strictly lower key, so the first candidate at the lowest key
/// keeps it, and the order of the candidates does not reach the answer when
/// the keys are unique.[^1]
///
/// # References
///
/// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
#[must_use]
pub fn nearest_site(candidates: impl Iterator<Item = (u32, u32, TileIdx)>) -> Option<TileIdx> {
    let mut leader: Option<(u32, u32, TileIdx)> = None;
    for candidate in candidates {
        match leader {
            Some((distance, slot, _)) if (candidate.0, candidate.1) >= (distance, slot) => {}
            _ => leader = Some(candidate),
        }
    }
    leader.map(|(_, _, tile)| tile)
}

/// The campaign register the world holds.
#[derive(Clone, Debug)]
pub struct CampaignRegister {
    rows: Vec<CampaignRow>,
    register_size: u32,
    cohort_size: u32,
    /// How many ticks a campaign runs before it expires. Zero means never.
    ///
    /// The close pass reads this on every step, so it enters the state hash
    /// beside the rows it closes.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
    deadline: Tick,
    log: Vec<CampaignEvent>,
}

impl CampaignRegister {
    /// Builds an empty register for a world.
    #[must_use]
    pub fn new(faction_count: u16) -> Self {
        let factions = usize::from(faction_count.max(1));
        Self {
            rows: vec![CampaignRow::default(); factions * REGISTER_SIZE_DEFAULT as usize],
            register_size: REGISTER_SIZE_DEFAULT,
            cohort_size: COHORT_SIZE_DEFAULT,
            deadline: DEADLINE_DEFAULT,
            log: Vec::new(),
        }
    }

    /// Returns how many rows each faction holds.
    #[must_use]
    pub const fn register_size(&self) -> u32 {
        self.register_size
    }

    /// Returns how many units a raise takes.
    #[must_use]
    pub const fn cohort_size(&self) -> u32 {
        self.cohort_size
    }

    /// Sets how many units a raise takes.
    /// Returns how many ticks a campaign runs before it expires.
    #[must_use]
    pub const fn deadline(&self) -> Tick {
        self.deadline
    }

    /// Sets how many ticks a campaign runs before it expires.
    ///
    /// A deadline of zero means that no campaign expires.
    pub const fn set_deadline(&mut self, deadline: Tick) {
        self.deadline = deadline;
    }

    pub const fn set_cohort_size(&mut self, cohort_size: u32) {
        self.cohort_size = cohort_size;
    }

    /// Returns the rows of one faction, in slot order. Empty when the world
    /// has no such faction.
    #[must_use]
    pub fn rows_of(&self, faction: FactionId) -> &[CampaignRow] {
        let size = self.register_size as usize;
        let start = usize::from(faction.0) * size;
        self.rows.get(start..start + size).unwrap_or(&[])
    }

    /// Returns every row, in faction and then slot order.
    #[must_use]
    pub fn rows(&self) -> &[CampaignRow] {
        &self.rows
    }

    /// Returns the live campaign of a faction, or `None`.
    #[must_use]
    pub fn live(&self, faction: FactionId) -> Option<CampaignRow> {
        self.rows_of(faction)
            .iter()
            .copied()
            .find(CampaignRow::is_live)
    }

    /// Writes a live row for a faction into the lowest free slot.
    ///
    /// A free slot is one that holds no live campaign. The lowest such slot
    /// is taken, so the row a closed campaign left is reused before an empty
    /// one only when it comes first.
    ///
    /// Returns `false` and writes nothing when the faction holds a live
    /// campaign or the world has no such faction.
    pub fn open(&mut self, row: CampaignRow) -> bool {
        let size = self.register_size as usize;
        let start = usize::from(row.faction.0) * size;
        let Some(block) = self.rows.get_mut(start..start + size) else {
            return false;
        };
        if block.iter().any(CampaignRow::is_live) {
            return false;
        }
        let Some(slot) = block.iter_mut().find(|slot| !slot.is_live()) else {
            return false;
        };
        *slot = CampaignRow {
            state: STATE_LIVE,
            padding: [0; 2],
            ..row
        };
        true
    }

    /// Closes the live campaign of a faction with a state, and returns the
    /// row as it was.
    ///
    /// Returns `None` and changes nothing when the faction holds no live
    /// campaign.
    pub fn close(&mut self, faction: FactionId, state: u8) -> Option<CampaignRow> {
        let size = self.register_size as usize;
        let start = usize::from(faction.0) * size;
        let block = self.rows.get_mut(start..start + size)?;
        let slot = block.iter_mut().find(|slot| slot.is_live())?;
        let before = *slot;
        slot.state = state;
        Some(before)
    }

    /// Returns the events of the last tick, in the order they happened.
    #[must_use]
    pub fn log(&self) -> &[CampaignEvent] {
        &self.log
    }

    /// Returns the log as bytes, for the byte comparison the thread-count
    /// test makes.
    #[must_use]
    pub fn log_bytes(&self) -> &[u8] {
        bytemuck::cast_slice(&self.log)
    }

    /// Empties the log of the last tick.
    pub fn clear_log(&mut self) {
        self.log.clear();
    }

    /// Records one event.
    pub fn push(&mut self, event: CampaignEvent) {
        self.log.push(event);
    }

    /// Returns how many events of one kind the last tick logged.
    #[must_use]
    pub fn count(&self, kind: u8) -> i64 {
        self.log.iter().filter(|event| event.kind == kind).count() as i64
    }

    /// Folds every value that a later frame reads into the hash.
    ///
    /// The rows and the two parameters enter. The log does not, because it
    /// is a log of one tick and the next step empties it.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
    #[must_use]
    pub fn hash_into(&self, hash: StateHash) -> StateHash {
        hash.write_u64(self.rows.len() as u64)
            .write(bytemuck::cast_slice(&self.rows))
            .write_u64(u64::from(self.register_size))
            .write_u64(u64::from(self.cohort_size))
            .write_u64(self.deadline.0)
    }
}
