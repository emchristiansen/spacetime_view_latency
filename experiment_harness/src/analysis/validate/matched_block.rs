//! One validated repetition block: a matched arm/control run pair.

use crate::analysis::validate::arm_run::ArmRun;
use crate::analysis::validate::control_run::ControlRun;
use crate::analysis::validate::trusted_run::TrustedRun;
use crate::observation::record_seq::RecordSeq;

/// One repetition block of a cell: exactly its two matched runs — the arm under test and its matched
/// direct-base-table control. The arm and control are role-indexed [`TrustedRun`] types ([`ArmRun`] and
/// [`ControlRun`]), not two same-typed fields, so a block with two arms, two controls, or a swapped
/// arm/control pair is *unrepresentable*.
///
/// The block index is **not** stored here: both run coordinates already carry their repetition-block
/// index, so a third copy would be independent drift surface. The validation pass proves the two
/// coordinates carry the same block index and that it matches the block's position in the enclosing
/// fixed-size [`CellDataset`](super::cell_dataset::CellDataset) array; a read-only block-index accessor
/// is derived from either run after that equality is proven.
///
/// Fields are private with no defaults; the module-owned `pub(super)` [`Self::new`] is the only
/// assembler, so a `MatchedBlock` is built only from within the `validate` subtree.
pub(crate) struct MatchedBlock {
    /// The module-view arm run under test for this block.
    arm: TrustedRun<ArmRun>,
    /// The matched direct-base-table control run for this block.
    control: TrustedRun<ControlRun>,
}

impl MatchedBlock {
    /// Assemble a matched block from its two already-validated, correctly-typed runs. The arm/control
    /// roles are enforced by the field types; the caller (the validation pass) owns proving the two
    /// coordinates agree on their shared block index and that it matches the array position.
    pub(super) fn new(arm: TrustedRun<ArmRun>, control: TrustedRun<ControlRun>) -> Self {
        Self { arm, control }
    }

    /// This block's position in the durable collection order: the earlier of its two runs' manifest
    /// sequences. Temporal analyses (collection-order plots, lag-1 autocorrelation) order blocks by this
    /// key to recover when each block was actually recorded, independent of the canonical
    /// cell→role→block traversal that the trusted-graph shape imposes.
    // Phase 1 scaffolding: consumed by the Phase 2 classify/report temporal layer; allow until then.
    #[allow(dead_code)]
    pub(crate) fn collection_order_key(&self) -> RecordSeq {
        self.arm.manifest_seq().min(self.control.manifest_seq())
    }

    /// Test-only read of the arm run, for asserting the minted graph shape. `#[cfg(test)]` so it never
    /// widens the production API.
    #[cfg(test)]
    pub(super) fn arm(&self) -> &TrustedRun<ArmRun> {
        &self.arm
    }

    /// Test-only read of the matched control run, for asserting the minted graph shape.
    #[cfg(test)]
    pub(super) fn control(&self) -> &TrustedRun<ControlRun> {
        &self.control
    }
}
