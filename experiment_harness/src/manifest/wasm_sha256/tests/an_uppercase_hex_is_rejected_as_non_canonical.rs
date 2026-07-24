//! A length-correct digest string with an uppercase digit is rejected as non-canonical case, located
//! precisely rather than claimed from length alone.

use crate::manifest::wasm_sha256::WasmSha256;
use crate::manifest::wasm_sha256_parse_error::WasmSha256ParseError;

#[test]
fn an_uppercase_hex_is_rejected_as_non_canonical() {
    let hex = format!("A{}", "0".repeat(WasmSha256::CANONICAL_HEX_LEN - 1));
    let error = WasmSha256::parse_canonical_hex(&hex)
        .expect_err("an uppercase hex digit is valid hex but not the canonical lowercase form");
    assert_eq!(
        error,
        WasmSha256ParseError::NonCanonicalCase {
            offending_index: 0,
            offending_byte: b'A',
        },
        "the typed error locates the first uppercase digit by index and raw byte"
    );
}
