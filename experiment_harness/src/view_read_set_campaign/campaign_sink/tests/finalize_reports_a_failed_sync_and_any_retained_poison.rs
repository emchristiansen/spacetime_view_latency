//! Finalization is best-effort and hides nothing: a failed final sync is reported on its own, and
//! alongside a retained poison when the ledger was already terminal.

use crate::view_read_set_campaign::campaign_sink::CampaignSink;

use super::fixture;
use super::scripted_writer::ScriptedWriter;

/// Coverage: the two failing corners of finalization's four-arm `(poison, final sync)` match. The
/// clean corner is covered by
/// [`a_created_ledger_holds_exactly_the_lines_it_appended`](super::a_created_ledger_holds_exactly_the_lines_it_appended)
/// over the real filesystem, and the poisoned-with-clean-sync corner by
/// [`the_first_persist_failure_makes_the_ledger_terminal`](super::the_first_persist_failure_makes_the_ledger_terminal);
/// together the four are exhausted.
///
/// **Why both failures must appear, not the worse of the two.** They answer different questions. The
/// poison says a record's durability is already uncertain; the sync failure says the file metadata
/// and directory entry may not be durable either. Reporting only one would tell an operator to
/// investigate half of what is actually unknown — and this sink retains the poison as a plain string
/// rather than a typed tail state, so the text *is* the whole record of it.
///
/// **Why the final sync runs even when poisoned.** Records already written are worth preserving:
/// refusing to sync a ledger because it is terminal would discard durable evidence to punish an
/// error that already happened.
#[test]
fn finalize_reports_a_failed_sync_and_any_retained_poison() {
    // A healthy ledger whose final sync fails: the sync failure is the whole of the report.
    let mut healthy = CampaignSink::from_writer(Box::new(ScriptedWriter::finalize_failing(1)));
    healthy
        .append(fixture::inventory_record())
        .expect("the scripted writer accepts the first line");
    let sync_only = healthy
        .finalize()
        .expect_err("a failing final sync is not clean success");
    let sync_only = format!("{sync_only:#}");
    assert!(
        sync_only.contains("syncing the campaign ledger at finalize"),
        "a failed final sync must be reported, got {sync_only:?}"
    );
    assert!(
        !sync_only.contains("terminal before finalize"),
        "an unpoisoned ledger must not be described as terminal, got {sync_only:?}"
    );

    // A ledger poisoned by its very first write, whose final sync then also fails.
    let mut poisoned = CampaignSink::from_writer(Box::new(ScriptedWriter::finalize_failing(0)));
    poisoned
        .append(fixture::inventory_record())
        .expect_err("the writer seam errors on the first line");
    let both = poisoned
        .finalize()
        .expect_err("finalizing a poisoned ledger is not clean success");
    let both = format!("{both:#}");
    assert!(
        both.contains("syncing the campaign ledger at finalize"),
        "the fresh sync failure must survive the retained poison, got {both:?}"
    );
    assert!(
        both.contains("terminal before finalize"),
        "the retained poison must survive the fresh sync failure, got {both:?}"
    );
    assert!(
        both.contains("seq 0"),
        "the retained poison must still name the uncertain sequence, got {both:?}"
    );
}
