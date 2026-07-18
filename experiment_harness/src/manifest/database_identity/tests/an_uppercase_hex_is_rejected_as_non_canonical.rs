//! A length-correct hex string with an uppercase digit is rejected as non-canonical case, located
//! precisely rather than claimed from length alone.

use crate::manifest::database_identity::DatabaseIdentity;
use crate::manifest::database_identity_parse_error::DatabaseIdentityParseError;

#[test]
fn an_uppercase_hex_is_rejected_as_non_canonical() {
    let hex = format!("A{}", "0".repeat(DatabaseIdentity::CANONICAL_HEX_LEN - 1));
    let error = DatabaseIdentity::parse_canonical_hex(&hex)
        .expect_err("an uppercase hex digit is valid hex but not the canonical lowercase form");
    assert_eq!(
        error,
        DatabaseIdentityParseError::NonCanonicalCase {
            offending_index: 0,
            offending_byte: b'A',
        },
        "the typed error locates the first uppercase digit by index and raw byte"
    );
}
