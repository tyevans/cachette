//! The panel that shows the population, the holdings and the stores.
//!
//! This file is the whole panel. It states its own name, its own title and its
//! own lines, and the standard draws it.[^1]
//!
//! # References
//!
//! [^1]: The panel standard. `crates/cachette-view/src/panel/mod.rs`

use super::{Line, Panel, View};

/// The panel that shows the population, the holdings and the stores.
pub struct Statistics;

impl Panel for Statistics {
    fn name(&self) -> &'static str {
        "statistics"
    }

    fn title(&self) -> &'static str {
        "STATISTICS"
    }

    fn lines(&self, _view: &View<'_>) -> Vec<Line> {
        vec![Line::note("nothing is built here yet")]
    }
}
