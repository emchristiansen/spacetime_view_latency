//! Swap-out reads `pswpout` and not its five-character-shared neighbour `pswpin`.

use super::super::parse_swap_out_pages;

/// Verbatim `/proc/vmstat` excerpt with `pswpin` immediately before `pswpout`.
const VMSTAT: &str = "pgpgin 1234\npswpin 7443573924\npswpout 8827959175\npgfault 99\n";

#[test]
fn swap_out_reads_the_cumulative_pswpout_counter() {
    assert_eq!(
        parse_swap_out_pages(VMSTAT).expect("a well-formed vmstat parses"),
        8_827_959_175,
        "the counter read is pswpout, in pages"
    );
    assert!(
        parse_swap_out_pages("pswpin 7443573924\n").is_err(),
        "swap-in alone is not a swap-out reading, despite the shared prefix"
    );
    assert!(
        parse_swap_out_pages("pswpoutx 1\n").is_err(),
        "a key merely prefixed by pswpout counts something else"
    );
}
