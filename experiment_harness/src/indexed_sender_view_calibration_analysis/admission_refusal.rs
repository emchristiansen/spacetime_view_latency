//! Why a decoded ledger line is not a complete replicate.

use std::fmt;

/// The typed reason a decoded record cannot be admitted as a §569 replicate.
///
/// **Refusal is an ordinary outcome, not an error.** A ledger recording one `NotRun` and one
/// `Attempted` is a perfectly honest artifact; what it is *not* is analysable, and saying so
/// precisely is more useful than either failing the whole parse or silently analysing one series.
/// Every variant names a specific admission requirement so the refusal can be reported per record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum AdmissionRefusal {
    /// The line restates a method other than the frozen one §569's decision rule is stated over.
    MethodNotFrozen,
    /// The record shape cannot hold a series at all: the slot never ran, never provisioned, never
    /// opened its window, or never closed its bracket.
    NotAttempted,
    /// The attempt ran and failed, so it has no sealed series.
    AttemptFailed,
    /// The sealed series is not exactly the frozen sample count.
    SampleCountNotFrozen { samples: usize },
    /// A sample at this 0-based position is not strictly positive.
    NonPositiveSample { position: usize },
    /// The attempt identity is not one this freeze can contain — a foreign candidate, axis, rung,
    /// ordinal, or candidate version, or the `Control` role the calibration ceiling excludes.
    KeyNotFrozen,
    /// The arm cache was proven at a size a complete attempt cannot have had.
    ArmRowsNotFrozen { arm_rows: u64, expected: u64 },
    /// The witness cache was proven at a size a complete attempt cannot have had.
    WitnessRowsNotFrozen { witness_rows: u64, expected: u64 },
    /// The composition was verified against a different number of appends than the freeze requires.
    VerifiedAppendsNotFrozen {
        verified_appends: u64,
        expected: u64,
    },
}

impl fmt::Display for AdmissionRefusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AdmissionRefusal::MethodNotFrozen => write!(
                f,
                "this record restates a method other than the frozen one, so its series is not \
                 evidence about the method the decision rule is stated over"
            ),
            AdmissionRefusal::NotAttempted => write!(
                f,
                "this record shape cannot hold a series, so the attempt produced no replicate"
            ),
            AdmissionRefusal::AttemptFailed => write!(
                f,
                "this attempt ran and failed, so its retained samples are a rejected series rather \
                 than a replicate"
            ),
            AdmissionRefusal::SampleCountNotFrozen { samples } => write!(
                f,
                "a replicate is exactly the frozen sample count, got {samples}"
            ),
            AdmissionRefusal::NonPositiveSample { position } => write!(
                f,
                "sample {position} is not strictly positive, so this series invalidates its own \
                 attempt"
            ),
            AdmissionRefusal::KeyNotFrozen => write!(
                f,
                "this attempt identity is not one the freeze contains, so its series is not a \
                 replicate of the two originals"
            ),
            AdmissionRefusal::ArmRowsNotFrozen { arm_rows, expected } => write!(
                f,
                "a complete attempt's arm cache is proven at {expected} rows, not {arm_rows}"
            ),
            AdmissionRefusal::WitnessRowsNotFrozen {
                witness_rows,
                expected,
            } => write!(
                f,
                "a complete attempt's witness cache is proven at {expected} rows, not {witness_rows}"
            ),
            AdmissionRefusal::VerifiedAppendsNotFrozen {
                verified_appends,
                expected,
            } => write!(
                f,
                "a complete series must have been verified against all {expected} frozen appends, \
                 not {verified_appends}"
            ),
        }
    }
}
