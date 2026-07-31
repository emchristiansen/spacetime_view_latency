//! Focused tests for the durable campaign ledger. One test entity per file.
//!
//! The sink is one state machine over three methods, and this tree covers most of it: the exclusive
//! create, the line shape and sequence assignment of every *purely constructible* record shape —
//! five of the six the ledger can carry — the terminal poisoning a persist failure causes and the
//! refusal every successor then gets, and all four `(poison, final sync)` outcomes of finalization.
//! Three things it does not reach are stated below rather than implied away.
//!
//! **The one record shape it cannot reach: `Provisioned`.** That variant carries an
//! [`AttemptProvenance`](crate::view_read_set_campaign::attempt_provenance::AttemptProvenance),
//! whose sole constructor snapshots a verified distribution, a running server, and a published
//! module artifact — none of which a unit test owns. So the line-shape coverage below is five of the
//! six variants, and the sixth is compile-checked only. The gap is stated rather than closed: a
//! test-only provenance seam would make the reconciliation gate forgeable, which is exactly what
//! that sealed constructor exists to prevent, and it would buy one more JSON object here.
//!
//! **The one poison branch it cannot reach: the serialization failure.** `append` poisons from two
//! places — a failing `serde_json::to_vec`, and a failing write/flush/`sync_data` — and only the
//! second is exercised. [`ScriptedWriter`](scripted_writer::ScriptedWriter) injects at the seam
//! *below* serialization, which is the only seam the sink has; and no record this tree can construct
//! makes `serde_json` fail, since the closed
//! [`CampaignRecord`](crate::view_read_set_campaign::campaign_record::CampaignRecord) vocabulary is
//! built from integers, strings, and closed enums rather than the shapes that can fail. Reaching
//! that arm would take either a serializer seam production would never use or a record type
//! deliberately weakened to fail — so it stays direct-inspection-only, and the honest reading of the
//! terminal-poisoning test is that it proves the transition for one of the two entry points into it.
//!
//! **What "the sequence advances only after full durability" can and cannot show.** Its forward half
//! is observable — consecutive successful appends take 0, 1, 2 — and is asserted in
//! [`each_record_is_one_line_carrying_its_sequence_and_kind`]. Its failure half is not directly
//! observable, because the sink is terminal on the first failure: no write can follow one to reveal
//! whether the counter moved. What is observable, and is asserted in
//! [`the_first_persist_failure_makes_the_ledger_terminal`], is the consequence that matters — the
//! failed sequence is named in the retained diagnostic, no successor ever writes at all, and every
//! successor re-reports that original reason instead of attempting a line of its own.
//!
//! The two real-filesystem tests use their own scratch directory, as every other test tree in this
//! campaign does; the two fake writers are the fixtures the historical sinks' trees already use.

mod capturing_writer;
mod fixture;
mod scratch_dir;
mod scripted_writer;

mod a_created_ledger_holds_exactly_the_lines_it_appended;
mod create_is_exclusive;
mod each_record_is_one_line_carrying_its_sequence_and_kind;
mod finalize_reports_a_failed_sync_and_any_retained_poison;
mod the_first_persist_failure_makes_the_ledger_terminal;
