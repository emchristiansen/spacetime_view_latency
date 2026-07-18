//! Why folding the untrusted wire records into the trusted campaign graph failed.

use std::fmt;

use crate::analysis::validate::integrity_error_category::IntegrityErrorCategory;

/// A total-validation failure: the ingested wire records do not form a structurally complete, internally
/// consistent campaign. This is the *semantic/completeness* boundary, categorically distinct from two
/// neighbours:
///
/// - the *syntactic* [`IngestError`](crate::analysis::ingest::ingest_error::IngestError), which is
///   already resolved before validation begins (a malformed line never reaches this pass), and
/// - the classify-time `InvalidControl` outcome — a structurally complete cell whose control fails the
///   preregistered equivalence gate — which is a *reportable per-cell result*, never an integrity error
///   (spec: "Make complete-campaign validation and statistical control validity distinct typed
///   outcomes"). An `IntegrityError` aborts report generation; an invalid control does not.
///
/// Each variant is classified by a closed [`IntegrityErrorCategory`] and carries a fail-loud diagnostic.
/// The type is `pub(crate)` so the report layer can name and match it; the `pub(super)` constructors are
/// the canonical minting path used by the `validate` subtree. Behavior-phase work replaces the
/// diagnostic-only payloads with coordinate-bearing variants (which cell/run/dose failed) so the failing
/// location is typed structure, not only diagnostic text.
#[derive(Debug)]
pub(crate) enum IntegrityError {
    /// Record sequence numbers are not the contiguous `0..N` set, or a sequence number repeats.
    SequenceNotContiguous { diagnostic: String },
    /// The manifests do not cover exactly the nine preregistered cells (one missing, duplicated, or extra).
    CellCensus { diagnostic: String },
    /// A cell does not carry exactly `REPETITION_BLOCKS` (30) blocks with both the arm and control roles.
    BlockCensus { diagnostic: String },
    /// A run does not carry doses `1..=NUM_DOSES` (1–10) each exactly once.
    DoseCensus { diagnostic: String },
    /// A run's recorded role disagrees with the arm/control position it was matched into.
    RunRoleMismatch { diagnostic: String },
    /// A record's kind-carried dose index disagrees with its body's dose coordinate.
    RecordCoordinateMismatch { diagnostic: String },
    /// A dose observation's manifest reference is dangling, or the references are not bijective.
    ManifestReferenceBinding { diagnostic: String },
    /// A dose observation's coordinate disagrees with the manifest it references.
    CoordinateReferenceMismatch { diagnostic: String },
    /// A campaign-stable provenance or parameter value is itself malformed or off the preregistered
    /// specification, independent of cross-run homogeneity.
    MalformedStableProvenance { diagnostic: String },
    /// A well-formed campaign-stable fact is not homogeneous across the campaign's runs. Per-run facts
    /// (pid, listen address, client URL, data/keys dirs, database identity) are *not* checked here — they
    /// legitimately vary and are validated only for shape, internal consistency, and reference binding.
    CampaignFactHeterogeneity { diagnostic: String },
    /// A per-run server-provenance fact has an invalid shape or relationship (nonzero pid, parseable
    /// listen address, `client_url == http://{listen_addr}`, sibling `data`/`keys` directories).
    ServerProvenanceShape { diagnostic: String },
    /// A run's cumulative logical dose ladder is not strictly monotonic increasing.
    NonMonotonicLadder { diagnostic: String },
    /// A matched block's arm and control logical-cardinality ladders disagree dose-for-dose.
    CardinalityLadderMismatch { diagnostic: String },
    /// A dose's recorded physical cardinalities do not equal the deterministic
    /// `CampaignDataset::physical_cardinalities` expectation for its run and dose.
    PhysicalCardinalityMismatch { diagnostic: String },
    /// A dose's raw latency vector does not carry exactly `BATCH_SIZE` samples.
    SampleCount { diagnostic: String },
    /// A dose's recorded median/IQR summary is not the exact R-1 recomputation of its raw latencies.
    SummaryRecomputationMismatch { diagnostic: String },
    /// A dose's event evidence violates the insert/delete/net-delta identity or the deterministic expected
    /// net delta for its role.
    EventIdentityViolation { diagnostic: String },
}

impl IntegrityError {
    /// Record sequence numbers are not the contiguous `0..N` set, or one repeats.
    pub(super) fn sequence_not_contiguous(diagnostic: String) -> Self {
        Self::SequenceNotContiguous { diagnostic }
    }

    /// The manifests do not cover exactly the nine preregistered cells.
    pub(super) fn cell_census(diagnostic: String) -> Self {
        Self::CellCensus { diagnostic }
    }

    /// A cell does not carry exactly 30 blocks with both roles.
    pub(super) fn block_census(diagnostic: String) -> Self {
        Self::BlockCensus { diagnostic }
    }

    /// A run does not carry doses 1–10 each exactly once.
    pub(super) fn dose_census(diagnostic: String) -> Self {
        Self::DoseCensus { diagnostic }
    }

    /// A run's recorded role disagrees with its matched arm/control position.
    pub(super) fn run_role_mismatch(diagnostic: String) -> Self {
        Self::RunRoleMismatch { diagnostic }
    }

    /// A record's kind-carried dose index disagrees with its body's dose coordinate.
    pub(super) fn record_coordinate_mismatch(diagnostic: String) -> Self {
        Self::RecordCoordinateMismatch { diagnostic }
    }

    /// A manifest reference is dangling or the references are not bijective.
    pub(super) fn manifest_reference_binding(diagnostic: String) -> Self {
        Self::ManifestReferenceBinding { diagnostic }
    }

    /// A dose observation's coordinate disagrees with its referenced manifest.
    pub(super) fn coordinate_reference_mismatch(diagnostic: String) -> Self {
        Self::CoordinateReferenceMismatch { diagnostic }
    }

    /// A campaign-stable provenance or parameter value is malformed or off the preregistered spec.
    pub(super) fn malformed_stable_provenance(diagnostic: String) -> Self {
        Self::MalformedStableProvenance { diagnostic }
    }

    /// A well-formed campaign-stable fact is not homogeneous across runs.
    pub(super) fn campaign_fact_heterogeneity(diagnostic: String) -> Self {
        Self::CampaignFactHeterogeneity { diagnostic }
    }

    /// A per-run server-provenance fact has an invalid shape or relationship.
    pub(super) fn server_provenance_shape(diagnostic: String) -> Self {
        Self::ServerProvenanceShape { diagnostic }
    }

    /// A run's cumulative logical dose ladder is not strictly monotonic.
    pub(super) fn non_monotonic_ladder(diagnostic: String) -> Self {
        Self::NonMonotonicLadder { diagnostic }
    }

    /// A matched block's arm and control cardinality ladders disagree.
    pub(super) fn cardinality_ladder_mismatch(diagnostic: String) -> Self {
        Self::CardinalityLadderMismatch { diagnostic }
    }

    /// A dose's recorded physical cardinalities disagree with the deterministic dataset expectation.
    pub(super) fn physical_cardinality_mismatch(diagnostic: String) -> Self {
        Self::PhysicalCardinalityMismatch { diagnostic }
    }

    /// A dose's raw latency vector does not carry exactly `BATCH_SIZE` samples.
    pub(super) fn sample_count(diagnostic: String) -> Self {
        Self::SampleCount { diagnostic }
    }

    /// A dose's summary is not the exact R-1 recomputation of its raw latencies.
    pub(super) fn summary_recomputation_mismatch(diagnostic: String) -> Self {
        Self::SummaryRecomputationMismatch { diagnostic }
    }

    /// A dose's event evidence violates the count identity or the deterministic expected net delta.
    pub(super) fn event_identity_violation(diagnostic: String) -> Self {
        Self::EventIdentityViolation { diagnostic }
    }

    /// The closed typed obligation category this failure belongs to.
    pub(crate) fn category(&self) -> IntegrityErrorCategory {
        match self {
            Self::SequenceNotContiguous { .. } => IntegrityErrorCategory::SequenceNotContiguous,
            Self::CellCensus { .. } => IntegrityErrorCategory::CellCensus,
            Self::BlockCensus { .. } => IntegrityErrorCategory::BlockCensus,
            Self::DoseCensus { .. } => IntegrityErrorCategory::DoseCensus,
            Self::RunRoleMismatch { .. } => IntegrityErrorCategory::RunRoleMismatch,
            Self::RecordCoordinateMismatch { .. } => IntegrityErrorCategory::RecordCoordinateMismatch,
            Self::ManifestReferenceBinding { .. } => IntegrityErrorCategory::ManifestReferenceBinding,
            Self::CoordinateReferenceMismatch { .. } => {
                IntegrityErrorCategory::CoordinateReferenceMismatch
            }
            Self::MalformedStableProvenance { .. } => {
                IntegrityErrorCategory::MalformedStableProvenance
            }
            Self::CampaignFactHeterogeneity { .. } => {
                IntegrityErrorCategory::CampaignFactHeterogeneity
            }
            Self::ServerProvenanceShape { .. } => IntegrityErrorCategory::ServerProvenanceShape,
            Self::NonMonotonicLadder { .. } => IntegrityErrorCategory::NonMonotonicLadder,
            Self::CardinalityLadderMismatch { .. } => {
                IntegrityErrorCategory::CardinalityLadderMismatch
            }
            Self::PhysicalCardinalityMismatch { .. } => {
                IntegrityErrorCategory::PhysicalCardinalityMismatch
            }
            Self::SampleCount { .. } => IntegrityErrorCategory::SampleCount,
            Self::SummaryRecomputationMismatch { .. } => {
                IntegrityErrorCategory::SummaryRecomputationMismatch
            }
            Self::EventIdentityViolation { .. } => IntegrityErrorCategory::EventIdentityViolation,
        }
    }

    /// The fail-loud diagnostic for this failure.
    pub(crate) fn diagnostic(&self) -> &str {
        match self {
            Self::SequenceNotContiguous { diagnostic }
            | Self::CellCensus { diagnostic }
            | Self::BlockCensus { diagnostic }
            | Self::DoseCensus { diagnostic }
            | Self::RunRoleMismatch { diagnostic }
            | Self::RecordCoordinateMismatch { diagnostic }
            | Self::ManifestReferenceBinding { diagnostic }
            | Self::CoordinateReferenceMismatch { diagnostic }
            | Self::MalformedStableProvenance { diagnostic }
            | Self::CampaignFactHeterogeneity { diagnostic }
            | Self::ServerProvenanceShape { diagnostic }
            | Self::NonMonotonicLadder { diagnostic }
            | Self::CardinalityLadderMismatch { diagnostic }
            | Self::PhysicalCardinalityMismatch { diagnostic }
            | Self::SampleCount { diagnostic }
            | Self::SummaryRecomputationMismatch { diagnostic }
            | Self::EventIdentityViolation { diagnostic } => diagnostic,
        }
    }
}

impl fmt::Display for IntegrityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "campaign integrity violation ({:?}): {}",
            self.category(),
            self.diagnostic()
        )
    }
}
