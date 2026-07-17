//! The owning bind of a run's immutable manifest to the dataset resolved from its coordinate.

use crate::dataset::campaign_dataset::CampaignDataset;
use crate::dataset::subscribed_table::SubscribedTable;
use crate::manifest::validated_run_manifest::ValidatedRunManifest;
use crate::roles::role_identities::RoleIdentities;

/// One run's immutable manifest bound to the [`CampaignDataset`] resolved for that same manifest's run.
///
/// [`Self::resolve`] is the sole constructor: it takes the manifest **by value** and resolves the dataset
/// from `manifest.run_coordinate().run()`, so the dataset is never derived from an independently supplied
/// run — the manifest is the single carrier of run identity and the dataset co-derives from it. Because
/// the two halves can only ever be constructed together from one coordinate, record assembly consumes this
/// bound pair rather than a manifest/dataset pair that could disagree, retiring the run-shape cross-check
/// that a separate manifest and dataset once needed.
pub(crate) struct RunDataset {
    manifest: ValidatedRunManifest,
    dataset: CampaignDataset,
}

impl RunDataset {
    /// Bind the manifest to the dataset resolved for its own run. Moves the manifest in (nothing needs it
    /// by value afterward) and resolves the dataset from the manifest's coordinate, so the pair is
    /// single-sourced by construction.
    pub(crate) fn resolve(manifest: ValidatedRunManifest, identities: &RoleIdentities) -> Self {
        let dataset = CampaignDataset::resolve(manifest.run_coordinate().run(), identities);
        Self { manifest, dataset }
    }

    /// The immutable run manifest — the single carrier of this run's identity.
    pub(crate) fn manifest(&self) -> &ValidatedRunManifest {
        &self.manifest
    }

    /// The dataset resolved for the manifest's run.
    pub(crate) fn dataset(&self) -> &CampaignDataset {
        &self.dataset
    }

    /// The single table this run subscribes to, derived from the manifest's own run. The one place a
    /// [`SubscribedTable`] is minted for the run, so the subscription target, the per-dose reads, and the
    /// event-callback registration all resolve through this one context rather than an independently
    /// reconstructed `Run → SubscribedTable` step that could drift.
    pub(crate) fn subscribed_table(&self) -> SubscribedTable {
        SubscribedTable::from_run(self.manifest.run_coordinate().run())
    }
}
