//! The one-minute load average is `/proc/loadavg`'s first field, in exact hundredths.

use super::super::parse_one_minute_load_centi;

/// Verbatim `/proc/loadavg`, all five fields distinct.
const LOADAVG: &str = "6.91 8.87 9.77 6/9099 1918361\n";

#[test]
fn a_load_average_reads_its_first_field_in_hundredths() {
    assert_eq!(
        parse_one_minute_load_centi(LOADAVG).expect("a well-formed loadavg line parses"),
        691,
        "the one-minute average is field zero, in hundredths"
    );
    assert!(
        parse_one_minute_load_centi("6.9 8.87 9.77 6/9099 1918361\n").is_err(),
        "one decimal place is a changed contract, not 90 hundredths"
    );
    assert!(
        parse_one_minute_load_centi("6.91 8.87 9.77 6/9099\n").is_err(),
        "a short line is rejected rather than read for its first token"
    );
    assert!(
        parse_one_minute_load_centi("6.91 x 9.77 6/9099 1918361\n").is_err(),
        "all three averages are checked, so a line shaped unlike loadavg cannot place field zero"
    );
    assert!(
        parse_one_minute_load_centi("6.91 8.87 9.77 6-9099 1918361\n").is_err(),
        "a task pair that is not running/total is a malformed line, though nothing reads it"
    );
    assert!(
        parse_one_minute_load_centi("6.91 8.87 9.77 6/x 1918361\n").is_err(),
        "both halves of the task pair must be counts"
    );
    assert!(
        parse_one_minute_load_centi("6.91 8.87 9.77 6/9099 x\n").is_err(),
        "a nonnumeric last pid is a malformed line"
    );
}
