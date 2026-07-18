//! A length-short digest string is rejected with the typed `Length` evidence, before any per-byte check.

use crate::manifest::wasm_sha256::WasmSha256;
use crate::manifest::wasm_sha256_parse_error::WasmSha256ParseError;

#[test]
fn a_wrong_length_hex_is_rejected() {
    let hex = "0".repeat(WasmSha256::CANONICAL_HEX_LEN - 1);
    let error = WasmSha256::parse_canonical_hex(&hex)
        .expect_err("a hex string one character short of canonical length must be rejected");
    assert_eq!(
        error,
        WasmSha256ParseError::Length {
            expected: WasmSha256::CANONICAL_HEX_LEN,
            observed: WasmSha256::CANONICAL_HEX_LEN - 1,
        },
        "the typed error carries the canonical expected length and the observed short length"
    );
}
