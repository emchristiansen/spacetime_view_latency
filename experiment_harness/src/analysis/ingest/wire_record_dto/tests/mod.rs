//! Tests for the whole-line wire envelope's closed, field-order-independent kind dispatch. One test
//! entity per file; `wire_lines` is the shared line-building fixture.

mod wire_lines;

mod a_duplicate_body_field_is_rejected;
mod a_duplicate_envelope_field_is_rejected;
mod a_missing_body_field_is_rejected;
mod a_missing_envelope_field_is_rejected;
mod an_unknown_body_field_is_rejected;
mod an_unknown_envelope_field_is_rejected;
mod an_unknown_record_kind_is_rejected;
mod body_field_may_precede_the_record_field;
mod manifest_and_dose_bodies_dispatch_exactly;
