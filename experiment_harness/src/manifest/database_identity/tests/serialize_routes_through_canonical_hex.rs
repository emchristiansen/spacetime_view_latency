//! `DatabaseIdentity`'s `Serialize` output is exactly its single-owner `canonical_hex` rendering, so the
//! report's `String` projection and the serialized form share one format and cannot drift.

use crate::manifest::database_identity::DatabaseIdentity;

#[test]
fn serialize_routes_through_canonical_hex() {
    let hex = "0".repeat(DatabaseIdentity::CANONICAL_HEX_LEN);
    let identity =
        DatabaseIdentity::parse_canonical_hex(&hex).expect("canonical lowercase hex must parse");

    let serialized = serde_json::to_value(&identity).expect("an identity serializes");

    // The serialized form is exactly what the single-owner render method produces...
    assert_eq!(
        serialized,
        serde_json::Value::String(identity.canonical_hex()),
        "Serialize emits exactly the single-owner canonical_hex rendering"
    );
    // ...which round-trips to the canonical hex it was parsed from.
    assert_eq!(serialized, serde_json::Value::String(hex));
}
