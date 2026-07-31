//! `CellStatistic::validated` is the only door in, and it admits exactly the positive rationals.

use crate::analysis::stats::rational::Rational;
use crate::view_read_set_campaign::cell_statistic::CellStatistic;

/// Coverage: zero and negatives are refused at the sole constructor, so no later code can be handed
/// a statistic that would make `T = S_last / S_first` undefined, or let a negative ratio satisfy the
/// flat inequality and be reported as flat.
///
/// The negative cases are written both ways round — `-1/2` and `1/-2` — because `Rational` canonicalizes
/// the sign onto the numerator, and a check that read the denominator instead would pass one of them.
#[test]
fn validated_admits_only_strictly_positive_statistics() {
    for (num, den) in [(1, 1), (1, 1_000_000), (i128::MAX, 1), (3, 7)] {
        let value = Rational::new(num, den);
        let statistic = CellStatistic::validated(value)
            .unwrap_or_else(|e| panic!("{num}/{den} is strictly positive: {e:#}"));
        assert_eq!(
            statistic.get(),
            value,
            "a validated statistic must carry its exact value unchanged"
        );
    }

    for (num, den) in [(0, 1), (0, 7), (-1, 2), (1, -2), (i128::MIN + 1, 1)] {
        assert!(
            CellStatistic::validated(Rational::new(num, den)).is_err(),
            "{num}/{den} is not strictly positive and must be refused"
        );
    }
}
