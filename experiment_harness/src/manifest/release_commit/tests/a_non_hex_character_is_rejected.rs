//! A length-correct commit string whose first offending byte is not a hexadecimal digit at all is
//! categorized as non-hex — distinct from an uppercase hex digit, which is non-canonical case.

use crate::manifest::release_commit::ReleaseCommit;
use crate::manifest::release_commit_parse_error::ReleaseCommitParseError;

#[test]
fn a_non_hex_character_is_rejected() {
    let hex = format!("g{}", "0".repeat(ReleaseCommit::CANONICAL_HEX_LEN - 1));
    let error = ReleaseCommit::parse_canonical_hex(&hex)
        .expect_err("'g' is not a hexadecimal digit and must be rejected");
    assert_eq!(
        error,
        ReleaseCommitParseError::NonHex {
            offending_index: 0,
            offending_byte: b'g',
        },
        "the typed error locates the first non-hexadecimal byte by index and raw byte"
    );
}
