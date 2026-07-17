//! Parsed, version-verified `spacetimedb-standalone --version` proof.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use semver::Version;

use crate::manifest::strict_version_output::strict_version_lines;
use crate::manifest::tool_version_line::ToolVersionLine;
use crate::params::EXPECTED_VERSION;

/// Proof that a specific executable is the expected `spacetimedb-standalone`. Constructed
/// only by parsing the exact single-line `--version` grammar and checking the reported
/// version against [`EXPECTED_VERSION`]. The exact raw output and absolute executable
/// path are retained because the spec records both. The standalone reports **no** commit,
/// so — unlike [`crate::manifest::verified_cli::VerifiedCli`] — this proof cannot carry
/// one.
#[derive(Debug, Clone)]
pub(crate) struct VerifiedStandalone {
    exe: PathBuf,
    raw: String,
    version: Version,
}

impl VerifiedStandalone {
    const PREFIX: &'static str = "spacetimedb ";

    /// Parse and verify the full `spacetimedb-standalone --version` output for `exe`.
    ///
    /// Grammar (exactly one line, no commit):
    /// ```text
    /// spacetimedb spacetimedb tool version <v>; spacetimedb-lib version <v>;
    /// ```
    pub(crate) fn parse(exe: &Path, raw: &str) -> Result<Self> {
        let lines = strict_version_lines(raw, 1)?;

        let core = lines[0].strip_prefix(Self::PREFIX).with_context(|| {
            format!(
                "standalone --version missing {:?} prefix: {:?}",
                Self::PREFIX,
                lines[0]
            )
        })?;
        let versions = ToolVersionLine::parse(core)?;

        let expected_version = Version::parse(EXPECTED_VERSION)
            .with_context(|| format!("EXPECTED_VERSION is not semver: {EXPECTED_VERSION:?}"))?;
        versions.ensure_both(&expected_version)?;

        Ok(Self {
            exe: exe.to_path_buf(),
            raw: raw.to_string(),
            version: versions.tool().clone(),
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
}
