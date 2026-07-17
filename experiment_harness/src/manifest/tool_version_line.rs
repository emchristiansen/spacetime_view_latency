//! The shared `spacetimedb tool version …; spacetimedb-lib version …;` version line.

use anyhow::{ensure, Context, Result};
use semver::Version;

/// The parsed tool/lib version line common to both binaries' `--version` output. The CLI
/// reports it verbatim as its third line; the standalone reports it after a leading
/// `spacetimedb ` prefix. Parsed strictly against the exact grammar
/// `spacetimedb tool version <tool>; spacetimedb-lib version <lib>;` so extra or
/// malformed text is rejected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ToolVersionLine {
    tool: Version,
    lib: Version,
}

impl ToolVersionLine {
    const PREFIX: &'static str = "spacetimedb tool version ";
    const SEP: &'static str = "; spacetimedb-lib version ";
    const SUFFIX: &'static str = ";";

    /// Parse the exact tool/lib version line.
    pub(crate) fn parse(line: &str) -> Result<Self> {
        let rest = line
            .strip_prefix(Self::PREFIX)
            .with_context(|| format!("version line missing {:?} prefix: {line:?}", Self::PREFIX))?;
        let (tool_str, rest) = rest
            .split_once(Self::SEP)
            .with_context(|| format!("version line missing {:?} separator: {line:?}", Self::SEP))?;
        let lib_str = rest.strip_suffix(Self::SUFFIX).with_context(|| {
            format!("version line missing trailing {:?}: {line:?}", Self::SUFFIX)
        })?;

        let tool = Version::parse(tool_str)
            .with_context(|| format!("tool version is not semver: {tool_str:?}"))?;
        let lib = Version::parse(lib_str)
            .with_context(|| format!("lib version is not semver: {lib_str:?}"))?;
        Ok(Self { tool, lib })
    }

    /// The `spacetimedb tool` semantic version.
    pub(crate) fn tool(&self) -> &Version {
        &self.tool
    }

    /// The `spacetimedb-lib` semantic version.
    pub(crate) fn lib(&self) -> &Version {
        &self.lib
    }

    /// Assert both reported versions equal `expected`.
    pub(crate) fn ensure_both(&self, expected: &Version) -> Result<()> {
        ensure!(
            &self.tool == expected && &self.lib == expected,
            "reported versions tool={} lib={} do not both equal expected {expected}",
            self.tool,
            self.lib
        );
        Ok(())
    }
}
