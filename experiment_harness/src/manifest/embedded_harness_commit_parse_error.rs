//! Why a candidate hex string is not a canonical embedded harness commit.

use std::fmt;

use crate::manifest::canonical_hex_shape_error::CanonicalHexShapeError;

/// The exhaustive typed reason a hex string is not a canonical embedded harness commit. A canonical
/// embedded harness commit is exactly
/// [`EmbeddedHarnessCommit::CANONICAL_HEX_LEN`](super::embedded_harness_commit::EmbeddedHarnessCommit::CANONICAL_HEX_LEN)
/// *lowercase* hex characters (a 20-byte SHA-1); this enumerates the mutually distinct ways that fails,
/// mirroring [`ReleaseCommitParseError`](super::release_commit_parse_error::ReleaseCommitParseError) as a
/// distinct domain type so an embedded-harness-commit failure can never be paired with the release-commit,
/// database-identity, or module-hash fact.
///
/// This is the embedded-harness-commit-specific *translation* of the shared [`CanonicalHexShapeError`]
/// (`From<CanonicalHexShapeError>`): the shape algorithm is single-sourced.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EmbeddedHarnessCommitParseError {
    /// The string is not the canonical embedded-harness-commit-hex length: `expected` is
    /// [`EmbeddedHarnessCommit::CANONICAL_HEX_LEN`](super::embedded_harness_commit::EmbeddedHarnessCommit::CANONICAL_HEX_LEN),
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

impl From<CanonicalHexShapeError> for EmbeddedHarnessCommitParseError {
    /// Translate the shared shape failure into the embedded-harness-commit-specific typed error, one
    /// variant to one.
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

impl fmt::Display for EmbeddedHarnessCommitParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Length { expected, observed } => write!(
                f,
                "embedded harness commit hex must be exactly {expected} lowercase hex characters, found \
                 {observed}"
            ),
            Self::NonHex {
                offending_index,
                offending_byte,
            } => write!(
                f,
                "embedded harness commit hex has a non-hexadecimal byte {offending_byte:#04x} at index \
                 {offending_index}"
            ),
            Self::NonCanonicalCase {
                offending_index,
                offending_byte,
            } => write!(
                f,
                "embedded harness commit hex has an uppercase (non-canonical) byte {offending_byte:#04x} \
                 at index {offending_index}; the canonical form is lowercase"
            ),
        }
    }
}
