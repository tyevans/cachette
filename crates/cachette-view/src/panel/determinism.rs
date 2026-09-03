//! The panel that shows the tick and the state hash.
//!
//! This file is the whole panel. It states its own name, its own title and its
//! own lines, and the standard draws it.[^1]
//!
//! # References
//!
//! [^1]: The panel standard. `crates/cachette-view/src/panel/mod.rs`

use super::{Line, Panel, View};

/// The panel that shows the tick and the state hash.
pub struct Determinism;

impl Panel for Determinism {
    fn name(&self) -> &'static str {
        "determinism"
    }

    fn title(&self) -> &'static str {
        "THE RUN"
    }

    fn lines(&self, _view: &View<'_>) -> Vec<Line> {
        vec![Line::note("nothing is built here yet")]
    }
}
