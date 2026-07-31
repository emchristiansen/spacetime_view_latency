//! One attempt identity can create its artifact directory exactly once; a different block cannot
//! collide with it.

use crate::view_read_set_campaign::composition_validation::attempt_artifact_directory::AttemptArtifactDirectory;

use super::pilot_attempt::pilot_attempt;
use super::scratch_dir::scratch_dir;

/// Coverage: exclusive creation is what makes a repeated attempt identity a loud failure instead of
/// two attempts quietly sharing one directory. Sharing would be the worse outcome by far — the
/// second attempt's artifacts would land beside the first's, and its exclusive *file* creation would
/// then fail further along, at a point where the first attempt's retained evidence has already been
/// made ambiguous.
///
/// The second half is the other side of the same claim: a directory named by a *different* identity
/// must still be creatable under the same root. Without it, this test would also pass for an
/// implementation that refused every create after the first, which would break the campaign at its
/// second attempt.
#[test]
fn a_repeated_attempt_identity_fails_exclusively() {
    let root = scratch_dir("repeated-identity");
    let attempt = pilot_attempt(0);

    let first = AttemptArtifactDirectory::create(&root, attempt)
        .expect("the first create for an unused attempt identity succeeds");

    let error = AttemptArtifactDirectory::create(&root, attempt)
        .expect_err("a second create for the same attempt identity must not succeed");
    assert!(
        format!("{error:#}").contains(&attempt.canonical_tag()),
        "the failure must name the directory it could not exclusively create, got {error:#}"
    );
    assert!(
        first.path().is_dir(),
        "the failed second create must leave the first attempt's directory intact"
    );

    // A different identity under the same root is a different directory, so the campaign's next
    // attempt is unaffected.
    let sibling = pilot_attempt(1);
    let second = AttemptArtifactDirectory::create(&root, sibling)
        .expect("a distinct attempt identity creates its own directory under the same root");
    assert_ne!(
        first.path(),
        second.path(),
        "two attempts differing only by block must not share an artifact directory"
    );
}
