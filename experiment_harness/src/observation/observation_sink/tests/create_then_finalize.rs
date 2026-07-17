//! A freshly created, unpoisoned sink finalizes cleanly.

use crate::observation::observation_sink::ObservationSink;
use crate::observation::output_path::OutputPath;

use super::scratch_dir::scratch_dir;

#[test]
fn create_then_finalize() {
    let dir = scratch_dir("create-then-finalize");
    let output = OutputPath::new(dir.join("observations.ndjson"));

    let sink = ObservationSink::create(&output).expect("the exclusive create succeeds");
    sink.finalize()
        .expect("a fresh, unpoisoned sink finalizes cleanly");
}
