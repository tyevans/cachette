//! The register of every event log, and one way to read any of them.
//!
//! The engine holds one append-only array for each event type.[^1] Before
//! this module the binding gave one method for each of those arrays, each
//! with its own name and its own signature. A log added to the engine reached
//! the control plane only when somebody remembered to write a thirteenth
//! method, a thirteenth stub entry and a thirteenth line of documentation.
//! Nothing failed when they forgot.[^2]
//!
//! This module gives the field list one home at the boundary as well. A
//! caller asks what logs exist and reads any one of them by name. The name of
//! a log is the name the event declares, so this module names no event of its
//! own and the two lists cannot drift.[^3]
//!
//! The register below is the one place that says which logs cross. A test
//! compares it against the declared layouts and fails when a declared event
//! has no log here, or a log here names an event the engine does not
//! declare.[^2]
//!
//! # References
//!
//! [^1]: ADR-0031, events live in type-segregated arenas of plain data. `docs/adrs/REGISTRY.md`
//! [^2]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
//! [^3]: ADR-0163, an event declares its layout once and the binding derives every column. `docs/adrs/draft/adr-0163-an-event-declares-its-layout-once-and-the-binding-derives-every-column.md`

use cachette_core::campaign::CampaignEvent;
use cachette_core::cohort::{SiteRationed, UnitStarved};
use cachette_core::contest::UnitFell;
use cachette_core::conversion::UnitConverted;
use cachette_core::event::{
    FactionEliminated, ResourceTaken, SettlementFounded, SiteTaken, TileChanged, UpgradeCollapsed,
    UpgradeFinished,
};
use cachette_core::event_layout::EventLayout;
use cachette_core::hex::Axial;
use cachette_core::promotion::UnitPromoted;
use cachette_core::rates::SiteShortfall;
use cachette_core::relation::RelationCrossed;
use cachette_core::trade::TradeSpoken;
use cachette_core::{TileIdx, World as CoreWorld};
use numpy::ToPyArray;
use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::columns::columns_of;

/// Reads one log of a world into one array for each column it declares.
pub type LogReader = for<'py> fn(Python<'py>, &CoreWorld) -> PyResult<Bound<'py, PyDict>>;

/// Counts the records one log of a world holds.
pub type LogCounter = fn(&CoreWorld) -> usize;

/// One log that crosses to the control plane.
pub struct LogEntry {
    /// The name of the log. It is the name the event declares.
    pub name: &'static str,
    /// How a caller reads the log.
    pub read: LogReader,
    /// How a caller counts the log.
    pub count: LogCounter,
}

/// Declares the register of logs. Give the reader of the world, the event
/// type it holds, and an optional reader that adds a derived column.
macro_rules! log_registry {
    (
        $( $accessor:ident : $event:ty $( , $custom:expr )? );* $(;)?
    ) => {
        /// Every log the boundary offers, in the order it declares them.
        pub const LOGS: &[LogEntry] = &[
            $(
                LogEntry {
                    name: <$event as EventLayout>::EVENT_NAME,
                    read: log_reader!($accessor $(, $custom)?),
                    count: |world| world.$accessor().len(),
                },
            )*
        ];
    };
}

/// Gives the reader of one declared log. Not called directly.
macro_rules! log_reader {
    ($accessor:ident) => {
        |python, world| columns_of(python, world.$accessor())
    };
    ($accessor:ident, $custom:expr) => {
        $custom
    };
}

log_registry! {
    event_log: TileChanged;
    gather_log: ResourceTaken;
    fell_log: UnitFell;
    starved_log: UnitStarved;
    shortfall_log: SiteShortfall;
    rationed_log: SiteRationed;
    promoted_log: UnitPromoted;
    relation_log: RelationCrossed;
    converted_log: UnitConverted;
    trade_log: TradeSpoken;
    campaign_log: CampaignEvent, campaign_columns;
    collapsed_log: UpgradeCollapsed;
    finished_log: UpgradeFinished;
    founded_log: SettlementFounded;
    taken_log: SiteTaken;
    eliminated_log: FactionEliminated;
}

/// Reads the campaign log, and adds the axial address of the objective.
///
/// The event holds the objective as a row-major tile index and no address.
/// The grid turns one into the other, and a caller that held its own grid
/// arithmetic would hold a second statement of the world shape.[^1]
///
/// # References
///
/// [^1]: ADR-0017, the world is a rhombus, so a tile index is raw axial, decision D1. `docs/adrs/accepted/adr-0017-the-world-is-a-rhombus-so-a-tile-index-is-raw-axial.md`
fn campaign_columns<'py>(python: Python<'py>, world: &CoreWorld) -> PyResult<Bound<'py, PyDict>> {
    let log = world.campaign_log();
    let grid = world.grid();
    let address = |tile: u32| grid.address_of(TileIdx(tile)).unwrap_or(Axial::new(0, 0));
    let columns = columns_of(python, log)?;
    let q: Vec<i32> = log
        .iter()
        .map(|event| address(event.objective_tile).q)
        .collect();
    let r: Vec<i32> = log
        .iter()
        .map(|event| address(event.objective_tile).r)
        .collect();
    columns.set_item("objective_q", q.to_pyarray(python))?;
    columns.set_item("objective_r", r.to_pyarray(python))?;
    Ok(columns)
}

/// Returns the name of every log the boundary offers.
#[must_use]
pub fn log_names() -> Vec<&'static str> {
    LOGS.iter().map(|entry| entry.name).collect()
}

/// Returns the log of the given name, or `None` when no log has that name.
#[must_use]
pub fn log_of(name: &str) -> Option<&'static LogEntry> {
    LOGS.iter().find(|entry| entry.name == name)
}

/// Returns the message a caller gets when it names a log the engine has not.
///
/// The message lists every name, so a caller that mistyped one sees the set
/// it could have meant without opening another document.
#[must_use]
pub fn unknown_log_message(name: &str) -> String {
    format!(
        "no log is named `{name}`; the logs are {}",
        log_names().join(", ")
    )
}
