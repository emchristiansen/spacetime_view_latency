//! An observation's immutable reference back to its run manifest.

use serde::Serialize;

use crate::manifest::database_identity::DatabaseIdentity;
use crate::manifest::run_coordinate::RunCoordinate;
use crate::manifest::schedule_seed::ScheduleSeed;
use crate::manifest::validated_run_manifest::ValidatedRunManifest;

/// A deterministic, canonical reference from a dose observation back to its run manifest. It is the
/// manifest's existing immutable identity tuple — the run coordinate, the server-issued database
/// identity, and the schedule seed — not an ad-hoc digest of serialized bytes (which would require a
/// proven canonicalization). A reader resolves an observation to its manifest by equality of this
/// tuple.
///
/// This type only supplies that structural identity; it does not order records. The upcoming private
/// run-session owner — not sink-call discipline — will write the manifest record first and exactly
/// once per run; [`ObservationSink`](super::observation_sink::ObservationSink) itself does not enforce
/// manifest-before-observations. Keying an observation to a manifest by this tuple is structural; it
/// is not a claim that the observation's samples were measured under that manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct ManifestReference {
    run: RunCoordinate,
    database_identity: DatabaseIdentity,
    schedule_seed: ScheduleSeed,
}

impl ManifestReference {
    /// Derive the reference from the immutable facts of a manifest.
    pub(crate) fn of(manifest: &ValidatedRunManifest) -> Self {
        Self {
            run: manifest.run_coordinate(),
            database_identity: manifest.database_identity(),
            schedule_seed: manifest.schedule_seed(),
        }
    }
}
