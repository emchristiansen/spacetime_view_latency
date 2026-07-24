//! A canonical 64-character lowercase-hex string parses and round-trips to the same canonical hex.

use crate::manifest::database_identity::DatabaseIdentity;

#[test]
fn a_canonical_lowercase_hex_identity_parses() {
    let hex = "0".repeat(DatabaseIdentity::CANONICAL_HEX_LEN);
    let identity =
        DatabaseIdentity::parse_canonical_hex(&hex).expect("canonical lowercase hex must parse");
    assert_eq!(
        identity.identity().to_hex().to_string(),
        hex,
        "the parsed identity round-trips to the exact canonical hex it was parsed from"
    );
}
