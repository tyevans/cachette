//! The frame step, in the order its stages run.
//!
//! The step is the one caller that drives every stage of one tick. It writes
//! each parallel result to an indexed output slot, it draws every random
//! value from a counter, and it routes all arithmetic through the arithmetic
//! module. It sits alone, because the order of its stages is the thing a
//! reader comes here for.

use super::admission::{admit, Guests};
use super::errors::StepError;
use super::movement::{soldier_moves, Destinations, Steering, UnitWalk};
use super::tiles::{update_range, ChunkResult};
use super::upgrades::Building;
use super::World;
use crate::event::TileChanged;
use crate::slots::Slots;
use crate::stage::{self, Stage};
use crate::types::Tick;

impl World {
    /// Runs one frame on the given number of threads.
    ///
    /// The result does not depend on the thread count. Every thread writes
    /// to its own output slot, and the step joins the slots in slot order.
    ///
    /// # Errors
    ///
    /// Returns an error when the caller asks for zero threads.
    pub fn step(&mut self, threads: usize) -> Result<&[TileChanged], StepError> {
        if threads == 0 {
            return Err(StepError::ZeroThreads);
        }

        self.trade_log.clear();
        // **These three logs are cleared here, before any system runs.** The
        // rule for every log in this engine is the same: a log holds what
        // happened since the last step began, and a reader that misses a step
        // misses the events. A settlement founded by a caller between two
        // steps therefore survives to the next read.
        self.collapsed_log.clear();
        self.finished_log.clear();
        self.founded_log.clear();
        self.taken_log.clear();
        self.fire_started_log.clear();
        self.fire_ended_log.clear();
        self.burned_log.clear();
        self.eliminated_log.clear();
        self.fold_relations_into_the_census();
        self.relations.clear_log();
        self.tick = Tick(self.tick.0.wrapping_add(1));

        let tick = self.tick;
        let seed = self.config.seed;
        let count = self.grid.tile_count();
        let chunk_len = (count as usize).div_ceil(threads).max(1) as u32;

        let mut slots: Slots<ChunkResult> =
            Slots::filled(threads, ChunkResult::default()).map_err(|_| StepError::ZeroThreads)?;

        // **Each worker writes its own contiguous range of the field.** The
        // field hands out disjoint chunks, so no two workers touch one tile
        // and no worker needs an atomic. The requirement on a parallel stage
        // is met by the type and not by a rule a reviewer has to check.[^12]
        //
        // The field used to be read here and written afterwards, by a merge
        // that took the joined changes, sorted them and rebuilt the stored
        // list. That merge is gone. It cost a share of the frame that grew
        // with the tiles that had ever changed, and the dense array it now
        // writes needs no run, no sort and no join.[^14] [^15]
        //
        // [^12]: ADR-0009, parallel stages write disjoint outputs, decision D1. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
        // [^14]: ADR-0103, the tile value field stores a dense delta, never a sparse change list, the consequences. `docs/adrs/draft/adr-0103-the-tile-value-field-stores-a-dense-delta.md`
        // [^15]: Findings register, FND-292. `docs/FINDINGS.md`
        {
            let _span = stage::open(Stage::TileScan);
            // The array is allocated here, on the first frame that runs, and
            // never when the world is built.
            self.values.prepare();
            crate::parallel::fan_out_each(
                slots
                    .entries_mut()
                    .iter_mut()
                    .zip(self.values.chunks_mut(chunk_len))
                    .map(move |(slot, chunk)| {
                        move || {
                            *slot = update_range(tick, seed, chunk);
                        }
                    }),
            );
        }

        // The count of changed tiles is a sum over the chunks. Addition of
        // integers does not depend on the order of the terms, so the count is
        // the same at any thread count.[^13]
        //
        // [^13]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
        self.values
            .absorb_changed(slots.combine(0i64, |total, slot| total + slot.changed));

        {
            let _span = stage::open(Stage::LogJoin);
            let mut log = core::mem::take(&mut self.log);
            log.clear();
            self.log = slots.combine(log, |mut joined, slot| {
                joined.extend_from_slice(&slot.events);
                joined
            });
        }

        // Every soldier chooses a neighbour, then the step applies the
        // choices. The choice is a pure read of the world, so the two halves
        // never interleave and no soldier sees a half-applied world.[^1]
        // A spawn or a despawn made between two frames is a structural change
        // that has not passed a barrier, and it leaves the derived structure
        // stale. Admission reads the occupancy of a target from that
        // structure, so the step opens by giving those changes their
        // barrier.[^4]
        //
        // This is not a second barrier. The rebuild at the end of this
        // function is the barrier of this frame, and it stays last.
        {
            let _span = stage::open(Stage::BridgeRefreshOpening);
            self.refresh_bridge()?;
        }

        // The choice runs before movement, and it is what movement reads.
        // It reads level 1 as the last barrier left it, and it writes
        // nothing to any level above level 0.[^11]
        //
        // [^11]: ADR-0022, level 0 is the only truth, and every level above it is derived, decisions D1 and D3. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
        {
            let _span = stage::open(Stage::Choose);
            self.choose(threads)?;
        }

        // The walk order of the movement pass. The bridge holds every live
        // unit in block-major tile order, and it sorted them at the barrier,
        // so this order costs the frame nothing more than it already paid.
        let live = self.bridge.units(&self.soldiers)?.to_vec();
        let intents = {
            let _span = stage::open(Stage::MovementIntents);
            soldier_moves(
                tick,
                seed,
                self.terrain,
                &UnitWalk {
                    soldiers: &self.soldiers,
                    live: &live,
                },
                &Steering {
                    layout: self.pyramid.layout(),
                    exits: &self.exits,
                    approaches: &self.approaches,
                    home_approaches: &self.home_approaches,
                    stock_approaches: &self.stock_approaches,
                    site_tiles: self.settlements.tile_column(),
                    returns: &self.returns,
                    destinations: &self.destinations,
                },
                &Building {
                    holding: &self.holding,
                    plan: &self.plan,
                    unit_types: &self.unit_types,
                    upgrades: &self.upgrades,
                    table: &self.upgrade_table,
                    fire: &self.fire,
                },
                threads,
            )?
        };

        // Admission grants the intents. It reads the occupancy of a target
        // from the derived structure, which the last barrier rebuilt, so it
        // must run before anything moves.[^3]
        let granted = {
            let _span = stage::open(Stage::Admit);
            admit(
                &intents,
                &self.soldiers,
                &self.bridge,
                self.terrain,
                &self.upgrades,
                &self.upgrade_table,
                self.grid,
                Guests {
                    holders: self.holding.holders(),
                    relations: &self.relations,
                },
                threads,
            )?
        };
        {
            let _span = stage::open(Stage::PlaceGranted);
            for (soldier, address) in granted {
                self.soldiers
                    .place(soldier, address)
                    .expect("the granted address is inside the world and admits a unit");
            }
            // **The release runs here, in the stage that applies the step,
            // and it reads the tile the unit now stands on.** A unit that
            // reached the tile it was sent to is free from this line on, so
            // it reads its option row again and it gathers and delivers.
            // Nothing released a sent unit before, and the defect was
            // invisible for as long as arrival was impossible.[^31]
            //
            // The release takes no frame from the unit. The choice pass
            // already wrote an intent for it, and the gather and the delivery
            // of this frame run below and read no send, so a unit released
            // here acts on the frame it is released.
            //
            // [^31]: Findings register, FND-572. `docs/FINDINGS.md`
            self.release_sent_units();
        }

        // The bridge rebuilds here, at the barrier, and after the structural
        // apply. Rebuilding before the apply would leave a dead identity in
        // the unit array for the whole frame.[^2] The movement above is the
        // structural apply, so this call stays last in the step.
        //
        // [^1]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D2. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
        // [^2]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D3. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
        // [^3]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D3. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
        // [^4]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D3. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
        {
            let _span = stage::open(Stage::BridgeRefreshBarrier);
            self.refresh_bridge()?;
        }

        // The gather resolve runs after the barrier of this frame. It reads
        // where each unit stands, and the movement above has just moved
        // them, so a resolve before the barrier would take from the tile the
        // unit left.[^6]
        //
        // The resolve changes no structure. It writes a load into a column
        // and an amount into the ledger, and neither moves a unit, so the
        // barrier above stays the barrier of this frame.
        //
        // [^6]: ADR-0073, gathering is admitted by sort-then-admit against the tile, decision D3. `docs/adrs/accepted/adr-0073-gathering-is-admitted-by-sort-then-admit-against-the-tile.md`
        // Recovery runs before the gather resolve, so a unit takes what the
        // deposit holds at this tick. A resolve that ran first would take
        // against an amount that the world had already moved past.[^14]
        //
        // The pass walks the depleted set and no tile, so a world that
        // gathered nothing does no work here, at any tile count.[^14]
        //
        // [^14]: ADR-0080, a depleted deposit recovers by ageing the stored take, decisions D1 and D2. `docs/adrs/accepted/adr-0080-a-depleted-deposit-recovers-by-ageing-the-stored-take.md`
        {
            let _span = stage::open(Stage::DepletionRecover);
            // The ledger comes out of the world for the pass, so that the
            // pass can read the weather and the upgrade map beside it. It
            // goes back in the same block, and nothing between the two lines
            // reads it.
            let mut depletion = core::mem::take(&mut self.depletion);
            depletion.recover(tick, &|tile| self.tile_ground(tile));
            self.depletion = depletion;
        }

        {
            let _span = stage::open(Stage::Gather);
            self.gather(threads)?;
        }

        // The build advance runs after the barrier of this frame, for the
        // same reason the gather resolve does: it reads where each unit
        // stands, and the movement above has just moved them.[^16]
        //
        // The advance writes the upgrade map and nothing else. It moves no
        // unit, so the barrier above stays the barrier of this frame.
        //
        // The pass reads the builders and the sites. It takes no grid and no
        // tile count, so a world in which nobody built does no work here, at
        // any tile count.[^16]
        //
        // [^16]: ADR-0090, a tile upgrade is stored sparsely, as the difference from the generated world, decisions D1 and D2. `docs/adrs/draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md`
        {
            let _span = stage::open(Stage::Build);
            self.build(threads)?;
        }

        // The wear runs after the build of this frame, so a worker mends its
        // site and the weather then takes from what the worker left. The
        // other order would let a site collapse in the tick a worker filled
        // it, and the worker would have bought nothing.
        //
        // The pass reads the bridge, which the barrier above rebuilt, so it
        // reads where each unit stands after the movement of this frame. It
        // writes the upgrade map and moves no unit, so the barrier above
        // stays the barrier of this frame.
        //
        // The weather solve runs later in this step, so the pass reads the
        // field the previous step left. That is a fixed order and not a stale
        // read: every tick reads the field of the tick before it.[^17]
        //
        // [^17]: ADR-0140, weather is a field over the level 1 cell lattice, decision D3. `docs/adrs/draft/adr-0140-weather-is-a-field-over-the-level-1-cell-lattice.md`
        {
            let _span = stage::open(Stage::UpgradeWear);
            self.wear_upgrades();
        }

        // The cities rewrite the holder column here, after the barrier of
        // this frame and after the build above. The reach of a city counts
        // the finished upgrades on the ground it held at the end of the
        // previous step, so the count reads the column this stage is about to
        // overwrite.[^7]
        //
        // The pass reads no unit position, so a unit gives its faction no
        // claim on the ground it stands on. It writes a tile column and moves
        // no unit, so the barrier above stays the barrier of this frame.
        //
        // [^7]: ADR-0150, held ground is the ground within reach of a city its faction owns, decisions D1, D2 and D3. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
        // **The lease moves before the holder is decided.** A tile that
        // carries a unit moves its lease one step toward the faction present,
        // and the decay then pulls every live lease toward zero on a fixed
        // schedule. The rewrite below reads the lease this pass wrote.[^21]
        //
        // The pass reads the bridge, which the barrier above rebuilt, so it
        // reads where each unit stands after the movement of this frame. It
        // writes two tile columns and moves no unit, so the barrier above
        // stays the barrier of this frame.
        //
        // [^21]: ADR-0153, a tile's lease follows the units that stand on it, decisions D2, D3, D4 and D7. `docs/adrs/accepted/adr-0153-a-tiles-lease-follows-the-units-that-stand-on-it.md`
        // The occupancy outlives this block, because the capture pass below
        // reads the same list. Who stands on a tile is stated once, and the
        // lease and the capture both read that one statement.[^22]
        let occupancy = {
            let _span = stage::open(Stage::HoldingLease);
            let occupancy = self.tile_occupancy(threads)?;
            self.holding.advance_leases(&occupancy, tick.0);
            occupancy
        };

        // **A site changes hands here, between the lease and the spread.**
        // The occupancy the lease pass built is the one statement of who
        // stands on a tile, and this pass reads the same list rather than
        // counting again.[^22] The spread below then reads the settlement
        // faction column that this pass has just written, so a city taken on
        // this tick reaches its new ground on this tick and not the next.
        //
        // The pass is serial and it walks the settlements in slot order,
        // which is a stable key.[^23]
        //
        // [^22]: ADR-0180, a site changes hands or the taker destroys it, decision D4. `docs/adrs/draft/adr-0180-a-site-changes-hands-or-the-taker-destroys-it.md`
        // [^23]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
        {
            let _span = stage::open(Stage::SiteCapture);
            self.capture_sites(&occupancy);
        }

        {
            let _span = stage::open(Stage::HoldingSpread);
            self.holding
                .rewrite(self.terrain, &self.settlements, &self.upgrades, threads)?;
        }

        // **A faction leaves the game here, after the spread.** The spread
        // above has just written the holder column, so the ground this pass
        // releases is the ground the faction holds now and not the ground it
        // held at the last barrier.[^24]
        //
        // The pass runs once for each tick. An elimination that releases
        // ground gives that ground to a rival city through the spread of the
        // next tick, by the rule that already decides every holder. Nothing
        // loops here, and no elimination cascades inside one tick.[^24]
        //
        // [^24]: ADR-0181, a faction that holds no site and no unit leaves the game, decisions D2, D3 and D4. `docs/adrs/draft/adr-0181-a-faction-that-holds-no-site-and-no-unit-leaves-the-game.md`
        {
            let _span = stage::open(Stage::FactionEliminate);
            self.eliminate_factions(threads);
        }

        // The event reports the tile as this frame left it, so the holder is
        // stamped here and not in the value pass above. The value pass runs
        // at the top of the step and the spread above is the last thing in
        // this step that writes the holder column, so a stamp taken any
        // earlier would publish the holder of the frame before. A stale read
        // is a confident wrong answer, and it is the defect this event
        // carried.[^15]
        //
        // The pass costs one write for each event, and not one for each
        // tile.
        //
        // [^15]: Findings register, FND-029 and FND-079. `docs/FINDINGS.md`
        {
            let _span = stage::open(Stage::StampHolders);
            let holders = self.holding.holders();
            for event in &mut self.log {
                event.holder = holders[event.tile.0 as usize];
            }
        }
        // The delivery runs after the gather resolve and before the rate
        // pass. It reads where each unit stands, so it runs after the barrier
        // that the movement of this frame passed. It moves a quantity, and
        // two records say that the rate pass and the consumption pass run
        // after every stage that moves one.[^16] [^17] It changes no
        // structure, so it is not a barrier and it needs none.
        //
        // [^16]: ADR-0062, production and upkeep are rates attached to a site, decision D5. `docs/adrs/accepted/adr-0062-production-and-upkeep-are-rates-attached-to-a-site.md`
        // [^17]: ADR-0063, a need is a rate with a threshold, and crossing it is a fact, decision D5. `docs/adrs/accepted/adr-0063-a-need-is-a-rate-with-a-threshold-and-crossing-it-is-a-fact.md`
        self.deliver(threads)?;

        // The meeting resolves here, after the barrier of this frame and
        // after the holding spread. It reads where each unit stands, and the
        // movement above has just moved them, so a resolution before the
        // barrier would fight on the tile a unit left.[^19]
        //
        // **It resolves at the tile, and never at a level 1 cell.** A cell
        // summarises a whole block of tiles, and a fight resolved there kills
        // units spread over all of them.[^19]
        //
        // It removes units, so it is a structural change. Nothing between
        // here and the refresh below reads the derived unit structure, and
        // that refresh is the barrier of the change.[^20]
        //
        // [^19]: ADR-0121, a meeting between two factions resolves at the tile, decisions D1 and D2. `docs/adrs/draft/adr-0121-a-meeting-between-two-factions-resolves-at-the-tile.md`
        // [^20]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D3. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
        {
            let _span = stage::open(Stage::Contest);
            self.contest(threads)?;
        }
        // The contract settlement runs directly after the ordinary delivery,
        // for the same reason that one runs where it does. It reads where each
        // unit stands, so it runs after the barrier that the movement of this
        // frame passed. It moves a quantity, so it runs before the rate pass
        // and before the consumption pass.[^18] [^19] It changes no structure,
        // so it is not a barrier and it needs none.
        //
        // [^18]: ADR-0062, production and upkeep are rates attached to a site, decision D5. `docs/adrs/accepted/adr-0062-production-and-upkeep-are-rates-attached-to-a-site.md`
        // [^19]: ADR-0128, a contract moves a quantity only when a unit carries it onto the ground of the other party, decision D3. `docs/adrs/draft/adr-0128-a-contract-moves-a-quantity-only-when-a-unit-carries-it.md`
        self.settle_trades(threads)?;

        // The site rates apply after the barrier of this frame and after the
        // gather resolve, and before level 1 rebuilds.
        //
        // The position is stated against the barrier on purpose. The pass
        // reads no derived structure and changes no structure, so it is not a
        // barrier and it does not need one. What it needs is to run after
        // everything that moves a quantity in this frame, so that the store a
        // derived level reads is the store the frame settled on. The gather
        // resolve is that work today, and level 1 is the derived level.[^8]
        //
        // The pass is skipped on a tick the schedule does not name, and the
        // schedule is a parameter of the world rather than a constant of this
        // function.[^9]
        //
        // [^8]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D2. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
        // [^9]: ADR-0062, production and upkeep are rates attached to a site, decision D4. `docs/adrs/accepted/adr-0062-production-and-upkeep-are-rates-attached-to-a-site.md`
        {
            let _span = stage::open(Stage::ApplyRates);
            self.apply_rates(threads)?;
        }

        // Consumption runs after the rates, on the same schedule. A unit
        // draws from the store of the site it belongs to, and the rates are
        // what filled that store this frame, so a draw before them would
        // spend the store of the frame before.[^10]
        //
        // The pass reads no derived structure and changes no structure, so
        // it is not a barrier. It reads the home column and the store
        // column, and it writes the need column, the deficit column and the
        // store column.
        //
        // [^10]: ADR-0063, a need is a rate with a threshold, and crossing it is a fact, decision D5. `docs/adrs/accepted/adr-0063-a-need-is-a-rate-with-a-threshold-and-crossing-it-is-a-fact.md`
        {
            let _span = stage::open(Stage::Consume);
            self.consume(threads)?;
        }

        // The scan of the death plane runs after consumption, because
        // consumption is what moves a deficit to the bound. It is a
        // structural change, so it is batched into the plane during the
        // pass and applied here, in one ascending scan, after the frame has
        // settled.[^12]
        //
        // The scan removes units, so the derived structure that the barrier
        // above rebuilt now names a dead identity. The refresh below is that
        // barrier taken again over the structural apply, and it must run
        // before the derived level reads either of them.[^13]
        //
        // [^12]: ADR Registry, row 0020. `docs/adrs/REGISTRY.md`
        // [^13]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D3. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
        {
            let _span = stage::open(Stage::Reap);
            self.reap(threads)?;
        }

        // **The fire runs here, after the pass that ends a starved unit and
        // before the barrier that follows both.** It ends units of its own,
        // and the refresh below the growth is the barrier of that structural
        // change, in the way it is the barrier of the reap above.[^40]
        //
        // It reads the bridge that the barrier of this frame rebuilt, and it
        // resolves every unit it reads through the arena, so a slot the reap
        // above freed answers `None` and the fire skips it.
        //
        // It reads the weather field that the previous step left, because the
        // weather solve runs later in this step. That is a fixed order and
        // not a stale read: every tick reads the field of the tick before
        // it, in the way the upgrade wear does.[^41]
        //
        // The stage takes no thread count. It walks the burning tiles and no
        // tile of the world, and a fire is small beside a world, so the walk
        // is the cheapest thing in the frame. A pass that takes no thread
        // count also cannot take a thread completion order.[^42]
        //
        // [^40]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D3. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
        // [^41]: ADR-0140, weather is a field over the level 1 cell lattice, decision D3. `docs/adrs/draft/adr-0140-weather-is-a-field-over-the-level-1-cell-lattice.md`
        // [^42]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
        {
            let _span = stage::open(Stage::Fire);
            self.burn();
        }

        // The queue advance runs after the shortage scan and before the
        // barrier below it. It reads the store, so it runs after the rate
        // pass and after the consumption pass, which are what move a
        // quantity in this frame.[^25] It removes a resident and adds a
        // typed unit, so it is a structural change, and the refresh below is
        // the barrier of that change.[^26]
        //
        // **It runs after the scan and not before it.** The scan holds a
        // plane of the slots it ends. A stage that freed a slot and filled it
        // again before the scan applied would give the scan a live unit that
        // it never marked.
        //
        // The stage takes no thread count. It visits the sites and their
        // entries, and it walks the units once on a tick where an entry
        // finishes.[^27]
        //
        // [^25]: ADR-0062, production and upkeep are rates attached to a site, decision D5. `docs/adrs/accepted/adr-0062-production-and-upkeep-are-rates-attached-to-a-site.md`
        // [^26]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D3. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
        // [^27]: ADR-0158, a site builds a typed unit from a bounded queue its store pays for, decision D5. `docs/adrs/accepted/adr-0158-a-site-builds-a-typed-unit-from-a-bounded-queue-its-store-pays-for.md`
        // Growth runs after the pass that ends a starved unit, in the same
        // frame. A place that a death freed this frame is free this frame,
        // and the occupancy that the admission reads is the settled one.[^28]
        // The reap above applied its plane already, so no slot this stage
        // fills can carry a death that the scan had not yet marked.
        //
        // It runs before the queue advance, because growth is the only source
        // of people and the queue is the only consumer of them.[^29] A queue
        // that spent a person before growth added one would decide against
        // the population of the frame before.
        //
        // The stage takes no thread count. It visits the sites, and it walks
        // no unit.[^30]
        //
        // [^28]: ADR-0082, the store sets the rate of a birth and the housing admits it, decision D4. `docs/adrs/draft/adr-0082-the-store-sets-the-rate-of-a-birth-and-the-housing-admits-it.md`
        // [^29]: ADR-0157, a site's free places are its built housing less the residents the engine counts, decision D5. `docs/adrs/accepted/adr-0157-a-sites-free-places-are-its-built-housing-less-the-residents-the-engine-counts.md`
        // [^30]: ADR-0157, a site's free places are its built housing less the residents the engine counts, decision D4. `docs/adrs/accepted/adr-0157-a-sites-free-places-are-its-built-housing-less-the-residents-the-engine-counts.md`
        {
            let _span = stage::open(Stage::Grow);
            self.grow();
        }
        {
            let _span = stage::open(Stage::QueueAdvance);
            self.advance_queues();
        }
        {
            let _span = stage::open(Stage::BridgeRefreshAfterReap);
            self.refresh_bridge()?;
        }

        // The positions of the sites settle after the deaths of this frame.
        // A position that named a unit the scan above ended would hold a
        // stale identity, and the invariant check refuses that state.[^17]
        //
        // [^17]: ADR-0065, a group is a site membership, not a region, decision D2. `docs/adrs/draft/adr-0065-a-group-is-a-site-membership-not-a-region.md`
        {
            let _span = stage::open(Stage::SettlePositions);
            self.settle_positions(threads)?;
        }

        // The promotion scan runs after the deaths of this frame, so it never
        // promotes a unit that the shortage ended in the same frame. It reads
        // no derived structure and changes none, so it needs no barrier of
        // its own.[^18]
        //
        // [^18]: ADR-0104, a soldier is promoted from a level that never falls, decision D5. `docs/adrs/draft/adr-0104-a-soldier-is-promoted-from-a-level-that-never-falls.md`
        {
            let _span = stage::open(Stage::Promote);
            self.promote(threads)?;
        }

        // Level 1 rebuilds after the structure it reads, and after every
        // change to level 0 that this frame made. It is derived, so it is
        // last.[^5]
        //
        // The rebuild is called here rather than through the public wrapper,
        // because the wrapper refreshes the structure first and the barrier
        // above has already done that. Two refreshes would be one decision in
        // two places, and the second would hide a rebuild that ran in the
        // wrong order: a structure left stale by a barrier out of order would
        // be quietly repaired instead of refused.
        //
        // [^5]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D2. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
        {
            let _span = stage::open(Stage::RebuildLevel1);
            self.rebuild_level_1(threads, Destinations::AtTheEndOfTheStep)?;
        }

        // The influence solve runs last, after every change this frame made
        // and after the derived level it reads was rebuilt. It runs the same
        // fixed number of passes whatever the field holds and whatever the
        // sources hold, and it takes no branch on whether a source
        // exists.[^16]
        //
        // [^16]: ADR-0087, an influence solve runs a fixed iteration count over the whole plane, decision D1. `docs/adrs/draft/adr-0087-an-influence-solve-runs-a-fixed-iteration-count.md`
        {
            let _span = stage::open(Stage::InfluenceSolve);
            self.influence.solve(threads)?;
        }

        // The presence relation is derived last, after every structural
        // change this frame made and after the holding spread that decides
        // who holds each tile. A fold before the reap would name a unit the
        // frame ended, and a fold before the spread would answer against the
        // holders of the previous frame.[^19]
        //
        // The relation is derived and never stored, so it reaches no state
        // hash and no event.[^19]
        //
        // [^19]: ADR-0111, the presence relation is derived at the end of the step and never stored as a fact, decisions D1 and D2. `docs/adrs/draft/adr-0111-the-presence-relation-is-derived-at-the-end-of-the-step.md`
        // The weather solve runs after level 1 rebuilds, because it reads
        // the height and the water share of each cell from the summaries
        // that rebuild produced. It runs the same fixed number of spread
        // passes whatever the field holds, and it takes no branch on whether
        // a storm exists.[^21]
        //
        // The gather resolve above reads the ground of the cell as the
        // previous frame left it, in the way movement reads the exit field of
        // the previous barrier. A solve placed before the gather would answer
        // from a level 1 that this frame had not yet rebuilt.[^22]
        //
        // [^21]: ADR-0141, a weather pass moves water and never scales it, decision D3. `docs/adrs/draft/adr-0141-a-weather-pass-moves-water-and-never-scales-it.md`
        // [^22]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D2. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
        {
            let _span = stage::open(Stage::WeatherSolve);
            self.weather
                .solve(tick, seed, &self.weather_ground, threads)?;
        }

        // Conversion runs after the influence solve, because it reads the
        // field that solve produced, and before the presence fold, because
        // the fold reads the faction of every unit. A conversion after the
        // fold would leave the relation answering for the factions of the
        // frame before, and the freshness check would pass, because the fold
        // records the arena revision it read.[^20] [^21]
        //
        // It changes no unit structurally, so no barrier stands between it
        // and the fold. It does raise the arena revision, because the
        // relation below is derived from the faction column.[^22]
        //
        // [^20]: ADR-0087, an influence solve runs a fixed iteration count over the whole plane, decision D1. `docs/adrs/draft/adr-0087-an-influence-solve-runs-a-fixed-iteration-count.md`
        // [^21]: ADR-0133, a unit converts to the faction that leads the influence field at its cell, decision D5. `docs/adrs/draft/adr-0133-a-unit-converts-to-the-faction-that-leads-the-field.md`
        // [^22]: ADR-0111, the presence relation is derived at the end of the step and never stored as a fact, decision D4. `docs/adrs/draft/adr-0111-the-presence-relation-is-derived-at-the-end-of-the-step.md`
        {
            let _span = stage::open(Stage::Convert);
            self.convert(threads)?;
        }
        {
            let _span = stage::open(Stage::PresenceFold);
            self.presence
                .rebuild(&self.soldiers, &self.holding, threads)?;
        }
        // The observation pass runs after every change to a unit position
        // and after every change to a unit faction, because it reads both.
        // A pass placed before the reap would give sight to a unit the frame
        // ended, and a pass placed before the conversion would give the
        // tiles of a converted unit to the faction it left.[^26]
        //
        // The layer of what a faction sees now is derived here. The layer of
        // what a faction has ever seen is state, and this pass is the only
        // writer of it.[^26]
        //
        // [^26]: ADR-0059, fog storage grows with observed area, not with world area, decisions D1 and D3. `docs/adrs/accepted/adr-0059-fog-storage-grows-with-observed-area.md`
        {
            let _span = stage::open(Stage::Observe);
            self.observation
                .rebuild(&self.soldiers, self.terrain, threads, tick)?;
        }
        // The drift runs after every cause of this frame has written the
        // relation and before the controller reads it, so the controller
        // plans against the relation this frame settled on. It takes no
        // thread count: the matrix follows the square of the faction ceiling
        // and no term follows the population.[^25]
        //
        // [^25]: ADR-0146, a faction relation is one signed integer per ordered pair, and a pass reads a threshold, decision D3. `docs/adrs/accepted/adr-0146-a-faction-relation-is-one-signed-integer-per-ordered-pair-and-a-pass-reads-a-threshold.md`
        {
            let _span = stage::open(Stage::RelationDrift);
            self.relations.drift(tick);
        }
        // The controller runs last, after every derived structure of the
        // frame describes the frame. It takes no thread count. It checks the
        // game end readers first, and it emits nothing once the record is
        // written.[^23] [^24]
        //
        // [^23]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decision D1. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
        // [^24]: ADR-0148, a game end is recorded once and stops the controllers, decision D4. `docs/adrs/accepted/adr-0148-a-game-end-is-recorded-once-and-stops-the-controllers.md`
        {
            let _span = stage::open(Stage::Controller);
            // **The controller sends several times in one frame.** Each send
            // leaves the field derived for a caller that reads it between two
            // steps, and inside a frame that derivation is thrown away by the
            // next send. The flag says that the step derives the field below,
            // so the frame pays for one derivation.[^26]
            //
            // [^26]: Findings register, FND-664. `docs/FINDINGS.md`
            self.destinations_deferred = true;
            self.run_controller();
            self.destinations_deferred = false;
        }
        // **The destination field is derived here, and once at most for each
        // frame.** The barrier above left it, and the controller changed the
        // seed set of a plane whenever it sent that plane somewhere new. This
        // is the last thing in the frame that touches the field, and a caller
        // between two steps reads what it wrote.[^26]
        //
        // **A frame that changed no seed set derives nothing.** The field
        // reads the seeds, the crossings and the ground, the ground never
        // changes, and the controller re-sends the same objective on most
        // frames. The field then already holds the answer a derivation would
        // give.[^43]
        //
        // [^43]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D2. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
        //
        // The probe switch removes this derivation, so the field describes
        // the seed set as it stood before the controller sent anything. A
        // test that a stale field would pass proves nothing.
        #[cfg(not(feature = "probe-stale-destinations"))]
        {
            let _span = stage::open(Stage::RebuildDestinations);
            self.derive_changed_destination_fields();
        }
        // **A step leaves the world readable.** The controller founds cities,
        // and a founding seats a group and spends the settler. Both change
        // the unit arena, so the refresh above no longer describes it. The
        // derived unit structure counts the changes of the arena and refuses
        // every answer once the counts differ, so a reader between two steps
        // met a refusal on each tick a faction founded.[^25]
        //
        // The opening refresh of the next step would repair it, so the
        // simulation carried on and only a reader saw the fault. A reader is
        // not obliged to check, and the drawing is not the only one, so the
        // step repairs it here instead.
        //
        // **The refresh costs nothing on a tick that founded nothing.** It
        // compares the two counts and returns, and it rebuilds only when the
        // controller changed the arena. A tick that changed the arena pays
        // for one rebuild, and no tick pays for two.
        //
        // [^25]: ADR-0018, the unit to tile bridge is derived and rebuilds at the barrier, decisions D3 and D4. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
        {
            let _span = stage::open(Stage::BridgeRefreshClosing);
            self.refresh_bridge()?;
        }
        self.advance_event_memory();
        Ok(&self.log)
    }

    /// Restores the derived unit structure after a verb changed the arena.
    ///
    /// **A verb that a caller runs between two steps ends here.** The step
    /// rebuilds at its barriers, and a verb outside a step reaches no
    /// barrier, so the verb itself is the only thing that can leave the
    /// world readable. A reader is not obliged to step first, and four
    /// reports of one refusal came from readers that did not.[^1] [^2]
    ///
    /// The call compares two revisions and returns when the verb changed no
    /// structure. It rebuilds when the verb changed one.
    ///
    /// The rebuild refuses only when the arena describes another world, and
    /// a verb of this world cannot produce that state. The assertion states
    /// that, so a build with the assertions on stops rather than carrying
    /// on.
    ///
    /// # References
    ///
    /// [^1]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decisions D3 and D4. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
    /// [^2]: Findings register, FND-647. `docs/FINDINGS.md`
    pub(super) fn leave_the_world_readable(&mut self) {
        let outcome = self.refresh_bridge();
        debug_assert!(
            outcome.is_ok(),
            "a verb of this world leaves a structure that describes it"
        );
    }
}
