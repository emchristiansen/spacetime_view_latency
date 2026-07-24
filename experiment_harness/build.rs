//! Embeds the enclosing checkout's current commit and tree cleanliness into the binary via
//! `cargo:rustc-env`, so `BuildProvenance::from_build_env()` (`src/manifest/build_provenance.rs`) can
//! parse them into a closed authoritative/development dispatch. Dependency-free: shells directly to the
//! `git` binary rather than pulling in a git library.

use std::process::Command;

fn main() {
    // `Cargo.toml/.force-build-script-rerun` can never exist — `Cargo.toml` is the package manifest
    // regular file, so this path can never resolve to a real file. Cargo therefore always considers this
    // rerun-if-changed target stale and reruns this build script on every invocation that evaluates the
    // package's build-script unit (Cargo FAQ: a missing watched file forces a rerun).
    println!("cargo:rerun-if-changed=Cargo.toml/.force-build-script-rerun");

    let commit = run_git(&["rev-parse", "HEAD"]);
    println!("cargo:rustc-env=HARNESS_BUILD_COMMIT={commit}");

    // A dirty tree does not fail this build script; it is embedded as ordinary evidence, and
    // `BuildProvenance::from_build_env` demotes it to `Development` rather than rejecting it.
    let tree_state = if run_git(&["status", "--porcelain"]).is_empty() {
        "clean"
    } else {
        "dirty"
    };
    println!("cargo:rustc-env=HARNESS_BUILD_TREE_STATE={tree_state}");
}

fn run_git(args: &[&str]) -> String {
    let output = Command::new("git").args(args).output().unwrap_or_else(|error| {
        panic!("running `git {args:?}` for harness build provenance: {error}")
    });
    assert!(
        output.status.success(),
        "`git {args:?}` for harness build provenance exited {}: {}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout)
        .unwrap_or_else(|error| panic!("`git {args:?}` output was not UTF-8: {error}"))
        .trim()
        .to_string()
}
