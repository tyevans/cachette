//! The state hash of the world.
//!
//! The hash folds every column of the world into one value, in a fixed order.
//! A golden file holds the value the engine must reproduce, so this pass sits
//! alone: a reader who changes it changes what every stored hash means.

use super::World;
use crate::hash::StateHash;

impl World {
    /// Returns the hash of the whole state.
    ///
    /// The golden test compares this value against a stored file.[^1]
    ///
    /// **Every stored value that a pass or a verb reads enters this hash.** A
    /// derived projection stays out, and the inputs of the derivation enter
    /// instead. A parameter that decides what a pass writes enters, even when
    /// the column the pass writes is already here: a hash of the effect
    /// reports a changed parameter one or more ticks after the change.[^2]
    ///
    /// A new value needs a test that changes it through the public interface
    /// and asserts that the hash differs. The golden file cannot do that work,
    /// because it cannot say which input the hash stopped depending on.[^3]
    ///
    /// # References
    ///
    /// [^1]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
    /// [^2]: ADR-0164, every stored value the step reads enters the state hash, decisions D1 to D3. `docs/adrs/draft/adr-0164-every-stored-value-the-step-reads-enters-the-state-hash.md`
    /// [^3]: ADR-0164, every stored value the step reads enters the state hash, decision D4. `docs/adrs/draft/adr-0164-every-stored-value-the-step-reads-enters-the-state-hash.md`
    #[must_use]
    pub fn state_hash(&self) -> StateHash {
        let hash = StateHash::new()
            .write_u64(self.tick.0)
            .write_u64(self.config.seed)
            .write_u64(u64::from(self.config.width))
            .write_u64(u64::from(self.config.height))
            .write_u64(u64::from(self.config.faction_count));
        // The tile value field is generated from the seed and stores only
        // what the frames changed. The hash writes the value of every tile
        // and not the stored part alone: the seed and the extent above are
        // the inputs of the generator, and a change to the generator moves
        // every tile of every world while leaving both untouched.[^3]
        //
        // [^3]: ADR-0068, terrain is generated from the seed and is never stored as a map, the consequences. `docs/adrs/accepted/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
        let hash = self.values.hash_into(hash);
        // The ground is part of the world, so the whole-world hash covers
        // it. The seed and the extent are already above, but they are the
        // inputs of the generator, not its output. A change to the generator
        // moves every tile of every world, and only the tiles report it.[^1]
        //
        // [^1]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
        let hash = self.terrain.hash_into(hash);
        // The stock a tile started with is generated, so the same argument
        // holds for it: the seed is the input of the generator and only the
        // tiles report a change to the generator itself.
        let hash = self.resources.hash_into(hash);
        // The ledger writes the recovery rules with its entries. The pass
        // reads a period on every tick, and a hash that wrote the takes and
        // not the periods reports the effect one or more ticks after the
        // cause.[^20]
        //
        // [^20]: Findings register, FND-480. `docs/FINDINGS.md`
        let mut hash = self.depletion.hash_into(hash);
        for amount in &self.departed {
            hash = hash.write_u64(*amount);
        }
        // What has been delivered is a stored total that a later frame reads,
        // in the way the departed total is. It is the term that links the
        // carry account to the store account.
        for amount in &self.delivered {
            hash = hash.write_u64(*amount);
        }
        // An upgrade is the difference between the world the generator made
        // and the world the units made. It is simulated state, and an
        // unfinished build is state that the next frame reads, so both the
        // kind and the progress enter the hash.[^2]
        //
        // [^2]: ADR-0090, a tile upgrade is stored sparsely, as the difference from the generated world, decision D2. `docs/adrs/draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md`
        let hash = self.upgrades.hash_into(hash);
        // Who holds each tile is simulated state, so the whole-world hash
        // covers it.
        let hash = self.holding.hash_into(hash);
        // The remembered layer of each faction is state that a later frame
        // reads, and the sight rules decide what the next rebuild writes, so
        // both enter. The layer of what a faction sees now is derived from
        // the units and the rules, so it stays out and its inputs enter
        // instead.[^21]
        //
        // [^21]: ADR-0164, every stored value the step reads enters the state hash, decisions D1 to D3. `docs/adrs/draft/adr-0164-every-stored-value-the-step-reads-enters-the-state-hash.md`
        let hash = self.observation.hash_into(hash);
        // The unit type table decides what the next meeting does, so the
        // whole-world hash covers it. Two worlds that hold the same units and
        // different tables must diverge at the next meeting.
        let hash = self.unit_types.hash_into(hash);
        // The queue of every site is state that a later frame reads, and the
        // cost table decides what a later frame does, so both enter. The
        // schedule decides which ticks act, so it enters too.[^17]
        //
        // [^17]: ADR-0158, a site builds a typed unit from a bounded queue its store pays for, decision D1. `docs/adrs/accepted/adr-0158-a-site-builds-a-typed-unit-from-a-bounded-queue-its-store-pays-for.md`
        let hash = self.queues.hash_into(hash);
        let hash = self.build_costs.hash_into(hash);
        let hash = hash
            .write_u64(u64::from(self.queue_schedule.period()))
            .write_u64(u64::from(self.queue_schedule.phase()));
        // The growth parameters decide what a later frame does, so the
        // whole-world hash covers them. The housing of each site is a column
        // of the settlement arena, and that arena hashes its own columns.[^18]
        //
        // [^18]: ADR-0157, a site's free places are its built housing less the residents the engine counts, decision D6. `docs/adrs/accepted/adr-0157-a-sites-free-places-are-its-built-housing-less-the-residents-the-engine-counts.md`
        let hash = hash
            .write_u64(u64::from(self.growth_schedule.period()))
            .write_u64(u64::from(self.growth_schedule.phase()))
            .write_u64(u64::from(self.housing_per_person))
            .write_u64(u64::from(self.founding_housing))
            .write(&self.birth_chance.0.to_le_bytes());
        let hash = self
            .food_per_birth
            .iter()
            .fold(hash, |hash, cost| hash.write(&cost.0.to_le_bytes()));
        // The upgrade table decides what a build order does and what an
        // upgrade changes, so the whole-world hash covers it. Two worlds
        // built with different tables never hash the same.[^5]
        //
        // [^5]: ADR-0151, an upgrade is a category with a ground fit and a level, decision D1. `docs/adrs/accepted/adr-0151-an-upgrade-is-a-category-with-a-ground-fit-and-a-level.md`
        let hash = self.upgrade_table.hash_into(hash);
        // A luxury is authored rather than generated, so no input above
        // produces it. Two worlds that carry different luxuries are
        // different worlds, and only the field says so.[^4]
        //
        // [^4]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
        let hash = self.luxuries.hash_into(hash);
        let hash = self.soldiers.hash_into(hash);
        let hash = self.settlements.hash_into(hash);
        let hash = self.characters.hash_into(hash);
        // A rate is state that a later frame reads, and the ledger is the
        // record of what the rates have already done. A hash that covered
        // the stores and neither of these would report the same value for
        // two worlds that must diverge on the next application.
        let hash = self
            .rates
            .hash_into(hash)
            .write_u64(u64::from(self.schedule.period()))
            .write_u64(u64::from(self.schedule.phase()));
        let hash = self.rate_ledger.hash_into(hash);
        // What growth took is a term of the world conservation statement, in
        // the way the rate ledger is, so the whole-world hash covers it.
        let hash = self
            .growth_ledger
            .iter()
            .fold(hash, |hash, total| hash.write(&total.0.to_le_bytes()));
        // The need of a unit and the deficit that follows it are simulated
        // state, and the unit columns already carry them into the hash. The
        // rule, the cohorts and the draw ledger are the rest of the pass:
        // two worlds that hold the same needs and different rules must
        // diverge on the next application.
        let hash = self.need_rule.hash_into(hash);
        let hash = self.cohorts.hash_into(hash);
        // What each faction reaches is state that a later frame reads: the
        // next solve starts from the field this one left. Two worlds that
        // hold the same tiles and different fields must diverge.
        let mut hash = self.influence.hash_into(hash);
        // The seed set of each destination is state that a later frame reads.
        // The field itself is derived from it and from level 1, and a derived
        // value states no fact of its own, so the seeds enter the hash and the
        // field does not. The send column of each unit is already in the
        // arena hash above.[^4]
        //
        // [^4]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D1. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
        for tiles in &self.destination_seeds {
            hash = hash.write_u64(tiles.len() as u64);
            for tile in tiles {
                hash = hash.write(&tile.0.to_le_bytes());
            }
        }
        // Whether a plane conducts through water is stored beside its seeds,
        // and the derivation reads it, so it enters the hash for the reason
        // the seeds do.
        for crossing in &self.destination_crossings {
            hash = hash.write(&crossing.to_le_bytes());
        }
        // A position is state that a later frame reads: a unit that holds
        // one still holds it on the next frame, and the preference decides
        // what the next rebalance opens. Two worlds that hold the same
        // stores and different preferences must diverge.
        let hash = self
            .positions
            .hash_into(hash)
            .write_u64(u64::from(self.position_schedule.period()))
            .write_u64(u64::from(self.position_schedule.phase()))
            // The promotion schedule decides which frames promote, so two
            // worlds that differ in it diverge and the hash must say so. The
            // deed threshold and the columns it governs are folded by the
            // unit arena, which holds them.
            .write_u64(u64::from(self.character_schedule.period()))
            .write_u64(u64::from(self.character_schedule.phase()))
            .write_u64(u64::from(self.promotion_budget));
        // The four parameters of the choice are read on every tick, and each
        // one decides what a unit does next. The intent column carries the
        // outcome, and an outcome states the effect and never the cause: two
        // worlds that hold the same intents and different parameters hash the
        // same and diverge at the next frame that chooses.[^21]
        //
        // [^21]: Findings register, FND-537. `docs/FINDINGS.md`
        let hash = self
            .choice
            .hash_into(hash)
            .write_u64(u64::from(self.carry_mark.0));
        let hash = self.buckets.hash_into(hash);
        let hash = self.weights.hash_into(hash);
        // The bound on a land side is a parameter that a verb reads. Two
        // worlds that differ in it answer the same offer differently.
        let hash = hash.write_u64(u64::from(self.land_list_bound));
        // Whether the world took a luxury seed is a fact the field cannot
        // state: a seed that placed nothing leaves the field as an unseeded
        // world leaves it, and the two worlds refuse and accept the next seed
        // differently.[^22]
        //
        // [^22]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
        let hash = hash.write_u64(u64::from(self.luxuries_seeded));
        let mut hash = self.draw_ledger.hash_into(hash);
        for total in &self.store_account {
            hash = hash.write_u64(total.0 as u64);
        }
        // Whether a faction has left the game is stored state that the step
        // reads. The elimination pass reads it to leave a faction that has
        // gone alone, and every game end reader reads it to refuse a faction
        // that is out, so it enters the hash.[^24]
        //
        // [^24]: ADR-0164, every stored value the step reads enters the state hash. `docs/adrs/draft/adr-0164-every-stored-value-the-step-reads-enters-the-state-hash.md`
        let hash = hash.write(&self.eliminated);
        // What two factions agreed is state that a later frame reads: the
        // settlement pass moves a quantity because a contract says so. The
        // plane holds no row until somebody speaks, and it then folds nothing,
        // so a world that never traded hashes as it did before trade
        // existed.[^16]
        //
        // [^16]: ADR-0126, a trade negotiation is engine state and the words are not, decision D1. `docs/adrs/draft/adr-0126-a-trade-negotiation-is-engine-state.md`
        let hash = self.trade.hash_into(hash);
        // The controller rows, its two parameters and the game end record are
        // each read by a later frame. The evaluation count and the tick limit
        // enter for the reason the recovery rules do: a value the step reads
        // on every tick and that the hash does not cover lets two worlds hash
        // the same and diverge on the next tick.[^18]
        //
        // [^18]: ADR-0148, a game end is recorded once and stops the controllers, decision D1. `docs/adrs/accepted/adr-0148-a-game-end-is-recorded-once-and-stops-the-controllers.md`
        let hash = self.controller.hash_into(hash);
        // The win-path balance values are read by a later frame: a reader
        // compares one on every tick, and the renown pass adds another. Two
        // worlds that differ only in a balance value must diverge, and the
        // hash must say so.[^20]
        //
        // [^20]: ADR-0175, a win threshold decides when a reader fires and never what the simulation does, decision D1. `docs/adrs/draft/adr-0175-a-win-threshold-decides-when-a-reader-fires.md`
        let hash = self.balance.hash_into(hash);
        // What each faction marches on is state that a later frame reads: the
        // stage closes a campaign against the holder it recorded at the
        // raise, and the raise refuses while one is live.
        let hash = self.campaigns.hash_into(hash);
        // The plan of each faction decides where a unit may build, and the
        // step reads it, so two worlds that differ only in a plan must
        // diverge and the hash must say so.[^7]
        //
        // [^7]: ADR-0152, a faction plans its roads and zones with one solver, decision D1. `docs/adrs/accepted/adr-0152-a-faction-plans-its-roads-and-zones-with-one-solver.md`
        let hash = self.plan.hash_into(hash);
        // What each faction advertises is state that a controller reads. The
        // table holds no row until somebody advertises, and it then folds
        // nothing, so a world with no board hashes as it did before.
        let hash = self.market.hash_into(hash);
        // What each faction feels toward each other is state that a later
        // frame reads: the next contest fires or not because of it. The edges
        // and the steps enter with the entries, for the reason the controller
        // parameters do.[^19]
        //
        // [^19]: ADR-0146, a faction relation is one signed integer per ordered pair, and a pass reads a threshold, decision D1. `docs/adrs/accepted/adr-0146-a-faction-relation-is-one-signed-integer-per-ordered-pair-and-a-pass-reads-a-threshold.md`
        let hash = self.relations.hash_into(hash);
        // The water in the air and on the ground is state that a later frame
        // reads: the next solve starts from the field this one left. Two
        // worlds that hold the same tiles and different weather must
        // diverge.[^17]
        //
        // [^17]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
        let hash = self.weather.hash_into(hash);
        // What burns is state that a later frame reads: the next tick spreads
        // from the tiles this one left burning, and it refuses the tiles this
        // one left spent. The lightning chance enters with it, because two
        // worlds that hold the same fire and different chances diverge on the
        // next strike.[^17]
        let hash = self.fire.hash_into(hash).write_u64(self.lightning_chance);
        // The climate is stored, and the terrain readers of this world read
        // it, so two worlds that hold the same seed and different climates
        // must diverge.[^17]
        self.climate.hash_into(hash)
    }
}
