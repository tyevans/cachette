//! Ground changes hands when a cohort holds it, and it returns when the
//! cohort leaves.
//!
//! A tile carries one lease. A lease is one faction and one count, and the
//! count follows the units that stand on the tile. A tile is held by its
//! lease faction at the claim threshold, then by the nearest city in reach,
//! then by nobody. The count falls toward zero on a fixed schedule.[^1]
//!
//! Two more rules make that reachable in a run. A holder in the war band
//! refuses no guest, so a cohort can cross a hostile border.[^2] A campaign
//! that reaches nothing closes at its deadline, so a faction that marched
//! once can march again.[^3]
//!
//! **Every fixture here is built for an extreme, not for a typical world.**
//! A tile inside the reach of an enemy city, a lease at the bound, a cohort
//! that leaves, a border between two factions deep in the war band, and a
//! faction whose idle units are all walking to a project. A fixture that
//! models the typical case never supplies the input that fails the
//! assertion.[^4]
//!
//! The tests drive the world step and the public verbs, because the step and
//! the verbs are what must invoke the rules.[^5]
//!
//! # References
//!
//! [^1]: ADR-0153, a tile's lease follows the units that stand on it, decisions D1, D2, D3, D4 and D5. `docs/adrs/accepted/adr-0153-a-tiles-lease-follows-the-units-that-stand-on-it.md`
//! [^2]: ADR-0167, war opens the border that tension closes, decision D1. `docs/adrs/draft/adr-0167-war-opens-the-border-that-tension-closes.md`
//! [^3]: Findings register, FND-542. `docs/FINDINGS.md`
//! [^4]: Testing rules, section 2a. `.agents/rules/testing.md`
//! [^5]: Testing rules, section 5. `.agents/rules/testing.md`
//! [^6]: ADR-0005, a solver runs a fixed iteration count, decision D1. `docs/adrs/accepted/adr-0005-a-solver-runs-a-fixed-iteration-count.md`
//! [^7]: ADR-0125, one plane names the seed set of a destination field, decision D4. `docs/adrs/draft/adr-0125-one-plane-names-the-seed-set-of-a-destination-field.md`

use cachette_core::campaign;
use cachette_core::holding::{Holder, LeaseRules, ReachRules};
use cachette_core::relation::RelationRules;
use cachette_core::{Axial, Entity, FactionId, Tick, World, WorldConfig};

/// The extent of the worlds below.
///
/// The extent is wide enough that the generator puts water in it, because two
/// fixtures here need a tile whose every neighbour refuses a unit.
const EXTENT: u32 = 192;

/// Builds a world of the extent.
fn world(seed: u64, factions: u16) -> World {
    World::new(WorldConfig {
        width: EXTENT,
        height: EXTENT,
        seed,
        faction_count: factions,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    })
    .expect("the extent must describe a world")
}

/// Returns every address of a world, in tile index order.
fn addresses(world: &World) -> Vec<Axial> {
    let grid = world.grid();
    (0..grid.tile_count())
        .map(|index| Axial::new((index % grid.width()) as i32, (index / grid.width()) as i32))
        .collect()
}

/// Returns an island: an open tile whose every neighbour refuses a unit.
///
/// A unit on an island never moves, whatever the choice pass decides, so the
/// fixture holds a unit on one tile for as many ticks as it needs.
fn island(world: &World) -> Option<Axial> {
    addresses(world).into_iter().find(|address| {
        world.admits_a_unit(*address)
            && world
                .grid()
                .neighbours(*address)
                .iter()
                .all(|side| side.is_none_or(|next| !world.admits_a_unit(next)))
    })
}

/// Returns a pair of open addresses, the second `gap` steps east of the
/// first, or `None` when the seed has no such pair.
fn open_pair(world: &World, gap: i32) -> Option<(Axial, Axial)> {
    addresses(world).into_iter().find_map(|seat| {
        let other = Axial::new(seat.q + gap, seat.r);
        (world.admits_a_unit(seat) && world.admits_a_unit(other)).then_some((seat, other))
    })
}

#[test]
fn a_cohort_that_stands_on_enemy_ground_takes_it_and_gives_it_back_when_it_leaves() {
    // The extreme is a tile deep inside the reach of an enemy city, which
    // the city rule gives to the enemy on every step. A fixture that used
    // unheld ground would pass whatever the lease did, because the city test
    // would already answer nobody.
    //
    // The target is an island, whose every neighbour refuses a unit. A unit
    // on it never moves, so the fixture holds one faction on one tile for as
    // many ticks as the claim needs. A unit on open ground walks off inside
    // a few ticks and the test would then measure the walk.
    let rules = LeaseRules::new(1, 4, 1, 4, 0, 64, 16);
    for seed in 0..40u64 {
        let mut field = world(seed, 2);
        field.set_reach_rules(ReachRules::new(6, 1, 8));
        field.set_lease_rules(rules);
        let Some(target) = island(&field) else {
            continue;
        };
        // The city stands within its reach of the island. The reach is a
        // distance and not a path, so water between them changes nothing.
        let Some(seat) = addresses(&field).into_iter().find(|address| {
            *address != target && address.distance(target) <= 5 && field.admits_a_unit(*address)
        }) else {
            continue;
        };
        if field.found_settlement(seat, FactionId(0)).is_err() {
            continue;
        }
        field.step(1).expect("the step must run");
        assert_eq!(
            field.tile_holder(target).and_then(Holder::faction),
            Some(FactionId(0)),
            "the fixture must start with the enemy holding the target"
        );

        let invader = field
            .spawn_soldier(target, FactionId(1))
            .expect("the ground admits a unit");
        // The lease starts at nobody. It rises by the raise step on each
        // tick and falls by the decay step on each decay tick, so the claim
        // needs more ticks than the threshold alone.
        for _ in 0..(rules.claim_threshold() * rules.decay_period() as i32) {
            field.step(2).expect("the step must run");
        }
        assert!(field.check_invariants());
        assert_eq!(
            field.soldiers().address(invader),
            Some(target),
            "the island did not hold the unit"
        );
        let (holder, count) = field.tile_lease(target).expect("the tile is in the world");
        assert_eq!(
            holder.faction(),
            Some(FactionId(1)),
            "the lease did not turn"
        );
        assert!(count >= rules.claim_threshold(), "the count is {count}");
        assert_eq!(
            field.tile_holder(target).and_then(Holder::faction),
            Some(FactionId(1)),
            "the lease at the claim threshold did not outrank the city"
        );

        // The cohort leaves. The decay must give the tile back, and the city
        // rule must then answer the enemy again.
        assert!(field.despawn_soldier(invader));
        let ticks = (rules.bound() as u32 + 1) * rules.decay_period();
        for _ in 0..ticks {
            field.step(2).expect("the step must run");
        }
        let (holder, count) = field.tile_lease(target).expect("the tile is in the world");
        assert_eq!(count, 0, "the lease did not decay to zero");
        assert!(holder.is_nobody(), "a lease at zero still names a faction");
        assert_eq!(
            field.tile_holder(target).and_then(Holder::faction),
            Some(FactionId(0)),
            "the ground did not return to the city that reaches it"
        );
        return;
    }
    panic!("no seed below 40 gives the fixture an island inside a city reach");
}

#[test]
fn a_tile_with_two_factions_gives_the_tick_to_the_side_with_more_units() {
    // The extreme is an unequal crowd on one tile, and then a tie. A rule
    // that took the first unit it found would take the packing of the arena,
    // which is not a stable key.
    for seed in 0..60u64 {
        let mut field = world(seed, 2);
        field.set_lease_rules(LeaseRules::new(1, 1, 0, 1, 0, 64, 8));
        let Some(tile) = addresses(&field)
            .into_iter()
            .find(|address| field.admits_a_unit(*address))
        else {
            continue;
        };
        // The higher faction spawns first, so a rule that read the packing
        // would answer faction one.
        let mut many = Vec::new();
        for _ in 0..3 {
            let Ok(unit) = field.spawn_soldier(tile, FactionId(1)) else {
                break;
            };
            many.push(unit);
        }
        if many.len() < 3 {
            continue;
        }
        let Ok(one) = field.spawn_soldier(tile, FactionId(0)) else {
            continue;
        };
        field.step(2).expect("the step must run");
        let (holder, _) = field.tile_lease(tile).expect("the tile is in the world");
        assert_eq!(
            holder.faction(),
            Some(FactionId(1)),
            "the tick did not go to the side with more units"
        );

        // Two more units of the lower faction make a tie, and a tie goes to
        // the lowest faction identifier.
        let mut evened: Vec<Entity> = vec![one];
        for _ in 0..2 {
            let Ok(unit) = field.spawn_soldier(tile, FactionId(0)) else {
                break;
            };
            evened.push(unit);
        }
        if evened.len() < 3 {
            continue;
        }
        // The lease of faction one must fall to zero and then turn, so the
        // pass runs until it does.
        for _ in 0..64 {
            field.step(2).expect("the step must run");
        }
        let (holder, _) = field.tile_lease(tile).expect("the tile is in the world");
        assert_eq!(
            holder.faction(),
            Some(FactionId(0)),
            "the tie did not go to the lowest faction identifier"
        );
        return;
    }
    panic!("no seed below 60 gives the fixture the ground it needs");
}

#[test]
fn a_holder_at_war_admits_the_guest_it_refuses_in_tension() {
    // The extreme is a pair held exactly at the war edge, and the same pair
    // one step below it. The rule must answer differently at two values one
    // apart, so a fixture that used one value would measure nothing.
    //
    // The two runs are two worlds of one seed. Every draw is keyed on the
    // system, the tick, the entity and the draw index, so the two guests
    // walk the same walk until a refusal parts them. One world therefore
    // isolates the relation and nothing else.
    //
    // The relation is written on every tick, because the drift moves an
    // entry toward the peace band on a schedule.
    let rules = RelationRules::DEFAULT;
    const TICKS: u32 = 600;
    for seed in 0..40u64 {
        let Some((seat, outside)) = fixture_places(seed) else {
            continue;
        };
        let mut inside_at_war = false;
        let mut inside_at_tension = false;
        for (value, reached) in [
            (rules.war_edge - 1, &mut inside_at_war),
            (rules.war_edge, &mut inside_at_tension),
        ] {
            let mut field = war_world(seed);
            field
                .found_settlement(seat, FactionId(0))
                .expect("the ground admits a city");
            // One step writes the holder column. The admission of the first
            // step reads the column the step before it left, so a guest that
            // walked on tick zero would meet a world in which nobody holds
            // anything.
            field.step(1).expect("the step must run");
            assert_eq!(
                field.tile_holder(outside).and_then(Holder::faction),
                None,
                "the guest starts on ground the holder holds"
            );
            let guest = field
                .spawn_soldier(outside, FactionId(1))
                .expect("the ground admits a unit");
            for _ in 0..TICKS {
                assert!(field.set_relation(FactionId(0), FactionId(1), value));
                field.step(2).expect("the step must run");
                let stands_inside = field
                    .soldiers()
                    .address(guest)
                    .and_then(|address| field.tile_holder(address))
                    .and_then(Holder::faction)
                    == Some(FactionId(0));
                if stands_inside {
                    *reached = true;
                    break;
                }
            }
        }
        if !inside_at_war {
            // The walk of this seed never came near the disc, so the seed
            // supplies no case. It is not a failure of the rule.
            continue;
        }
        assert!(
            !inside_at_tension,
            "a holder in the tension band admitted the guest"
        );
        return;
    }
    panic!("no seed below 40 walks the guest onto the ground of the holder");
}

/// The nearest an objective of the deadline test may stand.
///
/// The cohort walks one tile in a tick, and the deadline is eight ticks, so an
/// objective nearer than this is one the cohort could reach.
const OBJECTIVE_NEAREST: u32 = 16;

/// The furthest an objective of the deadline test may stand.
///
/// The destination field steers a unit toward a seed a bounded number of level
/// 1 cells away. A seed further off leaves the unit reading no direction, and
/// the engine then releases it.
const OBJECTIVE_FURTHEST: u32 = 48;

/// The extent of the world that the border test uses.
///
/// The test needs open ground and a walk, and it needs no island, so it takes
/// a narrower world than the lease test. A narrow world runs the walk in
/// fewer seconds.
const BORDER_EXTENT: u32 = 64;

/// Builds the world that the border test uses.
///
/// Every unit chooses on every tick, so the walk of a guest is a walk and not
/// one step in each interval.
fn war_world(seed: u64) -> World {
    let mut field = World::new(WorldConfig {
        width: BORDER_EXTENT,
        height: BORDER_EXTENT,
        seed,
        faction_count: 2,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    })
    .expect("the extent must describe a world");
    field.set_reach_rules(ReachRules::new(6, 1, 8));
    field
        .set_choice_schedule(0)
        .expect("the exponent is inside the range");
    field
}

/// Returns a seat for the city and a start for the guest, or `None` when the
/// seed has no such pair.
///
/// The start lies one step outside the reach of the city, so every tile of
/// the holder the guest reaches is a tile admission had to let it onto.
fn fixture_places(seed: u64) -> Option<(Axial, Axial)> {
    let field = war_world(seed);
    let seat = addresses(&field)
        .into_iter()
        .find(|address| field.admits_a_unit(*address))?;
    let outside = addresses(&field)
        .into_iter()
        .find(|address| field.admits_a_unit(*address) && address.distance(seat) == 7)?;
    Some((seat, outside))
}

#[test]
fn a_campaign_that_reaches_nothing_closes_and_lets_the_next_one_raise() {
    // The extreme is a cohort that walks and arrives nowhere inside the
    // deadline: an objective many ticks of walking away, with a deadline far
    // below the ticks the walk costs. A fixture whose objective was one step
    // away would arrive and never reach the deadline.
    //
    // **The objective sits inside the reach of the destination field.** The
    // field runs a fixed number of relaxation passes over level 1 cells, so it
    // steers a unit toward a seed a bounded number of cells away and no
    // further.[^6] A unit that reads no direction from the field holds a plane
    // that leads it nowhere, and the engine releases it.[^7] A cohort of
    // released units is empty, and an empty cohort closes the campaign as lost
    // rather than as expired. The far corner of this world is outside that
    // reach, so a fixture that took it measured the release and never the
    // deadline. The window below is wide enough to outlast the deadline and
    // narrow enough for the field to answer.
    for seed in 0..60u64 {
        let mut field = world(seed, 2);
        field.set_campaign_deadline(Tick(8));
        let Some((home, _)) = open_pair(&field, 1) else {
            continue;
        };
        let Some(far) = addresses(&field).into_iter().find(|address| {
            field.admits_a_unit(*address)
                && address.distance(home) > OBJECTIVE_NEAREST
                && address.distance(home) < OBJECTIVE_FURTHEST
        }) else {
            continue;
        };
        let unit = field
            .spawn_soldier(home, FactionId(0))
            .expect("the ground admits a unit");
        assert!(field.raise_campaign(FactionId(0), far, 1).is_ok());
        // A second raise is refused while the first is live.
        assert!(field.raise_campaign(FactionId(0), far, 1).is_err());
        for _ in 0..12 {
            field.step(2).expect("the step must run");
        }
        let rows = field.campaigns_of(FactionId(0));
        // **The lost state is named, so a dissolved cohort cannot read as a
        // deadline that did not fire.** A campaign closes as lost when its
        // cohort is empty, and that test runs before the deadline test. A
        // fixture whose units the engine released would close every campaign
        // as lost, and a bare assertion on the expired state would report only
        // that the campaign did not expire.
        assert!(
            !rows.iter().any(|row| row.state == campaign::STATE_LOST),
            "the campaign closed as lost, so the cohort dissolved before the \
             deadline and this fixture measures the release and not the deadline"
        );
        assert!(
            rows.iter().any(|row| row.state == campaign::STATE_EXPIRED),
            "the campaign did not expire"
        );
        assert!(
            field
                .campaigns_of(FactionId(0))
                .iter()
                .all(|row| !row.is_live()),
            "an expired campaign is still live"
        );
        // The faction can raise again, which is what the deadline is for.
        assert!(
            field.raise_campaign(FactionId(0), far, 1).is_ok(),
            "the next raise was refused"
        );
        assert!(field.soldiers().address(unit).is_some());
        return;
    }
    panic!("no seed below 60 gives the fixture the ground it needs");
}

#[test]
fn a_raise_takes_the_number_of_units_it_asks_for_when_they_are_already_walking() {
    // The extreme is a faction whose every unit is already sent on its own
    // destination plane, which is what a project order leaves. A fixture
    // whose units were idle would pass whatever the raise did.
    for seed in 0..60u64 {
        let mut field = world(seed, 2);
        let Some((home, far)) = open_pair(&field, 8) else {
            continue;
        };
        let mut units = Vec::new();
        for _ in 0..4 {
            let Ok(unit) = field.spawn_soldier(home, FactionId(0)) else {
                break;
            };
            units.push(unit);
        }
        if units.len() < 4 {
            continue;
        }
        // Every unit walks somewhere on the plane of its own faction, which
        // is the number of the faction. Nothing is idle.
        assert!(field.send_units_to(&units, &[far], 0).is_ok());
        let row = field
            .raise_campaign(FactionId(0), far, 4)
            .expect("the raise must find a cohort");
        assert_eq!(
            row.cohort_size, 4,
            "the raise took {} units of the four it asked for",
            row.cohort_size
        );
        return;
    }
    panic!("no seed below 60 gives the fixture the ground it needs");
}
