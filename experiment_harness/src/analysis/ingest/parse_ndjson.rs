//! Decode a whole NDJSON artifact into the untrusted wire DTOs, one line at a time.

use crate::analysis::ingest::ingest_error::IngestError;
use crate::analysis::ingest::wire_record_dto::WireRecordDto;

/// Decode every line of an already-loaded NDJSON artifact into a [`WireRecordDto`], preserving file
/// order. Each line is decoded independently under the closed wire contract; the first line that fails
/// to decode aborts with a typed [`IngestError`] tagged with its 1-based position. This is purely the
/// *syntactic* boundary over in-memory text: opening/reading the artifact file is owned by a higher
/// typed report-input error, never collapsed in here, and sequence contiguity, completeness,
/// cross-references, and recomputation are the separate `validate` pass over the returned records.
pub(crate) fn parse_ndjson(contents: &str) -> std::result::Result<Vec<WireRecordDto>, IngestError> {
    contents
        .lines()
        .enumerate()
        .map(|(index, line)| {
            let line_number = u64::try_from(index + 1).expect("a 1-based line number fits u64");
            WireRecordDto::parse(line)
                .map_err(|diagnostic| IngestError::malformed_line(line_number, diagnostic))
        })
        .collect()
}
