//! Which hash a recorded digest was taken with.

use serde::Serialize;

/// The hash function a recorded digest was produced by.
///
/// A closed enum so a later algorithm is a variant every reader must handle, rather than an
/// unrecognized string that silently reads as "some hash". Carried alongside every digest this
/// module records, because a bare hex string becomes unauditable the moment the algorithm changes.
///
/// Freely constructible, and harmlessly so: naming an algorithm asserts nothing about what was
/// hashed. The claims live in the types that carry it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) enum DigestAlgorithm {
    Sha256,
}
