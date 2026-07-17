//! A run's manifest write fails loud when handed a manifest for a different run's coordinate.

use crate::manifest::run_coordinate::RunCoordinate;
use crate::manifest::validated_run_manifest::ValidatedRunManifest;
use crate::plan::schedule::Schedule;

use super::drive;
use super::driving_writer::DrivingWriter;

/// The coordinate binding is load-bearing: a [`RunWritingManifest`](crate::campaign::run_cursor::RunWritingManifest)
/// for the first run must reject a manifest built for the block's *other* run (the matched
/// control/arm at a different role). This proves a foreign manifest cannot be lifted into a run — the
/// assertion fails loud rather than recording a mismatched coordinate.
#[test]
#[should_panic(expected = "run coordinate")]
fn write_manifest_rejects_a_foreign_coordinate() {
    let first = drive::open_first_run(drive::SEED, DrivingWriter::always_ok());

    // The block's other run (the matched role) — a coordinate distinct from `first.coord`.
    let blocks = Schedule::preregistered().randomized_block_order(drive::SEED);
    let first_block = &blocks[0];
    let runs = first_block.ordered_runs(drive::SEED);
    let foreign = RunCoordinate::new(first_block, runs[1].role());

    let manifest = ValidatedRunManifest::fixture_for(foreign, drive::SEED);
    // Handing the first run a manifest for the other run's coordinate must panic.
    let _ = first.writing.write_manifest(&manifest);
}
