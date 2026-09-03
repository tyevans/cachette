//! The panel that shows what the last tick logged.
//!
//! This file is the whole panel. It states its own name, its own title and its
//! own lines, and the standard draws it.[^1]
//!
//! # References
//!
//! [^1]: The panel standard. `crates/cachette-view/src/panel/mod.rs`

use super::{Line, Panel, View};

/// The panel that shows what the last tick logged.
pub struct Events;

impl Panel for Events {
    fn name(&self) -> &'static str {
        "events"
    }

    fn title(&self) -> &'static str {
        "WHAT HAPPENED"
    }

    fn lines(&self, _view: &View<'_>) -> Vec<Line> {
        vec![Line::note("nothing is built here yet")]
    }
}
