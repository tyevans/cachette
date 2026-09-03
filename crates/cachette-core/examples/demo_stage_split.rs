//! Prices every stage of a frame on the demonstration world.
//!
//! Every measurement of a frame so far was taken at the target extent. A cost
//! that does not follow the world would be invisible in all of them, and it
//! would dominate a world this small. This prints the split so the two shapes
//! can be compared.
use cachette_core::stage::{self, STAGES};
use cachette_core::{World, WorldConfig};

fn main() {
    let mut args = std::env::args().skip(1);
    let width: u32 = args.next().and_then(|a| a.parse().ok()).unwrap_or(256);
    let group: u32 = args.next().and_then(|a| a.parse().ok()).unwrap_or(64);
    let threads: usize = args.next().and_then(|a| a.parse().ok()).unwrap_or(4);
    let frames: u64 = 120;

    let mut world = World::new(WorldConfig {
        width,
        height: width,
        seed: 0x0123_4567_89ab_cdef,
        faction_count: 4,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    })
    .expect("world");
    let _ = world.found_run_for_every_faction(group);
    // Warm up, so the split is of a running world and not of its first frames.
    for _ in 0..30 {
        world.step(threads).expect("step");
    }

    stage::reset();
    for _ in 0..frames {
        world.step(threads).expect("step");
    }
    let costs = stage::costs();

    let tiles = u64::from(width) * u64::from(width);
    let units = world.soldiers().iter().count();
    let total = costs.total_nanos();
    println!("world {width} by {width} = {tiles} tiles, {units} units, {threads} threads");
    println!(
        "frame total {} ms",
        thousandths_of_a_millisecond(total, frames)
    );
    // The hash is printed so that a sweep over thread counts is also a
    // determinism check. One binary must give one answer at any thread count.
    println!("state hash {}", world.state_hash());
    println!();
    println!(
        "{:<30} {:>11} {:>8} {:>9}",
        "stage", "ms a frame", "share", "entries"
    );
    let mut rows: Vec<_> = STAGES
        .iter()
        .map(|stage| (*stage, costs.cost(*stage)))
        .collect();
    rows.sort_by_key(|(_, cost)| std::cmp::Reverse(cost.nanos));
    for (stage, cost) in rows {
        if cost.nanos == 0 {
            continue;
        }
        let nested = if stage.is_nested() { "  nested" } else { "" };
        println!(
            "{:<30} {:>11} {:>7}% {:>9}{nested}",
            stage.name(),
            thousandths_of_a_millisecond(cost.nanos, frames),
            tenths_of_a_percent(cost.nanos, total),
            cost.entries
        );
    }
}

/// Returns a mean in milliseconds, to three decimal places, as text.
///
/// **The arithmetic is integer.** A float here would be a display convenience
/// in a binary that also prints the state hash, and the lint that bans the
/// float types does not distinguish a print from a sum. It should not: this
/// project holds the boundary with a lint and a script because one is not
/// enough, and an exception carved for a print is where the next sum
/// hides.[^1]
///
/// # References
///
/// [^1]: ADR-0002, state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
fn thousandths_of_a_millisecond(nanos: u64, frames: u64) -> String {
    let frames = frames.max(1);
    let thousandths = nanos / frames / 1_000;
    format!("{}.{:03}", thousandths / 1_000, thousandths % 1_000)
}

/// Returns a share of a total as a percentage, to one decimal place, as text.
///
/// The arithmetic is integer, for the reason the function above gives.[^1]
///
/// # References
///
/// [^1]: ADR-0002, state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
fn tenths_of_a_percent(part: u64, whole: u64) -> String {
    // A total of zero means no stage recorded anything, so every share is
    // zero. The division is guarded rather than special-cased in the text, so
    // one format statement produces every row.
    let tenths = part.saturating_mul(1_000).checked_div(whole).unwrap_or(0);
    format!("{}.{}", tenths / 10, tenths % 10)
}
