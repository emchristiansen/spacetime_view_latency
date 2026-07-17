//! One cumulative dose of the measured ladder: the [`BATCH_SIZE`] writes that advance the driving
//! slice by one rung.

use spacetimedb_sdk::Identity;

use crate::dataset::campaign_dataset::CampaignDataset;
use crate::dataset::dose_index::DoseIndex;
use crate::dataset::seed_op::SeedOp;
use crate::params::BATCH_SIZE;
use crate::plan::control_table::ControlTable;

/// One cumulative dose handed to the measured executor: the [`BATCH_SIZE`] writes that advance the
/// driving slice from `(dose-1) * BATCH_SIZE` to `dose * BATCH_SIZE` rows, every one attributed to
/// the driving identity.
///
/// The only constructor is [`Self::new`], which derives the driving identity, key base, and family
/// from a resolved [`CampaignDataset`] plus a valid [`DoseIndex`] — never from raw caller-supplied
/// values. So a `DoseBatch` always writes the driving role's rows, in the run's family, at the key
/// range this dose owns; it can never carry the pinned background identity or a mismatched family.
#[derive(Debug, Clone, Copy)]
pub(crate) struct DoseBatch {
    dose: DoseIndex,
    driving: Identity,
    key_base: u64,
    family: ControlTable,
}

impl DoseBatch {
    /// Mint the batch for `dose` from a resolved dataset, deriving the driving identity, key base,
    /// and family from `dataset`. `dose` is a [`DoseIndex`], which can only be one of the ten fixed
    /// valid rungs, so no argument can express an out-of-range or off-family dose.
    pub(crate) fn new(dataset: &CampaignDataset, dose: DoseIndex) -> Self {
        Self {
            dose,
            driving: dataset.driving(),
            key_base: dataset.driving_key_base(),
            family: dataset.family(),
        }
    }

    /// This dose's 1-based ladder index.
    pub(crate) fn dose(&self) -> DoseIndex {
        self.dose
    }

    /// The driving role's cumulative **logical** row count once this dose is applied
    /// (`dose * BATCH_SIZE`) — the preregistered ladder x-axis. For the Chronicle family one
    /// logical row is a visibility/message pair, so the physical table footprint is twice this;
    /// this value is the logical x-axis, not a physical row count.
    pub(crate) fn cumulative_driving_rows(&self) -> u64 {
        self.dose.get() * BATCH_SIZE
    }

    /// The half-open primary-key range this dose writes: exactly [`BATCH_SIZE`] contiguous keys
    /// advancing the driving slice, disjoint from the pinned background base by construction.
    fn keys(&self) -> std::ops::Range<u64> {
        let start = self.key_base + (self.dose.get() - 1) * BATCH_SIZE;
        start..start + BATCH_SIZE
    }

    /// The ordered insert intents for this dose, every one attributed to the driving identity.
    pub(crate) fn operations(&self) -> Vec<SeedOp> {
        self.keys()
            .map(|key| match self.family {
                ControlTable::Message => SeedOp::Message {
                    id: key,
                    sender: self.driving,
                },
                ControlTable::ChronicleMessage => SeedOp::ChroniclePair {
                    key,
                    viewer: self.driving,
                },
            })
            .collect()
    }
}
