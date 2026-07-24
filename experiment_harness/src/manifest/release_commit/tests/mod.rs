//! Tests for the single-owner canonical release-commit hex parser. One test entity per file.

mod a_canonical_lowercase_hex_commit_parses;
mod a_non_hex_character_is_rejected;
mod a_wrong_length_hex_is_rejected;
mod an_uppercase_hex_is_rejected_as_non_canonical;
