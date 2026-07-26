//! A CLI built from a release commit other than the campaign's pin is refused.

use super::fixture;

/// Coverage: the release commit is the finer-grained half of the runtime identity. Two builds can
/// report the same semantic version and be different upstream commits, so a campaign that pinned
/// only the version would admit evidence from a runtime it never preregistered. Changing only this
/// field, and leaving both versions equal to the pin, is what shows the commit is checked in its own
/// right rather than implied by the version.
#[test]
fn a_release_commit_other_than_the_pin_is_refused() {
    let pinned = fixture::pinned_version();
    let unpinned_commit = fixture::other_release_commit();

    let mut observed = fixture::agreeing(&pinned);
    observed.cli_release_commit = unpinned_commit;

    let campaign = fixture::campaign();
    let error = observed
        .agrees_with(&campaign)
        .expect_err("a release commit other than the campaign's pin must be refused");
    let rendered = format!("{error:#}");
    assert!(
        rendered.contains(&unpinned_commit.to_string())
            && rendered.contains(&campaign.expected_release_commit().to_string()),
        "the refusal must name the observed commit and the pin it disagreed with, got {rendered}"
    );
}
