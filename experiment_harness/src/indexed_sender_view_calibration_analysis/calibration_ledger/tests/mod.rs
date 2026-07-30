//! Focused tests for the calibration ledger's syntactic boundary. One test entity per file.
//!
//! These live under `calibration_ledger` rather than beside it so they are **descendants** of the
//! module that owns the decoder, and can therefore call the module-private
//! `decode_complete_contents` directly. They exercise the exact function the production
//! [`read`](super::CalibrationLedger::read) path runs, with no test-only entry point in between.

mod an_unknown_field_is_rejected_with_its_line_number;
mod an_unknown_variant_spelling_fails_decoding;
