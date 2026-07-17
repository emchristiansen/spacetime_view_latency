//! The consuming-typestate run cursor: `WritingManifest → Dosing → (exhausted | stopped) → cleanup`.
//!
//! Each state owns the campaign's [`ObservationSink`](crate::observation::observation_sink) by move and
//! yields the next state from a `self`-consuming transition, so a run's steps cannot be called out of
//! order or twice. Execution ends at one of two **inert** pre-cleanup carriers — [`RunExecuted`] (dose
//! ladder exhausted) or [`RunExecutionStopped`] (a failing manifest/dose edge) — neither of which can
//! mint a terminal. Only the run's linear cleanup owner
//! ([`RunCleanup`](crate::campaign::run_cleanup::RunCleanup)) can consume a carrier and, after the ordered
//! disconnect-then-teardown, mint the terminal [`RunDone`]/[`RunIncomplete`] (and the affine
//! [`RunComplete`]) — a run cannot report completion before its client and server have been cleaned up.
//! This entry file is declarative module declarations and re-exports only.

mod run_complete;
mod run_done;
mod run_dose_step;
mod run_executed;
mod run_execution_stopped;
mod run_incomplete;
mod run_manifest_step;
mod run_observation_step;
mod run_settled;
mod run_writing_manifest;

pub(crate) use run_complete::RunComplete;
pub(crate) use run_done::RunDone;
pub(crate) use run_dose_step::RunDoseStep;
pub(crate) use run_executed::RunExecuted;
pub(crate) use run_execution_stopped::RunExecutionStopped;
pub(crate) use run_incomplete::RunIncomplete;
pub(crate) use run_manifest_step::RunManifestStep;
pub(crate) use run_observation_step::RunObservationStep;
pub(crate) use run_settled::RunSettled;
pub(crate) use run_writing_manifest::RunAwaitingDose;
pub(crate) use run_writing_manifest::RunDosing;
pub(crate) use run_writing_manifest::RunSeedingBackground;
pub(crate) use run_writing_manifest::RunWritingManifest;
