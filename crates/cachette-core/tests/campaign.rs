//! A faction at war raises a campaign, and a faction at peace raises none.
//!
//! Every test that reads the controller drives the engine through `step`
//! and reads the campaign register, the campaign log or the census. The raise
//! itself is one public verb, and the tests of the cohort rule call it as a
//! caller would.[^1]
//!
//! The campaign draw gets one test for each field of its key, because the
//! determinism tests cannot tell a draw keyed on the wrong field from a
//! correct one.[^2] The fixture reaches the extremes: two seated factions at
//! peace, the same two at war, a busy unit beside an idle one, and an
//! objective that changes holder.[^3]
//!
//! # References
//!
//! [^1]: Testing Rules, sections 5 and 6. `.agents/rules/testing.md`
//! [^2]: Testing Rules, section 2. `.agents/rules/testing.md`
//! [^3]: Testing Rules, section 2a. `.agents/rules/testing.md`

use cachette_core::campaign::{
    self, wants_campaign, COHORT_SIZE_DEFAULT, EVENT_LOST, EVENT_RAISED, EVENT_WON,
    OBJECTIVE_TAKE_SITE, STATE_LIVE, STATE_LOST, STATE_WON,
};
use cachette_core::types::Entity;
use cachette_core::unit_type::{SOLDIER, WORKER};
use cachette_core::{
    Axial, CampaignError, FactionId, FactionWeights, Tick, World, WorldConfig, COMMAND_CAMPAIGN,
};

const A: FactionId = FactionId(0);
const B: FactionId = FactionId(1);

/// The people each founding settles. Small, so a step is cheap.
const GROUP: u32 = 8;

/// The most ticks a test waits for the war weight to roll a raise. The
/// weight is at least one in nine, so the wait is generous.
const PATIENCE: usize = 400;

fn config(seed: u64) -> WorldConfig {
    WorldConfig {
        width: 48,
        height: 48,
        seed,
        faction_count: 2,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    }
}

fn addresses(world: &World) -> impl Iterator<Item = Axial> {
    let grid = world.grid();
    (0..grid.tile_count())
        .map(move |index| Axial::new((index % grid.width()) as i32, (index / grid.width()) as i32))
}

/// Founds one group for each of the two factions, far enough apart that the
/// survey does not refuse the second for the first, and returns the two
/// settlement tiles.
fn seat_two(world: &mut World) -> [Axial; 2] {
    let mut places: Vec<Axial> = Vec::new();
    for faction in 0..2u16 {
        let mut founded = None;
        for spacing in [12, 0] {
            let candidates: Vec<Axial> = addresses(world).collect();
            for address in candidates {
                if !world.admits_a_unit(address) {
                    continue;
                }
                if places.iter().any(|place| {
                    (place.q - address.q).abs() < spacing || (place.r - address.r).abs() < spacing
                }) {
                    continue;
                }
                if world
                    .found_group_at(address, GROUP, FactionId(faction))
                    .is_ok()
                {
                    founded = Some(address);
                    break;
                }
            }
            if founded.is_some() {
                break;
            }
        }
        places.push(founded.expect("the faction finds a place"));
    }
    [places[0], places[1]]
}

/// Puts the pair deep in the war band, so that the drift toward peace cannot
/// end the war inside the wait. One step of drift moves one, one period at a
/// time, so the depth is the wait itself.
fn declare_war(world: &mut World) {
    let war = world.relation_rules().war_edge - PATIENCE as i32;
    assert!(world.set_relation(A, B, war));
    assert!(world.set_relation(B, A, war));
    assert!(world.at_war(A, B));
}

fn census(world: &World, name: &str) -> i64 {
    world
        .subsystem_census()
        .iter()
        .find(|(row, _)| *row == name)
        .map(|row| row.1)
        .expect("the row exists")
}

/// Steps until the controller raises a campaign for A, and returns the tick.
fn step_until_raised(world: &mut World, threads: usize) -> u64 {
    for _ in 0..PATIENCE {
        world.step(threads).expect("the step runs");
        if world
            .campaign_log()
            .iter()
            .any(|event| event.kind == EVENT_RAISED && event.faction == A)
        {
            return world.tick().0;
        }
    }
    panic!("the controller raised no campaign in {PATIENCE} ticks");
}

const fn middling() -> FactionWeights {
    FactionWeights {
        war: 4,
        trade: 4,
        build: 4,
        renown: 4,
    }
}

fn draws(seed: u64, faction: FactionId, draw: u32, first_tick: u64) -> Vec<bool> {
    (0..64)
        .map(|offset| wants_campaign(seed, Tick(first_tick + offset), faction, draw, middling()))
        .collect()
}

#[test]
fn the_tick_is_in_the_campaign_draw_key() {
    let a = draws(7, A, 3, 1);
    let b = draws(7, A, 3, 2);
    assert_ne!(a, b, "shifting the tick by one must change the sequence");
    assert!(a.iter().any(|yes| *yes) && a.iter().any(|yes| !*yes));
}

#[test]
fn the_faction_is_in_the_campaign_draw_key() {
    assert_ne!(draws(7, A, 3, 1), draws(7, B, 3, 1));
}

#[test]
fn the_draw_index_is_in_the_campaign_draw_key() {
    assert_ne!(draws(7, A, 3, 1), draws(7, A, 4, 1));
}

#[test]
fn the_seed_is_in_the_campaign_draw_key() {
    assert_ne!(draws(7, A, 3, 1), draws(8, A, 3, 1));
}

#[test]
fn the_war_weight_biases_the_campaign_draw() {
    let count = |war: u8| {
        (0..512u64)
            .filter(|tick| {
                wants_campaign(9, Tick(*tick), A, 3, FactionWeights { war, ..middling() })
            })
            .count()
    };
    assert!(count(8) > count(1), "a higher war weight raises more often");
}

#[test]
fn nearest_site_takes_the_lowest_distance_and_then_the_lowest_slot() {
    use cachette_core::types::TileIdx;
    let tile = campaign::nearest_site(
        [
            (5, 2, TileIdx(20)),
            (3, 9, TileIdx(90)),
            (3, 4, TileIdx(40)),
        ]
        .into_iter(),
    );
    assert_eq!(tile, Some(TileIdx(40)));
    let reversed = campaign::nearest_site(
        [
            (3, 4, TileIdx(40)),
            (3, 9, TileIdx(90)),
            (5, 2, TileIdx(20)),
        ]
        .into_iter(),
    );
    assert_eq!(
        reversed,
        Some(TileIdx(40)),
        "the order does not reach the answer"
    );
    assert_eq!(campaign::nearest_site(std::iter::empty()), None);
}

#[test]
fn a_faction_at_peace_raises_no_campaign() {
    let mut world = World::new(config(31)).expect("the extent describes a world");
    seat_two(&mut world);
    assert!(!world.at_war(A, B));
    for _ in 0..PATIENCE {
        world.step(2).expect("the step runs");
        assert!(world.campaign_log().is_empty(), "no campaign at peace");
        assert!(
            world
                .controller_log()
                .iter()
                .all(|command| command.kind != COMMAND_CAMPAIGN),
            "the controller plans no campaign at peace"
        );
    }
    assert!(world.campaigns_of(A).iter().all(|row| !row.is_set()));
    assert!(world.campaigns_of(B).iter().all(|row| !row.is_set()));
    assert_eq!(census(&world, "campaigns_raised"), 0);
}

#[test]
fn a_faction_at_war_raises_a_campaign_on_the_nearest_enemy_site() {
    let mut world = World::new(config(32)).expect("the extent describes a world");
    let [_, seat_b] = seat_two(&mut world);
    declare_war(&mut world);
    let tick = step_until_raised(&mut world, 2);
    let raised: Vec<_> = world
        .campaign_log()
        .iter()
        .filter(|event| event.kind == EVENT_RAISED && event.faction == A)
        .collect();
    assert_eq!(
        raised.len(),
        1,
        "one raise, because one campaign may be live"
    );
    let event = raised[0];
    assert_eq!(event.tick.0, tick);
    assert_eq!(event.objective_kind, OBJECTIVE_TAKE_SITE);
    let objective = world
        .grid()
        .address_of(cachette_core::types::TileIdx(event.objective_tile))
        .expect("the objective is a tile");
    assert_eq!(objective, seat_b, "the only enemy site is the objective");
    assert!(event.cohort_size >= 1 && event.cohort_size <= COHORT_SIZE_DEFAULT);
    assert_eq!(census(&world, "campaigns_raised"), 1);
    // The controller log holds the command, and the verb took it.
    let command = world
        .controller_log()
        .iter()
        .find(|command| command.kind == COMMAND_CAMPAIGN && command.faction == A)
        .expect("the command is logged");
    assert_eq!(command.applied, 1);
    assert_eq!(command.argument, OBJECTIVE_TAKE_SITE);
    assert_eq!(
        command.sequence,
        world.controller_evaluations() + 1,
        "the campaign draw index is one past the relation draw"
    );
    // The register holds one live row, and the cohort marches as soldiers
    // on the plane of the faction.
    let live: Vec<_> = world
        .campaigns_of(A)
        .iter()
        .filter(|row| row.is_live())
        .collect();
    assert_eq!(live.len(), 1);
    assert_eq!(live[0].state, STATE_LIVE);
    assert_eq!(live[0].raised_at.0, tick);
    let marching: Vec<Entity> = world
        .soldiers()
        .iter_faction(A)
        .filter(|unit| world.sent_to(*unit) == Some(Some(A.0)))
        .collect();
    assert_eq!(marching.len() as u32, event.cohort_size);
    assert!(marching
        .iter()
        .all(|unit| world.unit_type(*unit) == Some(SOLDIER)));
}

#[test]
fn a_faction_holds_at_most_one_live_campaign() {
    let mut world = World::new(config(33)).expect("the extent describes a world");
    let [_, seat_b] = seat_two(&mut world);
    declare_war(&mut world);
    step_until_raised(&mut world, 2);
    assert_eq!(
        world.raise_campaign(A, seat_b, 1),
        Err(CampaignError::LiveCampaign)
    );
    // The controller plans none either, for as long as the first is live.
    for _ in 0..64 {
        world.step(2).expect("the step runs");
        let live = world
            .campaigns_of(A)
            .iter()
            .filter(|row| row.is_live())
            .count();
        assert!(live <= 1, "two live campaigns for one faction");
        // A raise on a tick means the earlier campaign closed, on this tick
        // or on an earlier one, so the one live row is the new one.
        let log = world.campaign_log();
        if log
            .iter()
            .any(|event| event.kind == EVENT_RAISED && event.faction == A)
        {
            let fresh = world
                .campaigns_of(A)
                .iter()
                .filter(|row| row.is_live())
                .all(|row| row.raised_at == world.tick());
            assert!(fresh, "a raise while the first campaign was live");
        }
        let applied = world
            .controller_log()
            .iter()
            .filter(|command| {
                command.kind == COMMAND_CAMPAIGN && command.faction == A && command.applied == 1
            })
            .count();
        assert!(applied <= 1, "two raises for one faction on one tick");
    }
}

/// Spawns `count` workers of A on one open tile and returns them in spawn
/// order, which is ascending identity order in a fresh arena.
fn spawn_row(world: &mut World, count: usize) -> Vec<Entity> {
    let open = addresses(world)
        .find(|address| world.admits_a_unit(*address))
        .expect("some ground admits a unit");
    (0..count)
        .map(|_| {
            world
                .spawn_soldier(open, A)
                .expect("the tile admits the group")
        })
        .collect()
}

#[test]
fn the_cohort_is_the_lowest_identities_among_the_idle_units() {
    let mut world = World::new(config(34)).expect("the extent describes a world");
    let units = spawn_row(&mut world, 6);
    let far = addresses(&world)
        .filter(|address| world.admits_a_unit(*address))
        .last()
        .expect("some ground admits a unit");
    // The lowest identity is busy: a caller sent it elsewhere on another
    // plane. The raise must step over it.
    world
        .send_units_to(&units[..1], &[far], 3)
        .expect("the send is valid");
    let row = world
        .raise_campaign(A, far, 3)
        .expect("three idle units exist");
    assert_eq!(row.cohort_size, 3);
    let mut cohort: Vec<Entity> = units
        .iter()
        .copied()
        .filter(|unit| world.sent_to(*unit) == Some(Some(A.0)))
        .collect();
    cohort.sort_by_key(|unit| unit.to_bits());
    assert_eq!(
        cohort,
        units[1..4].to_vec(),
        "the three lowest idle identities"
    );
    assert_eq!(
        world.unit_type(units[0]),
        Some(WORKER),
        "the busy unit keeps its type"
    );
    assert_eq!(
        world.sent_to(units[0]),
        Some(Some(3)),
        "the busy unit keeps its order"
    );
    for unit in &units[1..4] {
        assert_eq!(world.unit_type(*unit), Some(SOLDIER));
    }
    for unit in &units[4..] {
        assert_eq!(world.unit_type(*unit), Some(WORKER));
        assert_eq!(world.sent_to(*unit), Some(None));
    }
    assert_eq!(
        world.campaign_log().len(),
        1,
        "the raise logs one event until the next step empties the log"
    );
}

#[test]
fn the_raise_refuses_what_a_caller_would_be_refused() {
    let mut world = World::new(config(35)).expect("the extent describes a world");
    let open = addresses(&world)
        .find(|address| world.admits_a_unit(*address))
        .expect("some ground admits a unit");
    assert_eq!(
        world.raise_campaign(A, open, 2),
        Err(CampaignError::NoIdleUnit)
    );
    spawn_row(&mut world, 2);
    assert_eq!(
        world.raise_campaign(A, open, 0),
        Err(CampaignError::EmptyCohort)
    );
    assert_eq!(
        world.raise_campaign(FactionId(9), open, 2),
        Err(CampaignError::NoSuchFaction(9))
    );
    assert_eq!(
        world.raise_campaign(A, Axial::new(-1, -1), 2),
        Err(CampaignError::OutsideWorld(Axial::new(-1, -1)))
    );
    world.set_destination_count(0);
    assert_eq!(
        world.raise_campaign(A, open, 2),
        Err(CampaignError::NoPlane(0))
    );
}

#[test]
fn a_campaign_is_lost_when_every_unit_of_the_cohort_has_fallen() {
    let mut world = World::new(config(36)).expect("the extent describes a world");
    let units = spawn_row(&mut world, 3);
    let far = addresses(&world)
        .filter(|address| world.admits_a_unit(*address))
        .last()
        .expect("some ground admits a unit");
    world
        .raise_campaign(A, far, 2)
        .expect("two idle units exist");
    world.step(2).expect("the step runs");
    assert!(
        world.campaigns_of(A)[0].is_live(),
        "the cohort still stands"
    );
    assert!(world.despawn_soldier(units[0]));
    assert!(world.despawn_soldier(units[1]));
    world.step(2).expect("the step runs");
    let row = world.campaigns_of(A)[0];
    assert_eq!(row.state, STATE_LOST);
    assert_eq!(world.campaign_log().len(), 1);
    assert_eq!(world.campaign_log()[0].kind, EVENT_LOST);
    assert_eq!(world.campaign_log()[0].cohort_size, 2);
    // The third unit was never in the cohort, and a new raise may take it.
    assert_eq!(world.sent_to(units[2]), Some(None));
    assert!(world.raise_campaign(A, far, 1).is_ok());
}

#[test]
fn a_campaign_is_won_when_the_objective_passes_to_the_campaigner() {
    let mut world = World::new(config(37)).expect("the extent describes a world");
    let units = spawn_row(&mut world, 2);
    let objective = addresses(&world)
        .filter(|address| world.admits_a_unit(*address))
        .last()
        .expect("some ground admits a unit");
    assert_eq!(
        world
            .holding()
            .holder(objective)
            .and_then(|holder| holder.faction()),
        None,
        "nobody holds the objective yet"
    );
    world
        .raise_campaign(A, objective, 2)
        .expect("two idle units exist");
    world.step(2).expect("the step runs");
    assert!(world.campaigns_of(A)[0].is_live());
    // A crowd of A arrives at the objective by another road, and the holding
    // pass gives A the ground. The campaign then closes as won, and the
    // cohort is released from its order.
    for _ in 0..8 {
        if world.spawn_soldier(objective, A).is_err() {
            break;
        }
    }
    let mut won = false;
    for _ in 0..8 {
        world.step(2).expect("the step runs");
        if world
            .campaign_log()
            .iter()
            .any(|event| event.kind == EVENT_WON)
        {
            won = true;
            break;
        }
    }
    assert!(won, "the holder passed to A and the campaign did not close");
    assert_eq!(world.campaigns_of(A)[0].state, STATE_WON);
    assert_eq!(census(&world, "campaigns_won"), 1);
    for unit in &units {
        assert_eq!(
            world.sent_to(*unit),
            Some(None),
            "a survivor goes back to its own choice"
        );
    }
}

/// Runs the war fixture to a raise and then on, and returns what the
/// determinism tests compare.
fn run_at_war(threads: usize) -> (Vec<u8>, Vec<u8>, u64) {
    let mut world = World::new(config(38)).expect("the extent describes a world");
    seat_two(&mut world);
    declare_war(&mut world);
    step_until_raised(&mut world, threads);
    for _ in 0..12 {
        world.step(threads).expect("the step runs");
    }
    (
        world.event_log_bytes().to_vec(),
        world.campaign_log_bytes().to_vec(),
        world.state_hash().finish(),
    )
}

#[test]
fn a_world_with_a_campaign_in_flight_gives_one_answer_at_every_thread_count() {
    let (log, campaigns, hash) = run_at_war(1);
    for threads in [2, 12] {
        let (other_log, other_campaigns, other_hash) = run_at_war(threads);
        assert_eq!(log, other_log, "the event log differs at {threads} threads");
        assert_eq!(
            campaigns, other_campaigns,
            "the campaign log differs at {threads} threads"
        );
        assert_eq!(
            hash, other_hash,
            "the state hash differs at {threads} threads"
        );
    }
}

#[test]
fn the_campaign_register_enters_the_state_hash() {
    let mut one = World::new(config(39)).expect("the extent describes a world");
    let mut other = World::new(config(39)).expect("the extent describes a world");
    spawn_row(&mut one, 2);
    spawn_row(&mut other, 2);
    let far = addresses(&one)
        .filter(|address| one.admits_a_unit(*address))
        .last()
        .expect("some ground admits a unit");
    // Both raise, so the arenas agree. Only one register differs, because
    // one campaign closed by hand.
    one.raise_campaign(A, far, 1).expect("an idle unit exists");
    other
        .raise_campaign(A, far, 1)
        .expect("an idle unit exists");
    assert_eq!(one.state_hash().finish(), other.state_hash().finish());
    other.set_campaign_cohort_size(COHORT_SIZE_DEFAULT + 1);
    assert_ne!(
        one.state_hash().finish(),
        other.state_hash().finish(),
        "a parameter the step reads must enter the hash"
    );
}
