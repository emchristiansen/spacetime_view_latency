//! Parsed, version-and-commit-verified `spacetimedb-cli --version` proof.

use std::path::{Path, PathBuf};

use anyhow::{ensure, Context, Result};
use semver::Version;

use crate::manifest::release_commit::ReleaseCommit;
use crate::manifest::strict_version_output::strict_version_lines;
use crate::manifest::tool_version_line::ToolVersionLine;
use crate::params::{EXPECTED_RELEASE_COMMIT, EXPECTED_VERSION};

/// Proof that a specific executable is the expected `spacetimedb-cli`. Constructed only
/// by parsing the exact three-line `--version` grammar and checking the reported version
/// and release commit against [`EXPECTED_VERSION`]/[`EXPECTED_RELEASE_COMMIT`]. The exact
/// raw output and absolute executable path are retained because the spec records both.
/// Only the CLI reports a commit, so this proof — unlike the standalone's — carries one.
#[derive(Debug, Clone)]
pub(crate) struct VerifiedCli {
    exe: PathBuf,
    raw: String,
    reported_path: PathBuf,
    version: Version,
    commit: ReleaseCommit,
}

impl VerifiedCli {
    /// Parse and verify the full `spacetimedb-cli --version` output for `exe`.
    ///
    /// Grammar (exactly three lines):
    /// ```text
    /// spacetime Path: <path>
    /// Commit: <40-hex>
    /// spacetimedb tool version <v>; spacetimedb-lib version <v>;
    /// ```
    pub(crate) fn parse(exe: &Path, raw: &str) -> Result<Self> {
        let lines = strict_version_lines(raw, 3)?;

        let reported_path = lines[0]
            .strip_prefix("spacetime Path: ")
            .with_context(|| format!("cli --version line 1 malformed: {:?}", lines[0]))?;
        let commit_hex = lines[1]
            .strip_prefix("Commit: ")
            .with_context(|| format!("cli --version line 2 malformed: {:?}", lines[1]))?;
        let versions = ToolVersionLine::parse(lines[2])?;

        let expected_version = Version::parse(EXPECTED_VERSION)
            .with_context(|| format!("EXPECTED_VERSION is not semver: {EXPECTED_VERSION:?}"))?;
        versions.ensure_both(&expected_version)?;

        let commit = ReleaseCommit::parse(commit_hex)?;
        let expected_commit = ReleaseCommit::parse(EXPECTED_RELEASE_COMMIT)
            .context("EXPECTED_RELEASE_COMMIT is not valid")?;
        ensure!(
            commit == expected_commit,
            "cli release commit {commit} != expected {expected_commit}"
        );

        let reported_path = PathBuf::from(reported_path);
        ensure!(
            reported_path == exe,
            "cli reports its path as {reported_path:?}, not the invoked {exe:?}"
        );

        Ok(Self {
            exe: exe.to_path_buf(),
            raw: raw.to_string(),
            reported_path,
            version: versions.tool().clone(),
            commit,
        })
    }

    /// The absolute executable path.
    pub(crate) fn exe(&self) -> &Path {
        &self.exe
    }

    /// The exact raw `--version` output.
    pub(crate) fn raw(&self) -> &str {
        &self.raw
    }

    /// The verified semantic version.
    pub(crate) fn version(&self) -> &Version {
        &self.version
    }

    /// The verified release commit.
    pub(crate) fn commit(&self) -> &ReleaseCommit {
        &self.commit
    }
}
