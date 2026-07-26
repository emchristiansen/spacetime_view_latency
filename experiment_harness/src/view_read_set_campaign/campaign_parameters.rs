//! The complete measurement-affecting parameter set of this campaign.
//!
//! The type lives in the private, *childless* inline module [`sealed`] because its entire content is
//! the claim that every field came from the frozen constants — a claim carried by
//! [`CampaignParameters::preregistered`] and by nothing in the field types. A private field is
//! visible to its declaring module **and every descendant**, so a `#[cfg(test)] mod tests` child, or
//! any child added later, could write the struct literal and record a preregistration the driver
//! never used. That matters most for `unrelated_global_rows_ladder`, which is *the* durable binding
//! of the ladder: the frozen literal is recorded rather than a version label precisely so a reader
//! compares actual values, and a forged literal would defeat exactly that. `sealed` has no children,
//! so the sole constructor really is the only door.

mod sealed {
    use serde::Serialize;

    use crate::params::EXPERIMENT_ISSUER;
    use crate::view_read_set_campaign::campaign_params::{
        ACTIVE_EXACT_KEYS_BASELINE, CAMPAIGN_SEED, CHANNEL_SAMPLE_COUNT, CONFIRMED_READS,
        ENVIRONMENT_MAX_MEMORY_PSI_CENTI, ENVIRONMENT_MIN_AVAILABLE_RAM_BYTES,
        ENVIRONMENT_SAMPLE_SEPARATION_NANOS, GLOBAL_IDENTITY_SUBJECT, GLOBAL_KEY_BASE,
        OWNED_KEY_BASE, PACED_SAMPLE_DELAY_MS, PILOT_BLOCKS, RUNG_ORDER_DOMAIN, SEEDED_ROW_PAYLOAD,
        SUBSCRIBER_VISIBLE_ROWS_BASELINE, SYNTHETIC_FAN_OUT_BASELINE,
        UNRELATED_GLOBAL_ROWS_BASELINE, UNRELATED_GLOBAL_ROWS_LADDER,
        UNRELATED_GLOBAL_ROWS_LADDER_LEN,
    };
    use crate::view_read_set_campaign::composition_validation::observed_row_set::OBSERVED_ROW_SET_DOMAIN;
    use crate::view_read_set_campaign::measurement_channel::MeasurementChannel;

    /// Every preregistered parameter that can move a measured number, recorded once with the frozen
    /// inventory.
    ///
    /// The ladder and the channel order are recorded **literally** rather than as version labels: a
    /// reader compares the actual frozen values the run used instead of trusting a name that could be
    /// reused after an edit. The seed and permutation domain determine the execution order; the
    /// canonicalization domain determines every recorded row-set digest.
    ///
    /// `confirmed_reads` is recorded **explicitly**, which the completed seed-7 ledger did not do —
    /// that campaign's confirmed-read state is verifiable only through build provenance. The spec now
    /// requires every new inventory and provenance record to carry it so a ledger-only check can
    /// verify it without inference.
    ///
    /// [`AttemptProvenance`](crate::view_read_set_campaign::attempt_provenance::AttemptProvenance)
    /// records it per attempt as well, and the honest limit of that is worth stating: both copies
    /// read [`crate::params::CONFIRMED_READS`], the same constant
    /// [`ConnectedClient::connect`](crate::client::connected_client::ConnectedClient::connect)
    /// passes to `with_confirmed_reads`, so they cannot disagree and comparing them proves nothing.
    /// What the pair buys is that a reader holding either line alone knows the setting, instead of
    /// having to infer it from build provenance as the seed-7 ledger requires.
    ///
    /// The global identity is derived from the issuer *and* the subject together, so both are
    /// recorded: the issuer is a historical [`crate::params`] constant this campaign depends on but
    /// does not own, and without it a ledger could claim an identical preregistration while every
    /// unrelated row carries a different owner.
    ///
    /// Every field is private to this childless module and [`Self::preregistered`] is the only
    /// constructor, so a recorded parameter set is constructed only from the module's own constants
    /// and cannot disagree with what the driver actually used.
    #[derive(Debug, Clone, Serialize)]
    pub(crate) struct CampaignParameters {
        unrelated_global_rows_ladder: [u64; UNRELATED_GLOBAL_ROWS_LADDER_LEN],
        pilot_blocks: u32,
        channel_execution_order: [MeasurementChannel; 4],
        channel_sample_count: u64,
        paced_sample_delay_ms: u64,
        confirmed_reads: bool,
        unrelated_global_rows_baseline: u64,
        subscriber_visible_rows_baseline: u64,
        active_exact_keys_baseline: u64,
        synthetic_fan_out_baseline: u64,
        owned_key_base: u64,
        global_key_base: u64,
        seeded_row_payload: &'static str,
        global_identity_issuer: &'static str,
        global_identity_subject: &'static str,
        campaign_seed: u64,
        rung_order_domain: &'static str,
        observed_row_set_domain: &'static str,
        environment_sample_separation_nanos: u64,
        environment_min_available_ram_bytes: u64,
        environment_max_memory_psi_centi: u64,
    }

    impl CampaignParameters {
        /// This campaign's frozen preregistration.
        pub(crate) fn preregistered() -> Self {
            Self {
                unrelated_global_rows_ladder: UNRELATED_GLOBAL_ROWS_LADDER,
                pilot_blocks: PILOT_BLOCKS,
                channel_execution_order: MeasurementChannel::EXECUTION_ORDER,
                channel_sample_count: CHANNEL_SAMPLE_COUNT,
                paced_sample_delay_ms: PACED_SAMPLE_DELAY_MS,
                confirmed_reads: CONFIRMED_READS,
                unrelated_global_rows_baseline: UNRELATED_GLOBAL_ROWS_BASELINE,
                subscriber_visible_rows_baseline: SUBSCRIBER_VISIBLE_ROWS_BASELINE,
                active_exact_keys_baseline: ACTIVE_EXACT_KEYS_BASELINE,
                synthetic_fan_out_baseline: SYNTHETIC_FAN_OUT_BASELINE,
                owned_key_base: OWNED_KEY_BASE,
                global_key_base: GLOBAL_KEY_BASE,
                seeded_row_payload: SEEDED_ROW_PAYLOAD,
                global_identity_issuer: EXPERIMENT_ISSUER,
                global_identity_subject: GLOBAL_IDENTITY_SUBJECT,
                campaign_seed: CAMPAIGN_SEED,
                rung_order_domain: RUNG_ORDER_DOMAIN,
                observed_row_set_domain: OBSERVED_ROW_SET_DOMAIN,
                environment_sample_separation_nanos: ENVIRONMENT_SAMPLE_SEPARATION_NANOS,
                environment_min_available_ram_bytes: ENVIRONMENT_MIN_AVAILABLE_RAM_BYTES,
                environment_max_memory_psi_centi: ENVIRONMENT_MAX_MEMORY_PSI_CENTI,
            }
        }

        /// The confirmed-read setting this campaign preregistered, read by the ledger-only check
        /// that compares it against each attempt's recorded value.
        pub(crate) fn confirmed_reads(&self) -> bool {
            self.confirmed_reads
        }
    }
}

pub(crate) use sealed::CampaignParameters;
