//! A pre-write serialization failure is `DefinitelyNotWritten`, poisons the sink, and its refusal
//! carries no attempted successor. Because nothing was written, finalization is incomplete but not
//! durability-ambiguous.

use crate::observation::observation_sink::ObservationSink;
use crate::observation::persist_error::PersistError;
use crate::observation::record_seq::RecordSeq;
use crate::observation::tail_state::TailState;

use super::failing_serialize::FailingSerialize;
use super::scripted_writer::ScriptedWriter;

#[test]
fn serialization_failure_is_definitely_not_written() {
    // The writer would accept lines, but serialization fails before any bytes reach it.
    let mut sink = ObservationSink::from_writer(Box::new(ScriptedWriter::ok_for(4)));

    let first = sink
        .persist_test_record(&FailingSerialize)
        .expect_err("serialization fails before any write");
    assert!(
        matches!(first, PersistError::BeforeWrite { .. }),
        "a serialization failure is definitely-not-written, got {first:?}"
    );
    assert_eq!(first.tail_state(), TailState::DefinitelyNotWritten);
    assert_eq!(sink.pending_seq(), RecordSeq::zero());
    assert_eq!(sink.last_durable_seq(), None);
    assert!(
        sink.poisoned().is_some(),
        "the sink is terminal after the failure"
    );

    // A later, perfectly serializable value is still refused, with no attempted record of its own,
    // and the retained original state stays definitely-not-written.
    let refused = sink
        .persist_test_record(&1u64)
        .expect_err("a poisoned sink refuses further writes");
    match &refused {
        PersistError::SinkPoisoned { original } => {
            assert_eq!(original.tail_state(), TailState::DefinitelyNotWritten);
        }
        other => panic!("expected a refusal, got {other:?}"),
    }
    assert!(
        refused.attempted_record().is_none(),
        "a refusal wrote nothing and has no attempted record"
    );
    assert_eq!(sink.pending_seq(), RecordSeq::zero());

    // Finalization runs best-effort: the sync itself succeeds, so the outcome retains the prior
    // poison but is *not* durability-ambiguous — nothing was ever written.
    match sink.finalize() {
        Err(e) => {
            assert!(e.prior().is_some(), "the prior poison reason is retained");
            assert!(
                e.sync_failures().is_none(),
                "the final sync itself succeeded"
            );
            assert!(
                !e.is_durability_ambiguous(),
                "a pre-write poison leaves the tail clean, not ambiguous"
            );
        }
        Ok(()) => panic!("a poisoned sink does not finalize cleanly"),
    }
}
