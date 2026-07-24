//! Proof that the running server process is the expected standalone executable.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{ensure, Context, Result};

use crate::manifest::server_pid::ServerPid;

/// Proof that a running process is the expected `spacetimedb-standalone` binary,
/// established by resolving `/proc/<pid>/exe` and requiring it to equal the verified
/// standalone path (spec: "resolve `/proc/<pid>/exe`; require it to equal the asserted
/// Nix-store `spacetimedb-standalone` path before publishing or measuring"). This proves
/// the running local server binary without assuming a remote version endpoint.
#[derive(Debug, Clone)]
pub(crate) struct VerifiedServerProcess {
    pid: ServerPid,
    resolved_exe: PathBuf,
}

impl VerifiedServerProcess {
    /// Prove the process `pid` is executing `expected_exe`.
    ///
    /// `expected_exe` must already be canonical; `/proc/<pid>/exe` is canonicalized
    /// before comparison so symlink form cannot mask a mismatch.
    pub(crate) fn prove(pid: ServerPid, expected_exe: &Path) -> Result<Self> {
        let proc_exe = PathBuf::from(format!("/proc/{}/exe", pid.get()));
        let resolved_exe = fs::canonicalize(&proc_exe)
            .with_context(|| format!("resolving {proc_exe:?} for the running server"))?;
        ensure!(
            resolved_exe == expected_exe,
            "running server /proc exe {resolved_exe:?} != expected standalone {expected_exe:?}"
        );
        Ok(Self { pid, resolved_exe })
    }

    /// The verified process id.
    pub(crate) fn pid(&self) -> ServerPid {
        self.pid
    }

    /// The `/proc/<pid>/exe`-resolved executable path.
    pub(crate) fn resolved_exe(&self) -> &Path {
        &self.resolved_exe
    }
}
