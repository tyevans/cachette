//! The settings that build a world, and the defaults they take.
//!
//! The settings are one plain value. A caller reads them, changes a field and
//! builds a world from the result. They sit apart from the world because they
//! describe the world before it exists.

use crate::weather::{Latitudes, WeatherError};

/// The settings that build a world.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WorldConfig {
    /// The number of columns in the world.
    ///
    /// The world is a rhombus, so the extent is a width and a height and the
    /// tile count follows from them.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0017, the world is a rhombus, so a tile index is raw axial, decision D1. `docs/adrs/accepted/adr-0017-the-world-is-a-rhombus-so-a-tile-index-is-raw-axial.md`
    pub width: u32,
    /// The number of rows in the world.
    pub height: u32,
    /// The world seed. Every random draw takes it.
    pub seed: u64,
    /// The number of factions.
    ///
    /// The ceiling is 63, because a faction is one bit in a 64-bit mask and
    /// one value is reserved for no faction. The scale constants table holds
    /// the value.[^1]
    ///
    /// # References
    ///
    /// [^1]: Budgets and costs, the scale constants. `docs/reference/budgets.md`
    pub faction_count: u16,
    /// The number of unit slots that the world reserves.
    ///
    /// The world reserves this many entries in each unit column when it is
    /// built, and it opens no more. A spawn past the reservation gets a
    /// typed refusal.[^1]
    ///
    /// **This is the one place that states the reservation.** The arena
    /// takes the value from here and names no default of its own, so no
    /// second site can disagree with this one.[^2]
    ///
    /// The reservation is paid once, at construction. The cost of a tick
    /// grows with the number of units that live, not with the number the
    /// world reserved.[^3]
    ///
    /// # References
    ///
    /// [^1]: ADR-0084, the world reserves the unit columns at construction, decisions D1 and D3. `docs/adrs/draft/adr-0084-the-world-reserves-the-unit-columns-at-construction.md`
    /// [^2]: ADR-0084, the world reserves the unit columns at construction, decision D2. `docs/adrs/draft/adr-0084-the-world-reserves-the-unit-columns-at-construction.md`
    /// [^3]: PRD-0012, a world starts small and grows. `docs/product/accepted/prd-0012-a-world-starts-small-and-grows.md`
    pub unit_capacity: u32,
    /// The latitude of the middle row of the world, in hundredths of a
    /// degree.
    ///
    /// **A map is one region of a planet, and the region stands somewhere.**
    /// The row axis of the world carries a latitude, and this value says
    /// where the middle of that axis stands. A positive value is north of the
    /// equator.[^1]
    ///
    /// The default stands at forty-five degrees north, which is the middle
    /// latitude of the temperate belt.
    ///
    /// # References
    ///
    /// [^1]: ADR-0177, the row axis of a world is a latitude that the world states, decision D1. `docs/adrs/draft/adr-0177-the-row-axis-of-a-world-is-a-latitude-that-the-world-states.md`
    pub latitude_centre: i32,
    /// The latitude from the first row of the world to the last, in
    /// hundredths of a degree.
    ///
    /// **The default is three degrees, because that is how wide a world of
    /// the target tile count is.** The scale register derives a world extent
    /// of about 330 kilometres from the tile edge at the target tile count,
    /// and that distance is about three degrees of latitude.[^1]
    ///
    /// A caller that wants the published pressure belts states a span from
    /// pole to pole instead, which is 18000 hundredths of a degree. No belt
    /// is three degrees wide, so a world of the default span carries one
    /// prevailing wind and not three.[^2]
    ///
    /// **This is the one place that states the span.** The weather field, the
    /// climate spin and the Python constructor all take it from here, so no
    /// second site can disagree with this one.[^3]
    ///
    /// # References
    ///
    /// [^1]: Budgets and costs, the scale constants. `docs/reference/budgets.md`
    /// [^2]: ADR-0177, the row axis of a world is a latitude that the world states, decision D1. `docs/adrs/draft/adr-0177-the-row-axis-of-a-world-is-a-latitude-that-the-world-states.md`
    /// [^3]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
    pub latitude_span: i32,
}

impl WorldConfig {
    /// The population that the project targets, counted over everybody.
    ///
    /// One million is the whole population. Soldiers are a fraction of it,
    /// and civilians are not separate entities on top of the million. The
    /// project owner answered this, and the scale constants table holds the
    /// row.[^1]
    ///
    /// This is the reservation that a world takes when the caller states no
    /// other. It is a target the project chose, not a figure anybody
    /// measured.[^2]
    ///
    /// # References
    ///
    /// [^1]: Budgets and costs, the scale constants. `docs/reference/budgets.md`
    /// [^2]: Blockers register, BLK-007. `docs/BLOCKERS.md`
    pub const TARGET_UNIT_POPULATION: u32 = 1_000_000;

    /// The number of destination planes that a world holds when the caller
    /// states no other.
    ///
    /// **This is a fixture-facing parameter and not a budget.** It says how
    /// many places a control plane may send units to at one time, before it
    /// re-aims a plane it already used. No record holds the value and no
    /// measurement chooses it.[^1]
    ///
    /// # References
    ///
    /// [^1]: Blockers register, BLK-007. `docs/BLOCKERS.md`
    pub const DEFAULT_DESTINATION_COUNT: u16 = 4;

    /// The destination planes that one faction climbs at the same time.
    ///
    /// A faction climbs three planes, and none of the three may yield to
    /// another. The first carries its campaign, its carriers and its project
    /// order, which already take turns among themselves. The second carries a
    /// crossing, because a faction on an island that waits for its war to end
    /// waits for a war it cannot reach. The third carries a settling, because
    /// a faction fights for most of a run and a settler that waits for the
    /// war to end never founds anything.
    ///
    /// **This is a structural property of the controller and not a budget.**
    /// It counts the purposes the controller sends units for, and each of the
    /// three is a purpose that stands in the code.
    pub const PLANES_FOR_ONE_FACTION: u16 = 3;

    /// The settings that a caller gets when it states none.
    ///
    /// **This is the one site that states each default.** The trait
    /// implementation below returns this value, so a constant and a runtime
    /// caller read the same settings and no second site can disagree with
    /// this one.[^1]
    ///
    /// # References
    ///
    /// [^1]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
    pub const DEFAULT: Self = Self {
        width: 64,
        height: 64,
        seed: 0x0123_4567_89ab_cdef,
        faction_count: 4,
        unit_capacity: Self::TARGET_UNIT_POPULATION,
        // The region span is declared once, beside the planet span it stands
        // against. This reads it rather than repeating it.
        latitude_centre: Latitudes::REGION.centre(),
        latitude_span: Latitudes::REGION.span(),
    };

    /// Returns the latitudes that the rows of this world stand at.
    ///
    /// **Every reader of the span goes through this call.** The weather
    /// field and the climate spin both read the span of the world they run
    /// over, so neither of them holds a default of its own.[^1]
    ///
    /// # Errors
    ///
    /// Returns an error when the span is negative, when it is wider than the
    /// globe, and when either end of it passes a pole.
    ///
    /// # References
    ///
    /// [^1]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
    pub const fn latitudes(&self) -> Result<Latitudes, WeatherError> {
        Latitudes::new(self.latitude_centre, self.latitude_span)
    }

    /// Returns the destination planes that a world of this shape holds.
    ///
    /// The count gives each faction the planes it climbs at the same time,
    /// and it never falls below the count a caller that states no faction
    /// gets. A caller may raise it or lower it afterwards.
    #[must_use]
    pub const fn destination_plane_count(&self) -> u16 {
        let wanted = self
            .faction_count
            .saturating_mul(Self::PLANES_FOR_ONE_FACTION);
        if wanted > Self::DEFAULT_DESTINATION_COUNT {
            wanted
        } else {
            Self::DEFAULT_DESTINATION_COUNT
        }
    }
}

impl Default for WorldConfig {
    fn default() -> Self {
        Self::DEFAULT
    }
}
