//! `WasmSha256`'s `Serialize` output is exactly its single-owner `canonical_hex` rendering, so the report's
//! `String` projection and the serialized form share one format and cannot drift.

use crate::manifest::wasm_sha256::WasmSha256;

#[test]
fn serialize_routes_through_canonical_hex() {
    let hex = "0123456789abcdef".repeat(WasmSha256::CANONICAL_HEX_LEN / 16);
    let digest = WasmSha256::parse_canonical_hex(&hex).expect("canonical lowercase hex must parse");

    let serialized = serde_json::to_value(&digest).expect("a digest serializes");

    // The serialized form is exactly what the single-owner render method produces...
    assert_eq!(
        serialized,
        serde_json::Value::String(digest.canonical_hex()),
        "Serialize emits exactly the single-owner canonical_hex rendering"
    );
    // ...which round-trips to the canonical hex it was parsed from.
    assert_eq!(serialized, serde_json::Value::String(hex));
}
