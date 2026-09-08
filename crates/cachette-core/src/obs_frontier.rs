//! The frontier block of the observation, by sector.
//!
//! The block answers two questions that a strategy policy asks on every
//! decision. Where should the faction expand, and where is it under threat.
//! It answers both on the twelve-sector axis of the egocentric ring frame, so
//! a direction in this block names the same ground as the same direction in
//! the ring stack.[^1]
//!
//! # The perimeter is the subject
//!
//! The pass walks the tiles the faction holds and keeps the ones that touch
//! ground the faction does not hold. That set is the perimeter. A perimeter
//! tile that touches unclaimed passable ground is a frontier tile, and a
//! perimeter tile that touches ground a rival holds is a contested tile.
//!
//! The perimeter is where a faction can grow and where it can be attacked, so
//! it is the right subject. The cost of the walk follows the ground somebody
//! holds and not the ground the world holds.[^2]
//!
//! # A faction is not fogged from its own border
//!
//! The pass reads the holding directly. A faction that read its own border
//! through the fog of the present frame would watch that border move as its
//! units move, because a tile it holds and does not watch reports no
//! holder.[^3] The unclaimed ground beyond the border is fogged in the normal
//! way, because the faction learns about that ground by looking at it.
//!
//! # The expansion score is the score the engine already computes
//!
//! The pass collects the unclaimed passable ground beside the perimeter and
//! asks the founding survey to score it. That survey is the scorer the engine
//! uses when it chooses where to settle, so the policy reads the same number
//! the engine acts on. The pass computes no score of its own, because a
//! second scorer is a second declaration site and nothing would fail when the
//! two disagreed.[^4]
//!
//! # The threat pressure reads the ring stack
//!
//! The threat pressure of a sector is the rival unit presence in that sector,
//! weighted by the inverse of the distance of the ring it stands in. The ring
//! stack already published that presence for each cell, so this block reads
//! the published channel rather than accumulating a second time.
//!
//! The design asks for remembered rival military strength. The engine holds
//! an attack column and an armour column for each unit type, and it holds no
//! strength quantity and no record that defines one, so the pass uses the
//! rival unit presence instead and this paragraph says so.
//!
//! # The positions this block cannot fill
//!
//! One position of the design reads zero. It is the reach headroom in tiles,
//! which is the ground a faction could still claim if it finished the
//! upgrades it has under way. The engine turns a finished upgrade count into
//! a reach radius, and it holds no per-settlement upgrade count, so the pass
//! cannot say what the headroom is.
//!
//! # References
//!
//! [^1]: Report 42, what a policy should be able to see, section 9.6. `docs/research/reports/42-what-a-policy-should-be-able-to-see.md`
//! [^2]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D4. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
//! [^3]: Findings register, FND-671. `docs/FINDINGS.md`
//! [^4]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`

use crate::faction_view::FactionViewError;
use crate::hex::Axial;
use crate::obs_ring::{
    ring_band, ring_of_cell, sector_of_cell, shared_sector_of, FAR_SECTORS, RING_STACK_CELLS,
};
use crate::obs_ring_stack::{RingStack, RIVAL_PRESENCE_CHANNEL};
use crate::obs_token::{SiteEntry, SITE_TOKENS};
use crate::sim_math;
use crate::types::FactionId;
use crate::world::World;

/// The positions that the frontier block holds.
pub const FRONTIER_SLOTS: u32 = 2 * FAR_SECTORS + 8;

/// The reach headroom position, which the engine holds no source for.
///
/// The module documentation states why. The position stays in the layout,
/// because a later engine that counts the upgrades of a settlement can fill
/// it without moving any other position.
const RESERVED_REACH_HEADROOM: i64 = 0;

/// The group size that the pass asks the founding survey to place.
///
/// The score of a place is a property of the ground and does not follow the
/// group. The group only decides whether the place has room, which the pass
/// reads as the eligibility of the candidate. One person is therefore the
/// right question: it asks whether the ground could hold a settlement at all.
const SURVEY_GROUP: u32 = 1;

/// What the perimeter of one faction holds, by sector and in total.
#[derive(Clone, Debug, Default)]
struct Perimeter {
    tiles: i64,
    frontier_tiles: i64,
    contested_tiles: i64,
    candidates: Vec<Axial>,
}

/// The frontier block of one faction.
#[derive(Clone, Debug)]
pub struct Frontier {
    slots: Vec<i64>,
    sites: Vec<SiteEntry>,
    perimeter_tiles: i64,
    candidate_addresses: i64,
}

impl Frontier {
    /// Returns every position of the block, in the order the layout states.
    #[must_use]
    pub fn slots(&self) -> &[i64] {
        &self.slots
    }

    /// Returns the tiles of the perimeter that the pass walked.
    ///
    /// A test reads this to assert that the cost follows the ground the
    /// faction holds.
    #[must_use]
    pub const fn perimeter_tiles(&self) -> i64 {
        self.perimeter_tiles
    }

    /// Returns the unclaimed addresses that the pass scored.
    #[must_use]
    pub const fn candidate_addresses(&self) -> i64 {
        self.candidate_addresses
    }

    /// Returns the best candidate sites the survey found, best first.
    ///
    /// The founding survey orders its candidates by eligibility, then by
    /// descending score, then by ascending tile index. That order is total and
    /// it names no faction, so the token set that reads it carries no
    /// identity. The entity token block reads this rather than surveying the
    /// same ground a second time.[^1]
    ///
    /// # References
    ///
    /// [^1]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
    #[must_use]
    pub fn best_sites(&self) -> &[SiteEntry] {
        &self.sites
    }
}

impl World {
    /// Builds the frontier block of the observation of one faction.
    ///
    /// The block reads the ring stack of the same faction, because the threat
    /// pressure is a weighted sum over the rival presence the stack already
    /// published. Passing the stack in keeps one accumulation for both
    /// blocks.
    ///
    /// # Errors
    ///
    /// Returns an error when the number names no faction of this world.
    pub fn faction_frontier(
        &self,
        faction: FactionId,
        stack: &RingStack,
    ) -> Result<Frontier, FactionViewError> {
        if self.standing(faction).is_none() {
            return Err(FactionViewError::NoSuchFaction(faction));
        }
        let centre = stack.centre();
        let perimeter = self.walk_perimeter(faction);
        let mut slots = vec![0i64; FRONTIER_SLOTS as usize];

        let survey = self
            .survey_places(&perimeter.candidates, SURVEY_GROUP, &self.standing_places())
            .ok();
        let mut expansion = [0i64; FAR_SECTORS as usize];
        let mut food_total = 0i64;
        let mut value_total = 0i64;
        let mut eligible = 0i64;
        let mut best = 0i64;
        let mut sites = Vec::new();
        if let Some(survey) = survey.as_ref() {
            for candidate in survey.candidates() {
                let address = candidate.address();
                let sector =
                    shared_sector_of(Axial::new(address.q - centre.q, address.r - centre.r));
                if let Some(place) = expansion.get_mut(sector as usize) {
                    *place = place.saturating_add(candidate.score().0);
                }
                best = best.max(candidate.score().0);
                food_total = food_total.saturating_add(i64::from(candidate.provision().food.0));
                if candidate.is_eligible() {
                    eligible += 1;
                }
                if let Some(value) = self.tile_value(address) {
                    value_total = value_total.saturating_add(i64::from(value.0));
                }
                if sites.len() < SITE_TOKENS as usize {
                    sites.push(SiteEntry {
                        address,
                        score: candidate.score().0,
                    });
                }
            }
        }

        let largest_expansion = expansion.iter().copied().max().unwrap_or(0);
        for (sector, score) in expansion.iter().enumerate() {
            slots[sector] = share(*score, largest_expansion);
        }

        let pressure = sector_pressure(stack);
        let largest_pressure = pressure.iter().copied().max().unwrap_or(0);
        for (sector, force) in pressure.iter().enumerate() {
            slots[FAR_SECTORS as usize + sector] = share(*force, largest_pressure);
        }

        let scored = perimeter.candidates.len() as i64;
        let base = 2 * FAR_SECTORS as usize;
        slots[base] = share(perimeter.frontier_tiles, perimeter.tiles);
        slots[base + 1] = magnitude(mean(food_total, scored));
        slots[base + 2] = magnitude(mean(value_total, scored));
        slots[base + 3] = magnitude(scored);
        slots[base + 4] = magnitude(best);
        slots[base + 5] = magnitude(eligible);
        slots[base + 6] = RESERVED_REACH_HEADROOM;
        slots[base + 7] = share(perimeter.contested_tiles, perimeter.tiles);

        Ok(Frontier {
            slots,
            sites,
            perimeter_tiles: perimeter.tiles,
            candidate_addresses: scored,
        })
    }

    /// Walks the perimeter of the ground one faction holds.
    ///
    /// The pass visits the held tiles in ascending tile order, which is the
    /// order the holding already keeps. For each tile it reads the six
    /// neighbours. A tile with no neighbour outside the holding is interior
    /// and contributes nothing.
    ///
    /// An unclaimed neighbour joins the candidate list only when the faction
    /// has seen it. The faction learns about the ground beyond its border by
    /// looking at it, so that ground is fogged in the normal way.
    ///
    /// The candidate list holds each address once. The pass keeps it in
    /// ascending tile order, so the survey receives one stated order.
    fn walk_perimeter(&self, faction: FactionId) -> Perimeter {
        let grid = self.grid();
        let mut perimeter = Perimeter::default();
        let mut candidates = Vec::new();
        for tile in self.holding().tiles_held_by(faction) {
            let Some(address) = grid.address_of(tile) else {
                continue;
            };
            let mut outside = false;
            let mut frontier = false;
            let mut contested = false;
            for neighbour in grid.neighbours(address).into_iter().flatten() {
                let holder = self.tile_holder(neighbour).and_then(|held| held.faction());
                match holder {
                    Some(other) if other == faction => {}
                    Some(_) => {
                        outside = true;
                        contested = true;
                    }
                    None => {
                        outside = true;
                        if self.faction_has_seen(faction, neighbour)
                            && self
                                .tile_kind(neighbour)
                                .is_some_and(|kind| kind.is_passable())
                        {
                            frontier = true;
                            if let Some(index) = grid.index_of(neighbour) {
                                candidates.push((index.0, neighbour));
                            }
                        }
                    }
                }
            }
            if outside {
                perimeter.tiles += 1;
            }
            if frontier {
                perimeter.frontier_tiles += 1;
            }
            if contested {
                perimeter.contested_tiles += 1;
            }
        }
        candidates.sort_unstable_by_key(|(index, _)| *index);
        candidates.dedup_by_key(|(index, _)| *index);
        perimeter.candidates = candidates.into_iter().map(|(_, address)| address).collect();
        perimeter
    }
}

/// Returns the threat pressure of each sector, from the published ring stack.
///
/// The pressure of a sector is the rival presence of each of its cells,
/// divided by one plus the lowest distance of the ring that cell stands in.
/// A rival at distance 2 therefore weighs about thirty times a rival at
/// distance 64.
///
/// The weight is an integer division and it truncates, which is the rounding
/// rule of the whole observation.
fn sector_pressure(stack: &RingStack) -> [i64; FAR_SECTORS as usize] {
    let mut pressure = [0i64; FAR_SECTORS as usize];
    for cell in 0..RING_STACK_CELLS {
        let presence = stack.channel(cell, RIVAL_PRESENCE_CHANNEL);
        if presence == 0 {
            continue;
        }
        let ring = ring_of_cell(cell);
        let weight = i64::from(ring_band(ring).0) + 1;
        let sector = if ring <= 1 {
            sector_of_cell(cell) * 2
        } else {
            sector_of_cell(cell)
        };
        if let Some(place) = pressure.get_mut((sector % FAR_SECTORS) as usize) {
            *place = place.saturating_add(presence / weight);
        }
    }
    pressure
}

/// Returns a part of a whole as one position of the array.
fn share(part: i64, whole: i64) -> i64 {
    i64::from(sim_math::bounded_share(part, whole).0)
}

/// Returns a compressed magnitude as one position of the array.
fn magnitude(value: i64) -> i64 {
    i64::from(sim_math::compressed_magnitude(value).0)
}

/// Returns a mean, or zero when the count is zero.
fn mean(total: i64, count: i64) -> i64 {
    if count < 1 {
        return 0;
    }
    total / count
}
