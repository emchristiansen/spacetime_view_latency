//! A successful writer return advances the sequence and the last-confirmed marker in lockstep.
//!
//! The scripted writer performs no I/O — a `write_line` that returns success stands in for the
//! sync-contract seam completing. This proves the sink's *sequencing* on a successful return, not
//! physical durability.

use crate::observation::observation_sink::ObservationSink;
use crate::observation::record_seq::RecordSeq;

use super::scripted_writer::ScriptedWriter;

#[test]
fn successful_writer_returns_advance_the_sequence() {
    let mut sink = ObservationSink::from_writer(Box::new(ScriptedWriter::ok_for(2)));
    assert_eq!(sink.pending_seq(), RecordSeq::zero());
    assert_eq!(sink.last_durable_seq(), None);

    let first = sink
        .persist_test_record(&1u64)
        .expect("the first writer return is treated as success");
    assert_eq!(first.get(), 0);
    assert_eq!(
        sink.last_durable_seq()
            .expect("the marker advances after the writer confirmed the first record")
            .get(),
        0
    );
    assert_eq!(sink.pending_seq().get(), 1);

    let second = sink
        .persist_test_record(&2u64)
        .expect("the second writer return is treated as success");
    assert_eq!(second.get(), 1);
    assert_eq!(
        sink.last_durable_seq()
            .expect("the marker advances after the writer confirmed the second record")
            .get(),
        1
    );
    assert_eq!(sink.pending_seq().get(), 2);

    sink.finalize()
        .expect("a sink whose writer returns only successes finalizes cleanly");
}
