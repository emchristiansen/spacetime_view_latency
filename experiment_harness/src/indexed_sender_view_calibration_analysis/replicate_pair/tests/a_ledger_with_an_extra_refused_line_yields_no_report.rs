//! Two perfect originals plus one refused line is not the frozen inventory, so no `W` is evaluated.

use crate::indexed_sender_view_calibration_analysis::admission_refusal::AdmissionRefusal;
use crate::indexed_sender_view_calibration_analysis::calibration_ledger::CalibrationLedger;
use crate::indexed_sender_view_calibration_analysis::ledger_admission::LedgerAdmission;
use crate::indexed_sender_view_calibration_analysis::ledger_fixture::LedgerFixture;
use crate::indexed_sender_view_calibration_analysis::pair_refusal::PairRefusal;
use crate::indexed_sender_view_calibration_analysis::refused_line::RefusedLine;
use crate::indexed_sender_view_calibration_analysis::replicate_pair::ReplicatePair;

/// Coverage: the drop an earlier draft made, closed structurally.
///
/// That draft admitted every line, kept the refusals in a local vector, and then passed only the
/// admitted replicates to pairing — so the refusals were forwarded *only* when pairing failed. A
/// ledger holding the two originals plus a third line therefore produced a full report, with nothing
/// anywhere in it recording that the third line existed.
///
/// **The frozen inventory is exactly two original records.** A file with a third is not that
/// inventory, whatever its other two lines contain, and the third may be the most informative thing
/// in it: a failed attempt says something about the method that two successes do not. Analysing the
/// good two would state a §569 pair drawn from an artifact whose extra content nobody looked at.
///
/// Two shapes are covered because they fail for different reasons. A **failed attempt** is a
/// well-formed record of the frozen method that simply has no series; a **`Control` line** is a
/// record whose identity the freeze does not contain at all. Both leave the ledger unequal to the
/// inventory, and both must surface with their position and reason rather than being counted as
/// absent.
#[test]
fn a_ledger_with_an_extra_refused_line_yields_no_report() {
    let failed_third = refusal(&[
        LedgerFixture::complete(0).line(),
        LedgerFixture::complete(1).line(),
        LedgerFixture::complete(1).failed_line(),
    ]);
    assert_eq!(
        failed_third,
        PairRefusal::LedgerHasRefusedLines {
            refused: vec![RefusedLine {
                line_number: 3,
                reason: AdmissionRefusal::AttemptFailed,
            }],
        },
        "the two originals are individually perfect, yet the ledger is not the frozen inventory; the \
         refused line is named with its position rather than dropped"
    );

    let control_third = refusal(&[
        LedgerFixture::complete(0).line(),
        LedgerFixture::complete(1).line(),
        LedgerFixture::complete(0).with_role("Control").line(),
    ]);
    assert_eq!(
        control_third,
        PairRefusal::LedgerHasRefusedLines {
            refused: vec![RefusedLine {
                line_number: 3,
                reason: AdmissionRefusal::KeyNotFrozen,
            }],
        },
        "a line whose identity the freeze does not contain is reported as such, not silently ignored \
         because two other lines happened to pair"
    );

    let untampered = [
        LedgerFixture::complete(0).line(),
        LedgerFixture::complete(1).line(),
    ];
    let ledger_of_fixtures =
        CalibrationLedger::from_complete_contents_for_tests(&untampered.join("\n"))
            .expect("the lines decode");
    ReplicatePair::of(LedgerAdmission::of(ledger_of_fixtures))
        .expect("exactly the two originals pair, so the refusals above are about the extra line");
}

/// Decode `lines` as one ledger, admit it whole, and return the pairing refusal it must produce.
fn refusal(lines: &[String]) -> PairRefusal {
    let ledger_of_fixtures = CalibrationLedger::from_complete_contents_for_tests(&lines.join("\n"))
        .expect("the fixture lines decode");
    ReplicatePair::of(LedgerAdmission::of(ledger_of_fixtures))
        .expect_err("a ledger that is not exactly the frozen inventory produces no pair")
}
