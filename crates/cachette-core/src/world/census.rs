//! The census: one row for each subsystem, and the totals the world keeps.
//!
//! A census row names a subsystem and reads one count out of the world. The
//! table, the running totals and the folds that fill them sit together,
//! because a reader who adds a row must add a total beside it.

use super::World;
use crate::campaign;

/// What the run has produced, for each subsystem that counts one tick at a
/// time.
///
/// Every field is a total since the world was built. A total never falls.
///
/// **The world folds a per-tick count into a field here at the one site that
/// empties that count.** The per-tick counter stays the only place that
/// counts the act, so this is a fold of one number and not a second place
/// that counts the same thing.[^1]
///
/// # References
///
/// [^1]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct CensusTotals {
    /// The commands a verb took, over the run.
    pub(super) controller_commands: i64,
    /// The commands a verb refused, over the run.
    pub(super) controller_refused: i64,
    /// The relation moves the controller made through the verb, over the run.
    pub(super) relation_moves: i64,
    /// The boards the trading stage wrote, over the run.
    pub(super) boards_written: i64,
    /// The offers the trading stage opened, over the run.
    pub(super) offers_made: i64,
    /// The contracts the trading stage bound, over the run.
    pub(super) contracts_bound: i64,
    /// The carriers the trading stage assigned, over the run.
    pub(super) carriers_assigned: i64,
    /// The crossings into the war band, over the run, from any cause.
    pub(super) wars_declared: i64,
    /// The campaigns raised, over the run.
    pub(super) campaigns_raised: i64,
    /// The campaigns whose objective passed to the campaigner, over the run.
    pub(super) campaigns_won: i64,
    /// The units the queues produced, over the run.
    pub(super) queue_produced: i64,
    /// The finished entries an advance refused for want of a resident, over
    /// the run.
    pub(super) queue_refused_without_a_person: i64,
    /// The finished entries an advance refused for want of goods, over the
    /// run.
    pub(super) queue_refused_without_goods: i64,
    /// The orders the queue verb refused, over the run.
    pub(super) queue_refused_at_the_verb: i64,
    /// The people the growth stage added, over the run.
    pub(super) births: i64,
}

/// One row of the subsystem census: a name and the reader that counts it.
///
/// **This table is the only declaration of the list.** A caller that prints
/// the census, and a test that checks it, both read this table. A list
/// written anywhere else would be a second declaration site, and nothing
/// would fail when the two disagreed.[^1]
///
/// # References
///
/// [^1]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
#[derive(Clone, Copy)]
pub struct CensusRow {
    /// The name of the subsystem, as the census prints it.
    pub name: &'static str,
    /// What the count measures: what the world holds, or what the run made.
    pub basis: CensusBasis,
    /// The reader that counts what the subsystem produced.
    pub read: fn(&World) -> i64,
}

/// What a census count measures.
///
/// **Every row states its basis, so a reader never has to know one.** A table
/// that held a per-tick row beside a run total would give one zero two
/// meanings, and a reader could not tell them apart.[^1]
///
/// # References
///
/// [^1]: Findings register, FND-498. `docs/FINDINGS.md`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CensusBasis {
    /// What the world holds at the tick the reader runs. The count falls when
    /// the world loses what it counts.
    Held,
    /// What the run has made since the world was built. The count never
    /// falls, so a zero means that the thing never happened.
    Total,
}

impl std::fmt::Debug for CensusRow {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("CensusRow")
            .field("name", &self.name)
            .finish()
    }
}

/// The subsystem census, one row for each subsystem.
///
/// The order is the order the census prints. Every reader is a whole count
/// in a 64-bit accumulator, so no count depends on the margin of a narrower
/// type.[^1]
///
/// **No row reports one tick.** A row states its basis. A held row counts
/// what the world holds now. A total row counts what the run made since the
/// world was built, and a zero in it means that the thing never happened.
/// Two rows once reported the last tick beside rows that reported the run,
/// and a reader could not tell which was which.[^2]
///
/// # References
///
/// [^1]: ADR-0002, simulated and aggregated state holds no floating point number, decision D3. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
/// [^2]: Findings register, FND-498. `docs/FINDINGS.md`
pub const SUBSYSTEM_CENSUS: &[CensusRow] = &[
    CensusRow {
        name: "units",
        basis: CensusBasis::Held,
        read: |world| i64::from(world.soldiers.len()),
    },
    CensusRow {
        name: "settlements",
        basis: CensusBasis::Held,
        read: |world| i64::from(world.settlements.len()),
    },
    // The people the growth stage added over the run. The stage counts one
    // tick and clears that count before it acts, so the world folds the tick
    // into a run total at the site that clears it.[^6]
    //
    // [^6]: ADR-0082, the store sets the rate of a birth and the housing admits it, decision D1. `docs/adrs/draft/adr-0082-the-store-sets-the-rate-of-a-birth-and-the-housing-admits-it.md`
    CensusRow {
        name: "births",
        basis: CensusBasis::Total,
        read: |world| world.census.births + i64::from(world.births),
    },
    // What the build queue of every site has made, and what it has refused.
    // The two refusals of a finished entry stay apart, because they mean
    // different things to a watcher. A watcher then tells a site with no
    // resident to spend from a site whose store cannot pay.[^4]
    //
    // [^4]: ADR-0158, a site builds a typed unit from a bounded queue its store pays for, decision D6. `docs/adrs/accepted/adr-0158-a-site-builds-a-typed-unit-from-a-bounded-queue-its-store-pays-for.md`
    CensusRow {
        name: "queue_produced",
        basis: CensusBasis::Total,
        read: |world| world.census.queue_produced + i64::from(world.queues.produced()),
    },
    CensusRow {
        name: "queue_refused_without_a_person",
        basis: CensusBasis::Total,
        read: |world| {
            world.census.queue_refused_without_a_person
                + i64::from(world.queues.refused_without_a_person())
        },
    },
    CensusRow {
        name: "queue_refused_without_goods",
        basis: CensusBasis::Total,
        read: |world| {
            world.census.queue_refused_without_goods
                + i64::from(world.queues.refused_without_goods())
        },
    },
    CensusRow {
        name: "queue_refused_at_the_verb",
        basis: CensusBasis::Total,
        read: |world| {
            world.census.queue_refused_at_the_verb + i64::from(world.queues.refused_at_the_verb())
        },
    },
    CensusRow {
        name: "seats_filled",
        basis: CensusBasis::Held,
        read: |world| {
            world
                .positions
                .rows()
                .iter()
                .filter(|seat| seat.exists() && seat.holder_bits() != 0)
                .count() as i64
        },
    },
    CensusRow {
        name: "characters",
        basis: CensusBasis::Held,
        read: |world| world.characters.iter().count() as i64,
    },
    CensusRow {
        name: "tiles_burning",
        basis: CensusBasis::Held,
        read: |world| world.fire.burning_count() as i64,
    },
    CensusRow {
        name: "fires_started",
        basis: CensusBasis::Total,
        read: |world| world.fire.started_total(),
    },
    CensusRow {
        name: "tiles_burnt_out",
        basis: CensusBasis::Total,
        read: |world| world.fire.burnt_out_total(),
    },
    CensusRow {
        name: "fires_doused",
        basis: CensusBasis::Total,
        read: |world| world.fire.doused_total(),
    },
    CensusRow {
        name: "units_burned",
        basis: CensusBasis::Total,
        read: |world| world.fire.burned_units_total(),
    },
    CensusRow {
        name: "upgrades_complete",
        basis: CensusBasis::Held,
        read: |world| {
            world
                .upgrades
                .sites()
                .iter()
                .filter(|site| site.is_complete())
                .count() as i64
        },
    },
    CensusRow {
        name: "wonders_complete",
        basis: CensusBasis::Held,
        // The count reads the victory claim column of the row that stands on
        // each tile. It names no category.[^1]
        //
        // [^1]: ADR-0151, an upgrade is a category with a ground fit and a level, decision D4. `docs/adrs/accepted/adr-0151-an-upgrade-is-a-category-with-a-ground-fit-and-a-level.md`
        read: |world| {
            world
                .standing_rows()
                .filter(|(_, row)| row.victory_claim > 0)
                .count() as i64
        },
    },
    CensusRow {
        name: "stores_built",
        basis: CensusBasis::Held,
        // The count reads the store capacity column of the row that stands on
        // each tile. It names no category.[^1]
        //
        // [^1]: ADR-0151, an upgrade is a category with a ground fit and a level, decision D4. `docs/adrs/accepted/adr-0151-an-upgrade-is-a-category-with-a-ground-fit-and-a-level.md`
        read: |world| {
            world
                .standing_rows()
                .filter(|(_, row)| row.capacity_of_store_change > 0)
                .count() as i64
        },
    },
    CensusRow {
        name: "luxury_tiles",
        basis: CensusBasis::Held,
        read: |world| world.luxuries.len() as i64,
    },
    // The storms a god raised over the run. One call of the divine power
    // that put water into the air is one storm. The row once read whether
    // the raised total stood above zero, which counted no storm and fell
    // back to zero when the water dried.[^5]
    //
    // [^5]: Findings register, FND-498. `docs/FINDINGS.md`
    CensusRow {
        name: "storms_raised",
        basis: CensusBasis::Total,
        read: |world| world.weather.storms(),
    },
    CensusRow {
        name: "contracts",
        basis: CensusBasis::Held,
        read: |world| {
            world
                .trade
                .rows()
                .iter()
                .filter(|row| row.is_bound())
                .count() as i64
        },
    },
    // The four rows below are what the trading controller has done over the
    // run. Each one counts an act of the stage and not a state of the world,
    // in the way the controller command row does.
    CensusRow {
        name: "boards_written",
        basis: CensusBasis::Total,
        read: |world| world.census.boards_written + i64::from(world.controller.boards_written()),
    },
    CensusRow {
        name: "offers_made",
        basis: CensusBasis::Total,
        read: |world| world.census.offers_made + i64::from(world.controller.offers_made()),
    },
    CensusRow {
        name: "contracts_bound",
        basis: CensusBasis::Total,
        read: |world| world.census.contracts_bound + i64::from(world.controller.contracts_bound()),
    },
    CensusRow {
        name: "carriers_assigned",
        basis: CensusBasis::Total,
        read: |world| {
            world.census.carriers_assigned + i64::from(world.controller.carriers_assigned())
        },
    },
    CensusRow {
        name: "controller_commands",
        basis: CensusBasis::Total,
        read: |world| world.census.controller_commands + i64::from(world.controller.applied()),
    },
    CensusRow {
        name: "controller_refused",
        basis: CensusBasis::Total,
        read: |world| world.census.controller_refused + i64::from(world.controller.refused()),
    },
    // What the plans of every faction have taken, finished, dropped and
    // refused. The record asks that a drop and a refusal each be counted.[^3]
    //
    // **The drop row and the refusal row are disjoint.** A write the plan
    // turned away at its bound is a drop and nothing else, and every other
    // refusal of a write or a build is a refusal and nothing else. A reader
    // adds the two rows and counts each act once. The two once overlapped,
    // and the sum double-counted a full plan.[^7]
    //
    // [^7]: Findings register, FND-496. `docs/FINDINGS.md`
    // [^3]: ADR-0152, a faction plans its roads and zones with one solver, decisions D1, D4 and D5. `docs/adrs/accepted/adr-0152-a-faction-plans-its-roads-and-zones-with-one-solver.md`
    CensusRow {
        name: "projects_zoned",
        basis: CensusBasis::Total,
        read: |world| world.plan.zoned_count(),
    },
    CensusRow {
        name: "projects_finished",
        basis: CensusBasis::Total,
        read: |world| world.plan.finished_count(),
    },
    CensusRow {
        name: "projects_dropped",
        basis: CensusBasis::Total,
        read: |world| world.plan.dropped_count(),
    },
    CensusRow {
        name: "projects_refused",
        basis: CensusBasis::Total,
        read: |world| world.plan.refused_count(),
    },
    CensusRow {
        name: "plan_passes",
        basis: CensusBasis::Total,
        read: |world| world.plan.pass_count(),
    },
    CensusRow {
        name: "game_ended",
        basis: CensusBasis::Total,
        read: |world| i64::from(world.controller.game_end().is_set()),
    },
    // The relation moves the controller made through the verb over the run,
    // and the crossings into the war band over the run, from any cause.
    CensusRow {
        name: "relation_moves",
        basis: CensusBasis::Total,
        read: |world| world.census.relation_moves + world.relation_moves_of_the_log(),
    },
    CensusRow {
        name: "wars_declared",
        basis: CensusBasis::Total,
        read: |world| world.census.wars_declared + world.relations.declarations(),
    },
    // The campaigns raised over the run, by the controller or by a caller,
    // and the campaigns whose objective passed to the campaigner.
    CensusRow {
        name: "campaigns_raised",
        basis: CensusBasis::Total,
        read: |world| world.census.campaigns_raised + world.campaigns.count(campaign::EVENT_RAISED),
    },
    CensusRow {
        name: "campaigns_won",
        basis: CensusBasis::Total,
        read: |world| world.census.campaigns_won + world.campaigns.count(campaign::EVENT_WON),
    },
    // The two acts of a conquest over the run. A taker keeps a city its own
    // reach supplies and burns one it does not, and these two rows are the
    // only place a reader sees which of the two a run reached.[^8]
    //
    // [^8]: ADR-0180, a site changes hands or the taker destroys it, decision D7. `docs/adrs/draft/adr-0180-a-site-changes-hands-or-the-taker-destroys-it.md`
    CensusRow {
        name: "sites_captured",
        basis: CensusBasis::Total,
        read: |world| world.sites_captured,
    },
    CensusRow {
        name: "sites_razed",
        basis: CensusBasis::Total,
        read: |world| world.sites_razed,
    },
    CensusRow {
        name: "sieges_pressed",
        basis: CensusBasis::Total,
        read: |world| world.sieges_pressed,
    },
    CensusRow {
        name: "sieges_relieved",
        basis: CensusBasis::Total,
        read: |world| world.sieges_relieved,
    },
];

impl World {
    /// Adds what the controller log holds to the run total.
    ///
    /// The caller calls this immediately before it empties the log.
    pub(super) fn fold_the_controller_into_the_census(&mut self) {
        let moves = self.relation_moves_of_the_log();
        let totals = &mut self.census;
        totals.controller_commands += i64::from(self.controller.applied());
        totals.controller_refused += i64::from(self.controller.refused());
        totals.relation_moves += moves;
        totals.boards_written += i64::from(self.controller.boards_written());
        totals.offers_made += i64::from(self.controller.offers_made());
        totals.contracts_bound += i64::from(self.controller.contracts_bound());
        totals.carriers_assigned += i64::from(self.controller.carriers_assigned());
    }

    /// Adds what the campaign log holds to the run total.
    ///
    /// The caller calls this immediately before it empties the log.
    pub(super) fn fold_campaigns_into_the_census(&mut self) {
        let raised = self.campaigns.count(campaign::EVENT_RAISED);
        let won = self.campaigns.count(campaign::EVENT_WON);
        self.census.campaigns_raised += raised;
        self.census.campaigns_won += won;
    }

    /// Adds what the relation log holds to the run total.
    ///
    /// The caller calls this immediately before it empties the log.
    pub(super) fn fold_relations_into_the_census(&mut self) {
        let declarations = self.relations.declarations();
        self.census.wars_declared += declarations;
    }

    /// Adds what the queue counts hold to the run total.
    ///
    /// The caller calls this immediately before it empties the counts.
    pub(super) fn fold_queues_into_the_census(&mut self) {
        let totals = &mut self.census;
        totals.queue_produced += i64::from(self.queues.produced());
        totals.queue_refused_without_a_person += i64::from(self.queues.refused_without_a_person());
        totals.queue_refused_without_goods += i64::from(self.queues.refused_without_goods());
        totals.queue_refused_at_the_verb += i64::from(self.queues.refused_at_the_verb());
    }

    /// Returns the subsystem census: one count for each row of the one
    /// table, in table order.
    #[must_use]
    pub fn subsystem_census(&self) -> Vec<(&'static str, i64)> {
        SUBSYSTEM_CENSUS
            .iter()
            .map(|row| (row.name, (row.read)(self)))
            .collect()
    }
}
