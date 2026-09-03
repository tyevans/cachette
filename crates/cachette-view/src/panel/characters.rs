//! The panel that shows the characters the world has promoted.
//!
//! This file is the whole panel. It states its own name, its own title and
//! its own lines, and the standard draws it.[^1]
//!
//! # What the panel shows
//!
//! The panel walks the character tier once and reads the identity, the
//! faction, the sex, the birth tick, the house and the parents of each
//! character it shows.[^2] Every one of those fields has a write site that a
//! step of the engine reaches: the arena writes them when it creates a
//! character or bears one.[^3]
//!
//! **The panel does not show a renown.** The arena carries a renown column,
//! but no step of the engine writes to it. Only a test writes it directly.
//! A field that nothing in a step writes is a capability the panel must not
//! declare, because a renown of zero would then read as a real value that
//! nothing set.[^4]
//!
//! # The bound
//!
//! The character tier holds a bounded population, and a walk of it is the
//! one walk this panel may run.[^5] Even so, the panel draws a row for at
//! most a fixed count of characters. The count beside the rows states how
//! many characters the world holds, so a reader knows the list is the first
//! few and not the whole population.
//!
//! # References
//!
//! [^1]: The panel standard. `crates/cachette-view/src/panel/mod.rs`
//! [^2]: ADR-0070, the head-up display reports what the drawing pass read, decision D1. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
//! [^3]: The living character column set. `crates/cachette-core/src/character.rs`
//! [^4]: Recurring Defect Shapes, shape 3. `.claude/rules/recurring-defects.md`
//! [^5]: ADR-0054, an entity belongs to one of three tiers, declared at creation, decision D1. `docs/adrs/accepted/adr-0054-an-entity-belongs-to-one-of-three-tiers-declared-at-creation.md`

use super::{Line, Panel, View};
use crate::hud;
use crate::paint::faction_colour;

/// The most character rows this panel draws.
///
/// **This is a bound on the cost of the panel, and not a budget on the
/// population.** The character tier may hold far more than this many
/// characters. A count beside the rows states the true population, so a
/// reader is never told that the list is the whole of it.
const CHARACTER_ROWS: u32 = 6;

/// The panel that shows the characters the world has promoted.
pub struct Characters;

impl Panel for Characters {
    fn name(&self) -> &'static str {
        "characters"
    }

    fn title(&self) -> &'static str {
        "THE CHARACTERS"
    }

    fn lines(&self, view: &View<'_>) -> Vec<Line> {
        let arena = view.world.characters();
        let live = arena.len();
        if live == 0 {
            return vec![Line::note("no characters exist")];
        }

        let mut lines = vec![
            Line::row("population", hud::grouped(u64::from(live))),
            Line::Rule,
        ];

        for character in arena.iter().take(CHARACTER_ROWS as usize) {
            let (Some(faction), Some(sex)) = (arena.faction(character), arena.sex(character))
            else {
                // A character that the walk just named must resolve, because
                // the walk only ever names a live one. The guard is here so
                // that a future change to the walk cannot draw a row with a
                // value it did not read.
                continue;
            };
            lines.push(Line::swatch(
                faction_colour(faction),
                format!("character {}", character.index()),
                sex.to_string(),
            ));
        }

        if live > CHARACTER_ROWS {
            lines.push(Line::note(format!(
                "first {CHARACTER_ROWS} of {} shown",
                hud::grouped(u64::from(live))
            )));
        }

        if let Some(focused) = focused_character(view) {
            lines.extend(focused);
        }

        lines
    }
}

/// Returns the lines that describe the character of the focused unit.
///
/// Returns nothing when the drawing pass fixed on no unit, when that unit
/// carries no character, or when the character it names is gone. Showing
/// the character of the unit nearest the middle of the window costs no
/// walk of its own, because the focus is already read.[^1]
///
/// # References
///
/// [^1]: The panel standard. `crates/cachette-view/src/panel/mod.rs`
fn focused_character(view: &View<'_>) -> Option<Vec<Line>> {
    let focus = view.focus?;
    let character = view.world.unit_character(focus.entity())??;
    let arena = view.world.characters();
    let faction = arena.faction(character)?;
    let sex = arena.sex(character)?;
    let birth = arena.birth(character)?;

    let mut lines = vec![
        Line::Rule,
        Line::heading("FOCUSED CHARACTER"),
        Line::swatch(
            faction_colour(faction),
            format!("character {}", character.index()),
            sex.to_string(),
        ),
        Line::row("born", hud::grouped(birth.0)),
    ];

    if let Some(house) = arena.house(character) {
        lines.push(Line::row(
            "house",
            format!("founder {}", house.founder().birth_order()),
        ));
    }
    if let Some(parents) = arena.parents(character) {
        let value = if parents.is_founder() { "none" } else { "two" };
        lines.push(Line::row("parents", value));
    }

    Some(lines)
}

#[cfg(test)]
mod tests {
    //! Tests for the character browser panel.
    //!
    //! A test builds a small world of its own rather than the demonstration
    //! world, because the demonstration world is chosen to look right and
    //! not to reach an extreme.[^1]
    //!
    //! # References
    //!
    //! [^1]: Testing Rules, section 2a. `.claude/rules/testing.md`

    use cachette_core::types::FACTION_CEILING;
    use cachette_core::{FactionId, World, WorldConfig, CHARACTER_CEILING};

    use super::*;
    use crate::paint::Camera;

    /// Builds a small world with no character.
    fn empty_world() -> World {
        World::new(WorldConfig {
            width: 8,
            height: 8,
            seed: 1,
            faction_count: 4,
            ..WorldConfig::default()
        })
        .expect("the world builds")
    }

    /// Builds a view over a world, with no focus and no pointer.
    fn view_of(world: &World) -> View<'_> {
        View {
            world,
            camera: Camera {
                tile_width: 1.0,
                tile_height: 1.0,
                origin_x: 0.0,
                origin_y: 0.0,
            },
            frame_width: 900,
            frame_height: 700,
            focus: None,
            pointer: None,
        }
    }

    #[test]
    fn a_world_with_no_character_says_so_and_shows_no_zero_row() {
        let world = empty_world();
        let view = view_of(&world);
        let lines = Characters.lines(&view);
        assert_eq!(lines, vec![Line::note("no characters exist")]);
    }

    #[test]
    fn a_world_with_characters_names_them_and_the_values_match_what_you_created() {
        let mut world = empty_world();
        let first = world
            .create_character(FactionId(0))
            .expect("the arena holds a character");
        let second = world
            .create_character(FactionId(1))
            .expect("the arena holds a character");

        let view = view_of(&world);
        let lines = Characters.lines(&view);
        let said: Vec<String> = lines.iter().filter_map(Line::says).collect();

        let arena = world.characters();
        for character in [first, second] {
            let sex = arena.sex(character).expect("the character is live");
            let expected = format!("character {}: {sex}", character.index());
            assert!(
                said.contains(&expected),
                "the panel must state {expected}, it said {said:?}"
            );
        }
        assert!(said.contains(&"population: 2".to_string()));
    }

    #[test]
    fn no_line_is_cut_at_the_widest_plausible_values() {
        // The widest identity a live character can carry is one below the
        // ceiling of the character tier. The widest faction is one below the
        // project ceiling. The widest plausible tick is large but short of
        // the type's own range, because a tick counts steps and not an
        // unbounded quantity.
        let widest_index = CHARACTER_CEILING - 1;
        let widest_faction = FACTION_CEILING - 1;
        let widest_tick: u64 = 4_000_000_000;

        let population_row = Line::row("population", hud::grouped(u64::from(widest_index)));
        assert!(!population_row.is_cut());

        let swatch_row = Line::swatch(
            faction_colour(FactionId(widest_faction)),
            format!("character {widest_index}"),
            "female".to_string(),
        );
        assert!(!swatch_row.is_cut());

        let born_row = Line::row("born", hud::grouped(widest_tick));
        assert!(!born_row.is_cut());

        let house_row = Line::row("house", format!("founder {widest_index}"));
        assert!(!house_row.is_cut());

        let parents_row = Line::row("parents", "none");
        assert!(!parents_row.is_cut());

        let shown_note = Line::note(format!(
            "first {CHARACTER_ROWS} of {} shown",
            hud::grouped(u64::from(widest_index))
        ));
        assert!(!shown_note.is_cut());

        let none_note = Line::note("no characters exist");
        assert!(!none_note.is_cut());

        let heading = Line::heading("FOCUSED CHARACTER");
        assert!(!heading.is_cut());
    }

    #[test]
    fn the_row_count_stays_bounded() {
        let mut world = empty_world();
        for _ in 0..CHARACTER_ROWS + 3 {
            world
                .create_character(FactionId(0))
                .expect("the arena holds a character");
        }

        let view = view_of(&world);
        let lines = Characters.lines(&view);
        let rows = lines
            .iter()
            .filter(|line| matches!(line, Line::Swatch(_, _, _)))
            .count();
        assert_eq!(rows as u32, CHARACTER_ROWS);
        assert!(lines
            .iter()
            .filter_map(Line::says)
            .any(|said| said.contains("shown")));
    }
}
