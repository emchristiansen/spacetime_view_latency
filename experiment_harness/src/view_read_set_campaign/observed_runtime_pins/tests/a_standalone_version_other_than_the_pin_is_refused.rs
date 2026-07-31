//! A standalone server reporting a version other than the campaign's pin is refused.

use super::fixture;

/// Coverage: the standalone binary is the one that actually *served* the measured reads, so it is
/// checked separately from the CLI rather than assumed to match it — a distribution can in principle
/// resolve two binaries whose reported versions differ, and that attempt measured a runtime the
/// campaign did not pin.
///
/// This test is what makes the second version clause load-bearing. Both fields compare against the
/// same `expected_version`, so a comparison that checked only the CLI would pass every other test in
/// this tree, including the matching case.
///
/// It does not, however, catch a *projection* that wrote the CLI's version into both fields — the
/// two are the same Rust type, and this test builds the bundle directly. That residue is accepted on
/// source inspection of the five-line projection in `AttemptProvenance::agrees_with`.
#[test]
fn a_standalone_version_other_than_the_pin_is_refused() {
    let pinned = fixture::pinned_version();
    let unpinned = fixture::other_version();

    let mut observed = fixture::agreeing(&pinned);
    observed.standalone_version = &unpinned;

    let error = observed
        .agrees_with(&fixture::campaign())
        .expect_err("a standalone version other than the campaign's pin must be refused");
    let rendered = format!("{error:#}");
    assert!(
        rendered.contains(&unpinned.to_string()) && rendered.contains(&pinned.to_string()),
        "the refusal must name the observed version and the pin it disagreed with, got {rendered}"
    );
    assert!(
        rendered.contains("standalone"),
        "the refusal must say which binary reported it, got {rendered}"
    );
}
