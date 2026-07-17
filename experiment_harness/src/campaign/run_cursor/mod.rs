//! The consuming-typestate run cursor: `WritingManifest → Dosing → Disconnecting → Teardown → done`.
//!
//! Each state owns the campaign's [`ObservationSink`](crate::observation::observation_sink) by move and
//! yields the next state (or a terminal [`RunDone`]/[`RunIncomplete`]) from a `self`-consuming
//! transition, so a run's steps cannot be called out of order or twice. Completion evidence
//! ([`RunComplete`]) is minted only by the teardown transition, so a run cannot report completion
//! before it has advanced past every dose, the disconnect, and the teardown. This entry file is
//! declarative module declarations and re-exports only.

mod run_awaiting_dose;
mod run_complete;
mod run_disconnecting;
mod run_done;
mod run_dose_step;
mod run_dosing;
mod run_incomplete;
mod run_manifest_step;
mod run_observation_step;
mod run_teardown;
mod run_writing_manifest;

pub(crate) use run_awaiting_dose::RunAwaitingDose;
pub(crate) use run_complete::RunComplete;
pub(crate) use run_disconnecting::RunDisconnecting;
pub(crate) use run_done::RunDone;
pub(crate) use run_dose_step::RunDoseStep;
pub(crate) use run_dosing::RunDosing;
pub(crate) use run_incomplete::RunIncomplete;
pub(crate) use run_manifest_step::RunManifestStep;
pub(crate) use run_observation_step::RunObservationStep;
pub(crate) use run_teardown::RunTeardown;
pub(crate) use run_writing_manifest::RunWritingManifest;
