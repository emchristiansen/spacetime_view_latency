//! The paired host readings the mechanical environment gate is decided from.
//!
//! The type lives in the private, *childless* inline module [`sealed`] because the sampling-separation
//! precondition holds only if its sole constructor ran. A private field is visible to its declaring
//! module **and every descendant**, so a `#[cfg(test)] mod tests` child — or any child added later —
//! could write the struct literal for two readings taken moments apart, or for a zero CPU count, and
//! [`EnvironmentGateEvidence::passes`] would then return a verdict the pair cannot support. `sealed`
//! has no children, so [`EnvironmentGateEvidence::paired`] really is the only door.

mod sealed {
    use anyhow::{ensure, Result};
    use serde::Serialize;

    use crate::view_read_set_campaign::campaign_params::{
        ENVIRONMENT_MAX_MEMORY_PSI_CENTI, ENVIRONMENT_MIN_AVAILABLE_RAM_BYTES,
        ENVIRONMENT_SAMPLE_SEPARATION_NANOS,
    };
    use crate::view_read_set_campaign::environment_sample::EnvironmentSample;

    /// The two preflight host readings an attempt is cleared to launch on, and the CPU count they
    /// are judged against.
    ///
    /// **Prospective, not retrospective.** Both samples are taken *before* the attempt launches; the
    /// pair ends immediately before it. So this never invalidates a measurement — it decides whether
    /// one happens at all, and a refusal is
    /// [`AttemptOutcome::PreflightRejected`](crate::view_read_set_campaign::attempt_outcome::AttemptOutcome::PreflightRejected),
    /// a terminal disposition with no evidence field, not a verdict on evidence. The separate reading
    /// taken *after* a measured attempt is supporting diagnostics and is deliberately not this type.
    ///
    /// **Why the evidence, not a verdict, is the recorded thing.** The gate is mechanical: given
    /// these two samples and the logical CPU count, [`Self::passes`] is a total function with no
    /// discretion in it. Storing the inputs and deriving the verdict means a reader can recompute the
    /// decision rather than trust a boolean, and means an attempt cannot record a verdict its own
    /// samples contradict. "Invalid attempts remain in the ledger" is only auditable if the readings
    /// that invalidated them are there too.
    ///
    /// The logical CPU count is stored once alongside the pair rather than on each sample: it is a
    /// fact about the host the comparison is made against, and two samples that disagreed about it
    /// would be a state with no meaning.
    ///
    /// **Who can construct it.** Any code, through [`Self::paired`], which enforces the frozen
    /// sampling separation from the samples' own offsets. Every field is private to this childless
    /// module, so that constructor is the only door and no future sibling of it can widen. The
    /// readings themselves are as trustworthy as the driver's sampling path and no more; what this
    /// type rules out is a gate decided from samples taken too close together to show a falling
    /// trend.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
    pub(crate) struct EnvironmentGateEvidence {
        first: EnvironmentSample,
        second: EnvironmentSample,
        logical_cpus: u64,
    }

    impl EnvironmentGateEvidence {
        /// Pair two readings, failing loud unless the second follows the first by at least
        /// [`ENVIRONMENT_SAMPLE_SEPARATION_NANOS`] and the host reports at least one CPU.
        ///
        /// The separation is a precondition rather than a gate term: the rule requires the second
        /// load to be *strictly lower* than the first, and two samples taken moments apart would
        /// satisfy that on noise alone, so a short pair does not fail the gate — it cannot decide it.
        /// It is derived from the samples' own monotonic offsets, so no caller can assert an interval
        /// the readings do not support.
        ///
        /// The frozen protocol timing is exactly sixty seconds. The check is `>=` rather than `==`
        /// because the sampler initiates the second read *after* a sixty-second wait, and the
        /// scheduler delay between the wait expiring and the read completing is unavoidable and
        /// unbounded above by anything this process controls. So `>=` is the mechanical expression of
        /// "wait sixty seconds, then sample", not a relaxed threshold: a pair shorter than sixty
        /// seconds cannot have been produced by the frozen procedure at all.
        pub(crate) fn paired(
            first: EnvironmentSample,
            second: EnvironmentSample,
            logical_cpus: u64,
        ) -> Result<Self> {
            let separation_nanos = second
                .taken_at_offset_nanos()
                .checked_sub(first.taken_at_offset_nanos())
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "the environment gate's second sample was taken before its first ({} ns vs \
                         {} ns from the same monotonic origin)",
                        second.taken_at_offset_nanos(),
                        first.taken_at_offset_nanos(),
                    )
                })?;
            ensure!(
                separation_nanos >= ENVIRONMENT_SAMPLE_SEPARATION_NANOS,
                "the environment gate needs samples at least \
                 {ENVIRONMENT_SAMPLE_SEPARATION_NANOS} ns apart to read a falling trend; these are \
                 {separation_nanos} ns apart",
            );
            ensure!(
                logical_cpus > 0,
                "the environment gate compares load against the logical CPU count, which cannot be \
                 zero",
            );
            Ok(Self {
                first,
                second,
                logical_cpus,
            })
        }

        /// Whether every clause of the frozen mechanical gate holds.
        ///
        /// The spec's clauses are all governed by "the second sample": its one-minute load is at
        /// most the logical CPU count and strictly lower than the first sample's; its available RAM
        /// is at least [`ENVIRONMENT_MIN_AVAILABLE_RAM_BYTES`]; the swap-out byte delta between the
        /// two samples is exactly zero; and its memory PSI `full avg60` is at most
        /// [`ENVIRONMENT_MAX_MEMORY_PSI_CENTI`].
        ///
        /// The first sample enters exactly twice, and only where the rule names it: as the value the
        /// second sample's load must fall below, and as the base of the swap-out delta. Applying the
        /// RAM or PSI thresholds to it as well would be a stricter gate than the one preregistered,
        /// and a gate is not something to tighten unilaterally.
        ///
        /// A cumulative counter cannot decrease, so equality is the whole delta clause: a decrease
        /// would mean a counter reset, which is not a zero delta and is treated as a failure rather
        /// than clamped to one.
        pub(crate) fn passes(&self) -> bool {
            let load_ceiling_centi = self.logical_cpus.saturating_mul(100);
            let load_ok = self.second.one_minute_load_centi() <= load_ceiling_centi
                && self.second.one_minute_load_centi() < self.first.one_minute_load_centi();
            let ram_ok = self.second.available_ram_bytes() >= ENVIRONMENT_MIN_AVAILABLE_RAM_BYTES;
            let swap_ok =
                self.second.cumulative_swap_out_bytes() == self.first.cumulative_swap_out_bytes();
            let psi_ok =
                self.second.memory_psi_full_avg60_centi() <= ENVIRONMENT_MAX_MEMORY_PSI_CENTI;

            load_ok && ram_ok && swap_ok && psi_ok
        }
    }
}

pub(crate) use sealed::EnvironmentGateEvidence;

// A sibling of `sealed`, not a child: the tests reach the type only through `paired`, exactly as
// production does, so no fixture can assert a pair the constructor would refuse.
#[cfg(test)]
mod tests;
