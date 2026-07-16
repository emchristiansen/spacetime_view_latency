//! An owned, fresh data directory (plus ephemeral JWT key dir) for one isolated server.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use tempfile::TempDir;

/// File name (inside `logs/`) the started server's stdout is redirected to.
const SERVER_STDOUT_FILE: &str = "server.stdout";

/// File name (inside `logs/`) the started server's stderr is redirected to.
const SERVER_STDERR_FILE: &str = "server.stderr";

/// An experiment-specific, freshly created data directory for one standalone server, so no
/// arm can inherit another arm's table size (spec: "Use a fresh or explicitly cleared
/// database for each arm/repetition"). Owns a [`TempDir`] whose lifetime it gates: the
/// directory is removed only by an explicit [`Self::cleanup`], and [`Drop`] asserts that
/// cleanup happened rather than silently leaking or best-effort deleting.
///
/// The JWT key directory lives *inside* this owned tree. The pinned 2.6.1 standalone
/// auto-generates its `id_ecdsa`/`id_ecdsa.pub` there on first start via the (hidden)
/// `--jwt-key-dir` flag — source-confirmed at `v2.6.1` in
/// `crates/standalone/src/subcommands/start.rs:55` (arg, `hide(true)`) and `:180`
/// (`CertificateAuthority::in_cli_config_dir` → `get_or_create_keys`). Only the path is ever
/// recorded downstream — no private key material is read into or serialized by the manifest.
///
/// The server's stdout/stderr are redirected to files under an owned `logs/` subdirectory (so
/// they cannot deadlock on a full pipe buffer and are removed with the rest of the tree on
/// cleanup); they are read for exit diagnostics before cleanup.
#[derive(Debug)]
pub(crate) struct FreshDataDir {
    /// `Some` while owned; taken by [`Self::cleanup`]. `Drop` requires it to be `None`.
    root: Option<TempDir>,
    data_dir: PathBuf,
    keys_dir: PathBuf,
    logs_dir: PathBuf,
}

impl FreshDataDir {
    /// Create a fresh temp root with `data/`, `keys/`, and `logs/` subdirectories.
    pub(crate) fn new() -> Result<Self> {
        let root = tempfile::Builder::new()
            .prefix("stdb-view-experiment-")
            .tempdir()
            .context("creating fresh server data root")?;
        let data_dir = root.path().join("data");
        let keys_dir = root.path().join("keys");
        let logs_dir = root.path().join("logs");
        fs::create_dir(&data_dir)
            .with_context(|| format!("creating server data directory {data_dir:?}"))?;
        fs::create_dir(&keys_dir)
            .with_context(|| format!("creating server JWT key directory {keys_dir:?}"))?;
        fs::create_dir(&logs_dir)
            .with_context(|| format!("creating server log capture directory {logs_dir:?}"))?;
        Ok(Self {
            root: Some(root),
            data_dir,
            keys_dir,
            logs_dir,
        })
    }

    /// The server data directory.
    pub(crate) fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    /// The ephemeral JWT key directory (path only; contents are never read).
    pub(crate) fn keys_dir(&self) -> &Path {
        &self.keys_dir
    }

    /// Path (inside the owned tree) the server's stdout is redirected to.
    pub(crate) fn server_stdout_path(&self) -> PathBuf {
        self.logs_dir.join(SERVER_STDOUT_FILE)
    }

    /// Path (inside the owned tree) the server's stderr is redirected to.
    pub(crate) fn server_stderr_path(&self) -> PathBuf {
        self.logs_dir.join(SERVER_STDERR_FILE)
    }

    /// Remove the fresh data directory, surfacing any removal error.
    pub(crate) fn cleanup(mut self) -> Result<()> {
        let root = self
            .root
            .take()
            .expect("FreshDataDir::cleanup called after the data dir was already cleaned up");
        root.close().context("removing fresh server data directory")
    }
}

impl Drop for FreshDataDir {
    fn drop(&mut self) {
        assert!(
            self.root.is_none(),
            "FreshDataDir dropped without an explicit cleanup(); the data directory would leak"
        );
    }
}
