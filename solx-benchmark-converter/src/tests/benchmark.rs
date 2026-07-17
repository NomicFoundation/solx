//!
//! Tests for the benchmark representation.
//!

use crate::benchmark::Benchmark;
use crate::benchmark::test::Test;
use crate::benchmark::test::input::Input;
use crate::benchmark::test::metadata::Metadata;
use crate::benchmark::test::selector::Selector;

impl Benchmark {
    fn insert_test(&mut self, project: &str, contract: &str, function: &str) {
        let selector = Selector {
            project: project.to_owned(),
            case: Some(contract.to_owned()),
            input: Some(Input::Runtime {
                input_index: 0,
                name: function.to_owned(),
            }),
        };
        self.tests.insert(
            selector.to_string(),
            Test::new(Metadata::new(selector, vec![])),
        );
    }
}

#[test]
fn remove_blacklisted_drops_only_listed_rows() {
    let mut benchmark = Benchmark::default();
    benchmark.insert_test(
        "solady",
        "src/accounts/ERC6551Proxy.sol:ERC6551Proxy",
        "fallback()",
    );
    benchmark.insert_test(
        "solady",
        "src/accounts/ERC6551Proxy.sol:ERC6551Proxy",
        "someFunction()",
    );
    benchmark.insert_test(
        "other-project",
        "src/accounts/ERC6551Proxy.sol:ERC6551Proxy",
        "fallback()",
    );

    benchmark.remove_blacklisted();

    assert_eq!(benchmark.tests.len(), 2);
    for test in benchmark.tests.values() {
        let selector = &test.metadata.selector;
        assert!(
            selector.project != "solady"
                || selector
                    .input
                    .as_ref()
                    .and_then(|input| input.runtime_name())
                    != Some("fallback()")
        );
    }
}
