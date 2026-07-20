//! Tests for the single-owner canonical database-identity hex parser. One test entity per file.

mod a_canonical_lowercase_hex_identity_parses;
mod a_non_hex_character_is_rejected;
mod a_wrong_length_hex_is_rejected;
mod an_uppercase_hex_is_rejected_as_non_canonical;
mod serialize_routes_through_canonical_hex;
