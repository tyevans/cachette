//! The panel that shows what is known about one character.
//!
//! This file is the whole panel. It states its own name, its own title and its
//! own lines, and the standard draws it.[^1]
//!
//! # References
//!
//! [^1]: The panel standard. `crates/cachette-view/src/panel/mod.rs`

use super::{Line, Panel, View};

/// The panel that shows what is known about one character.
pub struct Characters;

impl Panel for Characters {
    fn name(&self) -> &'static str {
        "characters"
    }

    fn title(&self) -> &'static str {
        "THE CHARACTERS"
    }

    fn lines(&self, _view: &View<'_>) -> Vec<Line> {
        vec![Line::note("nothing is built here yet")]
    }
}
