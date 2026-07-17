//! Successful line-seam returns followed by a finalizer failure: finalization is durability-ambiguous
//! with no prior poison — the ambiguity is contributed solely by the failed finalizer.
//!
//! The scripted writer performs no I/O; its `write_line` returns success and its `finalize` returns
//! an error. This proves the aggregation/classification of the finalize outcome, not any write.

use crate::observation::observation_sink::ObservationSink;

use super::scripted_writer::ScriptedWriter;

#[test]
fn successful_line_returns_then_finalizer_failure() {
    // Every line-seam call returns success, but the finalizer returns an error.
    let mut sink = ObservationSink::from_writer(Box::new(ScriptedWriter::finalize_failing(2)));

    sink.persist_test_record(&1u64)
        .expect("the first line-seam call returns success");
    sink.persist_test_record(&2u64)
        .expect("the second line-seam call returns success");
    assert!(
        sink.poisoned().is_none(),
        "no persist failed, so there is no prior poison"
    );

    match sink.finalize() {
        Err(e) => {
            assert!(e.prior().is_none(), "there is no prior poison reason");
            let sync_failures = e
                .sync_failures()
                .expect("the finalizer reported failure");
            assert!(
                sync_failures.file().is_some() && sync_failures.directory().is_some(),
                "the scripted finalizer reports both the file and directory sync as failed"
            );
            assert!(
                e.is_durability_ambiguous(),
                "a finalizer failure makes finalization durability-ambiguous"
            );
        }
        Ok(()) => panic!("a failed finalizer is not clean success"),
    }
}
