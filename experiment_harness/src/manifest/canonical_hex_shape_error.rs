//! The one shared internal validator for canonical fixed-length lowercase-hex shape.

use std::fmt;

/// The exhaustive typed reason a candidate string is not canonical fixed-length lowercase hex, plus
/// [`Self::check`] — the single internal implementation of that shape rule.
///
/// This type is deliberately *shape-only* and semantically anonymous: it says a string is the wrong
/// length or has an offending byte, but names no domain. Each semantic domain that is canonical hex
/// ([`DatabaseIdentity`](super::database_identity::DatabaseIdentity),
/// [`ReleaseCommit`](super::release_commit::ReleaseCommit), and
/// [`WasmSha256`](super::wasm_sha256::WasmSha256)) owns its own typed parse error and *translates* this
/// shared failure into it (`From<CanonicalHexShapeError>`), so the fallible algorithm is single-sourced
/// while a release-commit failure can never be paired with the database-identity fact, or vice versa
/// (spec: "Make total-validation ownership and failure locations explicit"). This shared type is
/// therefore never itself carried in an integrity variant — only the domain-specific translations are.
///
/// Exactly one mode is reported: a length mismatch first (it makes per-character positions meaningless),
/// otherwise the *first offending byte in index order*. Each byte is unambiguously one category or the
/// other — [`NonHex`](Self::NonHex) (not a hexadecimal digit at all) and
/// [`NonCanonicalCase`](Self::NonCanonicalCase) (an uppercase hex digit) are disjoint per byte — so there
/// is no precedence between them: the reported byte is simply the earliest that is not a canonical
/// lowercase-hex character, tagged with the category it falls into.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CanonicalHexShapeError {
    /// The string is not the required canonical hex length: `expected` is the domain's fixed length,
    /// `observed` is the input's length.
    Length { expected: usize, observed: usize },
    /// The first byte that is not a hexadecimal digit at all (neither `0-9`, `a-f`, nor `A-F`), by
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

impl CanonicalHexShapeError {
    /// Validate that `hex` is exactly `expected_len` canonical lowercase-hex characters, returning the
    /// first shape failure or `Ok(())`. Shape only: it does not decode the hex into bytes — a string that
    /// passes is guaranteed decodable into exactly `expected_len / 2` bytes, so each domain's post-check
    /// decode is an infallible invariant, not a recoverable error.
    pub(crate) fn check(hex: &str, expected_len: usize) -> Result<(), Self> {
        if hex.len() != expected_len {
            return Err(Self::Length {
                expected: expected_len,
                observed: hex.len(),
            });
        }
        for (offending_index, offending_byte) in hex.bytes().enumerate() {
            if offending_byte.is_ascii_digit() || (b'a'..=b'f').contains(&offending_byte) {
                continue;
            }
            if (b'A'..=b'F').contains(&offending_byte) {
                return Err(Self::NonCanonicalCase {
                    offending_index,
                    offending_byte,
                });
            }
            return Err(Self::NonHex {
                offending_index,
                offending_byte,
            });
        }
        Ok(())
    }
}

impl fmt::Display for CanonicalHexShapeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Length { expected, observed } => write!(
                f,
                "must be exactly {expected} lowercase hex characters, found {observed}"
            ),
            Self::NonHex {
                offending_index,
                offending_byte,
            } => write!(
                f,
                "has a non-hexadecimal byte {offending_byte:#04x} at index {offending_index}"
            ),
            Self::NonCanonicalCase {
                offending_index,
                offending_byte,
            } => write!(
                f,
                "has an uppercase (non-canonical) byte {offending_byte:#04x} at index \
                 {offending_index}; the canonical form is lowercase"
            ),
        }
    }
}
