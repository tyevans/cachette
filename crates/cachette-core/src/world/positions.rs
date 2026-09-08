//! The positions of a faction, and the units that hold a seat.
//!
//! A position is a named seat at a site. A unit takes it, works it and leaves
//! it. The readers, the preference a faction states and the pass that settles
//! the seats each tick sit together.

use super::errors::StepError;
use super::World;
use crate::controller::FactionRow;
use crate::hex::Axial;
use crate::position::{self, Position, PositionError, PositionTable, SitePreference};
use crate::rates::{RateError, RateSchedule};
use crate::resource::ResourceKind;
use crate::types::{Entity, FactionId, Fix32, TileIdx};

impl World {
    /// Returns the positions of every site.
    #[must_use]
    pub const fn positions(&self) -> &PositionTable {
        &self.positions
    }

    /// Returns the positions that one site holds.
    ///
    /// A dead identity gives `None` rather than the row of the settlement
    /// that now stands in the slot.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0014, entity identity is an index plus a generation, decision D2. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
    #[must_use]
    pub fn site_positions(&self, site: Entity) -> Option<&[Position]> {
        let slot = self.settlements.slot_of(site)?;
        self.positions.row(slot)
    }

    /// Returns what one site wants of each kind of work.
    #[must_use]
    pub fn site_preference(&self, site: Entity) -> Option<SitePreference> {
        let slot = self.settlements.slot_of(site)?;
        self.positions.preference(slot)
    }

    /// Returns the unit that holds one position of one site.
    ///
    /// The call resolves the stored identity against the unit arena. A unit
    /// that died gives `None`, and the unit that took its slot is never the
    /// answer.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0014, entity identity is an index plus a generation, decision D2. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
    #[must_use]
    pub fn position_holder(&self, site: Entity, index: usize) -> Option<Entity> {
        let slot = self.settlements.slot_of(site)?;
        self.positions.occupant(slot, index, &self.soldiers)
    }

    /// Gives one position of one site to one unit.
    ///
    /// This is the setter. It states no rule about who should hold a
    /// position, because the rule that chooses is separate work.[^1]
    ///
    /// # Errors
    ///
    /// Returns an error when the identity names no live settlement, when the
    /// identity names no live unit, when the site holds no position at that
    /// index, and when the unit already holds another position at that site.
    ///
    /// # References
    ///
    /// [^1]: ADR-0065, a group is a site membership, not a region, decision D1. `docs/adrs/draft/adr-0065-a-group-is-a-site-membership-not-a-region.md`
    pub fn seat_in_position(
        &mut self,
        site: Entity,
        index: usize,
        unit: Entity,
    ) -> Result<(), PositionError> {
        let slot = self
            .settlements
            .slot_of(site)
            .ok_or(PositionError::NoSuchSlot(site.index()))?;
        if !self.soldiers.contains(unit) {
            return Err(PositionError::NoSuchSlot(unit.index()));
        }
        self.positions.seat(slot, index, unit)
    }

    /// Changes what a set of sites wants of one kind of work.
    ///
    /// **The command names no unit.** It states what a place wants, and the
    /// rebalance turns that into a number of positions of each kind. A
    /// caller that wanted to name the workers would be looping over
    /// entities, which the control plane never does.[^1]
    ///
    /// **The set is all or nothing.** Every identity resolves, and the
    /// target is checked, before anything is written.[^2]
    ///
    /// # Errors
    ///
    /// Returns an error when an identity names no live settlement, or when
    /// the target is below zero.
    ///
    /// # References
    ///
    /// [^1]: ADR-0040, Python is a control plane, not a data plane, decision D1. `docs/adrs/draft/adr-0040-python-is-a-control-plane-not-a-data-plane.md`
    /// [^2]: ADR-0085, an entity crosses to Python as one opaque identity that the engine resolves, decision D3. `docs/adrs/accepted/adr-0085-an-entity-crosses-to-python-as-one-opaque-identity.md`
    pub fn prefer_at_sites(
        &mut self,
        sites: &[Entity],
        kind: ResourceKind,
        target: Fix32,
    ) -> Result<(), PositionError> {
        if target.0 < 0 {
            return Err(PositionError::TargetBelowZero(target));
        }
        let mut slots = Vec::with_capacity(sites.len());
        for site in sites {
            slots.push(
                self.settlements
                    .slot_of(*site)
                    .ok_or(PositionError::NoSuchSlot(site.index()))?,
            );
        }
        for slot in slots {
            self.positions.set_target(slot, kind, target)?;
        }
        Ok(())
    }

    /// Returns when the site positions are rebalanced.
    #[must_use]
    pub const fn position_schedule(&self) -> RateSchedule {
        self.position_schedule
    }

    /// Sets when the site positions are rebalanced.
    ///
    /// The interval is a parameter of the world. This function holds no
    /// recommended value.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0065, a group is a site membership, not a region, decision D3. `docs/adrs/draft/adr-0065-a-group-is-a-site-membership-not-a-region.md`
    ///
    /// # Errors
    ///
    /// Returns an error when the period is zero, and when the period is
    /// above the range that the schedule takes.
    pub fn set_position_schedule(&mut self, period: u32, phase: u32) -> Result<(), RateError> {
        self.position_schedule =
            RateSchedule::new(period, phase).ok_or(RateError::PeriodOutsideRange(period))?;
        Ok(())
    }

    /// Rebuilds the derived structure when it no longer describes the arena.
    ///
    /// One rule governs both rebuild sites in the step: rebuild when the
    /// arena has moved since the last rebuild, and not otherwise. The
    /// structure holds the revision it was built from, so the test is one
    /// comparison and it reads no unit.
    ///
    /// **A frame in which no unit moved rebuilds nothing.** A structure that
    /// already describes the arena is the structure a rebuild would produce,
    /// so skipping it is not an optimisation that trades a guarantee. The
    /// record sanctions a rebuild each frame and argues from the merge order
    /// of incremental writes rather than from frequency, so it neither
    /// requires the rebuild nor forbids the test.[^1]
    ///
    /// A crowd is where this pays. A unit whose every target is full is
    /// refused every frame, and a world in which nothing was admitted leaves
    /// the arena untouched.
    ///
    /// # Errors
    ///
    /// Returns an error when the rebuild refuses to run.
    ///
    /// # References
    ///
    /// [^1]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D3. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
    /// Applies the production rate and the upkeep rate of every site.
    ///
    /// The pass writes the store column and nothing else. It runs on the
    /// tick that the schedule names, and it does nothing on every other
    /// tick.
    ///
    /// The account of what the stores hold moves by the net of the pass.
    /// That net is what landed minus what was taken, and both are exact
    /// integers, so the account and the column stay equal.[^1]
    ///
    /// # Errors
    ///
    /// Returns an error when the caller asks for zero threads.
    ///
    /// # References
    ///
    /// [^1]: ADR-0023, an aggregate combines exactly, in any order, decision D3. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
    /// Settles the positions of every site.
    ///
    /// The release runs on every frame, because a unit dies on any frame and
    /// a position that named it would hold a stale identity until the next
    /// rebalance.[^1]
    ///
    /// The rebalance runs on the interval that the schedule names. It reads
    /// the store as this frame left it, so it runs after the rates and after
    /// the consumption draw.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0065, a group is a site membership, not a region, decision D2. `docs/adrs/draft/adr-0065-a-group-is-a-site-membership-not-a-region.md`
    /// [^2]: ADR-0065, a group is a site membership, not a region, decision D3. `docs/adrs/draft/adr-0065-a-group-is-a-site-membership-not-a-region.md`
    pub(super) fn settle_positions(&mut self, threads: usize) -> Result<(), StepError> {
        position::release_the_dead(&mut self.positions, &self.soldiers, threads)?;
        if !self.position_schedule.due(self.tick) {
            return Ok(());
        }
        position::rebalance(
            &mut self.positions,
            self.settlements.live_column(),
            self.settlements.tile_column(),
            self.settlements.store_column(),
            self.terrain,
            threads,
        )?;
        // The resize opens the positions and seats nobody. This fills them,
        // and it runs on the same schedule because a seat cannot be taken
        // before it is opened.[^1]
        //
        // [^1]: ADR-0099, a site fills its positions by one sort and one scan, decision D2. `docs/adrs/draft/adr-0099-a-site-fills-its-positions-by-one-sort-and-one-scan.md`
        position::assign(&mut self.positions, &self.soldiers, threads)?;
        Ok(())
    }

    /// Records the seat of a faction: the tile of its first founding.
    ///
    /// A later founding of the same faction leaves the seat where it is. The
    /// controller plans around the seat, so a faction with no seat receives
    /// no evaluation.
    pub(super) fn record_seat(&mut self, faction: FactionId, place: Axial) {
        if let Some(tile) = self.grid.index_of(place) {
            self.controller.set_seat(faction, tile);
        }
    }

    /// Returns the seat of a faction: the tile of its first founding, or
    /// `None` when the faction founded nothing.
    #[must_use]
    pub fn seat(&self, faction: FactionId) -> Option<TileIdx> {
        self.controller.row(faction).and_then(FactionRow::seat)
    }
}
