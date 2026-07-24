//! A second create on the same path fails exclusively, leaving the first file in place.

use crate::observation::observation_sink::ObservationSink;
use crate::observation::output_path::OutputPath;
use crate::observation::sink_create_error::SinkCreateError;

use super::scratch_dir::scratch_dir;

#[test]
fn create_is_exclusive() {
    let dir = scratch_dir("create-is-exclusive");
    let output = OutputPath::new(dir.join("observations.ndjson"));

    let first = ObservationSink::create(&output).expect("the first exclusive create succeeds");
    // Release the first sink's handles cleanly before the second attempt.
    first.finalize().expect("finalizing the first sink cleanly");

    match ObservationSink::create(&output) {
        Err(SinkCreateError::NoFileCreatedByThisCall { .. }) => {}
        Err(other) => panic!("expected NoFileCreatedByThisCall, got {other:?}"),
        Ok(_) => panic!("a second exclusive create on an existing path must not succeed"),
    }
}
