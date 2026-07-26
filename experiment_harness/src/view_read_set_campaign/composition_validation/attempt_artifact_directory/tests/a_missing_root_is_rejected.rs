//! A campaign root that does not exist, or is not a directory, is rejected rather than created.

use std::fs;

use crate::view_read_set_campaign::composition_validation::attempt_artifact_directory::AttemptArtifactDirectory;

use super::pilot_attempt::pilot_attempt;
use super::scratch_dir::scratch_dir;

/// Coverage: the root is an operator-supplied path, and `create_dir_all` on a mistyped one would
/// succeed — producing a plausible-looking empty campaign tree beside the real evidence, whose
/// findings point at artifacts nobody will look for. Requiring the root to exist is what turns that
/// into a loud failure.
///
/// Both non-directory shapes are checked, because they fail for different reasons and only one of
/// them is caught by an existence test: a missing path, and a path that exists as a *file*. The
/// second is the one a bare `exists()` check would have let through, and it is a realistic mistyping
/// of an operator's own ledger path.
///
/// The test also requires that nothing was created on either failure. A rejection that had already
/// made a directory somewhere would leave exactly the stray tree this check exists to prevent.
#[test]
fn a_missing_root_is_rejected() {
    let scratch = scratch_dir("missing-root");
    let attempt = pilot_attempt(0);

    let absent = scratch.join("no-such-campaign-root");
    let error = AttemptArtifactDirectory::create(&absent, attempt)
        .expect_err("a campaign root that does not exist must be rejected, not created");
    assert!(
        format!("{error:#}").contains("not an existing directory"),
        "the failure must name the missing root, got {error:#}"
    );
    assert!(
        !absent.exists(),
        "a rejected root must not have been created on the way out"
    );

    let file_root = scratch.join("campaign-root-that-is-a-file");
    fs::write(&file_root, b"an operator's mistyped path").expect("writing the stand-in file");
    let error = AttemptArtifactDirectory::create(&file_root, attempt)
        .expect_err("a campaign root that is a file must be rejected");
    assert!(
        format!("{error:#}").contains("not an existing directory"),
        "the failure must name the non-directory root, got {error:#}"
    );
    assert!(
        file_root.is_file(),
        "a rejected root must be left exactly as it was found"
    );
}
