//! The world, and the frame step.
//!
//! The world holds one tile field, one event type, and two systems. The
//! tile system is a stub, and it exists so that the determinism harnesses
//! have a subject before the first solver is written.[^1] The movement
//! system gives each soldier one neighbour tile each frame. Replace the body
//! of a system. Do not replace the shape of the step.
//!
//! The step shows three rules that every later system must follow.
//!
//! The step writes each parallel result to an indexed output slot. It never
//! uses thread completion order and it never uses work-stealing order.[^2]
//! The target has a weak memory model, so an atomic costs a real barrier
//! where a strong model would absorb it. Disjoint outputs are therefore a
//! requirement and not a preference.[^3]
//!
//! The step draws every random value from a counter, keyed on the tuple of
//! system, frame, entity and draw index.[^4]
//!
//! The step routes all arithmetic through the arithmetic module.[^5]
//!
//! # What this file holds
//!
//! This file holds the world type, the errors of its submodules, and the
//! re-exports that keep every public name at the path it had before. Nothing
//! else lives here. The field order of the world type is load-bearing, so the
//! declaration stays whole and in one place.
//!
//! The methods of the world type sit in the sibling modules, one module for
//! each subject. Rust admits an implementation block for one type in several
//! modules of one crate, so the grouping costs no layout and no indirection.
//! Each sibling module states in its own header what it holds.
//!
//! # References
//!
//! [^1]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
//! [^2]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
//! [^3]: ADR-0009, parallel stages write disjoint outputs, because the memory model is weak. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
//! [^4]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
//! [^5]: ADR-0002, simulated and aggregated state holds no floating point number, decision D2. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`

use self::census::CensusTotals;
use crate::balance::Balance;
use crate::bridge::{BlockLayout, UnitTileBridge};
use crate::campaign::CampaignRegister;
use crate::character::CharacterArena;
use crate::choose::{ChoiceSchedule, NeedBuckets, WeightProfile};
use crate::climate::ClimateField;
use crate::cohort::{CohortTable, DeathPlane, DrawLedger, NeedRule, SiteRationed, UnitStarved};
use crate::contest::{Grievance, UnitFell};
use crate::controller::Controller;
use crate::conversion::{Convert, UnitConverted};
use crate::event::{
    FactionEliminated, ResourceTaken, SettlementFounded, SiteTaken, TileChanged, UpgradeCollapsed,
    UpgradeFinished,
};
use crate::event_memory::EventMemory;
use crate::fire::{FireEnded, FireField, FireStarted, UnitBurned};
use crate::hex::Grid;
use crate::holding::Holding;
use crate::influence::InfluenceField;
use crate::luxury::{LuxuryField, VarietyLevel};
use crate::observation::Observation;
use crate::padded::PaddedLattice;
use crate::plan::PlanRegister;
use crate::position::PositionTable;
use crate::presence::PresenceRelation;
use crate::production::{BuildCostTable, QueueTable};
use crate::promotion::UnitPromoted;
use crate::pyramid::{ApproachField, ExitField, Pyramid, ReturnField, SeededField};
use crate::rates::{RateLedger, RateSchedule, RateTable, SiteShortfall};
use crate::relation::RelationMatrix;
use crate::resource::{Amount, DepletionLedger, ResourceField, RESOURCE_KIND_COUNT};
use crate::site::{SettlementArena, SiegeRules, COMMODITY_COUNT};
use crate::soldier::SoldierArena;
use crate::terrain::Terrain;
use crate::tile_value::TileValues;
use crate::trade::{MarketTable, TradeSpoken, TradeTable};
use crate::types::{Accum, Fix32, Tick, TileIdx};
use crate::unit_type::UnitTypeTable;
use crate::upgrade::{UpgradeMap, UpgradeTable};
use crate::weather::{CellGround, WeatherField};

mod actions;
mod admission;
mod campaign;
mod carry;
mod census;
mod character;
mod choose;
mod cohorts;
mod config;
mod contest;
mod controller;
mod controller_targets;
mod conversion;
#[cfg(test)]
mod destination_guard;
mod errors;
mod event_memory;
mod fire;
mod founding;
mod gather;
mod growth;
mod hash;
mod holding;
mod households;
mod invariants;
mod luxuries;
mod movement;
mod observation;
mod plans;
mod positions;
mod pyramid;
mod queue;
mod rates;
mod relations;
mod resources;
mod seeding;
mod sites;
mod step;
mod tiles;
mod trade;
mod unit_types;
mod units;
mod upgrades;
mod victory;
mod weather;

pub use self::carry::CARRY_MARK_DEFAULT;
pub use self::census::{CensusBasis, CensusRow, SUBSYSTEM_CENSUS};
pub use self::config::WorldConfig;
pub use self::errors::{
    CampaignError, ConvertError, IdentityError, MoveRelationError, RazeError, SeedError, SendError,
    StepError, WorldError,
};
pub use self::founding::FOUNDING_GROUP_DEFAULT;
pub use self::luxuries::LUXURY_DEPOSITS_DEFAULT;
pub use self::sites::STOCK_CEILING_OF_ONE_SETTLEMENT;
pub use self::victory::Standing;

/// A simulated world.
///
/// The world holds no global mutable state, so one process may hold many
/// worlds and step them in parallel.[^1]
///
/// # References
///
/// [^1]: ADR-0047, many worlds live in one interpreter. `docs/adrs/REGISTRY.md`
#[derive(Clone, Debug)]
pub struct World {
    config: WorldConfig,
    grid: Grid,
    tick: Tick,
    /// The tile stub value of every tile.
    ///
    /// The field generates the value of a tile from the seed and stores only
    /// what the frames changed, so building a world visits no tile and
    /// allocates nothing that grows with the tile count.[^1] [^2]
    ///
    /// # References
    ///
    /// [^1]: PRD-0003, a developer sees a world worth looking at, what it costs at the target scale. `docs/product/accepted/prd-0003-a-developer-sees-a-world-worth-looking-at.md`
    /// [^2]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D4. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
    values: TileValues,
    log: Vec<TileChanged>,
    /// The events of the last step, from the gather resolve.
    gather_log: Vec<ResourceTaken>,
    soldiers: SoldierArena,
    settlements: SettlementArena,
    characters: CharacterArena,
    bridge: UnitTileBridge,
    /// Which factions stand on the ground of which other factions.
    ///
    /// The relation is derived at the end of a step and never stored as a
    /// fact.[^3]
    ///
    /// [^3]: ADR-0111, the presence relation is derived at the end of the step and never stored as a fact, decision D1. `docs/adrs/draft/adr-0111-the-presence-relation-is-derived-at-the-end-of-the-step.md`
    presence: PresenceRelation,
    /// What each faction sees now, and what each faction has ever seen.
    ///
    /// The layer of what a faction sees now is derived, and the step rebuilds
    /// it. The layer of what a faction has ever seen is state, and the step
    /// carries it forward.[^4]
    ///
    /// [^4]: ADR-0059, fog storage grows with observed area, not with world area, decision D3. `docs/adrs/accepted/adr-0059-fog-storage-grows-with-observed-area.md`
    observation: Observation,
    terrain: Terrain,
    /// The stock that each tile started with. It stores nothing.
    resources: ResourceField,
    /// What has been taken from each tile that somebody gathered from.
    depletion: DepletionLedger,
    /// What left the world in the hands of a dead unit.
    ///
    /// A unit that dies takes its load out of the world. Conservation must
    /// still balance, so the world records where the load went rather than
    /// letting it disappear.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D5. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
    departed: [u64; RESOURCE_KIND_COUNT],
    /// What every delivery has moved out of a carry and into a store, for
    /// each kind.
    ///
    /// A delivery takes a quantity out of the account that the conservation
    /// check balances against the carries, and puts it into the account that
    /// the store check balances against the stores. This total is the term
    /// that links the two, and without it the first check fails on the frame
    /// of the first delivery.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D5. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
    delivered: [u64; RESOURCE_KIND_COUNT],
    /// The upgrade that each improved tile carries.
    ///
    /// The map holds one entry for each tile that somebody built on, and it
    /// holds nothing else. A world in which nobody built holds no entry, so
    /// the memory cost follows the building and not the size of the
    /// world.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0090, a tile upgrade is stored sparsely, as the difference from the generated world, decision D1. `docs/adrs/draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md`
    upgrades: UpgradeMap,
    /// Level 1 of the pyramid, derived from level 0 at the barrier.
    pyramid: Pyramid,
    /// The arena revision that the last level 1 rebuild read.
    ///
    /// **Level 1 states its own freshness here.** A verb that a caller runs
    /// between two steps restores the derived unit structure and rebuilds no
    /// level, so the freshness of that structure says nothing about this
    /// level. A check that read one to answer for the other stated one fact
    /// in two places.[^1] [^2]
    ///
    /// It holds nothing before the first rebuild.
    ///
    /// # References
    ///
    /// [^1]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
    /// [^2]: Findings register, FND-647. `docs/FINDINGS.md`
    level_1_arena: Option<u64>,
    /// The direction that each level 1 cell holds, for each option.
    ///
    /// The array is what movement steers by. It is a projection of level 1,
    /// derived again at every rebuild of it, and it sits beside the summaries
    /// rather than inside them, because two directions do not add.[^1] [^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, decisions D2 and D3. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
    /// [^2]: ADR-0024, every summary field is declared extensive or intensive, decision D2. `docs/adrs/accepted/adr-0024-every-summary-field-is-declared-extensive-or-intensive.md`
    exits: ExitField,
    /// The direction of the nearest site of a faction, for each level 1 cell.
    ///
    /// It steers a unit that carries a load home. The field is derived at
    /// every rebuild of level 1, beside the exit field.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0110, a unit returns by climbing a reach field seeded at every site of its faction, decision D1. `docs/adrs/draft/adr-0110-a-unit-returns-by-climbing-a-reach-field.md`
    returns: ReturnField,
    /// The direction of the nearest site tile of a faction, for each tile of
    /// a seeded block and each faction plane.
    ///
    /// **The return field above steers a laden unit to the cell that holds a
    /// site, and no further.** A delivery reads the tile the unit stands on,
    /// so a carrier that reached the cell has delivered nothing. This field
    /// resolves that last cell at the pitch of one tile.[^1] [^2]
    ///
    /// It is the same mechanism the destination planes use, keyed on the
    /// faction instead of the destination. It is seeded from the same set as
    /// the return field, so it answers wherever the return field led.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0110, a unit returns by climbing a reach field seeded at every site of its faction, decision D1. `docs/adrs/draft/adr-0110-a-unit-returns-by-climbing-a-reach-field.md`
    /// [^2]: Findings register, FND-315. `docs/FINDINGS.md`
    home_approaches: ApproachField,
    /// The direction of the nearest tile that holds stock, for each tile of a
    /// seeded block and each resource kind.
    ///
    /// **The exit field answers at the pitch of a level 1 cell, and the stock
    /// of a tile is a level 0 property.** A unit that stands on barren ground
    /// inside the cell with the most food reads that its own cell is the best
    /// one, and nothing tells it to step two tiles sideways. It then strips
    /// the ground it stands on and stands there.[^1]
    ///
    /// This field resolves the cell at the pitch of one tile. It is the same
    /// mechanism the home approach above uses, keyed on the resource kind
    /// instead of the faction.[^2]
    ///
    /// **The engine seeds it only over the blocks that a gatherer stands
    /// in.** The set of gatherers is the whole set the field serves, so the
    /// derivation follows that set and never the world.[^3]
    ///
    /// # References
    ///
    /// [^1]: Findings register, FND-589. `docs/FINDINGS.md`
    /// [^2]: Findings register, FND-315. `docs/FINDINGS.md`
    /// [^3]: ADR-0096, cost follows the lattice, not the population, and a unit is a reader, decision D1. `docs/adrs/draft/adr-0096-cost-follows-the-lattice-not-the-population.md`
    stock_approaches: ApproachField,
    /// The direction of the nearest tile of a named destination, for each
    /// level 1 cell and each destination plane.
    ///
    /// It steers a unit that the control plane sent somewhere. The field is
    /// derived at every rebuild of level 1, beside the exit field and the
    /// return field, and it is derived again when the seeds change.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0125, the control plane names the seed set of a destination field, decision D1. `docs/adrs/draft/adr-0125-the-control-plane-names-the-seed-set-of-a-destination-field.md`
    destinations: SeededField,
    /// The seed tiles of each destination plane, in ascending order.
    ///
    /// **The control plane names these, and nothing else writes them.** The
    /// entry of one plane holds each tile once and in ascending order, so the
    /// derivation reads one set whatever order the caller named the tiles
    /// in.[^1]
    ///
    /// **The set holds tiles, and the cell of each is derived from it.** The
    /// coarse field is at block pitch and it reads the cells. The approach
    /// field is at tile pitch and it reads the tiles. Storing the cell beside
    /// the tile would be one fact in two places, and the two would then
    /// disagree with nothing to fail.[^2] [^3]
    ///
    /// # References
    ///
    /// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    /// [^2]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
    /// [^3]: Findings register, FND-315. `docs/FINDINGS.md`
    destination_seeds: Vec<Vec<TileIdx>>,
    /// The direction of the nearest seed tile, for each tile of a seeded
    /// block and each destination plane.
    ///
    /// **The coarse field steers a unit to the cell that holds its target,
    /// and no further.** This one resolves that last cell at the pitch of one
    /// tile, so a unit sent at a tile arrives at the tile.[^1]
    ///
    /// It is derived beside the coarse field, from the same seed set.[^2]
    ///
    /// # References
    ///
    /// [^1]: Findings register, FND-315. `docs/FINDINGS.md`
    /// [^2]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D1. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
    approaches: ApproachField,
    /// Whether the reach of each destination plane spreads through open
    /// water.
    ///
    /// The send verb writes this, and it writes what the set it was given
    /// says: a plane conducts through water when every unit sent to it
    /// crosses water. A plane that carries one unit the water refuses does
    /// not, because the field would then steer that unit at a coast.[^1]
    ///
    /// The entry is simulated state. A later frame derives the field from it,
    /// so two worlds that hold the same seeds and different conduction must
    /// diverge.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, decision D5. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
    /// [^2]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D1. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
    destination_crossings: Vec<u8>,
    /// Whether the step derives the destination field before it returns.
    ///
    /// **The field is derived once for each frame, after the last verb that
    /// changes its seeds.** The send verb ends by deriving the field, because
    /// a caller may read a direction between two steps and a derived value
    /// that one path leaves stale is a confident wrong answer.[^1] That
    /// reason asks for one derivation and not one for each send. The
    /// controller sends several times in one frame, and the barrier derives
    /// the field before it, so a frame that took three orders derived the
    /// whole field four times and read three of them never.
    ///
    /// The flag says that the step will derive the field before it returns.
    /// It states a fact about the frame and not about the caller: the verb
    /// asks nothing about who called it, and every caller reaches the same
    /// verb.[^2] The step sets the flag around the controller alone, so a
    /// caller outside a step always finds the field derived.
    ///
    /// **Nothing between the barrier and the end of the step reads the
    /// field.** The movement pass and the release of sent units are the two
    /// readers, and both run near the start of a frame.[^3]
    ///
    /// # References
    ///
    /// [^1]: Findings register, FND-029. `docs/FINDINGS.md`
    /// [^2]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decision D2. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
    /// [^3]: Findings register, FND-664. `docs/FINDINGS.md`
    destinations_deferred: bool,
    /// Whether a seed set or a crossing changed since the last derivation of
    /// the destination field.
    ///
    /// **The destination field is a pure function of three things.** It reads
    /// the seed set of each plane, the crossing of each plane, and the
    /// ground. The ground is generated from the world seed and it never
    /// changes, and the relaxation reads the level 1 cell only for the open
    /// tile count, which the ground alone decides.[^1] [^2] The field
    /// therefore holds the answer a fresh derivation gives for as long as no
    /// send changes a seed set.
    ///
    /// The send verb writes this flag, and it writes it only when the set it
    /// stores differs from the set the plane held. The controller re-sends
    /// the same objective on most frames, so most frames change nothing and
    /// pay for no derivation. Skipping a derivation whose answer is already
    /// held is an incremental update, and the record that permits one asks
    /// that it give the answer a full rebuild would give.[^3]
    ///
    /// **The public rebuild derives the field whatever this flag says.** A
    /// test compares the field the step leaves against a fresh derivation,
    /// and a rebuild that read the flag would compare the field against
    /// itself.[^4]
    ///
    /// # References
    ///
    /// [^1]: ADR-0068, terrain is generated from the seed and is never stored as a map, decision D1. `docs/adrs/accepted/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
    /// [^2]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, decision D5. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
    /// [^3]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D2. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
    /// [^4]: Testing rules, section 1. `.agents/rules/testing.md`
    destination_seeds_changed: bool,
    /// The load at which a unit counts as laden.
    ///
    /// A laden unit takes the option that carries its load home, and a unit
    /// below the mark does not.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0109, the choice key holds a bounded class of the unit's own state, decision D3. `docs/adrs/draft/adr-0109-the-choice-key-holds-a-bounded-class-of-the-unit-state.md`
    carry_mark: Amount,
    /// Who holds each tile, and what each faction holds.
    ///
    /// The holder column is level 0 and it is the truth. It is the one value
    /// that names the faction which owns a tile, and the tile event carries
    /// it.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D2. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
    holding: Holding,
    /// What each faction reaches, over the level 1 cell lattice.
    ///
    /// The field is a plane over the level 1 cells and it is not a summary:
    /// a cell of it is the result of a relaxation that reads the neighbours
    /// of the cell, and it carries what the last solve left. The record
    /// states the boundary that draws against the record which owns level 0,
    /// and an open choice asks a reviewer to settle it.[^1] [^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0087, an influence solve runs a fixed iteration count over the whole plane. `docs/adrs/draft/adr-0087-an-influence-solve-runs-a-fixed-iteration-count.md`
    /// [^2]: Decisions register, DEC-067. `docs/DECISIONS.md`
    influence: InfluenceField,
    /// When the site rates apply.
    schedule: RateSchedule,
    /// The base production rate and the upkeep rate of each site.
    ///
    /// This is stored state and it enters the state hash. The founding writes
    /// the production column of a site once, from the ground the survey
    /// measured.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0062, production and upkeep are rates attached to a site, decision D1. `docs/adrs/accepted/adr-0062-production-and-upkeep-are-rates-attached-to-a-site.md`
    rates: RateTable,
    /// The effective rate that the last application read, as scratch.
    ///
    /// **This is derived, not stored.** The rate pass fills it again from the
    /// ground, the moisture, the upgrades and the residents on every
    /// application, so it holds no fact of its own and it stays out of the
    /// state hash. Its four inputs are stored and they enter the hash.[^1]
    ///
    /// The field exists so that the pass allocates once rather than on every
    /// application. Nothing outside the pass reads it.
    ///
    /// # References
    ///
    /// [^1]: ADR-0164, every stored value the step reads enters the state hash, decision D2. `docs/adrs/draft/adr-0164-every-stored-value-the-step-reads-enters-the-state-hash.md`
    effective_rates: RateTable,
    /// Every rate that has applied since the world was built.
    rate_ledger: RateLedger,
    /// The sites that could not pay at the last application.
    shortfall_log: Vec<SiteShortfall>,
    /// What a unit needs, and how fast it needs it.
    need_rule: NeedRule,
    /// The cohorts of every site, derived from the home column of the units.
    cohorts: CohortTable,
    /// Every draw that has run since the world was built.
    draw_ledger: DrawLedger,
    /// The sites that could not serve every cohort at the last draw.
    rationed_log: Vec<SiteRationed>,
    /// One bit for each unit that the last shortage ended.
    ///
    /// The plane is the batch of a structural change, and the scan of it
    /// applies the change after the barrier.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR Registry, row 0020. `docs/adrs/REGISTRY.md`
    death_plane: DeathPlane,
    /// The units that a shortage ended at the last scan, in slot order.
    starved_log: Vec<UnitStarved>,
    /// The shared table that a unit type indexes.
    ///
    /// **The table is data that the world is built with.** It holds no code,
    /// and a lookup in it is not a callback. A caller fills the rows it
    /// wants, and a world that filled none holds no contest at all.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0120, a unit carries a type, and the type is an index into a table the world is built with, decisions D1 and D2. `docs/adrs/draft/adr-0120-a-unit-carries-a-type-that-indexes-a-table.md`
    unit_types: UnitTypeTable,
    /// The table that a category and a level index.
    ///
    /// **The table is data that the world is built with.** A row is one
    /// category at one level. It names the ground it fits, the work it takes
    /// and the columns a pass reads. No pass branches on a category.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0151, an upgrade is a category with a ground fit and a level, decisions D1 and D4. `docs/adrs/accepted/adr-0151-an-upgrade-is-a-category-with-a-ground-fit-and-a-level.md`
    upgrade_table: UpgradeTable,
    /// The build queue of every site, indexed by the slot of the site.
    ///
    /// **A site holds a bounded, ordered queue of plain-data entries.** The
    /// order is the order the entries were queued, and nothing reorders
    /// them. The whole table enters the state hash.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0158, a site builds a typed unit from a bounded queue its store pays for, decision D1. `docs/adrs/accepted/adr-0158-a-site-builds-a-typed-unit-from-a-bounded-queue-its-store-pays-for.md`
    queues: QueueTable,
    /// The shared table that a unit type indexes for its build cost.
    ///
    /// **The table is data that the world is built with**, in the way the
    /// unit type table and the upgrade table are. The work that finishes an
    /// entry is a value of it, and it is never a constant of the kernel.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0158, a site builds a typed unit from a bounded queue its store pays for, decision D3. `docs/adrs/accepted/adr-0158-a-site-builds-a-typed-unit-from-a-bounded-queue-its-store-pays-for.md`
    build_costs: BuildCostTable,
    /// When the queue advance acts.
    ///
    /// The interval is a parameter of the world and never a constant of the
    /// stage.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0158, a site builds a typed unit from a bounded queue its store pays for, decision D5. `docs/adrs/accepted/adr-0158-a-site-builds-a-typed-unit-from-a-bounded-queue-its-store-pays-for.md`
    queue_schedule: RateSchedule,
    /// When the growth stage acts.
    ///
    /// The interval is a parameter of the world and never a constant of the
    /// stage.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0157, a site's free places are its built housing less the residents the engine counts, decision D4. `docs/adrs/accepted/adr-0157-a-sites-free-places-are-its-built-housing-less-the-residents-the-engine-counts.md`
    growth_schedule: RateSchedule,
    /// The housing that one person takes.
    ///
    /// The value is a parameter of the world. The balance register holds the
    /// row and this field states no value of its own.[^1]
    ///
    /// # References
    ///
    /// [^1]: Balance register, the population. `docs/reference/balance.md`
    housing_per_person: u32,
    /// The store that one birth costs.[^1]
    ///
    /// # References
    ///
    /// [^1]: Balance register, the population. `docs/reference/balance.md`
    food_per_birth: [Fix32; COMMODITY_COUNT],
    /// The chance that one proposal of one site becomes a birth.[^1]
    ///
    /// # References
    ///
    /// [^1]: Balance register, the population. `docs/reference/balance.md`
    birth_chance: Fix32,
    /// The housing that a founded site starts with.[^1]
    ///
    /// # References
    ///
    /// [^1]: Balance register, the population. `docs/reference/balance.md`
    founding_housing: u32,
    /// How many people the growth stage added on the last tick.
    ///
    /// The count is a census of one tick, and the growth stage clears it
    /// before it acts. A zero is therefore visible: a world that cannot grow
    /// and a world that chose not to both read zero, and the free places
    /// beside it say which.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0157, a site's free places are its built housing less the residents the engine counts, the consequences. `docs/adrs/accepted/adr-0157-a-sites-free-places-are-its-built-housing-less-the-residents-the-engine-counts.md`
    births: u32,
    /// What growth took out of the stores, for each commodity.
    ///
    /// The total is cumulative over the life of the world. It is the growth
    /// term of the conservation statement: what the stores held, plus what
    /// production put in, less what upkeep spent, less what the cohorts drew,
    /// less this, is what the stores hold.
    growth_ledger: [Accum; COMMODITY_COUNT],
    /// One bit for each unit that the last meeting ended.
    ///
    /// The plane is the batch of a structural change, in the way the plane of
    /// the shortage is, and the scan of it applies the change in ascending
    /// slot order.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    fell_plane: DeathPlane,
    /// The units that a meeting ended at the last resolution, in slot order.
    fell_log: Vec<UnitFell>,
    /// Which faction lost units to which at the last resolution, summed by
    /// pair. The relation pass reads it after the contest applies.
    grievances: Vec<Grievance>,
    /// What each faction feels toward each other faction.
    ///
    /// The matrix is simulated state and enters the hash. The contest, the
    /// conversion and the admission read it, and the settle path, the
    /// contest apply, the conversion apply, the drift and the verb write
    /// it.[^rel]
    ///
    /// [^rel]: ADR-0146, a faction relation is one signed integer per ordered pair, and a pass reads a threshold. `docs/adrs/accepted/adr-0146-a-faction-relation-is-one-signed-integer-per-ordered-pair-and-a-pass-reads-a-threshold.md`
    relations: RelationMatrix,
    /// The units that the conversion pass marked this frame, in slot order.
    ///
    /// The buffer is reused across frames, so the pass allocates once rather
    /// than once for each frame.
    convert_marks: Vec<Convert>,
    /// The units that changed faction this frame.
    converted_log: Vec<UnitConverted>,
    /// The promotions of the last frame that ran the pass.
    ///
    /// The log is cleared at the start of each pass, in the way every other
    /// per-frame log is, so a reader sees the promotions of one frame and
    /// never an accumulation.
    promoted_log: Vec<UnitPromoted>,
    /// The schedule that the promotion pass runs on.
    ///
    /// **The scan runs at a barrier of its own and not on every tick.** A
    /// pass over every unit on every frame spends the frame for an answer
    /// that changes rarely, and the eligibility of a unit is a level that
    /// only rises.[^1]
    ///
    /// The interval is a parameter of the world, so a caller changes how
    /// often the world looks for somebody to promote without touching the
    /// engine.
    ///
    /// # References
    ///
    /// [^1]: ADR-0104, a soldier is promoted from a level that never falls, decisions D2 and D5. `docs/adrs/draft/adr-0104-a-soldier-is-promoted-from-a-level-that-never-falls.md`
    character_schedule: RateSchedule,
    /// The most characters that one promotion pass may create.
    ///
    /// **This is the cut at the rank, and it is not the ceiling.** The
    /// character arena is built at the ceiling of its declared tier and
    /// refuses a create beyond it, so the population bound has one
    /// enforcement site whatever this says.[^1] This says how fast the world
    /// is willing to approach it.
    ///
    /// The default admits as many as the arena has room for, so a caller who
    /// sets nothing gets the ceiling and no second bound.
    ///
    /// # References
    ///
    /// [^1]: Blockers register, BLK-004. `docs/BLOCKERS.md`
    promotion_budget: u32,
    /// When each unit re-reads the world and chooses again.
    choice: ChoiceSchedule,
    /// How finely the choice tells two needs apart.
    ///
    /// **The width of the bucket is the mechanism of the decision and not a
    /// detail of it.** A need is a Q16.16 quantity, so unbucketed two units in
    /// one cell almost never share a need and the pass computes one answer for
    /// each unit.[^1] The width is a parameter of the world because no record
    /// sets it and no measurement chooses it, and a blocker governs every cost
    /// figure this project holds.[^2] [^3]
    ///
    /// # References
    ///
    /// [^1]: ADR-0098, the choice is decided for each cell and each bucket of need, decision D1. `docs/adrs/draft/adr-0098-the-choice-is-decided-for-each-cell-and-each-bucket-of-need.md`
    /// [^2]: Blockers register, BLK-007. `docs/BLOCKERS.md`
    /// [^3]: Decisions register, DEC-097. `docs/DECISIONS.md`
    buckets: NeedBuckets,
    /// The weight that a unit puts on each option of the choice.
    ///
    /// The profile is content, and it is a table of values. The engine reads
    /// it and never calls into it.[^1]
    ///
    /// The profile reaches the state hash. The pass reads it on every tick,
    /// so it is a value the world holds and not only an input to it. The
    /// intent column carries the outcome, and an outcome states the effect
    /// and never the cause: two worlds that hold the same intents and
    /// different weights hash the same and diverge at the next choice.[^2]
    /// [^3]
    ///
    /// # References
    ///
    /// [^1]: ADR-0007, content supplies a key vector, never a comparator, decision D3. `docs/adrs/accepted/adr-0007-content-supplies-a-key-vector-never-a-comparator.md`
    /// [^2]: ADR-0064, a unit chooses by scoring a small fixed option set, decision D2. `docs/adrs/accepted/adr-0064-a-unit-chooses-by-scoring-a-small-fixed-option-set.md`
    /// [^3]: Findings register, FND-537. `docs/FINDINGS.md`
    weights: WeightProfile,
    /// The positions that each site holds, and what each site wants.
    ///
    /// The table follows the slot column of the settlement arena. It is
    /// stored per site and never per tile.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0065, a group is a site membership, not a region, decision D1. `docs/adrs/draft/adr-0065-a-group-is-a-site-membership-not-a-region.md`
    positions: PositionTable,
    /// When the site positions are rebalanced.
    ///
    /// The interval is a parameter of the world. The pass that reads it
    /// holds no period of its own.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0065, a group is a site membership, not a region, decision D3. `docs/adrs/draft/adr-0065-a-group-is-a-site-membership-not-a-region.md`
    position_schedule: RateSchedule,
    /// What the live stores hold, by the account of this world.
    ///
    /// The store column states the same total a second time, and the
    /// conservation check is what fails when the two disagree.[^1] The
    /// world adjusts this account at each place where a store changes
    /// outside the rate pass: a write from the control plane, and the loss
    /// of a settlement.
    ///
    /// # References
    ///
    /// [^1]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
    /// The luxuries that each tile carries.
    ///
    /// The field holds one entry for each tile that carries a luxury, and it
    /// holds nothing else. A world in which nobody seeded a luxury holds no
    /// entry.[^1]
    ///
    /// The control plane seeds it once, and nothing writes it after that. It
    /// is simulated state all the same, because two worlds that carry
    /// different luxuries are different worlds, so it enters the state
    /// hash.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0090, a tile upgrade is stored sparsely, as the difference from the generated world, decision D1. `docs/adrs/draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md`
    /// [^2]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
    luxuries: LuxuryField,
    /// The luxuries of each level 1 cell.
    ///
    /// The level is derived from the field above, and level 0 is the truth.
    /// It sits beside the cell summaries rather than inside them, because a
    /// union of luxuries is idempotent and two idempotent values do not
    /// add.[^1] It does not enter the state hash, because it holds no fact of
    /// its own.
    ///
    /// The field never changes after the world is seeded, so the level is
    /// derived when the seed lands and never again.
    ///
    /// # References
    ///
    /// [^1]: ADR-0024, every summary field is declared extensive or intensive, decision D2. `docs/adrs/accepted/adr-0024-every-summary-field-is-declared-extensive-or-intensive.md`
    variety: VarietyLevel,
    /// Whether the world has taken a luxury seed.
    ///
    /// The flag is not a second copy of the field. A seed that places nothing
    /// leaves the field empty, and an unseeded world leaves it empty as well,
    /// so the field cannot answer the question on its own.[^1]
    ///
    /// The flag reaches the state hash. Two worlds that hold the same
    /// luxuries are not the same world when one took a seed and the other did
    /// not, because the next seed call is accepted by one and refused by the
    /// other.
    ///
    /// # References
    ///
    /// [^1]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
    luxuries_seeded: bool,
    store_account: [Accum; COMMODITY_COUNT],
    /// What each ordered pair of factions has agreed, and what it still owes.
    ///
    /// The plane holds one row for each ordered pair, and it holds nothing
    /// until somebody speaks. It never follows the population.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0126, a trade negotiation is engine state and the words are not, decision D1. `docs/adrs/draft/adr-0126-a-trade-negotiation-is-engine-state.md`
    trade: TradeTable,
    /// What the last step said about trade.
    trade_log: Vec<TradeSpoken>,
    /// The upgrades the wear pass removed, since the last step began.
    ///
    /// **A removed upgrade leaves nothing behind.** The entry is dropped and
    /// the tile returns to the world the generator made, so this log is the
    /// only record that anything stood there. The step clears it before any
    /// system runs.
    ///
    /// The log is written and never read by a pass, so it enters no state
    /// hash.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
    collapsed_log: Vec<UpgradeCollapsed>,
    /// The upgrade levels that finished, since the last step began.
    ///
    /// The step clears it before any system runs, and it enters no state
    /// hash.
    finished_log: Vec<UpgradeFinished>,
    /// The settlements that were founded, since the last step began.
    ///
    /// A settlement founded by a caller between two steps lands here and
    /// stays until the next step clears it. The log enters no state hash.
    founded_log: Vec<SettlementFounded>,
    /// The sites that changed hands or were razed, since the last step began.
    ///
    /// A raze by a caller between two steps lands here and stays until the
    /// next step clears it. The log enters no state hash, in the same way
    /// that every other log does not.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0164, every stored value the step reads enters the state hash. `docs/adrs/draft/adr-0164-every-stored-value-the-step-reads-enters-the-state-hash.md`
    taken_log: Vec<SiteTaken>,
    /// What a site resists, and what a raze costs over a capture.
    ///
    /// A site falls to work and never to a moment, and these are the two
    /// balance rows that price the work.[^1] [^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0180, a site changes hands or the taker destroys it, decisions D8 and D9. `docs/adrs/draft/adr-0180-a-site-changes-hands-or-the-taker-destroys-it.md`
    /// [^2]: Balance register, the siege. `docs/reference/balance.md`
    siege_rules: SiegeRules,
    /// The sites that changed hands over the run, and the sites that burned.
    ///
    /// **These two count the acts and never the sites that stand.** A run
    /// that keeps every city it takes and a run that burns every one differ
    /// here and nowhere else, because the site count reports the outcome of
    /// both. The census reports them, and no rule of the simulation reads
    /// them, so they enter no state hash.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0164, every stored value the step reads enters the state hash, decision D1. `docs/adrs/draft/adr-0164-every-stored-value-the-step-reads-enters-the-state-hash.md`
    sites_captured: i64,
    /// The sites that burned over the run. See the field above.
    sites_razed: i64,
    /// The site ticks that a siege pressed over the run.
    ///
    /// One site under siege for one tick adds one. A run in which no army
    /// ever holds a rival site reads zero here, and that is the reading
    /// that tells a keeping run from a run with no conquest in it.
    sieges_pressed: i64,
    /// The sieges that a relief ended before the site fell, over the run.
    ///
    /// A unit of the owning faction on the site tile ends a siege and takes
    /// its work away. A besieger that leaves ends nothing, so it adds
    /// nothing here.
    sieges_relieved: i64,
    /// The factions that left the game, since the last step began.
    ///
    /// The log enters no state hash. The column below is the stored fact,
    /// and the log is what a reader sees of it for one tick.
    eliminated_log: Vec<FactionEliminated>,
    /// One for a faction that has left the game, zero otherwise.
    ///
    /// **This is stored state and the step reads it, so it enters the state
    /// hash.**[^1] The elimination pass reads it to leave a faction that has
    /// already gone alone, and every game end reader reads it to refuse a
    /// faction that is out.
    ///
    /// The column is indexed by the faction identifier, and it holds one
    /// entry for each faction the world was built with. It is a one-byte
    /// integer and never a boolean, because it crosses into a hash.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0164, every stored value the step reads enters the state hash. `docs/adrs/draft/adr-0164-every-stored-value-the-step-reads-enters-the-state-hash.md`
    /// [^2]: ADR-0006, an event is plain data and applying it is pure, decision D1. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
    eliminated: Vec<u8>,
    /// What each faction offers and wants, one fixed block of rows for each.
    ///
    /// The table holds nothing until a faction advertises. It is simulated
    /// state and it enters the hash, because a controller of a later pass
    /// reads it inside the step.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0126, a trade negotiation is engine state and the words are not, decision D1. `docs/adrs/draft/adr-0126-a-trade-negotiation-is-engine-state.md`
    market: MarketTable,
    /// The most tiles one land consideration names.
    ///
    /// The bound is a balance value and the register calls it unset.[^1] The
    /// value here is a stand-in that a caller replaces.
    ///
    /// # References
    ///
    /// [^1]: Balance register, land list bound. `docs/reference/balance.md`
    land_list_bound: u32,
    /// The water in the air and on the ground, over the level 1 cells.
    ///
    /// The field is a plane over the level 1 cell lattice and it is not a
    /// summary: a cell of it holds what the last solve left there, and that
    /// value appears nowhere at level 0. It is simulated state, so it enters
    /// the state hash.[^1] [^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0140, weather is a field over the level 1 cell lattice, decision D1. `docs/adrs/draft/adr-0140-weather-is-a-field-over-the-level-1-cell-lattice.md`
    /// [^2]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
    weather: WeatherField,
    /// The block geometry of the weather lattice.
    ///
    /// **The weather has a pitch of its own, and it is a parameter of the
    /// world.** The layout says which weather cell covers a tile. It is the
    /// same layout as the level 1 layout when the world takes the level 1
    /// pitch, and a different one at every other pitch.[^1]
    ///
    /// # References
    ///
    /// [^1]: The weather scale. [`WeatherScale`]
    weather_layout: BlockLayout,
    /// The weather lattice, and the margin of cells around the world that the
    /// weather solve steps and no reader sees.
    ///
    /// **This is the one site that turns a cell of the world into a cell of
    /// the weather plane.** Every reader of the weather names a tile, the
    /// reader below turns that tile into a cell, and this adds the margin
    /// offset. A second site that added the offset could disagree with this
    /// one, so there is not one.[^1]
    ///
    /// # References
    ///
    /// [^1]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
    weather_lattice: PaddedLattice,
    /// The ground under each weather cell, in cell index order.
    ///
    /// **The weather reads the ground, and the ground does not change.** The
    /// three numbers it holds are the tiles the cell covers, the tiles of it
    /// that admit a unit, and the sum of their heights. Every one of them is
    /// a pure function of the world seed and the address, so the world folds
    /// this once and never rebuilds it.[^1]
    ///
    /// It is derived from level 0 and it is not simulated state, so it enters
    /// no state hash.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0068, terrain is generated from the seed and is never stored as a map. `docs/adrs/accepted/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
    /// [^2]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
    weather_ground: Vec<CellGround>,
    /// The climate that the weather left over each weather cell.
    ///
    /// **The field is quiet unless the caller asked for a spin.** A quiet
    /// field reads temperate at every address, so a world that asked for no
    /// spin generates the ground that the seed alone gives.[^1]
    ///
    /// The field is stored, and the terrain readers of this world read it, so
    /// it enters the state hash.[^2]
    ///
    /// # References
    ///
    /// [^1]: The climate field. [`ClimateField`]
    /// [^2]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
    climate: ClimateField,
    /// The tiles that burn, and the tiles that have burned.
    ///
    /// The field holds one entry for each burning tile and one for each tile
    /// that stopped burning. It holds nothing else, so a world that nobody
    /// set alight pays nothing for it, whatever the tile count.[^1]
    ///
    /// It is simulated state and it enters the state hash. Two worlds that
    /// hold the same ground and different fires are different worlds, because
    /// the next tick spreads from what this one left.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0096, cost follows the lattice, not the population, and a unit is a reader, decision D1. `docs/adrs/draft/adr-0096-cost-follows-the-lattice-not-the-population.md`
    /// [^2]: ADR-0164, every stored value the step reads enters the state hash, decision D1. `docs/adrs/draft/adr-0164-every-stored-value-the-step-reads-enters-the-state-hash.md`
    fire: FireField,
    /// The chance, out of the whole the fire module states, that lightning
    /// starts a fire on one tick.
    ///
    /// **The value is a parameter of the world and never a constant of the
    /// stage.** A world that states zero never catches by itself, and a
    /// caller that wants a world that burns on its own raises it.[^1]
    ///
    /// # References
    ///
    /// [^1]: Blockers register, BLK-007. `docs/BLOCKERS.md`
    lightning_chance: u64,
    /// The tiles that caught fire since the last step began.
    ///
    /// The step clears it before any system runs, in the way every other log
    /// works, so a tile a caller set alight between two steps survives to the
    /// next read. The log enters no state hash.
    fire_started_log: Vec<FireStarted>,
    /// The tiles that stopped burning since the last step began.
    fire_ended_log: Vec<FireEnded>,
    /// The units the fire ended since the last step began.
    burned_log: Vec<UnitBurned>,
    /// The mean standing water of the climate field, folded once.
    ///
    /// A terrain reader asks the climate over one address, and the climate of
    /// a cell is a share against the mean over the whole field. Folding the
    /// field for each tile would make a tile read cost the size of the
    /// lattice, so the world folds it once here.
    ///
    /// **This is derived, and the field is the declaration.** It is refreshed
    /// only when the field is built, and the field never changes after that.
    climate_reference: i64,
    /// The faction controller: one row for each faction, the two parameters
    /// the step reads on every tick, and the game end record.
    ///
    /// Every value in it is state that a later frame reads, so the whole of
    /// it enters the hash.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0148, a game end is recorded once and stops the controllers, decision D1. `docs/adrs/accepted/adr-0148-a-game-end-is-recorded-once-and-stops-the-controllers.md`
    controller: Controller,
    /// The campaign register: what each faction marches on, and with how
    /// many. A later frame reads it, so it enters the hash.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
    campaigns: CampaignRegister,
    /// The projects each faction has zoned.
    ///
    /// The plan is simulated state that the step reads, so it enters the
    /// whole-world hash.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0152, a faction plans its roads and zones with one solver, decision D1. `docs/adrs/accepted/adr-0152-a-faction-plans-its-roads-and-zones-with-one-solver.md`
    plan: PlanRegister,
    /// The win-path balance values: the value each game end reader compares,
    /// the rate that feeds one of them, and whether the readers run.
    ///
    /// **A caller sets these at run time, and each holds a constant as its
    /// default.** The step reads them, so they enter the state hash.[^1] [^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0175, a win threshold decides when a reader fires and never what the simulation does, decisions D1 and D2. `docs/adrs/draft/adr-0175-a-win-threshold-decides-when-a-reader-fires.md`
    /// [^2]: ADR-0164, every stored value the step reads enters the state hash, decision D1. `docs/adrs/draft/adr-0164-every-stored-value-the-step-reads-enters-the-state-hash.md`
    balance: Balance,
    /// What the run has produced, for the subsystems that keep a per-tick
    /// count.
    ///
    /// The controller, the campaign register, the relation matrix and the
    /// queue table each empty their counts when the next tick starts. The
    /// world folds each count into this total at the one site that empties
    /// it, so the census reads a total for the whole run and never a reading
    /// of one tick.[^1]
    ///
    /// **This is not simulated state and it does not enter the hash.** No
    /// frame reads it. It is a derived count of what already happened, in
    /// the way the per-tick counts it folds are.[^2]
    ///
    /// # References
    ///
    /// [^1]: Findings register, FND-498. `docs/FINDINGS.md`
    /// [^2]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
    census: CensusTotals,
    /// What has happened to each faction lately, and which faction did it.
    ///
    /// The history holds a decayed count of each kind of event for each
    /// faction, and a second count keyed on the ordered pair of the subject
    /// and the faction that caused the event. The step advances it once for
    /// each step, because every event log holds one step and no more: a
    /// reader that sampled the logs at its own pace would miss every step it
    /// did not sample.[^1]
    ///
    /// **This is not simulated state and it does not enter the hash.** No
    /// pass and no verb reads it. It is a derived count of what already
    /// happened, in the way the census totals beside it are.[^2] [^3]
    ///
    /// # References
    ///
    /// [^1]: The event history. [`EventMemory`]
    /// [^2]: ADR-0164, every stored value the step reads enters the state hash, decision D1. `docs/adrs/draft/adr-0164-every-stored-value-the-step-reads-enters-the-state-hash.md`
    /// [^3]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
    event_memory: EventMemory,
}

impl World {
    /// Returns the settings that built the world.
    #[must_use]
    pub const fn config(&self) -> WorldConfig {
        self.config
    }

    /// Returns the current tick.
    #[must_use]
    pub const fn tick(&self) -> Tick {
        self.tick
    }

    /// Returns how many factions this world holds.
    ///
    /// A faction identifier below this number names a faction. The ceiling is
    /// a property of the mask that holds a relation between factions.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D2. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
    #[must_use]
    pub const fn faction_count(&self) -> u16 {
        self.config.faction_count
    }

    /// Returns the events of the last step.
    #[must_use]
    pub fn event_log(&self) -> &[TileChanged] {
        &self.log
    }

    /// Returns the events of the last step as bytes.
    ///
    /// The thread-count equivalence test compares this slice byte for
    /// byte.[^1] The cast is safe because the event type is plain data.
    ///
    /// # References
    ///
    /// [^1]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
    #[must_use]
    pub fn event_log_bytes(&self) -> &[u8] {
        bytemuck::cast_slice(&self.log)
    }
}
