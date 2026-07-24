//! Stage 6: the authoritative, self-contained, machine-readable JSON report (stdout is JSON only).
//!
//! Every type here is a `Serialize` *projection* DTO: it owns its serialized shape, and the exact primary
//! types ([`Rational`](crate::stats), median CI, [`BetaCandidate`](crate::analysis::beta::beta_candidate::BetaCandidate))
//! remain non-`Serialize` and cross the lossy floating-point boundary only inside report construction.
//! Every newly stored serializable float is a [`FiniteF64`](crate::analysis::finite_f64::FiniteF64) or a
//! typed undefined variant, so every report value is valid JSON by construction — no NaN/±∞ sentinel and
//! no non-finite "number" can be serialized.
//!
//! One public entity per file; this entry file is declarative module declarations only. The one report
//! projection is [`CampaignReport::of`](campaign_report::CampaignReport::of), which takes exactly one
//! `&ValidatedCampaign`; every nested DTO likewise has a single-input `of`/`from` constructor, so no report
//! value can pair evidence from two different sources.

pub(crate) mod autocorrelation_report;
pub(crate) mod basin_refinement_report;
pub(crate) mod basin_selection_report;
pub(crate) mod basin_termination_report;
pub(crate) mod beta_candidate_report;
pub(crate) mod beta_label_report;
pub(crate) mod beta_population_report;
pub(crate) mod block_convergence_report;
pub(crate) mod block_fit_report;
pub(crate) mod block_report;
pub(crate) mod block_search_report;
pub(crate) mod campaign_report;
pub(crate) mod cell_evidence_report;
pub(crate) mod cell_identity_report;
pub(crate) mod cell_report;
pub(crate) mod classified_cell_report;
pub(crate) mod collection_order_plot_report;
pub(crate) mod collection_order_point_report;
pub(crate) mod dose_report;
pub(crate) mod environment_report;
pub(crate) mod escalation_assessment_report;
pub(crate) mod escalation_recommendation;
pub(crate) mod estimator_convergence_report;
pub(crate) mod golden_stop_report;
pub(crate) mod grid_node_report;
pub(crate) mod grid_nodes_report;
pub(crate) mod grid_outcome_report;
pub(crate) mod mechanism_citation_report;
pub(crate) mod mechanism_kind;
pub(crate) mod median_interval_report;
pub(crate) mod non_empty_cells;
pub(crate) mod non_empty_unassessable_cells;
pub(crate) mod non_increasing_response_report;
pub(crate) mod preregistered_parameters_report;
pub(crate) mod raw_latencies_report;
pub(crate) mod run_provenance_report;
pub(crate) mod run_report;
pub(crate) mod secondary_descriptor_report;
pub(crate) mod source_claims_report;
pub(crate) mod stock_observability_limit;
pub(crate) mod stock_observability_report;
pub(crate) mod temporal_diagnostics_report;
pub(crate) mod unassessable_cell;
pub(crate) mod unassessable_reason;
