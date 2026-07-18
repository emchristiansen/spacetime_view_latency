//! Why a candidate hex string is not a canonical SpacetimeDB database identity.

use std::fmt;

use crate::manifest::canonical_hex_shape_error::CanonicalHexShapeError;

/// The exhaustive typed reason a hex string is not a canonical SpacetimeDB identity. A canonical
/// identity is exactly [`DatabaseIdentity::CANONICAL_HEX_LEN`](super::database_identity::DatabaseIdentity::CANONICAL_HEX_LEN)
/// *lowercase* hex characters; this enumerates the mutually distinct ways that fails, each with typed
/// positional evidence, so a length-correct but non-hex or uppercase input is located precisely rather
/// than claimed from length alone. The rejected structure is retained as typed data, not only as
/// diagnostic text.
///
/// This is the database-identity-specific *translation* of the shared
/// [`CanonicalHexShapeError`] (`From<CanonicalHexShapeError>`): the shape algorithm is single-sourced,
/// but this remains a distinct domain type so an identity failure can never be paired with the
/// release-commit or module-hash fact.
///
/// Exactly one mode is reported: a length mismatch first (it makes per-character positions meaningless),
/// otherwise the *first offending byte in index order*. [`NonHex`](Self::NonHex) (not a hexadecimal digit
/// at all) and [`NonCanonicalCase`](Self::NonCanonicalCase) (an uppercase hex digit) are disjoint per
/// byte, so there is no precedence between them: the reported byte is simply the earliest that is not a
/// canonical lowercase-hex character, tagged with the category it falls into.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DatabaseIdentityParseError {
    /// The string is not the canonical identity-hex length: `expected` is
    /// [`DatabaseIdentity::CANONICAL_HEX_LEN`](super::database_identity::DatabaseIdentity::CANONICAL_HEX_LEN),
    /// `observed` is the input's length.
    Length { expected: usize, observed: usize },
    /// The first character that is not a hexadecimal digit at all (neither `0-9`, `a-f`, nor `A-F`), by
    /// zero-based byte index and raw byte value.
    NonHex {
        offending_index: usize,
        offending_byte: u8,
    },
    /// The first hexadecimal digit that is uppercase (`A-F`) — valid hex but not the canonical lowercase
    /// form — by zero-based byte index and raw byte value.
    NonCanonicalCase {
        offending_index: usize,
        offending_byte: u8,
    },
}

impl From<CanonicalHexShapeError> for DatabaseIdentityParseError {
    /// Translate the shared shape failure into the identity-specific typed error, one variant to one.
    fn from(shape: CanonicalHexShapeError) -> Self {
        match shape {
            CanonicalHexShapeError::Length { expected, observed } => {
                Self::Length { expected, observed }
            }
            CanonicalHexShapeError::NonHex {
                offending_index,
                offending_byte,
            } => Self::NonHex {
                offending_index,
                offending_byte,
            },
            CanonicalHexShapeError::NonCanonicalCase {
                offending_index,
                offending_byte,
            } => Self::NonCanonicalCase {
                offending_index,
                offending_byte,
            },
        }
    }
}

impl fmt::Display for DatabaseIdentityParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Length { expected, observed } => write!(
                f,
                "database identity hex must be exactly {expected} lowercase hex characters, found {observed}"
            ),
            Self::NonHex {
                offending_index,
                offending_byte,
            } => write!(
                f,
                "database identity hex has a non-hexadecimal byte {offending_byte:#04x} at index {offending_index}"
            ),
            Self::NonCanonicalCase {
                offending_index,
                offending_byte,
            } => write!(
                f,
                "database identity hex has an uppercase (non-canonical) byte {offending_byte:#04x} at \
                 index {offending_index}; the canonical form is lowercase"
            ),
        }
    }
}
