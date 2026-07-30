//! A zero sample invalidates its own attempt, wherever in the series it sits.

use crate::indexed_sender_view_calibration_analysis::admission_refusal::AdmissionRefusal;
use crate::indexed_sender_view_calibration_analysis::calibration_ledger::CalibrationLedger;
use crate::indexed_sender_view_calibration_analysis::complete_replicate::CompleteReplicate;
use crate::indexed_sender_view_calibration_analysis::ledger_fixture::LedgerFixture;
use crate::indexed_sender_view_calibration_pilot::calibration_params::MAX_PACED_SAMPLES_USIZE;

/// Coverage: positivity at the first, a middle, and the last position.
///
/// Three positions rather than one, because a scan that stopped early, started late, or short-cut on
/// the first element would still pass a single-position test. The spec invalidates a cell on a
/// nonpositive statistic rather than counting it as flat, so the refusal names the offending
/// position — a reader must be able to see *where* the series went wrong without re-scanning it.
#[test]
fn a_nonpositive_sample_refuses_admission() {
    for position in [0, MAX_PACED_SAMPLES_USIZE / 2, MAX_PACED_SAMPLES_USIZE - 1] {
        let mut samples: Vec<u128> = (0..MAX_PACED_SAMPLES_USIZE)
            .map(|index| 1_000_000 + index as u128)
            .collect();
        samples[position] = 0;

        let line = LedgerFixture::complete(0).with_samples(samples).line();
        let lines = CalibrationLedger::from_complete_contents_for_tests(&line)
            .expect("the fixture line decodes")
            .into_lines();

        assert_eq!(
            CompleteReplicate::admit(&lines[0].record).unwrap_err(),
            AdmissionRefusal::NonPositiveSample { position },
            "a zero sample at position {position} invalidates the attempt, and the refusal says where"
        );
    }
}
