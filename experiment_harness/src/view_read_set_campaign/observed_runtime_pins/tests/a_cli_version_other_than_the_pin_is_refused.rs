//! A CLI reporting a version other than the campaign's pin is refused.

use super::fixture;

/// Coverage: the CLI is what publishes the module, so an attempt run against an unpinned CLI
/// measured a different toolchain than the campaign claims. Only this field is changed, so the
/// refusal cannot be coming from any other clause.
///
/// The rendered error is asserted to name both sides. A gate that fails the whole reconciliation
/// leaves a reader one message to work from, and "the pins disagreed" without the two values would
/// not say which build to look for.
#[test]
fn a_cli_version_other_than_the_pin_is_refused() {
    let pinned = fixture::pinned_version();
    let unpinned = fixture::other_version();

    let mut observed = fixture::agreeing(&pinned);
    observed.cli_version = &unpinned;

    let error = observed
        .agrees_with(&fixture::campaign())
        .expect_err("a cli version other than the campaign's pin must be refused");
    let rendered = format!("{error:#}");
    assert!(
        rendered.contains(&unpinned.to_string()) && rendered.contains(&pinned.to_string()),
        "the refusal must name the observed version and the pin it disagreed with, got {rendered}"
    );
    assert!(
        rendered.contains("cli"),
        "the refusal must say which binary reported it, got {rendered}"
    );
}
