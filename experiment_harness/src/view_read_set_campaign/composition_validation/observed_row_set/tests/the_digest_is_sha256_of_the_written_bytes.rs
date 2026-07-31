//! The recorded digest is SHA-256 of exactly the retained file, with no unrecorded framing.

use std::fs;

use sha2::{Digest, Sha256};

use crate::view_read_set_campaign::composition_validation::digest_algorithm::DigestAlgorithm;
use crate::view_read_set_campaign::composition_validation::observed_row_set::{
    ObservedRowSet, OBSERVED_ROW_SET_DOMAIN,
};
use crate::view_read_set_campaign::composition_validation::observed_row_set_label::ObservedRowSetLabel;

use super::attempt_directory::attempt_directory;
use super::row::row;

/// Coverage: the recomputation contract is the artifact path, the digest, the row count, the
/// algorithm, and the domain — and it is only a contract if a reader holding the retained file can
/// reproduce the digest with a stock tool. Any prefix, separator, or domain folded into the preimage
/// would make `sha256sum` of the artifact disagree with the recorded value, and the reader would
/// have no way to tell an unrecorded framing apart from a corrupted file.
///
/// So this recomputes the digest from the bytes *on disk*, not from the buffer the constructor
/// hashed. That distinction is the whole test: hashing the same in-memory string again would prove
/// only that SHA-256 is deterministic, whereas reading the file back also establishes that the bytes
/// written are the bytes hashed.
///
/// The domain is asserted to be the recorded label it claims to be, and deliberately *not* found in
/// the preimage: it selects the parser a reader uses, and folding it into the hash is exactly the
/// mistake this checks against.
#[test]
fn the_digest_is_sha256_of_the_written_bytes() {
    let directory = attempt_directory("digest-addresses-bytes");

    let observed = ObservedRowSet::persisted(
        &directory,
        ObservedRowSetLabel::AfterSaturatedBatch,
        vec![
            row(
                0,
                "view-read-set-campaign-mutation:e1-saturated-queue-growth:990",
            ),
            row(
                9,
                "view-read-set-campaign-mutation:e1-saturated-queue-growth:999",
            ),
        ],
    )
    .expect("distinct keys and line-break-free records persist");

    let bytes = fs::read(observed.path()).expect("reading back the retained artifact");
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let recomputed: [u8; 32] = hasher.finalize().into();

    let (algorithm, domain, recorded_hex, row_count) = observed.content_address();
    assert_eq!(
        recorded_hex,
        hex::encode(recomputed),
        "the recorded digest must be reproducible by hashing the retained file itself, with no \
         unrecorded prefix or separator"
    );
    assert_eq!(
        algorithm,
        DigestAlgorithm::Sha256,
        "the recorded algorithm must name the hash actually used"
    );
    assert_eq!(
        domain, OBSERVED_ROW_SET_DOMAIN,
        "the recorded domain names the encoding convention a reader parses with"
    );
    assert_eq!(
        row_count, 2,
        "the recorded row count is the number of rows persisted"
    );
    assert_eq!(
        bytes.iter().filter(|byte| **byte == b'\n').count(),
        usize::try_from(row_count).expect("a two-row artifact's count fits usize"),
        "wc -l on the artifact must equal the recorded row count"
    );
}
