//! SHA-256 digest of the built WASM module artifact.

use serde::{Serialize, Serializer};

/// The SHA-256 of the built WASM module. The committed constant
/// [`crate::module_artifact::module_wasm_sha256::MODULE_WASM_SHA256`] is the expected
/// value that the runtime-read WASM is verified against before publication, so the
/// published artifact and the committed bindings provably correspond (spec: "Official
/// 2.6.1 server provisioning"). A fixed 32-byte digest, serialized as canonical hex.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct WasmSha256([u8; 32]);

impl WasmSha256 {
    pub(crate) fn new(digest: [u8; 32]) -> Self {
        Self(digest)
    }

    /// The raw 32-byte digest.
    pub(crate) fn bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl Serialize for WasmSha256 {
    /// Serialized as canonical lowercase 64-character hex.
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        serializer.serialize_str(&hex::encode(self.0))
    }
}
