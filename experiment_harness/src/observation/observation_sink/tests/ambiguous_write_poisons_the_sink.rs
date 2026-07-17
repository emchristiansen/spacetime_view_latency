//! A durability-seam failure poisons the sink: later writes are refused (not re-attempted) and
//! finalize reports durability ambiguity.
//!
//! The scripted writer performs no I/O — it injects an error from the post-serialization seam. The
//! sink conservatively classifies any seam failure as durability-ambiguous, because the production
//! seam may fail at write, flush, or `sync_data`, any of which can leave a full line durable. This
//! proves that classification and the terminal (poisoning) transition, not that a write occurred.

use crate::observation::observation_sink::ObservationSink;
use crate::observation::persist_error::PersistError;
use crate::observation::record_seq::RecordSeq;
use crate::observation::tail_state::TailState;

use super::scripted_writer::ScriptedWriter;

#[test]
fn ambiguous_write_poisons_the_sink() {
    // The writer errors on its first line, standing in for a failure of the post-serialization seam.
    let mut sink = ObservationSink::from_writer(Box::new(ScriptedWriter::ok_for(0)));

    let first = sink
        .persist_test_record(&1u64)
        .expect_err("the writer seam errors on the first line");
    assert!(
        matches!(first, PersistError::DurabilityAmbiguous { .. }),
        "a seam failure is conservatively classified durability-ambiguous, got {first:?}"
    );
    // The failed record's sequence did not advance and nothing is marked confirmed.
    assert_eq!(sink.pending_seq(), RecordSeq::zero());
    assert_eq!(sink.last_durable_seq(), None);
    assert!(
        sink.poisoned().is_some(),
        "the sink is terminal after the failure"
    );

    // A later write is refused, not re-attempted: it carries the original ambiguous tail state, has
    // no attempted record of its own, and does not advance the sequence.
    let refused = sink
        .persist_test_record(&2u64)
        .expect_err("a poisoned sink refuses further writes");
    match &refused {
        PersistError::SinkPoisoned { original } => {
            assert_eq!(original.tail_state(), TailState::DurabilityAmbiguous);
        }
        other => panic!("expected a refusal, got {other:?}"),
    }
    assert!(
        refused.attempted_record().is_none(),
        "a refusal wrote nothing and has no attempted record"
    );
    assert_eq!(
        sink.pending_seq(),
        RecordSeq::zero(),
        "a refusal never advances the sequence"
    );

    // Finalization still runs best-effort and reports the poisoned tail's durability ambiguity.
    match sink.finalize() {
        Err(e) => assert!(
            e.is_durability_ambiguous(),
            "a poisoned ambiguous tail makes finalization durability-ambiguous"
        ),
        Ok(()) => panic!("finalizing a poisoned sink is not clean success"),
    }
}
