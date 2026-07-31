//! Memory pressure reads the `full` line's `avg60`, not `some`'s and not another average.

use super::super::parse_memory_psi_full_avg60_centi;

/// Verbatim `/proc/pressure/memory`, every average distinct.
const PRESSURE: &str = "some avg10=1.10 avg60=2.20 avg300=3.30 total=31668020845\n\
                        full avg10=4.40 avg60=5.50 avg300=6.60 total=30672397021\n";

#[test]
fn memory_pressure_reads_full_avg60_and_not_some() {
    assert_eq!(
        parse_memory_psi_full_avg60_centi(PRESSURE).expect("a well-formed pressure file parses"),
        550,
        "the gate names the full line's sixty-second average"
    );
    assert!(
        parse_memory_psi_full_avg60_centi("some avg10=1.10 avg60=2.20 avg300=3.30 total=1\n")
            .is_err(),
        "a file with only a some line cannot answer the full clause"
    );
    assert!(
        parse_memory_psi_full_avg60_centi("full avg10=4.40 avg30=5.50 avg300=6.60 total=1\n")
            .is_err(),
        "every field is checked against its own key, so a reordered or renamed line cannot place \
         avg60 by position"
    );
    assert!(
        parse_memory_psi_full_avg60_centi("full avg10=x avg60=5.50 avg300=6.60 total=1\n").is_err(),
        "a malformed ten-second average is a malformed line, though nothing reads it"
    );
    assert!(
        parse_memory_psi_full_avg60_centi("full avg10=4.40 avg60=5.50 avg300=6.6 total=1\n")
            .is_err(),
        "the three-hundred-second average is held to the same two decimal places"
    );
    assert!(
        parse_memory_psi_full_avg60_centi("full avg10=4.40 avg60=5.50 avg300=6.60 total=x\n")
            .is_err(),
        "a nonnumeric cumulative total is a malformed line"
    );
}
