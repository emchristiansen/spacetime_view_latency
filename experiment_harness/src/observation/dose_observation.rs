//! One dose's complete, lossless, machine-readable observation record.

use serde::Serialize;

use crate::dataset::dose_batch::DoseBatch;
use crate::dataset::dose_index::DoseIndex;
use crate::dataset::physical_cardinalities::PhysicalCardinalities;
use crate::dataset::run_dataset::RunDataset;
use crate::manifest::run_coordinate::RunCoordinate;
#[cfg(test)]
use crate::manifest::validated_run_manifest::ValidatedRunManifest;
use crate::observation::dose_coordinate::DoseCoordinate;
use crate::observation::dose_evidence::DoseEvidence;
use crate::observation::event_evidence::EventEvidence;
use crate::observation::latency_summary::LatencySummary;
use crate::observation::manifest_reference::ManifestReference;
use crate::observation::raw_latencies::RawLatencies;
#[cfg(test)]
use crate::plan::run_role::RunRole;

/// The record *schema* for one cumulative dose: a reference to the immutable run manifest, the
/// schedule/ladder coordinate, both physical table cardinalities, the lossless raw latency vector,
/// its derived median/IQR summary, and the SDK logical event evidence. It is designed to be compact
/// yet sufficient to reproduce every reported summary and classification (spec: "A per-dose object
/// containing the full raw vector is compact while still sufficient to reproduce every summary").
/// It is a skeleton until the measurement milestone: no run driver yet assembles one from real
/// measured latencies and delivered SDK events.
///
/// [`Self::assemble`] takes the bound [`RunDataset`] and derives the manifest reference, coordinate, and
/// cardinality from the *same* manifest/dataset pair — there is no independent `reference`/`run` argument
/// to mismatch, and no cross-check to run: the [`RunDataset`] can only have been built by resolving the
/// dataset from that manifest's own coordinate, so reference, coordinate, and family-specific cardinality
/// are single-sourced by construction. It is a consistency guarantee, not a claim that the samples were
/// actually measured under that manifest.
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
    /// Assemble the record from the bound [`RunDataset`] — one manifest and the dataset resolved from its
    /// own coordinate. The reference, coordinate, and cardinalities are derived here from that single pair;
    /// only the raw latencies and event evidence come from the typed [`DoseEvidence`]. No manifest/dataset
    /// cross-check is needed or possible: the [`RunDataset`] binds them at resolution, so a mismatched pair
    /// is unrepresentable rather than a runtime assertion.
    pub(crate) fn assemble(
        context: &RunDataset,
        batch: &DoseBatch,
        evidence: DoseEvidence,
    ) -> Self {
        let manifest = context.manifest();
        let dataset = context.dataset();
        let run = manifest.run_coordinate();

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
    /// and a specific `dose`. It builds the fields directly (not through [`Self::assemble`], which
    /// consumes a real [`DoseEvidence`] a no-I/O fixture cannot produce) but derives the coordinate and
    /// cardinalities through
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
