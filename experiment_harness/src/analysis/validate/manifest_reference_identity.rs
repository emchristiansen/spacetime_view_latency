//! The immutable identity tuple an observation uses to bind to its run manifest.

use crate::manifest::database_identity::DatabaseIdentity;
use crate::manifest::run_coordinate::RunCoordinate;
use crate::manifest::schedule_seed::ScheduleSeed;

/// A manifest's canonical identity — its run coordinate, server-issued database identity, and schedule
/// seed — reconstructed from validated wire primitives.
///
/// This mirrors the production
/// [`ManifestReference`](crate::observation::manifest_reference::ManifestReference) tuple (which cannot
/// be rebuilt from wire, since the production manifest is `Serialize`-only), and is the key the
/// validation pass uses to bind each observation to exactly one manifest and to carry typed evidence
/// when that binding fails. Every component is its own domain type, so the identity is fully typed
/// rather than a normalized string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ManifestReferenceIdentity {
    run: RunCoordinate,
    database_identity: DatabaseIdentity,
    schedule_seed: ScheduleSeed,
}

impl ManifestReferenceIdentity {
    /// Bind the three already-validated identity components into one comparable identity.
    pub(crate) fn new(
        run: RunCoordinate,
        database_identity: DatabaseIdentity,
        schedule_seed: ScheduleSeed,
    ) -> Self {
        Self {
            run,
            database_identity,
            schedule_seed,
        }
    }

    /// The run coordinate this identity binds to — the narrowest location for a dangling or
    /// unreferenced binding failure.
    pub(crate) fn run(&self) -> &RunCoordinate {
        &self.run
    }
}
