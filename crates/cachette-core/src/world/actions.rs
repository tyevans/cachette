//! The action table, the legality answer, and the verb that one integer names.
//!
//! A learner plays a faction through one integer. The schema turns the
//! integer into a verb and its arguments, the legality readers say whether
//! the world admits it, and the apply call runs it. The three sit together,
//! because the integer means nothing without all of them.

use super::World;
use crate::action::{ActionSchema, ActionShape, Verb};
use crate::campaign;
use crate::controller::{self, ControllerCommand};
use crate::holding::Holder;
use crate::production::QueueOrder;
use crate::resource::ResourceKind;
use crate::types::{Entity, FactionId, TileIdx};
use crate::unit_type::UnitTypeId;
use crate::upgrade::UpgradeCategory;

/// The action table, the legality answer, and the verb that takes one
/// action integer.
///
/// # One integer, one action
///
/// A learner plays one faction, and it acts by one integer that indexes a
/// bounded table the engine declares.[^1] The schema of that table is a
/// mixed radix over the argument positions each verb declares, and the
/// action module holds it.[^2] This block joins the table to the verbs.
///
/// # The answer and the verb read one rule
///
/// The engine answers one byte for each row of the table, and the byte says
/// whether the verb would refuse that row at this tick.[^3] **The answer
/// restates no refusal rule.** Each verb of the enumeration carries a check
/// that reports its refusal without acting, and the verb itself calls that
/// check before it changes anything. The answer calls the same check.
///
/// **The verb that takes an action does not read the answer first.** It
/// runs the verb and reports what the verb did. A row the answer allows and
/// the verb then refuses is therefore a defect a test can find, rather than
/// a disagreement the engine hides from itself.[^3]
///
/// # The answer names no target the faction cannot see
///
/// Two verbs act on a place: a campaign marches at an objective, and a
/// crossing sends at a tile. The engine resolves both, so neither carries an
/// argument position.[^2] **The engine resolves both through the readers
/// that answer for one faction.** A campaign objective is a settlement the
/// faction has observed, and a crossing target is a place the faction has
/// observed. A learner therefore cannot learn from a legality byte that a
/// settlement it has never seen stands somewhere.[^4]
///
/// # Determinism
///
/// The answer walks the rows in ascending action order on the calling
/// thread. Every candidate list it resolves is already sorted by a stable
/// key. Nothing here draws, reads a thread, or reads a completion
/// order.[^5]
///
/// # References
///
/// [^1]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decision D4. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
/// [^2]: ADR-0176, an action integer is a mixed radix over the argument positions each verb declares, decisions D1 and D2. `docs/adrs/accepted/adr-0176-an-action-integer-is-a-mixed-radix-over-the-positions-a-verb-declares.md`
/// [^3]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decision D5. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
/// [^4]: PRD-0001, a faction sees only what it observes. `docs/product/accepted/prd-0001-a-faction-sees-only-what-it-observes.md`
/// [^5]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
impl World {
    /// Returns the declared layout of the action table of this world.
    ///
    /// The schema names each verb, the argument positions it declares, the
    /// bound of each position, and the stride that position moves the action
    /// integer by. A caller decodes an integer by arithmetic over this
    /// schema, and never by a table it holds.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0176, an action integer is a mixed radix over the argument positions each verb declares, decision D1. `docs/adrs/accepted/adr-0176-an-action-integer-is-a-mixed-radix-over-the-positions-a-verb-declares.md`
    #[must_use]
    pub fn action_schema(&self) -> ActionSchema {
        ActionSchema::of(ActionShape {
            faction_count: u32::from(self.config.faction_count.max(1)),
        })
    }

    /// Returns one byte for each row of the action table of this world.
    ///
    /// The byte is one when the verb would take that row at this tick, and
    /// zero when it would refuse it. Row zero is the no-op, and it is always
    /// one, so the answer is never empty.[^1]
    ///
    /// The answer holds only what the faction observes. **No argument
    /// widens it.**[^2]
    ///
    /// Returns `None` when the number names no faction of this world.
    ///
    /// # References
    ///
    /// [^1]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decision D5. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
    /// [^2]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decision D3. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
    #[must_use]
    pub fn legal_actions(&self, faction: FactionId) -> Option<Vec<u8>> {
        if faction.0 >= self.config.faction_count.max(1) {
            return None;
        }
        let schema = self.action_schema();
        let mut answer = vec![0u8; schema.length() as usize];
        // A verb is asked once whether it could act at all, and then once
        // for each row it holds. A verb with no argument position therefore
        // costs one question.
        for row in schema.rows() {
            let allowed = self.verb_is_legal(faction, row.verb);
            for offset in 0..row.rows {
                let action = row.first + offset;
                let Some((_, arguments)) = schema.decode(action) else {
                    continue;
                };
                let legal = allowed && self.arguments_are_legal(faction, row.verb, &arguments);
                answer[action as usize] = u8::from(legal);
            }
        }
        Some(answer)
    }

    /// Applies one action of one faction, through the same verbs a caller
    /// and the built-in controller use.[^1]
    ///
    /// Returns whether the verb took the action. **This runs the verb and
    /// reports what the verb did.** It does not read the legality answer
    /// first, so a disagreement between the two is a defect a test can
    /// find.[^2]
    ///
    /// The engine writes one row of the command log for the action, whether
    /// the verb took it or refused it.[^3]
    ///
    /// Returns `false` when the number names no faction, and when the
    /// integer is at or above the length of the table.
    ///
    /// # References
    ///
    /// [^1]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decision D2. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
    /// [^2]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decision D5. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
    /// [^3]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decision D6. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
    pub fn act(&mut self, faction: FactionId, action: u32) -> bool {
        if faction.0 >= self.config.faction_count.max(1) {
            return false;
        }
        let schema = self.action_schema();
        let Some((verb, arguments)) = schema.decode(action) else {
            return false;
        };
        let applied = self.apply_verb(faction, verb, &arguments);
        // **An action leaves the world readable.** A verb of the table
        // founds a city, spends a settler, or changes the faction of a unit,
        // and each of those moves the arena past the derived unit
        // structure.[^4] A caller sends an action between two steps, so the
        // barrier of the next step is too late: the reader that runs before
        // it met a refusal, and the refusal named nothing.[^5]
        //
        // The refresh compares two revisions on a tick that changed no
        // structure, and it rebuilds on a tick that did.
        //
        // [^4]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decisions D3 and D4. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
        // [^5]: Findings register, FND-647. `docs/FINDINGS.md`
        self.leave_the_world_readable();
        let tick = self.tick;
        self.controller.push(ControllerCommand {
            tick,
            faction,
            action,
            sequence: 0,
            applied: u8::from(applied),
            // The schema decoded the integer above, so the column holds the
            // action the caller named.
            encoded: 1,
            padding: [0; 4],
        });
        applied
    }

    /// Reports whether one verb could act at all for one faction this tick,
    /// without reading its arguments.
    ///
    /// The verbs that declare no argument position answer here alone.
    fn verb_is_legal(&self, faction: FactionId, verb: Verb) -> bool {
        match verb {
            // The no-op changes nothing, so nothing can refuse it.
            Verb::NoOp => true,
            // A gather order and a build order reach the units of the
            // faction. The order verb refuses a unit that is not live, and
            // the set below holds live units only.
            Verb::Gather | Verb::Build => !self.faction_units(faction).is_empty(),
            Verb::Relation => self.speaker_of(faction).is_some(),
            Verb::Campaign => self
                .observed_campaign_objective(faction)
                .and_then(|tile| self.grid.address_of(tile))
                .is_some_and(|address| {
                    self.campaign_cohort(faction, address, self.campaigns.cohort_size())
                        .is_ok()
                }),
            Verb::Advertise => self.check_faction(faction).is_ok(),
            Verb::Trade => self.trade_step_is_due(faction),
            Verb::Carry => self.controller_carry_work(faction),
            Verb::Project => self.project_work(faction),
            Verb::Queue => self.controller_queue_site(faction).is_some(),
            Verb::Cross => self.observed_crossing_target(faction).is_some(),
            Verb::Settle => self.settle_work(faction),
        }
    }

    /// Reports whether one verb would take the arguments of one row.
    ///
    /// A verb with no argument position answers yes here, because the check
    /// above already read its whole refusal.
    fn arguments_are_legal(&self, faction: FactionId, verb: Verb, arguments: &[u32]) -> bool {
        let first = arguments.first().copied().unwrap_or(u32::MAX);
        match verb {
            Verb::Gather => u8::try_from(first)
                .ok()
                .and_then(ResourceKind::from_u8)
                .is_some(),
            Verb::Build => {
                let Some(category) = u8::try_from(first).ok().and_then(UpgradeCategory::from_u8)
                else {
                    return false;
                };
                self.faction_units(faction)
                    .iter()
                    .any(|unit| self.build_refusal(*unit, category).is_ok())
            }
            Verb::Relation => {
                let (Some(speaker), Ok(other)) = (self.speaker_of(faction), u16::try_from(first))
                else {
                    return false;
                };
                self.move_relation_refusal(speaker, FactionId(other), controller::RELATION_STEP)
                    .is_ok()
            }
            // The queue verb refuses a type the table does not hold. The
            // site with room is already read above.
            Verb::Queue => u8::try_from(first)
                .ok()
                .and_then(UnitTypeId::from_u8)
                .is_some(),
            Verb::NoOp
            | Verb::Campaign
            | Verb::Advertise
            | Verb::Trade
            | Verb::Carry
            | Verb::Project
            | Verb::Cross
            | Verb::Settle => true,
        }
    }

    /// Runs one verb of the action table for one faction.
    ///
    /// Every arm goes through the verb the built-in controller goes
    /// through, so a learner reaches no store the controller cannot
    /// reach.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decision D2. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
    fn apply_verb(&mut self, faction: FactionId, verb: Verb, arguments: &[u32]) -> bool {
        let first = arguments.first().copied().unwrap_or(0);
        match verb {
            Verb::NoOp => true,
            Verb::Gather => {
                let Some(kind) = u8::try_from(first).ok().and_then(ResourceKind::from_u8) else {
                    return false;
                };
                let set = self.faction_units(faction);
                !set.is_empty() && self.order_gather_set(&set, kind) < set.len()
            }
            Verb::Build => {
                let Some(category) = u8::try_from(first).ok().and_then(UpgradeCategory::from_u8)
                else {
                    return false;
                };
                let set = self.faction_units(faction);
                !set.is_empty() && self.order_build_set(&set, category) < set.len()
            }
            Verb::Relation => {
                let (Some(speaker), Ok(other)) = (self.speaker_of(faction), u16::try_from(first))
                else {
                    return false;
                };
                self.move_relation(speaker, FactionId(other), controller::RELATION_STEP)
                    .is_ok()
            }
            Verb::Campaign => {
                let cohort = self.campaigns.cohort_size();
                let Some(address) = self
                    .observed_campaign_objective(faction)
                    .and_then(|tile| self.grid.address_of(tile))
                else {
                    return false;
                };
                self.raise_campaign(faction, address, cohort).is_ok()
            }
            // The learner writes its board from the same site economies the
            // controller writes from. The draw index is zero, because a
            // learner emits one action for one faction on one tick.
            Verb::Advertise => self.controller_write_board(faction, 0),
            Verb::Trade => self.controller_trade_step(faction),
            Verb::Carry => self.controller_carriers(faction),
            Verb::Project => self.controller_take_projects(faction),
            Verb::Queue => {
                let (Some(unit_type), Some(site)) = (
                    u8::try_from(first).ok().and_then(UnitTypeId::from_u8),
                    self.controller_queue_site(faction),
                ) else {
                    return false;
                };
                self.order_site_queue(faction, site, QueueOrder::Push(unit_type))
                    .is_ok()
            }
            Verb::Cross => {
                let Some(tile) = self.observed_crossing_target(faction) else {
                    return false;
                };
                self.controller_cross(faction, tile)
            }
            Verb::Settle => self.controller_settle(faction),
        }
    }

    /// Returns the live units of one faction, in identity order.
    ///
    /// The arena yields live identities only, so every unit of the answer
    /// passes the liveness test that the order verbs apply.
    fn faction_units(&self, faction: FactionId) -> Vec<Entity> {
        let mut units: Vec<Entity> = self.soldiers.iter_faction(faction).collect();
        units.sort_unstable_by_key(|unit| unit.to_bits());
        units
    }

    /// Returns the unit of one faction that carries command reach, by the
    /// lowest identity.
    ///
    /// The relation verb reads the command reach column, and a faction with
    /// no such unit moves no relation.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decision D2. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
    fn speaker_of(&self, faction: FactionId) -> Option<Entity> {
        self.faction_units(faction).into_iter().find(|unit| {
            self.soldiers
                .unit_type(*unit)
                .is_some_and(|unit_type| self.unit_types.row(unit_type).command_reach > 0)
        })
    }

    /// Returns the objective that one faction would march on, over the
    /// settlements it has observed.
    ///
    /// **This reads the tiles the faction has seen, and never the world.**
    /// The controller reads the whole world for its own factions, and a
    /// learner may not, because a legality byte that answered from the whole
    /// world would tell the learner that a settlement stands somewhere it
    /// has never looked.[^1]
    ///
    /// The rule is the controller's own: a relief comes before a take, and
    /// the nearest site wins with a tie to the lowest slot.
    ///
    /// # References
    ///
    /// [^1]: PRD-0001, a faction sees only what it observes. `docs/product/accepted/prd-0001-a-faction-sees-only-what-it-observes.md`
    fn observed_campaign_objective(&self, faction: FactionId) -> Option<TileIdx> {
        let count = self.config.faction_count.max(1);
        let seat = self.grid.address_of(self.seat(faction)?)?;
        if self.campaigns.live(faction).is_some() {
            return None;
        }
        let at_war =
            |other: FactionId| other != faction && self.relations.war_between(faction, other);
        if !(0..count).any(|other| at_war(FactionId(other))) {
            return None;
        }
        let observed = |tile: TileIdx| {
            self.grid
                .address_of(tile)
                .and_then(|address| self.grid.index_of(address))
                .is_some_and(|index| self.observation.has_seen(faction, index))
        };
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
            .filter(|(_, _, tile)| observed(*tile))
            .collect();
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
        if relief.is_some() {
            return relief;
        }
        campaign::nearest_site(
            sites
                .iter()
                .filter(|(_, owner, _)| at_war(*owner))
                .map(|(slot, _, tile)| (distance(*tile), *slot, *tile)),
        )
    }

    /// Returns the tile one faction would send its water-crossing units at,
    /// when the faction has observed that tile.
    ///
    /// **The engine surveys the target, and a learner may only reach one it
    /// has seen.** The survey itself is the controller's, so the rule has
    /// one statement. This reader drops a target the faction has never
    /// observed, so a legality byte states nothing about unseen ground.[^1]
    ///
    /// # References
    ///
    /// [^1]: PRD-0001, a faction sees only what it observes. `docs/product/accepted/prd-0001-a-faction-sees-only-what-it-observes.md`
    fn observed_crossing_target(&self, faction: FactionId) -> Option<TileIdx> {
        let tile = self.controller_crossing_target(faction)?;
        let index = self.grid.index_of(self.grid.address_of(tile)?)?;
        if self.observation.has_seen(faction, index) {
            Some(tile)
        } else {
            None
        }
    }

    /// Reports whether one faction has a negotiation step to take this tick.
    ///
    /// The two readers below are the controller's own, and the step verb
    /// takes one branch or the other from them. A faction that neither
    /// reader names takes no step.
    fn trade_step_is_due(&self, faction: FactionId) -> bool {
        self.controller_answer_due(faction).is_some()
            || self.controller_match_due(faction).is_some()
    }

    /// Reports whether one faction has project work this tick.
    ///
    /// The plan holds the projects, and the plane the send needs is busy
    /// while a campaign runs or while the carriers hold it. Those are the
    /// three gates the project verb reads before it moves a unit.
    fn project_work(&self, faction: FactionId) -> bool {
        let Some((standing, walking)) = self.project_partition(faction) else {
            return false;
        };
        if !walking.is_empty() && faction.0 < self.destinations.plane_count() {
            return true;
        }
        standing
            .iter()
            .any(|(unit, category)| self.build_refusal(*unit, *category).is_ok())
    }

    /// Reports whether one faction would found a city or walk a settler this
    /// tick.
    ///
    /// The settle verb founds from the settlers that stand on ground a city
    /// may take, and it walks the rest at the place the survey names. A
    /// faction with no settler does neither.
    fn settle_work(&self, faction: FactionId) -> bool {
        let settlers = self.settlers_of(faction);
        if settlers.is_empty() {
            return false;
        }
        if settlers
            .iter()
            .any(|unit| self.settle_refusal(*unit).is_ok())
        {
            return true;
        }
        self.settling_plane_of(faction).is_some() && self.settling_target(faction).is_some()
    }
}
