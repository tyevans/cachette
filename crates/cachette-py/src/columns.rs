//! One column for each field of an event, built from the declared layout.
//!
//! The engine declares the fields of an event in one place. That declaration
//! gives the name of each field, where it starts, how wide it is, and how a
//! reader reads it as a number.[^1] This module walks the declaration and
//! builds one array for each field that crosses.
//!
//! No function here names a field of an event. A field added to an event
//! reaches the control plane without an edit to this crate, and a field
//! renamed there renames the column.[^1]
//!
//! Every element type is an integer. A fixed-point field crosses as its raw
//! integer, because a float in an interface is the same defect as a float in
//! the state, one layer further out.[^2]
//!
//! # References
//!
//! [^1]: ADR-0163, an event declares its layout once and the binding derives every column. `docs/adrs/draft/adr-0163-an-event-declares-its-layout-once-and-the-binding-derives-every-column.md`
//! [^2]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`

use cachette_core::event_layout::{
    column_i32, column_i64, column_u16, column_u32, column_u64, column_u8, ColumnKind, EventLayout,
};
use numpy::ToPyArray;
use pyo3::prelude::*;
use pyo3::types::PyDict;

/// Builds one array for each field of the log that crosses to the reader.
///
/// The keys are the column names that the event declares. Every array has
/// one entry for each record, and all of them are the same length. A padding
/// field crosses nowhere and gets no key.
///
/// This function copies each column. The log of one step is small next to the
/// world.[^1]
///
/// # Errors
///
/// Returns an error when the interpreter refuses to hold the dictionary.
///
/// # References
///
/// [^1]: ADR-0044, what copies and what does not is declared at the call site. `docs/adrs/REGISTRY.md`
pub fn columns_of<'py, T: EventLayout>(
    python: Python<'py>,
    log: &[T],
) -> PyResult<Bound<'py, PyDict>> {
    let columns = PyDict::new(python);
    for field in T::FIELDS {
        let (Some(name), Some(kind)) = (field.column, field.kind) else {
            continue;
        };
        match kind {
            ColumnKind::U8 => columns.set_item(name, column_u8(log, field).to_pyarray(python))?,
            ColumnKind::U16 => columns.set_item(name, column_u16(log, field).to_pyarray(python))?,
            ColumnKind::U32 => columns.set_item(name, column_u32(log, field).to_pyarray(python))?,
            ColumnKind::U64 => columns.set_item(name, column_u64(log, field).to_pyarray(python))?,
            ColumnKind::I32 => columns.set_item(name, column_i32(log, field).to_pyarray(python))?,
            ColumnKind::I64 => columns.set_item(name, column_i64(log, field).to_pyarray(python))?,
        }
    }
    Ok(columns)
}
