//! The constructors of the world, and the one call that seeds it.
//!
//! A constructor builds the fields, the arenas and the tables of a world from
//! the settings. The seed call founds one run for every faction and places
//! the luxuries. Both answer the same question: what does the world hold
//! before the first tick?

use super::carry::CARRY_MARK_DEFAULT;
use super::census::CensusTotals;
use super::config::WorldConfig;
use super::errors::{SeedError, WorldError};
use super::founding::FOUNDING_GROUP_DEFAULT;
use super::luxuries::LUXURY_DEPOSITS_DEFAULT;
use super::movement::Destinations;
use super::World;
use crate::balance::Balance;
use crate::bridge::{BlockLayout, UnitTileBridge, BLOCK_BITS_DEFAULT};
use crate::campaign::CampaignRegister;
use crate::character::CharacterArena;
use crate::choose::{ChoiceSchedule, NeedBuckets, WeightProfile};
use crate::climate::ClimateField;
use crate::cohort::{CohortTable, DeathPlane, DrawLedger, NeedRule};
use crate::controller::Controller;
use crate::event_memory::EventMemory;
use crate::fire::FireField;
use crate::founding::FoundingOutcome;
use crate::growth;
use crate::hex::Grid;
use crate::holding::Holding;
use crate::influence::InfluenceField;
use crate::luxury::{LuxuryError, LuxuryField, LuxuryId, VarietyLevel};
use crate::observation::Observation;
use crate::padded::PaddedLattice;
use crate::plan::{PlanRegister, PlanRules};
use crate::position::PositionTable;
use crate::presence::PresenceRelation;
use crate::production::{BuildCostTable, QueueTable, QUEUE_PERIOD_DEFAULT, QUEUE_PHASE_DEFAULT};
use crate::pyramid::{ApproachField, ExitField, Pyramid, ReturnField, SeededField};
use crate::rates::{RateLedger, RateSchedule, RateTable};
use crate::relation::RelationMatrix;
use crate::resource::{DepletionLedger, ResourceField, RESOURCE_KIND_COUNT};
use crate::rng;
use crate::site::{SettlementArena, SiegeRules, COMMODITY_COUNT};
use crate::soldier::SoldierArena;
use crate::terrain::Terrain;
use crate::tile_value::TileValues;
use crate::trade::{MarketTable, TradeTable, DEFAULT_BOARD_ROWS, DEFAULT_LAND_LIST_BOUND};
use crate::types::{Accum, Tick, TileIdx, FACTION_CEILING};
use crate::unit_type::DEFAULT_UNIT_TYPE_TABLE;
use crate::upgrade::{self, UpgradeMap};
use crate::weather::{ground_over_lattice, Latitudes, WeatherField, WeatherScale};

impl World {
    /// Builds a world from the settings.
    ///
    /// # Errors
    ///
    /// Returns an error when the configured extent does not describe a grid.
    pub fn new(config: WorldConfig) -> Result<Self, WorldError> {
        Self::with_weather_scale(config, WeatherScale::DEFAULT)
    }

    /// Builds a world from the settings, at a stated weather resolution.
    ///
    /// **The resolution of the weather is a parameter of the world.** The
    /// scale states the tiles along one side of a weather cell, and one of
    /// the values it takes gives each tile a cell of its own.[^1]
    ///
    /// The cost of the field follows the cell count, so a fine pitch costs
    /// the world on every tick and a coarse one costs a small fraction of
    /// it. The default is the level 1 pitch.
    ///
    /// # Errors
    ///
    /// Returns an error when the configured extent does not describe a grid,
    /// and when the scale does not describe a lattice over that extent.
    ///
    /// # References
    ///
    /// [^1]: The weather scale. [`WeatherScale`]
    /// Builds a world whose ground the weather of the world shaped.
    ///
    /// **The engine runs the weather forward over an empty world before the
    /// world starts.** It accumulates the temperature and the standing water
    /// of every weather cell over a fixed tick count, stores that small field,
    /// and the terrain readers of the world then read it. A cell that stands
    /// wetter than the world grows more forest. A cell that stands drier, or
    /// colder, grows less.
    ///
    /// **The spin is not a loop.** The weather reads the mean height of a cell
    /// and its open water share, and the climate changes neither. The spin
    /// hands the weather the ground folded from the terrain that the seed
    /// alone gives, the classification runs once afterwards, and nothing feeds
    /// back. The tick count is fixed, so there is no convergence test.[^1]
    ///
    /// A tick count of zero builds the world that the seed alone gives.
    ///
    /// # Errors
    ///
    /// Returns an error when the world refuses to build, and when the weather
    /// refuses the spin.
    ///
    /// # References
    ///
    /// [^1]: ADR-0087, an influence solve runs a fixed iteration count over the whole plane, decision D1. `docs/adrs/draft/adr-0087-an-influence-solve-runs-a-fixed-iteration-count.md`
    pub fn with_climate(
        config: WorldConfig,
        weather_scale: WeatherScale,
        spin_ticks: u64,
        threads: usize,
    ) -> Result<Self, WorldError> {
        let mut world = Self::with_weather_scale(config, weather_scale)?;
        if spin_ticks == 0 {
            return Ok(world);
        }
        let climate = ClimateField::spin(world.terrain, weather_scale, spin_ticks, threads)?;
        world.climate_reference = climate.wetness_reference();
        world.climate = climate;
        Ok(world)
    }

    pub fn with_weather_scale(
        config: WorldConfig,
        weather_scale: WeatherScale,
    ) -> Result<Self, WorldError> {
        Self::with_weather_margin(config, weather_scale, weather_scale.margin_cells())
    }

    /// Builds a world at a stated weather resolution and a stated margin.
    ///
    /// **The margin is the ring of weather cells around the world that the
    /// solve steps and no reader sees.** It exists so that the border of the
    /// world has real upwind. Without it, air leaves through one edge of the
    /// lattice and nothing arrives through the other, so the cells beside an
    /// edge stay starved and a mass can only be born inside the frame.
    ///
    /// The scale derives the margin that a world takes when the caller states
    /// none, from the distance the transport carries air while a mass
    /// forms.[^1]
    ///
    /// **A margin of zero gives the lattice that covers the world and nothing
    /// more**, which is the field the engine held before the margin existed.
    /// A test states that the two agree.
    ///
    /// # Errors
    ///
    /// Returns an error when the configured extent does not describe a grid,
    /// when the scale does not describe a lattice over that extent, and when
    /// the margin makes the lattice too large to index.
    ///
    /// # References
    ///
    /// [^1]: The margin derivation. [`WeatherScale::margin_cells`]
    pub fn with_weather_margin(
        config: WorldConfig,
        weather_scale: WeatherScale,
        weather_margin: u32,
    ) -> Result<Self, WorldError> {
        Self::with_weather_latitudes(config, weather_scale, weather_margin, Latitudes::DEFAULT)
    }

    /// Builds a world at a stated weather resolution, margin and latitude
    /// span.
    ///
    /// **The span says what the row axis of the world means.** A span from
    /// pole to pole makes the world a planet: it has poles, a banded
    /// circulation, subtropical deserts and an equatorial rain belt. A narrow
    /// span makes the world one region of a planet, and the latitude term
    /// then goes flat and the climate comes from the ground and the sea
    /// alone.[^1]
    ///
    /// A world that states no span is a planet.[^1]
    ///
    /// # Errors
    ///
    /// Returns an error when the configured extent does not describe a grid,
    /// when the scale does not describe a lattice over that extent, and when
    /// the margin makes the lattice too large to index.
    ///
    /// # References
    ///
    /// [^1]: ADR-0177, the row axis of a world is a latitude that the world states, decision D1. `docs/adrs/draft/adr-0177-the-row-axis-of-a-world-is-a-latitude-that-the-world-states.md`
    pub fn with_weather_latitudes(
        config: WorldConfig,
        weather_scale: WeatherScale,
        weather_margin: u32,
        latitudes: Latitudes,
    ) -> Result<Self, WorldError> {
        if config.faction_count > FACTION_CEILING {
            return Err(WorldError::FactionCountAboveCeiling(config.faction_count));
        }
        let grid = Grid::new(config.width, config.height)?;
        // The tile value field stores nothing here. It holds the seed and
        // the extent, and it generates a tile when a reader asks for one, so
        // the cost is paid per reader and never once for every tile before
        // the first frame.[^2]
        //
        // [^2]: PRD-0003, a developer sees a world worth looking at, what it costs at the target scale. `docs/product/accepted/prd-0003-a-developer-sees-a-world-worth-looking-at.md`
        let values = TileValues::new(config.seed, grid);
        let layout = BlockLayout::new(grid, BLOCK_BITS_DEFAULT)?;
        // The influence field is a plane over the level 1 cells, so its
        // lattice is the block lattice at the pitch of one block. It is a hex
        // grid for the same reason level 0 is one, and building it here means
        // the field states no geometry of its own.[^1]
        //
        // [^1]: ADR-0017, the world is a rhombus, so a tile index is raw axial, decision D4. `docs/adrs/accepted/adr-0017-the-world-is-a-rhombus-so-a-tile-index-is-raw-axial.md`
        let cell_lattice = Grid::new(layout.blocks_wide(), layout.blocks_high())?;
        // **The weather lattice states its own pitch.** It is the level 1
        // lattice when the world takes the level 1 pitch, and a lattice of
        // its own at any other. The ground under it is folded once here,
        // because every field the weather reads from it is a pure function
        // of the seed and the address.[^3]
        //
        // [^3]: ADR-0068, terrain is generated from the seed and is never stored as a map. `docs/adrs/accepted/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
        let weather_layout = BlockLayout::new(grid, weather_scale.bits())?;
        // **The weather lattice is larger than the world.** The margin is the
        // ring of cells around it that the solve steps and no reader sees. It
        // gives the border of the world real upwind: air that arrives at an
        // edge has crossed ground of the right character rather than arriving
        // from nothing.[^4]
        //
        // [^4]: The margin derivation. [`WeatherScale::margin_cells`]
        let weather_cells = Grid::new(weather_layout.blocks_wide(), weather_layout.blocks_high())?;
        let weather_lattice = PaddedLattice::new(weather_cells, weather_margin)?;
        let soldiers = SoldierArena::new(grid, config.unit_capacity);
        let settlements = SettlementArena::new(grid);
        // The character tier states its own ceiling, and the arena checks
        // it here, once, when the world is built. Nothing checks a count on
        // a later call.[^1]
        //
        // [^1]: ADR-0054, an entity belongs to one of three tiers, declared at creation, decision D3. `docs/adrs/accepted/adr-0054-an-entity-belongs-to-one-of-three-tiers-declared-at-creation.md`
        let characters = CharacterArena::new();
        let mut bridge = UnitTileBridge::new(layout);
        bridge.rebuild(&soldiers)?;
        let terrain = Terrain::new(config.seed, grid);
        let mut world = Self {
            config,
            grid,
            tick: Tick(0),
            values,
            log: Vec::new(),
            gather_log: Vec::new(),
            soldiers,
            settlements,
            characters,
            bridge,
            presence: PresenceRelation::new(),
            observation: Observation::new(layout),
            terrain,
            resources: ResourceField::new(terrain),
            depletion: DepletionLedger::new(),
            departed: [0; RESOURCE_KIND_COUNT],
            delivered: [0; RESOURCE_KIND_COUNT],
            upgrades: UpgradeMap::new(),
            pyramid: Pyramid::new(layout, ResourceField::new(terrain))?,
            level_1_arena: None,
            exits: ExitField::new(cell_lattice),
            returns: ReturnField::new(cell_lattice, config.faction_count),
            home_approaches: ApproachField::new(layout),
            stock_approaches: ApproachField::new(layout),
            destinations: SeededField::new(cell_lattice, config.destination_plane_count()),
            approaches: ApproachField::new(layout),
            destination_seeds: vec![Vec::new(); config.destination_plane_count() as usize],
            destination_crossings: vec![0; config.destination_plane_count() as usize],
            destinations_deferred: false,
            carry_mark: CARRY_MARK_DEFAULT,
            holding: Holding::new(layout),
            luxuries: LuxuryField::new(),
            variety: VarietyLevel::derive(layout, &LuxuryField::new()),
            luxuries_seeded: false,
            influence: InfluenceField::new(cell_lattice, config.faction_count)?,
            weather: WeatherField::with_latitudes(
                weather_lattice,
                weather_scale,
                latitudes,
                config.faction_count,
            )?,
            weather_layout,
            weather_lattice,
            weather_ground: ground_over_lattice(weather_lattice, weather_layout, terrain),
            climate: ClimateField::quiet(weather_layout),
            fire: FireField::new(),
            lightning_chance: 0,
            fire_started_log: Vec::new(),
            fire_ended_log: Vec::new(),
            burned_log: Vec::new(),
            climate_reference: 0,
            controller: Controller::new(config.seed, config.faction_count),
            balance: Balance::default(),
            campaigns: CampaignRegister::new(config.faction_count),
            plan: PlanRegister::new(config.faction_count, PlanRules::DEFAULT),
            census: CensusTotals::default(),
            event_memory: EventMemory::new(config.faction_count),
            schedule: RateSchedule::DEFAULT,
            rates: RateTable::new(),
            effective_rates: RateTable::new(),
            rate_ledger: RateLedger::ZERO,
            shortfall_log: Vec::new(),
            need_rule: NeedRule::DEFAULT,
            cohorts: CohortTable::new(),
            draw_ledger: DrawLedger::ZERO,
            rationed_log: Vec::new(),
            death_plane: DeathPlane::new(),
            starved_log: Vec::new(),
            // The world is built with the default table, so a unit that
            // nothing typed is a worker and gathers, builds and carries.[^1]
            //
            // [^1]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decision D4. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
            unit_types: DEFAULT_UNIT_TYPE_TABLE,
            queues: QueueTable::new(),
            build_costs: BuildCostTable::default(),
            queue_schedule: RateSchedule::new(QUEUE_PERIOD_DEFAULT, QUEUE_PHASE_DEFAULT)
                .expect("the default period is inside the range"),
            growth_schedule: RateSchedule::new(
                growth::GROWTH_PERIOD_DEFAULT,
                growth::GROWTH_PHASE_DEFAULT,
            )
            .expect("the default period is inside the range"),
            housing_per_person: growth::HOUSING_PER_PERSON_DEFAULT,
            food_per_birth: growth::FOOD_PER_BIRTH_DEFAULT,
            birth_chance: growth::BIRTH_CHANCE_DEFAULT,
            founding_housing: growth::FOUNDING_HOUSING_DEFAULT,
            births: 0,
            growth_ledger: [Accum(0); COMMODITY_COUNT],
            // The world is built with the default upgrade table, so a build
            // order names one of the categories that table holds.[^2]
            //
            // [^2]: ADR-0151, an upgrade is a category with a ground fit and a level, decision D1. `docs/adrs/accepted/adr-0151-an-upgrade-is-a-category-with-a-ground-fit-and-a-level.md`
            upgrade_table: upgrade::DEFAULT_UPGRADE_TABLE,
            fell_plane: DeathPlane::new(),
            fell_log: Vec::new(),
            grievances: Vec::new(),
            relations: RelationMatrix::new(config.faction_count),
            convert_marks: Vec::new(),
            converted_log: Vec::new(),
            promoted_log: Vec::new(),
            character_schedule: RateSchedule::DEFAULT,
            promotion_budget: u32::MAX,
            choice: ChoiceSchedule::DEFAULT,
            buckets: NeedBuckets::DEFAULT,
            weights: WeightProfile::EVEN,
            positions: PositionTable::new(),
            position_schedule: RateSchedule::DEFAULT,
            store_account: [Accum(0); COMMODITY_COUNT],
            trade: TradeTable::new(config.faction_count),
            trade_log: Vec::new(),
            collapsed_log: Vec::new(),
            finished_log: Vec::new(),
            founded_log: Vec::new(),
            taken_log: Vec::new(),
            siege_rules: SiegeRules::DEFAULT,
            sites_captured: 0,
            sites_razed: 0,
            sieges_pressed: 0,
            sieges_relieved: 0,
            eliminated_log: Vec::new(),
            eliminated: vec![0u8; FACTION_CEILING as usize],
            market: MarketTable::new(config.faction_count, DEFAULT_BOARD_ROWS),
            land_list_bound: DEFAULT_LAND_LIST_BOUND,
        };
        // A world that has never stepped still answers a question about a
        // region. A level that nothing rebuilt would describe an empty world
        // and would be wrong rather than absent.
        world.rebuild_level_1(1, Destinations::Derive)?;
        // The conductance of a cell follows the ground it covers, and the
        // ground does not change for the life of a world, so this runs once
        // and never again. It reads the level that the rebuild above just
        // filled rather than sweeping the tiles a second time.[^1]
        //
        // [^1]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D2. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
        world.influence.read_the_ground(world.pyramid.cells())?;
        // A world that has never stepped still answers the presence
        // question. A relation that nothing derived would refuse the read,
        // and a caller cannot tell that refusal from a stale one.[^2]
        //
        // [^2]: ADR-0111, the presence relation is derived at the end of the step and never stored as a fact, decision D4. `docs/adrs/draft/adr-0111-the-presence-relation-is-derived-at-the-end-of-the-step.md`
        world.presence.rebuild(&world.soldiers, &world.holding, 1)?;
        // A world that has never stepped still answers what a faction sees.
        // A layer that nothing rebuilt would say that every faction is
        // blind, and a caller cannot tell that from a faction with no
        // unit.[^3]
        //
        // [^3]: ADR-0059, fog storage grows with observed area, not with world area, decision D3. `docs/adrs/accepted/adr-0059-fog-storage-grows-with-observed-area.md`
        world
            .observation
            .rebuild(&world.soldiers, world.terrain, 1, world.tick)?;
        Ok(world)
    }
    /// Seeds the world: founds one run for every faction and places the
    /// luxuries, both from the seed.
    ///
    /// **This takes no parameter.** The founding group and the deposit count
    /// are provisional values that the balance register holds, so a caller
    /// that builds a world from a seed calls this once and names nothing.[^1]
    /// The founding draws as the founding verb draws, and the luxuries are
    /// placed by a keyed draw on the deposit index at frame zero.[^2]
    ///
    /// Returns what each faction got, in faction order.
    ///
    /// **The call leaves the world readable.** A founding seats a group and
    /// spends nothing, and a seated group is a change of the unit arena. The
    /// derived unit structure counts the changes of the arena and refuses
    /// every answer once the counts differ, so a reader that drew before the
    /// first step met a refusal.[^3] The seeding therefore refreshes the
    /// structure before it returns, in the way a step does at its end.
    ///
    /// **The refresh costs one rebuild.** The rebuild runs on one thread, and
    /// it runs once here, whatever the faction count is.[^4] A world is
    /// seeded once, so the world pays this at creation and never again.
    ///
    /// # Errors
    ///
    /// Returns an error when the world was seeded before, and when the
    /// rebuild of the derived unit structure refuses. A world is seeded once.
    ///
    /// # References
    ///
    /// [^1]: Balance register, the founding group and the luxury deposits. `docs/reference/balance.md`
    /// [^2]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
    /// [^3]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decisions D3 and D4. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
    /// [^4]: ADR-0071, the bridge rebuild orders on one thread, decision D2. `docs/adrs/accepted/adr-0071-the-bridge-rebuild-orders-on-one-thread.md`
    pub fn seed_world(&mut self) -> Result<Vec<FoundingOutcome>, SeedError> {
        if self.luxuries_seeded {
            return Err(SeedError::Luxury(LuxuryError::AlreadySeeded));
        }
        let outcomes = self.found_run_for_every_faction(FOUNDING_GROUP_DEFAULT);
        let tiles = u64::from(self.grid.tile_count());
        let mut placements = Vec::with_capacity(LUXURY_DEPOSITS_DEFAULT as usize);
        for deposit in 0..LUXURY_DEPOSITS_DEFAULT {
            let tile = rng::draw_below(
                self.config.seed,
                rng::SYSTEM_LUXURY,
                0,
                u64::from(deposit),
                0,
                tiles,
            );
            let luxury = LuxuryId((deposit % u32::from(crate::luxury::LUXURY_CEILING)) as u8);
            placements.push((TileIdx(tile as u32), luxury));
        }
        self.seed_luxuries(&placements)?;
        // **A verb that changes the structure leaves the world readable.**
        // The foundings above seated a group for every faction, and each
        // seating changed the unit arena. A caller that drew here before the
        // first step met a refusal, and the opening refresh of that step was
        // what repaired it. A reader is not obliged to step first, and the
        // drawing is not the only reader, so the verb repairs it here.
        self.refresh_bridge()?;
        Ok(outcomes)
    }
}
