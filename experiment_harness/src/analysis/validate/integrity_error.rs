//! Why folding the untrusted wire records into the trusted campaign graph failed.

use std::fmt;

use crate::analysis::ingest::event_evidence_dto::EventEvidenceDto;
use crate::analysis::ingest::latency_summary_dto::LatencySummaryDto;
use crate::analysis::ingest::physical_cardinalities_dto::PhysicalCardinalitiesDto;
use crate::analysis::validate::block_census_fault::BlockCensusFault;
use crate::analysis::validate::cell_census_fault::CellCensusFault;
use crate::analysis::validate::coordinate_reference_fault::CoordinateReferenceFault;
use crate::analysis::validate::dose_census_fault::DoseCensusFault;
use crate::analysis::validate::integrity_error_category::IntegrityErrorCategory;
use crate::analysis::validate::malformed_stable_fault::MalformedStableFault;
use crate::analysis::validate::manifest_reference_fault::ManifestReferenceFault;
use crate::analysis::validate::record_coordinate_fault::RecordCoordinateFault;
use crate::analysis::validate::server_provenance_fault::ServerProvenanceFault;
use crate::analysis::validate::stable_fact_contradiction::StableFactContradiction;
use crate::dataset::dose_index::DoseIndex;
use crate::dataset::physical_cardinalities::PhysicalCardinalities;
use crate::manifest::run_coordinate::RunCoordinate;
use crate::observation::latency_summary::LatencySummary;
use crate::plan::cell::Cell;
use crate::plan::run_role::RunRole;

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
/// Each variant is classified by a closed [`IntegrityErrorCategory`] and carries **strongly typed
/// evidence** locating the failure at the narrowest structure available and pairing typed expected and
/// observed values — a trusted [`RunCoordinate`] (and [`DoseIndex`]) wherever a trusted coordinate
/// already exists, a fact-specific contradiction enum wherever a value disagreement must be located, and
/// a raw wire value only where parsing it into its domain type is itself the failure. A supplemental
/// prose `diagnostic` accompanies each variant but is never the *sole* record of the failure's identity;
/// the typed fields are (spec: "Make total-validation ownership and failure locations explicit").
///
/// The type is `pub(crate)` so the report layer can name and match it; the `pub(super)` constructors are
/// the canonical minting path used by the `validate` subtree.
#[derive(Debug)]
pub(crate) enum IntegrityError {
    /// Record sequence numbers are not the contiguous `0..total` set, or one repeats: at the zero-based
    /// sorted `position`, the record sequence was `found_seq` rather than the contiguous `position`.
    SequenceNotContiguous {
        position: usize,
        found_seq: u64,
        total: usize,
        diagnostic: String,
    },
    /// The manifests do not cover exactly the nine preregistered cells, or the static `CELL_COUNT` array
    /// length has drifted from the canonical set. The typed [`CellCensusFault`] carries the expected and
    /// observed distinct cell sets/counts.
    CellCensus {
        fault: CellCensusFault,
        diagnostic: String,
    },
    /// A `(cell, role)`'s repetition-block census failed. The typed [`BlockCensusFault`] carries the
    /// out-of-range, missing, or duplicated block with its expected range/occurrence evidence.
    BlockCensus {
        cell: Cell,
        role: RunRole,
        fault: BlockCensusFault,
        diagnostic: String,
    },
    /// A run's dose census failed. The typed [`DoseCensusFault`] carries the out-of-range, missing, or
    /// duplicated dose with its expected range/occurrence evidence.
    DoseCensus {
        run: RunCoordinate,
        fault: DoseCensusFault,
        diagnostic: String,
    },
    /// A run coordinate's recorded role disagrees with the `expected` arm/control position it was minted
    /// into.
    RunRoleMismatch {
        coordinate: RunCoordinate,
        expected: RunRole,
        diagnostic: String,
    },
    /// A dose record's own coordinate is internally inconsistent (kind-carried dose vs body dose, or an
    /// off-formula `logical_n`). The typed [`RecordCoordinateFault`] carries the trusted run and the
    /// expected/observed values; `seq` is the wire record sequence.
    RecordCoordinateMismatch {
        seq: u64,
        fault: RecordCoordinateFault,
        diagnostic: String,
    },
    /// The manifest↔reference binding failed (malformed identity, embedded disagreement, duplicate
    /// manifest key, dangling reference, or unreferenced manifest). The typed [`ManifestReferenceFault`]
    /// carries the identity components and multiplicity evidence.
    ManifestReferenceBinding {
        fault: ManifestReferenceFault,
        diagnostic: String,
    },
    /// A dose observation's coordinate disagrees with the manifest it references (reference-run
    /// disagreement or a non-canonical driving-role tag). The typed [`CoordinateReferenceFault`] carries
    /// the trusted run and both sides; `seq` is the wire record sequence.
    CoordinateReferenceMismatch {
        seq: u64,
        fault: CoordinateReferenceFault,
        diagnostic: String,
    },
    /// A campaign-stable provenance or parameter value is itself malformed or off the preregistered
    /// specification, independent of cross-run homogeneity. The typed [`MalformedStableFault`] carries
    /// either a fact-specific typed parse failure (the exact parse error plus the raw wire string that
    /// could not become its domain value), or a fact-specific expected/observed contradiction. `run` is
    /// the manifest's run coordinate.
    MalformedStableProvenance {
        run: RunCoordinate,
        fault: MalformedStableFault,
        diagnostic: String,
    },
    /// A well-formed campaign-stable fact is not homogeneous across the campaign's runs. The
    /// [`StableFactContradiction`] fixes both sides to one fact in one domain type: `expected` is the
    /// reference run's value, `observed` is the diverging run's. `run` is the diverging run; `reference_run`
    /// is the run whose value it should have matched.
    CampaignFactHeterogeneity {
        run: RunCoordinate,
        reference_run: RunCoordinate,
        contradiction: StableFactContradiction,
        diagnostic: String,
    },
    /// A per-run server-provenance fact has an invalid shape or relationship. The typed
    /// [`ServerProvenanceFault`] carries the fact-specific expected/observed evidence. `run` is the
    /// manifest's run coordinate.
    ServerProvenanceShape {
        run: RunCoordinate,
        fault: ServerProvenanceFault,
        diagnostic: String,
    },
    /// A run's cumulative logical dose ladder is not strictly monotonic increasing: at `dose` the
    /// cumulative logical count `logical_n` is not strictly greater than the previous dose's
    /// `previous_logical_n`.
    NonMonotonicLadder {
        run: RunCoordinate,
        dose: DoseIndex,
        previous_logical_n: u64,
        logical_n: u64,
        diagnostic: String,
    },
    /// A matched block's arm and control logical-cardinality ladders disagree at `dose`:
    /// `arm_logical_n` versus `control_logical_n`.
    CardinalityLadderMismatch {
        arm: RunCoordinate,
        control: RunCoordinate,
        dose: DoseIndex,
        arm_logical_n: u64,
        control_logical_n: u64,
        diagnostic: String,
    },
    /// A dose's recorded physical cardinalities (`observed`) do not equal the deterministic
    /// [`PhysicalCardinalities::expected`] `expected` value for its run and dose.
    PhysicalCardinalityMismatch {
        run: RunCoordinate,
        dose: DoseIndex,
        expected: PhysicalCardinalities,
        observed: PhysicalCardinalitiesDto,
        diagnostic: String,
    },
    /// A dose's raw latency vector does not carry exactly `expected` (`BATCH_SIZE_USIZE`) samples — it
    /// carried `observed`. Both are `usize`, the natural length type of the wire latency vector and of
    /// the fixed-size array [`RawLatencies`](crate::observation::raw_latencies::RawLatencies) seals into,
    /// so no count conversion is introduced.
    SampleCount {
        run: RunCoordinate,
        dose: DoseIndex,
        expected: usize,
        observed: usize,
        diagnostic: String,
    },
    /// A dose's `recorded` median/IQR summary is not the exact R-1 `recomputed` value from its raw
    /// latencies.
    SummaryRecomputationMismatch {
        run: RunCoordinate,
        dose: DoseIndex,
        recomputed: LatencySummary,
        recorded: LatencySummaryDto,
        diagnostic: String,
    },
    /// A dose's event evidence violates the insert/delete/net-delta identity: the recorded `observed`
    /// counts imply `expected_net_delta` (`inserts − deletes`), which is not the recorded
    /// `observed_net_delta`.
    EventIdentityViolation {
        run: RunCoordinate,
        dose: DoseIndex,
        observed: EventEvidenceDto,
        expected_net_delta: i128,
        observed_net_delta: i64,
        diagnostic: String,
    },
}

impl IntegrityError {
    /// Record sequence numbers are not the contiguous `0..total` set, or one repeats.
    pub(super) fn sequence_not_contiguous(
        position: usize,
        found_seq: u64,
        total: usize,
        diagnostic: String,
    ) -> Self {
        Self::SequenceNotContiguous {
            position,
            found_seq,
            total,
            diagnostic,
        }
    }

    /// The manifests do not cover exactly the nine preregistered cells, or `CELL_COUNT` drifted.
    pub(super) fn cell_census(fault: CellCensusFault, diagnostic: String) -> Self {
        Self::CellCensus { fault, diagnostic }
    }

    /// A `(cell, role)`'s repetition-block census failed.
    pub(super) fn block_census(
        cell: Cell,
        role: RunRole,
        fault: BlockCensusFault,
        diagnostic: String,
    ) -> Self {
        Self::BlockCensus {
            cell,
            role,
            fault,
            diagnostic,
        }
    }

    /// A run's dose census failed.
    pub(super) fn dose_census(
        run: RunCoordinate,
        fault: DoseCensusFault,
        diagnostic: String,
    ) -> Self {
        Self::DoseCensus {
            run,
            fault,
            diagnostic,
        }
    }

    /// A run coordinate's recorded role disagrees with its matched arm/control position.
    pub(super) fn run_role_mismatch(
        coordinate: RunCoordinate,
        expected: RunRole,
        diagnostic: String,
    ) -> Self {
        Self::RunRoleMismatch {
            coordinate,
            expected,
            diagnostic,
        }
    }

    /// A dose record's own coordinate is internally inconsistent.
    pub(super) fn record_coordinate_mismatch(
        seq: u64,
        fault: RecordCoordinateFault,
        diagnostic: String,
    ) -> Self {
        Self::RecordCoordinateMismatch {
            seq,
            fault,
            diagnostic,
        }
    }

    /// The manifest↔reference binding failed.
    pub(super) fn manifest_reference_binding(
        fault: ManifestReferenceFault,
        diagnostic: String,
    ) -> Self {
        Self::ManifestReferenceBinding { fault, diagnostic }
    }

    /// A dose observation's coordinate disagrees with its referenced manifest.
    pub(super) fn coordinate_reference_mismatch(
        seq: u64,
        fault: CoordinateReferenceFault,
        diagnostic: String,
    ) -> Self {
        Self::CoordinateReferenceMismatch {
            seq,
            fault,
            diagnostic,
        }
    }

    /// A campaign-stable provenance or parameter value is malformed or off the preregistered spec.
    pub(super) fn malformed_stable_provenance(
        run: RunCoordinate,
        fault: MalformedStableFault,
        diagnostic: String,
    ) -> Self {
        Self::MalformedStableProvenance {
            run,
            fault,
            diagnostic,
        }
    }

    /// A well-formed campaign-stable fact is not homogeneous across runs.
    pub(super) fn campaign_fact_heterogeneity(
        run: RunCoordinate,
        reference_run: RunCoordinate,
        contradiction: StableFactContradiction,
        diagnostic: String,
    ) -> Self {
        Self::CampaignFactHeterogeneity {
            run,
            reference_run,
            contradiction,
            diagnostic,
        }
    }

    /// A per-run server-provenance fact has an invalid shape or relationship.
    pub(super) fn server_provenance_shape(
        run: RunCoordinate,
        fault: ServerProvenanceFault,
        diagnostic: String,
    ) -> Self {
        Self::ServerProvenanceShape {
            run,
            fault,
            diagnostic,
        }
    }

    /// A run's cumulative logical dose ladder is not strictly monotonic.
    pub(super) fn non_monotonic_ladder(
        run: RunCoordinate,
        dose: DoseIndex,
        previous_logical_n: u64,
        logical_n: u64,
        diagnostic: String,
    ) -> Self {
        Self::NonMonotonicLadder {
            run,
            dose,
            previous_logical_n,
            logical_n,
            diagnostic,
        }
    }

    /// A matched block's arm and control cardinality ladders disagree.
    pub(super) fn cardinality_ladder_mismatch(
        arm: RunCoordinate,
        control: RunCoordinate,
        dose: DoseIndex,
        arm_logical_n: u64,
        control_logical_n: u64,
        diagnostic: String,
    ) -> Self {
        Self::CardinalityLadderMismatch {
            arm,
            control,
            dose,
            arm_logical_n,
            control_logical_n,
            diagnostic,
        }
    }

    /// A dose's recorded physical cardinalities disagree with the deterministic expectation.
    pub(super) fn physical_cardinality_mismatch(
        run: RunCoordinate,
        dose: DoseIndex,
        expected: PhysicalCardinalities,
        observed: PhysicalCardinalitiesDto,
        diagnostic: String,
    ) -> Self {
        Self::PhysicalCardinalityMismatch {
            run,
            dose,
            expected,
            observed,
            diagnostic,
        }
    }

    /// A dose's raw latency vector does not carry exactly `BATCH_SIZE_USIZE` samples.
    pub(super) fn sample_count(
        run: RunCoordinate,
        dose: DoseIndex,
        expected: usize,
        observed: usize,
        diagnostic: String,
    ) -> Self {
        Self::SampleCount {
            run,
            dose,
            expected,
            observed,
            diagnostic,
        }
    }

    /// A dose's recorded summary is not the exact R-1 recomputation of its raw latencies.
    pub(super) fn summary_recomputation_mismatch(
        run: RunCoordinate,
        dose: DoseIndex,
        recomputed: LatencySummary,
        recorded: LatencySummaryDto,
        diagnostic: String,
    ) -> Self {
        Self::SummaryRecomputationMismatch {
            run,
            dose,
            recomputed,
            recorded,
            diagnostic,
        }
    }

    /// A dose's event evidence violates the insert/delete/net-delta identity.
    pub(super) fn event_identity_violation(
        run: RunCoordinate,
        dose: DoseIndex,
        observed: EventEvidenceDto,
        expected_net_delta: i128,
        observed_net_delta: i64,
        diagnostic: String,
    ) -> Self {
        Self::EventIdentityViolation {
            run,
            dose,
            observed,
            expected_net_delta,
            observed_net_delta,
            diagnostic,
        }
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

    /// The supplemental prose diagnostic for this failure. It accompanies — never replaces — the typed
    /// evidence fields, which remain the authoritative record of *where* the failure is and *what* the
    /// contradiction is.
    pub(crate) fn diagnostic(&self) -> &str {
        match self {
            Self::SequenceNotContiguous { diagnostic, .. }
            | Self::CellCensus { diagnostic, .. }
            | Self::BlockCensus { diagnostic, .. }
            | Self::DoseCensus { diagnostic, .. }
            | Self::RunRoleMismatch { diagnostic, .. }
            | Self::RecordCoordinateMismatch { diagnostic, .. }
            | Self::ManifestReferenceBinding { diagnostic, .. }
            | Self::CoordinateReferenceMismatch { diagnostic, .. }
            | Self::MalformedStableProvenance { diagnostic, .. }
            | Self::CampaignFactHeterogeneity { diagnostic, .. }
            | Self::ServerProvenanceShape { diagnostic, .. }
            | Self::NonMonotonicLadder { diagnostic, .. }
            | Self::CardinalityLadderMismatch { diagnostic, .. }
            | Self::PhysicalCardinalityMismatch { diagnostic, .. }
            | Self::SampleCount { diagnostic, .. }
            | Self::SummaryRecomputationMismatch { diagnostic, .. }
            | Self::EventIdentityViolation { diagnostic, .. } => diagnostic,
        }
    }
}

impl fmt::Display for IntegrityError {
    /// Render the closed category and the supplemental prose. The typed evidence fields (matched by the
    /// report layer) remain the authoritative record; the full typed payload is available via `Debug`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "campaign integrity violation ({:?}): {}",
            self.category(),
            self.diagnostic()
        )
    }
}
