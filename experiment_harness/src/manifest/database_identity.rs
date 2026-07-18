//! The published database identity.

use serde::{Serialize, Serializer};
use spacetimedb_sdk::Identity;

use crate::manifest::canonical_hex_shape_error::CanonicalHexShapeError;
use crate::manifest::database_identity_parse_error::DatabaseIdentityParseError;

/// The SpacetimeDB identity of the published experiment database, recorded in the run
/// manifest. Wraps the SDK's own `Identity` type rather than a hex string, so the value
/// carries its domain meaning; serialized as canonical hex.
///
/// This type is the **single owner** of the canonical identity-hex format: what counts as a canonical
/// identity (length and lowercase-hex) and how it is parsed live here, in [`Self::CANONICAL_HEX_LEN`],
/// [`Self::canonical_hex_error`], and [`Self::parse_canonical_hex`]. Both the provisioning path (which
/// reads the published identity off `spacetimedb-cli` stdout) and the analysis validation path (which
/// re-parses recorded identities) parse through [`Self::parse_canonical_hex`], so the two cannot drift.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DatabaseIdentity(Identity);

impl DatabaseIdentity {
    /// Canonical identity hex length: a 32-byte SpacetimeDB identity as lowercase hex.
    pub(crate) const CANONICAL_HEX_LEN: usize = 64;

    pub(crate) fn new(identity: Identity) -> Self {
        Self(identity)
    }

    /// The published database identity.
    pub(crate) fn identity(&self) -> &Identity {
        &self.0
    }

    /// The exhaustive typed reason `hex` is not canonical, or `None` if it is. Delegates the shape rule
    /// to the shared [`CanonicalHexShapeError::check`] and translates its failure into the
    /// identity-specific typed error, so the algorithm is single-sourced; it does not decode.
    pub(crate) fn canonical_hex_error(hex: &str) -> Option<DatabaseIdentityParseError> {
        CanonicalHexShapeError::check(hex, Self::CANONICAL_HEX_LEN)
            .err()
            .map(DatabaseIdentityParseError::from)
    }

    /// Parse a canonical lowercase-hex identity into a trusted [`DatabaseIdentity`], or the exhaustive
    /// typed reason it is not canonical. A string that passes the canonical shape check always decodes,
    /// so a post-check decode failure is an invariant violation, not a recoverable error.
    pub(crate) fn parse_canonical_hex(hex: &str) -> Result<Self, DatabaseIdentityParseError> {
        if let Some(error) = Self::canonical_hex_error(hex) {
            return Err(error);
        }
        let identity = Identity::from_hex(hex).expect(
            "a canonical lowercase-hex string of CANONICAL_HEX_LEN always decodes into an Identity",
        );
        Ok(Self(identity))
    }
}

impl Serialize for DatabaseIdentity {
    /// Serialized as the canonical lowercase hex identity.
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0.to_hex().to_string())
    }
}

#[cfg(test)]
mod tests;
