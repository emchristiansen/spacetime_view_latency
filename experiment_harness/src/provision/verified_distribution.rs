//! The verified official 2.6.1 distribution: both binaries resolved and version-checked.

use std::ffi::OsStr;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, fs};

use anyhow::{ensure, Context, Result};

use crate::manifest::verified_cli::VerifiedCli;
use crate::manifest::verified_standalone::VerifiedStandalone;

/// Environment variable the flake devshell exports with the absolute bin directory of the
/// pinned official binaries (see `flake.nix`). The harness reads it fail-fast — no default,
/// no ambient PATH, no `/nix/store` glob (spec: "Official 2.6.1 server provisioning").
const STORE_BIN_DIR_ENV: &str = "SPACETIMEDB_2_6_1_BIN";

/// File name of the CLI executable inside the store bin directory.
const CLI_EXE_NAME: &str = "spacetimedb-cli";

/// File name of the standalone server executable inside the store bin directory.
const STANDALONE_EXE_NAME: &str = "spacetimedb-standalone";

/// The Nix store root. The verified bin directory must be exactly `<store>/<output>/bin` — an
/// immediate `bin` child of a *single* store output directly under the canonical store root —
/// so `/tmp`, non-store roots, and nested store subpaths are rejected. The expected output
/// name/hash is deliberately not asserted (that is the flake's pin, not the harness's to guess).
const NIX_STORE_ROOT: &str = "/nix/store";

/// The single official Nix `spacetimedb-2.6.1` store output, with both executables resolved
/// by absolute path from the same bin directory and each independently version-checked
/// (spec: "Resolve both from the same Nix store output"; "do not equate the CLI version with
/// the server version"). Constructed only by [`Self::resolve`], which fails before any run on
/// any mismatch.
#[derive(Debug, Clone)]
pub(crate) struct VerifiedDistribution {
    store_bin_dir: PathBuf,
    cli: VerifiedCli,
    standalone: VerifiedStandalone,
}

impl VerifiedDistribution {
    /// Resolve and verify both binaries from the flake-provided store bin directory.
    ///
    /// Reads [`STORE_BIN_DIR_ENV`] (fail-fast if unset), canonicalizes it, resolves both
    /// executables by absolute path under it, checks each is an executable regular file with
    /// that exact parent, and parses/verifies each `--version` output against the expected
    /// 2.6.1 version (and, for the CLI, release commit).
    pub(crate) fn resolve() -> Result<Self> {
        let raw_bin_dir = env::var(STORE_BIN_DIR_ENV).with_context(|| {
            format!("{STORE_BIN_DIR_ENV} is unset — run inside the flake devshell (direnv exec .)")
        })?;
        let store_bin_dir = fs::canonicalize(&raw_bin_dir)
            .with_context(|| format!("canonicalizing {STORE_BIN_DIR_ENV}={raw_bin_dir:?}"))?;
        assert_nix_store_bin_shape(&store_bin_dir)?;

        let cli_exe = resolve_executable(&store_bin_dir, CLI_EXE_NAME)?;
        let standalone_exe = resolve_executable(&store_bin_dir, STANDALONE_EXE_NAME)?;

        let cli = VerifiedCli::parse(&cli_exe, &run_version(&cli_exe)?)?;
        let standalone = VerifiedStandalone::parse(&standalone_exe, &run_version(&standalone_exe)?)?;

        Ok(Self {
            store_bin_dir,
            cli,
            standalone,
        })
    }

    /// The canonical Nix store bin directory both executables were resolved from.
    pub(crate) fn store_bin_dir(&self) -> &Path {
        &self.store_bin_dir
    }

    /// The verified CLI proof.
    pub(crate) fn cli(&self) -> &VerifiedCli {
        &self.cli
    }

    /// The verified standalone proof.
    pub(crate) fn standalone(&self) -> &VerifiedStandalone {
        &self.standalone
    }

    /// The absolute, canonical CLI executable path.
    pub(crate) fn cli_exe(&self) -> &Path {
        self.cli.exe()
    }

    /// The absolute, canonical standalone executable path.
    pub(crate) fn standalone_exe(&self) -> &Path {
        self.standalone.exe()
    }
}

/// Require `store_bin_dir` (already canonical) to have the exact Nix store shape
/// `<store>/<output>/bin`: its basename is `bin`, and its parent (`<output>`) is an immediate
/// child of the canonical [`NIX_STORE_ROOT`]. This rejects `/tmp`, non-store roots, and nested
/// store subpaths without inferring the expected output name or hash.
fn assert_nix_store_bin_shape(store_bin_dir: &Path) -> Result<()> {
    ensure!(
        store_bin_dir.file_name() == Some(OsStr::new("bin")),
        "store bin dir {store_bin_dir:?} basename is not `bin`"
    );
    let output_dir = store_bin_dir
        .parent()
        .with_context(|| format!("store bin dir {store_bin_dir:?} has no parent output dir"))?;
    let store_root = output_dir.parent().with_context(|| {
        format!("store output dir {output_dir:?} has no parent (expected the Nix store root)")
    })?;
    let canonical_store_root = fs::canonicalize(NIX_STORE_ROOT)
        .with_context(|| format!("canonicalizing the Nix store root {NIX_STORE_ROOT:?}"))?;
    ensure!(
        store_root == canonical_store_root,
        "resolved bin dir {store_bin_dir:?} is not <{NIX_STORE_ROOT}>/<output>/bin \
         (its output dir must be an immediate child of {canonical_store_root:?})"
    );
    Ok(())
}

/// Resolve `name` under `store_bin_dir` to a canonical, executable regular file whose parent
/// is exactly `store_bin_dir` (so a symlink cannot smuggle in a binary from elsewhere).
fn resolve_executable(store_bin_dir: &Path, name: &str) -> Result<PathBuf> {
    let candidate = store_bin_dir.join(name);
    let resolved = fs::canonicalize(&candidate)
        .with_context(|| format!("resolving executable {candidate:?}"))?;
    let metadata =
        fs::metadata(&resolved).with_context(|| format!("stat-ing executable {resolved:?}"))?;
    ensure!(metadata.is_file(), "{resolved:?} is not a regular file");
    ensure!(
        metadata.permissions().mode() & 0o111 != 0,
        "{resolved:?} is not executable"
    );
    ensure!(
        resolved.parent() == Some(store_bin_dir),
        "{resolved:?} does not live directly in the store bin dir {store_bin_dir:?}"
    );
    Ok(resolved)
}

/// Run `<exe> --version` and return its exact stdout, requiring a clean exit and empty stderr
/// so the strict version grammar sees exactly the reported output.
fn run_version(exe: &Path) -> Result<String> {
    let output = Command::new(exe)
        .arg("--version")
        .output()
        .with_context(|| format!("running {exe:?} --version"))?;
    ensure!(
        output.status.success(),
        "{exe:?} --version exited unsuccessfully: {}",
        output.status
    );
    ensure!(
        output.stderr.is_empty(),
        "{exe:?} --version wrote to stderr: {:?}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout)
        .with_context(|| format!("{exe:?} --version stdout is not valid UTF-8"))
}
