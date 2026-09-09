//! A verb names a place, and the place is a cell of the egocentric frame.
//!
//! # What these tests hold
//!
//! A draft record says that a verb names a place by a cell of the egocentric
//! frame the observation publishes.[^1] The campaign verb is the one verb
//! that takes such a place. The record makes four claims that a test can
//! reach.
//!
//! A place argument reaches the verb, and the engine marches on a settlement
//! that stands in the named cell.[^2] A place the faction cannot honour is a
//! refusal, and the engine writes the log row without acting.[^3] The
//! legality answer is a fixed length at every world size.[^4] One action
//! integer names one relative place on a world of any size.[^4]
//!
//! # The fixture supplies the case
//!
//! A faction marches only when it holds a seat, when it holds no live
//! campaign, when it is at war, and when it has observed a settlement it
//! could march on. A world that runs the built-in controller alone reaches
//! none of that inside a short run, because the controllers do not declare a
//! war. The fixture therefore sets the relation of every ordered pair above
//! the war edge, and it runs the world long enough for the factions to walk
//! and to see each other.[^5]
//!
//! # References
//!
//! [^1]: ADR-0197, a verb names a place by a cell of the egocentric frame the observation publishes, decisions D1 and D2. `docs/adrs/draft/adr-0197-a-verb-names-a-place-by-a-cell-of-the-egocentric-frame.md`
//! [^2]: ADR-0197, a verb names a place by a cell of the egocentric frame the observation publishes, decision D3. `docs/adrs/draft/adr-0197-a-verb-names-a-place-by-a-cell-of-the-egocentric-frame.md`
//! [^3]: ADR-0197, a verb names a place by a cell of the egocentric frame the observation publishes, decision D6. `docs/adrs/draft/adr-0197-a-verb-names-a-place-by-a-cell-of-the-egocentric-frame.md`
//! [^4]: ADR-0197, a verb names a place by a cell of the egocentric frame the observation publishes, decision D4. `docs/adrs/draft/adr-0197-a-verb-names-a-place-by-a-cell-of-the-egocentric-frame.md`
//! [^5]: Testing Rules, section 2a. `.agents/rules/testing.md`

use cachette_core::action::{
    place_cell, ActionSchema, ActionShape, CandidateKind, Verb, PLACE_ANYWHERE, PLACE_COUNT,
};
use cachette_core::controller::Choice;
use cachette_core::obs_ring::cell_of_delta;
use cachette_core::{Axial, FactionId, TileIdx, World, WorldConfig};

const THREADS: usize = 2;

/// The people each founding settles. Small, so a step is cheap.
const GROUP: u32 = 8;

/// The seed that seats three factions near enough to see each other.
const SEED: u64 = 29;

/// The faction whose seat lies between the other two, so it observes one.
const MARCHER: FactionId = FactionId(2);

/// The seats the fixture founds, in faction order.
///
/// The addresses are absolute, and the terrain of one address follows the
/// seed rather than the extent. Two worlds of two extents therefore seat the
/// same three factions on the same ground.
const SEATS: [Axial; 3] = [Axial::new(10, 10), Axial::new(22, 10), Axial::new(10, 22)];

/// The ticks the fixture runs before it reads the answer.
///
/// The factions walk, they see each other, and one of them holds no live
/// campaign at this tick. A shorter run supplies no legal place, so the
/// assertions below would never meet the case they guard.
const TICKS: u32 = 32;

/// Builds a world of one extent, seats three factions, and sets every
/// ordered pair at war.
///
/// Returns nothing when the extent refuses one of the foundings.
fn fixture(width: u32, height: u32) -> Option<World> {
    let mut world = World::new(WorldConfig {
        width,
        height,
        seed: SEED,
        faction_count: 3,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    })
    .expect("the extent describes a world");
    for (faction, address) in SEATS.iter().enumerate() {
        world
            .found_group_at(*address, GROUP, FactionId(faction as u16))
            .ok()?;
    }
    let war = world.relation_rules().war_edge - 8;
    for first in 0..3u16 {
        for second in 0..3u16 {
            if first != second {
                assert!(world.set_relation(FactionId(first), FactionId(second), war));
            }
        }
    }
    for _ in 0..TICKS {
        world.step(THREADS).expect("the step runs");
    }
    Some(world)
}

/// Returns every place value of the campaign verb the answer calls legal.
fn legal_places(world: &World, faction: FactionId) -> Vec<u32> {
    let schema = world.action_schema();
    let answer = world
        .legal_actions(faction)
        .expect("the faction is of this world");
    (0..PLACE_COUNT)
        .filter(|place| {
            schema
                .encode(Verb::Campaign, &[*place])
                .is_some_and(|action| answer[action as usize] == 1)
        })
        .collect()
}

/// Returns the cell of the egocentric frame that one tile falls in, for one
/// faction.
///
/// The centre comes from the ring stack the observation publishes, so this
/// reads the frame a policy reads and not a frame of the test's own.
fn cell_of(world: &World, faction: FactionId, tile: TileIdx) -> u32 {
    let centre = world
        .faction_ring_stack(faction)
        .expect("the faction is of this world")
        .centre();
    let address = world
        .grid()
        .address_of(tile)
        .expect("the tile is inside the world");
    cell_of_delta(Axial::new(address.q - centre.q, address.r - centre.r))
}

#[test]
fn the_campaign_verb_declares_one_place_position() {
    let schema = ActionSchema::of(ActionShape { faction_count: 3 });
    let row = schema
        .row(Verb::Campaign)
        .expect("the table holds the campaign verb");
    assert_eq!(
        row.positions.len(),
        1,
        "the campaign verb declares one position"
    );
    let position = &row.positions[0];
    assert_eq!(position.candidate, CandidateKind::Place);
    assert_eq!(
        position.bound, PLACE_COUNT,
        "the bound is the cell count of the frame, plus the whole-frame value"
    );
    assert_eq!(
        row.rows, PLACE_COUNT,
        "the verb holds one row for each place value"
    );
    // The first value names no cell. Every value above it names one.
    assert_eq!(place_cell(PLACE_ANYWHERE), None);
    assert_eq!(place_cell(1), Some(0));
    assert_eq!(place_cell(PLACE_COUNT - 1), Some(PLACE_COUNT - 2));
    assert_eq!(place_cell(PLACE_COUNT), None);
}

#[test]
fn a_place_argument_reaches_the_verb() {
    let mut world = fixture(48, 48).expect("the extent seats three factions");
    let schema = world.action_schema();
    let places = legal_places(&world, MARCHER);
    // **The fixture must supply a narrowed place**, or the assertions below
    // never meet the case they guard.
    let narrowed = places
        .iter()
        .copied()
        .find(|place| *place != PLACE_ANYWHERE)
        .expect("the fixture supplied no legal narrowed place");
    let cell = place_cell(narrowed).expect("the value names a cell");

    // **The place value is load-bearing.** A verb that ignored the value and
    // marched on the objective of the whole frame would pass the assertion
    // below, because the fixture supplies one legal cell. The negative half
    // closes that: a narrowed value the answer refuses must raise nothing.
    let ignored = (1..PLACE_COUNT)
        .find(|place| !places.contains(place))
        .expect("the fixture supplied no refused narrowed place");
    let refused_action = schema
        .encode(Verb::Campaign, &[ignored])
        .expect("the place is inside the bound");
    let quiet = world.campaign_log().len();
    assert!(
        !world.act(MARCHER, refused_action),
        "the verb ignored the place value and marched anyway"
    );
    assert_eq!(
        world.campaign_log().len(),
        quiet,
        "a refused narrowed place raised a campaign"
    );

    let action = schema
        .encode(Verb::Campaign, &[narrowed])
        .expect("the place is inside the bound");
    let before = world.campaign_log().len();
    assert!(
        world.act(MARCHER, action),
        "the verb refused a row the answer allowed"
    );
    let raised = &world.campaign_log()[before..];
    assert_eq!(raised.len(), 1, "one action raises one campaign");
    let objective = TileIdx(raised[0].objective_tile);

    // **The engine marched on a settlement of the named cell.** The centre
    // and the cell rule come from the frame the observation publishes, so
    // the cell of the objective is the cell the action named.
    assert_eq!(
        cell_of(&world, MARCHER, objective),
        cell,
        "the campaign marched outside the cell the action named"
    );
}

#[test]
fn an_illegal_place_is_refused_without_acting() {
    let mut world = fixture(48, 48).expect("the extent seats three factions");
    let schema = world.action_schema();
    let places = legal_places(&world, MARCHER);
    assert!(
        places.contains(&PLACE_ANYWHERE),
        "the fixture must let the faction march somewhere, or a refusal proves nothing"
    );
    // **The fixture must supply a refused place.** Most cells of the frame
    // hold no settlement the faction has seen, so most values are refused.
    let refused = (0..PLACE_COUNT)
        .find(|place| !places.contains(place))
        .expect("the fixture supplied no refused place");

    let action = schema
        .encode(Verb::Campaign, &[refused])
        .expect("the place is inside the bound");
    let campaigns = world.campaign_log().len();
    let commands = world.controller_log().len();
    assert!(
        !world.act(MARCHER, action),
        "the verb took a place the answer refused"
    );
    assert_eq!(
        world.campaign_log().len(),
        campaigns,
        "a refused place raised a campaign"
    );
    // **The refusal writes the log row.** The engine records the action and
    // the byte that says the verb did not take it.
    let written = &world.controller_log()[commands..];
    assert_eq!(written.len(), 1, "the refusal writes one row");
    assert_eq!(written[0].action, action);
    assert_eq!(written[0].applied, 0);
    assert_eq!(written[0].faction, MARCHER);
}

#[test]
fn the_answer_does_not_grow_with_the_world() {
    // Three extents, one faction count. The table and the answer are one
    // length at every extent, so a bound follows no part of the world
    // extent.
    let mut lengths: Vec<usize> = Vec::new();
    let mut schemas: Vec<ActionSchema> = Vec::new();
    let mut tiles: Vec<u32> = Vec::new();
    for (width, height) in [(24u32, 24u32), (48, 48), (96, 96)] {
        let world = World::new(WorldConfig {
            width,
            height,
            seed: SEED,
            faction_count: 3,
            unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
        })
        .expect("the extent describes a world");
        tiles.push(world.grid().tile_count());
        schemas.push(world.action_schema());
        lengths.push(
            world
                .legal_actions(FactionId(0))
                .expect("the faction is of this world")
                .len(),
        );
    }
    // The fixture must vary the thing the assertion holds constant.
    assert!(
        tiles[0] < tiles[1] && tiles[1] < tiles[2],
        "the three worlds must hold three tile counts"
    );
    assert_eq!(schemas[0], schemas[1]);
    assert_eq!(schemas[1], schemas[2]);
    assert_eq!(lengths[0], lengths[1]);
    assert_eq!(lengths[1], lengths[2]);
}

#[test]
fn one_integer_names_one_place_at_two_map_sizes() {
    let small = fixture(48, 48).expect("the small extent seats three factions");
    let large = fixture(96, 96).expect("the large extent seats three factions");
    assert!(
        large.grid().tile_count() > small.grid().tile_count(),
        "the two worlds must hold two extents"
    );

    let places = legal_places(&small, MARCHER);
    assert!(
        places.iter().any(|place| *place != PLACE_ANYWHERE),
        "the fixture supplied no legal narrowed place"
    );
    // **One integer names one relative place.** The seats sit at the same
    // addresses, so the settlements sit at the same delta from the same
    // frame centre. The answer therefore calls the same place values legal,
    // whatever the extent of the world around them.
    assert_eq!(
        places,
        legal_places(&large, MARCHER),
        "one place value named two places on two extents"
    );
}

#[test]
fn the_controller_choice_lands_on_the_whole_frame_row() {
    // **The built-in controller names no cell.** Its campaign choice
    // therefore encodes to the row of the whole-frame value, so the verb set
    // of the table does not move and a window of controller commands still
    // reads as one distribution over the table.
    let schema = ActionSchema::of(ActionShape { faction_count: 3 });
    let choice = Choice::Campaign {
        kind: 0,
        tile: TileIdx(7),
    };
    assert_eq!(
        choice.action(&schema),
        schema.encode(Verb::Campaign, &[PLACE_ANYWHERE]),
        "the controller's campaign choice left the whole-frame row"
    );
    let row = schema
        .row(Verb::Campaign)
        .expect("the table holds the campaign verb");
    assert_eq!(
        choice.action(&schema),
        Some(row.first),
        "the whole-frame row is the first row of the verb"
    );
}
