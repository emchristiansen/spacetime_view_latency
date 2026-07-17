//! An empty vector does not seal: "non-empty" is not the invariant; exactly `BATCH_SIZE` is.

use crate::observation::raw_latencies::RawLatencies;

use super::samples::samples;

#[test]
fn empty_is_rejected() {
    RawLatencies::sealed(samples(0)).expect_err("an empty vector must not seal");
}
