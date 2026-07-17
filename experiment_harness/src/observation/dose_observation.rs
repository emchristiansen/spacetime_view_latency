//! One dose's complete, lossless, machine-readable observation record.

use serde::Serialize;

use crate::dataset::campaign_dataset::CampaignDataset;
use crate::dataset::dose_batch::DoseBatch;
use crate::dataset::dose_index::DoseIndex;
use crate::dataset::physical_cardinalities::PhysicalCardinalities;
use crate::manifest::run_coordinate::RunCoordinate;
use crate::manifest::validated_run_manifest::ValidatedRunManifest;
use crate::plan::run_role::RunRole;
use crate::observation::dose_coordinate::DoseCoordinate;
use crate::observation::dose_evidence::DoseEvidence;
use crate::observation::event_evidence::EventEvidence;
use crate::observation::latency_summary::LatencySummary;
use crate::observation::manifest_reference::ManifestReference;
use crate::observation::raw_latencies::RawLatencies;

/// The record *schema* for one cumulative dose: a reference to the immutable run manifest, the
/// schedule/ladder coordinate, both physical table cardinalities, the lossless raw latency vector,
/// its derived median/IQR summary, and the SDK logical event evidence. It is designed to be compact
/// yet sufficient to reproduce every reported summary and classification (spec: "A per-dose object
/// containing the full raw vector is compact while still sufficient to reproduce every summary").
/// It is a skeleton until the measurement milestone: [`LatencySummary::from_raw`] is `todo!()` and
/// no event transition yet populates the evidence.
///
/// [`Self::assemble`] takes the real [`ValidatedRunManifest`] and derives the manifest reference and
/// coordinate from it — there is no independent `reference`/`run` argument to mismatch — and
/// checked-matches the manifest's run against the [`CampaignDataset`]'s retained source run. Because
/// dataset resolution is block-independent, the strongest meaningful correspondence is *run shape*
/// (cell, role, control table): the assertion proves the manifest and dataset agree on the run
/// shape, while the manifest alone supplies the repetition-block identity the dataset does not
/// carry. This makes the record structurally consistent — reference, coordinate, and
/// family-specific cardinality all come from one manifest/dataset pair of matching run shape. It is
/// a consistency guarantee, not a claim that the samples were actually measured under that manifest.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct DoseObservation {
    manifest_ref: ManifestReference,
    coordinate: DoseCoordinate,
    cardinalities: PhysicalCardinalities,
    latencies: RawLatencies,
    summary: LatencySummary,
    events: EventEvidence,
}

impl DoseObservation {
    /// Assemble the record from the manifest and the dataset resolved for the same run shape. The
    /// reference, coordinate, and cardinalities are derived here; only the raw latencies and event
    /// evidence come from the typed [`DoseEvidence`]. Fails loud (a wiring bug) if the manifest's
    /// run shape and the dataset's source run disagree — compared via full [`Run`](crate::plan::run::Run)
    /// equality so a future `Run` field cannot be silently forgotten from the correspondence.
    pub(crate) fn assemble(
        manifest: &ValidatedRunManifest,
        dataset: &CampaignDataset,
        batch: &DoseBatch,
        evidence: DoseEvidence,
    ) -> Self {
        let run = manifest.run_coordinate();
        // Reconstruct the manifest's run from its (cell, role) and compare the whole `Run` — cell,
        // role, and control table — against the dataset's retained source run.
        let [arm, control] = run.cell().matched_runs();
        let manifest_run = match run.role() {
            RunRole::Arm => arm,
            RunRole::Control => control,
        };
        let source = dataset.source_run();
        assert_eq!(
            manifest_run, source,
            "manifest run shape {manifest_run:?} and dataset source run {source:?} disagree"
        );

        let reference = ManifestReference::of(manifest);
        let cardinalities = dataset.physical_cardinalities(batch);
        let coordinate = DoseCoordinate::new(run, dataset, batch);
        let (latencies, events) = evidence.into_parts();
        let summary = LatencySummary::from_raw(&latencies);
        Self {
            manifest_ref: reference,
            coordinate,
            cardinalities,
            latencies,
            summary,
            events,
        }
    }

    /// This observation's ladder index, used to tag its durable record.
    pub(crate) fn dose(&self) -> DoseIndex {
        self.coordinate.dose()
    }

    /// This observation's run coordinate, used to bind it to the active run before a durable write.
    pub(crate) fn run_coordinate(&self) -> RunCoordinate {
        self.coordinate.run()
    }

    /// The immutable manifest reference this observation keys to, used to bind it to the active run's
    /// manifest receipt before a durable write.
    pub(crate) fn manifest_ref(&self) -> &ManifestReference {
        &self.manifest_ref
    }

    /// A deterministic, internally coherent test fixture observation for the fixture manifest's own run
    /// and first dose. Delegates to [`Self::fixture_for`]. Test-only: it exists so the sink's real
    /// [`write_observation`](crate::observation::observation_sink::ObservationSink::write_observation)
    /// path can be exercised end-to-end.
    #[cfg(test)]
    pub(crate) fn fixture() -> Self {
        let manifest = ValidatedRunManifest::fixture();
        Self::fixture_for(&manifest, DoseIndex::ALL[0])
    }

    /// A deterministic, internally coherent test fixture observation for an arbitrary `manifest`'s run
    /// and a specific `dose`. It builds the fields directly (not through [`Self::assemble`], which calls
    /// the `todo!()` [`LatencySummary::from_raw`]) but derives the coordinate and cardinalities through
    /// the *same* production paths `assemble` uses — [`DoseCoordinate::new`] and
    /// [`CampaignDataset::physical_cardinalities`] over the run's real resolved dataset and the given
    /// dose — so no field encodes a state impossible in production. The dataset is resolved for the
    /// manifest run's actual role (arm or matched control), so the family, regime, and cardinalities are
    /// coherent for that run. Only the summary and the not-yet-measured event evidence are supplied as
    /// coherent placeholders.
    ///
    /// The `manifest_ref` is [`ManifestReference::of`] the given manifest, so every observation built
    /// from one fixture manifest binds to that manifest's run — the exact binding a run cursor's
    /// `write_observation` checks. The supplied event decomposition (`BATCH_SIZE` inserts, 0 deletes, 0
    /// updates, net `+BATCH_SIZE`) is a mutually consistent hypothetical: [`EventEvidence::checked`]
    /// proves only arithmetic agreement (inserts − deletes = net delta), never runtime provenance,
    /// result-set observation, or how the stock SDK would actually label a refresh. The all-1 ns samples
    /// agree with the median-1/IQR-0 summary.
    #[cfg(test)]
    pub(crate) fn fixture_for(manifest: &ValidatedRunManifest, dose: DoseIndex) -> Self {
        use std::time::Duration;

        use spacetimedb_sdk::Identity;

        use crate::dataset::campaign_dataset::CampaignDataset;
        use crate::dataset::dose_batch::DoseBatch;
        use crate::observation::latency_sample::LatencySample;
        use crate::params::{BATCH_SIZE, BATCH_SIZE_USIZE};
        use crate::roles::role_identities::RoleIdentities;

        let run = manifest.run_coordinate();
        let cell = run.cell();

        // Resolve this run's real dataset for its actual role. The measured identity is a fixture
        // stand-in for the server-issued connection identity; the growth identity is derived and must
        // differ from it.
        let measured =
            Identity::from_claims("view-read-set-experiment-fixture", "fixture-measured");
        let identities = RoleIdentities::resolve(measured, manifest.schedule_seed(), cell)
            .expect("the fixture measured and growth identities are distinct");
        let [arm_run, control_run] = cell.matched_runs();
        let source_run = match run.role() {
            RunRole::Arm => arm_run,
            RunRole::Control => control_run,
        };
        let dataset = CampaignDataset::resolve(source_run, &identities);
        let batch = DoseBatch::new(&dataset, dose);

        let coordinate = DoseCoordinate::new(run, &dataset, &batch);
        let cardinalities = dataset.physical_cardinalities(&batch);

        // Every sample is exactly 1 ns, matching the median-1/IQR-0 fixture summary.
        let samples = vec![LatencySample::from_elapsed(Duration::from_nanos(1)); BATCH_SIZE_USIZE];
        let latencies =
            RawLatencies::sealed(samples).expect("the fixture supplies exactly BATCH_SIZE samples");

        // Supplied hypothetical decomposition and net delta — neither a dataset-forced nor an observed
        // value (see above). EventEvidence::checked proves only that inserts − deletes agrees with the
        // supplied net delta, no runtime provenance.
        let net_delta = i64::try_from(BATCH_SIZE).expect("BATCH_SIZE fits i64");
        let events = EventEvidence::checked(BATCH_SIZE, 0, 0, net_delta)
            .expect("the fixture event counts satisfy the net-delta identity");

        Self {
            manifest_ref: ManifestReference::of(manifest),
            coordinate,
            cardinalities,
            latencies,
            summary: LatencySummary::fixture(),
            events,
        }
    }
}
