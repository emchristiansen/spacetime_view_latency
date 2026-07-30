//! An admitted §569 replicate: one complete, positive, fully-verified series.

use crate::indexed_sender_view_calibration_analysis::admission_refusal::AdmissionRefusal;
use crate::indexed_sender_view_calibration_analysis::attempted_outcome_dto::AttemptedOutcomeDto;
use crate::indexed_sender_view_calibration_analysis::calibration_record_dto::CalibrationRecordDto;
use crate::indexed_sender_view_calibration_analysis::frozen_population::FrozenPopulation;
use crate::indexed_sender_view_calibration_pilot::calibration_params::MAX_PACED_SAMPLES_USIZE;

/// One ledger line admitted as a §569 replicate.
///
/// **A capability token, mirroring the pilot's own
/// [`VerifiedPopulation`](crate::indexed_sender_view_calibration_pilot::verified_population::VerifiedPopulation)
/// discipline.** The fields are private to this module and [`Self::admit`] is the only constructor,
/// so every value in existence has passed all five admission requirements. Nothing downstream —
/// window medians, trend, lag, between-replicate disagreement — takes a bare `Vec<u128>`, so a series
/// that is short, non-positive, or verified against the wrong append count cannot reach a diagnostic
/// at all.
///
/// That the pilot's own token does not survive serialization is exactly why this exists: the wire
/// form is an ordinary struct, so admission is *re-derived* here rather than inherited.
#[derive(Debug, Clone)]
pub(crate) struct CompleteReplicate {
    replicate: u32,
    samples: Vec<u128>,
}

impl CompleteReplicate {
    /// Admit one decoded record as a replicate, or say precisely why it is not one.
    ///
    /// **The pilot's capability token did not survive serialization, so the whole predicate is
    /// reconstructed here — not a subset of it.** `VerifiedPopulation` is mintable only by the
    /// row-by-row verifier, but what reaches the ledger is three plain integers, and an admission
    /// check that read only `verified_appends` would re-open exactly the same-cardinality
    /// substitution class that token exists to exclude. Every serialized fact a complete series
    /// depends on is therefore checked, in the order that makes the refusal most specific:
    ///
    /// 1. the line restates the frozen method ([`MethodFactsDto::matches_frozen`]);
    /// 2. the attempt identity is one the freeze contains ([`AttemptKeyDto::matches_frozen`]) —
    ///    candidate, axis, rung, `Arm` role, `Original` ordinal, and the frozen candidate version;
    /// 3. the record is an `Attempted` shape, the only one that can hold a series;
    /// 4. its outcome is `CalibrationRecorded` rather than `Failed`;
    /// 5. its series is exactly the frozen count, with every sample strictly positive;
    /// 6. its composition proof states all three frozen counts — arm rows, witness rows, and
    ///    verified appends ([`FrozenPopulation::complete`]).
    ///
    /// Restated, not trusted. A ledger line is untrusted text, and that some process once passed
    /// these checks before writing it is not evidence available to this reader.
    ///
    /// [`MethodFactsDto::matches_frozen`]: super::method_facts_dto::MethodFactsDto::matches_frozen
    /// [`AttemptKeyDto::matches_frozen`]: super::attempt_key_dto::AttemptKeyDto::matches_frozen
    /// [`FrozenPopulation::complete`]: super::frozen_population::FrozenPopulation::complete
    pub(crate) fn admit(record: &CalibrationRecordDto) -> Result<Self, AdmissionRefusal> {
        if !record.method().matches_frozen() {
            return Err(AdmissionRefusal::MethodNotFrozen);
        }
        if !record.key().matches_frozen() {
            return Err(AdmissionRefusal::KeyNotFrozen);
        }

        let CalibrationRecordDto::Attempted { outcome, .. } = record else {
            return Err(AdmissionRefusal::NotAttempted);
        };
        let AttemptedOutcomeDto::CalibrationRecorded { series } = outcome else {
            return Err(AdmissionRefusal::AttemptFailed);
        };

        if series.samples.len() != MAX_PACED_SAMPLES_USIZE {
            return Err(AdmissionRefusal::SampleCountNotFrozen {
                samples: series.samples.len(),
            });
        }
        // The samples are unsigned, so "not strictly positive" is exactly zero.
        if let Some(position) = series.samples.iter().position(|sample| *sample == 0) {
            return Err(AdmissionRefusal::NonPositiveSample { position });
        }

        let frozen = FrozenPopulation::complete();
        let population = series.population;
        if population.arm_rows != frozen.arm_rows {
            return Err(AdmissionRefusal::ArmRowsNotFrozen {
                arm_rows: population.arm_rows,
                expected: frozen.arm_rows,
            });
        }
        if population.witness_rows != frozen.witness_rows {
            return Err(AdmissionRefusal::WitnessRowsNotFrozen {
                witness_rows: population.witness_rows,
                expected: frozen.witness_rows,
            });
        }
        if population.verified_appends != frozen.verified_appends {
            return Err(AdmissionRefusal::VerifiedAppendsNotFrozen {
                verified_appends: population.verified_appends,
                expected: frozen.verified_appends,
            });
        }

        Ok(Self {
            replicate: record.key().replicate(),
            samples: series.samples.clone(),
        })
    }

    /// Which replicate this is — the only coordinate that varies across the freeze.
    pub(crate) fn replicate(&self) -> u32 {
        self.replicate
    }

    /// The complete ordered series, in issue order.
    ///
    /// Order is preserved and never sorted here: every §569 diagnostic is a statement about position.
    pub(crate) fn samples(&self) -> &[u128] {
        &self.samples
    }
}

#[cfg(test)]
mod tests;
