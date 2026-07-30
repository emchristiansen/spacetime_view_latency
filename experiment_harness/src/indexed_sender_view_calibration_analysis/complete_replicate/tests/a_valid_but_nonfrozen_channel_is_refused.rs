//! A real measurement channel that is not the frozen one decodes cleanly and is refused by value.

use crate::indexed_sender_view_calibration_analysis::admission_refusal::AdmissionRefusal;
use crate::indexed_sender_view_calibration_analysis::calibration_ledger::CalibrationLedger;
use crate::indexed_sender_view_calibration_analysis::complete_replicate::CompleteReplicate;
use crate::indexed_sender_view_calibration_analysis::ledger_fixture::LedgerFixture;

/// Coverage: the `channel` conjunct of `MethodFactsDto::matches_frozen`, which no other test reaches.
///
/// **Why this is not covered by the unknown-spelling test.**
/// `an_unknown_variant_spelling_fails_decoding` substitutes `SomeOtherChannel`, which is not a
/// variant of the mirrored DTO at all, so that line dies at *decode* and never reaches the frozen
/// comparison. `MeasurementChannelDto` restates all four real variants precisely so an unexpected
/// channel is rejected by name rather than silently accepted — which means a line naming a different
/// *real* channel is perfectly well-typed, decodes without complaint, and is stopped only by
/// `matches_frozen` comparing the value. Without this test, deleting that conjunct outright would
/// leave the suite green.
///
/// `ColdSubscriptionApplyTime` is chosen because it is a channel this campaign genuinely runs, so
/// the line under test is not a hypothetical: it is the shape a real cold-subscription record would
/// have, and admitting it would mean analysing one channel's series as though it were another's.
///
/// The contrast with `outcome_ceiling` is the reason these two fields are covered differently.
/// `OutcomeCeilingDto` mirrors a *one-variant* enum, so there is no valid-but-nonfrozen ceiling to
/// construct — any other value is not a value, and the decode failure is the whole refusal.
#[test]
fn a_valid_but_nonfrozen_channel_is_refused() {
    let line = LedgerFixture::complete(0)
        .with_channel("ColdSubscriptionApplyTime")
        .line();
    let lines = CalibrationLedger::from_complete_contents_for_tests(&line)
        .expect("a real but nonfrozen channel is well-typed, so the line decodes rather than failing")
        .into_lines();

    assert_eq!(
        CompleteReplicate::admit(&lines[0].record).unwrap_err(),
        AdmissionRefusal::MethodNotFrozen,
        "a series measured on the cold-subscription channel is not evidence about the paced channel \
         the decision rule is stated over, however well-formed its line is"
    );
}
