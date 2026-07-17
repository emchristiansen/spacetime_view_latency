//! A run's dose write fails loud when handed an observation for a dose other than the active one.

use crate::campaign::run_cursor::RunDoseStep;
use crate::campaign::run_cursor::RunManifestStep;
use crate::dataset::dose_index::DoseIndex;
use crate::manifest::validated_run_manifest::ValidatedRunManifest;
use crate::observation::dose_observation::DoseObservation;

use super::drive;
use super::driving_writer::DrivingWriter;

/// The active-dose binding is load-bearing: after the manifest write, the run awaits the *first* dose,
/// and [`RunAwaitingDose::write_observation`](crate::campaign::run_cursor::RunAwaitingDose) must reject
/// an observation for a later dose. The observation shares the active run and manifest (so the coordinate
/// and manifest-reference bindings pass), isolating the dose mismatch, which fails loud.
#[test]
#[should_panic(expected = "dose")]
fn write_observation_rejects_a_foreign_dose() {
    let first = drive::open_first_run(drive::SEED, DrivingWriter::always_ok());
    let manifest = ValidatedRunManifest::fixture_for(first.coord.clone(), drive::SEED);
    let dosing = match first.writing.write_manifest(&manifest) {
        RunManifestStep::Seeding(seeding) => seeding.seeded().checked(),
        RunManifestStep::Stopped(_) => panic!("the manifest write must succeed"),
    };
    let awaiting = match dosing.next_dose() {
        RunDoseStep::Awaiting(awaiting) => awaiting,
        RunDoseStep::Exhausted(_) => panic!("the first dose must be awaited"),
    };

    // An observation for the second dose while the first is active — same run and manifest, foreign dose.
    let foreign = DoseObservation::fixture_for(&manifest, DoseIndex::ALL[1]);
    let _ = awaiting.write_observation(&foreign);
}
