//! The action table, the legality answer, and the verb that one integer names.
//!
//! A learner plays a faction through one integer. The schema turns the
//! integer into a verb and its arguments, the legality readers say whether
//! the world admits it, and the apply call runs it. The three sit together,
//! because the integer means nothing without all of them.

use super::World;
use crate::action::{place_cell, ActionSchema, ActionShape, Verb, PLACE_ANYWHERE, PLACE_COUNT};
use crate::campaign;
use crate::controller::{self, ControllerCommand};
use crate::hex::Axial;
use crate::holding::Holder;
use crate::obs_ring::{cell_of_delta, RING_STACK_CELLS};
use crate::production::QueueOrder;
use crate::resource::ResourceKind;
use crate::types::{Entity, FactionId, TileIdx};
use crate::unit_type::UnitTypeId;
use crate::upgrade::UpgradeCategory;

/// One settlement that a campaign of one faction could march on.
///
/// The row carries the cell of the egocentric frame that the settlement falls
/// in, so the objective of one place is a filter over these rows and not a
/// second walk over the settlements.[^1]
///
/// # References
///
/// [^1]: ADR-0197, a verb names a place by a cell of the egocentric frame the observation publishes, decision D3. `docs/adrs/draft/adr-0197-a-verb-names-a-place-by-a-cell-of-the-egocentric-frame.md`
struct CampaignCandidate {
    /// The cell of the egocentric frame the settlement falls in.
    cell: u32,
    /// The hex distance from the seat of the marching faction.
    distance: u32,
    /// The settlement slot, which breaks a tie on the distance.
    slot: u32,
    /// The tile the settlement stands on.
    tile: TileIdx,
    /// One when the settlement is the faction's own and stands on ground an
    /// enemy holds, so a march on it is a relief.
    relief: u8,
}

/// Returns the objective that a campaign takes over one set of candidates.
///
/// **A relief comes before a take.** The rule is the controller's own, and
/// this function is its one statement for the action table. The nearest site
/// wins, and the settlement slot breaks a tie, so the answer is a property of
/// the arena and never of a visit order.[^1]
///
/// # References
///
/// [^1]: ADR-0004, iteration order is explicit, decision D4. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
fn nearest_objective(candidates: &[&CampaignCandidate]) -> Option<TileIdx> {
    let key = |candidate: &&CampaignCandidate| (candidate.distance, candidate.slot, candidate.tile);
    campaign::nearest_site(
        candidates
            .iter()
            .filter(|candidate| candidate.relief == 1)
            .map(key),
    )
    .or_else(|| {
        campaign::nearest_site(
            candidates
                .iter()
                .filter(|candidate| candidate.relief == 0)
                .map(key),
        )
    })
}

/// What every per-row legality question of one faction shares.
///
/// One action and one legality answer both build this once. The campaign
/// objective of every place therefore has one statement for one tick, and the
/// answer and the verb cannot disagree about it.[^1]
///
/// # References
///
/// [^1]: ADR-0197, a verb names a place by a cell of the egocentric frame the observation publishes, decision D3. `docs/adrs/draft/adr-0197-a-verb-names-a-place-by-a-cell-of-the-egocentric-frame.md`
struct ActionContext {
    /// The campaign objective of each place value, in ascending place order.
    campaign_objectives: Vec<Option<TileIdx>>,
}

impl ActionContext {
    /// Returns the campaign objective of one place value, and nothing when
    /// the faction observes no settlement it could march on there.
    ///
    /// Answers nothing for a value at or above the bound of the position, so
    /// a caller that decoded a wider integer refuses rather than acts.
    fn campaign_objective(&self, place: u32) -> Option<TileIdx> {
        self.campaign_objectives
            .get(place as usize)
            .copied()
            .flatten()
    }
}

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
/// crossing sends at a tile. **The engine resolves both through the readers
/// that answer for one faction.** A campaign objective is a settlement the
/// faction has observed, and a crossing target is a place the faction has
/// observed. A learner therefore cannot learn from a legality byte that a
/// settlement it has never seen stands somewhere.[^4]
///
/// # A campaign names the region it marches in
///
/// A campaign declares one place position. The position carries a cell of
/// the egocentric frame the observation reads, and its first value names the
/// whole frame.[^6] The engine still chooses the objective, by the rule it
/// already owns, over the settlements that fall in the named cell.
///
/// **One reader answers the objective of every place at once.** The legality
/// answer reads that list and the verb reads the same list, so the two cannot
/// disagree about which settlement a cell holds.[^7]
///
/// A crossing keeps no place position. It takes its target from a keyed
/// sample of the world, and a place position may not narrow a draw.[^6]
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
/// [^6]: ADR-0197, a verb names a place by a cell of the egocentric frame the observation publishes, decisions D1, D2 and D5. `docs/adrs/draft/adr-0197-a-verb-names-a-place-by-a-cell-of-the-egocentric-frame.md`
/// [^7]: ADR-0197, a verb names a place by a cell of the egocentric frame the observation publishes, decision D3. `docs/adrs/draft/adr-0197-a-verb-names-a-place-by-a-cell-of-the-egocentric-frame.md`
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
    /// A verb is asked once whether it could act at all, and then once for
    /// each row it holds. A verb with no argument position therefore costs
    /// one question. The context below answers every place of the campaign
    /// verb in one pass, so the rows of that verb cost one pass and not one
    /// pass each.
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
        let context = self.action_context(faction);
        let mut answer = vec![0u8; schema.length() as usize];
        for row in schema.rows() {
            let allowed = self.verb_is_legal(faction, row.verb, &context);
            for offset in 0..row.rows {
                let action = row.first + offset;
                let Some((_, arguments)) = schema.decode(action) else {
                    continue;
                };
                let legal =
                    allowed && self.arguments_are_legal(faction, row.verb, &arguments, &context);
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
        let context = self.action_context(faction);
        let applied = self.apply_verb(faction, verb, &arguments, &context);
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
    ///
    /// The campaign arm reads the objective of the whole frame, which is the
    /// objective the verb took before the place position existed. **The
    /// cohort refusal does not follow the objective**, beyond the objective
    /// standing inside the world, and every settlement stands inside the
    /// world. One cohort question therefore answers every place of the verb,
    /// and the place arm below reads the cell alone.
    fn verb_is_legal(&self, faction: FactionId, verb: Verb, context: &ActionContext) -> bool {
        match verb {
            // The no-op changes nothing, so nothing can refuse it.
            Verb::NoOp => true,
            // A gather order and a build order reach the units of the
            // faction. The order verb refuses a unit that is not live, and
            // the set below holds live units only.
            Verb::Gather | Verb::Build => !self.faction_units(faction).is_empty(),
            Verb::Relation => self.speaker_of(faction).is_some(),
            Verb::Campaign => context
                .campaign_objective(PLACE_ANYWHERE)
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
    ///
    /// The campaign arm asks whether the faction observes a settlement it
    /// could march on in the named cell of the egocentric frame.[^1] It reads
    /// the same list the verb reads, so the two cannot disagree.
    ///
    /// # References
    ///
    /// [^1]: ADR-0197, a verb names a place by a cell of the egocentric frame the observation publishes, decision D3. `docs/adrs/draft/adr-0197-a-verb-names-a-place-by-a-cell-of-the-egocentric-frame.md`
    fn arguments_are_legal(
        &self,
        faction: FactionId,
        verb: Verb,
        arguments: &[u32],
        context: &ActionContext,
    ) -> bool {
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
            Verb::Campaign => context.campaign_objective(first).is_some(),
            Verb::NoOp
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
    /// **A place the verb cannot honour is a refusal.** The campaign arm
    /// reads the objective of the named cell alone. It finds none, changes
    /// nothing, and reports that it did not act. The caller above writes the
    /// log row for the refusal, and no arm falls back to another cell.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decision D2. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
    /// [^2]: ADR-0197, a verb names a place by a cell of the egocentric frame the observation publishes, decision D6. `docs/adrs/draft/adr-0197-a-verb-names-a-place-by-a-cell-of-the-egocentric-frame.md`
    fn apply_verb(
        &mut self,
        faction: FactionId,
        verb: Verb,
        arguments: &[u32],
        context: &ActionContext,
    ) -> bool {
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
                let Some(address) = context
                    .campaign_objective(first)
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

    /// Returns the campaign objective of one faction for every place value,
    /// in ascending place order.
    ///
    /// **This is the one statement of the campaign objective rule.** The
    /// legality answer reads this list and the verb reads the same list, so
    /// the two cannot disagree about which settlement a cell holds.[^1]
    ///
    /// The first entry holds the objective the engine resolves over the whole
    /// frame, which is the objective the verb took before the place position
    /// existed.[^2] The entry of a place above it holds the objective the
    /// engine resolves over one cell of the egocentric frame alone.
    ///
    /// **The cell of a settlement comes from the frame the observation
    /// reads.** The centre, the ring rule and the sector rule have one
    /// statement in the engine, so a cell index names the same ground in the
    /// array a policy reads and in the integer that policy emits.[^2] [^3]
    ///
    /// **This reads the tiles the faction has seen, and never the world.**
    /// The controller reads the whole world for its own factions, and a
    /// learner may not, because a legality byte that answered from the whole
    /// world would tell the learner that a settlement stands somewhere it
    /// has never looked.[^4]
    ///
    /// The rule inside a cell is the controller's own: a relief comes before
    /// a take, and the nearest site wins with a tie to the lowest slot. The
    /// distance runs from the seat of the faction, as it does for the
    /// controller, and never from the centre of the frame.
    ///
    /// # References
    ///
    /// [^1]: ADR-0197, a verb names a place by a cell of the egocentric frame the observation publishes, decision D3. `docs/adrs/draft/adr-0197-a-verb-names-a-place-by-a-cell-of-the-egocentric-frame.md`
    /// [^2]: ADR-0197, a verb names a place by a cell of the egocentric frame the observation publishes, decisions D1 and D2. `docs/adrs/draft/adr-0197-a-verb-names-a-place-by-a-cell-of-the-egocentric-frame.md`
    /// [^3]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
    /// [^4]: PRD-0001, a faction sees only what it observes. `docs/product/accepted/prd-0001-a-faction-sees-only-what-it-observes.md`
    fn observed_campaign_objectives(&self, faction: FactionId) -> Vec<Option<TileIdx>> {
        let none = || vec![None; PLACE_COUNT as usize];
        let count = self.config.faction_count.max(1);
        let Some(seat) = self
            .seat(faction)
            .and_then(|tile| self.grid.address_of(tile))
        else {
            return none();
        };
        if self.campaigns.live(faction).is_some() {
            return none();
        }
        let at_war =
            |other: FactionId| other != faction && self.relations.war_between(faction, other);
        if !(0..count).any(|other| at_war(FactionId(other))) {
            return none();
        }
        let centre = self.frame_centre(faction);
        let observed = |tile: TileIdx| {
            self.grid
                .address_of(tile)
                .and_then(|address| self.grid.index_of(address))
                .is_some_and(|index| self.observation.has_seen(faction, index))
        };
        let holder_at_war = |tile: TileIdx| {
            self.grid
                .address_of(tile)
                .and_then(|address| self.holding.holder(address))
                .and_then(Holder::faction)
                .is_some_and(at_war)
        };
        let mut candidates: Vec<CampaignCandidate> = Vec::new();
        for site in self.settlements.iter() {
            let (Some(slot), Some(owner), Some(tile)) = (
                self.settlements.slot_of(site),
                self.settlements.faction(site),
                self.settlements.tile(site),
            ) else {
                continue;
            };
            let Some(address) = self.grid.address_of(tile) else {
                continue;
            };
            if !observed(tile) {
                continue;
            }
            let relief = owner == faction && holder_at_war(tile);
            if !relief && !at_war(owner) {
                continue;
            }
            let delta = Axial::new(address.q - centre.q, address.r - centre.r);
            candidates.push(CampaignCandidate {
                cell: cell_of_delta(delta),
                distance: seat.distance(address),
                slot,
                tile,
                relief: u8::from(relief),
            });
        }
        let mut by_cell: Vec<Vec<&CampaignCandidate>> = vec![Vec::new(); RING_STACK_CELLS as usize];
        for candidate in &candidates {
            if let Some(bucket) = by_cell.get_mut(candidate.cell as usize) {
                bucket.push(candidate);
            }
        }
        let whole_frame: Vec<&CampaignCandidate> = candidates.iter().collect();
        (0..PLACE_COUNT)
            .map(|place| match place_cell(place) {
                None => nearest_objective(&whole_frame),
                Some(cell) => by_cell
                    .get(cell as usize)
                    .and_then(|bucket| nearest_objective(bucket)),
            })
            .collect()
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

    /// Builds what the per-row legality questions of one faction share.
    ///
    /// The build resolves the campaign objective of every place in one pass,
    /// so the rows of that verb cost one pass and not one pass each. The
    /// caller of one action builds the same context, so the answer and the
    /// verb read one list.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0197, a verb names a place by a cell of the egocentric frame the observation publishes, decision D3. `docs/adrs/draft/adr-0197-a-verb-names-a-place-by-a-cell-of-the-egocentric-frame.md`
    fn action_context(&self, faction: FactionId) -> ActionContext {
        ActionContext {
            campaign_objectives: self.observed_campaign_objectives(faction),
        }
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
