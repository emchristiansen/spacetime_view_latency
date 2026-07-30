//! The emitted artifact covers all seven candidates and carries no verdict-shaped text anywhere.

use crate::indexed_sender_view_calibration_analysis::calibration_diagnostics_report::CalibrationDiagnosticsReport;
use crate::indexed_sender_view_calibration_analysis::calibration_ledger::CalibrationLedger;
use crate::indexed_sender_view_calibration_analysis::candidate_window::CandidateWindow;
use crate::indexed_sender_view_calibration_analysis::ledger_admission::LedgerAdmission;
use crate::indexed_sender_view_calibration_analysis::ledger_fixture::LedgerFixture;
use crate::indexed_sender_view_calibration_analysis::replicate_pair::ReplicatePair;
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

/// Coverage: a **byte-level tripwire** over the emitted artifact — not a proof of unrepresentability.
///
/// **What proves the ceiling and what this adds are different things, and conflating them overstates
/// both.** The proof that a verdict, recommendation, chosen `W`, threshold, or score cannot be
/// reported is the *type graph*: no type in this namespace has a field for one, so there is nothing
/// for such a value to be assigned to. That is a static property of the source, and this test does
/// not establish it.
///
/// What this adds is a tripwire on the bytes. It renders a real report and searches every key and
/// every string value in the tree for verdict-shaped words, so a field added anywhere at any depth
/// trips it. Its value is precisely that it keeps working as the type graph grows — it needs no
/// reader to re-audit every type after a change. Its limit is equally precise: it examines one
/// report built from one fixture pair, so it can only see text that this input actually produces,
/// and a verdict-shaped word it does not list would pass unnoticed. A tripwire catches the careless
/// addition; the type graph is what excludes the deliberate one.
///
/// **Scalar strings are searched, not only keys.** A key-only walk would miss the more likely
/// regression by far: not a field named `verdict`, but an existing field carrying a verdict-shaped
/// *value* — a `status: "pass"`, or an enum variant spelled `Acceptable`. Externally tagged enums
/// put their variant names in key position, but a `#[serde(untagged)]` or `to_string`-rendered one
/// would put it in value position, and the ceiling is about what the artifact says, not about where
/// in the JSON grammar it says it.
///
/// **The oracle checks itself first.** The real artifact contains no offending strings, so a walker
/// whose string branch was broken or unreachable would report "clean" exactly as convincingly as a
/// working one — the negative result would be evidence of nothing. It is therefore run against a
/// planted value nested inside an array inside an object, and must return that hit at its exact
/// path, before its silence on the report is given any weight.
///
/// It also pins that all seven candidates are present: a report that silently omitted `W = 1000`
/// would leave the rule unable to consider the candidate it most needs a complete series for.
#[test]
fn the_report_carries_no_verdict_and_every_candidate() {
    let ledger = [offset_line(0), offset_line(1)].join("\n");
    let ledger_of_fixtures = CalibrationLedger::from_complete_contents_for_tests(&ledger)
        .expect("the fixture lines decode");
    let pair =
        ReplicatePair::of(LedgerAdmission::of(ledger_of_fixtures)).expect("the fixtures pair");
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

    // The oracle is checked before it is trusted. A walker whose string branch had been dropped —
    // or never reached, since arrays and objects nest — would report "clean" on the real artifact
    // just as convincingly as a working one, because that artifact contains no offending strings at
    // all. A negative result is only evidence if the instrument can produce a positive one, so it is
    // first run against a value that must trip it, nested inside an array inside an object so the
    // traversal itself is exercised rather than only the top-level match arm.
    // Exactly one hit is expected, and the key is deliberately *not* verdict-shaped: `status`
    // contains none of `VERDICT_SHAPED`, so this isolates the value branch. If it also tripped as a
    // key, the value branch could be dead and this assertion would still pass.
    let planted = serde_json::json!({ "nested": [{ "status": "Recommended" }] });
    let mut tripped = Vec::new();
    collect_verdict_shaped_text(&planted, &mut String::new(), &mut tripped);
    assert_eq!(
        tripped,
        vec![String::from("/nested/0/status = \"Recommended\" (value)")],
        "the walker must find a verdict-shaped *value* nested inside an array, and name its exact \
         path, or its silence on the real report means nothing"
    );

    let mut offending = Vec::new();
    collect_verdict_shaped_text(&rendered, &mut String::new(), &mut offending);
    assert!(
        offending.is_empty(),
        "the report may carry no verdict, recommendation, chosen W, threshold, or score at any \
         depth, in a key or in a string value; found {offending:?}"
    );
}

/// Walk the tree, recording every verdict-shaped object key and string value with where it sits.
///
/// Arrays extend the path with the element index rather than being traversed anonymously, so a hit
/// inside the seven candidate rows names which row it came from.
fn collect_verdict_shaped_text(value: &serde_json::Value, path: &mut String, found: &mut Vec<String>) {
    match value {
        serde_json::Value::Object(fields) => {
            for (key, nested) in fields {
                if is_verdict_shaped(key) {
                    found.push(format!("{path}/{key} (key)"));
                }
                let restore = path.len();
                path.push('/');
                path.push_str(key);
                collect_verdict_shaped_text(nested, path, found);
                path.truncate(restore);
            }
        }
        serde_json::Value::Array(items) => {
            for (index, item) in items.iter().enumerate() {
                let restore = path.len();
                path.push_str(&format!("/{index}"));
                collect_verdict_shaped_text(item, path, found);
                path.truncate(restore);
            }
        }
        serde_json::Value::String(text) => {
            if is_verdict_shaped(text) {
                found.push(format!("{path} = {text:?} (value)"));
            }
        }
        // Numbers, booleans, and null carry no words to inspect.
        _ => {}
    }
}

/// Whether this text contains any verdict-shaped word, case-insensitively.
fn is_verdict_shaped(text: &str) -> bool {
    let lowered = text.to_lowercase();
    VERDICT_SHAPED.iter().any(|word| lowered.contains(word))
}

/// A valid wire line at `ordinal`, offset so the two series are not identical.
fn offset_line(ordinal: u32) -> String {
    LedgerFixture::complete(ordinal)
        .with_samples(
            (0..MAX_PACED_SAMPLES_USIZE)
                .map(|index| 1_000_000 + u128::from(ordinal) * 37 + index as u128)
                .collect(),
        )
        .line()
}
