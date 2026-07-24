//! A parsed 20-byte Git commit embedded into the binary at build time.

use std::fmt;

use anyhow::{anyhow, Result};
use serde::{Serialize, Serializer};

use crate::manifest::canonical_hex_shape_error::CanonicalHexShapeError;
use crate::manifest::embedded_harness_commit_parse_error::EmbeddedHarnessCommitParseError;

/// The harness checkout's commit as embedded by `build.rs` (`HARNESS_BUILD_COMMIT`), or as recorded
/// verbatim in a frozen `ValidatedRunManifest`. Stored as the decoded 20-byte SHA-1, so only a
/// well-formed commit is representable — mirroring
/// [`ReleaseCommit`](super::release_commit::ReleaseCommit). Available for diagnostic checkout
/// comparison in every build, authoritative or not (spec: "a parsed 20-byte Git commit newtype
/// available for diagnostic checkout comparison in every build"); it carries no authorization by
/// itself — that is [`HarnessCommit`](crate::manifest::harness_commit::HarnessCommit)'s sole role.
///
/// This type is the **single owner** of the canonical embedded-harness-commit-hex format, single-sourced
/// through [`CanonicalHexShapeError::check`] exactly as `ReleaseCommit` is, so provenance parsing and any
/// future re-parse both go through [`Self::parse_canonical_hex`] and cannot drift.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EmbeddedHarnessCommit([u8; 20]);

impl EmbeddedHarnessCommit {
    /// Canonical embedded-harness-commit hex length: a 20-byte SHA-1 as lowercase hex.
    pub(crate) const CANONICAL_HEX_LEN: usize = 40;

    /// The exhaustive typed reason `raw` is not a canonical embedded harness commit, or `None` if it is.
    /// Delegates the shape rule to the shared [`CanonicalHexShapeError::check`] and translates its failure
    /// into the domain-specific typed error, so the algorithm is single-sourced; it does not decode.
    pub(crate) fn canonical_hex_error(raw: &str) -> Option<EmbeddedHarnessCommitParseError> {
        CanonicalHexShapeError::check(raw, Self::CANONICAL_HEX_LEN)
            .err()
            .map(EmbeddedHarnessCommitParseError::from)
    }

    /// Parse a canonical lowercase-hex commit into its 20 raw bytes, or the exhaustive typed reason it is
    /// not canonical. A string that passes the canonical shape check always decodes into exactly 20 bytes,
    /// so a post-check decode failure is an invariant violation, not a recoverable error.
    pub(crate) fn parse_canonical_hex(
        raw: &str,
    ) -> std::result::Result<Self, EmbeddedHarnessCommitParseError> {
        if let Some(error) = Self::canonical_hex_error(raw) {
            return Err(error);
        }
        let bytes = hex::decode(raw)
            .expect("a canonical lowercase-hex string of CANONICAL_HEX_LEN always decodes");
        let bytes: [u8; 20] = bytes
            .as_slice()
            .try_into()
            .expect("CANONICAL_HEX_LEN / 2 == 20 bytes for a canonical embedded harness commit");
        Ok(Self(bytes))
    }

    /// Parse an exact 40-character *lowercase* hex commit into its 20 raw bytes, surfacing the typed
    /// [`EmbeddedHarnessCommitParseError`] as an `anyhow` error at this `anyhow`-based provenance boundary.
    pub(crate) fn parse(raw: &str) -> Result<Self> {
        Self::parse_canonical_hex(raw).map_err(|error| {
            anyhow!("embedded harness commit is not canonical lowercase hex: {raw:?}: {error}")
        })
    }

    /// The raw 20-byte SHA-1.
    pub(crate) fn bytes(&self) -> &[u8; 20] {
        &self.0
    }
}

impl fmt::Display for EmbeddedHarnessCommit {
    /// Canonical lowercase 40-character hex.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

impl Serialize for EmbeddedHarnessCommit {
    /// Serialized as canonical lowercase 40-character hex.
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}
