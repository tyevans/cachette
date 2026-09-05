//! A contract carries land or a relation, and a faction posts a board.
//!
//! Every test here goes through the public interface. It builds a world,
//! arranges the holding through the rule that holds ground, sends the verbs a
//! player would send, steps, and reads what the engine answers.[^1]
//!
//! **The fixture is built for the case.** A land transfer is checked against a
//! control world that ran the same step without the contract, so the tiles
//! that the holding rule moved on its own do not count as the contract's
//! work. The cell fixture is a level 1 cell on the world edge, which is
//! partial, and the list fixture is a list at the bound.[^2]
//!
//! # References
//!
//! [^1]: Testing Rules, section 5. `.claude/rules/testing.md`
//! [^2]: Testing Rules, section 2a. `.claude/rules/testing.md`

use cachette_core::holding::Holder;
use cachette_core::terrain::TileKind;
use cachette_core::upgrade::UpgradeKind;
use cachette_core::{
    Advert, Axial, Consideration, FactionId, Tick, TileIdx, TradeError, World, WorldConfig,
    ACT_SETTLE, ACT_STEP_RELATION, ACT_TRANSFER_LAND, ADVERT_OFFERS, ADVERT_WANTS, TRADE_BOUND,
    TRADE_DEFAULTED, TRADE_SETTLED,
};
use proptest::prelude::*;

const THREAD_COUNTS: [usize; 3] = [1, 2, 12];
const FOOD: u8 = 0;
const WOOD: u8 = 1;
const ZERO: FactionId = FactionId(0);
const ONE: FactionId = FactionId(1);

/// Builds a world in which both factions hold ground.
fn a_world(seed: u64) -> World {
    let config = WorldConfig {
        width: 48,
        height: 48,
        seed,
        faction_count: 2,
        ..WorldConfig::default()
    };
    let mut world = World::new(config).expect("the extent must describe a world");
    let _ = world.found_run_for_every_faction(24);
    for _ in 0..6 {
        world.step(1).expect("the step must run");
    }
    world
}

/// Returns every tile one faction holds, in ascending tile index.
fn tiles_held_by(world: &World, faction: FactionId) -> Vec<TileIdx> {
    let grid = world.grid();
    let mut found = Vec::new();
    for r in 0..grid.height() {
        for q in 0..grid.width() {
            let address = Axial::new(q as i32, r as i32);
            if world.tile_holder(address) == Some(Holder::of(faction)) {
                found.push(grid.index_of(address).expect("the address is inside"));
            }
        }
    }
    found
}

fn address_of(world: &World, tile: TileIdx) -> Axial {
    world.grid().address_of(tile).expect("the tile is inside")
}

/// Puts a unit of the speaker on ground the listener holds.
///
/// The spawn happens between two steps and the caller speaks before the next
/// one, because the holding rule takes the tile for the speaker on the next
/// step.
fn give_presence(world: &mut World, speaker: FactionId, listener: FactionId) {
    let tile = *tiles_held_by(world, listener)
        .first()
        .expect("the listener holds ground");
    world
        .spawn_soldier(address_of(world, tile), speaker)
        .expect("the spawn must succeed");
    assert!(world.stands_in_territory_of(speaker, listener));
}

/// Returns the holder of every tile.
fn holders(world: &World) -> Vec<Option<Holder>> {
    let grid = world.grid();
    let mut column = Vec::with_capacity(grid.tile_count() as usize);
    for r in 0..grid.height() {
        for q in 0..grid.width() {
            column.push(world.tile_holder(Axial::new(q as i32, r as i32)));
        }
    }
    column
}

/// Binds a contract in which faction zero gives the tiles for a relation
/// step from faction one.
fn bind_land_for_a_relation(world: &mut World, tiles: Vec<TileIdx>) {
    world
        .offer_consideration(
            ZERO,
            ONE,
            Consideration::land(tiles),
            Consideration::relation(0, 1),
            50,
        )
        .expect("the offer must be accepted");
    world
        .accept_trade(ONE, ZERO)
        .expect("the acceptance must succeed");
}

/// Runs the land contract in a trial world and the same step in a control
/// world, and asserts that exactly the listed tiles differ.
fn assert_exactly_the_listed_tiles_move(mut world: World, tiles: Vec<TileIdx>, threads: usize) {
    give_presence(&mut world, ZERO, ONE);
    give_presence(&mut world, ONE, ZERO);
    let mut control = world.clone();
    bind_land_for_a_relation(&mut world, tiles.clone());

    world.step(threads).expect("the step must run");
    control.step(threads).expect("the step must run");

    let after = holders(&world);
    let expected = holders(&control);
    for (index, (got, want)) in after.iter().zip(expected.iter()).enumerate() {
        let tile = TileIdx(index as u32);
        if tiles.contains(&tile) {
            assert_eq!(
                *got,
                Some(Holder::of(ONE)),
                "tile {index} was in the land set and did not move to the creditor"
            );
        } else {
            assert_eq!(
                *got, *want,
                "tile {index} was not in the land set and moved"
            );
        }
    }
    let row = world.trade_row(ZERO, ONE).expect("the pair exists");
    assert_eq!(row.status, TRADE_SETTLED);
    assert_eq!(row.given, row.give_amount);
    let acts: Vec<u8> = world.trade_log().iter().map(|event| event.act).collect();
    assert!(
        acts.contains(&ACT_TRANSFER_LAND),
        "no land transfer was logged"
    );
    assert!(
        acts.contains(&ACT_STEP_RELATION),
        "no relation step was logged"
    );
    assert!(acts.contains(&ACT_SETTLE), "no settlement was logged");
    assert!(world.check_invariants(), "the world lost an invariant");
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 12,
        ..ProptestConfig::default()
    })]

    /// A land set moves exactly the listed tiles and no other tile.
    ///
    /// The list is a random subset of what the debtor holds, up to the bound,
    /// and it is compared against a control world that stepped without the
    /// contract.
    #[test]
    fn a_land_set_moves_exactly_the_listed_tiles(
        picks in prop::collection::vec(any::<prop::sample::Index>(), 1..=64usize),
    ) {
        let world = a_world(7);
        let held = tiles_held_by(&world, ZERO);
        prop_assume!(!held.is_empty());
        let mut tiles: Vec<TileIdx> = picks.iter().map(|pick| *pick.get(&held)).collect();
        tiles.sort_unstable();
        tiles.dedup();
        assert_exactly_the_listed_tiles_move(world, tiles, 1);
    }
}

/// Returns a small world with two blocks, whose second block is a partial
/// cell on the world edge, and in which faction zero holds that whole cell.
///
/// The world is 40 tiles wide and 8 high, so the level 1 cell at the right
/// edge covers 8 columns and 8 rows. Water cannot be held, so the fixture
/// searches the seed for a cell with no water tile.
fn a_world_with_an_edge_cell() -> (World, Vec<TileIdx>) {
    for seed in 1..400u64 {
        let config = WorldConfig {
            width: 40,
            height: 8,
            seed,
            faction_count: 2,
            ..WorldConfig::default()
        };
        let mut world = World::new(config).expect("the extent must describe a world");
        let cell = world
            .cell_tiles(Axial::new(39, 7))
            .expect("the corner is inside");
        assert_eq!(cell.len(), 64, "the edge cell is 8 by 8");
        let all_land = cell
            .iter()
            .all(|tile| world.tile_kind(address_of(&world, *tile)) != Some(TileKind::Water));
        let left = world
            .grid()
            .index_of(Axial::new(0, 0))
            .expect("the origin is inside");
        let left_is_land = world.tile_kind(Axial::new(0, 0)) != Some(TileKind::Water);
        if !all_land || !left_is_land {
            continue;
        }
        // Four units meet the claim threshold of every kind of ground.
        for tile in &cell {
            for _ in 0..4 {
                world
                    .spawn_soldier(address_of(&world, *tile), ZERO)
                    .expect("the spawn must succeed");
            }
        }
        for _ in 0..4 {
            world
                .spawn_soldier(address_of(&world, left), ONE)
                .expect("the spawn must succeed");
        }
        world.step(1).expect("the step must run");
        let held = cell
            .iter()
            .all(|tile| world.tile_holder(address_of(&world, *tile)) == Some(Holder::of(ZERO)));
        let left_held = world.tile_holder(Axial::new(0, 0)) == Some(Holder::of(ONE));
        if held && left_held {
            return (world, cell);
        }
    }
    panic!("no seed below 400 gives an edge cell with no water");
}

#[test]
fn a_cell_on_the_world_edge_is_partial_and_ascending() {
    let (world, cell) = a_world_with_an_edge_cell();
    assert_eq!(cell.len(), 64);
    assert!(
        cell.windows(2).all(|pair| pair[0] < pair[1]),
        "the cell is not ascending"
    );
    for tile in &cell {
        let address = address_of(&world, *tile);
        assert!(
            address.q >= 32 && address.q < 40,
            "tile {tile:?} is outside the cell"
        );
    }
    // The same cell from another of its addresses.
    let again = world.cell_tiles(Axial::new(32, 0)).expect("inside");
    assert_eq!(again, cell);
    assert_eq!(
        world.cell_tiles(Axial::new(40, 0)),
        Err(TradeError::NoSuchCell)
    );
}

#[test]
fn a_whole_edge_cell_changes_holder_on_delivery() {
    let (world, cell) = a_world_with_an_edge_cell();
    assert_exactly_the_listed_tiles_move(world, cell, 1);
}

#[test]
fn a_list_at_the_bound_is_accepted_and_one_above_it_is_refused() {
    let (mut world, cell) = a_world_with_an_edge_cell();
    give_presence(&mut world, ZERO, ONE);
    give_presence(&mut world, ONE, ZERO);
    world.set_land_list_bound(63);
    let refused = world.offer_consideration(
        ZERO,
        ONE,
        Consideration::land(cell.clone()),
        Consideration::relation(0, 1),
        50,
    );
    assert_eq!(refused, Err(TradeError::TooMuchLand(64, 63)));
    assert!(world.trade_book().is_empty(), "a refused offer wrote a row");
    world.set_land_list_bound(64);
    bind_land_for_a_relation(&mut world, cell.clone());
    assert_eq!(world.trade_land(ZERO, ONE, false), cell.as_slice());
}

#[test]
fn a_land_side_the_debtor_does_not_hold_is_refused() {
    let mut world = a_world(7);
    give_presence(&mut world, ZERO, ONE);
    let mine = tiles_held_by(&world, ZERO);
    let theirs = *tiles_held_by(&world, ONE)
        .first()
        .expect("faction one holds ground");
    let mut tiles = vec![mine[0]];
    tiles.push(theirs);
    let refused = world.offer_consideration(
        ZERO,
        ONE,
        Consideration::land(tiles),
        Consideration::relation(0, 1),
        50,
    );
    assert_eq!(refused, Err(TradeError::LandNotHeld(theirs)));
    // The take side is checked against the responder, not the proposer.
    let refused = world.offer_consideration(
        ZERO,
        ONE,
        Consideration::relation(0, 1),
        Consideration::land(vec![mine[0]]),
        50,
    );
    assert_eq!(refused, Err(TradeError::LandNotHeld(mine[0])));
    let outside = TileIdx(world.grid().tile_count());
    let refused = world.offer_consideration(
        ZERO,
        ONE,
        Consideration::land(vec![outside]),
        Consideration::relation(0, 1),
        50,
    );
    assert_eq!(refused, Err(TradeError::NoSuchTile));
    let refused = world.offer_consideration(
        ZERO,
        ONE,
        Consideration {
            tag: 7,
            kind: 0,
            amount: 1,
            tiles: Vec::new(),
        },
        Consideration::relation(0, 1),
        50,
    );
    assert_eq!(refused, Err(TradeError::NoSuchTag(7)));
}

#[test]
fn a_land_side_whose_tile_carries_an_upgrade_is_refused() {
    let mut world = a_world(7);
    let mine = tiles_held_by(&world, ZERO);
    let site = mine[mine.len() / 2];
    let builder = world
        .spawn_soldier(address_of(&world, site), ZERO)
        .expect("the spawn must succeed");
    assert!(world.order_build(builder, UpgradeKind::Road));
    for _ in 0..12 {
        if world.upgrade_at(address_of(&world, site)).is_some() {
            break;
        }
        world.step(1).expect("the step must run");
    }
    assert!(
        world.upgrade_at(address_of(&world, site)).is_some(),
        "the fixture raised no upgrade"
    );
    assert_eq!(
        world.tile_holder(address_of(&world, site)),
        Some(Holder::of(ZERO)),
        "the fixture lost the tile"
    );
    give_presence(&mut world, ZERO, ONE);
    let refused = world.offer_consideration(
        ZERO,
        ONE,
        Consideration::land(vec![site]),
        Consideration::relation(0, 1),
        50,
    );
    assert_eq!(refused, Err(TradeError::UpgradeOnLand(site)));
}

#[test]
fn a_land_side_waits_for_its_price() {
    // A land side is not carried. A laden unit of the debtor standing on the
    // creditor's settlement must deliver nothing against it, so the land side
    // stays undelivered until the other side is paid. The test binds land for
    // a resource and steps: the resource side has no carrier, so nothing
    // settles, and the land does not move.
    let mut world = a_world(7);
    give_presence(&mut world, ZERO, ONE);
    give_presence(&mut world, ONE, ZERO);
    let mine = tiles_held_by(&world, ZERO);
    let tiles = vec![mine[0], mine[1]];
    world
        .offer_consideration(
            ZERO,
            ONE,
            Consideration::land(tiles.clone()),
            Consideration::resource(FOOD, 5),
            50,
        )
        .expect("the offer must be accepted");
    world
        .accept_trade(ONE, ZERO)
        .expect("the acceptance must succeed");
    world.step(1).expect("the step must run");
    let row = world.trade_row(ZERO, ONE).expect("the pair exists");
    assert_eq!(row.status, TRADE_BOUND);
    assert_eq!(row.given, 0, "the land side was delivered before its price");
    assert_eq!(row.taken, 0);
}

#[test]
fn only_the_carried_side_loses_its_direction_when_land_for_a_resource_fails() {
    // Faction zero owes land and faction one owes food. Nobody delivers, so the
    // contract fails at its deadline. The land side waited on the food, so it
    // cannot be short on its own, and faction zero keeps its direction. Faction
    // one loses the direction it would ask on.
    let mut world = a_world(7);
    give_presence(&mut world, ZERO, ONE);
    give_presence(&mut world, ONE, ZERO);
    let mine = tiles_held_by(&world, ZERO);
    world
        .offer_consideration(
            ZERO,
            ONE,
            Consideration::land(vec![mine[0]]),
            Consideration::resource(FOOD, 5),
            2,
        )
        .expect("the offer must be accepted");
    world
        .accept_trade(ONE, ZERO)
        .expect("the acceptance must succeed");
    for _ in 0..3 {
        world.step(1).expect("the step must run");
    }
    let row = world.trade_row(ZERO, ONE).expect("the pair exists");
    assert_eq!(row.status, TRADE_DEFAULTED);
    assert_eq!(
        row.closed_until,
        Tick(0),
        "the land side lost its direction"
    );
    let door = world.trade_row(ONE, ZERO).expect("the pair exists");
    assert!(door.closed_until.0 > 0, "the food side kept its direction");
    assert_eq!(
        row.given, 0,
        "the land side was delivered although its price never arrived"
    );
}

#[test]
fn a_relation_step_delivers_as_a_logged_no_op() {
    let mut world = a_world(7);
    give_presence(&mut world, ZERO, ONE);
    give_presence(&mut world, ONE, ZERO);
    let mut control = world.clone();
    world
        .offer_consideration(
            ZERO,
            ONE,
            Consideration::relation(0, 3),
            Consideration::relation(1, 2),
            50,
        )
        .expect("the offer must be accepted");
    world
        .accept_trade(ONE, ZERO)
        .expect("the acceptance must succeed");
    world.step(1).expect("the step must run");
    control.step(1).expect("the step must run");

    let steps = world
        .trade_log()
        .iter()
        .filter(|event| event.act == ACT_STEP_RELATION)
        .count();
    assert_eq!(steps, 2, "each relation side logs one step");
    let row = world.trade_row(ZERO, ONE).expect("the pair exists");
    assert_eq!(row.status, TRADE_SETTLED);
    assert_eq!(row.given, 3);
    assert_eq!(row.taken, 2);
    // Nothing moved. The holders are those of the control world.
    assert_eq!(holders(&world), holders(&control));
    assert_eq!(world.event_log_bytes(), control.event_log_bytes());
}

#[test]
fn the_board_replaces_whole_and_refuses_more_rows_than_the_bound() {
    let mut world = a_world(7);
    assert!(world.market(ZERO).is_empty());
    let rows = [
        Advert::new(FOOD, 10, ADVERT_OFFERS, WOOD, 4),
        Advert::new(WOOD, 3, ADVERT_WANTS, FOOD, 9),
    ];
    world.advertise(ZERO, &rows).expect("two rows fit");
    let board = world.market(ZERO);
    assert_eq!(board.len(), usize::from(world.board_rows()));
    assert_eq!(&board[..2], &rows);
    assert!(board[2..].iter().all(Advert::is_empty));

    world.advertise(ZERO, &rows[1..]).expect("one row fits");
    let board = world.market(ZERO);
    assert_eq!(board[0], rows[1]);
    assert!(
        board[1..].iter().all(Advert::is_empty),
        "the old row survived the replace"
    );

    let bound = u32::from(world.board_rows());
    let too_many: Vec<Advert> = (0..=bound)
        .map(|n| Advert::new(FOOD, n + 1, ADVERT_OFFERS, WOOD, 1))
        .collect();
    let refused = world.advertise(ZERO, &too_many);
    assert_eq!(refused, Err(TradeError::BoardOverfull(bound + 1, bound)));
    assert_eq!(
        world.market(ZERO)[0],
        rows[1],
        "a refused write changed the board"
    );

    assert_eq!(
        world.advertise(ZERO, &[Advert::new(FOOD, 1, 2, WOOD, 1)]),
        Err(TradeError::NoSuchSide(2))
    );
    assert_eq!(
        world.advertise(ZERO, &[Advert::new(9, 1, ADVERT_OFFERS, WOOD, 1)]),
        Err(TradeError::NoSuchKind(9))
    );
    assert_eq!(
        world.advertise(FactionId(2), &rows),
        Err(TradeError::NoSuchFaction(FactionId(2)))
    );
    assert!(world.market(ONE).iter().all(Advert::is_empty));
    assert!(world.check_invariants());
}

#[test]
fn two_worlds_that_differ_in_one_board_row_have_different_hashes() {
    let mut posted = a_world(7);
    let quiet = posted.clone();
    posted
        .advertise(ONE, &[Advert::new(FOOD, 1, ADVERT_WANTS, WOOD, 1)])
        .expect("one row fits");
    assert_ne!(
        posted.state_hash().finish(),
        quiet.state_hash().finish(),
        "the board is not in the state hash"
    );
    let mut other = quiet.clone();
    other
        .advertise(ONE, &[Advert::new(FOOD, 2, ADVERT_WANTS, WOOD, 1)])
        .expect("one row fits");
    assert_ne!(posted.state_hash().finish(), other.state_hash().finish());
}

#[test]
fn a_land_trade_is_identical_at_every_thread_count() {
    let mut world = a_world(7);
    give_presence(&mut world, ZERO, ONE);
    give_presence(&mut world, ONE, ZERO);
    let held = tiles_held_by(&world, ZERO);
    let tiles: Vec<TileIdx> = held.iter().copied().take(64).collect();
    bind_land_for_a_relation(&mut world, tiles);
    world
        .advertise(ONE, &[Advert::new(FOOD, 1, ADVERT_WANTS, WOOD, 1)])
        .expect("one row fits");

    let run = |threads: usize| {
        let mut trial = world.clone();
        for _ in 0..3 {
            trial.step(threads).expect("the step must run");
        }
        (
            trial.event_log_bytes().to_vec(),
            trial.trade_log_bytes().to_vec(),
            trial.state_hash().finish(),
            holders(&trial),
        )
    };
    let reference = run(THREAD_COUNTS[0]);
    for threads in &THREAD_COUNTS[1..] {
        let produced = run(*threads);
        assert_eq!(
            produced.0, reference.0,
            "the event log differs at {threads} threads"
        );
        assert_eq!(
            produced.1, reference.1,
            "the trade log differs at {threads} threads"
        );
        assert_eq!(
            produced.2, reference.2,
            "the state hash differs at {threads} threads"
        );
        assert_eq!(
            produced.3, reference.3,
            "the holders differ at {threads} threads"
        );
    }
}
