//! Each method fact that decodes cleanly but differs from the freeze refuses admission by itself.

use crate::indexed_sender_view_calibration_analysis::admission_refusal::AdmissionRefusal;
use crate::indexed_sender_view_calibration_analysis::calibration_ledger::CalibrationLedger;
use crate::indexed_sender_view_calibration_analysis::complete_replicate::CompleteReplicate;
use crate::indexed_sender_view_calibration_analysis::ledger_fixture::LedgerFixture;
use crate::indexed_sender_view_calibration_pilot::calibration_params::{
    MAX_PACED_SAMPLES, SUBSCRIBER_OWN_ROWS, WITH_CONFIRMED_READS,
};
use crate::view_read_set_campaign::campaign_params::PACED_SAMPLE_DELAY_MS;

/// Coverage: the four method facts whose wrong values are **type-correct**, so nothing but the
/// frozen-value comparison can catch them.
///
/// The method block's other two fields are mirrored enums, and they are covered separately because
/// they fail in two different ways. `OutcomeCeilingDto` has a single variant, so a wrong value there
/// is not a value at all and dies at decode — covered by `an_unknown_variant_spelling_fails_decoding`.
/// `MeasurementChannelDto` restates all four real channels, so a *different real channel* is a
/// perfectly valid value that decodes cleanly and is caught only by the frozen comparison — covered
/// by `a_valid_but_nonfrozen_channel_is_refused`. These four are a `u32`, two `u64`s, and a `bool`.
/// Any number is a valid number and either boolean is a valid boolean, so the *only* thing standing
/// between this analyzer and a series measured under a different method is
/// `MethodFactsDto::matches_frozen` actually comparing each one.
///
/// **Each is varied alone, and each must refuse on its own.** A test that changed all four at once
/// would pass while three of the four comparisons were missing entirely — the refusal would be
/// attributable to whichever one still worked. Varying one at a time is what makes this evidence
/// about four separate conjuncts rather than about their disjunction.
///
/// The series itself stays frozen throughout, so nothing here can be refused for its samples. In
/// particular the declared `sample_count` is varied *without* touching the series: a line claiming a
/// 500-sample method while carrying 1,000 samples is refused as `MethodNotFrozen`, not as
/// `SampleCountNotFrozen`, because the method is checked first and the method is what is wrong. A
/// line whose two claims disagree is not a line whose series can be trusted.
#[test]
fn a_valid_but_nonfrozen_method_fact_is_refused() {
    let cases = [
        (
            "sample_count",
            LedgerFixture::complete(0).with_sample_count(MAX_PACED_SAMPLES / 2),
        ),
        (
            "paced_sample_delay_ms",
            LedgerFixture::complete(0).with_paced_delay_ms(PACED_SAMPLE_DELAY_MS + 1),
        ),
        (
            "seeded_own_rows",
            LedgerFixture::complete(0).with_seeded_own_rows(SUBSCRIBER_OWN_ROWS + 1),
        ),
        (
            "with_confirmed_reads",
            LedgerFixture::complete(0).with_confirmed_reads(!WITH_CONFIRMED_READS),
        ),
    ];

    for (fact, fixture) in cases {
        let line = fixture.line();
        let lines = CalibrationLedger::from_complete_contents_for_tests(&line)
            .expect("a wrong-but-well-typed method fact still decodes; it is refused, not malformed")
            .into_lines();
        assert_eq!(
            CompleteReplicate::admit(&lines[0].record).unwrap_err(),
            AdmissionRefusal::MethodNotFrozen,
            "a line whose {fact} differs from the freeze restates a method the decision rule is not \
             stated over, so its series is not evidence about that method"
        );
    }

    // The unvaried fixture admits, so every refusal above is attributable to the single fact it
    // varied rather than to anything the fixture does by default.
    let line = LedgerFixture::complete(0).line();
    let lines = CalibrationLedger::from_complete_contents_for_tests(&line)
        .expect("the fixture line decodes")
        .into_lines();
    CompleteReplicate::admit(&lines[0].record)
        .expect("the untouched fixture restates the frozen method exactly");
}
