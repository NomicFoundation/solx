//!
//! Tests for the benchmark converter.
//!

#![cfg(test)]

#[test]
fn convert() {
    let foundry_report_1 = r#"
    { "toolchain": "solx", "project": "ProjectX", "data": [ {
        "contract": "src/test/utils/mocks/MockAuthority.sol:MockAuthority",
        "deployment": { "gas": 111281, "size": 406 },
        "functions": {
            "canCall(address,address,bytes4)": {
                "calls": 5654,
                "min": 462,
                "mean": 462,
                "median": 462,
                "max": 462
            },
            "allowance(address,address)": {
                "calls": 1801,
                "min": 753,
                "mean": 753,
                "median": 753,
                "max": 753
            }
        }
    }, {
        "contract": "src/test/utils/mocks/MockAuthorityHarder.sol:MockAuthorityHarder",
        "deployment": { "gas": 111281, "size": 406 },
        "functions": {
            "canCall(address,address)": {
                "calls": 5654,
                "min": 462,
                "mean": 462,
                "median": 462,
                "max": 462
            },
            "allowance(address,address)": {
                "calls": 1801,
                "min": 753,
                "mean": 753,
                "median": 753,
                "max": 753
            }
        }
    } ] }"#;
    let input_1 = serde_json::from_str::<solx_benchmark_converter::Input>(foundry_report_1)
        .expect("Failed to parse foundry report");

    let foundry_report_2 = r#"
    { "toolchain": "solx", "project": "ProjectY", "data": [ {
        "contract": "src/test/StrangeFile.sol:StrangeContract",
        "deployment": { "gas": 99999, "size": 1111 },
        "functions": {
            "anotherCall": {
                "calls": 15,
                "mean": 9000
            },
            "yetAnotherCall": {
                "calls": 5,
                "mean": 999
            }
        }
    }, {
        "contract": "src/test/OrdinaryFile.sol:OrdinaryContract",
        "deployment": { "gas": 99999, "size": 1111 },
        "functions": {
            "ordinaryCall": {
                "calls": 15,
                "mean": 9000
            },
            "evenMoreOrdinaryCall": {
                "calls": 5,
                "mean": 999
            }
        }
    } ] }"#;
    let input_2 = serde_json::from_str::<solx_benchmark_converter::Input>(foundry_report_2)
        .expect("Failed to parse foundry report");

    let mut benchmark = solx_benchmark_converter::Benchmark::default();
    benchmark
        .extend(input_1)
        .expect("Failed to extend a benchmark report with a Foundry report");
    benchmark
        .extend(input_2)
        .expect("Failed to extend a benchmark report with a Foundry report");

    let output: solx_benchmark_converter::Output = (
        benchmark,
        solx_benchmark_converter::InputSource::Tooling,
        solx_benchmark_converter::OutputFormat::Json,
    )
        .try_into()
        .expect("Failed to convert a benchmark report to output");
    let contents = match output {
        solx_benchmark_converter::Output::SingleFile(file) => file,
        solx_benchmark_converter::Output::MultipleFiles(mut files) => files.remove(0).content,
        _ => unimplemented!(),
    };

    eprintln!("Contents: {contents}");
    assert!(contents.contains("ProjectX::src/test/utils/mocks/MockAuthority.sol:MockAuthority[#deployer:src/test/utils/mocks/MockAuthority.sol:MockAuthority]"));
    assert!(contents.contains("ProjectX::src/test/utils/mocks/MockAuthorityHarder.sol:MockAuthorityHarder[#deployer:src/test/utils/mocks/MockAuthorityHarder.sol:MockAuthorityHarder]"));
    assert!(contents.contains("ProjectX::src/test/utils/mocks/MockAuthority.sol:MockAuthority[allowance(address,address):1]"));
    assert!(contents.contains("ProjectX::src/test/utils/mocks/MockAuthority.sol:MockAuthority[canCall(address,address,bytes4):2]"));
    assert!(contents.contains("ProjectX::src/test/utils/mocks/MockAuthorityHarder.sol:MockAuthorityHarder[allowance(address,address):1]"));
    assert!(contents.contains("ProjectX::src/test/utils/mocks/MockAuthorityHarder.sol:MockAuthorityHarder[canCall(address,address):2]"));

    assert!(contents
        .contains("ProjectY::src/test/StrangeFile.sol:StrangeContract[#deployer:src/test/StrangeFile.sol:StrangeContract]"));
    assert!(contents
        .contains("ProjectY::src/test/OrdinaryFile.sol:OrdinaryContract[#deployer:src/test/OrdinaryFile.sol:OrdinaryContract]"));
    assert!(contents.contains("ProjectY::src/test/StrangeFile.sol:StrangeContract[anotherCall:1]"));
    assert!(
        contents.contains("ProjectY::src/test/StrangeFile.sol:StrangeContract[yetAnotherCall:2]")
    );
    assert!(contents
        .contains("ProjectY::src/test/OrdinaryFile.sol:OrdinaryContract[evenMoreOrdinaryCall:1]"));
    assert!(
        contents.contains("ProjectY::src/test/OrdinaryFile.sol:OrdinaryContract[ordinaryCall:2]")
    );
}
