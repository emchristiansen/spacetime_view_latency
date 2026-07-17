//! The 1-based index of a cumulative dose within a run's monotonic ladder.

use serde::Serialize;

use crate::params::NUM_DOSES;

/// The 1-based position of a cumulative dose in the preregistered ladder.
///
/// This type proves **range validity only**: every `DoseIndex` that exists is a member of
/// `1..=NUM_DOSES`. It has no callable constructor — the private field plus the absence of any
/// minting API means the sole source of values is the module-owned fixed array [`Self::ALL`], so
/// no caller can fabricate a `DoseIndex` and range membership is a property of the type, not of
/// caller discipline.
///
/// Application *ordering* is a separate concern, proven by the consumer of [`Self::ALL`] (see
/// [`CampaignDataset::for_each_dose`](super::campaign_dataset::CampaignDataset::for_each_dose));
/// the index type says nothing about the sequence in which doses are applied.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub(crate) struct DoseIndex(u64);

impl DoseIndex {
    /// The fixed, module-owned monotonic sequence of every valid dose index, `1..=NUM_DOSES` in
    /// order. Built in a `const` block from the private field, so these are the only `DoseIndex`
    /// values in existence and there is no runtime minting path. Consuming this array in array
    /// order is how the ladder applies the doses sequentially.
    pub(crate) const ALL: [DoseIndex; NUM_DOSES as usize] = {
        let mut all = [DoseIndex(0); NUM_DOSES as usize];
        let mut i = 0usize;
        while i < NUM_DOSES as usize {
            all[i] = DoseIndex(i as u64 + 1);
            i += 1;
        }
        all
    };

    /// The 1-based dose number.
    pub(crate) fn get(self) -> u64 {
        self.0
    }
}
