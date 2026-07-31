//! The first persist failure is terminal: it names the record it lost, and every later append is
//! refused with that original reason rather than attempted.

use crate::view_read_set_campaign::campaign_sink::CampaignSink;

use super::fixture;
use super::scripted_writer::ScriptedWriter;

/// Coverage: the whole poison transition, which is the discipline the sink exists for. The scripted
/// writer performs no I/O — it injects an error from the post-serialization seam, standing in for a
/// write, flush, or `sync_data` failure, any of which can leave a full line durable, which is why
/// the sink treats the record's durability as ambiguous rather than absent.
///
/// **What the retained diagnostic must contain, and why each part matters.** The sequence, because
/// an operator reconciling a truncated ledger needs to know which position is uncertain; and the
/// record's *variant name*, because on a serialization failure the body cannot be rendered, so the
/// discriminant is the only thing that can name the lost record at all. The by-value record has
/// already moved into the line by then — reading the name before the move is what makes this
/// assertion satisfiable.
///
/// **Which of the two poisoning entry points this reaches.** The seam one, and only that one.
/// `append` also poisons when `serde_json` fails, and no record this tree can construct does that;
/// see [`super`] for why that arm stays direct-inspection-only rather than being reached with a
/// seam or a weakened record type. So this proves the transition, the refusal, and the retained
/// diagnostic's shape for the persist failure — not that both entry points were observed.
///
/// **Why the refusal is asserted to be a refusal and not a write.** It carries the original failure's
/// text and says nothing about its own record: no successor reuses the uncertain sequence, and none
/// appends a well-formed-looking line onto a torn tail. That is also what makes the failure
/// *terminal* rather than merely reported — there is no second attempt at any sequence.
///
/// Finalization then reports the retained poison even though the final sync itself succeeded, which
/// is the `(poisoned, clean sync)` corner of the four-arm finalize.
#[test]
fn the_first_persist_failure_makes_the_ledger_terminal() {
    // The writer accepts one line and errors on every one after it.
    let mut sink = CampaignSink::from_writer(Box::new(ScriptedWriter::ok_for(1)));

    let first = sink
        .append(fixture::inventory_record())
        .expect("the scripted writer accepts the first line");
    assert_eq!(first.get(), 0, "the first record takes the zeroth sequence");

    let failed = sink
        .append(fixture::terminal_record())
        .expect_err("the writer seam errors on the second line");
    let failed = format!("{failed:#}");
    assert!(
        failed.contains("seq 1"),
        "the retained diagnostic must name the uncertain sequence, got {failed:?}"
    );
    assert!(
        failed.contains("Terminal"),
        "the retained diagnostic must name the lost record's variant, got {failed:?}"
    );

    let refused = sink
        .append(fixture::supersession_record())
        .expect_err("a poisoned ledger refuses further writes");
    let refused = format!("{refused:#}");
    assert!(
        refused.contains(&failed),
        "a refusal must re-report the original failure, got {refused:?}"
    );
    assert!(
        !refused.contains("Supersession"),
        "a refusal wrote nothing, so it must not name a record of its own: {refused:?}"
    );

    let finalized = sink
        .finalize()
        .expect_err("finalizing a poisoned ledger is not clean success");
    let finalized = format!("{finalized:#}");
    assert!(
        finalized.contains("seq 1"),
        "finalization must surface the retained poison, got {finalized:?}"
    );
}
