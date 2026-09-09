//! The ring stack block of the observation, and the passes that build it.
//!
//! The block holds one cell for each cell of the egocentric ring frame, and
//! one group of channels for each cell. The channel table of this module
//! names every channel and is the one statement of the order. The frame
//! samples the ground around a faction at falling resolution, so the block
//! has one width on every world.[^1]
//!
//! # Where each channel comes from
//!
//! Three sources fill the block, and each has a cost that follows the ground
//! the faction has walked rather than the ground the world holds.[^2]
//!
//! The near pass reads level 0 tiles. It covers rings 0 to 3, which is every
//! hex distance below 8, which is 169 tiles. The count is a constant, so the
//! near pass costs the same on a world of 576 tiles and on a world of 16.7
//! million.
//!
//! The far pass reads the block lattice that the fog layer and the summary
//! level share. It walks the blocks that the faction has observed, in
//! ascending block index, and it assigns each block to a cell by the hex
//! distance and the direction of the block centre. It skips a block that
//! falls in ring 3 or below, because the near pass already read every tile
//! of that ground.
//!
//! Four entity passes fill the channels that no summary holds. One walks the
//! tiles the faction holds, one walks the settlements of the world, one walks
//! the upgrade sites of the world, and one walks the burning tiles. Each is
//! bounded by an entity count and none of them is bounded to a band.
//!
//! # A faction is not fogged from its own ground
//!
//! The channels that report the ground of the reader read the holding
//! directly and not through the fog. A faction that reads its own territory
//! through the fog of the present frame sees its own border move as its units
//! move, because a tile it holds and does not watch reports no holder.[^3]
//! Fog scopes the quantities of a rival, and it scopes the knowledge the
//! reader has of the ground. It does not hide the border of the reader from
//! the reader.
//!
//! # Every channel reads a source in every ring
//!
//! **A channel that a far cell cannot fill is worse than a reserved channel.**
//! The schema states a real bound for it, the gate channel says the cell lies
//! inside the world, and a reader then takes the zero for a real absence. The
//! gate works for a whole cell, and a channel that stops at ring 3 fails one
//! level below it.[^5]
//!
//! Each channel therefore names a source that both passes reach. The near
//! pass reads the tile, and the far pass reads the field of the summary that
//! holds the same quantity. Four channels needed a derivation to reach the
//! far band, and each one reads a quantity the summary already holds. The
//! water share is the ground the summary does not open, the resource share is
//! the deposit count, the height spread is the second moment of the height,
//! and the ground water is one reading for each block of the weather
//! lattice.[^6]
//!
//! Two channels reach every ring through a pass of their own rather than
//! through the summary. The hazard share walks the burning tiles, which the
//! fire field holds as a list. The memory age reads the clock that records
//! the tick each faction last saw each block.[^7]
//!
//! A test reads a live world and fails when a channel holds zero in every
//! cell of it, so a channel cannot go dark without something failing.[^8]
//!
//! # Determinism
//!
//! Every pass has one stated order. The near pass runs in ascending hex
//! distance and then in ascending position around each ring. The far pass
//! runs in ascending block index, which is the order the fog layer already
//! holds. The holding pass runs in ascending tile index. The settlement pass
//! and the upgrade pass run in ascending arena slot. The hazard pass runs in
//! the order the fire field holds. Nothing reads a thread identity and
//! nothing reads a completion order.[^4]
//!
//! # References
//!
//! [^1]: Findings register, FND-670. `docs/FINDINGS.md`
//! [^2]: ADR-0059, fog storage grows with observed area, not with world area, decision D1. `docs/adrs/accepted/adr-0059-fog-storage-grows-with-observed-area.md`
//! [^3]: Findings register, FND-671. `docs/FINDINGS.md`
//! [^4]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
//! [^5]: ADR-0195, the observation of a faction is a fixed-width scale-free table, decision D8. `docs/adrs/draft/adr-0195-the-observation-of-a-faction-is-a-fixed-width-scale-free-table.md`
//! [^6]: ADR-0024, every summary field is declared extensive or intensive, decision D2. `docs/adrs/accepted/adr-0024-every-summary-field-is-declared-extensive-or-intensive.md`
//! [^7]: ADR-0059, fog storage grows with observed area, not with world area, decision D5. `docs/adrs/accepted/adr-0059-fog-storage-grows-with-observed-area.md`
//! [^8]: Testing Rules, section 2a. `.agents/rules/testing.md`

use crate::faction_view::{Admit, BlockMask, FactionTile, FactionViewError};
use crate::hex::Axial;
use crate::obs_ring::{
    cell_of_delta, cell_of_ring_and_delta, hex_distance_from_origin, ring_of_distance,
    ring_position, CellExtent, FIRST_FAR_RING, NEAR_DISTANCE, RING_STACK_CELLS,
    RING_STACK_CHANNELS,
};
use crate::pyramid::spread_of;
use crate::sim_math;
use crate::types::{Accum, FactionId, Fix32};
use crate::world::World;

/// The positions that the ring stack block holds.
pub const RING_STACK_SLOTS: u32 = RING_STACK_CELLS * RING_STACK_CHANNELS;

/// The name a schema gives to the space the ring stack lays its cells out in.
///
/// A reader of the observation must tell a spatial field from a scalar one,
/// and the drawing tool of this project established the vocabulary before the
/// engine published it.[^1]
///
/// # References
///
/// [^1]: The drawing tool of the observation. `python/cachette/learn/picture.py`
pub const RING_SPACE: &str = "ring";

/// Returns the number of the channel of one name.
///
/// The channel table is the one place that pairs a name with a position, so
/// every named channel below reads its number from that table. A name the
/// table does not hold fails the build.
const fn channel_number(wanted: &str) -> u32 {
    let mut index = 0usize;
    while index < RING_STACK_CHANNEL_NAMES.len() {
        if same_name(RING_STACK_CHANNEL_NAMES[index], wanted) {
            return index as u32 + 1;
        }
        index += 1;
    }
    panic!("the ring stack channel table holds no channel of that name");
}

/// Returns whether two names hold the same bytes.
///
/// A const context cannot compare two strings with the equality operator, so
/// this walks the bytes of both.
pub(crate) const fn same_name(left: &str, right: &str) -> bool {
    let left = left.as_bytes();
    let right = right.as_bytes();
    if left.len() != right.len() {
        return false;
    }
    let mut index = 0usize;
    while index < left.len() {
        if left[index] != right[index] {
            return false;
        }
        index += 1;
    }
    true
}

/// The channel that carries how much of the cell lies inside the world.
///
/// **This channel gates the rest.** A ring that lies outside the world reads
/// zero here, and every other channel of that cell reads zero because there
/// is no ground to report. A policy reads this one and learns which rings its
/// world reaches.
pub const AREA_CHANNEL: u32 = channel_number(AREA_CHANNEL_NAME);

/// The name of the channel that gates the rest.
///
/// The schema publishes the name, so a reader of the observation tells an
/// absent cell from a cell that holds zero.[^1]
///
/// # References
///
/// [^1]: ADR-0195, the observation of a faction is a fixed-width scale-free table, decision D8. `docs/adrs/draft/adr-0195-the-observation-of-a-faction-is-a-fixed-width-scale-free-table.md`
pub const AREA_CHANNEL_NAME: &str = "area_inside_world";

/// The channel that carries the observed share of the cell.
pub const OBSERVED_CHANNEL: u32 = channel_number("observed_share");

/// The channel that carries the share of the cell the reader holds.
pub const OWN_HELD_CHANNEL: u32 = channel_number("own_held_share");

/// The channel that carries the rival unit presence of the cell.
///
/// **This is the one statement of that channel number.** The frontier block
/// weights this channel by distance to give its threat pressure, and it reads
/// the number here rather than holding a second copy of it.[^1]
///
/// # References
///
/// [^1]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
pub const RIVAL_PRESENCE_CHANNEL: u32 = channel_number("rival_unit_density");

/// The channel that carries the settlement count of the reader in the cell.
pub const OWN_SETTLEMENT_CHANNEL: u32 = channel_number("own_settlements");

/// The channel that carries the share of the cell inside the reach of the
/// reader.
pub const OWN_REACH_CHANNEL: u32 = channel_number("own_reach_share");

/// What one pass of the ring stack touched.
///
/// A test reads this to assert that the build cost follows the ground the
/// faction has walked. **The pass must not touch a cell the faction has never
/// observed**, and a count is the only way to prove that from outside.[^1]
///
/// # References
///
/// [^1]: Report 42, what a policy should be able to see, section 12.1. `docs/research/reports/42-what-a-policy-should-be-able-to-see.md`
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RingStackCost {
    /// The level 0 tiles the near pass asked the world about.
    pub near_tiles: i64,
    /// The blocks of the lattice the far pass read.
    pub summary_blocks: i64,
    /// The held tiles the holding pass walked.
    pub held_tiles: i64,
    /// The settlements the settlement pass walked.
    pub settlements: i64,
    /// The upgrade sites the upgrade pass walked.
    pub upgrade_sites: i64,
    /// The burning tiles the hazard pass walked.
    pub burning_tiles: i64,
}

/// The totals that one cell of the frame accumulates.
///
/// Every field is a 64-bit integer. A one-byte tile field summed over the
/// tile count of the target world overflows a 32-bit accumulator, and an
/// accumulator must not depend on that margin.[^1]
///
/// # References
///
/// [^1]: ADR-0023, an aggregate combines exactly, in any order, decision D3. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
#[derive(Clone, Copy, Debug, Default)]
struct CellTotals {
    observed: i64,
    seen_now: i64,
    open: i64,
    height_total: i64,
    height_square_total: i64,
    food_total: i64,
    ground_water_total: i64,
    memory_age_total: i64,
    value_total: i64,
    resource_tiles: i64,
    own_held: i64,
    rival_held: i64,
    unclaimed_open: i64,
    own_units: i64,
    rival_units: i64,
    own_strength: i64,
    rival_strength: i64,
    own_settlements: i64,
    rival_settlements: i64,
    own_upgrades: i64,
    rival_upgrades: i64,
    hazard_tiles: i64,
}

/// The ring stack of one faction, and what building it cost.
#[derive(Clone, Debug)]
pub struct RingStack {
    centre: Axial,
    slots: Vec<i64>,
    cost: RingStackCost,
}

impl RingStack {
    /// Returns the centre of the frame the stack was built around.
    ///
    /// The centre is the integer centroid of the tiles the faction holds. A
    /// faction that holds no tile takes the centroid of its units, and a
    /// faction with neither takes the centre of the world.
    #[must_use]
    pub const fn centre(&self) -> Axial {
        self.centre
    }

    /// Returns what the build touched.
    #[must_use]
    pub const fn cost(&self) -> RingStackCost {
        self.cost
    }

    /// Returns every position of the block, in ascending cell order.
    ///
    /// The channels of one cell are contiguous, so a set encoder over the
    /// cells reads one contiguous run for each cell and needs no stride
    /// table.
    #[must_use]
    pub fn slots(&self) -> &[i64] {
        &self.slots
    }

    /// Returns one channel of one cell.
    ///
    /// The channel number runs from one to the channel count, as the design
    /// layout numbers them. The channel table names every one of them.
    #[must_use]
    pub fn channel(&self, cell: u32, channel: u32) -> i64 {
        if channel == 0 || channel > RING_STACK_CHANNELS {
            return 0;
        }
        let position = cell * RING_STACK_CHANNELS + channel - 1;
        self.slots.get(position as usize).copied().unwrap_or(0)
    }
}

impl World {
    /// Builds the ring stack block of the observation of one faction.
    ///
    /// The stack holds what the faction observes about the ground around it,
    /// and it holds the ground the faction itself owns without a fog
    /// mask.[^1] **No argument widens the answer.**
    ///
    /// The build cost follows the ground the faction has walked and the
    /// entity counts of the world. It does not follow the world area.[^2]
    ///
    /// # Errors
    ///
    /// Returns an error when the number names no faction of this world, and
    /// when the derived unit structure does not describe the units.
    ///
    /// # References
    ///
    /// [^1]: Findings register, FND-671. `docs/FINDINGS.md`
    /// [^2]: ADR-0059, fog storage grows with observed area, not with world area, decision D1. `docs/adrs/accepted/adr-0059-fog-storage-grows-with-observed-area.md`
    pub fn faction_ring_stack(&self, faction: FactionId) -> Result<RingStack, FactionViewError> {
        if self.standing(faction).is_none() {
            return Err(FactionViewError::NoSuchFaction(faction));
        }
        let centre = self.frame_centre(faction);
        let extent = CellExtent::sample(self.grid(), centre);
        let mut totals = vec![CellTotals::default(); RING_STACK_CELLS as usize];
        let mut cost = RingStackCost::default();

        self.accumulate_near(faction, centre, &mut totals, &mut cost);
        self.accumulate_far(faction, centre, &mut totals, &mut cost)?;
        self.accumulate_holding(faction, centre, &mut totals, &mut cost);
        self.accumulate_settlements(faction, centre, &mut totals, &mut cost);
        self.accumulate_upgrades(faction, centre, &mut totals, &mut cost);
        self.accumulate_hazards(faction, centre, &mut totals, &mut cost);

        let mut slots = vec![0i64; RING_STACK_SLOTS as usize];
        for (cell, total) in totals.iter().enumerate() {
            let cell = cell as u32;
            let channels = channels_of(*total, &extent, cell);
            let first = (cell * RING_STACK_CHANNELS) as usize;
            for (offset, value) in channels.iter().enumerate() {
                slots[first + offset] = i64::from(value.0);
            }
        }
        Ok(RingStack {
            centre,
            slots,
            cost,
        })
    }

    /// Returns the centre of the egocentric frame of one faction.
    ///
    /// The centre is the integer centroid of the tiles the faction holds. The
    /// sums are 64 bits wide, because an axial coordinate of the target world
    /// reaches 4095 and the tile count reaches 16.7 million.
    ///
    /// A faction that holds no tile takes the centroid of its units instead,
    /// and that walk costs its unit count. A faction that holds neither takes
    /// the centre of the world, which costs nothing.
    ///
    /// **The action table reads this centre too.** A place argument of a verb
    /// carries a cell of this frame, so a cell index names the same ground in
    /// the array a policy reads and in the integer that policy emits. One
    /// function states the centre, and both callers read it.[^1] [^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0199, a verb names a place by a cell of the egocentric frame the observation publishes, decision D1. `docs/adrs/draft/adr-0199-a-verb-names-a-place-by-a-cell-of-the-egocentric-frame.md`
    /// [^2]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
    pub(crate) fn frame_centre(&self, faction: FactionId) -> Axial {
        let grid = self.grid();
        let mut sum_q = 0i64;
        let mut sum_r = 0i64;
        let mut count = 0i64;
        for tile in self.holding().tiles_held_by(faction) {
            if let Some(address) = grid.address_of(tile) {
                sum_q += i64::from(address.q);
                sum_r += i64::from(address.r);
                count += 1;
            }
        }
        if count == 0 {
            for unit in self.soldiers().iter_faction(faction) {
                if let Some(address) = self.soldiers().address(unit) {
                    sum_q += i64::from(address.q);
                    sum_r += i64::from(address.r);
                    count += 1;
                }
            }
        }
        if count == 0 {
            return Axial::new((grid.width() / 2) as i32, (grid.height() / 2) as i32);
        }
        Axial::new((sum_q / count) as i32, (sum_r / count) as i32)
    }

    /// Reads the level 0 tiles of rings 0 to 3.
    ///
    /// The pass visits every tile of the near band once, in ascending hex
    /// distance and then in ascending position around each ring. It asks the
    /// world about 169 addresses on every world, so its cost is a constant.
    ///
    /// **Every total the pass accumulates has a twin in the summary the far
    /// pass reads.** A total that only the near pass fills reads zero in 120
    /// of the 151 cells, and the cell gate cannot say so, because the gate
    /// answers for a whole cell and the failure is one channel of it.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0195, the observation of a faction is a fixed-width scale-free table, decision D8. `docs/adrs/draft/adr-0195-the-observation-of-a-faction-is-a-fixed-width-scale-free-table.md`
    fn accumulate_near(
        &self,
        faction: FactionId,
        centre: Axial,
        totals: &mut [CellTotals],
        cost: &mut RingStackCost,
    ) {
        let grid = self.grid();
        for distance in 0..=NEAR_DISTANCE {
            let positions = if distance == 0 { 1 } else { 6 * distance };
            for step in 0..positions {
                let delta = ring_position(distance, step);
                let address = centre.add(delta);
                if !grid.contains(address) {
                    continue;
                }
                cost.near_tiles += 1;
                let Some(tile) = self.faction_tile(faction, address) else {
                    continue;
                };
                let Some(ground) = tile.ground() else {
                    continue;
                };
                let cell = cell_of_delta(delta);
                let Some(total) = totals.get_mut(cell as usize) else {
                    continue;
                };
                total.observed += 1;
                total.height_total += i64::from(ground.height.0);
                total.height_square_total +=
                    i64::from(sim_math::mul(ground.height, ground.height).0);
                if ground.kind.is_passable() {
                    total.open += 1;
                }
                if ground.generated.iter().any(|amount| amount.0 > 0) {
                    total.resource_tiles += 1;
                }
                total.food_total += i64::from(ground.generated[0].0);
                if let Some(water) = self.ground_water_at(address) {
                    total.ground_water_total += water;
                }
                total.memory_age_total += self.tile_memory_age(faction, address);
                if let FactionTile::Seen(seen) = tile {
                    total.seen_now += 1;
                    total.value_total += i64::from(seen.value.0);
                    if let Some(holder) = seen.holder {
                        if holder != faction {
                            total.rival_held += 1;
                        }
                    } else if ground.kind.is_passable() {
                        total.unclaimed_open += 1;
                    }
                    self.split_units_on(faction, address, total);
                }
            }
        }
    }

    /// Returns the ticks since the faction last saw the ground of one tile.
    ///
    /// **The clock counts one entry for each block and not one for each
    /// tile**, so every tile of one block reads one age.[^1] A tile the
    /// faction sees now reads zero, and a tile it has never seen reads zero
    /// as well, because a cell it has never seen reports nothing at all.
    ///
    /// # References
    ///
    /// [^1]: ADR-0059, fog storage grows with observed area, not with world area, decision D5. `docs/adrs/accepted/adr-0059-fog-storage-grows-with-observed-area.md`
    fn tile_memory_age(&self, faction: FactionId, address: Axial) -> i64 {
        let layout = self.observation().layout();
        let Some(tile) = self.grid().index_of(address) else {
            return 0;
        };
        let Some(key) = layout.key_of(tile) else {
            return 0;
        };
        let block = layout.block_of_key(key);
        self.block_memory_age(faction, block)
    }

    /// Returns the ticks since the faction last saw one block.
    ///
    /// The clock answers nothing for a block the faction has never seen, and
    /// this reads that answer as zero. Such a block contributes no observed
    /// tile either, so it never reaches the denominator of the channel.
    fn block_memory_age(&self, faction: FactionId, block: u32) -> i64 {
        self.observation()
            .block_age(faction, block, self.tick())
            .map_or(0, |age| age.min(i64::MAX as u64) as i64)
    }

    /// Adds the units of one seen tile to the own totals and the rival
    /// totals.
    ///
    /// A tile carries at most its capacity in units, so the walk over one
    /// tile is bounded by the terrain rules and not by the population.
    ///
    /// **The strength of a unit comes from the unit type table**, which is
    /// the one place that pairs an attack column and an armour column with a
    /// strength. A second rule for the strength of a unit would be one fact
    /// in two places.[^1] [^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decision D1. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
    /// [^2]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
    fn split_units_on(&self, faction: FactionId, address: Axial, total: &mut CellTotals) {
        let Ok(units) = self.soldiers_on(address) else {
            return;
        };
        for unit in units {
            let strength = self
                .soldiers()
                .unit_type(*unit)
                .map_or(Fix32::ZERO, |kind| self.unit_types().strength(kind));
            match self.soldiers().faction(*unit) {
                Some(owner) if owner == faction => {
                    total.own_units += 1;
                    total.own_strength =
                        sim_math::accumulate(Accum(total.own_strength), strength).0;
                }
                Some(_) => {
                    total.rival_units += 1;
                    total.rival_strength =
                        sim_math::accumulate(Accum(total.rival_strength), strength).0;
                }
                None => {}
            }
        }
    }

    /// Reads the observed blocks of the lattice and fills rings 4 and above.
    ///
    /// The pass walks the blocks that the fog layer of the faction names, in
    /// ascending block index. **It reads no block that the faction has never
    /// observed**, so its cost follows the observed area.[^1]
    ///
    /// It skips a block whose centre falls in ring 3 or below, because the
    /// near pass already read every tile of that ground.
    ///
    /// **The pass fills every total the near pass fills, except the hazard
    /// count.** A total the far pass leaves at zero reads zero in 120 of the
    /// 151 cells while the cell gate still says the cell lies inside the
    /// world.[^2] The water count, the deposit count and the second moment of
    /// the height come from the summary. The ground water comes from the
    /// weather cell of the block, and the memory age from the clock of the
    /// fog layer. A separate pass walks the burning tiles, because the
    /// summary holds no hazard.
    ///
    /// The weather lattice and the fog lattice share one block, so one
    /// reading of the ground water serves every tile of the block.[^3] The
    /// pass multiplies that reading by the admitted tiles, which is what the
    /// near pass accumulates one tile at a time.
    ///
    /// # References
    ///
    /// [^1]: ADR-0059, fog storage grows with observed area, not with world area, decision D1. `docs/adrs/accepted/adr-0059-fog-storage-grows-with-observed-area.md`
    /// [^2]: ADR-0195, the observation of a faction is a fixed-width scale-free table, decision D8. `docs/adrs/draft/adr-0195-the-observation-of-a-faction-is-a-fixed-width-scale-free-table.md`
    /// [^3]: ADR-0160, the wind is carried state, and the pressure gradient accelerates it, decision D1. `docs/adrs/accepted/adr-0160-the-wind-is-carried-state-and-the-pressure-gradient-accelerates-it.md`
    fn accumulate_far(
        &self,
        faction: FactionId,
        centre: Axial,
        totals: &mut [CellTotals],
        cost: &mut RingStackCost,
    ) -> Result<(), FactionViewError> {
        let observation = self.observation();
        let layout = observation.layout();
        let visible = observation.visible_layer(faction);
        let remembered = observation.remembered_layer(faction);
        let blocks = merge_ascending(
            visible.map_or(&[][..], |layer| layer.populated_blocks()),
            remembered.map_or(&[][..], |layer| layer.populated_blocks()),
        );
        let edge = layout.block_edge() as i32;
        let wide = layout.blocks_wide().max(1);
        for block in blocks {
            let column = (block % wide) as i32;
            let row = (block / wide) as i32;
            let middle = Axial::new(column * edge + edge / 2, row * edge + edge / 2);
            let delta = Axial::new(middle.q - centre.q, middle.r - centre.r);
            let ring = ring_of_distance(hex_distance_from_origin(delta));
            if ring < FIRST_FAR_RING {
                continue;
            }
            cost.summary_blocks += 1;
            let mask = BlockMask::new(
                visible.and_then(|layer| layer.block(block)),
                remembered.and_then(|layer| layer.block(block)),
            );
            let masked = self.masked_block(&mask, faction, block, Admit::SeenEver)?;
            let summary = masked.summary();
            let cell = cell_of_ring_and_delta(ring, delta);
            let Some(total) = totals.get_mut(cell as usize) else {
                continue;
            };
            total.observed += masked.admitted();
            total.seen_now += i64::from(visible.map_or(0, |layer| layer.block_population(block)));
            total.open += summary.open_tiles();
            total.height_total += summary.height_total().0;
            total.height_square_total += summary.height_square_total().0;
            total.food_total += summary.food_total().0;
            total.value_total += summary.value_total().0;
            total.resource_tiles += summary.deposit_tiles();
            total.rival_held += masked.other_held_tiles();
            total.own_units += masked.own_units();
            total.rival_units += masked.other_units();
            total.own_strength += masked.own_strength().0;
            total.rival_strength += masked.other_strength().0;
            total.unclaimed_open += (summary.open_tiles() - summary.held_tiles()).max(0);
            total.ground_water_total += self
                .ground_water_at(Axial::new(column * edge, row * edge))
                .unwrap_or(0)
                .saturating_mul(masked.admitted());
            total.memory_age_total += self
                .block_memory_age(faction, block)
                .saturating_mul(masked.admitted());
        }
        Ok(())
    }

    /// Adds the tiles the faction holds to the cell each one falls in.
    ///
    /// The pass walks the held tile list, which grows with the ground that
    /// somebody holds and not with the world.[^1] It reads the holding and
    /// not the fog, because a faction is not fogged from its own border.[^2]
    ///
    /// The held count is the numerator of two channels. One divides it by the
    /// observed tiles of the cell, and the other divides it by the in-world
    /// tiles of the cell. Held ground is ground inside the reach of a city
    /// the faction owns, so the second channel is the reach channel.[^3]
    ///
    /// # References
    ///
    /// [^1]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D4. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
    /// [^2]: Findings register, FND-671. `docs/FINDINGS.md`
    /// [^3]: ADR-0150, held ground is the ground within reach of a city its faction owns, decision D3. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
    fn accumulate_holding(
        &self,
        faction: FactionId,
        centre: Axial,
        totals: &mut [CellTotals],
        cost: &mut RingStackCost,
    ) {
        let grid = self.grid();
        for tile in self.holding().tiles_held_by(faction) {
            cost.held_tiles += 1;
            let Some(address) = grid.address_of(tile) else {
                continue;
            };
            if let Some(total) = totals.get_mut(cell_at(centre, address) as usize) {
                total.own_held += 1;
            }
        }
    }

    /// Adds the settlements of the world to the cell each one falls in.
    ///
    /// The pass walks the settlement arena in ascending slot order. A
    /// settlement of the reader counts wherever it stands, because the reader
    /// is not fogged from its own settlements. A settlement of a rival counts
    /// only where the reader sees the ground this frame.
    fn accumulate_settlements(
        &self,
        faction: FactionId,
        centre: Axial,
        totals: &mut [CellTotals],
        cost: &mut RingStackCost,
    ) {
        let arena = self.settlements();
        for settlement in arena.iter() {
            cost.settlements += 1;
            let Some(owner) = self.settlement_faction(settlement) else {
                continue;
            };
            let Some(address) = arena.address(settlement) else {
                continue;
            };
            let Some(total) = totals.get_mut(cell_at(centre, address) as usize) else {
                continue;
            };
            if owner == faction {
                total.own_settlements += 1;
            } else if self.faction_sees_now(faction, address) {
                total.rival_settlements += 1;
            }
        }
    }

    /// Adds the finished upgrades of the world to the cell each one falls in.
    ///
    /// A site at level zero holds nothing yet, so the pass counts a site only
    /// when a level stands there. An upgrade of the reader counts wherever it
    /// stands, and an upgrade of a rival counts only where the reader sees
    /// the ground this frame.
    fn accumulate_upgrades(
        &self,
        faction: FactionId,
        centre: Axial,
        totals: &mut [CellTotals],
        cost: &mut RingStackCost,
    ) {
        let grid = self.grid();
        for site in self.upgrade_sites() {
            cost.upgrade_sites += 1;
            if site.level == 0 {
                continue;
            }
            let Some(address) = grid.address_of(site.tile) else {
                continue;
            };
            let Some(total) = totals.get_mut(cell_at(centre, address) as usize) else {
                continue;
            };
            let owner = self
                .tile_holder(address)
                .and_then(|holder| holder.faction());
            match owner {
                Some(holder) if holder == faction => total.own_upgrades += 1,
                Some(_) if self.faction_sees_now(faction, address) => total.rival_upgrades += 1,
                _ => {}
            }
        }
    }

    /// Adds the burning tiles of the world to the cell each one falls in.
    ///
    /// **The fire field holds the burning tiles as a list**, so the pass
    /// costs the burning count and never the world.[^1] The near pass reads
    /// each tile of its own band, and it could count a fire there. It does
    /// not, because a per-band count reads zero in every far cell and the
    /// cell gate cannot say so.
    ///
    /// A fire is a fact of the present frame, so the pass counts a fire only
    /// where the reader sees the ground this frame. A remembered tile carries
    /// the ground alone.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0096, cost follows the lattice, not the population, and a unit is a reader, decision D1. `docs/adrs/draft/adr-0096-cost-follows-the-lattice-not-the-population.md`
    /// [^2]: ADR-0059, fog storage grows with observed area, not with world area, decision D4. `docs/adrs/accepted/adr-0059-fog-storage-grows-with-observed-area.md`
    fn accumulate_hazards(
        &self,
        faction: FactionId,
        centre: Axial,
        totals: &mut [CellTotals],
        cost: &mut RingStackCost,
    ) {
        let grid = self.grid();
        for burning in self.fire().burning() {
            cost.burning_tiles += 1;
            let Some(address) = grid.address_of(burning.tile) else {
                continue;
            };
            if !self.faction_sees_now(faction, address) {
                continue;
            }
            if let Some(total) = totals.get_mut(cell_at(centre, address) as usize) {
                total.hazard_tiles += 1;
            }
        }
    }
}

/// Returns the cell of the frame that one world address falls in.
fn cell_at(centre: Axial, address: Axial) -> u32 {
    cell_of_delta(Axial::new(address.q - centre.q, address.r - centre.r))
}

/// Merges two ascending block lists into one ascending list with no repeats.
fn merge_ascending(first: &[u32], second: &[u32]) -> Vec<u32> {
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

/// Returns the density of a count over a tile count, in fixed point.
///
/// A count over a tile count is below one in almost every cell, so an integer
/// division would read zero and lose the signal. The value is therefore a
/// Q16.16 density, and the compressed magnitude reads it.
fn density(count: i64, tiles: i64) -> i64 {
    if tiles < 1 {
        return 0;
    }
    count.saturating_mul(i64::from(Fix32::ONE.0)) / tiles
}

/// Returns a fixed-point mean as a share of the full range of the quantity.
///
/// The quantity is already a Q16.16 fraction of its own range, so the mean is
/// the total divided by the count. Dividing by the count times one gives that
/// mean on the share scale.
fn mean_share(total: i64, count: i64) -> Fix32 {
    sim_math::bounded_share(total, count.saturating_mul(i64::from(Fix32::ONE.0)))
}

/// Returns the mean of a fixed-point total over a count.
///
/// The total is already a sum of Q16.16 values, so the mean needs no scale
/// of its own. A density of whole numbers needs one, and the density reader
/// above supplies it.
fn fixed_mean(total: i64, count: i64) -> i64 {
    if count < 1 {
        return 0;
    }
    total / count
}

/// Returns how much the height of a cell varies, over its observed tiles.
///
/// **The reading comes from the summary type**, which is the one place that
/// states what a spread of the height means and why the mean absolute
/// difference is not it.[^1] A second formula here would be one fact in two
/// places.[^2]
///
/// # References
///
/// [^1]: ADR-0023, an aggregate combines exactly, in any order, decision D1. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
/// [^2]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
fn height_spread(total: CellTotals) -> Fix32 {
    spread_of(
        Accum(total.height_total),
        Accum(total.height_square_total),
        total.observed,
    )
    .unwrap_or(Fix32::ZERO)
}

/// Declares the channel table of one cell of the ring stack.
///
/// **The table is the one place that pairs a channel name with a channel
/// position.** The macro takes one name and one expression for each channel,
/// so a name and the value it describes cannot drift apart. A second list of
/// names would be a second declaration of the order, and nothing would fail
/// when the two disagreed.[^1]
///
/// The macro takes the names of the five bindings that a channel expression
/// reads. A macro body declares its own bindings, and a caller expression
/// cannot see them, so the caller names them instead.
///
/// # References
///
/// [^1]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
macro_rules! declare_ring_channels {
    (
        $total:ident, $extent:ident, $cell:ident, $in_world:ident, $observed:ident,
        { $( $name:literal => $value:expr ; )* }
    ) => {
        /// The channels of one cell, in the order the block stores them.
        ///
        /// The schema publishes this list, so a reader of the observation
        /// names a channel rather than counting positions.
        pub const RING_STACK_CHANNEL_NAMES: [&str; RING_STACK_CHANNELS as usize] =
            [ $( $name, )* ];

        /// Returns every channel of one cell of the ring stack.
        ///
        /// Every channel is a share, a signed relation or a compressed
        /// magnitude, and the design publishes no raw count and no unbounded
        /// total.[^1] Each one goes through the arithmetic boundary of the
        /// project.[^2]
        ///
        /// **Every channel here names a source that the near pass and the
        /// far pass both reach.** A channel of the literal zero, or a
        /// channel that only one pass fills, publishes a real bound over a
        /// constant, and a reward term then reads a constant as truth.[^3]
        ///
        /// # References
        ///
        /// [^1]: Report 42, what a policy should be able to see, section 8.1. `docs/research/reports/42-what-a-policy-should-be-able-to-see.md`
        /// [^2]: ADR-0002, simulated and aggregated state holds no floating point number, decision D2. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
        /// [^3]: What a policy cannot see, section 2.3. `docs/research/what-a-policy-cannot-see.md`
        fn channels_of(
            $total: CellTotals,
            $extent: &CellExtent,
            $cell: u32,
        ) -> [Fix32; RING_STACK_CHANNELS as usize] {
            let $in_world = $extent.in_world_tiles($cell);
            let $observed = $total.observed;
            [ $( $value, )* ]
        }
    };
}

declare_ring_channels! {
    total, extent, cell, in_world, observed,
    {
        "area_inside_world" => sim_math::bounded_share(extent.inside(cell), extent.sampled(cell));
        "observed_share" => sim_math::bounded_share(observed, in_world);
        "seen_now_share" => sim_math::bounded_share(total.seen_now, in_world);
        "memory_age" => sim_math::compressed_magnitude(fixed_mean(total.memory_age_total, observed));
        "open_share" => sim_math::bounded_share(total.open, observed);
        "water_share" => sim_math::bounded_share(observed - total.open, observed);
        "mean_height" => mean_share(total.height_total, observed);
        "height_spread" => height_spread(total);
        "food_density" => sim_math::compressed_magnitude(density(total.food_total, observed));
        "ground_water_density" => sim_math::compressed_magnitude(density(total.ground_water_total, observed));
        "value_density" => sim_math::compressed_magnitude(density(total.value_total, observed));
        "resource_share" => sim_math::bounded_share(total.resource_tiles, observed);
        "own_held_share" => sim_math::bounded_share(total.own_held, observed);
        "rival_held_share" => sim_math::bounded_share(total.rival_held, observed);
        "unclaimed_open_share" => sim_math::bounded_share(total.unclaimed_open, total.open);
        "own_unit_density" => sim_math::compressed_magnitude(density(total.own_units, observed));
        "rival_unit_density" => sim_math::compressed_magnitude(density(total.rival_units, observed));
        "own_settlements" => sim_math::compressed_magnitude(total.own_settlements);
        "rival_settlements" => sim_math::compressed_magnitude(total.rival_settlements);
        "own_upgrades" => sim_math::compressed_magnitude(total.own_upgrades);
        "rival_upgrades" => sim_math::compressed_magnitude(total.rival_upgrades);
        "own_reach_share" => sim_math::bounded_share(total.own_held, in_world);
        "hazard_share" => sim_math::bounded_share(total.hazard_tiles, observed);
        "own_strength_density" => sim_math::compressed_magnitude(fixed_mean(total.own_strength, observed));
        "rival_strength_density" => sim_math::compressed_magnitude(fixed_mean(total.rival_strength, observed));
    }
}
