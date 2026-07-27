//! The gate is a conjunction: the baseline passes, and violating any one clause alone refuses it.

use super::fixture::{
    passing_first, passing_second, AVAILABLE_RAM_BYTES, FIRST_LOAD_CENTI, FIRST_OFFSET_NANOS,
    LOGICAL_CPUS, PSI_CENTI, SECOND_LOAD_CENTI, SECOND_OFFSET_NANOS, SWAP_OUT_BYTES,
};
use crate::view_read_set_campaign::environment_gate_evidence::EnvironmentGateEvidence;
use crate::view_read_set_campaign::environment_sample::EnvironmentSample;

/// The load ceiling this fixture's CPU count implies, in hundredths.
const LOAD_CEILING_CENTI: u64 = LOGICAL_CPUS * 100;

/// Coverage: each of the five frozen clauses, isolated. Every case varies one reading of the second
/// sample and leaves the rest at the baseline, so a refusal names the clause it came from and no
/// clause is proved by another one failing alongside it.
///
/// The two load clauses are separated deliberately: a load above the ceiling that also failed to fall
/// would prove neither, so the ceiling case starts from a *higher* first load and still falls.
#[test]
fn every_clause_of_the_frozen_gate_must_hold() {
    let baseline = EnvironmentGateEvidence::paired(passing_first(), passing_second(), LOGICAL_CPUS)
        .expect("the baseline pair is the frozen separation apart");
    assert!(
        baseline.passes(),
        "readings at every threshold the gate admits must pass it, or the bounds are exclusive"
    );

    let falling_from_higher = EnvironmentSample::observed(
        FIRST_OFFSET_NANOS,
        LOAD_CEILING_CENTI + 2,
        AVAILABLE_RAM_BYTES,
        SWAP_OUT_BYTES,
        PSI_CENTI,
    );
    assert_refused(
        falling_from_higher,
        second_sample(
            LOAD_CEILING_CENTI + 1,
            AVAILABLE_RAM_BYTES,
            SWAP_OUT_BYTES,
            PSI_CENTI,
        ),
        "a load above one runnable task per CPU is refused even while falling",
    );

    assert_refused(
        passing_first(),
        second_sample(
            FIRST_LOAD_CENTI,
            AVAILABLE_RAM_BYTES,
            SWAP_OUT_BYTES,
            PSI_CENTI,
        ),
        "a load equal to the first sample's is not strictly lower and is refused",
    );

    assert_refused(
        passing_first(),
        second_sample(
            SECOND_LOAD_CENTI,
            AVAILABLE_RAM_BYTES - 1,
            SWAP_OUT_BYTES,
            PSI_CENTI,
        ),
        "one byte below the frozen available-RAM floor is refused",
    );

    for swap_out_bytes in [SWAP_OUT_BYTES + 1, SWAP_OUT_BYTES - 1] {
        assert_refused(
            passing_first(),
            second_sample(
                SECOND_LOAD_CENTI,
                AVAILABLE_RAM_BYTES,
                swap_out_bytes,
                PSI_CENTI,
            ),
            "the swap-out delta must be exactly zero; a counter that moved either way is refused \
             rather than clamped",
        );
    }

    assert_refused(
        passing_first(),
        second_sample(
            SECOND_LOAD_CENTI,
            AVAILABLE_RAM_BYTES,
            SWAP_OUT_BYTES,
            PSI_CENTI + 1,
        ),
        "one hundredth above the frozen memory-pressure ceiling is refused",
    );
}

/// A second sample at the frozen offset, carrying the four gate readings.
fn second_sample(
    one_minute_load_centi: u64,
    available_ram_bytes: u64,
    cumulative_swap_out_bytes: u64,
    memory_psi_full_avg60_centi: u64,
) -> EnvironmentSample {
    EnvironmentSample::observed(
        SECOND_OFFSET_NANOS,
        one_minute_load_centi,
        available_ram_bytes,
        cumulative_swap_out_bytes,
        memory_psi_full_avg60_centi,
    )
}

/// Pair two readings the constructor accepts and assert the gate itself refuses them.
fn assert_refused(first: EnvironmentSample, second: EnvironmentSample, why: &str) {
    let evidence = EnvironmentGateEvidence::paired(first, second, LOGICAL_CPUS)
        .expect("every case keeps the frozen separation, so only the clause under test differs");
    assert!(!evidence.passes(), "{why}");
}
