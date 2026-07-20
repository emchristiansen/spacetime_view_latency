//! Stage 6 entry: validate a campaign artifact and project its authoritative report.

use std::path::Path;

use anyhow::Result;

use crate::analysis::report::campaign_report::CampaignReport;

/// Ingest, validate, and project the authoritative report for a complete campaign NDJSON artifact — the
/// whole stage-6 pipeline behind the `Analyze` command. The single input is the artifact path; the output
/// is the one [`CampaignReport`], the sole report projection. Phase-1 skeleton: a `todo!()` that names the
/// Phase-2 wiring (ingest the NDJSON into untrusted DTOs, fold them into a
/// [`ValidatedCampaign`](crate::analysis::validate::validated_campaign::ValidatedCampaign) via the single
/// total validation pass, then project [`CampaignReport::of`]).
pub(crate) fn analyze(input: &Path) -> Result<CampaignReport> {
    let _ = input;
    todo!("Phase 2: ingest the NDJSON artifact, validate into a ValidatedCampaign, and project CampaignReport::of")
}
