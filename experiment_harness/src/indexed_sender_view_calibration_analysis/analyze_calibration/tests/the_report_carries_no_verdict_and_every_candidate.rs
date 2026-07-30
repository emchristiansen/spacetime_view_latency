//! The emitted artifact covers all seven candidates and contains no verdict-shaped key anywhere.

use crate::indexed_sender_view_calibration_analysis::candidate_window::CandidateWindow;
use crate::indexed_sender_view_calibration_analysis::ledger_fixture::LedgerFixture;
use crate::indexed_sender_view_calibration_analysis::replicate_pair::ReplicatePair;
use crate::indexed_sender_view_calibration_analysis::calibration_diagnostics_report::CalibrationDiagnosticsReport;
use crate::indexed_sender_view_calibration_analysis::complete_replicate::CompleteReplicate;
use crate::indexed_sender_view_calibration_analysis::parse_calibration_ndjson::parse_calibration_ndjson;
use crate::indexed_sender_view_calibration_pilot::calibration_params::MAX_PACED_SAMPLES_USIZE;

/// Words that would indicate this report had started answering the question rather than informing
/// it. §568 caps the pilot's output at evidence for Control to freeze `W`.
const VERDICT_SHAPED: [&str; 10] = [
    "verdict",
    "recommend",
    "chosen",
    "selected",
    "pass",
    "fail",
    "threshold",
    "acceptable",
    "score",
    "conclusion",
];

/// Coverage: the ceiling, asserted against the emitted bytes rather than against the type list.
///
/// The structural argument — that no type in this namespace has a verdict-shaped field — is only as
/// good as a reader's willingness to check every type. This walks the actual JSON instead, so a
/// verdict field added anywhere in the tree, at any depth, fails here. It is the one test that keeps
/// working when the type graph grows.
///
/// It also pins that all seven candidates are present: a report that silently omitted `W = 1000`
/// would leave the rule unable to consider the candidate it most needs a complete series for.
#[test]
fn the_report_carries_no_verdict_and_every_candidate() {
    let pair = ReplicatePair::of(vec![admitted(0), admitted(1)]).expect("the fixtures pair");
    let rendered = serde_json::to_value(CalibrationDiagnosticsReport::of(&pair))
        .expect("the report serializes");

    let candidates = rendered["candidates"]
        .as_array()
        .expect("the candidate rows are a list");
    assert_eq!(
        candidates.len(),
        CandidateWindow::ALL.len(),
        "every candidate the rule names gets a row"
    );

    let mut offending = Vec::new();
    collect_verdict_shaped_keys(&rendered, &mut String::new(), &mut offending);
    assert!(
        offending.is_empty(),
        "the report may carry no verdict, recommendation, chosen W, threshold, or score at any \
         depth; found {offending:?}"
    );
}

/// Walk every key in the tree, recording any whose name is verdict-shaped.
fn collect_verdict_shaped_keys(value: &serde_json::Value, path: &mut String, found: &mut Vec<String>) {
    match value {
        serde_json::Value::Object(fields) => {
            for (key, nested) in fields {
                let lowered = key.to_lowercase();
                if VERDICT_SHAPED.iter().any(|word| lowered.contains(word)) {
                    found.push(format!("{path}/{key}"));
                }
                let restore = path.len();
                path.push('/');
                path.push_str(key);
                collect_verdict_shaped_keys(nested, path, found);
                path.truncate(restore);
            }
        }
        serde_json::Value::Array(items) => {
            for item in items {
                collect_verdict_shaped_keys(item, path, found);
            }
        }
        _ => {}
    }
}

/// An admitted replicate at `ordinal`, offset so the two series are not identical.
fn admitted(ordinal: u32) -> CompleteReplicate {
    let line = LedgerFixture::complete(ordinal)
        .with_samples(
            (0..MAX_PACED_SAMPLES_USIZE)
                .map(|index| 1_000_000 + u128::from(ordinal) * 37 + index as u128)
                .collect(),
        )
        .line();
    let records = parse_calibration_ndjson(&line).expect("the fixture line decodes");
    CompleteReplicate::admit(&records[0]).expect("the fixture line is admissible")
}
