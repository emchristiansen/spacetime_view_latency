//! A provenance line recording a confirmed-read setting other than the campaign's is refused.

use crate::view_read_set_campaign::campaign_params::CONFIRMED_READS;

use super::fixture;

/// Coverage: this clause is unlike the other four, and the difference is worth being exact about.
/// The observed value and the campaign's are read from the same constant, so within a single process
/// they cannot disagree — no run of this build can produce the state this test constructs.
///
/// That is precisely why the clause exists. A ledger is appended to over time and read back later,
/// and a line written by a build with a different `CONFIRMED_READS` would sit alongside lines that
/// disagree with it. Confirmed reads change what a measured latency *means* — whether it includes
/// the server's commit acknowledgement — so pooling both kinds would silently mix two measurements
/// of different quantities. A ledger-only reader has no other way to detect it, which is why the
/// spec requires every provenance record to carry the setting explicitly rather than let it be
/// inferred from build provenance.
///
/// The observed value is the negation of the constant rather than a literal `true` or `false`, so
/// the test asserts a disagreement whichever way the campaign is configured.
#[test]
fn a_confirmed_reads_setting_from_another_build_is_refused() {
    let pinned = fixture::pinned_version();

    let mut observed = fixture::agreeing(&pinned);
    observed.confirmed_reads = !CONFIRMED_READS;

    let error = observed
        .agrees_with(&fixture::campaign())
        .expect_err("a confirmed-read setting other than the campaign's must be refused");
    let rendered = format!("{error:#}");
    assert!(
        rendered.contains(&format!("confirmed_reads={}", !CONFIRMED_READS)),
        "the refusal must name the observed setting, got {rendered}"
    );
    assert!(
        rendered.contains(&CONFIRMED_READS.to_string()),
        "the refusal must name the campaign's recorded setting, got {rendered}"
    );
}
