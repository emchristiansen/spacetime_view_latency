//! Acquisition never reports a fact it dropped on the way to a later one.

use crate::control_registry_discovery_screen::provision_depth::ProvisionDepth;

/// Coverage: the four depths form a monotone chain, each adding exactly one fact and never losing
/// one.
///
/// Monotonicity is what makes a provisioning failure's record readable at all: a reader takes the
/// depth to mean "everything up to here was established", so a chain that could report a server
/// without the distribution that resolved it would make that reading false.
///
/// Checked on [`ProvisionDepth`] rather than `PartialProvision` because the latter's variants carry
/// `DistributionFacts` and `ServerFacts`, which can only be observed from live provisioning
/// capabilities — a rule expressed only over those variants could not be proven outside a real run.
/// The depths are the discriminant, and `PartialProvision::depth` is the total map onto them.
#[test]
fn partial_provision_prefixes_are_monotone() {
    // The chain's declared order is the acquisition order, so `ALL` must be strictly ascending.
    for pair in ProvisionDepth::ALL.windows(2) {
        let [earlier, later] = [pair[0], pair[1]];
        assert!(
            earlier < later,
            "{earlier:?} must precede {later:?} in the acquisition chain"
        );
        assert_eq!(
            later.established_facts(),
            earlier.established_facts() + 1,
            "{later:?} must add exactly one fact to {earlier:?}"
        );

        // No fact is ever lost by going deeper.
        assert!(
            !earlier.has_distribution() || later.has_distribution(),
            "{later:?} must retain the distribution {earlier:?} established"
        );
        assert!(
            !earlier.has_staged_module() || later.has_staged_module(),
            "{later:?} must retain the staged module {earlier:?} established"
        );
        assert!(
            !earlier.has_server() || later.has_server(),
            "{later:?} must retain the server {earlier:?} established"
        );
    }

    // The fact count is exactly how many of the three individual facts are present, so the count and
    // the flags cannot disagree about the same depth.
    for depth in ProvisionDepth::ALL {
        let flags = [
            depth.has_distribution(),
            depth.has_staged_module(),
            depth.has_server(),
        ];
        assert_eq!(
            depth.established_facts(),
            flags.iter().filter(|present| **present).count(),
            "{depth:?} must count exactly the facts it reports holding"
        );
        // Prefix-shaped: a later fact never appears without every earlier one.
        assert!(
            !depth.has_server() || depth.has_staged_module(),
            "{depth:?} cannot hold a server without a staged module"
        );
        assert!(
            !depth.has_staged_module() || depth.has_distribution(),
            "{depth:?} cannot hold a staged module without a resolved distribution"
        );
    }

    assert_eq!(
        ProvisionDepth::NothingResolved.established_facts(),
        0,
        "the empty prefix establishes nothing"
    );
    assert_eq!(
        ProvisionDepth::ServerStarted.established_facts(),
        3,
        "the deepest pre-publication prefix establishes all three facts"
    );
}
