//! A length-correct digest string whose first offending byte is not a hexadecimal digit at all is
//! categorized as non-hex — distinct from an uppercase hex digit, which is non-canonical case.

use crate::manifest::wasm_sha256::WasmSha256;
use crate::manifest::wasm_sha256_parse_error::WasmSha256ParseError;

#[test]
fn a_non_hex_character_is_rejected() {
    let hex = format!("g{}", "0".repeat(WasmSha256::CANONICAL_HEX_LEN - 1));
    let error = WasmSha256::parse_canonical_hex(&hex)
        .expect_err("'g' is not a hexadecimal digit and must be rejected");
    assert_eq!(
        error,
        WasmSha256ParseError::NonHex {
            offending_index: 0,
            offending_byte: b'g',
        },
        "the typed error locates the first non-hexadecimal byte by index and raw byte"
    );
}
