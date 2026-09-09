//! Prints the length of the observation and the place of a few fields.
//!
//! The example exists so that a change to the layout can be measured before
//! and after itself. A revision that moves a position must raise the version,
//! and a revision that moves nothing must not.

fn main() {
    let schema = cachette_core::faction_observation::observation_schema();
    println!("version={} length={}", schema.version(), schema.length());
    for name in [
        "memory_recent",
        "memory_lasting",
        "memory_trend",
        "memory_worst_rival",
        "memory_concentration",
        "layout_reserve",
        "military_strength",
        "power_strength",
    ] {
        let row = schema
            .row(name)
            .expect("the schema must declare every field this example names");
        println!("{name} start={} positions={}", row.start, row.positions);
    }
}
