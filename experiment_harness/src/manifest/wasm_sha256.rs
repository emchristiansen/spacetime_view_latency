//! SHA-256 digest of the built WASM module artifact.

use serde::{Serialize, Serializer};

use crate::manifest::canonical_hex_shape_error::CanonicalHexShapeError;
use crate::manifest::wasm_sha256_parse_error::WasmSha256ParseError;

/// The SHA-256 of the built WASM module. The committed constant
/// [`crate::module_artifact::module_wasm_sha256::MODULE_WASM_SHA256`] is the expected
/// value that the runtime-read WASM is verified against before publication, so the
/// published artifact and the committed bindings provably correspond (spec: "Official
/// 2.6.1 server provisioning"). A fixed 32-byte digest, serialized as canonical hex.
///
/// This type is the **single owner** of the canonical module-hash-hex format. Its shape rule (fixed
/// length, lowercase hex) is single-sourced through the shared [`CanonicalHexShapeError::check`], which
/// this owner *translates* into its own typed [`WasmSha256ParseError`]; digests read from disk are built
/// via [`Self::new`] from raw bytes, while a recorded manifest digest is re-parsed through
/// [`Self::parse_canonical_hex`], so the two cannot drift.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct WasmSha256([u8; 32]);

impl WasmSha256 {
    /// Canonical module-hash hex length: a 32-byte SHA-256 as lowercase hex.
    pub(crate) const CANONICAL_HEX_LEN: usize = 64;

    pub(crate) fn new(digest: [u8; 32]) -> Self {
        Self(digest)
    }

    /// The exhaustive typed reason `hex` is not a canonical module digest, or `None` if it is. Delegates
    /// the shape rule to the shared [`CanonicalHexShapeError::check`] and translates its failure into the
    /// module-hash-specific typed error, so the algorithm is single-sourced; it does not decode.
    pub(crate) fn canonical_hex_error(hex: &str) -> Option<WasmSha256ParseError> {
        CanonicalHexShapeError::check(hex, Self::CANONICAL_HEX_LEN)
            .err()
            .map(WasmSha256ParseError::from)
    }

    /// Parse a canonical lowercase-hex digest into a trusted [`WasmSha256`], or the exhaustive typed
    /// reason it is not canonical. A string that passes the canonical shape check always decodes into
    /// exactly 32 bytes, so a post-check decode failure is an invariant violation, not a recoverable
    /// error.
    pub(crate) fn parse_canonical_hex(hex: &str) -> Result<Self, WasmSha256ParseError> {
        if let Some(error) = Self::canonical_hex_error(hex) {
            return Err(error);
        }
        let bytes = hex::decode(hex)
            .expect("a canonical lowercase-hex string of CANONICAL_HEX_LEN always decodes");
        let bytes: [u8; 32] = bytes
            .as_slice()
            .try_into()
            .expect("CANONICAL_HEX_LEN / 2 == 32 bytes for a canonical module digest");
        Ok(Self::new(bytes))
    }

    /// The raw 32-byte digest.
    pub(crate) fn bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Canonical lowercase 64-character hex. The single owner of this type's canonical-hex rendering, so
    /// the report's `String` projection and the [`Serialize`] impl share one format and cannot drift.
    pub(crate) fn canonical_hex(&self) -> String {
        hex::encode(self.0)
    }
}

impl Serialize for WasmSha256 {
    /// Serialized as canonical lowercase 64-character hex.
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.canonical_hex())
    }
}

#[cfg(test)]
mod tests;
