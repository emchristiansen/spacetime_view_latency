//! A run awaiting its first dose assembles that dose's observation from its *own* owned context and drawn
//! active dose: the test supplies only measured evidence, and the serialized line carries the run's own
//! coordinate and the drawn dose — no observation crosses the boundary.

use crate::campaign::run_cursor::RunDoseStep;
use crate::campaign::run_cursor::RunManifestStep;
use crate::campaign::run_cursor::RunObservationStep;
use crate::campaign::run_cursor::RunWritingManifest;
use crate::dataset::dose_index::DoseIndex;
use crate::observation::dose_evidence::DoseEvidence;
use crate::observation::observation_sink::ObservationSink;

use super::capturing_line_writer::CapturingLineWriter;
use super::drive;

/// After the manifest write the run awaits the *first* dose, drawn from its own ladder — the awaiting state
/// names it, never a caller. Handing it only the measured [`DoseEvidence`],
/// [`RunAwaitingDose::write_observation`](crate::campaign::run_cursor::RunAwaitingDose) assembles the
/// observation from the owned context and that active dose and writes it, advancing the ladder. The bytes
/// the sink actually serialized are captured and parsed: the dose observation line carries the run cursor's
/// *own* run coordinate and the first dose it drew — runtime proof (not comment or API inspection) that the
/// observation is derived from the owned context, so a foreign observation is unrepresentable rather than
/// merely rejected. The parse envelope (`{ record, body: { observation: … } }`) is the same one the sink's
/// own `written_line_carries_the_assigned_record_identity` test reads.
#[test]
fn awaiting_dose_derives_and_binds_its_observation() {
    let coord = drive::first_run_coordinate(drive::SEED);
    let context = drive::resolve_context(&coord, drive::SEED);
    let (writer, lines) = CapturingLineWriter::new();
    let sink = ObservationSink::from_writer(Box::new(writer));

    let dosing = match RunWritingManifest::begin(sink).write_manifest(context) {
        RunManifestStep::Seeding(seeding) => seeding.seeded().checked(),
        RunManifestStep::Stopped(_) => {
            panic!("CapturingLineWriter never fails a write, so the manifest write must succeed")
        }
    };
    let awaiting = match dosing.next_dose() {
        RunDoseStep::Awaiting(awaiting) => awaiting,
        RunDoseStep::Exhausted(_) => panic!("the first dose must be awaited"),
    };
    assert_eq!(
        awaiting.active(),
        DoseIndex::ALL[0],
        "the run awaits the first dose drawn from its own ladder"
    );
    match awaiting.write_observation(DoseEvidence::fixture()) {
        RunObservationStep::Dosing(_) => {}
        RunObservationStep::Stopped(_) => {
            panic!(
                "CapturingLineWriter never fails a write, so the derived dose observation write must succeed"
            )
        }
    }

    // Inspect exactly the bytes the sink serialized: the manifest line, then the first dose's observation
    // line. The observation line's coordinate must carry the run's own coordinate and the drawn dose.
    let captured: Vec<Vec<u8>> = lines.try_iter().collect();
    assert_eq!(
        captured.len(),
        2,
        "one manifest line and one observation line were written"
    );
    let observation_line: serde_json::Value =
        serde_json::from_slice(&captured[1]).expect("the observation line is valid JSON");
    let coordinate = &observation_line["body"]["observation"]["coordinate"];
    assert_eq!(
        coordinate["run"],
        serde_json::to_value(&coord).expect("the run coordinate serializes"),
        "the serialized observation carries the run cursor's own coordinate"
    );
    assert_eq!(
        coordinate["dose"],
        serde_json::to_value(DoseIndex::ALL[0]).expect("the dose index serializes"),
        "and the first dose the run drew from its own ladder"
    );
}
