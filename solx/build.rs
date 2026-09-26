//!
//! The build script for native libraries used by `solx`.
//!

///
/// Links the solc and Boost libraries from `SOLC_PREFIX` / `BOOST_PREFIX`.
///
fn main() {
    println!("cargo:rerun-if-env-changed=BOOST_PREFIX");
    if let Ok(path) = std::env::var("BOOST_PREFIX") {
        println!("cargo:rerun-if-changed={path}");
    }

    println!("cargo:rerun-if-env-changed=SOLC_PREFIX");
    if let Ok(path) = std::env::var("SOLC_PREFIX") {
        println!("cargo:rerun-if-changed={path}");
    }

    println!("cargo:rustc-link-search=native={}", env!("BOOST_PREFIX"));
    for directory in [
        "libsolc",
        "libsolidity",
        "libsolutil",
        "liblangutil",
        "libevmasm",
        "libyul",
        "libsmtutil",
    ] {
        println!(
            "cargo:rustc-link-search=native={}/{directory}",
            env!("SOLC_PREFIX"),
        );
    }

    for library in ["boost_filesystem", "boost_system", "boost_program_options"] {
        println!("cargo:rustc-link-lib=static={library}");
    }
    for library in [
        "solc", "solidity", "solutil", "langutil", "evmasm", "yul", "smtutil",
    ] {
        println!("cargo:rustc-link-lib=static={library}");
    }
}
