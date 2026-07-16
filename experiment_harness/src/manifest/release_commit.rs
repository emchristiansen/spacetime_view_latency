//! A parsed 20-byte git release commit (SHA-1).

use std::fmt;

use anyhow::{ensure, Context, Result};
use serde::{Serialize, Serializer};

/// A git release commit as reported on the `Commit:` line of `spacetimedb-cli
/// --version`. Stored as the decoded 20-byte SHA-1, so only a well-formed commit is
/// representable — a malformed, wrong-length, or non-lowercase-hex string cannot be
/// constructed. Parsed (not merely validated) here; compared against
/// `crate::params::EXPECTED_RELEASE_COMMIT` when `VerifiedCli` is built. The standalone
/// binary reports no commit, so no release-commit proof exists for it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ReleaseCommit([u8; 20]);

impl ReleaseCommit {
    /// Parse an exact 40-character *lowercase* hex commit into its 20 raw bytes.
    ///
    /// The input is the exact `Commit:`-line value — no surrounding whitespace is
    /// tolerated. `hex::decode` accepts uppercase, so lowercase is enforced explicitly
    /// before decoding to keep the canonical-form invariant honest.
    pub(crate) fn parse(raw: &str) -> Result<Self> {
        ensure!(
            raw.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)),
            "release commit must be exact lowercase hex: {raw:?}"
        );
        let bytes =
            hex::decode(raw).with_context(|| format!("release commit is not valid hex: {raw:?}"))?;
        let bytes: [u8; 20] = bytes
            .as_slice()
            .try_into()
            .with_context(|| format!("release commit is not 20 bytes (40 hex chars): {raw:?}"))?;
        Ok(Self(bytes))
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
