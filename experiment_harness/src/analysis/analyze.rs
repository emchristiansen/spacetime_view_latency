//! Stage 6 entry: validate a campaign artifact and durably publish its authoritative report.

use anyhow::Result;

use crate::analysis::report::campaign_report::CampaignReport;
use crate::campaign::{CampaignPartialPath, CampaignRoot};

/// Ingest, validate, and durably publish the authoritative report for the complete campaign NDJSON
/// artifact at Campaign's fixed staged path under `campaign_root` — the whole stage-6 pipeline behind the
/// `Analyze` command. The single input is the shared `--campaign-root`; the output is the one
/// [`CampaignReport`], printed to stdout only after the corrected-v2 bundle is durably published. Phase-1
/// skeleton: a `todo!()` that names the Phase-2 wiring (ingest the fixed staged NDJSON at
/// [`CampaignPartialPath::under`] into untrusted DTOs, fold them into a
/// [`ValidatedCampaign`](crate::analysis::validate::validated_campaign::ValidatedCampaign) via the single
/// total validation pass, project [`CampaignReport::of`], then publish it through
/// [`CorrectedV2Staging::create`](crate::analysis::publish::corrected_v2_staging::CorrectedV2Staging::create)
/// `.promote()` `.fsync_parent()` before printing).
pub(crate) fn analyze(campaign_root: &CampaignRoot) -> Result<CampaignReport> {
    let partial_path = CampaignPartialPath::under(campaign_root);
    todo!(
        "Phase 2: ingest the fixed staged campaign NDJSON at {:?}, validate into a ValidatedCampaign, \
         project CampaignReport::of, then publish via CorrectedV2Staging::create/.promote()/\
         .fsync_parent() before printing",
        partial_path.path()
    )
}
