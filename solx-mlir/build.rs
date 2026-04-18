//!
//! The build script for `solx-mlir`.
//!

fn main() {
    println!("cargo:rerun-if-env-changed=LLVM_SYS_211_PREFIX");

    let prefix = std::env::var("LLVM_SYS_211_PREFIX")
        .expect("LLVM_SYS_211_PREFIX must be set — point it to the solx-llvm build output");

    let lib_path = std::path::PathBuf::from(&prefix).join("lib");
    println!("cargo:rustc-link-search=native={}", lib_path.display());

    // LLD C API — provides LLVMAssembleEVM used by inkwell's assemble_evm.
    // LLVM libs are already linked by mlir-sys; only the LLD linker libs are missing.
    println!("cargo:rustc-link-lib=static=lldC");
    println!("cargo:rustc-link-lib=static=lldCommon");
    println!("cargo:rustc-link-lib=static=lldELF");

    // Sol dialect — custom Solidity MLIR dialect defined in solx-llvm.
    println!("cargo:rustc-link-lib=static=MLIRSolDialect");
    println!("cargo:rustc-link-lib=static=MLIRCAPISol");
    println!("cargo:rustc-link-lib=static=MLIRSolToStandard");
    println!("cargo:rustc-link-lib=static=MLIRSolTransforms");

    // Yul dialect — dependency of the Sol-to-standard conversion pass.
    println!("cargo:rustc-link-lib=static=MLIRYulDialect");
    println!("cargo:rustc-link-lib=static=MLIRCAPIYul");

    let include_path = std::path::PathBuf::from(&prefix).join("include");

    // Track Sol/Yul dialect .td files so that Cargo re-expands the
    // `melior::dialect!` macros in `src/ods.rs` when any definition changes.
    for td_file in &[
        "mlir/Dialect/Sol/SolOps.td",
        "mlir/Dialect/Sol/SolBase.td",
        "mlir/Dialect/Sol/SolInterfaces.td",
        "mlir/Dialect/Yul/YulOps.td",
        "mlir/Dialect/Yul/YulBase.td",
    ] {
        println!(
            "cargo:rerun-if-changed={}",
            include_path.join(td_file).display()
        );
    }

    // Compile stub definitions for the six MLIR ExecutionEngine C API symbols
    // that melior references unconditionally. See mlir_execution_engine_stubs.c
    // for the full explanation.
    println!("cargo:rerun-if-changed=mlir_execution_engine_stubs.c");
    cc::Build::new()
        .file("mlir_execution_engine_stubs.c")
        .compile("mlir_execution_engine_stubs");

    // Compile C++ wrappers for Sol dialect attribute creation.
    // The Sol C API does not expose ContractKindAttr/StateMutabilityAttr
    // constructors, so we provide thin extern "C" wrappers.
    println!("cargo:rerun-if-changed=sol_attr_stubs.cpp");
    cc::Build::new()
        .cpp(true)
        .file("sol_attr_stubs.cpp")
        .include(&include_path)
        .flag("-std=c++17")
        .compile("sol_attr_stubs");
}
