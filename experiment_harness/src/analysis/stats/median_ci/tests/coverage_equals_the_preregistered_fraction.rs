//! The exact coverage equals the preregistered rational `Σ_{k=10}^{20} C(30,k) / 2^30`, which is the
//! 95.7226% figure the spec records — checked as an exact rational, not merely a rounded decimal.

use crate::analysis::stats::median_ci::MedianCi;
use crate::analysis::stats::rational::Rational;

#[test]
fn coverage_equals_the_preregistered_fraction() {
    // Numerator hand-computed by symmetry: 2^30 - 2 * Σ_{k=0}^{9} C(30,k)
    //   = 1_073_741_824 - 2 * 22_964_087 = 1_027_813_650.
    // `Rational::new` reduces both this and `coverage()` to the same canonical form, so equality is
    // exact rational equality, not a floating-point approximation.
    let expected = Rational::new(1_027_813_650, 1_073_741_824);
    assert_eq!(MedianCi::coverage(), expected);

    // And it renders to the preregistered 95.7226% decimal.
    assert!((MedianCi::coverage().to_f64() - 0.957_226).abs() < 1e-6);
}
