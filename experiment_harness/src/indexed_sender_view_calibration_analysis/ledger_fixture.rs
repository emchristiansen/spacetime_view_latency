//! Hand-built calibration ledger lines, for pure server-free tests.

use serde_json::json;

use crate::indexed_sender_view_calibration_analysis::frozen_candidate_version::frozen_candidate_version;
use crate::indexed_sender_view_calibration_analysis::frozen_population::FrozenPopulation;
use crate::indexed_sender_view_calibration_pilot::calibration_params::{
    MAX_PACED_SAMPLES, MAX_PACED_SAMPLES_USIZE, SUBSCRIBER_OWN_ROWS, WITH_CONFIRMED_READS,
};
use crate::view_read_set_campaign::campaign_params::PACED_SAMPLE_DELAY_MS;

/// A wire line whose every field is the frozen value, varying only what a test needs to vary.
///
/// **Built as JSON rather than by serializing a real [`CalibrationRecord`].** Two reasons. The pilot's
/// record type needs an `AttemptProvenance`, `HostObservations`, and a `PinnedArtifactIdentity` that
/// only a real run can produce, so constructing one here would drag a server-shaped dependency into a
/// pure test. And a test that round-trips the writer's own types could not detect a wire-shape
/// disagreement at all — it would agree with itself by construction. Writing the bytes out by hand is
/// what makes these tests evidence that the DTOs match the *documented* shape.
///
/// The frozen values themselves still come from the pilot's constants, so a freeze change moves the
/// fixture with it rather than leaving it asserting a stale contract.
pub(crate) struct LedgerFixture {
    replicate: u32,
    samples: Vec<u128>,
    arm_rows: u64,
    witness_rows: u64,
    verified_appends: u64,
    role: &'static str,
    version: u32,
    channel: &'static str,
    sample_count: u32,
    paced_sample_delay_ms: u64,
    seeded_own_rows: u64,
    with_confirmed_reads: bool,
}

impl LedgerFixture {
    /// A fully valid `Attempted`/`CalibrationRecorded` line at `replicate`, with a strictly positive
    /// series of exactly the frozen length.
    pub(crate) fn complete(replicate: u32) -> Self {
        let population = FrozenPopulation::complete();
        Self {
            replicate,
            // Strictly increasing so no two samples coincide, and positive throughout.
            samples: (0..MAX_PACED_SAMPLES_USIZE)
                .map(|index| 1_000_000 + index as u128)
                .collect(),
            arm_rows: population.arm_rows,
            witness_rows: population.witness_rows,
            verified_appends: population.verified_appends,
            role: "Arm",
            version: frozen_candidate_version(),
            channel: "PacedVisibleApplyLatency",
            sample_count: MAX_PACED_SAMPLES,
            paced_sample_delay_ms: PACED_SAMPLE_DELAY_MS,
            seeded_own_rows: SUBSCRIBER_OWN_ROWS,
            with_confirmed_reads: WITH_CONFIRMED_READS,
        }
    }

    /// Override the declared measurement channel, by wire spelling.
    ///
    /// **Takes a spelling rather than a variant** for the same reason the whole method block is
    /// written out as literals: a fixture that named the mirrored DTO's variant would agree with it
    /// by construction. Passing a real `MeasurementChannel` spelling here produces a line that
    /// decodes cleanly and is refused only by the frozen comparison; passing an invented one
    /// produces a line that fails to decode at all. Those are different properties, and this lets a
    /// test choose which one it is about.
    pub(crate) fn with_channel(mut self, channel: &'static str) -> Self {
        self.channel = channel;
        self
    }

    /// Override the method's declared sample count, leaving the series itself frozen.
    ///
    /// **Separate from [`Self::with_samples`] on purpose.** These are two different claims: one is
    /// the method the run declares it followed, the other is what it recorded. A line whose method
    /// says 500 while its series holds 1,000 is refused for the *method*, not the count — and only
    /// varying them independently can show that.
    pub(crate) fn with_sample_count(mut self, sample_count: u32) -> Self {
        self.sample_count = sample_count;
        self
    }

    /// Override the declared pacing delay.
    pub(crate) fn with_paced_delay_ms(mut self, paced_sample_delay_ms: u64) -> Self {
        self.paced_sample_delay_ms = paced_sample_delay_ms;
        self
    }

    /// Override the declared seeded own-row count.
    pub(crate) fn with_seeded_own_rows(mut self, seeded_own_rows: u64) -> Self {
        self.seeded_own_rows = seeded_own_rows;
        self
    }

    /// Override whether the method declares confirmed reads.
    pub(crate) fn with_confirmed_reads(mut self, with_confirmed_reads: bool) -> Self {
        self.with_confirmed_reads = with_confirmed_reads;
        self
    }

    /// Replace the series wholesale.
    pub(crate) fn with_samples(mut self, samples: Vec<u128>) -> Self {
        self.samples = samples;
        self
    }

    /// Override the proven arm row count.
    pub(crate) fn with_arm_rows(mut self, arm_rows: u64) -> Self {
        self.arm_rows = arm_rows;
        self
    }

    /// Override the proven witness row count.
    pub(crate) fn with_witness_rows(mut self, witness_rows: u64) -> Self {
        self.witness_rows = witness_rows;
        self
    }

    /// Override the proven append count.
    pub(crate) fn with_verified_appends(mut self, verified_appends: u64) -> Self {
        self.verified_appends = verified_appends;
        self
    }

    /// Override the run role, so a `Control` line can be refused by name.
    pub(crate) fn with_role(mut self, role: &'static str) -> Self {
        self.role = role;
        self
    }

    /// Override the candidate version.
    pub(crate) fn with_version(mut self, version: u32) -> Self {
        self.version = version;
        self
    }

    /// Render the line exactly as the pilot's `append_all` would.
    pub(crate) fn line(&self) -> String {
        json!({
            "Attempted": {
                "key": self.key(),
                "method": self.method(),
                "pinned": pinned(),
                "schedule_seed": 0u64,
                "provenance": opaque(),
                "host": opaque(),
                "outcome": {
                    "CalibrationRecorded": {
                        "series": {
                            "samples": self.samples,
                            "population": {
                                "arm_rows": self.arm_rows,
                                "witness_rows": self.witness_rows,
                                "verified_appends": self.verified_appends,
                            },
                        },
                    },
                },
                "release": opaque(),
            }
        })
        .to_string()
    }

    /// The same attempt, but recorded as a failure rather than a recorded series.
    pub(crate) fn failed_line(&self) -> String {
        json!({
            "Attempted": {
                "key": self.key(),
                "method": self.method(),
                "pinned": pinned(),
                "schedule_seed": 0u64,
                "provenance": opaque(),
                "host": opaque(),
                "outcome": { "Failed": { "failure": opaque() } },
                "release": opaque(),
            }
        })
        .to_string()
    }

    /// The same attempt, but as a slot that never ran.
    pub(crate) fn not_run_line(&self) -> String {
        json!({
            "NotRun": {
                "key": self.key(),
                "method": self.method(),
                "pinned": pinned(),
                "schedule_seed": 0u64,
                "reason": opaque(),
            }
        })
        .to_string()
    }

    /// The method block: frozen by default, with whatever this fixture varies.
    ///
    /// **The split here is deliberate.** The field *names* and variant *spellings* are written out
    /// as literals, because that independence is the whole point: a fixture that took its shape from
    /// the writer's `Serialize` impl would agree with the DTOs by construction and could never
    /// detect a wire-shape disagreement. The frozen *values*, by contrast, follow the SSOT constants
    /// — writing `true` for confirmed reads or `1000` for the sample count would create a second
    /// copy of a truth the pilot deliberately single-sources, so flipping the real constant would
    /// leave these tests asserting a method no run uses. The four scalar varying facts default to
    /// exactly those constants and the channel defaults to the frozen spelling, so a fixture that
    /// overrides none of them still restates the freeze.
    fn method(&self) -> serde_json::Value {
        json!({
            "channel": self.channel,
            "sample_count": self.sample_count,
            "paced_sample_delay_ms": self.paced_sample_delay_ms,
            "seeded_own_rows": self.seeded_own_rows,
            "with_confirmed_reads": self.with_confirmed_reads,
            "outcome_ceiling": "MethodCalibrationOnly",
        })
    }

    fn key(&self) -> serde_json::Value {
        json!({
            "candidate": "IndexedControlActivitySenderView",
            "axis": "UnrelatedGlobalRows",
            "rung": "Baseline",
            "role": self.role,
            "stage": { "Calibration": self.replicate },
            "ordinal": "Original",
            "version": self.version,
        })
    }
}

/// A stand-in for a field this analyzer retains verbatim and never interprets.
fn opaque() -> serde_json::Value {
    json!({ "unmodelled": true })
}

/// A stand-in pinned-artifact block.
fn pinned() -> serde_json::Value {
    json!({ "unmodelled": true })
}
