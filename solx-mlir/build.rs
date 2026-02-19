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
}
