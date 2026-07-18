//! A length-short hex string is rejected with the typed `Length` evidence, before any per-byte check.

use crate::manifest::database_identity::DatabaseIdentity;
use crate::manifest::database_identity_parse_error::DatabaseIdentityParseError;

#[test]
fn a_wrong_length_hex_is_rejected() {
    let hex = "0".repeat(DatabaseIdentity::CANONICAL_HEX_LEN - 1);
    let error = DatabaseIdentity::parse_canonical_hex(&hex)
        .expect_err("a hex string one character short of canonical length must be rejected");
    assert_eq!(
        error,
        DatabaseIdentityParseError::Length {
            expected: DatabaseIdentity::CANONICAL_HEX_LEN,
            observed: DatabaseIdentity::CANONICAL_HEX_LEN - 1,
        },
        "the typed error carries the canonical expected length and the observed short length"
    );
}
