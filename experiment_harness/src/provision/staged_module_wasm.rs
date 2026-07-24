//! The runtime-verified WASM bytes, staged to an owned tempfile for publication.

use std::fs;
use std::io::Write;
use std::path::Path;

use anyhow::{ensure, Context, Result};
use sha2::{Digest, Sha256};
use tempfile::NamedTempFile;

use crate::manifest::wasm_sha256::WasmSha256;

/// The exact WASM bytes to publish, read from disk and verified against the committed
/// provenance hash before being written to an owned private tempfile.
///
/// This closes the provenance chain (spec: "verify the harness-generated bindings and
/// published artifact correspond to that hash before measuring"): the committed bindings and
/// the committed [`WasmSha256`] constant were produced from one build; here the runtime-read
/// bytes are re-hashed and required to equal that constant, then written to a tempfile that is
/// the one and only thing published — so bindings, hash, and published bytes all correspond,
/// with no committed binary.
///
/// Owns a [`NamedTempFile`] whose lifetime it gates: removed only by an explicit
/// [`Self::cleanup`], and [`Drop`] asserts that happened rather than best-effort deleting.
#[derive(Debug)]
pub(crate) struct StagedModuleWasm {
    sha256: WasmSha256,
    /// `Some` while owned; taken by [`Self::cleanup`]. `Drop` requires it to be `None`.
    temp: Option<NamedTempFile>,
}

impl StagedModuleWasm {
    /// Read the WASM at `wasm_path`, verify its SHA-256 equals `expected`, and stage the exact
    /// verified bytes to an owned tempfile.
    pub(crate) fn load(wasm_path: &Path, expected: WasmSha256) -> Result<Self> {
        let bytes =
            fs::read(wasm_path).with_context(|| format!("reading built WASM {wasm_path:?}"))?;

        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let digest: [u8; 32] = hasher.finalize().into();
        let sha256 = WasmSha256::new(digest);
        ensure!(
            sha256 == expected,
            "built WASM {wasm_path:?} sha256 {} != committed provenance hash {}",
            hex::encode(sha256.bytes()),
            hex::encode(expected.bytes())
        );

        let mut temp = NamedTempFile::new().context("creating staged WASM tempfile")?;
        temp.write_all(&bytes)
            .context("writing verified WASM bytes to the staged tempfile")?;
        temp.as_file_mut()
            .flush()
            .context("flushing the staged WASM tempfile")?;
        temp.as_file_mut()
            .sync_all()
            .context("syncing the staged WASM tempfile")?;

        Ok(Self {
            sha256,
            temp: Some(temp),
        })
    }

    /// The path of the staged tempfile to hand to `spacetimedb-cli publish --bin-path`.
    pub(crate) fn bin_path(&self) -> &Path {
        self.temp
            .as_ref()
            .expect("StagedModuleWasm::bin_path called after cleanup consumed the tempfile")
            .path()
    }

    /// The verified SHA-256 of the staged bytes.
    pub(crate) fn sha256(&self) -> WasmSha256 {
        self.sha256
    }

    /// Remove the staged tempfile, surfacing any removal error.
    pub(crate) fn cleanup(mut self) -> Result<()> {
        let temp = self
            .temp
            .take()
            .expect("StagedModuleWasm::cleanup called after the tempfile was already cleaned up");
        temp.close().context("removing the staged WASM tempfile")?;
        Ok(())
    }
}

impl Drop for StagedModuleWasm {
    fn drop(&mut self) {
        assert!(
            self.temp.is_none(),
            "StagedModuleWasm dropped without an explicit cleanup(); the tempfile would leak"
        );
    }
}
