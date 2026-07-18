//! A length-short commit string is rejected with the typed `Length` evidence, before any per-byte check.

use crate::manifest::release_commit::ReleaseCommit;
use crate::manifest::release_commit_parse_error::ReleaseCommitParseError;

#[test]
fn a_wrong_length_hex_is_rejected() {
    let hex = "0".repeat(ReleaseCommit::CANONICAL_HEX_LEN - 1);
    let error = ReleaseCommit::parse_canonical_hex(&hex)
        .expect_err("a hex string one character short of canonical length must be rejected");
    assert_eq!(
        error,
        ReleaseCommitParseError::Length {
            expected: ReleaseCommit::CANONICAL_HEX_LEN,
            observed: ReleaseCommit::CANONICAL_HEX_LEN - 1,
        },
        "the typed error carries the canonical expected length and the observed short length"
    );
}
