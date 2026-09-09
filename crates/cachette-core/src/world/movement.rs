//! Where a unit is sent, and the pass that steps every unit one tile.
//!
//! A send names a destination. The step reads the destination field, picks a
//! neighbour tile and admits the move. The readers, the field derivation and
//! the pass sit together, because the pass is the only thing that reads what
//! the readers report.

use super::errors::{SendError, StepError};
use super::fire::douse_holds_unit;
use super::upgrades::{build_holds_unit, Building};
use super::World;
use crate::bridge::{BlockLayout, BridgeError};
use crate::choose::{Ranked, OPTIONS};
use crate::hex::{Axial, Grid, NEIGHBOUR_COUNT};
use crate::plan;
use crate::pyramid::{ApproachField, ExitField, ReturnField, SeededField, AT_SEED};
use crate::rng;
use crate::slots::Slots;
use crate::soldier::{SoldierArena, SoldierError};
use crate::stage::{self, Stage};
use crate::terrain::{Terrain, TileKind};
use crate::types::{Entity, Tick, TileIdx};

/// Whether a level 1 rebuild derives the destination field, or leaves it to
/// the step.
///
/// **The step derives the field after the controller.** The controller sends
/// several times in one frame, and each send changes the seed set of a plane,
/// so a field derived at the barrier is overwritten before anything reads
/// it.[^1]
///
/// Every other path leaves the field derived, because a caller outside a
/// frame reads it as soon as the call returns.[^2]
///
/// # References
///
/// [^1]: Findings register, FND-664. `docs/FINDINGS.md`
/// [^2]: Findings register, FND-029. `docs/FINDINGS.md`
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Destinations {
    /// Derive the field before the call returns.
    Derive,
    /// Leave the field to the one derivation the step makes at its end.
    AtTheEndOfTheStep,
}

/// Returns the neighbour a unit steps onto, or nothing when the ground there
/// refuses it.
///
/// **The gate is the capacity table, and this states no rule of its own.**
/// The `water_crossing` argument is the column of the type of the unit that
/// is stepping, and zero means cannot. The table takes it and answers the
/// capacity of the target, and passability is what it always was: a capacity
/// above zero.[^1] [^2]
///
/// # References
///
/// [^1]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D4. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
/// [^2]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decision D2. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
fn step_target(
    grid: Grid,
    terrain: Terrain,
    here: Axial,
    direction: usize,
    water_crossing: u32,
) -> Option<Axial> {
    let target = grid.neighbour(here, direction)?;
    if terrain.kind(target)?.is_passable_for(water_crossing) {
        Some(target)
    } else {
        None
    }
}

/// The draw index of the movement direction.
///
/// The movement system takes one draw for each soldier in each frame. A
/// second draw in the same system and frame must take the next index.
const DRAW_MOVE_DIRECTION: u32 = 0;

/// The draw index of the direction a unit takes when the ground refuses the
/// exit of its cell.
///
/// The fall-back is a second draw in the same system and the same frame, so
/// it takes the next index. A fall-back that reused the first index would
/// give the refused unit the direction that was just refused.[^1]
///
/// # References
///
/// [^1]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
const DRAW_MOVE_FALLBACK: u32 = 1;

/// Returns the move that each live soldier chose, in slot order.
///
/// Each soldier draws one direction from the counter-based generator. The
/// key is the tuple of the system, the frame, the entity and the draw
/// index, so the same soldier in the same frame gets the same direction
/// however the work was scheduled.[^1] The key holds the entity identity,
/// which pairs the slot index with the generation, and not the slot index
/// alone.[^2]
///
/// A soldier that holds no intent does not move at all. The choice pass
/// writes the intent, and it runs before this one.[^7]
///
/// A soldier whose chosen neighbour falls outside the world stays put. The
/// world is a rhombus and it does not wrap, so an address outside the
/// extent names no tile.[^3]
///
/// A soldier whose chosen neighbour holds ground that admits no unit also
/// stays put.[^6] This refusal belongs to the intent half. The ground refuses
/// every unit on every frame, whatever else stands there, so the intent never
/// reaches admission and the soldier takes no lateral step. A tile that is
/// full is a different refusal, and admission owns it.[^5]
///
/// The soldiers are read in slot order, each thread writes its own output
/// slot, and the step joins the slots in slot order. The result never
/// depends on thread completion order.[^4]
///
/// This is the intent half of movement only. A separate step admits the
/// intents against the capacity of each target, and it may refuse any of
/// them.[^5]
///
/// # References
///
/// [^1]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
/// [^2]: ADR-0014, entity identity is an index plus a generation, decision D1. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
/// [^3]: ADR-0017, the world is a rhombus, so a tile index is raw axial, decision D3. `docs/adrs/accepted/adr-0017-the-world-is-a-rhombus-so-a-tile-index-is-raw-axial.md`
/// [^4]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
/// [^5]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D2. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
/// [^6]: ADR-0068, terrain is generated from the seed and is never stored as a map, decision D4. `docs/adrs/accepted/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
/// [^7]: ADR-0064, a unit chooses by scoring a small fixed option set, decision D3. `docs/adrs/accepted/adr-0064-a-unit-chooses-by-scoring-a-small-fixed-option-set.md`
/// [^8]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, decision D1. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
/// [^9]: ADR-0017, the world is a rhombus, so a tile index is raw axial, decision D2. `docs/adrs/accepted/adr-0017-the-world-is-a-rhombus-so-a-tile-index-is-raw-axial.md`
/// [^10]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
/// The live units of one frame, and the arena that holds their columns.
///
/// The two travel together. The order is a property of the walk and the
/// columns are a property of the arena, and a caller that could pass one
/// without the other could pass an order taken from a different arena.
/// The per-cell fields that steer a step, and the lattice they are indexed by.
///
/// The three travel together. Every option takes its direction from one of the
/// two fields, and both are indexed by the level 1 cell that the lattice
/// names, so a caller that could pass one without the others could pass a
/// field taken from a different lattice.[^1] [^2]
///
/// # References
///
/// [^1]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, decision D1. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
/// [^2]: ADR-0110, a unit returns by climbing a reach field seeded at every site of its faction, decision D1. `docs/adrs/draft/adr-0110-a-unit-returns-by-climbing-a-reach-field.md`
pub(super) struct Steering<'a> {
    /// The block lattice that both fields below are indexed by.
    pub(super) layout: BlockLayout,
    /// One direction for each cell and each option that ranks a cell field.
    pub(super) exits: &'a ExitField,
    /// One direction for each cell and each faction.
    pub(super) returns: &'a ReturnField,
    /// One direction for each cell and each destination plane.
    ///
    /// The control plane names the seed set of a plane. A unit it sent to
    /// that plane reads one entry of it, in place of the entry that its own
    /// option would have read.[^3]
    ///
    /// [^3]: ADR-0125, the control plane names the seed set of a destination field, decision D1. `docs/adrs/draft/adr-0125-the-control-plane-names-the-seed-set-of-a-destination-field.md`
    pub(super) destinations: &'a SeededField,
    /// One direction for each destination plane and each tile of a seeded
    /// block.
    ///
    /// **The coarse field above steers a unit to the cell that holds its
    /// target and no further.** This one resolves that last cell at the pitch
    /// of one tile. A unit reads one entry of it, keyed on its own tile, and
    /// it reads no neighbouring tile, so it still searches nothing.[^4]
    ///
    /// [^4]: Findings register, FND-315. `docs/FINDINGS.md`
    pub(super) approaches: &'a ApproachField,
    /// One direction for each faction plane and each tile of a block that
    /// holds a site of that faction.
    ///
    /// **The return field steers a laden unit to the cell that holds a site
    /// and no further**, and a delivery reads the tile. This one resolves
    /// that last cell at the pitch of one tile. It is the same mechanism as
    /// the approach field above, keyed on the faction.[^4]
    ///
    /// [^4]: Findings register, FND-315. `docs/FINDINGS.md`
    pub(super) home_approaches: &'a ApproachField,
    /// One direction for each resource kind plane and each tile of a block
    /// that a gatherer stands in.
    ///
    /// **The exit field steers a unit to the cell that holds the most stock
    /// and no further**, and a gather resolve reads the tile. A unit that
    /// reached the cell stood on barren ground and took nothing, which is the
    /// defect the sent unit and the carrier both had.[^6] [^7]
    ///
    /// [^6]: Findings register, FND-315. `docs/FINDINGS.md`
    /// [^7]: Findings register, FND-589. `docs/FINDINGS.md`
    pub(super) stock_approaches: &'a ApproachField,
    /// The tile of every settlement slot.
    ///
    /// **A unit stops on the tile of its own home and on no other.** The
    /// field above is seeded at every site of the faction, in the way the
    /// return field is, so its seed offset says only that the unit stands on
    /// some site of its faction. A unit that stopped on a site that is not
    /// its home would hold its load for ever, because a delivery reads the
    /// home tile.[^5]
    ///
    /// [^5]: ADR-0062, production and upkeep are rates attached to a site, decision D2. `docs/adrs/accepted/adr-0062-production-and-upkeep-are-rates-attached-to-a-site.md`
    pub(super) site_tiles: &'a [TileIdx],
}

pub(super) struct UnitWalk<'a> {
    /// The arena that every identity below resolves against.
    pub(super) soldiers: &'a SoldierArena,
    /// Every live unit, in the order the pass walks them.
    pub(super) live: &'a [Entity],
}

/// What the destination plane of a sent unit says about the tile it stands on.
///
/// A plane answers one of three things, and the movement pass and the release
/// pass both read this answer. **The rule is one function, so no reader
/// declares a second time what arrival means.**[^1]
///
/// # References
///
/// [^1]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SendState {
    /// The plane names the neighbour that the unit steps onto.
    Steer(u8),
    /// The unit stands on a tile of the seed set. It has arrived.
    Arrived,
    /// The plane names no step and no arrival. It leads the unit nowhere.
    Lost,
}

/// Returns what the destination plane says to a unit on one tile.
///
/// **The fine field wins over the coarse one.** The coarse field answers for
/// a block of tiles, and the fine one answers for a tile of a block that
/// holds a seed. A unit outside every seeded block reads no fine entry and
/// takes the coarse answer, which is the answer it read before the fine field
/// existed.[^1]
///
/// The lost answer covers three cases with one rule: the plane holds no seed,
/// the seed it held is gone, and the unit stands where neither field reaches.
/// A unit that reads it is released rather than steered, because a plane that
/// leads nowhere would otherwise hold the unit for ever.[^2]
///
/// The function reads one entry of each field, keyed on the tile and the cell
/// of the unit. It reads no neighbour and it scores none, so a unit still
/// searches nothing.[^3]
///
/// # References
///
/// [^1]: Findings register, FND-315. `docs/FINDINGS.md`
/// [^2]: Findings register, FND-572. `docs/FINDINGS.md`
/// [^3]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, decision D1. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
fn send_state(
    destination: u16,
    tile: TileIdx,
    cell: u32,
    approaches: &ApproachField,
    destinations: &SeededField,
) -> SendState {
    match approaches.offset(destination, tile) {
        Some(AT_SEED) => SendState::Arrived,
        Some(direction) => SendState::Steer(direction),
        None => match destinations.direction(destination, cell) {
            Some(Some(direction)) => SendState::Steer(direction),
            _ => SendState::Lost,
        },
    }
}

pub(super) fn soldier_moves(
    tick: Tick,
    seed: u64,
    terrain: Terrain,
    walk: &UnitWalk<'_>,
    steering: &Steering<'_>,
    building: &Building<'_>,
    threads: usize,
) -> Result<Vec<(Entity, Axial)>, StepError> {
    let UnitWalk { soldiers, live } = *walk;
    let Steering {
        layout,
        exits,
        returns,
        destinations,
        approaches,
        home_approaches,
        stock_approaches,
        site_tiles,
    } = *steering;
    // **The walk is in cell order, not in slot order.** The two hold the same
    // units and differ only in the order. Every read below the filter is a
    // read of the tile side of the world at the tile the unit stands on: the
    // exit of its cell, the ground of its target, and the address of both.
    // The tile side is the larger of the two footprints, so the order that
    // makes it ascending is the order to walk.
    //
    // The arena cannot supply that order. A slot is half of an identity, so a
    // slot never moves, and an arena filled in tile order drifts away from it
    // as units die and slots return.[^13] The bridge already sorts every live
    // unit on the tile key once for each frame, at the barrier, so this order
    // costs the frame nothing more than it already paid.[^14]
    //
    // Nothing downstream reads this order. Admission sorts what it receives
    // on a total key of the target tile and the whole identity, so the same
    // set in another order gives the same result.[^15] The golden state hash
    // is the check that this claim holds, and it was checked by reversing the
    // walk rather than by asserting that reversing it would be safe.
    //
    // The caller passes the order in, and the caller is the one place that
    // takes it from the bridge. This function never chooses it.
    //
    // [^13]: ADR-0014, entity identity is an index plus a generation, decision D1. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
    // [^14]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D1. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
    // [^15]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D3. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
    if live.is_empty() {
        return Ok(Vec::new());
    }
    let grid = soldiers.grid();
    let chunk_len = live.len().div_ceil(threads).max(1);
    let mut slots: Slots<Vec<(Entity, Axial)>> =
        Slots::filled(threads, Vec::new()).map_err(|_| StepError::ZeroThreads)?;

    crate::parallel::fan_out_each(live.chunks(chunk_len).zip(slots.entries_mut()).map(
        move |(chunk, slot)| {
            move || {
                *slot = chunk
                    .iter()
                    .filter_map(|soldier| {
                        let here = soldiers.address(*soldier)?;
                        // **A unit that stands on the work it was ordered to
                        // do does not move.** The build advance adds the work
                        // of a builder to the tile the builder stands on, and
                        // one level of one upgrade asks for more work than one
                        // builder adds in one tick. A unit that walked away
                        // between two ticks therefore spread its labour over
                        // many tiles and finished none of them.[^22] [^23]
                        //
                        // **The hold is derived here and stored nowhere.** The
                        // question is asked again on every tick, from the
                        // order, the ground, the plan and the type. It stops
                        // being true on the tick the work finishes, on the
                        // tick the plan drops the project, and on the tick the
                        // ground changes hands. There is no hold to clear, so
                        // no unit can carry a stale one.[^22]
                        //
                        // **The clause sits above the send and above the
                        // intent**, because a unit the controller sent to a
                        // project still carries that send when it arrives. A
                        // send that outranked the hold would walk the builder
                        // off the tile it was sent to.[^22]
                        //
                        // [^22]: ADR-0168, a build order holds a unit on its tile, and the hold is derived, decisions D1, D2 and D3. `docs/adrs/draft/adr-0168-a-build-order-holds-a-unit-on-its-tile.md`
                        // [^23]: Findings register, FND-545. `docs/FINDINGS.md`
                        if build_holds_unit(
                            *soldier,
                            soldiers,
                            building,
                            plan::Ground {
                                grid,
                                terrain,
                                upgrades: building.upgrades,
                                table: building.table,
                            },
                        ) {
                            return None;
                        }
                        // **A unit that stands in the fire it was ordered to
                        // fight does not move.** The suppression the unit adds
                        // is charged against the tile it stands on, so a unit
                        // that walked away between two ticks would spread its
                        // effort over many tiles and put none of them out.
                        //
                        // The clause sits beside the build hold and above the
                        // send, for the reason the build hold does: a unit the
                        // control plane sent to the fire still carries that
                        // send when it arrives, and a send that outranked the
                        // hold would walk it straight back off the tile.
                        if douse_holds_unit(*soldier, soldiers, building.fire) {
                            return None;
                        }
                        // **A unit the control plane sent somewhere climbs
                        // the plane it was sent to, and it reads no intent.**
                        // An order from the control plane is not a score, so
                        // it does not join the option set and it does not
                        // compete with one. It replaces the field that steers
                        // the step, and it replaces nothing else.[^20]
                        //
                        // The test sits above the intent filter on purpose. A
                        // unit that has chosen nothing yet holds no intent,
                        // and a sent unit that waited for one would stand
                        // still until its cell next chose.[^21]
                        //
                        // [^20]: ADR-0125, the control plane names the seed set of a destination field, decision D2. `docs/adrs/draft/adr-0125-the-control-plane-names-the-seed-set-of-a-destination-field.md`
                        // [^21]: ADR-0064, a unit chooses by scoring a small fixed option set, decision D4. `docs/adrs/accepted/adr-0064-a-unit-chooses-by-scoring-a-small-fixed-option-set.md`
                        let sent = soldiers.sent(*soldier)?;
                        // A unit that holds no intent does not move. The
                        // choice pass writes the intent, and a unit whose
                        // every option scored below the floor holds what it
                        // was doing.[^7]
                        let option = match sent {
                            Some(_) => None,
                            None => Some(soldiers.intent(*soldier)??),
                        };
                        // **The option steers the step.** The unit reads the
                        // entry of its own cell and its own option, and it
                        // never scores a neighbouring cell of its own.[^8]
                        //
                        // The direction index of the cell lattice names the
                        // same offset as the direction index of the tile
                        // lattice, because both are the six neighbour offsets
                        // of a hex.[^9]
                        //
                        // A cell that no neighbour beats holds no direction,
                        // and the unit falls back to the uniform draw. The
                        // draw is keyed on the system, the frame, the entity
                        // and the draw index, so it never reads a
                        // thread-local state.[^10]
                        let cell = layout.block_of_key(layout.key_of(soldiers.tile(*soldier)?)?);
                        // **Which field steers the step comes from the option
                        // row.** A row that ranks a summary field of the cell
                        // is steered by the exit field. A row that ranks the
                        // state of the unit is steered by the return field,
                        // which holds one direction for each cell and each
                        // faction.[^18]
                        //
                        // The unit still reads one entry. It reads no
                        // neighbouring cell, it scores no neighbour, and it
                        // computes nothing from its own address toward its own
                        // site, so no unit searches anything.[^19]
                        //
                        // [^18]: ADR-0110, a unit returns by climbing a reach field seeded at every site of its faction, decision D1. `docs/adrs/draft/adr-0110-a-unit-returns-by-climbing-a-reach-field.md`
                        // [^19]: ADR-0095, a behavioural strategy arrives as a field over cells, never as a search from a unit, decision D1. `docs/adrs/draft/adr-0095-a-behavioural-strategy-arrives-as-a-field-over-cells.md`
                        // **A unit that stands on the tile it was sent to
                        // takes no step.** The coarse field holds one
                        // direction for a block of tiles, so it says nothing
                        // once the unit is inside the block that holds its
                        // target, and the unit fell back to the keyed draw
                        // and walked away from the tile it had reached. The
                        // approach field answers at tile pitch, and the seed
                        // offset is how it says the unit has arrived.[^25]
                        //
                        // The clause sits above the steering, because a unit
                        // that arrived has nowhere further to be steered.
                        //
                        // **A sent unit never draws a direction.** It steps
                        // where its plane says, or it stands still and the
                        // step releases it once it has applied the moves of
                        // this frame. The fall-back to the keyed draw made a
                        // sent unit wander, and that wandering was the only
                        // thing that ever carried a load home.[^26]
                        //
                        // [^25]: Findings register, FND-315. `docs/FINDINGS.md`
                        // [^26]: Findings register, FND-572. `docs/FINDINGS.md`
                        let send = match sent {
                            Some(destination) => Some(send_state(
                                destination,
                                soldiers.tile(*soldier)?,
                                cell,
                                approaches,
                                destinations,
                            )),
                            None => None,
                        };
                        let steered = match send {
                            Some(SendState::Steer(direction)) => Some(direction),
                            // A unit that arrived, and a unit whose plane
                            // leads nowhere, both take no step.
                            Some(_) => return None,
                            None => None,
                        };
                        // **The homeward leg reads the same mechanism, keyed
                        // on the faction.** The return field holds one
                        // direction for a block of tiles, so it says nothing
                        // once a laden unit is inside the block that holds a
                        // site, and the unit fell back to the keyed draw. A
                        // delivery reads the tile the unit stands on, so a
                        // carrier that reached the cell delivered nothing,
                        // which is the defect the sent unit had.[^25]
                        let homeward = match (sent, option) {
                            (None, Some(option))
                                if matches!(OPTIONS[option as usize].ranked, Ranked::Carry) =>
                            {
                                home_approaches
                                    .offset(soldiers.faction(*soldier)?.0, soldiers.tile(*soldier)?)
                            }
                            _ => None,
                        };
                        // **The unit stops on the tile of its own home and on
                        // no other.** The field is seeded at every site of
                        // the faction, so the seed offset says only that the
                        // unit stands on some site of its faction. A unit
                        // that stopped on a site that is not its home would
                        // hold its load for ever. Such a unit keeps the
                        // answer it had before this field existed.
                        if homeward == Some(AT_SEED) {
                            let home = soldiers.home(*soldier)?;
                            let at_home = home.and_then(|slot| site_tiles.get(slot as usize))
                                == Some(&soldiers.tile(*soldier)?);
                            if at_home {
                                return None;
                            }
                        }
                        // **The gathering leg reads the same mechanism, keyed
                        // on the resource kind the unit was ordered to
                        // gather.** The exit field holds one direction for a
                        // block of tiles, and the stock of a tile is a level 0
                        // property. A unit inside the cell with the most food
                        // read that its own cell was the best one, stripped
                        // the ground under it, and stood there while the world
                        // still held food two tiles away.[^27]
                        //
                        // The plane is the order column, because the gather
                        // resolve reads that column. A field keyed on the
                        // option row instead would walk a unit to food and let
                        // it take stone.[^28]
                        //
                        // A unit that a caller sent somewhere reads its
                        // destination plane instead, and a laden unit reads
                        // the home field above, so this answers only for a
                        // unit that is steering itself by the ground.
                        //
                        // [^27]: Findings register, FND-589. `docs/FINDINGS.md`
                        // [^28]: Findings register, FND-590. `docs/FINDINGS.md`
                        let toward_stock = match (sent, option) {
                            (None, Some(option))
                                if matches!(OPTIONS[option as usize].ranked, Ranked::Cell(_)) =>
                            {
                                soldiers.gather_order(*soldier).flatten().and_then(|kind| {
                                    stock_approaches
                                        .offset(u16::from(kind.to_u8()), soldiers.tile(*soldier)?)
                                })
                            }
                            _ => None,
                        };
                        // **A unit that stands on stock of the kind it wants
                        // takes no step.** The seed offset is how the field
                        // says so. A unit that stepped away would strip one
                        // tile of one unit each frame and walk on.
                        if toward_stock == Some(AT_SEED) {
                            return None;
                        }
                        let steer = match (steered, option) {
                            // **The destination plane wins over the option
                            // row.** A caller that sends a unit somewhere has
                            // said where it goes, and the option the unit
                            // scored for itself says only what it wants.[^20]
                            // Which field answered is decided above, and the
                            // fine field wins over the coarse one there.[^25]
                            (Some(direction), _) => Some(Some(direction)),
                            (None, Some(option)) => match OPTIONS[option as usize].ranked {
                                // **The fine field wins over the coarse
                                // one**, in the way it does for a sent unit
                                // and for a laden one. A unit whose block
                                // holds no stock of its kind reads no fine
                                // entry and takes the coarse answer, which is
                                // the answer it read before this field
                                // existed. The seed offset never reaches
                                // here, because a unit that stands on stock
                                // already left the walk.[^27]
                                Ranked::Cell(_) => match toward_stock {
                                    Some(direction) => Some(Some(direction)),
                                    None => exits.exit(cell, option),
                                },
                                // **The fine field wins over the coarse
                                // one**, in the way it does for a sent unit.
                                // A unit outside every seeded block reads no
                                // fine entry and takes the coarse answer.
                                // The seed offset falls through to the coarse
                                // answer too, because the unit that reached
                                // its own home already left the walk.[^25]
                                Ranked::Carry => match homeward {
                                    Some(direction) if direction != AT_SEED => {
                                        Some(Some(direction))
                                    }
                                    _ => returns.direction(soldiers.faction(*soldier)?, cell),
                                },
                            },
                            // A unit that holds no intent and no destination
                            // left the walk at the filter above.
                            (None, None) => None,
                        };
                        let direction = match steer {
                            Some(Some(direction)) => direction as usize,
                            _ => rng::draw_below(
                                seed,
                                rng::SYSTEM_SOLDIER_MOVE,
                                tick.0,
                                soldier.to_bits(),
                                DRAW_MOVE_DIRECTION,
                                NEIGHBOUR_COUNT as u64,
                            ) as usize,
                        };
                        // **A refused direction falls back to a draw, and it
                        // never freezes the unit.** The exit of a cell is one
                        // direction for a block of tiles, and the ground under
                        // one unit of that block may refuse it. That refusal
                        // repeats every frame, because the cell, the option
                        // and the direction all hold, so a unit that only
                        // stayed put would stay put for ever. A unit against
                        // a shoreline is the case that showed it.[^16]
                        //
                        // The fall-back is the same keyed draw that a cell
                        // with no exit already takes, at the next draw index.
                        // It is keyed on the frame, so a unit the draw refuses
                        // again draws a different direction on the next
                        // frame.[^10]
                        //
                        // The fall-back reads the six neighbours of no cell.
                        // It reads one tile, which is the tile the unit would
                        // step onto, so the rule that a unit never searches
                        // its neighbourhood holds.[^17]
                        //
                        // [^16]: Findings register, FND-315. `docs/FINDINGS.md`
                        // [^17]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, decision D1. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
                        // **The crossing of the unit is a column of its own
                        // type row.** The row is data and the movement pass
                        // reads it, so a type that crosses water and a type
                        // that does not take the same code path and differ
                        // only in the number they hand the capacity
                        // table.[^24]
                        //
                        // A unit whose type the arena cannot answer for
                        // crosses nothing, which is the answer every type
                        // gave before the column existed.
                        //
                        // [^24]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decision D2. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
                        let water_crossing = soldiers.unit_type(*soldier).map_or(0, |unit_type| {
                            building.unit_types.row(unit_type).water_crossing
                        });
                        let target = step_target(grid, terrain, here, direction, water_crossing);
                        let target = match target {
                            Some(target) => target,
                            None => {
                                let again = rng::draw_below(
                                    seed,
                                    rng::SYSTEM_SOLDIER_MOVE,
                                    tick.0,
                                    soldier.to_bits(),
                                    DRAW_MOVE_FALLBACK,
                                    NEIGHBOUR_COUNT as u64,
                                ) as usize;
                                step_target(grid, terrain, here, again, water_crossing)?
                            }
                        };
                        Some((*soldier, target))
                    })
                    .collect();
            }
        },
    ));

    Ok(slots.combine(Vec::new(), |mut joined, slot| {
        joined.extend_from_slice(slot);
        joined
    }))
}

impl World {
    /// Sends a set of units to a set of tiles, through one destination plane.
    ///
    /// **The control plane names the seed set, and the engine builds one
    /// field.** The seeds are the tiles the caller wants the units at. The
    /// engine takes the level 1 cell of each of them, seeds the plane at every
    /// one, and relaxes a reach outward from the whole set at once. Every unit
    /// the call names then reads one entry of that plane and steps.[^1] [^2]
    ///
    /// **No unit gains a search.** A unit reads the entry of its own cell and
    /// its own plane. It reads no neighbouring cell, it scores no neighbour,
    /// and it computes nothing from its own address toward a destination.[^3]
    ///
    /// **The set is all or nothing.** Every identity resolves and every
    /// address is inside the world before anything changes.
    ///
    /// The order of the seeds does not reach the field. The engine sorts the
    /// cells and holds each of them once, so two calls that name one set in
    /// two orders derive one field.[^4]
    ///
    /// A caller that names the same plane again replaces the seed set of that
    /// plane. Every unit already sent to it then climbs the new one.
    ///
    /// # Errors
    ///
    /// Returns an error when the number names no destination plane, when an
    /// identity names no live soldier, or when an address is outside the
    /// world.
    ///
    /// # References
    ///
    /// [^1]: ADR-0125, the control plane names the seed set of a destination field, decision D1. `docs/adrs/draft/adr-0125-the-control-plane-names-the-seed-set-of-a-destination-field.md`
    /// [^2]: ADR-0095, a behavioural strategy arrives as a field over cells, never as a search from a unit, decision D3. `docs/adrs/draft/adr-0095-a-behavioural-strategy-arrives-as-a-field-over-cells.md`
    /// [^3]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, decision D1. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
    /// [^4]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    pub fn send_units_to(
        &mut self,
        units: &[Entity],
        seeds: &[Axial],
        destination: u16,
    ) -> Result<(), SendError> {
        if destination >= self.destinations.plane_count() {
            return Err(SendError::NoSuchDestination(destination));
        }
        for unit in units {
            if self.soldiers.slot_of(*unit).is_none() {
                return Err(SendError::DeadUnit(*unit));
            }
        }
        let mut tiles = Vec::with_capacity(seeds.len());
        for address in seeds {
            let tile = self
                .grid
                .index_of(*address)
                .ok_or(SendError::AddressOutsideWorld(*address))?;
            // The cell is derived from the tile at every read, and it is
            // stored nowhere, so the two cannot disagree.
            self.cell_of(tile)
                .ok_or(SendError::AddressOutsideWorld(*address))?;
            tiles.push(tile);
        }
        // The set is a set. The sort and the dedup make the stored order a
        // property of the tiles and never of the order the caller named them
        // in.[^4]
        tiles.sort_unstable_by_key(|tile: &TileIdx| tile.0);
        tiles.dedup();
        self.destination_seeds[destination as usize] = tiles;
        // **The plane conducts through water when every unit sent to it
        // crosses water.** The caller states no flag. It states a set, and
        // the crossing of that set follows from the type of each unit in it,
        // which is a column of the shared table.[^6]
        //
        // A mixed set does not cross. The field is one direction for each
        // cell, so a plane that crossed for the whole set would steer the
        // units that the water refuses at a coast, which is the failure the
        // land rule was written against.[^7]
        //
        // An empty set does not cross. A send of no unit steers nobody, and
        // the safe answer costs nothing.
        //
        // [^6]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decision D2. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
        // [^7]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, decision D5. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
        let crosses = !units.is_empty()
            && units.iter().all(|unit| {
                self.soldiers
                    .unit_type(*unit)
                    .is_some_and(|unit_type| self.unit_types.row(unit_type).water_crossing > 0)
            });
        self.destination_crossings[destination as usize] = u8::from(crosses);
        for unit in units {
            assert!(
                self.soldiers.set_sent(*unit, Some(destination)),
                "a resolved identity must name a soldier the arena can send"
            );
        }
        // The verb leaves the field derived. A caller reads the direction
        // between two steps, and a derived value that one path leaves stale
        // is a confident wrong answer.[^5]
        //
        // **The step derives the field once, after the controller.** It sets
        // the flag while the controller runs, and it derives the field before
        // it returns, so the reason above holds for every caller and the
        // frame pays for one derivation rather than one for each send.[^8]
        //
        // [^5]: Findings register, FND-029. `docs/FINDINGS.md`
        // [^8]: Findings register, FND-664. `docs/FINDINGS.md`
        if !self.destinations_deferred {
            let _span = stage::open(Stage::SendDeriveDestinations);
            self.derive_destination_fields();
        }
        Ok(())
    }

    /// Stops sending a set of units.
    ///
    /// Every unit the call names goes back to the option it chose for itself.
    /// **The set is all or nothing**: every identity resolves before anything
    /// changes.
    ///
    /// # Errors
    ///
    /// Returns an error when an identity names no live soldier.
    pub fn stop_sending(&mut self, units: &[Entity]) -> Result<(), SendError> {
        for unit in units {
            if self.soldiers.slot_of(*unit).is_none() {
                return Err(SendError::DeadUnit(*unit));
            }
        }
        for unit in units {
            assert!(
                self.soldiers.set_sent(*unit, None),
                "a resolved identity must name a soldier the arena can stop"
            );
        }
        Ok(())
    }

    /// Returns the destination that the control plane sent one unit to.
    ///
    /// The outer option reports whether the identity names a live soldier.
    /// The inner one reports whether anybody sent it anywhere.
    #[must_use]
    pub fn sent_to(&self, entity: Entity) -> Option<Option<u16>> {
        self.soldiers.sent(entity)
    }

    /// Returns the number of destination planes that the world holds.
    #[must_use]
    pub const fn destination_count(&self) -> u16 {
        self.destinations.plane_count()
    }

    /// Sets the number of destination planes that the world holds.
    ///
    /// **The caller names the plane that carries an order, and the engine
    /// allocates none.** The count says how many places a control plane may
    /// send units to at one time, before it re-aims a plane it already
    /// used.[^1]
    ///
    /// The call clears the seed set of every plane, so no order steers
    /// anything until the caller sends a set again. A unit that was sent to a
    /// plane the world no longer holds reads no direction, and the next step
    /// releases it rather than leaving it sent for ever.[^2]
    ///
    /// The cost is the cell count times the number of planes, in bytes. No
    /// figure appears here, because one blocker governs every cost figure this
    /// project holds.[^3]
    ///
    /// # References
    ///
    /// [^1]: ADR-0125, the control plane names the seed set of a destination field, decision D3. `docs/adrs/draft/adr-0125-the-control-plane-names-the-seed-set-of-a-destination-field.md`
    /// [^2]: ADR-0125, the control plane names the seed set of a destination field, decision D4. `docs/adrs/draft/adr-0125-the-control-plane-names-the-seed-set-of-a-destination-field.md`
    /// [^3]: Blockers register, BLK-007. `docs/BLOCKERS.md`
    pub fn set_destination_count(&mut self, count: u16) {
        self.destinations = SeededField::new(self.destinations.cells(), count);
        self.destination_seeds = vec![Vec::new(); count as usize];
        self.destination_crossings = vec![0; count as usize];
        self.derive_destination_fields();
    }

    /// Returns the direction that a unit sent to one destination takes from
    /// one address.
    ///
    /// The outer option reports whether the address and the destination name
    /// an entry. The inner one reports whether the cell holds a direction at
    /// all. **A cell that holds a seed, and a cell the reach never arrived
    /// at, both hold none.** The fine field answers the first case at the
    /// pitch of one tile, and the step releases a sent unit that neither
    /// field steers.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0125, the control plane names the seed set of a destination field, decision D4. `docs/adrs/draft/adr-0125-the-control-plane-names-the-seed-set-of-a-destination-field.md`
    #[must_use]
    pub fn destination_direction(&self, destination: u16, address: Axial) -> Option<Option<u8>> {
        let tile = self.grid.index_of(address)?;
        self.destinations
            .direction(destination, self.cell_of(tile)?)
    }

    /// Returns the destination field of the world.
    ///
    /// The field holds one direction for each level 1 cell and each
    /// destination plane.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0125, the control plane names the seed set of a destination field, decision D1. `docs/adrs/draft/adr-0125-the-control-plane-names-the-seed-set-of-a-destination-field.md`
    #[must_use]
    pub const fn destination_field(&self) -> &SeededField {
        &self.destinations
    }

    /// Returns one seed for each destination plane and each cell of its set.
    ///
    /// The walk is over the planes in ascending order and over the cells of
    /// each plane in ascending order, so the set does not depend on a thread
    /// count.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    fn destination_seed_pairs(&self) -> Vec<(u16, u32)> {
        let mut seeds = Vec::new();
        for (plane, tiles) in self.destination_seeds.iter().enumerate() {
            for tile in tiles {
                let Some(cell) = self.cell_of(*tile) else {
                    continue;
                };
                seeds.push((plane as u16, cell));
            }
        }
        seeds
    }

    /// Returns one seed for each destination plane and each tile of its set.
    ///
    /// The walk is over the planes in ascending order and over the tiles of
    /// each plane in ascending order, so the set does not depend on a thread
    /// count.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    fn destination_seed_tiles(&self) -> Vec<(u16, TileIdx)> {
        let mut seeds = Vec::new();
        for (plane, tiles) in self.destination_seeds.iter().enumerate() {
            for tile in tiles {
                seeds.push((plane as u16, *tile));
            }
        }
        seeds
    }

    /// Derives the coarse and the fine field of every destination plane.
    ///
    /// **This is the one place that derives either of them.** Both come from
    /// one seed set, and a path that wrote one without the other would leave
    /// a stale value that nothing fails on.[^1] [^2]
    ///
    /// # References
    ///
    /// [^1]: Findings register, FND-029. `docs/FINDINGS.md`
    /// [^2]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
    pub(super) fn derive_destination_fields(&mut self) {
        self.destinations.derive(
            &self.pyramid,
            &self.destination_seed_pairs(),
            &self.destination_crossings,
        );
        self.approaches.derive(
            self.terrain,
            &self.destination_seed_tiles(),
            &self.destination_crossings,
        );
    }

    /// Reports whether the ground at an address admits a unit.
    ///
    /// The answer is a property of the ground alone.[^1] It does not depend
    /// on the tick, on the faction, or on what already stands there. An
    /// address outside the world gives `false`, and the caller reports that
    /// refusal under its own name.
    ///
    /// # References
    ///
    /// [^1]: ADR-0068, terrain is generated from the seed and is never stored as a map, decision D4. `docs/adrs/accepted/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
    #[must_use]
    pub fn admits_a_unit(&self, address: Axial) -> bool {
        self.terrain
            .kind(address)
            .is_some_and(TileKind::is_passable)
    }

    /// Returns the water crossing column of the type of one unit.
    ///
    /// **This is the one place that reads the column for a unit.** A unit
    /// whose type the arena cannot answer for crosses nothing, which is the
    /// answer every type gave before the column existed.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decision D2. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
    fn crossing_of(&self, unit: Entity) -> u32 {
        self.soldiers
            .unit_type(unit)
            .map_or(0, |unit_type| self.unit_types.row(unit_type).water_crossing)
    }

    /// Reports whether the ground at an address admits one named unit.
    ///
    /// The ground states a capacity for the crossing the type of the unit
    /// holds, and a capacity above zero admits it. A type with no crossing
    /// therefore gets the answer the ground alone gives, and a type that
    /// crosses open water may stand on it.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decision D2. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
    pub(super) fn admits_this_unit(&self, unit: Entity, address: Axial) -> bool {
        self.terrain
            .kind(address)
            .is_some_and(|kind| kind.is_passable_for(self.crossing_of(unit)))
    }

    /// Refuses an address that lies inside the world on ground that admits
    /// no unit.
    ///
    /// The extent refusal stays with the arena, which owns the grid. This
    /// call therefore says nothing about an address outside the world.
    pub(super) fn refuse_impassable(&self, address: Axial) -> Result<(), SoldierError> {
        match self.terrain.kind(address) {
            Some(kind) if !kind.is_passable() => Err(SoldierError::TileImpassable(address)),
            _ => Ok(()),
        }
    }

    /// Frees every sent unit that its destination plane no longer steers.
    ///
    /// **A verb that sends a unit must state what releases it.** A sent unit
    /// climbs its destination plane and reads no option row, so a unit that
    /// arrives and stays sent neither gathers nor delivers. Nothing released
    /// such a unit, and the defect was invisible for as long as arrival was
    /// impossible.[^1]
    ///
    /// The pass frees two kinds of unit, and one rule names both. A unit that
    /// stands on a tile of the seed set has arrived. A unit that reads no
    /// direction from the fine field and none from the coarse one is somewhere
    /// its plane leads nowhere: the plane holds no seed, the seed it held is
    /// gone, or the ground between the two refuses the unit. Neither kind may
    /// stay sent, because neither takes another step.
    ///
    /// **The release is the destination the caller set, and nothing else.**
    /// The pass clears the home site of no unit and the intent of no unit.
    ///
    /// The walk is over the arena in slot order, and it writes the send column
    /// of one unit for each entry. No entry reads what another wrote, so the
    /// order decides nothing and the result is the same at any thread
    /// count.[^2]
    ///
    /// # References
    ///
    /// [^1]: Findings register, FND-572. `docs/FINDINGS.md`
    /// [^2]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    pub(super) fn release_sent_units(&mut self) {
        let layout = self.pyramid.layout();
        let mut released: Vec<Entity> = Vec::new();
        for unit in self.soldiers.iter() {
            let Some(Some(destination)) = self.soldiers.sent(unit) else {
                continue;
            };
            let Some(tile) = self.soldiers.tile(unit) else {
                continue;
            };
            let Some(key) = layout.key_of(tile) else {
                continue;
            };
            let state = send_state(
                destination,
                tile,
                layout.block_of_key(key),
                &self.approaches,
                &self.destinations,
            );
            if matches!(state, SendState::Steer(_)) {
                continue;
            }
            released.push(unit);
        }
        for unit in released {
            self.soldiers.set_sent(unit, None);
        }
    }

    /// Rebuilds the derived unit structure when the arena has moved past it.
    ///
    /// **This is the one place that makes the world readable again.** The
    /// step calls it at each of its barriers, and a verb that changes the
    /// arena outside a step calls it before it returns. A second body that
    /// compared the revisions for itself would be one rule in two places.[^1]
    ///
    /// The call returns a bridge error and not a step error, because a verb
    /// outside a step calls it too and a verb makes no step.
    ///
    /// # References
    ///
    /// [^1]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
    pub(super) fn refresh_bridge(&mut self) -> Result<(), BridgeError> {
        if self.bridge.describes(&self.soldiers).is_ok() {
            return Ok(());
        }
        self.bridge.rebuild(&self.soldiers)?;
        Ok(())
    }
}
