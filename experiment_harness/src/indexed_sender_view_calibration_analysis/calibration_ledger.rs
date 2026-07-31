//! A whole calibration ledger file, decoded, with no way to construct one from a fragment.

use std::path::Path;

use crate::indexed_sender_view_calibration_analysis::calibration_analysis_error::CalibrationAnalysisError;
use crate::indexed_sender_view_calibration_analysis::calibration_ingest_error::CalibrationIngestError;
use crate::indexed_sender_view_calibration_analysis::ledger_line::LedgerLine;

/// Every line of one calibration ledger **file**, in file order, with source positions minted at
/// decode.
///
/// **The provenance is the invariant, and in the production configuration it is enforced by what
/// cannot be called.** §569 is stated over a whole artifact: "the ledger contains exactly the two
/// frozen originals" is only a meaningful claim if the thing examined is the entire file. An earlier
/// shape decoded a crate-visible `&str`, so any caller could have decoded `&contents[..cut]`, a
/// hand-assembled subset, or a single record and passed the result on as a ledger — and every
/// downstream refusal, including the whole-inventory check, would then have been evaluated against a
/// fragment while reporting on a file.
///
/// [`Self::read`] is therefore the only way to obtain this type in any build of this crate that is
/// not itself a test binary. It takes a [`Path`], performs the read itself, and hands the complete
/// contents to a decoder that is **private to this module**; in that configuration no entry point
/// anywhere accepts contents, a line, a record, or a collection of [`LedgerLine`]s.
///
/// **The one exception is stated rather than glossed.**
/// `Self::from_complete_contents_for_tests` does accept contents, so under `cfg(test)` the
/// invariant above is upheld by the tests' own construction rather than by the type. It is
/// `#[cfg(test)]`, so it does not exist in the production configuration, and `pub(crate)`, so no
/// other crate can name it in any configuration.
///
/// Yielding the lines back out ([`Self::into_lines`]) is safe and unrelated in every configuration:
/// reading a decoded file grants no way to manufacture one.
///
/// The two failure modes stay distinct, as
/// [`CalibrationAnalysisError`](super::calibration_analysis_error::CalibrationAnalysisError)
/// documents: a path that cannot be opened or read is never collapsed into "malformed".
#[derive(Debug, Clone)]
pub(crate) struct CalibrationLedger {
    lines: Vec<LedgerLine>,
}

impl CalibrationLedger {
    /// Read and decode the complete ledger at `path`.
    ///
    /// **The only constructor in the production configuration**, and it owns both halves — the read
    /// and the decode — so no caller there can supply contents of its own choosing. That is the whole
    /// reason the read is not left to the caller as it once was. Under `cfg(test)`
    /// `Self::from_complete_contents_for_tests` also exists, and is named to say so.
    pub(crate) fn read(path: &Path) -> Result<Self, CalibrationAnalysisError> {
        let contents = std::fs::read_to_string(path).map_err(|error| {
            CalibrationAnalysisError::LedgerUnreadable {
                path: path.to_path_buf(),
                diagnostic: error.to_string(),
            }
        })?;
        decode_complete_contents(&contents).map_err(CalibrationAnalysisError::Malformed)
    }

    /// Decode complete in-memory ledger contents as though they had just been read from a file.
    ///
    /// **Test-only, and deliberately named to say so at every call site.** The pure tests in this
    /// namespace build their wire lines by hand — that independence from the writer's own
    /// `Serialize` impl is what makes them evidence of the documented shape — so they need a decode
    /// that does not go through the filesystem. This bypasses **file provenance only**: the caller
    /// supplies contents, so the resulting value's claim to be a whole file rests on the test's own
    /// construction rather than on [`Self::read`]. Everything after decoding — line numbering,
    /// admission, whole-inventory refusal — is the same code the production path runs.
    ///
    /// `#[cfg(test)]` means this associated function does not exist in a production build of the
    /// crate, following `ledger_fixture::LedgerFixture`, which is gated the same way for the same
    /// reason. It is `pub(crate)` rather than private because the fixtures that need it live in
    /// sibling modules; nothing outside this crate can name it in any configuration.
    #[cfg(test)]
    pub(crate) fn from_complete_contents_for_tests(
        contents: &str,
    ) -> Result<Self, CalibrationIngestError> {
        decode_complete_contents(contents)
    }

    /// Consume the ledger, yielding every decoded line with its source position.
    pub(crate) fn into_lines(self) -> Vec<LedgerLine> {
        self.lines
    }
}

/// Decode complete ledger contents, one line at a time, preserving file order.
///
/// **Private to this module by design** — it is the single point where text becomes a ledger, and
/// making it reachable from anywhere else would restore exactly the fragment-decoding hole
/// [`CalibrationLedger`] exists to close. Rust's visibility rules give this module's own descendants
/// access, which is what lets the focused decode tests exercise the real function rather than a
/// stand-in.
///
/// Each line is decoded independently under the closed wire contract, and the first line that fails
/// aborts with a typed error tagged with its 1-based position. Copies
/// [`parse_ndjson`](crate::analysis::ingest::parse_ndjson::parse_ndjson) exactly in that respect.
///
/// Blank lines are not skipped. A ledger written by the pilot's `append_all` has exactly one record
/// per line with no blank separators, so a blank line means the artifact is not the one this analyzer
/// was pointed at, and silently tolerating it would let a truncated or concatenated file look whole.
fn decode_complete_contents(contents: &str) -> Result<CalibrationLedger, CalibrationIngestError> {
    let lines = contents
        .lines()
        .enumerate()
        .map(|(index, line)| {
            let line_number = u64::try_from(index + 1).expect("a 1-based line number fits u64");
            let record = serde_json::from_str(line).map_err(|error| {
                CalibrationIngestError::malformed_line(line_number, error.to_string())
            })?;
            Ok(LedgerLine {
                line_number,
                record,
            })
        })
        .collect::<Result<Vec<LedgerLine>, CalibrationIngestError>>()?;
    Ok(CalibrationLedger { lines })
}

#[cfg(test)]
mod tests;
