//! The typed contradiction behind a
//! [`CoordinateReferenceMismatch`](super::integrity_error::IntegrityError) failure.

use crate::manifest::run_coordinate::RunCoordinate;
use crate::roles::role::Role;

/// Why a dose observation's coordinate disagrees with the manifest it references. Each mode carries the
/// trusted run location plus typed expected/observed evidence; the driving-role tag's `observed` is the
/// raw wire tag string precisely because parsing it into the canonical [`Role`] tag is the check that
/// failed.
#[derive(Debug)]
pub(crate) enum CoordinateReferenceFault {
    /// The observation coordinate's run disagrees with its manifest reference's run.
    ReferenceRunDisagreement {
        coordinate_run: RunCoordinate,
        reference_run: RunCoordinate,
    },
    /// The serialized driving-role tag (`observed`) is not the canonical tag
    /// ([`Role::canonical_tag`]) of the run's driving role (`expected`), derived from the single
    /// [`GrowthRegime::driving_role`](crate::plan::growth_regime::GrowthRegime::driving_role) owner. The
    /// run is carried here directly so this fault's semantic location is the run, not merely the
    /// enclosing record sequence.
    DrivingRoleTag {
        run: RunCoordinate,
        expected: Role,
        observed: String,
    },
}
