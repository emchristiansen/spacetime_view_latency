//! A prior persist failure *and* a final-sync failure: the combined finalization outcome retains
//! both parts rather than discarding either.

use crate::observation::observation_sink::ObservationSink;
use crate::observation::persist_error::PersistError;
use crate::observation::tail_state::TailState;

use super::scripted_writer::ScriptedWriter;

#[test]
fn prior_failure_and_finalize_sync_failure_retain_both() {
    // The first line-seam call reports failure (poisoning the sink) and the finalizer reports failure
    // too.
    let mut sink = ObservationSink::from_writer(Box::new(ScriptedWriter::finalize_failing(0)));

    let first = sink
        .persist_test_record(&1u64)
        .expect_err("the line-seam call reports failure on the first line");
    assert!(
        matches!(first, PersistError::DurabilityAmbiguous { .. }),
        "a seam failure is conservatively classified durability-ambiguous, got {first:?}"
    );
    assert!(
        sink.poisoned().is_some(),
        "the sink is terminal after the failure"
    );

    match sink.finalize() {
        Err(e) => {
            let prior = e
                .prior()
                .expect("the prior ambiguous poison reason is retained");
            assert_eq!(prior.tail_state(), TailState::DurabilityAmbiguous);
            let sync_failures = e
                .sync_failures()
                .expect("the final sync failure is retained alongside the prior poison");
            // Both independent typed slots are retained — neither the file nor the directory error is
            // merged into or discarded in favor of the other.
            assert!(
                sync_failures.file().is_some(),
                "the file sync failure is retained in its own slot"
            );
            assert!(
                sync_failures.directory().is_some(),
                "the directory sync failure is retained in its own slot"
            );
            assert!(e.is_durability_ambiguous());
        }
        Ok(()) => panic!("a poisoned sink with a failed final sync does not finalize cleanly"),
    }
}
