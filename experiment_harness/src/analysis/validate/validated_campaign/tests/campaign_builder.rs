//! A programmatic fixture owning the complete, spec-correct campaign as a `Vec<WireRecordDto>`.
//!
//! Every field is set to the exact value the total-validation fold requires: canonical-hex identities,
//! `dose * BATCH_SIZE` cumulative logical counts, R-1 summaries recomputed from the raw latencies, the
//! deterministic [`PhysicalCardinalities::expected`] footprint, an insert/delete/net-delta-consistent
//! event stream, homogeneous provenance, and contiguous sequence numbers. Category proofs own a
//! [`CampaignFixture`], mutate one record through it, and fold — so exactly one obligation fails.

use std::net::SocketAddr;

use crate::analysis::ingest::cell_dto::CellDto;
use crate::analysis::ingest::chronicle_physical_rows_dto::ChroniclePhysicalRowsDto;
use crate::analysis::ingest::distribution_facts_dto::DistributionFactsDto;
use crate::analysis::ingest::dose_coordinate_dto::DoseCoordinateDto;
use crate::analysis::ingest::dose_observation_dto::DoseObservationDto;
use crate::analysis::ingest::event_evidence_dto::EventEvidenceDto;
use crate::analysis::ingest::key_scoped_arm_dto::KeyScopedArmDto;
use crate::analysis::ingest::latency_summary_dto::LatencySummaryDto;
use crate::analysis::ingest::manifest_body_dto::ManifestBodyDto;
use crate::analysis::ingest::manifest_reference_dto::ManifestReferenceDto;
use crate::analysis::ingest::message_physical_rows_dto::MessagePhysicalRowsDto;
use crate::analysis::ingest::module_facts_dto::ModuleFactsDto;
use crate::analysis::ingest::observation_body_dto::ObservationBodyDto;
use crate::analysis::ingest::physical_cardinalities_dto::PhysicalCardinalitiesDto;
use crate::analysis::ingest::preregistered_parameters_dto::PreregisteredParametersDto;
use crate::analysis::ingest::record_id_dto::RecordIdDto;
use crate::analysis::ingest::record_kind_dto::RecordKindDto;
use crate::analysis::ingest::role_identities_dto::RoleIdentitiesDto;
use crate::analysis::ingest::run_coordinate_dto::RunCoordinateDto;
use crate::analysis::ingest::run_role_dto::RunRoleDto;
use crate::analysis::ingest::server_facts_dto::ServerFactsDto;
use crate::analysis::ingest::table_scoped_arm_dto::TableScopedArmDto;
use crate::analysis::ingest::validated_run_manifest_dto::ValidatedRunManifestDto;
use crate::analysis::ingest::wire_record_dto::WireRecordDto;
use crate::analysis::validate::record_slot::RecordSlot;
use crate::dataset::dose_index::DoseIndex;
use crate::dataset::physical_cardinalities::PhysicalCardinalities;
use crate::manifest::database_identity::DatabaseIdentity;
use crate::manifest::run_coordinate::RunCoordinate;
use crate::manifest::schedule_seed::ScheduleSeed;
use crate::module_artifact::module_wasm_sha256::MODULE_WASM_SHA256;
use crate::observation::latency_sample::LatencySample;
use crate::observation::latency_summary::LatencySummary;
use crate::observation::raw_latencies::RawLatencies;
use crate::params::{
    BATCH_DELAY_MS, BATCH_SIZE, CONFIRMED_READS, EXPECTED_RELEASE_COMMIT, EXPECTED_VERSION,
    NUM_DOSES, REPETITION_BLOCKS,
};
use crate::plan::cell::Cell;
use crate::plan::key_scoped_arm::KeyScopedArm;
use crate::plan::run_role::RunRole;
use crate::plan::schedule::Schedule;
use crate::plan::table_scoped_arm::TableScopedArm;

/// The one schedule seed shared by every run (proven homogeneous by the fold).
const SCHEDULE_SEED: u64 = 7_654_321;
/// A parseable listen authority; the client URL is derived from its `SocketAddr` display exactly as the
/// fold derives it.
const LISTEN_ADDR: &str = "127.0.0.1:3000";
/// The pinned Nix store bin dir shared by every run (proven homogeneous by the fold).
const NIX_STORE_BIN_DIR: &str = "/nix/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-stdb/bin";
/// The pinned CLI executable path shared by every run.
const CLI_EXE: &str = "/nix/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-stdb/bin/spacetimedb-cli";
/// The pinned standalone executable path shared by every run; the server's resolved exe equals it.
const STANDALONE_EXE: &str =
    "/nix/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-stdb/bin/spacetimedb-standalone";
/// The server data directory; a sibling `data` child of a shared root.
const DATA_DIR: &str = "/var/lib/stdb/data";
/// The server keys directory; a sibling `keys` child of the same root.
const KEYS_DIR: &str = "/var/lib/stdb/keys";
/// A nonzero server pid.
const SERVER_PID: u32 = 4242;

/// Owns the complete, spec-correct campaign records and is the sole handle the proofs use to read,
/// mutate, and surrender them. A category proof builds a [`Self::valid`] fixture, mutates one record via
/// [`Self::records_mut`] (or drops a cell and [`Self::renumber`]s), and folds [`Self::into_records`].
pub(crate) struct CampaignFixture {
    records: Vec<WireRecordDto>,
}

impl CampaignFixture {
    /// The complete, spec-correct campaign: 9 cells × 30 blocks × 2 roles manifests, each followed by
    /// its ten cumulative-dose observations, with contiguous sequence numbers over `0..5940`, emitted in
    /// the exact production schedule order.
    ///
    /// Records are laid down in the same order the fold's schedule-order stage reconstructs from the seed:
    /// the seed-permuted global block order, each block's seed-selected adjacent arm/control order, and
    /// each run's manifest followed by its ten cumulative doses in canonical ladder order. Building through
    /// the production [`Schedule`] — never a parallel test-only grammar — means a spec-correct campaign is
    /// one the stage accepts record-for-record, so an ordering proof perturbs this true order to fail it.
    pub(crate) fn valid() -> Self {
        let latencies = latency_nanos();
        let summary = summary_dto(&latencies);
        let seed = Self::schedule_seed();
        let mut records = Vec::new();
        let mut seq = 0u64;
        for block in Schedule::preregistered().randomized_block_order(seed) {
            let cell = block.cell();
            let block_index = block.block_index();
            for run in block.ordered_runs(seed) {
                let role = run.role();
                records.push(manifest_record(seq, cell, role, block_index));
                seq += 1;
                for dose in DoseIndex::ALL {
                    records.push(dose_record(
                        seq,
                        cell,
                        role,
                        block_index,
                        dose,
                        latencies.clone(),
                        summary,
                    ));
                    seq += 1;
                }
            }
        }
        Self { records }
    }

    /// A read-only borrow of the current records.
    pub(super) fn records(&self) -> &[WireRecordDto] {
        &self.records
    }

    /// A mutable borrow of the records, so a proof can perturb a single field or drop records.
    pub(super) fn records_mut(&mut self) -> &mut Vec<WireRecordDto> {
        &mut self.records
    }

    /// Reassign contiguous sequence numbers `0..len` in the records' current order, restoring stage-1
    /// contiguity after a mutation that added or removed records.
    pub(super) fn renumber(&mut self) {
        for (index, record) in self.records.iter_mut().enumerate() {
            let seq = index as u64;
            match record {
                WireRecordDto::Manifest { record, .. } => record.seq = seq,
                WireRecordDto::Dose { record, .. } => record.seq = seq,
            }
        }
    }

    /// The trusted [`Cell`] a record belongs to, decoded from its wire coordinate.
    pub(super) fn record_cell(record: &WireRecordDto) -> Cell {
        let coordinate = match record {
            WireRecordDto::Manifest { body, .. } => &body.manifest.run,
            WireRecordDto::Dose { body, .. } => &body.observation.coordinate.run,
        };
        super::super::cell_from_dto(coordinate.cell)
    }

    /// The index of the manifest record for the canonical `(cell, role, block)`, located by decoding each
    /// record's wire coordinate rather than assuming emission order — so an ordering-sensitive proof
    /// addresses a run by identity and stays correct under the seed-randomized record order [`Self::valid`]
    /// emits.
    pub(super) fn manifest_index(&self, cell: Cell, role: RunRole, block: u32) -> usize {
        self.records
            .iter()
            .position(|record| match record {
                WireRecordDto::Manifest { body, .. } => {
                    Self::coordinate_is(&body.manifest.run, cell, role, block)
                }
                WireRecordDto::Dose { .. } => false,
            })
            .expect("the fixture carries a manifest for every canonical (cell, role, block)")
    }

    /// The index of the cumulative-dose observation for the canonical `(cell, role, block, dose)`, located
    /// by identity for the same reason as [`Self::manifest_index`].
    pub(super) fn dose_index(&self, cell: Cell, role: RunRole, block: u32, dose: DoseIndex) -> usize {
        self.records
            .iter()
            .position(|record| match record {
                WireRecordDto::Dose { body, .. } => {
                    Self::coordinate_is(&body.observation.coordinate.run, cell, role, block)
                        && body.observation.coordinate.dose == dose.get()
                }
                WireRecordDto::Manifest { .. } => false,
            })
            .expect("the fixture carries an observation for every canonical (cell, role, block, dose)")
    }

    /// The record index of the `n`-th manifest in emission (sequence) order, 0-based — for a proof that
    /// must address the fold's reference (first) manifest and a distinct later one without assuming which
    /// run the seed schedule places at either position.
    pub(super) fn nth_manifest_index(&self, n: usize) -> usize {
        self.records
            .iter()
            .enumerate()
            .filter_map(|(index, record)| match record {
                WireRecordDto::Manifest { .. } => Some(index),
                WireRecordDto::Dose { .. } => None,
            })
            .nth(n)
            .expect("the fixture carries one manifest per canonical run")
    }

    /// The full schedule identity a record carries: its trusted run coordinate `(cell, role, block)`, its
    /// 0-based repetition block, and its within-run slot — decoded from the wire coordinate. An ordering
    /// proof captures a pristine expected identity with this *before* perturbing the record order, so the
    /// expected descriptor is derived from the record itself rather than re-encoded and able to drift from
    /// the fold's own decode.
    pub(super) fn record_identity(record: &WireRecordDto) -> (RunCoordinate, u32, RecordSlot) {
        match record {
            WireRecordDto::Manifest { body, .. } => {
                let run = &body.manifest.run;
                (Self::decode_coordinate(run), run.repetition_block, RecordSlot::Manifest)
            }
            WireRecordDto::Dose { body, .. } => {
                let coordinate = &body.observation.coordinate;
                (
                    Self::decode_coordinate(&coordinate.run),
                    coordinate.run.repetition_block,
                    RecordSlot::Dose(DoseIndex::ALL[(coordinate.dose - 1) as usize]),
                )
            }
        }
    }

    /// Whether a wire run coordinate decodes to the trusted `(cell, role, block)`.
    fn coordinate_is(run: &RunCoordinateDto, cell: Cell, role: RunRole, block: u32) -> bool {
        let coordinate = Self::decode_coordinate(run);
        coordinate.cell() == cell && coordinate.role() == role && run.repetition_block == block
    }

    /// The trusted run coordinate a well-formed wire coordinate decodes to, through the fold's own
    /// `build_coordinate`. The fixture's coordinates are all spec-correct, so its range check never fails
    /// here.
    fn decode_coordinate(run: &RunCoordinateDto) -> RunCoordinate {
        super::super::build_coordinate(run).expect("the fixture's coordinates are all well-formed")
    }

    /// Surrender the records to the fold under test.
    pub(crate) fn into_records(self) -> Vec<WireRecordDto> {
        self.records
    }

    /// The one schedule seed the fixture stamps into every manifest and reference, as the trusted
    /// [`ScheduleSeed`] the fold reconstructs. An ordering proof reconstructs the exact
    /// [`ManifestReferenceIdentity`](super::super::manifest_reference_identity::ManifestReferenceIdentity)
    /// through this method rather than re-encoding the literal.
    pub(super) fn schedule_seed() -> ScheduleSeed {
        ScheduleSeed::new(SCHEDULE_SEED)
    }

    /// The one canonical database identity the fixture stamps into every manifest and reference, as the
    /// trusted [`DatabaseIdentity`] the fold reconstructs — the single canonical-hex value, parsed once
    /// through the domain owner so the fixture and a proof cannot drift on it.
    pub(super) fn database_identity() -> DatabaseIdentity {
        DatabaseIdentity::parse_canonical_hex(&identity_hex())
            .expect("the fixture identity is canonical lowercase hex of the exact length")
    }
}

/// A manifest wire record for one run coordinate.
fn manifest_record(seq: u64, cell: Cell, role: RunRole, block: u32) -> WireRecordDto {
    WireRecordDto::Manifest {
        record: RecordIdDto {
            seq,
            kind: RecordKindDto::Manifest,
        },
        body: ManifestBodyDto {
            reference: reference_dto(cell, role, block),
            manifest: ValidatedRunManifestDto {
                run: coordinate_dto(cell, role, block),
                schedule_seed: SCHEDULE_SEED,
                parameters: parameters_dto(),
                distribution: distribution_dto(),
                server: server_dto(),
                module: module_dto(),
                role_identities: role_identities_dto(),
            },
        },
    }
}

/// A cumulative-dose observation wire record for one run coordinate and dose.
fn dose_record(
    seq: u64,
    cell: Cell,
    role: RunRole,
    block: u32,
    dose: DoseIndex,
    latencies: Vec<u128>,
    summary: LatencySummaryDto,
) -> WireRecordDto {
    WireRecordDto::Dose {
        record: RecordIdDto {
            seq,
            kind: RecordKindDto::Dose(dose.get()),
        },
        body: ObservationBodyDto {
            observation: DoseObservationDto {
                manifest_ref: reference_dto(cell, role, block),
                coordinate: DoseCoordinateDto {
                    run: coordinate_dto(cell, role, block),
                    dose: dose.get(),
                    driving_role_tag: cell
                        .growth_regime()
                        .driving_role()
                        .canonical_tag()
                        .to_string(),
                    logical_n: dose.cumulative_driving_rows(),
                },
                cardinalities: cardinalities_dto(cell, dose),
                latencies,
                summary,
                events: events_dto(),
            },
        },
    }
}

/// The manifest reference identity components for one run coordinate.
fn reference_dto(cell: Cell, role: RunRole, block: u32) -> ManifestReferenceDto {
    ManifestReferenceDto {
        run: coordinate_dto(cell, role, block),
        database_identity: identity_hex(),
        schedule_seed: SCHEDULE_SEED,
    }
}

/// The wire run coordinate for a trusted `(cell, role, block)` position.
fn coordinate_dto(cell: Cell, role: RunRole, block: u32) -> RunCoordinateDto {
    RunCoordinateDto {
        cell: cell_to_dto(cell),
        role: role_to_dto(role),
        repetition_block: block,
    }
}

/// The preregistered parameters, exactly the frozen constants.
fn parameters_dto() -> PreregisteredParametersDto {
    PreregisteredParametersDto {
        batch_size: BATCH_SIZE,
        num_doses: NUM_DOSES,
        dose_ladder: (1..=NUM_DOSES).map(|dose| dose * BATCH_SIZE).collect(),
        batch_delay_ms: BATCH_DELAY_MS,
        repetition_blocks: REPETITION_BLOCKS,
        confirmed_reads: CONFIRMED_READS,
    }
}

/// The pinned distribution facts, homogeneous across runs and on the frozen spec.
fn distribution_dto() -> DistributionFactsDto {
    DistributionFactsDto {
        nix_store_bin_dir: NIX_STORE_BIN_DIR.to_string(),
        cli_exe: CLI_EXE.to_string(),
        cli_version: EXPECTED_VERSION.to_string(),
        cli_release_commit: EXPECTED_RELEASE_COMMIT.to_string(),
        cli_version_raw: EXPECTED_VERSION.to_string(),
        standalone_exe: STANDALONE_EXE.to_string(),
        standalone_version: EXPECTED_VERSION.to_string(),
        standalone_version_raw: EXPECTED_VERSION.to_string(),
    }
}

/// The per-run server facts: nonzero pid, parseable listen authority, derived client URL, sibling
/// data/keys directories, and a resolved exe equal to the pinned standalone.
fn server_dto() -> ServerFactsDto {
    let listen_addr = LISTEN_ADDR
        .parse::<SocketAddr>()
        .expect("the fixture listen address is a valid socket address");
    ServerFactsDto {
        pid: SERVER_PID,
        resolved_exe: STANDALONE_EXE.to_string(),
        listen_addr: LISTEN_ADDR.to_string(),
        client_url: format!("http://{listen_addr}"),
        data_dir: DATA_DIR.to_string(),
        keys_dir: KEYS_DIR.to_string(),
    }
}

/// The module facts: the committed WASM digest and the server-issued database identity.
fn module_dto() -> ModuleFactsDto {
    ModuleFactsDto {
        wasm_sha256: module_wasm_hex(),
        database_identity: identity_hex(),
    }
}

/// The per-run role-identity evidence: two distinct canonical-hex identities. `check_role_identities`
/// is not yet wired into this fold (Phase 2), so these values are not yet cross-checked against the
/// fixture's schedule seed/cell derivation; they are kept distinct only to avoid an incidentally
/// colliding fixture.
fn role_identities_dto() -> RoleIdentitiesDto {
    RoleIdentitiesDto {
        measured: "1".repeat(DatabaseIdentity::CANONICAL_HEX_LEN),
        growth: "2".repeat(DatabaseIdentity::CANONICAL_HEX_LEN),
    }
}

/// The deterministic physical cardinalities for a cell and dose, mirrored to the wire family.
fn cardinalities_dto(cell: Cell, dose: DoseIndex) -> PhysicalCardinalitiesDto {
    match PhysicalCardinalities::expected(cell, dose) {
        PhysicalCardinalities::Message(rows) => {
            PhysicalCardinalitiesDto::Message(MessagePhysicalRowsDto {
                message_rows: rows.message_rows(),
            })
        }
        PhysicalCardinalities::Chronicle(rows) => {
            PhysicalCardinalitiesDto::Chronicle(ChroniclePhysicalRowsDto {
                visibility_rows: rows.visibility_rows(),
                message_rows: rows.message_rows(),
            })
        }
    }
}

/// An insert/delete/net-delta-consistent event stream: `inserts − deletes == delivered_net_row_delta`.
fn events_dto() -> EventEvidenceDto {
    EventEvidenceDto {
        delivered_net_row_delta: 3,
        inserts: 3,
        deletes: 0,
        updates: 1,
    }
}

/// Exactly `BATCH_SIZE` strictly-positive nanosecond latency samples.
fn latency_nanos() -> Vec<u128> {
    (1..=u128::from(BATCH_SIZE)).collect()
}

/// The exact R-1 median/IQR summary recomputed from the raw latencies, so the recorded summary equals
/// the fold's recomputation.
fn summary_dto(latencies: &[u128]) -> LatencySummaryDto {
    let samples = latencies
        .iter()
        .map(|nanos| LatencySample::from_nanos(*nanos))
        .collect();
    let raw = RawLatencies::sealed(samples).expect("the fixture supplies exactly BATCH_SIZE samples");
    let summary = LatencySummary::from_raw(&raw);
    LatencySummaryDto {
        median_nanos: summary.median_nanos(),
        iqr_nanos: summary.iqr_nanos(),
    }
}

/// A canonical lowercase-hex database identity of the exact canonical length (all-zero nibbles).
fn identity_hex() -> String {
    "0".repeat(DatabaseIdentity::CANONICAL_HEX_LEN)
}

/// The canonical lowercase-hex rendering of the committed module WASM digest.
fn module_wasm_hex() -> String {
    let mut hex = String::with_capacity(MODULE_WASM_SHA256.len() * 2);
    for byte in MODULE_WASM_SHA256 {
        hex.push_str(&format!("{byte:02x}"));
    }
    hex
}

/// The wire mirror of a trusted cell.
fn cell_to_dto(cell: Cell) -> CellDto {
    match cell {
        Cell::TableScopedUnrelated(arm) => CellDto::TableScopedUnrelated(table_arm_to_dto(arm)),
        Cell::KeyScopedUnrelated(arm) => CellDto::KeyScopedUnrelated(key_arm_to_dto(arm)),
        Cell::KeyScopedOwnSlice(arm) => CellDto::KeyScopedOwnSlice(key_arm_to_dto(arm)),
    }
}

/// The wire mirror of a trusted table-scoped arm.
fn table_arm_to_dto(arm: TableScopedArm) -> TableScopedArmDto {
    match arm {
        TableScopedArm::ProceduralRange => TableScopedArmDto::ProceduralRange,
        TableScopedArm::QueryFull => TableScopedArmDto::QueryFull,
        TableScopedArm::QuerySemijoin => TableScopedArmDto::QuerySemijoin,
        TableScopedArm::QueryFullPk => TableScopedArmDto::QueryFullPk,
        TableScopedArm::QuerySemijoinPk => TableScopedArmDto::QuerySemijoinPk,
    }
}

/// The wire mirror of a trusted key-scoped arm.
fn key_arm_to_dto(arm: KeyScopedArm) -> KeyScopedArmDto {
    match arm {
        KeyScopedArm::PointFilter => KeyScopedArmDto::PointFilter,
        KeyScopedArm::PointSemijoin => KeyScopedArmDto::PointSemijoin,
    }
}

/// The wire mirror of a trusted run role.
fn role_to_dto(role: RunRole) -> RunRoleDto {
    match role {
        RunRole::Arm => RunRoleDto::Arm,
        RunRole::Control => RunRoleDto::Control,
    }
}
