//! An attempt that published a module other than the campaign's pinned artifact is refused.

use super::fixture;

/// Coverage: this is the clause that ties measured latency to the code that produced it. The pinned
/// digest is the same constant the harness verifies the built WASM against before publishing, so an
/// attempt whose published digest differs measured a different module — different reducers, possibly
/// a different view body — and pooling its evidence with the rest would compare two candidates as
/// though they were one.
///
/// The observed digest differs from the pin by a single flipped bit, which is the honest shape of
/// the risk: the failure this catches is a *wrong artifact*, not a corrupt one, and a
/// near-identical digest is exactly what a wrong-but-similar build produces.
#[test]
fn a_module_digest_other_than_the_pin_is_refused() {
    let pinned = fixture::pinned_version();
    let unpinned_digest = fixture::other_module_digest();

    let mut observed = fixture::agreeing(&pinned);
    observed.module_wasm_sha256 = unpinned_digest;

    let campaign = fixture::campaign();
    let error = observed
        .agrees_with(&campaign)
        .expect_err("a published module digest other than the campaign's pin must be refused");
    let rendered = format!("{error:#}");
    assert!(
        rendered.contains(&unpinned_digest.canonical_hex())
            && rendered.contains(&campaign.expected_module_wasm_sha256().canonical_hex()),
        "the refusal must name the observed digest and the pin it disagreed with, got {rendered}"
    );
}
