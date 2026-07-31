//! The accepted generated-tree digest, with its construction recipe.

use anyhow::{ensure, Result};
use serde::Serialize;

use crate::indexed_sender_view_calibration_pilot::calibration_params::{
    GENERATED_TREE_RECIPE, GENERATED_TREE_SHA256_HEX,
};

/// The digest of the generated binding tree the pinned module was produced from, carried with the
/// exact recipe that produced it.
///
/// The recipe travels with the digest deliberately: a tree hash is unreproducible without its root,
/// sort locale, and manifest-stream order.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct GeneratedTreeDigest {
    sha256_hex: String,
    recipe: &'static str,
}

impl GeneratedTreeDigest {
    /// The digest accepted at `ControlRegistry` Step 1, which this pilot measures against unchanged.
    pub(crate) fn accepted() -> Result<Self> {
        Self::parse_canonical_hex(GENERATED_TREE_SHA256_HEX)
    }

    /// Parse a canonical lowercase-hex SHA-256 digest.
    ///
    /// Validated rather than trusted: the constant is a hand-transcribed hex string, so a malformed
    /// digest fails loudly at startup instead of being written into every record of a completed run.
    fn parse_canonical_hex(hex: &str) -> Result<Self> {
        ensure!(
            hex.len() == 64,
            "a SHA-256 digest is 64 hex characters, got {}",
            hex.len()
        );
        ensure!(
            hex.chars()
                .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c)),
            "a canonical digest is lowercase hex, got {hex:?}"
        );
        Ok(Self {
            sha256_hex: hex.to_string(),
            recipe: GENERATED_TREE_RECIPE,
        })
    }
}
