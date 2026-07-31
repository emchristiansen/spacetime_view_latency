//! An unrecognised spelling of a one-variant key component fails to decode, loudly.

use super::super::decode_complete_contents;
use crate::indexed_sender_view_calibration_analysis::ledger_fixture::LedgerFixture;

/// Coverage: the syntactic half of the key boundary, for every component whose source enum has one
/// variant.
///
/// The **semantic** half — a valid-but-wrong `Control` role, or a foreign candidate version — is
/// covered by `a_forged_key_component_is_refused`, which expects `KeyNotFrozen` at admission. This
/// test covers the other half: a spelling that is not a variant of the mirrored enum at all must
/// fail at *decoding*, before admission ever runs.
///
/// The two must not be confused. If these components were retained as opaque JSON — as an earlier
/// draft had them — an unknown spelling would decode happily and then be admitted, since nothing
/// would ever look at it. Failing here is what proves the mirrored enums are load-bearing rather
/// than decorative, and the diagnostic naming the offending token is what makes the disagreement
/// actionable instead of a bare "malformed line".
#[test]
fn an_unknown_variant_spelling_fails_decoding() {
    // Each case is (text to find in the rendered line, its replacement, the token the diagnostic
    // must name). The find text is not always the bare variant: `Calibration` is a substring of
    // `MethodCalibrationOnly`, so replacing it bare would silently corrupt the outcome ceiling as
    // well and the case would then pass for the wrong reason. Matching the tag with its `":` suffix
    // pins it to the externally-tagged `StageRepetition` position.
    let cases = [
        (
            "IndexedControlActivitySenderView",
            "SomeOtherCandidate",
            "SomeOtherCandidate",
        ),
        ("UnrelatedGlobalRows", "SomeOtherAxis", "SomeOtherAxis"),
        ("Baseline", "SomeOtherRung", "SomeOtherRung"),
        ("Original", "Superseded", "Superseded"),
        (
            "MethodCalibrationOnly",
            "PerformanceConclusion",
            "PerformanceConclusion",
        ),
        (
            "PacedVisibleApplyLatency",
            "SomeOtherChannel",
            "SomeOtherChannel",
        ),
        ("\"Calibration\":", "\"SomeOtherStage\":", "SomeOtherStage"),
    ];

    for (frozen, replacement, forged) in cases {
        let line = LedgerFixture::complete(0).line().replace(frozen, replacement);
        assert_ne!(
            line,
            LedgerFixture::complete(0).line(),
            "the case for {frozen} must actually change the rendered line, or it proves nothing"
        );
        // `expect_err` takes a plain message, so the component name is interpolated here rather than
        // left as a brace placeholder that would print literally on failure.
        let error = decode_complete_contents(&line).expect_err(&format!(
            "an unrecognised spelling of {frozen} must fail the line rather than decode into \
             something this analyzer then treats as frozen"
        ));
        assert_eq!(error.line_number(), 1, "the sole line is the offending one");
        assert!(
            error.diagnostic().contains(forged),
            "the diagnostic names the unrecognised token {forged} so the disagreement is \
             actionable; got {}",
            error.diagnostic()
        );
    }

    decode_complete_contents(&LedgerFixture::complete(0).line())
        .expect("the untampered fixture decodes, so the refusals above are about the tampering");
}
