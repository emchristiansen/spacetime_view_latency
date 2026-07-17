//! Every written line serializes the full assigned [`RecordId`] (`{seq, kind}`), and the manifest
//! receipt's record identity agrees with the manifest line actually written. This exercises the real
//! [`ObservationSink::write_manifest`] and [`ObservationSink::write_observation`] paths — not a test
//! envelope — so a wiring mistake between the assigned sequence, the returned receipt, and the bytes
//! serialized into the line would be caught.

use crate::observation::dose_observation::DoseObservation;
use crate::observation::observation_sink::ObservationSink;
use crate::manifest::validated_run_manifest::ValidatedRunManifest;

use super::capturing_writer::CapturingWriter;

#[test]
fn written_line_carries_the_assigned_record_identity() {
    let (writer, lines) = CapturingWriter::new();
    let mut sink = ObservationSink::from_writer(Box::new(writer));

    // The real manifest write path returns a receipt carrying the assigned record identity.
    let manifest = ValidatedRunManifest::fixture();
    let receipt = sink
        .write_manifest(&manifest)
        .expect("the manifest line seam returns success");
    assert_eq!(receipt.seq().get(), 0, "the manifest is the first record");

    // The real observation write path returns a receipt binding the assigned record and its dose.
    let observation = DoseObservation::fixture();
    let observation_receipt = sink
        .write_observation(&observation)
        .expect("the observation line seam returns success");
    assert_eq!(
        observation_receipt.record().seq().get(),
        1,
        "the observation follows the manifest in sequence"
    );
    assert_eq!(
        observation_receipt.dose(),
        observation.dose(),
        "the receipt binds the observation's actual dose"
    );

    sink.finalize()
        .expect("the capturing writer finalizes cleanly");

    let captured = lines.borrow();
    assert_eq!(captured.len(), 2, "exactly one line was written per record");

    // The manifest line serializes the receipt's exact assigned record identity ({seq: 0, Manifest}).
    let manifest_line: serde_json::Value =
        serde_json::from_slice(&captured[0]).expect("the manifest line is valid JSON");
    assert_eq!(
        manifest_line["record"],
        serde_json::to_value(receipt.record()).expect("the record identity serializes"),
        "the manifest line serializes the assigned record identity the receipt reports"
    );

    // The observation line serializes {seq: 1, Dose(index)}, agreeing with the returned sequence.
    let observation_line: serde_json::Value =
        serde_json::from_slice(&captured[1]).expect("the observation line is valid JSON");
    assert_eq!(
        observation_line["record"],
        serde_json::to_value(observation_receipt.record())
            .expect("the record identity serializes"),
        "the observation line serializes its assigned {{seq, Dose}} identity"
    );
}
