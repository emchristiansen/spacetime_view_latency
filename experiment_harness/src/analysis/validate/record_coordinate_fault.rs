//! The typed contradiction behind a
//! [`RecordCoordinateMismatch`](super::integrity_error::IntegrityError) failure.

use crate::dataset::dose_index::DoseIndex;
use crate::manifest::run_coordinate::RunCoordinate;

/// Why a dose record's own coordinate is internally inconsistent — a semantic agreement beyond the
/// syntactic kind→body dispatch. Both modes carry the trusted [`RunCoordinate`] as the narrowest
/// semantic location plus the typed offending values, so the actual contradiction is never left to
/// prose.
#[derive(Debug)]
pub(crate) enum RecordCoordinateFault {
    /// The record kind's carried dose index (`kind_dose`) disagrees with the body coordinate's dose
    /// (`coordinate_dose`).
    KindDoseDisagreement {
        run: RunCoordinate,
        kind_dose: u64,
        coordinate_dose: u64,
    },
    /// The coordinate's recorded cumulative `logical_n` (`observed`) is off the preregistered
    /// `dose * BATCH_SIZE` formula (`expected`) for its trusted [`DoseIndex`].
    OffFormulaLogicalN {
        run: RunCoordinate,
        dose: DoseIndex,
        expected: u64,
        observed: u64,
    },
}
