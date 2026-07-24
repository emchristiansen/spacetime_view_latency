//! A canonical 40-character lowercase-hex commit parses and round-trips to the same canonical hex.

use crate::manifest::release_commit::ReleaseCommit;

#[test]
fn a_canonical_lowercase_hex_commit_parses() {
    let hex = "0".repeat(ReleaseCommit::CANONICAL_HEX_LEN);
    let commit =
        ReleaseCommit::parse_canonical_hex(&hex).expect("canonical lowercase hex must parse");
    assert_eq!(
        commit.to_string(),
        hex,
        "the parsed commit round-trips to the exact canonical hex it was parsed from"
    );
}
