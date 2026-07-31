//! `MemAvailable` is selected by exact key and converted from kibibytes, despite its `kB` label.

use super::super::parse_available_ram_bytes;

/// Verbatim `/proc/meminfo` head, with the confusable `MemTotal`/`MemFree` keys present and every
/// value distinct.
const MEMINFO: &str = "MemTotal:       263364228 kB\nMemFree:         2104920 kB\nMemAvailable:   13965436 kB\nBuffers:          412844 kB\n";

#[test]
fn available_memory_is_the_kernels_kibibyte_line_in_bytes() {
    assert_eq!(
        parse_available_ram_bytes(MEMINFO).expect("a well-formed meminfo parses"),
        13_965_436 * 1_024,
        "the kernel's `kB` is kibibytes, so the multiplier is 1024"
    );
    assert!(
        parse_available_ram_bytes("MemTotal:       263364228 kB\n").is_err(),
        "a file without MemAvailable is rejected rather than falling back to another line"
    );
    assert!(
        parse_available_ram_bytes("MemAvailable:   13965436\n").is_err(),
        "a missing unit is a changed contract"
    );
    assert!(
        parse_available_ram_bytes("MemAvailable:Extra 1 kB\n").is_err(),
        "a key merely prefixed by MemAvailable: is a different quantity, and satisfies every later \
         check"
    );
}
