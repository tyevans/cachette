//! The growth of a population, and the values the growth stage reads.
//!
//! # Why this exists
//!
//! Nothing in this engine created a person while a world ran. The seeding
//! layer founds a group once, and from that tick the population only fell. A
//! faction founds with a very small group, and a campaign raise asks for more
//! people than a faction founds with, so no faction could ever raise a cohort
//! and domination could never fire.[^1]
//!
//! This module holds the values that answer it. A site proposes a birth at a
//! rate its store sets. The free places of the site admit the proposals. An
//! admitted proposal becomes one worker who lives at that site.[^2] [^3]
//!
//! # The two limits, and how they compose
//!
//! The store sets a rate, and the rate proposes. The housing admits, and it
//! never scales the rate. A site with free places grows at the rate its store
//! sets. **A site at its housing bound grows nobody, however much food it
//! holds.** There is no third behaviour between the two.[^4]
//!
//! # What this module does not hold
//!
//! It holds no source of people other than growth, and it takes none away.
//! The production queue is the only consumer of people.[^5]
//!
//! It names no unit type. A grown person is a worker, which is the type the
//! spawn path gives, and the queue is how a faction turns a worker into
//! anything else.[^3]
//!
//! # Determinism
//!
//! Every draw is keyed on the tuple of the system, the tick, the site and the
//! index of the proposal within the site. The site fills the entity slot,
//! because the site is the actor. The proposal index fills the draw slot,
//! because two proposals of one site in one tick have nothing else to
//! distinguish them.[^6] No draw here holds state, and no item here uses a
//! floating-point type.[^7]
//!
//! # The values
//!
//! Every value below is a placeholder. The balance register holds one row for
//! each of them, marks each unset, and records how the placeholder was
//! chosen.[^8]
//!
//! # References
//!
//! [^1]: ADR-0157, a site's free places are its built housing less the residents the engine counts, the context. `docs/adrs/accepted/adr-0157-a-sites-free-places-are-its-built-housing-less-the-residents-the-engine-counts.md`
//! [^2]: ADR-0082, the store sets the rate of a birth and the housing admits it, decisions D1 and D2. `docs/adrs/draft/adr-0082-the-store-sets-the-rate-of-a-birth-and-the-housing-admits-it.md`
//! [^3]: ADR-0157, a site's free places are its built housing less the residents the engine counts, decision D5. `docs/adrs/accepted/adr-0157-a-sites-free-places-are-its-built-housing-less-the-residents-the-engine-counts.md`
//! [^4]: ADR-0157, a site's free places are its built housing less the residents the engine counts, decision D2. `docs/adrs/accepted/adr-0157-a-sites-free-places-are-its-built-housing-less-the-residents-the-engine-counts.md`
//! [^5]: ADR-0158, a site builds a typed unit from a bounded queue its store pays for, decision D4. `docs/adrs/accepted/adr-0158-a-site-builds-a-typed-unit-from-a-bounded-queue-its-store-pays-for.md`
//! [^6]: ADR-0082, the store sets the rate of a birth and the housing admits it, decision D4. `docs/adrs/draft/adr-0082-the-store-sets-the-rate-of-a-birth-and-the-housing-admits-it.md`
//! [^7]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
//! [^8]: Balance register, the population. `docs/reference/balance.md`

use crate::rng;
use crate::site::COMMODITY_COUNT;
use crate::types::Fix32;

/// The proposals that one site makes in one application, at the most.
///
/// **This is the width of the inner loop and not a budget.** It multiplies
/// the site count to give the cost of the growth stage, so it is small and it
/// is fixed. It never follows the population.[^1]
///
/// # References
///
/// [^1]: ADR-0157, a site's free places are its built housing less the residents the engine counts, decision D4. `docs/adrs/accepted/adr-0157-a-sites-free-places-are-its-built-housing-less-the-residents-the-engine-counts.md`
pub const PROPOSAL_CEILING: u32 = 4;

/// The scale that a birth chance is drawn against.
///
/// The chance is a Q16.16 value between zero and one, so the draw runs
/// against the raw value of one. The chance is therefore exact in the
/// fixed-point scale, and no part of it is a floating-point number.[^1]
///
/// # References
///
/// [^1]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
pub const CHANCE_SCALE: u64 = Fix32::ONE.0 as u64;

/// The placeholder housing that one person takes.[^1]
///
/// # References
///
/// [^1]: Balance register, the population, the housing per person row. `docs/reference/balance.md`
pub const HOUSING_PER_PERSON_DEFAULT: u32 = 1;

/// The placeholder housing that a founded site starts with.[^1]
///
/// # References
///
/// [^1]: Balance register, the population, the founding housing row. `docs/reference/balance.md`
pub const FOUNDING_HOUSING_DEFAULT: u32 = 16;

/// The placeholder store that one birth costs.[^1]
///
/// # References
///
/// [^1]: Balance register, the population, the food per birth row. `docs/reference/balance.md`
pub const FOOD_PER_BIRTH_DEFAULT: [Fix32; COMMODITY_COUNT] = [Fix32::ONE; COMMODITY_COUNT];

/// The placeholder chance that one proposal becomes a birth.[^1]
///
/// # References
///
/// [^1]: Balance register, the population, the birth rate row. `docs/reference/balance.md`
pub const BIRTH_CHANCE_DEFAULT: Fix32 = Fix32(Fix32::ONE.0 / 2);

/// The placeholder period of the growth schedule.[^1]
///
/// # References
///
/// [^1]: Balance register, the population, the growth schedule row. `docs/reference/balance.md`
pub const GROWTH_PERIOD_DEFAULT: u32 = 10;

/// The placeholder phase of the growth schedule.[^1]
///
/// # References
///
/// [^1]: Balance register, the population, the growth schedule row. `docs/reference/balance.md`
pub const GROWTH_PHASE_DEFAULT: u32 = 0;

/// Returns the free places of a site.
///
/// The free places are the people the housing holds, less the residents the
/// site has. The housing is a quantity of housing, so the people it holds is
/// that quantity divided by the housing one person takes.[^1]
///
/// **The answer is never below zero.** A site above its housing is a state of
/// the world and not a fault, and it has no free place.
///
/// A housing per person of zero gives no free place, because a housing that
/// nobody occupies states no bound.
///
/// # References
///
/// [^1]: ADR-0157, a site's free places are its built housing less the residents the engine counts, decisions D1 and D2. `docs/adrs/accepted/adr-0157-a-sites-free-places-are-its-built-housing-less-the-residents-the-engine-counts.md`
#[must_use]
pub const fn free_places(housing: u32, housing_per_person: u32, residents: u32) -> u32 {
    if housing_per_person == 0 {
        return 0;
    }
    let held = housing / housing_per_person;
    held.saturating_sub(residents)
}

/// Returns how many proposals one site makes in one application.
///
/// The store sets the rate. A site proposes once for each whole birth its
/// store pays for, up to the ceiling on the inner loop. A site with no
/// surplus proposes none.[^1]
///
/// **The free places take no part in this answer.** The housing admits the
/// proposals afterwards, and it never scales the rate.[^2]
///
/// # References
///
/// [^1]: ADR-0082, the store sets the rate of a birth and the housing admits it, decision D1. `docs/adrs/draft/adr-0082-the-store-sets-the-rate-of-a-birth-and-the-housing-admits-it.md`
/// [^2]: ADR-0082, the store sets the rate of a birth and the housing admits it, decision D2. `docs/adrs/draft/adr-0082-the-store-sets-the-rate-of-a-birth-and-the-housing-admits-it.md`
#[must_use]
pub fn proposals(store: &[Fix32; COMMODITY_COUNT], cost: &[Fix32; COMMODITY_COUNT]) -> u32 {
    let mut fewest = PROPOSAL_CEILING;
    for (held, wanted) in store.iter().zip(cost.iter()) {
        if wanted.0 <= 0 {
            continue;
        }
        let affordable = if held.0 <= 0 {
            0
        } else {
            (held.0 / wanted.0) as u32
        };
        if affordable < fewest {
            fewest = affordable;
        }
    }
    fewest
}

/// Reports whether one proposal of one site becomes a birth.
///
/// The draw is keyed on the system, the tick, the site and the index of the
/// proposal within the site. Two sites in one tick draw different values, one
/// site draws a different value in the next tick, and two proposals of one
/// site in one tick draw different values.[^1]
///
/// A chance at or above one always takes, and a chance at or below zero never
/// takes.
///
/// # References
///
/// [^1]: ADR-0082, the store sets the rate of a birth and the housing admits it, decision D4. `docs/adrs/draft/adr-0082-the-store-sets-the-rate-of-a-birth-and-the-housing-admits-it.md`
#[must_use]
pub fn proposal_takes(seed: u64, tick: u64, site: u32, index: u32, chance: Fix32) -> bool {
    if chance.0 <= 0 {
        return false;
    }
    if chance.0 as u64 >= CHANCE_SCALE {
        return true;
    }
    let drawn = rng::draw_below(
        seed,
        rng::SYSTEM_GROWTH,
        tick,
        u64::from(site),
        index,
        CHANCE_SCALE,
    );
    drawn < chance.0 as u64
}
