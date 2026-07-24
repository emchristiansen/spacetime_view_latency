//! The typed contradiction behind a [`CellCensus`](super::integrity_error::IntegrityError) failure.

use crate::plan::cell::Cell;

/// Why the manifest cell census failed. The trusted campaign must carry exactly the nine preregistered
/// cells ([`Cell::all`]); this locates the exact contradiction with typed set/count evidence rather than
/// prose.
///
/// There is deliberately no "extra/unexpected cell" mode. A wire cell is decoded through the closed
/// [`CellDto`](crate::analysis::ingest::cell_dto::CellDto), whose variant set is the exact mirror of
/// [`Cell`], so every observed cell is already a member of [`Cell::all`]. A census failure is therefore
/// always either a *missing* cell (the observed distinct set is a strict subset of the canonical
/// expected set) or a static `CELL_COUNT` drift between the trusted graph's compile-time cell array and
/// the canonical set — never a cell outside [`Cell::all`].
#[derive(Debug)]
pub(crate) enum CellCensusFault {
    /// The compile-time cell-array length `CELL_COUNT` does not equal the canonical [`Cell::all`]
    /// length — a static configuration drift between the trusted graph's array size and the
    /// preregistered set, independent of any wire input.
    CountDrift {
        cell_count: usize,
        canonical_count: usize,
    },
    /// The observed distinct cells (canonical [`Cell::all`] order) are not the full canonical expected
    /// set (canonical order). The missing cells are `expected \ observed`; extras are unrepresentable
    /// after closed-DTO decoding, so `observed` is always a strict subset of `expected`.
    DistinctSet {
        expected: Vec<Cell>,
        observed: Vec<Cell>,
    },
}
