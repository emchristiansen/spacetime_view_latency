//! The complete measurement-affecting parameter set of this Pilot.

use serde::Serialize;

use crate::entity_owner_pilot::pilot_params::{
    ENTITY_OWNER_PILOT_GLOBAL_SUBJECT, GLOBAL_KEY_BASE, GLOBAL_ROW_LADDER, GLOBAL_ROW_LADDER_LEN,
    OWNED_KEY_BASE, OWNED_SLICE_ROWS, PILOT_ARM_CONTROL_ORDER_DOMAIN, PILOT_BLOCKS,
    PILOT_BLOCK_ORDER_DOMAIN, PILOT_ROW_PAYLOAD,
};
use crate::params::{BATCH_SIZE, EXPERIMENT_ISSUER};

/// Every preregistered parameter that can move a measured number, recorded once in the ledger.
///
/// The ladder is recorded **literally** rather than as a version label: a reader compares the actual
/// frozen rungs the run walked instead of trusting a name that could be reused after an edit. The
/// permutation domains determine the execution order. The global identity is derived from the
/// issuer *and* the subject together, so both are recorded: the issuer is a historical
/// [`crate::params`] constant this candidate depends on but does not own, and without it a ledger
/// could claim an identical preregistration while every unrelated row carries a different owner.
///
/// Constructed only from the module's own constants, so it cannot disagree with what the driver
/// actually used.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct PilotParameters {
    global_row_ladder: [u64; GLOBAL_ROW_LADDER_LEN],
    pilot_blocks: u32,
    batch_size: u64,
    owned_slice_rows: u64,
    owned_key_base: u64,
    global_key_base: u64,
    row_payload: &'static str,
    global_identity_issuer: &'static str,
    global_identity_subject: &'static str,
    block_order_domain: &'static str,
    arm_control_order_domain: &'static str,
}

impl PilotParameters {
    /// This Pilot's frozen preregistration.
    pub(crate) fn preregistered() -> Self {
        Self {
            global_row_ladder: GLOBAL_ROW_LADDER,
            pilot_blocks: PILOT_BLOCKS,
            batch_size: BATCH_SIZE,
            owned_slice_rows: OWNED_SLICE_ROWS,
            owned_key_base: OWNED_KEY_BASE,
            global_key_base: GLOBAL_KEY_BASE,
            row_payload: PILOT_ROW_PAYLOAD,
            global_identity_issuer: EXPERIMENT_ISSUER,
            global_identity_subject: ENTITY_OWNER_PILOT_GLOBAL_SUBJECT,
            block_order_domain: PILOT_BLOCK_ORDER_DOMAIN,
            arm_control_order_domain: PILOT_ARM_CONTROL_ORDER_DOMAIN,
        }
    }
}
