//! The typed contradiction behind a [`DoseCensus`](super::integrity_error::IntegrityError) failure.

/// Why a run's dose census failed. Each run must carry doses `expected_min..=expected_max`
/// (`1..=NUM_DOSES`), each exactly once. Every mode carries typed range and occurrence-count evidence as
/// fields — never in prose — while the enclosing [`DoseCensus`](super::integrity_error::IntegrityError)
/// variant supplies the run location.
#[derive(Debug)]
pub(crate) enum DoseCensusFault {
    /// A raw wire dose is not in the valid inclusive range `expected_min..=expected_max`.
    OutOfRange {
        dose: u64,
        expected_min: u64,
        expected_max: u64,
    },
    /// A valid dose in range has no observation for this run: `expected_occurrences` is 1,
    /// `observed_occurrences` is 0.
    Missing {
        dose: u64,
        expected_occurrences: usize,
        observed_occurrences: usize,
    },
    /// A valid dose has more than one observation for this run: `expected_occurrences` is 1,
    /// `observed_occurrences` is the actual count.
    Duplicate {
        dose: u64,
        expected_occurrences: usize,
        observed_occurrences: usize,
    },
}
