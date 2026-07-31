//! One timestamped reading of the host facts the mechanical environment gate is decided on.

use serde::Serialize;

/// The four host quantities the spec requires recorded before and after each measured attempt, with
/// the monotonic offset they were read at.
///
/// **Why the offset is part of the sample.** The gate requires "two samples 60 seconds apart", so
/// the separation is a fact about the readings themselves. Carrying it here rather than passing it
/// alongside means the pair cannot claim a separation its own samples contradict — the same reason
/// [`SaturatedWriteTiming`](super::saturated_write_timing::SaturatedWriteTiming) stores offsets from
/// a common origin rather than a precomputed difference.
///
/// **Why integer centi-units and not floats.** `/proc/loadavg` and `/proc/pressure/memory` both
/// report two decimal places, so hundredths are the *exact* reported resolution rather than a
/// rounding of it. Storing them as integers keeps the gate's comparisons exact, matching this
/// contract's rule that classification decisions use exact arithmetic and floating point is
/// display-only. A gate that could flip on a parse artefact would be worse than no gate.
///
/// **Who can construct it.** Any code, through [`Self::observed`], which is infallible: every field
/// is an unsigned count and none has a forbidden value, so there is nothing here to reject. That the
/// numbers came from this host at the claimed offset is a property of the driver's sampling path,
/// exactly as genuineness is for the measured channels. Parsing `/proc` — which genuinely can fail —
/// is the driver's job and fails there, not behind a `Result` this constructor would never return.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) struct EnvironmentSample {
    taken_at_offset_nanos: u64,
    one_minute_load_centi: u64,
    available_ram_bytes: u64,
    cumulative_swap_out_bytes: u64,
    memory_psi_full_avg60_centi: u64,
}

impl EnvironmentSample {
    /// Record one reading of the four gate quantities, at a monotonic offset from the origin its
    /// pair shares.
    pub(crate) fn observed(
        taken_at_offset_nanos: u64,
        one_minute_load_centi: u64,
        available_ram_bytes: u64,
        cumulative_swap_out_bytes: u64,
        memory_psi_full_avg60_centi: u64,
    ) -> Self {
        Self {
            taken_at_offset_nanos,
            one_minute_load_centi,
            available_ram_bytes,
            cumulative_swap_out_bytes,
            memory_psi_full_avg60_centi,
        }
    }

    /// When this reading was taken, relative to the origin shared with the other sample of its pair.
    /// Monotonic, so it measures elapsed time rather than wall-clock adjustments.
    pub(crate) fn taken_at_offset_nanos(self) -> u64 {
        self.taken_at_offset_nanos
    }

    /// One-minute load average, in hundredths of a runnable task.
    pub(crate) fn one_minute_load_centi(self) -> u64 {
        self.one_minute_load_centi
    }

    /// Available — not free — memory in bytes, the figure that accounts for reclaimable cache.
    pub(crate) fn available_ram_bytes(self) -> u64 {
        self.available_ram_bytes
    }

    /// Cumulative bytes swapped out since boot. Only the *delta* between two samples is meaningful:
    /// swap occupancy alone is not evidence of active thrashing, which is why the gate reads flow.
    pub(crate) fn cumulative_swap_out_bytes(self) -> u64 {
        self.cumulative_swap_out_bytes
    }

    /// Memory pressure `full avg60`, in hundredths of a percent-of-time stalled.
    pub(crate) fn memory_psi_full_avg60_centi(self) -> u64 {
        self.memory_psi_full_avg60_centi
    }
}
