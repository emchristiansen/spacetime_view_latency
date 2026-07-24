//! A batch one sample over `BATCH_SIZE` does not seal.

use crate::observation::raw_latencies::RawLatencies;
use crate::params::BATCH_SIZE_USIZE;

use super::samples::samples;

#[test]
fn one_too_many_is_rejected() {
    RawLatencies::sealed(samples(BATCH_SIZE_USIZE + 1))
        .expect_err("an overfull batch must not seal");
}
