//! The entity token block of the observation.
//!
//! An aggregate loses the joint structure of a set. A policy cannot tell from
//! a mean and a maximum whether the strongest rival is also the nearest one.
//! The block therefore publishes four fixed-size token sets beside the
//! aggregates, and each token holds the channels of one subject.[^1]
//!
//! # A token slot carries no identity
//!
//! **A token position must never mean one particular settlement, one
//! particular rival or one particular place.** A league seats one policy in
//! one seat for one game and in another seat for the next, so a policy that
//! learned a slot number reads another subject under the same weight.[^2]
//!
//! The block therefore orders each set by a game quantity and never by an
//! identity. Position `k` of a set means the `k`th subject on that quantity,
//! and it means that in every world and in every game. An identity enters the
//! order only as the last tie-break, and it enters only to fix the published
//! bytes so that two runs agree.
//!
//! The design requires the policy to consume each set through a
//! permutation-invariant encoder, so the order does not change the output of
//! the policy. The order exists to make the bytes deterministic.[^1]
//!
//! The quantity of each set is stated at the function that selects it. A
//! missing token holds zero in every channel, and the first channel of every
//! token is the validity flag, so a reader tells a missing token from an
//! empty one.
//!
//! # The selection is bounded
//!
//! Each set scans one candidate list once and keeps a fixed number of
//! entries. The settlement set scans the settlement arena. The rival set
//! scans the same arena and folds it by faction. The threat set scans the
//! blocks that the faction has observed, and never the units, because a
//! faction with hundreds of thousands of rival units in view must not walk
//! them. The site set reads the candidates that the frontier block already
//! surveyed.
//!
//! Each token then reads a fixed disc of level 0 tiles around its subject. A
//! disc of radius 8 holds 217 tiles, and the token count is fixed, so that
//! read is a constant and it follows nothing about the world.
//!
//! # The channels this block cannot fill
//!
//! The engine holds no military strength quantity, no per-settlement
//! population, no per-settlement age, no per-settlement upgrade count and no
//! last-seen tick. Every channel that needs one of those reads zero, and the
//! function that writes each token names the channels it reserves. The block
//! publishes no number that no reader defines.
//!
//! # References
//!
//! [^1]: Report 42, what a policy should be able to see, sections 6.4 and 9.9. `docs/research/reports/42-what-a-policy-should-be-able-to-see.md`
//! [^2]: Findings register, FND-647. `docs/FINDINGS.md`

use crate::faction_view::FactionViewError;
use crate::hex::{Axial, Grid};
use crate::obs_frontier::Frontier;
use crate::obs_ring::{
    hex_distance_from_origin, ring_of_distance, ring_position, shared_sector_of, FAR_SECTORS,
    RING_CAP,
};
use crate::obs_ring_stack::RingStack;
use crate::sim_math;
use crate::types::{Entity, FactionId, Fix32};
use crate::world::World;

/// The settlement tokens the block holds.
pub const SETTLEMENT_TOKENS: u32 = 8;

/// The channels of one settlement token.
pub const SETTLEMENT_CHANNELS: u32 = 24;

/// The rival tokens the block holds.
///
/// A faction that ranks below sixth on threat does not drive a decision, and
/// the order statistics of the layout cover the whole field, so six loses
/// nothing.
pub const RIVAL_TOKENS: u32 = 6;

/// The channels of one rival token.
pub const RIVAL_CHANNELS: u32 = 24;

/// The threat cluster tokens the block holds.
pub const THREAT_TOKENS: u32 = 8;

/// The channels of one threat cluster token.
pub const THREAT_CHANNELS: u32 = 20;

/// The candidate site tokens the block holds.
pub const SITE_TOKENS: u32 = 8;

/// The channels of one candidate site token.
pub const SITE_CHANNELS: u32 = 16;

/// The positions that the entity token block holds.
pub const TOKEN_SLOTS: u32 = SETTLEMENT_TOKENS * SETTLEMENT_CHANNELS
    + RIVAL_TOKENS * RIVAL_CHANNELS
    + THREAT_TOKENS * THREAT_CHANNELS
    + SITE_TOKENS * SITE_CHANNELS;

/// The radius of the disc that a token reads around its subject.
///
/// A disc of this radius holds 217 tiles. The token count is fixed, so the
/// whole of the disc reading is a constant cost.
const DISC_RADIUS: u32 = 8;

/// The radius of the near disc that a candidate site token reads.
const SITE_DISC_RADIUS: u32 = 2;

/// The entity token block of one faction.
#[derive(Clone, Debug)]
pub struct EntityTokens {
    slots: Vec<i64>,
    disc_tiles: i64,
    scanned_settlements: i64,
    scanned_blocks: i64,
}

impl EntityTokens {
    /// Returns every position of the block, in the order the layout states.
    #[must_use]
    pub fn slots(&self) -> &[i64] {
        &self.slots
    }

    /// Returns the level 0 tiles the token discs read.
    ///
    /// A test reads this to assert that the disc cost is a constant.
    #[must_use]
    pub const fn disc_tiles(&self) -> i64 {
        self.disc_tiles
    }

    /// Returns the settlements the selection scanned.
    #[must_use]
    pub const fn scanned_settlements(&self) -> i64 {
        self.scanned_settlements
    }

    /// Returns the observed blocks the threat selection scanned.
    #[must_use]
    pub const fn scanned_blocks(&self) -> i64 {
        self.scanned_blocks
    }
}

/// What one disc of level 0 tiles around a place holds.
#[derive(Clone, Copy, Debug, Default)]
struct Disc {
    observed: i64,
    passable: i64,
    unclaimed_passable: i64,
    own_units: i64,
    rival_units: i64,
    own_upgrades: i64,
    hazard_tiles: i64,
    resource_tiles: i64,
    food_total: i64,
    water_total: i64,
    value_total: i64,
}

/// One settlement that the selection kept, with the quantity it ranked on.
#[derive(Clone, Copy, Debug)]
struct SettlementEntry {
    settlement: Entity,
    address: Axial,
    store: i64,
    tile: u32,
}

/// One rival that the selection kept, with the quantity it ranked on.
#[derive(Clone, Copy, Debug)]
struct RivalEntry {
    faction: FactionId,
    seen_settlements: i64,
    nearest: u32,
}

/// One observed block that the threat selection kept.
#[derive(Clone, Copy, Debug)]
struct ThreatEntry {
    address: Axial,
    rival_units: i64,
    observed: i64,
    block: u32,
}

impl World {
    /// Builds the entity token block of the observation of one faction.
    ///
    /// The block reads the ring stack for the centre of the egocentric frame,
    /// and it reads the frontier block for the candidate sites that the
    /// founding survey already scored. Passing both in keeps one survey and
    /// one centre for the whole observation.
    ///
    /// # Errors
    ///
    /// Returns an error when the number names no faction of this world, and
    /// when the derived unit structure does not describe the units.
    pub fn faction_entity_tokens(
        &self,
        faction: FactionId,
        stack: &RingStack,
        frontier: &Frontier,
    ) -> Result<EntityTokens, FactionViewError> {
        if self.standing(faction).is_none() {
            return Err(FactionViewError::NoSuchFaction(faction));
        }
        let centre = stack.centre();
        let mut tokens = EntityTokens {
            slots: vec![0i64; TOKEN_SLOTS as usize],
            disc_tiles: 0,
            scanned_settlements: 0,
            scanned_blocks: 0,
        };

        let settlements = self.select_settlements(faction, &mut tokens);
        let rivals = self.select_rivals(faction, &mut tokens);
        let threats = self.select_threats(faction, &mut tokens)?;

        let mut at = 0usize;
        for slot in 0..SETTLEMENT_TOKENS as usize {
            let channels = match settlements.get(slot) {
                Some(entry) => self.settlement_token(faction, centre, *entry, &mut tokens),
                None => [Fix32::ZERO; SETTLEMENT_CHANNELS as usize],
            };
            write_token(&mut tokens.slots, &mut at, &channels);
        }
        for slot in 0..RIVAL_TOKENS as usize {
            let channels = match rivals.get(slot) {
                Some(entry) => self.rival_token(faction, *entry, &settlements),
                None => [Fix32::ZERO; RIVAL_CHANNELS as usize],
            };
            write_token(&mut tokens.slots, &mut at, &channels);
        }
        for slot in 0..THREAT_TOKENS as usize {
            let channels = match threats.get(slot) {
                Some(entry) => {
                    self.threat_token(faction, centre, *entry, &settlements, &mut tokens)
                }
                None => [Fix32::ZERO; THREAT_CHANNELS as usize],
            };
            write_token(&mut tokens.slots, &mut at, &channels);
        }
        let sites = frontier.best_sites();
        for slot in 0..SITE_TOKENS as usize {
            let channels = match sites.get(slot) {
                Some(site) => self.site_token(faction, centre, *site, &settlements, &mut tokens),
                None => [Fix32::ZERO; SITE_CHANNELS as usize],
            };
            write_token(&mut tokens.slots, &mut at, &channels);
        }
        Ok(tokens)
    }

    /// Selects the settlement tokens of one faction.
    ///
    /// **The quantity is the store of the settlement, descending.** The store
    /// is what a settlement can spend, so the largest store is the settlement
    /// whose choices matter most. The tie-break is the ascending tile index,
    /// and it exists only to fix the published bytes.
    ///
    /// The scan reads the settlement arena in ascending slot order and keeps
    /// eight entries, so its cost follows the settlement count.
    fn select_settlements(
        &self,
        faction: FactionId,
        tokens: &mut EntityTokens,
    ) -> Vec<SettlementEntry> {
        let arena = self.settlements();
        let mut kept: Vec<SettlementEntry> = Vec::new();
        for settlement in arena.iter() {
            tokens.scanned_settlements += 1;
            if self.settlement_faction(settlement) != Some(faction) {
                continue;
            }
            let Some(address) = arena.address(settlement) else {
                continue;
            };
            let Some(tile) = self.grid().index_of(address) else {
                continue;
            };
            kept.push(SettlementEntry {
                settlement,
                address,
                store: self.settlement_store_total(settlement),
                tile: tile.0,
            });
        }
        kept.sort_unstable_by(|left, right| {
            right
                .store
                .cmp(&left.store)
                .then(left.tile.cmp(&right.tile))
        });
        kept.truncate(SETTLEMENT_TOKENS as usize);
        kept
    }

    /// Returns the whole store of one settlement.
    ///
    /// The store holds one quantity for each commodity class, so the total is
    /// the sum over the classes. The accumulator is 64 bits wide.
    fn settlement_store_total(&self, settlement: Entity) -> i64 {
        let mut total = 0i64;
        for commodity in 0..crate::site::COMMODITY_COUNT {
            let commodity = crate::site::CommodityId(commodity as u16);
            if let Some(quantity) = self.settlement_store(settlement, commodity) {
                total = total.saturating_add(i64::from(quantity.0));
            }
        }
        total
    }

    /// Selects the rival tokens of one faction.
    ///
    /// **The quantity is the settlements of the rival that the reader can
    /// see, descending.** A rival whose cities the reader can see is a rival
    /// the reader can act against, and the count is the only power estimate
    /// the fog admits without walking the units of the rival. The tie-breaks
    /// are the ascending nearest distance and then the ascending seat number,
    /// and the seat number enters only to fix the published bytes.
    ///
    /// The design asks for a threat sort over remembered rival military
    /// strength. The engine holds no strength quantity, so this order stands
    /// in for it and this paragraph says so.
    ///
    /// **A rival the reader has observed nothing of gets no token.** The seat
    /// exists, and the reader learns that from the seated faction count. A
    /// token for a rival the reader has never met would carry zero in every
    /// channel and a validity flag that said otherwise, and a reader could
    /// then not tell a missing subject from an empty one.
    fn select_rivals(&self, faction: FactionId, tokens: &mut EntityTokens) -> Vec<RivalEntry> {
        let arena = self.settlements();
        let count = usize::from(self.faction_count());
        let mut seen = vec![0i64; count];
        let mut nearest = vec![u32::MAX; count];
        for settlement in arena.iter() {
            tokens.scanned_settlements += 1;
            let Some(owner) = self.settlement_faction(settlement) else {
                continue;
            };
            if owner == faction {
                continue;
            }
            let Some(address) = arena.address(settlement) else {
                continue;
            };
            if !self.faction_sees_now(faction, address) {
                continue;
            }
            let slot = usize::from(owner.0);
            if let Some(place) = seen.get_mut(slot) {
                *place += 1;
            }
            if let Some(place) = nearest.get_mut(slot) {
                *place = (*place).min(self.nearest_own_settlement(faction, address));
            }
        }
        let mut kept: Vec<RivalEntry> = (0..count)
            .filter(|slot| *slot != usize::from(faction.0))
            .filter(|slot| seen.get(*slot).copied().unwrap_or(0) > 0)
            .map(|slot| RivalEntry {
                faction: FactionId(slot as u16),
                seen_settlements: seen.get(slot).copied().unwrap_or(0),
                nearest: nearest.get(slot).copied().unwrap_or(u32::MAX),
            })
            .collect();
        kept.sort_unstable_by(|left, right| {
            right
                .seen_settlements
                .cmp(&left.seen_settlements)
                .then(left.nearest.cmp(&right.nearest))
                .then(left.faction.0.cmp(&right.faction.0))
        });
        kept.truncate(RIVAL_TOKENS as usize);
        kept
    }

    /// Selects the threat cluster tokens of one faction.
    ///
    /// **The quantity is the rival units the block holds, descending.** The
    /// candidate list is the blocks the faction has observed, and never the
    /// units, because a faction with hundreds of thousands of rival units in
    /// view must not walk them. The tie-break is the ascending block index,
    /// and it exists only to fix the published bytes.
    ///
    /// The design asks for remembered rival military strength. The engine
    /// holds no strength quantity, so the unit count stands in for it.
    ///
    /// **The candidate list is the ground the faction sees this frame, and
    /// not the ground it remembers.** A remembered army is a place an army
    /// stood at some earlier tick, and the engine holds no last-seen tick, so
    /// it cannot say how stale that sighting is. A threat the reader cannot
    /// see now therefore gets no token.
    fn select_threats(
        &self,
        faction: FactionId,
        tokens: &mut EntityTokens,
    ) -> Result<Vec<ThreatEntry>, FactionViewError> {
        use crate::faction_view::{Admit, BlockMask};
        let observation = self.observation();
        let layout = observation.layout();
        let visible = observation.visible_layer(faction);
        let remembered = observation.remembered_layer(faction);
        let edge = layout.block_edge() as i32;
        let wide = layout.blocks_wide().max(1);
        let mut kept: Vec<ThreatEntry> = Vec::new();
        let blocks = visible.map_or(&[][..], |layer| layer.populated_blocks());
        for block in blocks {
            tokens.scanned_blocks += 1;
            let mask = BlockMask::new(
                visible.and_then(|layer| layer.block(*block)),
                remembered.and_then(|layer| layer.block(*block)),
            );
            let masked = self.masked_block(&mask, faction, *block, Admit::SeenNow)?;
            let rival_units = masked.other_units();
            if rival_units == 0 {
                continue;
            }
            let column = (*block % wide) as i32;
            let row = (*block / wide) as i32;
            kept.push(ThreatEntry {
                address: Axial::new(column * edge + edge / 2, row * edge + edge / 2),
                rival_units,
                observed: masked.admitted(),
                block: *block,
            });
        }
        kept.sort_unstable_by(|left, right| {
            right
                .rival_units
                .cmp(&left.rival_units)
                .then(left.block.cmp(&right.block))
        });
        kept.truncate(THREAT_TOKENS as usize);
        Ok(kept)
    }

    /// Reads the disc of level 0 tiles around one place.
    ///
    /// The disc holds every tile within the radius, and the radius is fixed,
    /// so the read is a constant. The pass visits the tiles in ascending hex
    /// distance and then in ascending position around each ring.
    ///
    /// Every quantity is fogged, except the own held ground and the own
    /// upgrades. A faction is not fogged from its own ground.[^1]
    ///
    /// # References
    ///
    /// [^1]: Findings register, FND-671. `docs/FINDINGS.md`
    fn read_disc(
        &self,
        faction: FactionId,
        place: Axial,
        radius: u32,
        tokens: &mut EntityTokens,
    ) -> Disc {
        let grid = self.grid();
        let mut disc = Disc::default();
        for distance in 0..=radius {
            let positions = if distance == 0 { 1 } else { 6 * distance };
            for step in 0..positions {
                let address = place.add(ring_position(distance, step));
                if !grid.contains(address) {
                    continue;
                }
                tokens.disc_tiles += 1;
                let Some(tile) = self.faction_tile(faction, address) else {
                    continue;
                };
                let Some(ground) = tile.ground() else {
                    continue;
                };
                disc.observed += 1;
                disc.food_total += i64::from(ground.generated[0].0);
                if ground.kind.is_passable() {
                    disc.passable += 1;
                }
                if ground.generated.iter().any(|amount| amount.0 > 0) {
                    disc.resource_tiles += 1;
                }
                if let Some(water) = self.ground_water_at(address) {
                    disc.water_total += water;
                }
                if let Some(value) = self.tile_value(address) {
                    disc.value_total += i64::from(value.0);
                }
                if self.tile_is_burning(address) == Some(true) {
                    disc.hazard_tiles += 1;
                }
                let own_ground = self.holds(faction, address) == Some(true);
                if own_ground {
                    if let Some(site) = self.upgrade_at(address) {
                        if site.level > 0 {
                            disc.own_upgrades += 1;
                        }
                    }
                }
                if !own_ground
                    && self
                        .tile_holder(address)
                        .is_some_and(|held| held.is_nobody())
                    && ground.kind.is_passable()
                {
                    disc.unclaimed_passable += 1;
                }
                if let Ok(units) = self.soldiers_on(address) {
                    for unit in units {
                        match self.soldiers().faction(*unit) {
                            Some(owner) if owner == faction => disc.own_units += 1,
                            Some(_) => disc.rival_units += 1,
                            None => {}
                        }
                    }
                }
            }
        }
        disc
    }

    /// Returns the hex distance to the nearest settlement of one faction.
    ///
    /// The scan reads the settlement arena in ascending slot order, so its
    /// cost follows the settlement count. A faction with no settlement gives
    /// the widest distance the type holds.
    fn nearest_own_settlement(&self, faction: FactionId, from: Axial) -> u32 {
        let arena = self.settlements();
        let mut nearest = u32::MAX;
        for settlement in arena.iter() {
            if self.settlement_faction(settlement) != Some(faction) {
                continue;
            }
            if let Some(address) = arena.address(settlement) {
                nearest = nearest.min(from.distance(address));
            }
        }
        nearest
    }

    /// Returns the hex distance to the nearest rival settlement the reader
    /// sees.
    ///
    /// A rival settlement on ground the reader does not see this frame does
    /// not count, because the reader does not know it is there.
    fn nearest_seen_rival_settlement(&self, faction: FactionId, from: Axial) -> u32 {
        let arena = self.settlements();
        let mut nearest = u32::MAX;
        for settlement in arena.iter() {
            let Some(owner) = self.settlement_faction(settlement) else {
                continue;
            };
            if owner == faction {
                continue;
            }
            if let Some(address) = arena.address(settlement) {
                if self.faction_sees_now(faction, address) {
                    nearest = nearest.min(from.distance(address));
                }
            }
        }
        nearest
    }

    /// Writes the 24 channels of one settlement token.
    ///
    /// Fourteen channels of the design have no source in the engine and read
    /// zero. They are the population share, the population, the finished
    /// upgrade count of the settlement, the upgrades under construction, the
    /// garrison share, the garrison strength, the food coverage in ticks, the
    /// distance to the reach boundary, the rival strength within the disc,
    /// the own strength within the disc, the tiles lost over the window, the
    /// population change over the window, the wonder progress at the
    /// settlement and the settlement age. The engine holds no per-settlement
    /// population, no per-settlement upgrade count, no per-settlement age, no
    /// window history and no military strength.
    fn settlement_token(
        &self,
        faction: FactionId,
        centre: Axial,
        entry: SettlementEntry,
        tokens: &mut EntityTokens,
    ) -> [Fix32; SETTLEMENT_CHANNELS as usize] {
        let grid = self.grid();
        let delta = Axial::new(entry.address.q - centre.q, entry.address.r - centre.r);
        let distance = hex_distance_from_origin(delta);
        let disc = self.read_disc(faction, entry.address, DISC_RADIUS, tokens);
        let own_total = self
            .standing(faction)
            .map_or(0, |standing| standing.store_total);
        let (first, second) = sector_pair(delta);
        let mut channels = [Fix32::ZERO; SETTLEMENT_CHANNELS as usize];
        channels[0] = Fix32::ONE;
        channels[1] =
            sim_math::bounded_share(i64::from(ring_of_distance(distance)), i64::from(RING_CAP));
        channels[2] = first;
        channels[3] = second;
        channels[4] = sim_math::compressed_magnitude(i64::from(distance));
        channels[9] = sim_math::bounded_share(entry.store, own_total);
        channels[10] = sim_math::compressed_magnitude(entry.store);
        channels[11] = sim_math::compressed_magnitude(disc.own_upgrades);
        channels[14] = sim_math::bounded_share(disc.hazard_tiles, disc.observed);
        channels[15] = distance_share(
            grid,
            self.nearest_seen_rival_settlement(faction, entry.address),
        );
        channels[17] = sim_math::compressed_magnitude(disc.rival_units);
        channels[18] = sim_math::compressed_magnitude(disc.own_units);
        channels[19] = sim_math::bounded_share(disc.unclaimed_passable, disc.passable);
        channels[22] = siege_share(self.siege_of(entry.settlement));
        channels
    }

    /// Writes the 24 channels of one rival token.
    ///
    /// The design gives twelve relative ratios, one for each power quantity.
    /// The fog admits an estimate of two of them, which are the settlements
    /// the reader can see and the ground the reader can see the rival hold.
    /// The other ten read zero, because publishing an unfogged total would
    /// let the reader read a quantity it has not observed.
    ///
    /// The war flag, the shared border length, the trade volume, the power
    /// trend, the observation confidence, the unit mix distance and the
    /// relation to the leader read zero. The engine holds no war state, no
    /// per-rival border length, no per-rival trade volume and no window
    /// history. The confidence would need the total the reader does not
    /// know, which is how many settlements the rival holds in all.
    fn rival_token(
        &self,
        faction: FactionId,
        entry: RivalEntry,
        own: &[SettlementEntry],
    ) -> [Fix32; RIVAL_CHANNELS as usize] {
        let grid = self.grid();
        let mut channels = [Fix32::ZERO; RIVAL_CHANNELS as usize];
        channels[0] = Fix32::ONE;
        channels[1] = sim_math::signed_relation(entry.seen_settlements, own.len() as i64);
        channels[13] = Fix32(self.relation(faction, entry.faction).unwrap_or(0));
        channels[14] = Fix32(self.relation(entry.faction, faction).unwrap_or(0));
        channels[17] = distance_share(grid, entry.nearest);
        channels
    }

    /// Writes the 20 channels of one threat cluster token.
    ///
    /// The strength, the strength share, the largest unit type class, the
    /// closing rate, the sighting staleness, the owner reach flag, the owner
    /// power share, the owner relation and the own strength read zero. The
    /// engine holds no military strength, no last-seen tick and no window
    /// history, and a summary cell names no owning faction.
    fn threat_token(
        &self,
        faction: FactionId,
        centre: Axial,
        entry: ThreatEntry,
        own: &[SettlementEntry],
        tokens: &mut EntityTokens,
    ) -> [Fix32; THREAT_CHANNELS as usize] {
        let grid = self.grid();
        let delta = Axial::new(entry.address.q - centre.q, entry.address.r - centre.r);
        let distance = hex_distance_from_origin(delta);
        let disc = self.read_disc(faction, entry.address, DISC_RADIUS, tokens);
        let (first, second) = sector_pair(delta);
        let mut channels = [Fix32::ZERO; THREAT_CHANNELS as usize];
        channels[0] = Fix32::ONE;
        channels[1] =
            sim_math::bounded_share(i64::from(ring_of_distance(distance)), i64::from(RING_CAP));
        channels[2] = first;
        channels[3] = second;
        channels[4] = sim_math::compressed_magnitude(i64::from(distance));
        channels[7] = sim_math::compressed_magnitude(entry.rival_units);
        channels[9] = distance_share(grid, nearest_of(own, entry.address));
        channels[12] = flag(self.holds(faction, entry.address) == Some(true));
        channels[16] =
            sim_math::bounded_share(disc.hazard_tiles, entry.observed.max(disc.observed));
        channels[17] = sim_math::bounded_share(disc.passable, disc.observed);
        channels[19] = sim_math::signed_relation(disc.own_units, disc.rival_units);
        channels
    }

    /// Writes the 16 channels of one candidate site token.
    ///
    /// The rival strength channel reads zero, because the engine holds no
    /// military strength quantity.
    fn site_token(
        &self,
        faction: FactionId,
        centre: Axial,
        site: SiteEntry,
        own: &[SettlementEntry],
        tokens: &mut EntityTokens,
    ) -> [Fix32; SITE_CHANNELS as usize] {
        let grid = self.grid();
        let delta = Axial::new(site.address.q - centre.q, site.address.r - centre.r);
        let distance = hex_distance_from_origin(delta);
        let disc = self.read_disc(faction, site.address, SITE_DISC_RADIUS, tokens);
        let (first, second) = sector_pair(delta);
        let mut channels = [Fix32::ZERO; SITE_CHANNELS as usize];
        channels[0] = Fix32::ONE;
        channels[1] =
            sim_math::bounded_share(i64::from(ring_of_distance(distance)), i64::from(RING_CAP));
        channels[2] = first;
        channels[3] = second;
        channels[4] = sim_math::compressed_magnitude(i64::from(distance));
        channels[5] = sim_math::compressed_magnitude(mean(disc.food_total, disc.observed));
        channels[6] = sim_math::compressed_magnitude(mean(disc.water_total, disc.observed));
        channels[7] = sim_math::compressed_magnitude(mean(disc.value_total, disc.observed));
        channels[8] = sim_math::bounded_share(disc.resource_tiles, disc.observed);
        channels[9] = sim_math::bounded_share(disc.passable, disc.observed);
        channels[10] = flag(self.holds(faction, site.address) == Some(true));
        channels[11] = distance_share(grid, nearest_of(own, site.address));
        channels[12] = distance_share(
            grid,
            self.nearest_seen_rival_settlement(faction, site.address),
        );
        channels[14] = sim_math::bounded_share(disc.hazard_tiles, disc.observed);
        channels[15] = sim_math::compressed_magnitude(site.score);
        channels
    }
}

/// One candidate site that the frontier block surveyed.
#[derive(Clone, Copy, Debug)]
pub struct SiteEntry {
    /// The address of the place.
    pub address: Axial,
    /// The score the founding survey gave the place.
    pub score: i64,
}

/// Returns the hex distance to the nearest of a set of settlements.
fn nearest_of(own: &[SettlementEntry], from: Axial) -> u32 {
    own.iter()
        .map(|entry| from.distance(entry.address))
        .min()
        .unwrap_or(u32::MAX)
}

/// Returns a hex distance as a share of the widest distance in the world.
///
/// The widest hex distance on an axial grid of one width and one height is
/// the width and the height less two. That is a structural property of the
/// world, so it is a legal denominator.
///
/// A distance of the widest value the type holds means that no subject
/// exists, and it reads as one.
fn distance_share(grid: Grid, distance: u32) -> Fix32 {
    let widest = i64::from(grid.width()) + i64::from(grid.height()) - 2;
    if distance == u32::MAX {
        return Fix32::ONE;
    }
    sim_math::bounded_share(i64::from(distance), widest)
}

/// Returns the sector of a delta as a pair of triangle waves.
///
/// One triangle wave over a cycle of twelve names two sectors at every value,
/// because the wave rises and falls. A second wave three sectors out of step
/// separates them, so the pair names one sector. The pair is also continuous
/// across the wrap, which a raw sector number is not.
fn sector_pair(delta: Axial) -> (Fix32, Fix32) {
    let sector = i64::from(shared_sector_of(delta));
    let period = i64::from(FAR_SECTORS);
    (
        sim_math::phase_triangle(sector, period),
        sim_math::phase_triangle(sector + period / 4, period),
    )
}

/// Returns a flag as one position of a token.
fn flag(set: bool) -> Fix32 {
    if set {
        Fix32::ONE
    } else {
        Fix32::ZERO
    }
}

/// Returns the siege of a settlement as a share.
///
/// A settlement under no siege reads zero. A settlement under siege reads the
/// compressed magnitude of the work the besieger has done.
fn siege_share(siege: Option<(FactionId, i64)>) -> Fix32 {
    match siege {
        Some((_, work)) => sim_math::compressed_magnitude(work),
        None => Fix32::ZERO,
    }
}

/// Returns a mean, or zero when the count is zero.
fn mean(total: i64, count: i64) -> i64 {
    if count < 1 {
        return 0;
    }
    total / count
}

/// Writes one token into the block and advances the cursor.
fn write_token(slots: &mut [i64], at: &mut usize, channels: &[Fix32]) {
    for value in channels {
        if let Some(place) = slots.get_mut(*at) {
            *place = i64::from(value.0);
        }
        *at += 1;
    }
}
