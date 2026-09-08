//! The block of the observation that says what happened to a faction lately,
//! and who did it.
//!
//! The rest of the observation is a snapshot of one frame. A snapshot cannot
//! tell a faction that holds two hundred tiles and is gaining from one that
//! holds two hundred and is losing. It cannot say that a rival is taking a
//! city now, and it cannot say that one rival keeps killing the people of the
//! reader. This block answers those three questions from the decayed event
//! history that the step advances.[^1]
//!
//! # Every value is a share, and no value is a count
//!
//! **A raw count of events is not comparable between two worlds.** Twenty
//! units lost is a rout on a small world and a skirmish on a large one, and a
//! policy trained on one shape could not run on the other. Each kind of event
//! therefore names the stock of the reader that the loss or the gain came out
//! of, and this module divides the counter by that stock.[^2]
//!
//! The counter of one kind reaches the arrival rate of that kind shifted left
//! by the decay bits, so the division is by the stock shifted left by the same
//! bits.[^1] The published value is therefore the part of the stock that the
//! events of one step move, in the fixed-point scale the array holds. Two
//! worlds of different size and the same event rate publish the same value.
//!
//! # Three views of one counter set
//!
//! The block publishes the recent share, the lasting share, and the signed
//! relation between the two. Both shares reach the same value for the same
//! constant arrival rate, so the relation reads zero while the rate holds,
//! positive while the rate rises and negative while it falls. **That is the
//! reading that separates a spike from a trend.** One rival that takes a city
//! this minute raises the recent share alone. One rival that keeps killing the
//! people of the reader raises both.
//!
//! # The rivals, without a seat number
//!
//! **No position of this block is indexed by a seat number.** A league seats
//! one policy in one seat for one game and in another seat for the next, so a
//! policy that learned a seat number reads another faction's quantities under
//! the same weight.[^3] This block therefore publishes two order statistics
//! over the rivals, which carry no identity at all: the share of the damage
//! that the single worst rival caused, and how concentrated the damage is over
//! the whole field.
//!
//! The concentration is the sum of the squared shares. Two rivals in equal
//! measure give one half. Four in equal measure give one quarter. One rival
//! that does everything gives one. A policy reads it to tell one enemy from a
//! field of them.
//!
//! The joint detail, which says whether the worst rival is also the nearest,
//! belongs in the block of rival tokens that a permutation-invariant encoder
//! consumes.[^4] That block reads the same rival shares this module reads.
//!
//! # Determinism
//!
//! Every pass here visits the seats in ascending order. The maximum and the
//! sum are order independent, and the order is stated all the same.[^5]
//!
//! # Where the layout of this block is stated
//!
//! **This module states no position of the array.** The kind list states how
//! many positions each field of this block holds, and the schema of the
//! observation states where each field starts.[^6] A caller that needs a
//! position asks the schema.
//!
//! # References
//!
//! [^1]: The event history and its decay. [`crate::event_memory`]
//! [^2]: Research report 42, what a policy should be able to see, section 8.1. `docs/research/reports/42-what-a-policy-should-be-able-to-see.md`
//! [^3]: Findings register, FND-647. `docs/FINDINGS.md`
//! [^4]: Research report 42, what a policy should be able to see, section 6.4. `docs/research/reports/42-what-a-policy-should-be-able-to-see.md`
//! [^5]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
//! [^6]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decision D1. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`

use crate::event_memory::{Decay, MemoryKind};
use crate::sim_math;
use crate::types::{Accum, FactionId, Fix32};
use crate::world::World;

/// The fixed-point value of one, which every share of this block bounds at.
pub(crate) const ONE: i64 = Fix32::ONE.0 as i64;

/// Writes the share of each kind at one memory length.
///
/// The value is the part of the stock of the reader that the events of one
/// step move. A faction that holds none of the stock of a kind reads the
/// bound, because it lost or gained the whole of what it had.
pub(crate) fn write_shares(world: &World, faction: FactionId, length: Decay, span: &mut [i64]) {
    for kind in MemoryKind::ALL {
        let Some(place) = span.get_mut(kind.position()) else {
            continue;
        };
        *place = kind_share(world, faction, kind, length);
    }
}

/// Writes the signed relation between the recent share and the lasting share
/// of each kind.
///
/// A positive value says that the rate of the kind is above its own trend, and
/// a negative value says that it is below. A steady rate reads zero.
pub(crate) fn write_trends(world: &World, faction: FactionId, span: &mut [i64]) {
    for kind in MemoryKind::ALL {
        let Some(place) = span.get_mut(kind.position()) else {
            continue;
        };
        let recent = kind_share(world, faction, kind, Decay::Recent);
        let lasting = kind_share(world, faction, kind, Decay::Lasting);
        *place = signed_relation(recent, lasting);
    }
}

/// Writes the share of each kind that the single worst rival caused.
///
/// The value answers which part of the damage of one kind comes from one
/// rival. It names no rival, because a position of this array names no
/// seat.[^1]
///
/// # References
///
/// [^1]: Findings register, FND-647. `docs/FINDINGS.md`
pub(crate) fn write_worst_rival(world: &World, faction: FactionId, span: &mut [i64]) {
    for kind in MemoryKind::ALL {
        let Some(slot) = kind.blame_position() else {
            continue;
        };
        let Some(place) = span.get_mut(slot) else {
            continue;
        };
        let shares = rival_shares(world, faction, kind, Decay::Lasting);
        *place = i64::from(shares.iter().copied().max().unwrap_or(0));
    }
}

/// Writes how concentrated the damage of each kind is over the rivals.
///
/// The value is the sum of the squared rival shares. Two rivals in equal
/// measure give one half, and one rival that does everything gives one.
pub(crate) fn write_concentration(world: &World, faction: FactionId, span: &mut [i64]) {
    for kind in MemoryKind::ALL {
        let Some(slot) = kind.blame_position() else {
            continue;
        };
        let Some(place) = span.get_mut(slot) else {
            continue;
        };
        let mut total = 0i64;
        for share in rival_shares(world, faction, kind, Decay::Lasting) {
            total += i64::from(sim_math::mul(Fix32(share), Fix32(share)).0);
        }
        *place = total.clamp(0, ONE);
    }
}

/// Returns the share of the damage of one kind that each rival caused, in
/// ascending seat order and without the reader itself.
///
/// **The block of rival tokens reads this same function.** A token holds the
/// share of each kind that its own rival caused, and it reads the shares in
/// the order the tokens sort in.[^1] One statement of the share therefore
/// serves both blocks, and neither can disagree with the other.[^2]
///
/// The denominator is the sum of the counters over the rivals, which is what
/// the whole field caused. A kind that nothing caused gives every rival a
/// share of zero.
///
/// # References
///
/// [^1]: Research report 42, what a policy should be able to see, section 6.4. `docs/research/reports/42-what-a-policy-should-be-able-to-see.md`
/// [^2]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
pub(crate) fn rival_shares(
    world: &World,
    faction: FactionId,
    kind: MemoryKind,
    length: Decay,
) -> Vec<i32> {
    let memory = world.event_memory();
    let seats = world.faction_count().max(1);
    let mut counters = Vec::with_capacity(usize::from(seats));
    let mut whole = 0i64;
    for seat in 0..seats {
        if seat == faction.0 {
            continue;
        }
        let held = memory.blamed_total(faction, FactionId(seat), kind, length);
        whole = whole.saturating_add(held);
        counters.push(held);
    }
    counters
        .iter()
        .map(|held| share(*held, whole) as i32)
        .collect()
}

/// Returns the share of the stock of one kind that the events of one step
/// move.
fn kind_share(world: &World, faction: FactionId, kind: MemoryKind, length: Decay) -> i64 {
    let held = world.event_memory().total(faction, kind, length);
    let stock = world.memory_stock(faction, kind.stock());
    let scale = 1i64 << length.bits();
    share(held, stock.saturating_mul(scale))
}

/// Returns a part of a whole, in the fixed-point scale the array holds.
///
/// A whole of zero or below gives zero, because a share of nothing is
/// nothing. The division truncates toward zero, which is the rounding rule of
/// the whole observation.
fn share(part: i64, whole: i64) -> i64 {
    if part <= 0 || whole <= 0 {
        return 0;
    }
    sim_math::share(Accum(ONE), Accum(part), Accum(whole))
        .map_or(0, |value| value.0)
        .clamp(0, ONE)
}

/// Returns the signed relation between two magnitudes.
///
/// The result is the difference over the sum of the two magnitudes, so it
/// lies between minus one and one without a chosen denominator. Two equal
/// values give zero.
fn signed_relation(first: i64, second: i64) -> i64 {
    let scale = first
        .saturating_abs()
        .saturating_add(second.saturating_abs());
    if scale == 0 {
        return 0;
    }
    sim_math::share(Accum(ONE), Accum(first - second), Accum(scale))
        .map_or(0, |value| value.0)
        .clamp(-ONE, ONE)
}
