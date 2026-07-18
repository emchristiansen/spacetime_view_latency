//! The root of the trusted campaign graph, and the sole untrusted→trusted validation entry point.

use crate::analysis::ingest::wire_record_dto::WireRecordDto;
use crate::analysis::validate::cell_dataset::CellDataset;
use crate::analysis::validate::integrity_error::IntegrityError;
use crate::manifest::preregistered_parameters::PreregisteredParameters;
use crate::manifest::schedule_seed::ScheduleSeed;

/// The number of preregistered cells — five table-scoped arms under unrelated growth plus the two
/// key-scoped arms under each of the two growth regimes (the length of
/// [`Cell::all`](crate::plan::cell::Cell::all)). It fixes the campaign's cell-array length, so *exactly
/// nine cell slots* is a property of the type.
///
/// That those nine slots equal `Cell::all()` — no missing, duplicated, or extra cell, and that this
/// length in fact agrees with the preregistered set — is **not** a compile-time guarantee: `Cell::all()`
/// returns a heap `Vec` and is not `const`, so it cannot back a `const` assertion here. The single
/// validation pass owns that agreement instead — a behavior-phase obligation, not yet implemented:
/// [`Self::from_records`] must census the manifests against `Cell::all()` and must assert
/// `CELL_COUNT == Cell::all().len()` before minting any cell, surfacing any drift as a typed
/// [`IntegrityError`]. Drift is to be caught by validation, not by the type.
const CELL_COUNT: usize = 9;

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
    /// Exactly the nine preregistered cells' complete datasets.
    cells: [CellDataset; CELL_COUNT],
}

impl ValidatedCampaign {
    /// The single total validation pass: fold the complete set of ingested wire records into the trusted
    /// campaign graph, or fail loud with a typed [`IntegrityError`]. It proves every preregistered
    /// completeness and integrity obligation — sequence contiguity, cell/block/dose census, bijective
    /// manifest references, coordinate/reference agreement, campaign-stable homogeneity, per-run
    /// provenance shape, monotonic and matched cardinality ladders, exact summary recomputation, sample
    /// count, and event identity — before any [`CellDataset`], [`MatchedBlock`], [`TrustedRun`], or
    /// [`TrustedDose`] is minted, so no partially valid campaign can escape into statistical code.
    ///
    /// [`MatchedBlock`]: super::matched_block::MatchedBlock
    /// [`TrustedRun`]: super::trusted_run::TrustedRun
    /// [`TrustedDose`]: super::trusted_dose::TrustedDose
    pub(crate) fn from_records(
        records: Vec<WireRecordDto>,
    ) -> std::result::Result<Self, IntegrityError> {
        let _ = records;
        todo!("fold the ingested wire records into the trusted campaign graph after total integrity validation")
    }

    /// Assemble the validated campaign root from its proven-homogeneous seed and parameters and its
    /// complete set of validated cell datasets. The caller ([`Self::from_records`]) owns proving seed and
    /// parameter homogeneity and the nine-cell census; this constructor only binds the proven parts.
    pub(super) fn new(
        schedule_seed: ScheduleSeed,
        parameters: PreregisteredParameters,
        cells: [CellDataset; CELL_COUNT],
    ) -> Self {
        Self {
            schedule_seed,
            parameters,
            cells,
        }
    }
}
