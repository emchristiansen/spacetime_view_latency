//! The closed, typed classification of campaign-integrity obligations.

/// Which preregistered integrity obligation an
/// [`IntegrityError`](super::integrity_error::IntegrityError) violated, as a closed typed value rather
/// than a bare category string. A report maps this typed value to a stable serialized tag in exactly one
/// place, and adding or renaming an obligation is a single exhaustive-match change, never a scattered
/// literal that could silently drift from the error surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum IntegrityErrorCategory {
    /// Record sequence numbers are not the contiguous `0..N` set, or one repeats.
    SequenceNotContiguous,
    /// The manifests do not cover exactly the nine preregistered cells.
    CellCensus,
    /// A cell does not carry exactly 30 blocks with both the arm and control roles.
    BlockCensus,
    /// A run does not carry doses 1–10 each exactly once.
    DoseCensus,
    /// A run's recorded role disagrees with the arm/control position it was matched into.
    RunRoleMismatch,
    /// A record's kind-carried dose index disagrees with its body's dose coordinate (a semantic
    /// agreement beyond the syntactic kind→body dispatch).
    RecordCoordinateMismatch,
    /// A dose observation's manifest reference is dangling, or the references are not bijective.
    ManifestReferenceBinding,
    /// A dose observation's coordinate disagrees with the manifest it references.
    CoordinateReferenceMismatch,
    /// A campaign-stable provenance or parameter value is itself malformed or off the preregistered
    /// specification (bad hex/semver, wrong version/commit, a parameter not equal to its preregistered
    /// value) — independent of whether it is homogeneous across runs.
    MalformedStableProvenance,
    /// A well-formed campaign-stable fact is not homogeneous across the campaign's runs.
    CampaignFactHeterogeneity,
    /// A per-run server-provenance fact has an invalid shape or relationship (nonzero pid, parseable
    /// listen address, `client_url == http://{listen_addr}`, sibling `data`/`keys` directories).
    ServerProvenanceShape,
    /// A run's cumulative logical dose ladder is not strictly monotonic increasing.
    NonMonotonicLadder,
    /// A matched block's arm and control logical-cardinality ladders disagree dose-for-dose.
    CardinalityLadderMismatch,
    /// A dose's recorded physical cardinalities do not equal the deterministic
    /// [`CampaignDataset::physical_cardinalities`](crate::dataset::campaign_dataset::CampaignDataset)
    /// expectation for its run and dose.
    PhysicalCardinalityMismatch,
    /// A dose's raw latency vector does not carry exactly `BATCH_SIZE` samples.
    SampleCount,
    /// A dose's recorded median/IQR summary is not the exact R-1 recomputation of its raw latencies.
    SummaryRecomputationMismatch,
    /// A dose's event evidence violates the recorded insert/delete/net-delta algebraic identity
    /// (`inserts − deletes = net delta`). It asserts *only* that identity through
    /// [`EventEvidence::checked`](crate::observation::event_evidence::EventEvidence); it never asserts
    /// role-specific absolute event counts, which subscription coalescing makes non-deterministic
    /// (spec: "Record observed counts; assert only the event/net-delta identity").
    EventIdentityViolation,
}
