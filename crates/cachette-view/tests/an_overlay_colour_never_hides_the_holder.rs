//! Two colour tables must stay apart, and a check measures them.
//!
//! The viewer holds one table of faction colours and one table of overlay
//! colours. A tile that a faction holds takes the faction colour over the
//! ground of the tile. An overlay goes on first, and at its high value it all
//! but replaces that ground. The holder tint is then read against the colour
//! of the overlay, and the ground it replaced no longer matters.
//!
//! An overlay colour that sits near a faction colour therefore hides the
//! holder. This happened once. A change to the engine moved which tiles a
//! faction holds, the strongest held tile of one overlay reached the top of
//! its span for the first time, and the holder of that tile stopped
//! reading.[^1]
//!
//! **This check reads the two tables and never draws a frame.** A test that
//! draws measures one fixture. It reports the tiles that one world happens to
//! hold, at the strengths that world happens to reach, through whatever the
//! drawing painted last over the pixel it samples. Three overlay colours sat
//! nearer a faction colour than the one that failed, and all three passed,
//! because no held tile of the fixture reached their high value.[^1] [^2]
//!
//! # References
//!
//! [^1]: Findings register, FND-758. `docs/FINDINGS.md`
//! [^2]: Findings register, FND-759. `docs/FINDINGS.md`

use cachette_view::overlay;
use cachette_view::paint::{colour_distance, faction_colours, LEAST_COLOUR_DISTANCE};

/// Keeps every declared overlay colour clear of every faction colour.
#[test]
fn no_overlay_colour_sits_near_a_faction_colour() {
    for layer in overlay::registered() {
        for (index, &colour) in layer.own_colours().iter().enumerate() {
            for (slot, &faction) in faction_colours().iter().enumerate() {
                let apart = colour_distance(colour, faction);
                assert!(
                    apart >= LEAST_COLOUR_DISTANCE,
                    "colour {index} of the {} overlay is {colour:06x}, and \
                     faction {slot} is {faction:06x}. They stand {apart} apart \
                     against the {LEAST_COLOUR_DISTANCE} the picture asks for, \
                     so the overlay hides that faction at its high value",
                    layer.name(),
                );
            }
        }
    }
}

/// Keeps the faction table above the bar that the faction table sets.
///
/// The bar is the distance between the closest two faction colours. A faction
/// colour that moves nearer another one takes the ground from the number
/// without saying so, and every overlay then clears a weaker check.
#[test]
fn the_faction_table_meets_the_bar_it_sets() {
    let table = faction_colours();
    for (slot, &one) in table.iter().enumerate() {
        for (other_slot, &other) in table.iter().enumerate().skip(slot + 1) {
            let apart = colour_distance(one, other);
            assert!(
                apart >= LEAST_COLOUR_DISTANCE,
                "faction {slot} is {one:06x} and faction {other_slot} is \
                 {other:06x}. They stand {apart} apart, under the \
                 {LEAST_COLOUR_DISTANCE} the picture states, so the bar every \
                 overlay clears is no longer the one the table holds",
            );
        }
    }
}

/// Keeps the declared list of an overlay equal to what the overlay paints.
///
/// The check above measures the declared list. A list that drifted from the
/// paint would leave a painted colour that nothing measures.
#[test]
fn every_overlay_that_declares_a_colour_paints_one_of_them() {
    for layer in overlay::registered() {
        let declared = layer.own_colours();
        if declared.is_empty() {
            continue;
        }
        for value in -2i64..64 {
            let painted = layer.colour(value);
            assert!(
                declared.contains(&painted),
                "the {} overlay paints {painted:06x} at the value {value}, and \
                 that colour is not in the list it declares",
                layer.name(),
            );
        }
    }
}

/// Reads a colour against itself, which must measure nothing.
#[test]
fn the_distance_of_a_colour_from_itself_is_nothing() {
    for &colour in faction_colours() {
        assert_eq!(
            colour_distance(colour, colour),
            0,
            "the measure called {colour:06x} distant from itself",
        );
    }
}

/// Reads a green step against a blue step of the same size.
///
/// A plain sum of the three channel differences calls these two equal. An eye
/// does not. The check would then pass a colour that only the green channel
/// separates from a faction colour.
#[test]
fn the_measure_weights_green_above_blue() {
    let grey = 0x0080_8080;
    let greener = 0x0080_a080;
    let bluer = 0x0080_80a0;
    assert!(
        colour_distance(grey, greener) > colour_distance(grey, bluer),
        "a green step of {} did not measure above a blue step of {}",
        colour_distance(grey, greener),
        colour_distance(grey, bluer),
    );
}
