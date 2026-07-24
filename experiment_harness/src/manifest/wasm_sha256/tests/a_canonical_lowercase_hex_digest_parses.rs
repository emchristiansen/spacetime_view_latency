//! A canonical 64-character lowercase-hex digest parses and round-trips to the same canonical hex.

use crate::manifest::wasm_sha256::WasmSha256;

#[test]
fn a_canonical_lowercase_hex_digest_parses() {
    let hex = "0".repeat(WasmSha256::CANONICAL_HEX_LEN);
    let digest = WasmSha256::parse_canonical_hex(&hex).expect("canonical lowercase hex must parse");
    assert_eq!(
        hex::encode(digest.bytes()),
        hex,
        "the parsed digest round-trips to the exact canonical hex it was parsed from"
    );
}
