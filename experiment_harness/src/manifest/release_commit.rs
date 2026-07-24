//! A parsed 20-byte git release commit (SHA-1).

use std::fmt;

use anyhow::{anyhow, Result};
use serde::{Serialize, Serializer};

use crate::manifest::canonical_hex_shape_error::CanonicalHexShapeError;
use crate::manifest::release_commit_parse_error::ReleaseCommitParseError;

/// A git release commit as reported on the `Commit:` line of `spacetimedb-cli
/// --version`. Stored as the decoded 20-byte SHA-1, so only a well-formed commit is
/// representable — a malformed, wrong-length, or non-lowercase-hex string cannot be
/// constructed. Parsed (not merely validated) here; compared against
/// `crate::params::EXPECTED_RELEASE_COMMIT` when `VerifiedCli` is built. The standalone
/// binary reports no commit, so no release-commit proof exists for it.
///
/// This type is the **single owner** of the canonical release-commit-hex format. Its shape rule
/// (fixed length, lowercase hex) is single-sourced through the shared
/// [`CanonicalHexShapeError::check`], which this owner *translates* into its own typed
/// [`ReleaseCommitParseError`]; the provisioning path (`--version` output) and any future re-parse both
/// go through [`Self::parse_canonical_hex`], so they cannot drift.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ReleaseCommit([u8; 20]);

impl ReleaseCommit {
    /// Canonical release-commit hex length: a 20-byte SHA-1 as lowercase hex.
    pub(crate) const CANONICAL_HEX_LEN: usize = 40;

    /// The exhaustive typed reason `raw` is not a canonical release commit, or `None` if it is. Delegates
    /// the shape rule to the shared [`CanonicalHexShapeError::check`] and translates its failure into the
    /// release-commit-specific typed error, so the algorithm is single-sourced; it does not decode.
    pub(crate) fn canonical_hex_error(raw: &str) -> Option<ReleaseCommitParseError> {
        CanonicalHexShapeError::check(raw, Self::CANONICAL_HEX_LEN)
            .err()
            .map(ReleaseCommitParseError::from)
    }

    /// Parse a canonical lowercase-hex commit into its 20 raw bytes, or the exhaustive typed reason it is
    /// not canonical. A string that passes the canonical shape check always decodes into exactly 20 bytes,
    /// so a post-check decode failure is an invariant violation, not a recoverable error.
    pub(crate) fn parse_canonical_hex(raw: &str) -> std::result::Result<Self, ReleaseCommitParseError> {
        if let Some(error) = Self::canonical_hex_error(raw) {
            return Err(error);
        }
        let bytes = hex::decode(raw)
            .expect("a canonical lowercase-hex string of CANONICAL_HEX_LEN always decodes");
        let bytes: [u8; 20] = bytes
            .as_slice()
            .try_into()
            .expect("CANONICAL_HEX_LEN / 2 == 20 bytes for a canonical release commit");
        Ok(Self(bytes))
    }

    /// Parse an exact 40-character *lowercase* hex commit into its 20 raw bytes, surfacing the typed
    /// [`ReleaseCommitParseError`] as an `anyhow` error at this `anyhow`-based provisioning boundary.
    ///
    /// The input is the exact `Commit:`-line value — no surrounding whitespace is tolerated.
    pub(crate) fn parse(raw: &str) -> Result<Self> {
        Self::parse_canonical_hex(raw)
            .map_err(|error| anyhow!("release commit is not canonical lowercase hex: {raw:?}: {error}"))
    }

    /// The raw 20-byte SHA-1.
    pub(crate) fn bytes(&self) -> &[u8; 20] {
        &self.0
    }
}

impl fmt::Display for ReleaseCommit {
    /// Canonical lowercase 40-character hex.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

impl Serialize for ReleaseCommit {
    /// Serialized as canonical lowercase 40-character hex.
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

#[cfg(test)]
mod tests;
