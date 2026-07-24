//! `PreregisteredParametersReport::of` reproduces the trusted parameters losslessly: `confirmed_reads` is
//! projected as the preregistered boolean flag (never a re-encoded count), and the ten-dose ladder collects
//! into the fixed array.

use crate::analysis::report::preregistered_parameters_report::PreregisteredParametersReport;
use crate::manifest::preregistered_parameters::PreregisteredParameters;
use crate::params::NUM_DOSES_USIZE;

#[test]
fn projects_the_preregistered_parameters_losslessly() {
    let parameters = PreregisteredParameters::preregistered();
    let report = PreregisteredParametersReport::of(&parameters);

    // `confirmed_reads` is the preregistered boolean flag exactly as the source models it. The field is a
    // `bool`, so a re-encoded count is unrepresentable; it must equal the source value bit-for-bit.
    assert_eq!(report.confirmed_reads, parameters.confirmed_reads());

    // Every remaining scalar is reproduced exactly across the projection boundary.
    assert_eq!(report.batch_size, parameters.batch_size());
    assert_eq!(report.num_doses, parameters.num_doses());
    assert_eq!(report.batch_delay_ms, parameters.batch_delay_ms());
    assert_eq!(report.repetition_blocks, parameters.repetition_blocks());

    // The cumulative dose ladder collects into the fixed NUM_DOSES-length array — its ten-dose cardinality
    // is a property of the report type — and reproduces the source ladder exactly.
    assert_eq!(report.dose_ladder.len(), NUM_DOSES_USIZE);
    assert_eq!(report.dose_ladder.as_slice(), parameters.dose_ladder());
}
