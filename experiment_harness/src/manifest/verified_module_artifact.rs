//! The published module artifact: its verified hash and resulting database identity.

use crate::manifest::database_identity::DatabaseIdentity;
use crate::manifest::wasm_sha256::WasmSha256;

/// The published module artifact fact: the SHA-256 of the exact WASM bytes that were
/// published (already verified against the committed provenance constant) and the
/// database identity the publish produced. There is no separate remote/published-artifact
/// hash at 2.6.1 — the published bytes are the staged bytes, so their hash is the one
/// verified before publication.
#[derive(Debug, Clone)]
pub(crate) struct VerifiedModuleArtifact {
    sha256: WasmSha256,
    database_identity: DatabaseIdentity,
}

impl VerifiedModuleArtifact {
    pub(crate) fn new(sha256: WasmSha256, database_identity: DatabaseIdentity) -> Self {
        Self {
            sha256,
            database_identity,
        }
    }

    /// The verified SHA-256 of the published WASM bytes.
    pub(crate) fn sha256(&self) -> WasmSha256 {
        self.sha256
    }

    /// The published database identity.
    pub(crate) fn database_identity(&self) -> DatabaseIdentity {
        self.database_identity
    }
}
