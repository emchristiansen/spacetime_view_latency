//! The created directory is the identity's canonical tag, as a single component directly under the
//! campaign root.

use std::path::{Component, MAIN_SEPARATOR};

use crate::view_read_set_campaign::composition_validation::attempt_artifact_directory::AttemptArtifactDirectory;

use super::pilot_attempt::pilot_attempt;
use super::scratch_dir::scratch_dir;

/// Coverage: the artifact path is the only route from a recorded identity back to its evidence, so
/// two properties have to hold at once — the directory really is named by the whole canonical tag,
/// and it really is *inside* the root the operator supplied.
///
/// The second is the traversal claim, and this test states it the way the type does: not by feeding
/// hostile input and checking for rejection — there is no input to feed, since the component is
/// derived from a typed [`AttemptKey`](crate::view_read_set_campaign::attempt_key::AttemptKey)
/// rather than supplied — but by requiring the created path to decompose into the root plus exactly
/// one normal component. A component containing a separator, or spelling `.` or `..`, could not
/// satisfy that. So the assertion below is the observable shadow of "traversal is unrepresentable":
/// it holds for every identity because the vocabulary the tag is assembled from is frozen and
/// contains no separator at all, which the separator check states directly.
#[test]
fn the_directory_is_one_named_component_inside_the_root() {
    let root = scratch_dir("one-named-component");
    let attempt = pilot_attempt(0);
    let tag = attempt.canonical_tag();

    let directory = AttemptArtifactDirectory::create(&root, attempt)
        .expect("an existing root and an unused attempt identity create the directory");

    assert_eq!(
        directory.path(),
        root.join(&tag),
        "the attempt directory is the canonical identity tag under the campaign root"
    );
    assert!(
        directory.path().is_dir(),
        "the directory named by {tag:?} must exist on disk after a successful create"
    );

    // The path decomposes as root + exactly one normal component: nothing was escaped, and nothing
    // was nested.
    let relative = directory
        .path()
        .strip_prefix(&root)
        .expect("the attempt directory is created inside the campaign root");
    let components: Vec<Component<'_>> = relative.components().collect();
    match components.as_slice() {
        [Component::Normal(only)] => assert_eq!(
            *only,
            tag.as_str(),
            "the single component under the root is the canonical identity tag"
        ),
        other => panic!(
            "the attempt directory must be exactly one normal component under the root, got \
             {other:?}"
        ),
    }

    assert!(
        !tag.contains(MAIN_SEPARATOR) && !tag.contains('/'),
        "the canonical tag {tag:?} is a filename component, so it can contain no path separator"
    );
}
