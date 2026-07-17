//! A batch one sample short of `BATCH_SIZE` does not seal, and the error names the required count.

use crate::observation::raw_latencies::RawLatencies;
use crate::params::{BATCH_SIZE, BATCH_SIZE_USIZE};

use super::samples::samples;

#[test]
fn one_too_few_is_rejected() {
    let err = RawLatencies::sealed(samples(BATCH_SIZE_USIZE - 1))
        .expect_err("a short batch must not seal");
    let message = format!("{err:#}");
    assert!(
        message.contains(&BATCH_SIZE.to_string()),
        "the error names the required count: {message}"
    );
}
