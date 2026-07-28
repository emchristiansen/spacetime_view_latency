//! A factor reaches the ledger as its exact reduced numerator and denominator, never as a float.

use crate::view_read_set_campaign::endpoint_factor::{endpoint_ratio, exact_pair};

use super::fixture;

/// Coverage: the serialized content, taken from a real endpoint ratio rather than a hand-built
/// rational. `10/3` is unrepresentable in binary floating point, so this pins that the ledger keeps
/// the value the classification used. The one-line `Serialize` delegation is covered by inspection,
/// since minting an `EndpointFactor` needs a live campaign's ladder.
#[test]
fn the_serialized_pair_is_the_exact_reduced_ratio() {
    let cells = fixture::ascending([3, 4, 5, 6, 8, 10]);

    assert_eq!(exact_pair(endpoint_ratio(cells)), [10, 3]);
}
