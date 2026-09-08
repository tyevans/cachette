//! A campaign: the cohort it raises, the objective it holds, and its close.
//!
//! A campaign sends a body of units at an objective and ends when the
//! objective falls or the deadline passes. The raise, the objective readers
//! and the close sit together, because they are three points on one life.

use super::errors::CampaignError;
use super::World;
use crate::campaign::{self, CampaignEvent, CampaignRow};
use crate::hex::Axial;
use crate::holding::Holder;
use crate::types::{Entity, FactionId, TileIdx};
use crate::unit_type::SOLDIER;

impl World {
    /// Raises a campaign: takes the idle units of a faction, makes them
    /// soldiers and sends them at an objective tile.
    ///
    /// **This is the one path the controller and a Python caller share.**
    /// The raise acts through the set form of the type verb and through the
    /// send verb, and it writes one row of the campaign register.[^1]
    ///
    /// An idle unit is a live unit of the faction that nobody has sent
    /// anywhere. The cohort is the lowest identities among them, up to the
    /// count asked for, so two runs over one arena take one cohort. The scan
    /// that finds them follows the population, as the verbs it feeds do.
    ///
    /// The cohort is sent on the destination plane whose number is the
    /// faction number. A caller that sends its own set on that plane re-aims
    /// the cohort.
    ///
    /// The objective kind is read from the tile: a settlement of the faction
    /// itself makes a relief, and anything else makes a take. The holder of
    /// the tile at the raise is recorded, and the campaign closes when the
    /// holder changes.
    ///
    /// # Errors
    ///
    /// Returns an error when the number names no faction, when the address is
    /// outside the world, when the cohort size is zero, when the faction holds
    /// a live campaign, when it has no idle unit, when the world holds no
    /// destination plane for it, and when the send verb refuses.
    ///
    /// # References
    ///
    /// [^1]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decision D2. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
    pub fn raise_campaign(
        &mut self,
        faction: FactionId,
        objective: Axial,
        cohort: u32,
    ) -> Result<CampaignRow, CampaignError> {
        // **The rule lives in the check, and this verb reads it.** The
        // legality answer reads the same check.[^lg]
        //
        // [^lg]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decision D5. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
        let idle = self.campaign_cohort(faction, objective, cohort)?;
        let tile = self
            .grid
            .index_of(objective)
            .ok_or(CampaignError::OutsideWorld(objective))?;
        let plane = faction.0;
        let objective_kind = if self
            .settlements
            .on_tile(objective)
            .and_then(|site| self.settlements.faction(site))
            == Some(faction)
        {
            campaign::OBJECTIVE_RELIEVE_SITE
        } else {
            campaign::OBJECTIVE_TAKE_SITE
        };
        let holder_at_raise = self
            .holding
            .holder(objective)
            .and_then(Holder::faction)
            .map_or(campaign::NO_HOLDER, |holder| holder.0);
        self.set_unit_type_set(&idle, SOLDIER);
        self.send_units_to(&idle, &[objective], plane)?;
        let row = CampaignRow {
            raised_at: self.tick,
            objective_tile: tile.0,
            cohort_size: idle.len() as u32,
            faction,
            holder_at_raise,
            objective_kind,
            state: campaign::STATE_LIVE,
            padding: [0; 2],
        };
        assert!(
            self.campaigns.open(row),
            "the faction exists and holds no live campaign, so the register takes the row"
        );
        self.campaigns.push(CampaignEvent {
            tick: self.tick,
            objective_tile: tile.0,
            cohort_size: row.cohort_size,
            faction,
            kind: campaign::EVENT_RAISED,
            objective_kind,
            padding: [0; 4],
        });
        Ok(row)
    }

    /// Returns the cohort that a raise would march, or the refusal it would
    /// give.
    ///
    /// **This is the one statement of the rule.** The raise verb calls it
    /// before it writes a register row, and the legality answer calls it to
    /// fill one row of the action table.[^1] Nothing here mutates.
    ///
    /// # Errors
    ///
    /// Returns the refusal the raise verb would return.
    ///
    /// # References
    ///
    /// [^1]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decision D5. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
    pub fn campaign_cohort(
        &self,
        faction: FactionId,
        objective: Axial,
        cohort: u32,
    ) -> Result<Vec<Entity>, CampaignError> {
        if faction.0 >= self.config.faction_count.max(1) {
            return Err(CampaignError::NoSuchFaction(faction.0));
        }
        self.grid
            .index_of(objective)
            .ok_or(CampaignError::OutsideWorld(objective))?;
        if cohort == 0 {
            return Err(CampaignError::EmptyCohort);
        }
        if self.campaigns.live(faction).is_some() {
            return Err(CampaignError::LiveCampaign);
        }
        let plane = faction.0;
        if plane >= self.destinations.plane_count() {
            return Err(CampaignError::NoPlane(plane));
        }
        // The lowest identities among the idle units. The arena walks in
        // slot order, and the sort puts the generation above the slot, so
        // the choice is a property of the identities and not of the slots.
        //
        // **A unit that carries command reach is never taken.** The raise
        // retypes the cohort to the soldier row, and the soldier row carries
        // no command reach, so a raise that swept up the one leader of a
        // faction spent the very unit that lets the faction declare a war.
        // The faction would then march once and never again.
        //
        // **A unit that carries a water crossing is never taken either**, for
        // the same reason. The soldier row carries no crossing, so a raise
        // that swept up the mariners of an island faction spent the very
        // units that let it leave its island, and it would do so on the tick
        // each one was built.
        let leads = |unit: &Entity| {
            self.soldiers.unit_type(*unit).is_some_and(|unit_type| {
                let row = self.unit_types.row(unit_type);
                row.command_reach > 0 || row.water_crossing > 0
            })
        };
        let mut idle: Vec<Entity> = self
            .soldiers
            .iter_faction(faction)
            .filter(|unit| self.soldiers.sent(*unit) == Some(None) && !leads(unit))
            .collect();
        idle.sort_unstable_by_key(|unit| unit.to_bits());
        // **The raise takes the number of units it asks for.** The project
        // order sends the idle units of a faction on the same destination
        // plane, so a raise that took the idle units alone found almost none
        // left and marched a cohort of one.[^2] A faction holds no live
        // campaign here, so every unit already on its own plane is a project
        // walker, and the raise re-aims it. The pool is therefore the idle
        // units first, then the walkers, and both in identity order.
        //
        // [^2]: Findings register, FND-542. `docs/FINDINGS.md`
        if idle.len() < cohort as usize {
            let mut walking: Vec<Entity> = self
                .soldiers
                .iter_faction(faction)
                .filter(|unit| self.soldiers.sent(*unit) == Some(Some(plane)) && !leads(unit))
                .collect();
            walking.sort_unstable_by_key(|unit| unit.to_bits());
            idle.extend_from_slice(&walking);
        }
        if idle.is_empty() {
            return Err(CampaignError::NoIdleUnit);
        }
        idle.truncate(cohort as usize);
        Ok(idle)
    }

    /// Returns the campaign rows of one faction, in slot order. Empty when the
    /// world has no such faction.
    #[must_use]
    pub fn campaigns_of(&self, faction: FactionId) -> &[CampaignRow] {
        self.campaigns.rows_of(faction)
    }

    /// Returns what happened to the campaigns on the last step, in the order
    /// it happened.
    #[must_use]
    pub fn campaign_log(&self) -> &[CampaignEvent] {
        self.campaigns.log()
    }

    /// Returns the campaign log as bytes, for the byte comparison the
    /// thread-count test makes.
    #[must_use]
    pub fn campaign_log_bytes(&self) -> &[u8] {
        self.campaigns.log_bytes()
    }

    /// Returns how many units the controller takes when it raises a
    /// campaign.[^1]
    ///
    /// # References
    ///
    /// [^1]: Balance register, the campaign cohort size. `docs/reference/balance.md`
    #[must_use]
    pub const fn campaign_cohort_size(&self) -> u32 {
        self.campaigns.cohort_size()
    }

    /// Sets how many units the controller takes when it raises a campaign.
    pub const fn set_campaign_cohort_size(&mut self, cohort: u32) {
        self.campaigns.set_cohort_size(cohort);
    }

    /// Closes every live campaign whose objective changed holder or whose
    /// cohort fell, and stops sending the survivors.
    ///
    /// The cohorts are the units sent on the plane of each faction, as the
    /// one scan of the stage found them. A campaign whose objective passed to
    /// the campaigner is won. One whose objective passed to anyone else has
    /// ended. One whose cohort is empty is lost. The survivors go back to the
    /// option they chose for themselves, through the stop verb a caller has.
    pub(super) fn close_campaigns(&mut self, cohorts: &[Vec<Entity>]) {
        let tick = self.tick;
        for (index, cohort) in cohorts.iter().enumerate() {
            let faction = FactionId(index as u16);
            let Some(row) = self.campaigns.live(faction) else {
                continue;
            };
            let holder = self
                .grid
                .address_of(TileIdx(row.objective_tile))
                .and_then(|address| self.holding.holder(address))
                .and_then(Holder::faction)
                .map_or(campaign::NO_HOLDER, |holder| holder.0);
            // **A campaign that reaches nothing closes at its deadline.** A
            // faction with a live campaign raises no other one, so a campaign
            // that never closes takes the whole run and the faction marches
            // once. The deadline is a balance value.[^1] [^2]
            //
            // [^1]: Findings register, FND-542. `docs/FINDINGS.md`
            // [^2]: Balance register, the campaign deadline. `docs/reference/balance.md`
            let deadline = self.campaigns.deadline();
            let expired = deadline.0 > 0 && tick.0.saturating_sub(row.raised_at.0) >= deadline.0;
            let (state, kind) = if holder != row.holder_at_raise {
                if holder == faction.0 {
                    (campaign::STATE_WON, campaign::EVENT_WON)
                } else {
                    (campaign::STATE_ENDED, campaign::EVENT_ENDED)
                }
            } else if cohort.is_empty() {
                (campaign::STATE_LOST, campaign::EVENT_LOST)
            } else if expired {
                (campaign::STATE_EXPIRED, campaign::EVENT_EXPIRED)
            } else {
                continue;
            };
            self.campaigns.close(faction, state);
            // Every survivor is live, because the scan found it live on this
            // tick, so the stop verb refuses nothing.
            let _ = self.stop_sending(cohort);
            self.campaigns.push(CampaignEvent {
                tick,
                objective_tile: row.objective_tile,
                cohort_size: row.cohort_size,
                faction,
                kind,
                objective_kind: row.objective_kind,
                padding: [0; 4],
            });
        }
    }

    /// Chooses, for each faction, the objective it would march on.
    ///
    /// A faction with no seat, with a live campaign, or with no pair in the
    /// war band gets none.[^1] Otherwise an own settlement whose ground a
    /// faction at war holds is a relief, and the nearest enemy settlement is
    /// a take. The relief comes first. Nearest is the hex distance from the
    /// seat, and a tie goes to the lowest settlement slot. The scan walks the
    /// settlements and no unit, so it follows the site count.
    ///
    /// # References
    ///
    /// [^1]: ADR-0146, a faction relation is one signed integer per ordered pair, and a pass reads a threshold, decision D2. `docs/adrs/accepted/adr-0146-a-faction-relation-is-one-signed-integer-per-ordered-pair-and-a-pass-reads-a-threshold.md`
    pub(super) fn campaign_objectives(&self) -> Vec<Option<(u8, TileIdx)>> {
        let count = self.config.faction_count.max(1);
        let sites: Vec<(u32, FactionId, TileIdx)> = self
            .settlements
            .iter()
            .filter_map(|site| {
                Some((
                    self.settlements.slot_of(site)?,
                    self.settlements.faction(site)?,
                    self.settlements.tile(site)?,
                ))
            })
            .collect();
        (0..count)
            .map(|index| {
                let faction = FactionId(index);
                let seat = self.seat(faction)?;
                if self.campaigns.live(faction).is_some() {
                    return None;
                }
                let at_war = |other: FactionId| {
                    other != faction && self.relations.war_between(faction, other)
                };
                if !(0..count).any(|other| at_war(FactionId(other))) {
                    return None;
                }
                let seat = self.grid.address_of(seat)?;
                let distance = |tile: TileIdx| {
                    self.grid
                        .address_of(tile)
                        .map_or(u32::MAX, |address| seat.distance(address))
                };
                let holder_at_war = |tile: TileIdx| {
                    self.grid
                        .address_of(tile)
                        .and_then(|address| self.holding.holder(address))
                        .and_then(Holder::faction)
                        .is_some_and(at_war)
                };
                let relief = campaign::nearest_site(
                    sites
                        .iter()
                        .filter(|(_, owner, tile)| *owner == faction && holder_at_war(*tile))
                        .map(|(slot, _, tile)| (distance(*tile), *slot, *tile)),
                );
                if let Some(tile) = relief {
                    return Some((campaign::OBJECTIVE_RELIEVE_SITE, tile));
                }
                campaign::nearest_site(
                    sites
                        .iter()
                        .filter(|(_, owner, _)| at_war(*owner))
                        .map(|(slot, _, tile)| (distance(*tile), *slot, *tile)),
                )
                .map(|tile| (campaign::OBJECTIVE_TAKE_SITE, tile))
            })
            .collect()
    }
}
