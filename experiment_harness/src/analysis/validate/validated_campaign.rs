//! The root of the trusted campaign graph, and the sole untrusted→trusted validation entry point.

use std::net::SocketAddr;
use std::path::Path;

use semver::Version;

use crate::analysis::ingest::cell_dto::CellDto;
use crate::analysis::ingest::key_scoped_arm_dto::KeyScopedArmDto;
use crate::analysis::ingest::manifest_reference_dto::ManifestReferenceDto;
use crate::analysis::ingest::physical_cardinalities_dto::PhysicalCardinalitiesDto;
use crate::analysis::ingest::record_kind_dto::RecordKindDto;
use crate::analysis::ingest::run_coordinate_dto::RunCoordinateDto;
use crate::analysis::ingest::run_role_dto::RunRoleDto;
use crate::analysis::ingest::table_scoped_arm_dto::TableScopedArmDto;
use crate::analysis::ingest::wire_record_dto::WireRecordDto;
use crate::analysis::validate::arm_run::ArmRun;
use crate::analysis::validate::block_census_fault::BlockCensusFault;
use crate::analysis::validate::cell_census_fault::CellCensusFault;
use crate::analysis::validate::cell_dataset::CellDataset;
use crate::analysis::validate::control_run::ControlRun;
use crate::analysis::validate::coordinate_reference_fault::CoordinateReferenceFault;
use crate::analysis::validate::dose_census_fault::DoseCensusFault;
use crate::analysis::validate::integrity_error::IntegrityError;
use crate::analysis::validate::malformed_stable_fault::MalformedStableFault;
use crate::analysis::validate::manifest_reference_fault::ManifestReferenceFault;
use crate::analysis::validate::manifest_reference_identity::ManifestReferenceIdentity;
use crate::analysis::validate::matched_block::MatchedBlock;
use crate::analysis::validate::record_coordinate_fault::RecordCoordinateFault;
use crate::analysis::validate::record_slot::RecordSlot;
use crate::analysis::validate::run_kind::RunKind;
use crate::analysis::validate::schedule_order_fault::ScheduleOrderFault;
use crate::analysis::validate::server_provenance_fault::ServerProvenanceFault;
use crate::analysis::validate::stable_fact_contradiction::StableFactContradiction;
use crate::analysis::validate::trusted_dose::TrustedDose;
use crate::analysis::validate::trusted_run::TrustedRun;
use crate::dataset::dose_index::DoseIndex;
use crate::dataset::physical_cardinalities::PhysicalCardinalities;
use crate::manifest::database_identity::DatabaseIdentity;
use crate::manifest::preregistered_parameters::PreregisteredParameters;
use crate::manifest::release_commit::ReleaseCommit;
use crate::manifest::repetition_block_index::RepetitionBlockIndex;
use crate::manifest::run_coordinate::RunCoordinate;
use crate::manifest::schedule_seed::ScheduleSeed;
use crate::manifest::wasm_sha256::WasmSha256;
use crate::module_artifact::module_wasm_sha256::MODULE_WASM_SHA256;
use crate::observation::event_evidence::EventEvidence;
use crate::observation::latency_sample::LatencySample;
use crate::observation::latency_summary::LatencySummary;
use crate::observation::raw_latencies::RawLatencies;
use crate::observation::record_seq::RecordSeq;
use crate::params::{
    BATCH_DELAY_MS, BATCH_SIZE, BATCH_SIZE_USIZE, CONFIRMED_READS, EXPECTED_RELEASE_COMMIT,
    EXPECTED_VERSION, NUM_DOSES, NUM_DOSES_USIZE, REPETITION_BLOCKS,
};
use crate::plan::cell::Cell;
use crate::plan::key_scoped_arm::KeyScopedArm;
use crate::plan::run_role::RunRole;
use crate::plan::schedule::{BlockCoordinate, BlockRun, Schedule};
use crate::plan::table_scoped_arm::TableScopedArm;

/// The number of preregistered cells — five table-scoped arms under unrelated growth plus the two
/// key-scoped arms under each of the two growth regimes (the length of
/// [`Cell::all`](crate::plan::cell::Cell::all)). It fixes the campaign's cell-array length, so *exactly
/// nine cell slots* is a property of the type.
///
/// That those nine slots equal `Cell::all()` — no missing, duplicated, or extra cell, and that this
/// length in fact agrees with the preregistered set — is **not** a compile-time guarantee: `Cell::all()`
/// returns a heap `Vec` and is not `const`, so it cannot back a `const` assertion here. The single
/// validation pass owns that agreement instead: [`Self::from_records`] censuses the manifests against
/// `Cell::all()` and asserts `CELL_COUNT == Cell::all().len()` before minting any cell, surfacing any
/// drift as a typed [`IntegrityError`]. Drift is caught by validation, not by the type.
const CELL_COUNT: usize = 9;

/// The two run roles per cell block (arm and its matched control).
const ROLE_COUNT: usize = 2;

/// [`REPETITION_BLOCKS`] as a `usize`, guarded by a compile-time round-trip assertion rather than a bare
/// `as` cast, mirroring [`NUM_DOSES_USIZE`]/[`BATCH_SIZE_USIZE`], so a platform on which the value does
/// not fit a `usize` fails to compile instead of silently truncating a slot index.
const REPETITION_BLOCKS_USIZE: usize = {
    let as_usize = REPETITION_BLOCKS as usize;
    assert!(
        as_usize as u32 == REPETITION_BLOCKS,
        "REPETITION_BLOCKS does not fit in usize on this platform"
    );
    as_usize
};

/// The number of canonical run slots: nine cells × two roles × thirty repetition blocks (540).
const RUN_SLOTS: usize = CELL_COUNT * ROLE_COUNT * REPETITION_BLOCKS_USIZE;

/// The number of canonical dose slots: every run slot × the ten-dose ladder (5,400).
const DOSE_SLOTS: usize = RUN_SLOTS * NUM_DOSES_USIZE;

/// The number of records in a complete campaign: each of the [`RUN_SLOTS`] runs contributes its one
/// manifest plus its ten cumulative-dose observations (5,940 = 540 × 11). It fixes the reconstructed
/// schedule grammar's cardinality, so the grammar is a heap-owned fixed array rather than a `Vec` whose
/// length must be checked at runtime.
const RECORD_COUNT: usize = RUN_SLOTS * (1 + NUM_DOSES_USIZE);

/// A complete campaign proven structurally sound: one schedule seed, the preregistered parameters, and
/// exactly the nine preregistered cells, each a fully validated [`CellDataset`]. This is the only value
/// statistical code may consume, and the only path to one is [`Self::from_records`] — the single total
/// validation pass. The seed and parameters are the campaign-stable facts proven homogeneous across
/// every run.
///
/// Fields are private with no defaults; the `pub(super)` [`Self::new`] assembles the root only from
/// within the `validate` subtree, while [`Self::from_records`] is the `pub(crate)` entry the report layer
/// calls.
pub(crate) struct ValidatedCampaign {
    /// The one schedule seed, proven identical across every run's manifest.
    schedule_seed: ScheduleSeed,
    /// The preregistered parameters, proven identical to the frozen values across every run.
    parameters: PreregisteredParameters,
    /// Exactly the nine preregistered cells' complete datasets. Heap-owned as a boxed fixed array so the
    /// nine-cell cardinality remains a property of the type while the campaign's by-value footprint is one
    /// pointer — keeping [`Self::from_records`]'s stack frame bounded regardless of [`CELL_COUNT`] on every
    /// return path, success or failure.
    cells: Box<[CellDataset; CELL_COUNT]>,
}

impl ValidatedCampaign {
    /// The single total validation pass: fold the complete set of ingested wire records into the trusted
    /// campaign graph, or fail loud with a typed [`IntegrityError`]. It proves every preregistered
    /// completeness and integrity obligation — sequence contiguity, per-record coordinate/value proofs,
    /// bijective manifest references, campaign-stable homogeneity and per-run provenance shape, exact
    /// cell/block/dose census, monotonic and matched cardinality ladders — before any [`CellDataset`],
    /// [`MatchedBlock`], [`TrustedRun`], or [`TrustedDose`] is minted, so no partially valid campaign can
    /// escape into statistical code.
    ///
    /// The stages run in the preregistered deterministic order and each reports its **first** failure in
    /// a fixed traversal order (records in sequence order; census/ladders/mint in canonical
    /// cell→role→block→dose order), so an identical campaign always yields an identical first error.
    /// Intermediate lookup is keyed by canonical *index* into fixed-size staging, never by a hashed or
    /// ordered wrapper of a trusted coordinate, and the trusted graph is minted only by the final
    /// canonical loop after every earlier stage has passed.
    ///
    /// [`MatchedBlock`]: super::matched_block::MatchedBlock
    /// [`TrustedRun`]: super::trusted_run::TrustedRun
    /// [`TrustedDose`]: super::trusted_dose::TrustedDose
    pub(crate) fn from_records(
        records: Vec<WireRecordDto>,
    ) -> std::result::Result<Self, IntegrityError> {
        // Stage 1 — sequence contiguity. The sorted sequence numbers must be exactly `0..total`; a gap
        // or a duplicate makes some sorted position disagree with its index.
        let total = records.len();
        let mut ordered: Vec<&WireRecordDto> = records.iter().collect();
        ordered.sort_by_key(|record| record.record().seq);
        for (position, record) in ordered.iter().enumerate() {
            let found_seq = record.record().seq;
            if found_seq != position as u64 {
                return Err(IntegrityError::sequence_not_contiguous(
                    position,
                    found_seq,
                    total,
                    format!(
                        "record sequence numbers must be the contiguous 0..{total} set; at sorted \
                         position {position} the sequence is {found_seq}"
                    ),
                ));
            }
        }

        // Stage 2 — per-record own coordinate/value proofs. Each record is validated in isolation and its
        // validated content is staged at its canonical slot; nothing cross-record is consulted yet.
        let mut manifests: Vec<StagedManifest> = Vec::new();
        let mut observation_refs: Vec<StagedObservationRef> = Vec::new();
        let mut dose_slots: Vec<Vec<StagedDose>> = (0..DOSE_SLOTS).map(|_| Vec::new()).collect();
        for &record in &ordered {
            match record {
                WireRecordDto::Manifest {
                    record: id,
                    body,
                } => {
                    let coordinate = build_coordinate(&body.manifest.run)?;
                    manifests.push(StagedManifest {
                        seq: id.seq,
                        coordinate,
                        body: body.clone(),
                    });
                }
                WireRecordDto::Dose {
                    record: id,
                    body,
                } => {
                    let (slot, staged) = validate_dose_record(id.seq, id.kind, &body.observation)?;
                    dose_slots[slot].push(staged);
                    observation_refs.push(StagedObservationRef {
                        seq: id.seq,
                        manifest_ref: body.observation.manifest_ref.clone(),
                    });
                }
            }
        }

        // Stage 3 — manifest↔reference binding. Parse identities into typed keys, then prove the
        // manifest→identity map injective and the reference↔manifest correspondence bijective.
        bind_manifest_references(&manifests, &observation_refs)?;

        // Stage 4 — campaign-stable homogeneity / off-specification and per-run provenance shape.
        validate_stable_and_provenance(&manifests)?;

        // Stage 5 — exact cell/block/dose census over the canonical slots.
        census(&manifests, &dose_slots)?;

        // Stage 6 — schedule-order grammar binding. Bind each record's sequence position to the
        // seed-derived preregistered schedule grammar (the global `Cell × block` permutation, the
        // seed-selected adjacent arm/control order, and the manifest-then-canonical-doses within-run
        // order), so the trusted collection order provably reflects the randomized schedule rather than
        // the generator's emission order. Runs after census, which has proven the canonical population is
        // complete, and before minting, so a block key is minted only once its schedule order is proven.
        validate_schedule_order(&ordered, &manifests)?;

        // Stage 7 — monotonic per-run and matched arm/control cardinality ladders.
        validate_ladders(&dose_slots)?;

        // Stage 8 — mint the trusted graph. Every earlier stage has passed, so each canonical slot holds
        // exactly one validated value; the canonical loop mints the fixed-cardinality tree.
        let cells = mint_cells(&mut dose_slots, &manifests)?;

        // Seed and parameters are the campaign-stable facts proven homogeneous in stage 4; take the
        // reference run's seed (all runs equal it) and snapshot the preregistered parameters.
        let schedule_seed = ScheduleSeed::new(
            manifests
                .first()
                .expect("a complete campaign has 540 manifests, so at least one")
                .body
                .manifest
                .schedule_seed,
        );
        Ok(Self::new(
            schedule_seed,
            PreregisteredParameters::preregistered(),
            cells,
        ))
    }

    /// Assemble the validated campaign root from its proven-homogeneous seed and parameters and its
    /// complete set of validated cell datasets. The caller ([`Self::from_records`]) owns proving seed and
    /// parameter homogeneity and the nine-cell census; this constructor only binds the proven parts.
    pub(super) fn new(
        schedule_seed: ScheduleSeed,
        parameters: PreregisteredParameters,
        cells: Box<[CellDataset; CELL_COUNT]>,
    ) -> Self {
        Self {
            schedule_seed,
            parameters,
            cells,
        }
    }

    /// The nine validated cell datasets, in canonical [`Cell::all`] order — the complete per-cell
    /// evidence the classifier folds each into a
    /// [`CellClassification`](crate::analysis::classify::cell_classification::CellClassification).
    pub(crate) fn cells(&self) -> &[CellDataset; CELL_COUNT] {
        &self.cells
    }
}

/// One manifest record staged for the cross-record binding, provenance, and census stages: its wire
/// sequence, its trusted run coordinate, and the retained wire body carrying the facts those stages
/// prove.
struct StagedManifest {
    seq: u64,
    coordinate: RunCoordinate,
    body: crate::analysis::ingest::manifest_body_dto::ManifestBodyDto,
}

/// One dose observation's manifest reference, retained for the binding stage's bijection proof.
struct StagedObservationRef {
    seq: u64,
    manifest_ref: ManifestReferenceDto,
}

/// One cumulative dose's validated content, staged by canonical dose slot until the mint stage binds it
/// into a [`TrustedDose`]. These are the production trusted content types, not graph nodes, so building
/// them during per-record proofs is not graph minting.
struct StagedDose {
    logical_n: u64,
    latencies: RawLatencies,
    summary: LatencySummary,
    cardinalities: PhysicalCardinalities,
    events: EventEvidence,
}

/// The zero-based position of `cell` in the canonical [`Cell::all`] ordering. Every trusted `Cell` is a
/// member of that set (the closed [`CellDto`] mirror cannot decode a non-member), so the lookup is
/// total.
fn cell_index(cell: Cell) -> usize {
    Cell::all()
        .iter()
        .position(|candidate| *candidate == cell)
        .expect("every trusted Cell is a member of Cell::all()")
}

/// The canonical role index: arm before control.
fn role_index(role: RunRole) -> usize {
    match role {
        RunRole::Arm => 0,
        RunRole::Control => 1,
    }
}

/// The flat canonical run-slot index for `(cell, role, block)`, in `0..RUN_SLOTS`.
fn run_slot_index(cell_idx: usize, role_idx: usize, block: usize) -> usize {
    (cell_idx * ROLE_COUNT + role_idx) * REPETITION_BLOCKS_USIZE + block
}

/// The flat canonical dose-slot index for a run slot and a 1-based dose, in `0..DOSE_SLOTS`.
fn dose_slot_index(run_slot: usize, dose_zero_based: usize) -> usize {
    run_slot * NUM_DOSES_USIZE + dose_zero_based
}

/// The trusted role for a wire role tag.
fn run_role_from_dto(role: RunRoleDto) -> RunRole {
    match role {
        RunRoleDto::Arm => RunRole::Arm,
        RunRoleDto::Control => RunRole::Control,
    }
}

/// The trusted table-scoped arm for its wire mirror.
fn table_arm_from_dto(arm: TableScopedArmDto) -> TableScopedArm {
    match arm {
        TableScopedArmDto::ProceduralRange => TableScopedArm::ProceduralRange,
        TableScopedArmDto::QueryFull => TableScopedArm::QueryFull,
        TableScopedArmDto::QuerySemijoin => TableScopedArm::QuerySemijoin,
        TableScopedArmDto::QueryFullPk => TableScopedArm::QueryFullPk,
        TableScopedArmDto::QuerySemijoinPk => TableScopedArm::QuerySemijoinPk,
    }
}

/// The trusted key-scoped arm for its wire mirror.
fn key_arm_from_dto(arm: KeyScopedArmDto) -> KeyScopedArm {
    match arm {
        KeyScopedArmDto::PointFilter => KeyScopedArm::PointFilter,
        KeyScopedArmDto::PointSemijoin => KeyScopedArm::PointSemijoin,
    }
}

/// The trusted cell for its closed wire mirror. Total: the [`CellDto`] variant set is the exact mirror of
/// [`Cell`], so an impossible arm/regime pairing is unrepresentable at the wire level and never reaches
/// here.
fn cell_from_dto(cell: CellDto) -> Cell {
    match cell {
        CellDto::TableScopedUnrelated(arm) => Cell::TableScopedUnrelated(table_arm_from_dto(arm)),
        CellDto::KeyScopedUnrelated(arm) => Cell::KeyScopedUnrelated(key_arm_from_dto(arm)),
        CellDto::KeyScopedOwnSlice(arm) => Cell::KeyScopedOwnSlice(key_arm_from_dto(arm)),
    }
}

/// Reconstruct a trusted [`RunCoordinate`] from a wire coordinate, proving the repetition-block index is
/// in `0..REPETITION_BLOCKS`. An out-of-range block is a typed [`BlockCensus`] out-of-range failure
/// (spec: "classify out-of-range repetition blocks as `BlockCensus`").
///
/// [`BlockCensus`]: super::integrity_error::IntegrityError
fn build_coordinate(dto: &RunCoordinateDto) -> std::result::Result<RunCoordinate, IntegrityError> {
    let cell = cell_from_dto(dto.cell);
    let role = run_role_from_dto(dto.role);
    match RepetitionBlockIndex::try_new(dto.repetition_block) {
        Ok(block) => Ok(RunCoordinate::from_parts(cell, role, block)),
        Err(_) => Err(IntegrityError::block_census(
            cell,
            role,
            BlockCensusFault::OutOfRange {
                block: dto.repetition_block,
                expected_min: 0,
                expected_max: REPETITION_BLOCKS - 1,
            },
            format!(
                "repetition block {} is outside the valid 0..={} range",
                dto.repetition_block,
                REPETITION_BLOCKS - 1
            ),
        )),
    }
}

/// Validate one dose observation record against every per-record obligation and return its canonical
/// dose slot and staged validated content. Nothing cross-record is consulted.
fn validate_dose_record(
    seq: u64,
    kind: RecordKindDto,
    observation: &crate::analysis::ingest::dose_observation_dto::DoseObservationDto,
) -> std::result::Result<(usize, StagedDose), IntegrityError> {
    let coordinate = build_coordinate(&observation.coordinate.run)?;
    let cell = coordinate.cell();

    // Dose range → a trusted DoseIndex (spec: out-of-range doses are `DoseCensus`).
    let dose_value = observation.coordinate.dose;
    if !(1..=NUM_DOSES).contains(&dose_value) {
        return Err(IntegrityError::dose_census(
            coordinate,
            DoseCensusFault::OutOfRange {
                dose: dose_value,
                expected_min: 1,
                expected_max: NUM_DOSES,
            },
            format!("dose {dose_value} is outside the valid 1..={NUM_DOSES} range"),
        ));
    }
    let dose_zero_based = (dose_value - 1) as usize;
    let dose = DoseIndex::ALL[dose_zero_based];

    // The record kind of a dose-bodied line always carries a `Dose` discriminant (the envelope dispatch
    // only builds this variant for a `Dose` kind), so extracting its carried dose is total.
    let RecordKindDto::Dose(kind_dose) = kind else {
        unreachable!("a dose observation record always carries a Dose record kind")
    };
    if kind_dose != dose_value {
        return Err(IntegrityError::record_coordinate_mismatch(
            seq,
            RecordCoordinateFault::KindDoseDisagreement {
                run: coordinate,
                kind_dose,
                coordinate_dose: dose_value,
            },
            format!(
                "record-kind dose {kind_dose} disagrees with the body coordinate dose {dose_value}"
            ),
        ));
    }

    // Cumulative logical count must be the preregistered `dose * BATCH_SIZE`.
    let expected_logical_n = dose.cumulative_driving_rows();
    if observation.coordinate.logical_n != expected_logical_n {
        return Err(IntegrityError::record_coordinate_mismatch(
            seq,
            RecordCoordinateFault::OffFormulaLogicalN {
                run: coordinate,
                dose,
                expected: expected_logical_n,
                observed: observation.coordinate.logical_n,
            },
            format!(
                "cumulative logical_n {} is off the preregistered dose*BATCH_SIZE = {}",
                observation.coordinate.logical_n, expected_logical_n
            ),
        ));
    }

    // The observation's own coordinate run must equal its manifest reference's run.
    let reference_coordinate = build_coordinate(&observation.manifest_ref.run)?;
    if coordinate != reference_coordinate {
        return Err(IntegrityError::coordinate_reference_mismatch(
            seq,
            CoordinateReferenceFault::ReferenceRunDisagreement {
                coordinate_run: coordinate,
                reference_run: reference_coordinate,
            },
            "the observation coordinate's run disagrees with its manifest reference's run".to_string(),
        ));
    }

    // The serialized driving-role tag must be the canonical tag of the cell's driving role.
    let expected_role = cell.growth_regime().driving_role();
    if observation.coordinate.driving_role_tag != expected_role.canonical_tag() {
        return Err(IntegrityError::coordinate_reference_mismatch(
            seq,
            CoordinateReferenceFault::DrivingRoleTag {
                run: coordinate,
                expected: expected_role,
                observed: observation.coordinate.driving_role_tag.clone(),
            },
            format!(
                "driving-role tag {:?} is not the canonical tag {:?} of the cell's driving role",
                observation.coordinate.driving_role_tag,
                expected_role.canonical_tag()
            ),
        ));
    }

    // Exactly BATCH_SIZE raw latency samples, then the trusted lossless vector.
    let observed_samples = observation.latencies.len();
    if observed_samples != BATCH_SIZE_USIZE {
        return Err(IntegrityError::sample_count(
            coordinate,
            dose,
            BATCH_SIZE_USIZE,
            observed_samples,
            format!(
                "dose latency vector carries {observed_samples} samples, not the required \
                 {BATCH_SIZE_USIZE}"
            ),
        ));
    }
    let latencies = RawLatencies::sealed(
        observation
            .latencies
            .iter()
            .map(|nanos| LatencySample::from_nanos(*nanos))
            .collect(),
    )
    .expect("sample count was just proven equal to BATCH_SIZE_USIZE");

    // The recorded summary must be the exact R-1 recomputation of the raw latencies.
    let recomputed = LatencySummary::from_raw(&latencies);
    if recomputed.median_nanos() != observation.summary.median_nanos
        || recomputed.iqr_nanos() != observation.summary.iqr_nanos
    {
        return Err(IntegrityError::summary_recomputation_mismatch(
            coordinate,
            dose,
            recomputed,
            observation.summary,
            "recorded median/IQR summary is not the exact R-1 recomputation of the raw latencies"
                .to_string(),
        ));
    }

    // The recorded physical cardinalities must equal the deterministic dataset expectation.
    let expected_cardinalities = PhysicalCardinalities::expected(cell, dose);
    if !cardinalities_match(&expected_cardinalities, &observation.cardinalities) {
        return Err(IntegrityError::physical_cardinality_mismatch(
            coordinate,
            dose,
            expected_cardinalities,
            observation.cardinalities,
            "recorded physical cardinalities disagree with the deterministic dataset expectation"
                .to_string(),
        ));
    }

    // The recorded event counts must satisfy the insert/delete/net-delta identity.
    let events = &observation.events;
    let expected_net_delta = i128::from(events.inserts) - i128::from(events.deletes);
    if expected_net_delta != i128::from(events.delivered_net_row_delta) {
        return Err(IntegrityError::event_identity_violation(
            coordinate,
            dose,
            observation.events,
            expected_net_delta,
            events.delivered_net_row_delta,
            format!(
                "delivered event net delta (inserts {} − deletes {} = {}) disagrees with the \
                 recorded net delta {}",
                events.inserts, events.deletes, expected_net_delta, events.delivered_net_row_delta
            ),
        ));
    }
    let events = EventEvidence::checked(
        events.inserts,
        events.deletes,
        events.updates,
        events.delivered_net_row_delta,
    )
    .expect("event insert/delete/net-delta identity was just proven");

    let run_slot = run_slot_index(
        cell_index(cell),
        role_index(coordinate.role()),
        observation.coordinate.run.repetition_block as usize,
    );
    let slot = dose_slot_index(run_slot, dose_zero_based);
    Ok((
        slot,
        StagedDose {
            logical_n: observation.coordinate.logical_n,
            latencies,
            summary: recomputed,
            cardinalities: expected_cardinalities,
            events,
        },
    ))
}

/// Whether a trusted physical-cardinality expectation equals its recorded wire mirror: the family must
/// match and every per-table count must agree.
fn cardinalities_match(
    expected: &PhysicalCardinalities,
    observed: &PhysicalCardinalitiesDto,
) -> bool {
    match (expected, observed) {
        (PhysicalCardinalities::Message(expected), PhysicalCardinalitiesDto::Message(observed)) => {
            expected.message_rows() == observed.message_rows
        }
        (
            PhysicalCardinalities::Chronicle(expected),
            PhysicalCardinalitiesDto::Chronicle(observed),
        ) => {
            expected.visibility_rows() == observed.visibility_rows
                && expected.message_rows() == observed.message_rows
        }
        _ => false,
    }
}

/// Parse a manifest reference's identity components into a typed [`ManifestReferenceIdentity`] key, or a
/// typed malformed-identity binding failure locating the run whose hex could not be parsed.
fn reference_identity(
    reference: &ManifestReferenceDto,
) -> std::result::Result<ManifestReferenceIdentity, IntegrityError> {
    let run = build_coordinate(&reference.run)?;
    let identity = parse_identity(run.clone(), &reference.database_identity)?;
    Ok(ManifestReferenceIdentity::new(
        run,
        identity,
        ScheduleSeed::new(reference.schedule_seed),
    ))
}

/// Parse a canonical-hex database identity, or a typed malformed-identity binding failure carrying the
/// run, the raw hex, and the domain-owned typed parse error.
fn parse_identity(
    run: RunCoordinate,
    hex: &str,
) -> std::result::Result<DatabaseIdentity, IntegrityError> {
    DatabaseIdentity::parse_canonical_hex(hex).map_err(|error| {
        IntegrityError::manifest_reference_binding(
            ManifestReferenceFault::MalformedIdentity {
                run,
                hex: hex.to_string(),
                error,
            },
            format!("database identity hex {hex:?} is not canonical: {error}"),
        )
    })
}

/// Stage 3: prove the manifest↔observation reference binding. Identities become typed keys, then the
/// manifest→identity map is proven injective and the reference↔manifest correspondence bijective.
fn bind_manifest_references(
    manifests: &[StagedManifest],
    observation_refs: &[StagedObservationRef],
) -> std::result::Result<(), IntegrityError> {
    // Parse every manifest's own identity (module database identity) and its embedded reference identity,
    // proving the two agree, in sequence order.
    let mut manifest_keys: Vec<ManifestReferenceIdentity> = Vec::with_capacity(manifests.len());
    for manifest in manifests {
        let module = &manifest.body.manifest.module;
        let own = ManifestReferenceIdentity::new(
            manifest.coordinate.clone(),
            parse_identity(manifest.coordinate.clone(), &module.database_identity)?,
            ScheduleSeed::new(manifest.body.manifest.schedule_seed),
        );
        let embedded = reference_identity(&manifest.body.reference)?;
        if own != embedded {
            return Err(IntegrityError::manifest_reference_binding(
                ManifestReferenceFault::EmbeddedDisagreement {
                    reference: embedded,
                    manifest: own,
                },
                format!(
                    "manifest at seq {} has an embedded reference disagreeing with its own manifest \
                     identity",
                    manifest.seq
                ),
            ));
        }
        manifest_keys.push(own);
    }

    // Parse every observation's reference identity, in sequence order.
    let mut observation_keys: Vec<ManifestReferenceIdentity> =
        Vec::with_capacity(observation_refs.len());
    for observation in observation_refs {
        observation_keys.push(reference_identity(&observation.manifest_ref)?);
    }

    // Injectivity: no two manifests share one identity key.
    for (position, key) in manifest_keys.iter().enumerate() {
        let occurrences = manifest_keys.iter().filter(|other| *other == key).count();
        if occurrences != 1 {
            return Err(IntegrityError::manifest_reference_binding(
                ManifestReferenceFault::DuplicateManifest {
                    identity: key.clone(),
                    expected_occurrences: 1,
                    observed_occurrences: occurrences,
                },
                format!(
                    "manifest at seq {} shares its identity key with {} manifests",
                    manifests[position].seq, occurrences
                ),
            ));
        }
    }

    // Surjectivity of references onto manifests: no observation references an absent manifest.
    for (position, key) in observation_keys.iter().enumerate() {
        let matches = manifest_keys.iter().filter(|other| *other == key).count();
        if matches == 0 {
            return Err(IntegrityError::manifest_reference_binding(
                ManifestReferenceFault::Dangling {
                    reference: key.clone(),
                    expected_manifest_matches: 1,
                    observed_manifest_matches: 0,
                },
                format!(
                    "observation at seq {} references an identity no manifest provides",
                    observation_refs[position].seq
                ),
            ));
        }
    }

    // No manifest is entirely unreferenced (a manifest referenced by some-but-not-all of its doses has a
    // present key here; its missing doses are caught by the dose census).
    for (position, key) in manifest_keys.iter().enumerate() {
        let references = observation_keys.iter().filter(|other| *other == key).count();
        if references == 0 {
            return Err(IntegrityError::manifest_reference_binding(
                ManifestReferenceFault::Unreferenced {
                    manifest: key.clone(),
                    expected_reference_occurrences: NUM_DOSES_USIZE,
                    observed_reference_occurrences: 0,
                },
                format!(
                    "manifest at seq {} is referenced by no observation",
                    manifests[position].seq
                ),
            ));
        }
    }

    Ok(())
}

/// Stage 4: prove campaign-stable facts off-specification/homogeneous and per-run provenance shape valid.
/// Facts with a frozen preregistered value are compared against it (off-specification); the pinned
/// environment/provenance facts and the seed, which have no preregistered constant, are proven identical
/// across runs (heterogeneity).
fn validate_stable_and_provenance(
    manifests: &[StagedManifest],
) -> std::result::Result<(), IntegrityError> {
    let expected_version =
        Version::parse(EXPECTED_VERSION).expect("EXPECTED_VERSION is a valid semver constant");
    let expected_commit = ReleaseCommit::parse_canonical_hex(EXPECTED_RELEASE_COMMIT)
        .expect("EXPECTED_RELEASE_COMMIT is a canonical lowercase-hex constant");
    let expected_wasm = WasmSha256::new(MODULE_WASM_SHA256);
    let expected_ladder: Vec<u64> = (1..=NUM_DOSES).map(|dose| dose * BATCH_SIZE).collect();

    for manifest in manifests {
        let run = &manifest.coordinate;
        let inner = &manifest.body.manifest;
        let distribution = &inner.distribution;
        let parameters = &inner.parameters;
        let module = &inner.module;
        let server = &inner.server;

        // Off-specification checks against frozen preregistered values.
        check_version(
            run,
            &distribution.cli_version,
            &expected_version,
            StableVersionFact::Cli,
        )?;
        check_version(
            run,
            &distribution.standalone_version,
            &expected_version,
            StableVersionFact::Standalone,
        )?;
        check_release_commit(run, &distribution.cli_release_commit, &expected_commit)?;
        check_wasm_sha256(run, &module.wasm_sha256, &expected_wasm)?;
        check_raw_version(
            run,
            &distribution.cli_version_raw,
            StableVersionFact::Cli,
        )?;
        check_raw_version(
            run,
            &distribution.standalone_version_raw,
            StableVersionFact::Standalone,
        )?;
        if parameters.batch_size != BATCH_SIZE {
            return Err(off_specification(
                run,
                StableFactContradiction::BatchSize {
                    expected: BATCH_SIZE,
                    observed: parameters.batch_size,
                },
                "batch_size parameter is off the preregistered constant",
            ));
        }
        if parameters.num_doses != NUM_DOSES {
            return Err(off_specification(
                run,
                StableFactContradiction::NumDoses {
                    expected: NUM_DOSES,
                    observed: parameters.num_doses,
                },
                "num_doses parameter is off the preregistered constant",
            ));
        }
        if parameters.dose_ladder != expected_ladder {
            return Err(off_specification(
                run,
                StableFactContradiction::DoseLadder {
                    expected: expected_ladder.clone(),
                    observed: parameters.dose_ladder.clone(),
                },
                "dose_ladder parameter is off the preregistered ladder",
            ));
        }
        if parameters.batch_delay_ms != BATCH_DELAY_MS {
            return Err(off_specification(
                run,
                StableFactContradiction::BatchDelayMs {
                    expected: BATCH_DELAY_MS,
                    observed: parameters.batch_delay_ms,
                },
                "batch_delay_ms parameter is off the preregistered constant",
            ));
        }
        if parameters.repetition_blocks != REPETITION_BLOCKS {
            return Err(off_specification(
                run,
                StableFactContradiction::RepetitionBlocks {
                    expected: REPETITION_BLOCKS,
                    observed: parameters.repetition_blocks,
                },
                "repetition_blocks parameter is off the preregistered constant",
            ));
        }
        if parameters.confirmed_reads != CONFIRMED_READS {
            return Err(off_specification(
                run,
                StableFactContradiction::ConfirmedReads {
                    expected: CONFIRMED_READS,
                    observed: parameters.confirmed_reads,
                },
                "confirmed_reads parameter is off the preregistered constant",
            ));
        }

        // Per-run server-provenance shape.
        validate_server_provenance(run, server, &distribution.standalone_exe)?;
    }

    // Homogeneity of the pinned environment/provenance facts and the seed: every run must equal the
    // reference (first, in sequence order) run's value.
    if let Some(reference) = manifests.first() {
        let reference_inner = &reference.body.manifest;
        for manifest in &manifests[1..] {
            let inner = &manifest.body.manifest;
            check_homogeneous_path(
                manifest,
                reference,
                &inner.distribution.nix_store_bin_dir,
                &reference_inner.distribution.nix_store_bin_dir,
                StableEnvFact::NixStoreBinDir,
            )?;
            check_homogeneous_path(
                manifest,
                reference,
                &inner.distribution.cli_exe,
                &reference_inner.distribution.cli_exe,
                StableEnvFact::CliExe,
            )?;
            check_homogeneous_path(
                manifest,
                reference,
                &inner.distribution.standalone_exe,
                &reference_inner.distribution.standalone_exe,
                StableEnvFact::StandaloneExe,
            )?;
            if inner.schedule_seed != reference_inner.schedule_seed {
                return Err(heterogeneity(
                    manifest,
                    reference,
                    StableFactContradiction::ScheduleSeed {
                        expected: ScheduleSeed::new(reference_inner.schedule_seed),
                        observed: ScheduleSeed::new(inner.schedule_seed),
                    },
                    "schedule seed is not homogeneous across the campaign",
                ));
            }
        }
    }

    Ok(())
}

/// Which version fact a semver/raw-version check concerns, so the typed contradiction/parse variant is
/// selected without duplicating the check body.
enum StableVersionFact {
    Cli,
    Standalone,
}

/// Which pinned environment path fact a homogeneity check concerns.
enum StableEnvFact {
    NixStoreBinDir,
    CliExe,
    StandaloneExe,
}

/// Build a `MalformedStableProvenance` off-specification failure for one run and contradiction.
fn off_specification(
    run: &RunCoordinate,
    contradiction: StableFactContradiction,
    diagnostic: &str,
) -> IntegrityError {
    IntegrityError::malformed_stable_provenance(
        run.clone(),
        MalformedStableFault::OffSpecification(contradiction),
        diagnostic.to_string(),
    )
}

/// Build a `CampaignFactHeterogeneity` failure between a diverging run and the reference run.
fn heterogeneity(
    diverging: &StagedManifest,
    reference: &StagedManifest,
    contradiction: StableFactContradiction,
    diagnostic: &str,
) -> IntegrityError {
    IntegrityError::campaign_fact_heterogeneity(
        diverging.coordinate.clone(),
        reference.coordinate.clone(),
        contradiction,
        format!(
            "{diagnostic} (run at seq {} diverges from reference run at seq {})",
            diverging.seq, reference.seq
        ),
    )
}

/// Off-specification check of a parsed semver version fact against the frozen expected version.
fn check_version(
    run: &RunCoordinate,
    raw: &str,
    expected: &Version,
    fact: StableVersionFact,
) -> std::result::Result<(), IntegrityError> {
    let observed = Version::parse(raw).map_err(|error| {
        let fault = match fact {
            StableVersionFact::Cli => MalformedStableFault::UnparseableCliVersion {
                raw: raw.to_string(),
                error,
            },
            StableVersionFact::Standalone => MalformedStableFault::UnparseableStandaloneVersion {
                raw: raw.to_string(),
                error,
            },
        };
        IntegrityError::malformed_stable_provenance(
            run.clone(),
            fault,
            format!("version string {raw:?} is not valid semver"),
        )
    })?;
    if observed != *expected {
        let contradiction = match fact {
            StableVersionFact::Cli => StableFactContradiction::CliVersion {
                expected: expected.clone(),
                observed,
            },
            StableVersionFact::Standalone => StableFactContradiction::StandaloneVersion {
                expected: expected.clone(),
                observed,
            },
        };
        return Err(off_specification(
            run,
            contradiction,
            "parsed version is off the preregistered expected version",
        ));
    }
    Ok(())
}

/// Off-specification check of a raw version string against the frozen expected version string.
fn check_raw_version(
    run: &RunCoordinate,
    raw: &str,
    fact: StableVersionFact,
) -> std::result::Result<(), IntegrityError> {
    if raw != EXPECTED_VERSION {
        let contradiction = match fact {
            StableVersionFact::Cli => StableFactContradiction::CliVersionRaw {
                expected: EXPECTED_VERSION.to_string(),
                observed: raw.to_string(),
            },
            StableVersionFact::Standalone => StableFactContradiction::StandaloneVersionRaw {
                expected: EXPECTED_VERSION.to_string(),
                observed: raw.to_string(),
            },
        };
        return Err(off_specification(
            run,
            contradiction,
            "raw version string is off the preregistered expected version",
        ));
    }
    Ok(())
}

/// Off-specification check of a parsed release commit against the frozen expected commit.
fn check_release_commit(
    run: &RunCoordinate,
    raw: &str,
    expected: &ReleaseCommit,
) -> std::result::Result<(), IntegrityError> {
    let observed = ReleaseCommit::parse_canonical_hex(raw).map_err(|error| {
        IntegrityError::malformed_stable_provenance(
            run.clone(),
            MalformedStableFault::UnparseableReleaseCommit {
                raw: raw.to_string(),
                error,
            },
            format!("release commit hex {raw:?} is not canonical"),
        )
    })?;
    if observed != *expected {
        return Err(off_specification(
            run,
            StableFactContradiction::CliReleaseCommit {
                expected: *expected,
                observed,
            },
            "parsed release commit is off the preregistered expected commit",
        ));
    }
    Ok(())
}

/// Off-specification check of a parsed module WASM digest against the frozen committed hash.
fn check_wasm_sha256(
    run: &RunCoordinate,
    raw: &str,
    expected: &WasmSha256,
) -> std::result::Result<(), IntegrityError> {
    let observed = WasmSha256::parse_canonical_hex(raw).map_err(|error| {
        IntegrityError::malformed_stable_provenance(
            run.clone(),
            MalformedStableFault::UnparseableWasmSha256 {
                raw: raw.to_string(),
                error,
            },
            format!("module wasm sha256 hex {raw:?} is not canonical"),
        )
    })?;
    if observed != *expected {
        return Err(off_specification(
            run,
            StableFactContradiction::WasmSha256 {
                expected: *expected,
                observed,
            },
            "parsed module wasm sha256 is off the committed provenance hash",
        ));
    }
    Ok(())
}

/// Homogeneity check of one pinned environment path fact against the reference run's value.
fn check_homogeneous_path(
    manifest: &StagedManifest,
    reference: &StagedManifest,
    observed: &str,
    expected: &str,
    fact: StableEnvFact,
) -> std::result::Result<(), IntegrityError> {
    if observed != expected {
        let expected_path = std::path::PathBuf::from(expected);
        let observed_path = std::path::PathBuf::from(observed);
        let contradiction = match fact {
            StableEnvFact::NixStoreBinDir => StableFactContradiction::NixStoreBinDir {
                expected: expected_path,
                observed: observed_path,
            },
            StableEnvFact::CliExe => StableFactContradiction::CliExe {
                expected: expected_path,
                observed: observed_path,
            },
            StableEnvFact::StandaloneExe => StableFactContradiction::StandaloneExe {
                expected: expected_path,
                observed: observed_path,
            },
        };
        return Err(heterogeneity(
            manifest,
            reference,
            contradiction,
            "pinned distribution path is not homogeneous across the campaign",
        ));
    }
    Ok(())
}

/// Validate one run's per-run server-provenance facts for shape and internal relationship (never for
/// homogeneity): nonzero pid, a parseable listen address, a client URL derived from it, sibling data/keys
/// directories, and a resolved executable equal to the pinned standalone.
fn validate_server_provenance(
    run: &RunCoordinate,
    server: &crate::analysis::ingest::server_facts_dto::ServerFactsDto,
    standalone_exe: &str,
) -> std::result::Result<(), IntegrityError> {
    if server.pid == 0 {
        return Err(IntegrityError::server_provenance_shape(
            run.clone(),
            ServerProvenanceFault::ZeroPid { observed: 0 },
            "server pid is zero".to_string(),
        ));
    }

    let listen_addr = server.listen_addr.parse::<SocketAddr>().map_err(|_| {
        IntegrityError::server_provenance_shape(
            run.clone(),
            ServerProvenanceFault::UnparseableListenAddr {
                raw: server.listen_addr.clone(),
            },
            format!("listen address {:?} does not parse as host:port", server.listen_addr),
        )
    })?;

    let expected_client_url = format!("http://{listen_addr}");
    if server.client_url != expected_client_url {
        return Err(IntegrityError::server_provenance_shape(
            run.clone(),
            ServerProvenanceFault::ClientUrlMismatch {
                expected_authority: listen_addr,
                observed: server.client_url.clone(),
            },
            format!(
                "client URL {:?} is not the derived {expected_client_url:?}",
                server.client_url
            ),
        ));
    }

    let data_dir = Path::new(&server.data_dir);
    let keys_dir = Path::new(&server.keys_dir);
    let siblings = data_dir.file_name() == Some(std::ffi::OsStr::new("data"))
        && keys_dir.file_name() == Some(std::ffi::OsStr::new("keys"))
        && data_dir.parent() == keys_dir.parent();
    if !siblings {
        return Err(IntegrityError::server_provenance_shape(
            run.clone(),
            ServerProvenanceFault::DataKeysNotSiblings {
                data_dir: data_dir.to_path_buf(),
                keys_dir: keys_dir.to_path_buf(),
            },
            "data and keys directories are not sibling data/keys children of one root".to_string(),
        ));
    }

    if server.resolved_exe != standalone_exe {
        return Err(IntegrityError::server_provenance_shape(
            run.clone(),
            ServerProvenanceFault::ResolvedExeMismatch {
                expected: std::path::PathBuf::from(standalone_exe),
                observed: std::path::PathBuf::from(&server.resolved_exe),
            },
            "resolved server executable is not the pinned standalone executable".to_string(),
        ));
    }

    Ok(())
}

/// Stage 5: exact cell/block/dose census over the canonical slots. Reports the first failure in canonical
/// cell→role→block→dose order.
fn census(
    manifests: &[StagedManifest],
    dose_slots: &[Vec<StagedDose>],
) -> std::result::Result<(), IntegrityError> {
    let all_cells = Cell::all();

    // Static cell-array length drift against the canonical set.
    if CELL_COUNT != all_cells.len() {
        return Err(IntegrityError::cell_census(
            CellCensusFault::CountDrift {
                cell_count: CELL_COUNT,
                canonical_count: all_cells.len(),
            },
            format!(
                "the trusted cell-array length {CELL_COUNT} has drifted from the canonical cell count {}",
                all_cells.len()
            ),
        ));
    }

    // Every one of the nine cells must appear in at least one manifest.
    let observed: Vec<Cell> = all_cells
        .iter()
        .copied()
        .filter(|cell| {
            manifests
                .iter()
                .any(|manifest| manifest.coordinate.cell() == *cell)
        })
        .collect();
    if observed.len() != all_cells.len() {
        return Err(IntegrityError::cell_census(
            CellCensusFault::DistinctSet {
                expected: all_cells.clone(),
                observed,
            },
            "the manifests do not cover all nine preregistered cells".to_string(),
        ));
    }

    // Each `(cell, role)` must carry exactly one manifest per block, indices 0..REPETITION_BLOCKS.
    for (cell_idx, cell) in all_cells.iter().copied().enumerate() {
        for role in [RunRole::Arm, RunRole::Control] {
            for block in 0..REPETITION_BLOCKS_USIZE {
                let slot = run_slot_index(cell_idx, role_index(role), block);
                let occurrences = manifests
                    .iter()
                    .filter(|manifest| {
                        run_slot_index(
                            cell_index(manifest.coordinate.cell()),
                            role_index(manifest.coordinate.role()),
                            manifest.body.manifest.run.repetition_block as usize,
                        ) == slot
                    })
                    .count();
                if occurrences != 1 {
                    let fault = if occurrences == 0 {
                        BlockCensusFault::Missing {
                            block: block as u32,
                            expected_occurrences: 1,
                            observed_occurrences: 0,
                        }
                    } else {
                        BlockCensusFault::Duplicate {
                            block: block as u32,
                            expected_occurrences: 1,
                            observed_occurrences: occurrences,
                        }
                    };
                    return Err(IntegrityError::block_census(
                        cell,
                        role,
                        fault,
                        format!(
                            "cell/role block {block} has {occurrences} manifests, expected exactly one"
                        ),
                    ));
                }
            }
        }
    }

    // Each run must carry exactly one observation per dose, indices 1..=NUM_DOSES.
    for (cell_idx, cell) in all_cells.iter().copied().enumerate() {
        for role in [RunRole::Arm, RunRole::Control] {
            for block in 0..REPETITION_BLOCKS_USIZE {
                let run_slot = run_slot_index(cell_idx, role_index(role), block);
                for dose_zero_based in 0..NUM_DOSES_USIZE {
                    let occurrences = dose_slots[dose_slot_index(run_slot, dose_zero_based)].len();
                    if occurrences != 1 {
                        let dose_value = (dose_zero_based + 1) as u64;
                        let fault = if occurrences == 0 {
                            DoseCensusFault::Missing {
                                dose: dose_value,
                                expected_occurrences: 1,
                                observed_occurrences: 0,
                            }
                        } else {
                            DoseCensusFault::Duplicate {
                                dose: dose_value,
                                expected_occurrences: 1,
                                observed_occurrences: occurrences,
                            }
                        };
                        return Err(IntegrityError::dose_census(
                            canonical_coordinate(cell, role, block),
                            fault,
                            format!(
                                "run dose {dose_value} has {occurrences} observations, expected \
                                 exactly one"
                            ),
                        ));
                    }
                }
            }
        }
    }

    Ok(())
}

/// The trusted coordinate for a canonical `(cell, role, block)` position. `block` is a census loop index
/// in `0..REPETITION_BLOCKS`, so the range check is infallible.
fn canonical_coordinate(cell: Cell, role: RunRole, block: usize) -> RunCoordinate {
    RunCoordinate::from_parts(
        cell,
        role,
        RepetitionBlockIndex::try_new(block as u32)
            .expect("a canonical census block index is in 0..REPETITION_BLOCKS"),
    )
}

/// Stage 6: bind each record's sequence position to the seed-derived preregistered schedule grammar, so
/// the trusted collection order provably reflects the randomized schedule (the global `Cell × block`
/// permutation, the seed-selected adjacent arm/control order, and the manifest-then-canonical-doses
/// within-run order) rather than the generator's emission order. Reconstructs the grammar from the
/// campaign's proven-homogeneous schedule seed and compares it against the ascending-sequence records,
/// reporting the first divergence as a typed [`ScheduleOrderMismatch`] classified by the narrowest
/// differing structure (block coordinate, then role, then within-run record slot).
///
/// [`ScheduleOrderMismatch`]: super::integrity_error::IntegrityError
fn validate_schedule_order(
    ordered: &[&WireRecordDto],
    manifests: &[StagedManifest],
) -> std::result::Result<(), IntegrityError> {
    // The schedule seed is proven homogeneous across every manifest in stage 4, so any manifest's seed is
    // the campaign's seed; take the first in sequence order.
    let seed = ScheduleSeed::new(
        manifests
            .first()
            .expect("a complete campaign has 540 manifests, so at least one")
            .body
            .manifest
            .schedule_seed,
    );

    // Reconstruct the preregistered grammar from the production schedule — never a parallel definition of
    // it: the seed-derived global block permutation, each block's seed-selected adjacent run order, and the
    // manifest-then-canonical-doses within-run order.
    let schedule = Schedule::preregistered();
    let permutation = schedule.randomized_block_order(seed);
    let grammar = schedule_grammar(&permutation, seed);

    // Census (stage 5) proved exactly one manifest per canonical run and one observation per run dose, so
    // the campaign holds exactly [`RECORD_COUNT`] records. Convert the ascending-sequence slice to a fixed
    // array of that cardinality — a fail-loud invariant check, not a campaign fault: census precedes this
    // stage, so a regressed count is a stage bug — so zipping it against the equally-fixed grammar pairs
    // every record with its expected slot, with neither side silently truncated.
    let ordered: &[&WireRecordDto; RECORD_COUNT] = ordered
        .try_into()
        .expect("census proved the campaign holds exactly RECORD_COUNT records");

    // Compare each record, in ascending sequence position, against the grammar record that position must
    // hold, reporting the first divergence classified by the narrowest differing structure: block
    // coordinate, then run role, then within-run slot.
    for (position, (record, expected)) in ordered.iter().copied().zip(grammar.iter()).enumerate() {
        let position =
            RecordSeq::new(u64::try_from(position).expect("a campaign record position fits u64"));
        let (observed_role, observed_block, observed_slot) =
            decode_scheduled_record(record, &permutation);

        // Priority 1 — block coordinate. A swapped block, or a run torn out of its matched pair, puts a
        // foreign block's record where this position's block belongs.
        if observed_block != expected.block {
            return Err(IntegrityError::schedule_order_mismatch(
                ScheduleOrderFault::BlockOutOfScheduleOrder {
                    position,
                    expected: expected.block,
                    observed: observed_block,
                },
                format!(
                    "record at sequence position {} belongs to a block the seed-derived schedule \
                     does not place there",
                    position.get()
                ),
            ));
        }

        // Priority 2 — run role. Within the correct block, the two matched runs execute in the
        // seed-selected adjacent order.
        if observed_role != expected.role {
            return Err(IntegrityError::schedule_order_mismatch(
                ScheduleOrderFault::RoleOutOfScheduleOrder {
                    position,
                    block: expected.block,
                    expected: expected.role,
                    observed: observed_role,
                },
                format!(
                    "record at sequence position {} carries the {:?} role where the seed-selected \
                     order expects {:?}",
                    position.get(),
                    observed_role,
                    expected.role
                ),
            ));
        }

        // Priority 3 — within-run slot. The run's manifest leads, then its ten cumulative doses in
        // canonical ladder order.
        if observed_slot != expected.slot {
            return Err(IntegrityError::schedule_order_mismatch(
                ScheduleOrderFault::RunRecordOutOfScheduleOrder {
                    position,
                    run: expected.run.clone(),
                    expected: expected.slot,
                    observed: observed_slot,
                },
                format!(
                    "record at sequence position {} is out of the manifest-then-doses within-run \
                     order",
                    position.get()
                ),
            ));
        }
    }

    Ok(())
}

/// One record the seed-derived schedule grammar expects at a single campaign sequence position: its block
/// coordinate, its run role, the canonical run coordinate, and the within-run slot. Reconstructed in
/// grammar order from the production schedule and compared position-for-position against the
/// sequence-sorted wire records, so the trusted collection order is bound to the preregistered randomized
/// schedule rather than to the generator's emission order.
struct ScheduledRecord {
    block: BlockCoordinate,
    role: RunRole,
    run: RunCoordinate,
    slot: RecordSlot,
}

/// Reconstruct the flat expected record grammar from the seed-permuted block order: for each block in the
/// seed permutation, for each run in the block's seed-selected order, the run's manifest followed by its
/// ten cumulative doses in canonical ladder order. The complete campaign has fixed cardinality
/// [`RECORD_COUNT`], so the grammar is sealed heap-first into a boxed fixed array — the count is a
/// property of the type, not a runtime-checked `Vec` length.
fn schedule_grammar(
    permutation: &[BlockRun],
    seed: ScheduleSeed,
) -> Box<[ScheduledRecord; RECORD_COUNT]> {
    let mut grammar: Vec<ScheduledRecord> = Vec::with_capacity(RECORD_COUNT);
    for block in permutation {
        let block_coordinate = BlockCoordinate::of(block);
        for run in block.ordered_runs(seed) {
            let role = run.role();
            let run_coordinate = RunCoordinate::new(block, role);
            grammar.push(ScheduledRecord {
                block: block_coordinate,
                role,
                run: run_coordinate.clone(),
                slot: RecordSlot::Manifest,
            });
            for dose in DoseIndex::ALL {
                grammar.push(ScheduledRecord {
                    block: block_coordinate,
                    role,
                    run: run_coordinate.clone(),
                    slot: RecordSlot::Dose(dose),
                });
            }
        }
    }
    // Heap-first: seal the grammar Vec into a boxed fixed array by pointer reinterpret — the fixed
    // cardinality proof stays in this fallible conversion and no by-value grammar array is materialized.
    grammar
        .into_boxed_slice()
        .try_into()
        .ok()
        .expect("the preregistered schedule yields exactly RECORD_COUNT grammar records")
}

/// Decode the schedule-relevant identity of one wire record: its run role, the scheduled block coordinate
/// it names, and its within-run slot. The block coordinate is constructed at decode by locating the
/// record's `(cell, block)` in the seed permutation — where every canonical pairing appears exactly once —
/// so the observed identity is the same typed [`BlockCoordinate`] the grammar carries, compared for
/// equality rather than by a raw index. Every field was proven well-formed by the per-record stage and
/// range-checked by census, so the coordinate rebuild and permutation lookup are infallible here.
fn decode_scheduled_record(
    record: &WireRecordDto,
    permutation: &[BlockRun],
) -> (RunRole, BlockCoordinate, RecordSlot) {
    match record {
        WireRecordDto::Manifest { body, .. } => {
            let run = &body.manifest.run;
            let coordinate = build_coordinate(run)
                .expect("stage 2 proved every manifest coordinate is well-formed");
            let block =
                scheduled_block_coordinate(permutation, coordinate.cell(), run.repetition_block);
            (coordinate.role(), block, RecordSlot::Manifest)
        }
        WireRecordDto::Dose { body, .. } => {
            let coordinate_dto = &body.observation.coordinate;
            let coordinate = build_coordinate(&coordinate_dto.run)
                .expect("stage 2 proved every dose coordinate is well-formed");
            let block = scheduled_block_coordinate(
                permutation,
                coordinate.cell(),
                coordinate_dto.run.repetition_block,
            );
            let dose = DoseIndex::ALL[(coordinate_dto.dose - 1) as usize];
            (coordinate.role(), block, RecordSlot::Dose(dose))
        }
    }
}

/// The canonical block coordinate for a census-proven record's `(cell, block)`. Every valid pairing
/// appears exactly once in the seed permutation, so the look-up is total.
fn scheduled_block_coordinate(
    permutation: &[BlockRun],
    cell: Cell,
    block_index: u32,
) -> BlockCoordinate {
    let block = permutation
        .iter()
        .find(|candidate| candidate.cell() == cell && candidate.block_index() == block_index)
        .expect("census proved every record names a canonical scheduled block");
    BlockCoordinate::of(block)
}

/// Stage 7: prove each run's cumulative logical ladder strictly monotonic and each block's arm and
/// control logical ladders equal, dose by dose. Runs after census, so every dose slot holds exactly one
/// staged dose.
fn validate_ladders(dose_slots: &[Vec<StagedDose>]) -> std::result::Result<(), IntegrityError> {
    let all_cells = Cell::all();

    // Per-run strict monotonicity.
    for (cell_idx, cell) in all_cells.iter().copied().enumerate() {
        for role in [RunRole::Arm, RunRole::Control] {
            for block in 0..REPETITION_BLOCKS_USIZE {
                let run_slot = run_slot_index(cell_idx, role_index(role), block);
                for dose_zero_based in 1..NUM_DOSES_USIZE {
                    let previous =
                        sole_dose(dose_slots, run_slot, dose_zero_based - 1).logical_n;
                    let current = sole_dose(dose_slots, run_slot, dose_zero_based).logical_n;
                    if current <= previous {
                        return Err(IntegrityError::non_monotonic_ladder(
                            canonical_coordinate(cell, role, block),
                            DoseIndex::ALL[dose_zero_based],
                            previous,
                            current,
                            format!(
                                "cumulative logical_n {current} is not strictly greater than the \
                                 previous dose's {previous}"
                            ),
                        ));
                    }
                }
            }
        }
    }

    // Per-block arm/control ladder agreement.
    for (cell_idx, cell) in all_cells.iter().copied().enumerate() {
        for block in 0..REPETITION_BLOCKS_USIZE {
            let arm_slot = run_slot_index(cell_idx, role_index(RunRole::Arm), block);
            let control_slot = run_slot_index(cell_idx, role_index(RunRole::Control), block);
            for dose_zero_based in 0..NUM_DOSES_USIZE {
                let arm_logical_n = sole_dose(dose_slots, arm_slot, dose_zero_based).logical_n;
                let control_logical_n =
                    sole_dose(dose_slots, control_slot, dose_zero_based).logical_n;
                if arm_logical_n != control_logical_n {
                    return Err(IntegrityError::cardinality_ladder_mismatch(
                        canonical_coordinate(cell, RunRole::Arm, block),
                        canonical_coordinate(cell, RunRole::Control, block),
                        DoseIndex::ALL[dose_zero_based],
                        arm_logical_n,
                        control_logical_n,
                        format!(
                            "arm logical_n {arm_logical_n} disagrees with control logical_n \
                             {control_logical_n}"
                        ),
                    ));
                }
            }
        }
    }

    Ok(())
}

/// The single staged dose at a run slot's dose position. Census proved exactly one, so this is total.
fn sole_dose(dose_slots: &[Vec<StagedDose>], run_slot: usize, dose_zero_based: usize) -> &StagedDose {
    let slot = &dose_slots[dose_slot_index(run_slot, dose_zero_based)];
    slot.first()
        .expect("census proved exactly one observation per run dose")
}

/// Stage 8: mint the nine trusted cell datasets by draining each canonical dose slot's sole staged dose.
/// The staged manifests are threaded in so each minted run carries its manifest's collection-order
/// sequence.
fn mint_cells(
    dose_slots: &mut [Vec<StagedDose>],
    manifests: &[StagedManifest],
) -> std::result::Result<Box<[CellDataset; CELL_COUNT]>, IntegrityError> {
    let all_cells = Cell::all();
    let mut cells: Vec<CellDataset> = Vec::with_capacity(CELL_COUNT);
    for (cell_idx, cell) in all_cells.iter().copied().enumerate() {
        let mut blocks: Vec<MatchedBlock> = Vec::with_capacity(REPETITION_BLOCKS_USIZE);
        for block in 0..REPETITION_BLOCKS_USIZE {
            let arm = mint_run::<ArmRun>(cell, RunRole::Arm, cell_idx, block, dose_slots, manifests)?;
            let control =
                mint_run::<ControlRun>(cell, RunRole::Control, cell_idx, block, dose_slots, manifests)?;
            blocks.push(MatchedBlock::new(arm, control));
        }
        // Heap-first: seal the block Vec into a boxed fixed array by pointer reinterpret — the fixed
        // cardinality proof stays in this fallible conversion and no by-value block array is materialized.
        let blocks: Box<[MatchedBlock; REPETITION_BLOCKS_USIZE]> = blocks
            .into_boxed_slice()
            .try_into()
            .ok()
            .expect("exactly REPETITION_BLOCKS matched blocks were minted for the cell");
        cells.push(CellDataset::new(cell, blocks));
    }
    Ok(cells
        .into_boxed_slice()
        .try_into()
        .ok()
        .expect("exactly CELL_COUNT cell datasets were minted"))
}

/// Mint one role-typed trusted run for a canonical `(cell, role, block)` position by draining its ten
/// sole staged doses. The coordinate is built with the matching role, so `TrustedRun::mint`'s role check
/// always passes for a well-formed campaign; a mismatch would surface as a typed
/// [`RunRoleMismatch`](super::integrity_error::IntegrityError).
fn mint_run<R: RunKind>(
    cell: Cell,
    role: RunRole,
    cell_idx: usize,
    block: usize,
    dose_slots: &mut [Vec<StagedDose>],
    manifests: &[StagedManifest],
) -> std::result::Result<TrustedRun<R>, IntegrityError> {
    let coordinate = canonical_coordinate(cell, role, block);
    let run_slot = run_slot_index(cell_idx, role_index(role), block);
    let mut doses: Vec<TrustedDose> = Vec::with_capacity(NUM_DOSES_USIZE);
    for dose_zero_based in 0..NUM_DOSES_USIZE {
        let staged = dose_slots[dose_slot_index(run_slot, dose_zero_based)]
            .pop()
            .expect("census proved exactly one observation per run dose");
        doses.push(TrustedDose::new(
            DoseIndex::ALL[dose_zero_based],
            staged.logical_n,
            staged.latencies,
            staged.summary,
            staged.cardinalities,
            staged.events,
        ));
    }
    // Heap-first: seal the dose Vec into a boxed fixed array by pointer reinterpret — the fixed cardinality
    // proof stays in this fallible conversion and no by-value dose array is materialized.
    let doses: Box<[TrustedDose; NUM_DOSES_USIZE]> = doses
        .into_boxed_slice()
        .try_into()
        .ok()
        .expect("exactly NUM_DOSES trusted doses were minted for the run");
    // The run's manifest sequence is its collection-order anchor. Census proved exactly one manifest per
    // canonical run coordinate, so this lookup is total.
    let manifest_seq = RecordSeq::new(
        manifests
            .iter()
            .find(|manifest| manifest.coordinate == coordinate)
            .expect("census proved exactly one manifest per canonical run coordinate")
            .seq,
    );
    TrustedRun::<R>::mint(coordinate, doses, manifest_seq)
}

// `pub(crate)` (test-only) so the `campaign_builder` fixture reachable through it is nameable from the
// campaign classifier's graph-level tests; the category-proof submodules remain private.
#[cfg(test)]
pub(crate) mod tests;
