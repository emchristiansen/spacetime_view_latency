//! Strict line splitting for `--version` output.

use anyhow::{ensure, Context, Result};

/// Split exact `--version` output into its `expected` lines, rejecting any deviation.
///
/// Unlike [`str::lines`], this requires the observed grammar precisely: the output must
/// end with exactly one terminal `\n`, contain no carriage returns, and — after removing
/// that single terminator — split into exactly `expected` lines with no blank trailing
/// line. A missing terminal newline, a CRLF, an extra newline, or extra output all fail.
pub(crate) fn strict_version_lines(raw: &str, expected: usize) -> Result<Vec<&str>> {
    ensure!(!raw.contains('\r'), "version output contains a carriage return: {raw:?}");
    let body = raw
        .strip_suffix('\n')
        .with_context(|| format!("version output is missing its terminal newline: {raw:?}"))?;
    ensure!(
        !body.ends_with('\n'),
        "version output has a blank trailing line (extra newline): {raw:?}"
    );
    let lines: Vec<&str> = body.split('\n').collect();
    ensure!(
        lines.len() == expected,
        "version output expected exactly {expected} line(s), got {}: {raw:?}",
        lines.len()
    );
    Ok(lines)
}
