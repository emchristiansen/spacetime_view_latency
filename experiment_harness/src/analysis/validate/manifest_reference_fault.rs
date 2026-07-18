//! The typed contradiction behind a
//! [`ManifestReferenceBinding`](super::integrity_error::IntegrityError) failure.

use crate::analysis::validate::manifest_reference_identity::ManifestReferenceIdentity;
use crate::manifest::database_identity_parse_error::DatabaseIdentityParseError;
use crate::manifest::run_coordinate::RunCoordinate;

/// Why the manifest-reference binding between manifests and dose observations failed. Dose observations
/// are **many-to-one** to manifests: each manifest is referenced by its complete canonical dose set
/// (`NUM_DOSES` observations), so it is only the *distinct identity-key sets* that must correspond
/// one-to-one — the set of manifest identity keys must equal the set of referenced identity keys, and
/// the manifest→identity map must be injective. Each mode carries typed identity components plus typed
/// occurrence/match counts locating the break; a repeated *reference* is expected and never a fault.
#[derive(Debug)]
pub(crate) enum ManifestReferenceFault {
    /// A manifest's or observation's database-identity hex is not a canonical identity, so no trusted
    /// identity — hence no binding key — can be formed. `hex` is the raw wire string precisely because
    /// parsing it into an identity is the check that failed; `error` is the domain-owned exhaustive
    /// typed reason (wrong length, or the first non-hex or non-canonical-case character), so the defect
    /// is never claimed from length alone.
    MalformedIdentity {
        run: RunCoordinate,
        hex: String,
        error: DatabaseIdentityParseError,
    },
    /// A manifest body's embedded self-reference disagrees with the identity derived from its own
    /// embedded manifest facts (a reference-binding failure, not a coordinate mismatch).
    EmbeddedDisagreement {
        reference: ManifestReferenceIdentity,
        manifest: ManifestReferenceIdentity,
    },
    /// Two or more manifests share one identity key — the manifest→identity map is not injective:
    /// `expected_occurrences` is 1, `observed_occurrences` is the actual manifest count for that key.
    DuplicateManifest {
        identity: ManifestReferenceIdentity,
        expected_occurrences: usize,
        observed_occurrences: usize,
    },
    /// An observation references an identity no manifest provides: `expected_manifest_matches` is 1,
    /// `observed_manifest_matches` is 0.
    Dangling {
        reference: ManifestReferenceIdentity,
        expected_manifest_matches: usize,
        observed_manifest_matches: usize,
    },
    /// A manifest identity key is absent from the reference set — no observation references it, so the
    /// reference→manifest map is not surjective: `expected_reference_occurrences` is the complete
    /// canonical dose set (`NUM_DOSES`) that a correct campaign references it with, `observed_reference_occurrences`
    /// is 0. This is specifically the zero-reference orphan case; a manifest referenced by *some but not
    /// all* of its doses has its key present in the reference set (so it is not unreferenced) and its
    /// missing doses are caught by the dose census, not here.
    Unreferenced {
        manifest: ManifestReferenceIdentity,
        expected_reference_occurrences: usize,
        observed_reference_occurrences: usize,
    },
}
