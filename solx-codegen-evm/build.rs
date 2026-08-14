//!
//! The build script for `solx-codegen-evm`.
//!

fn main() {
    println!("cargo:rerun-if-env-changed=LLVM_SYS_211_PREFIX");

    let prefix = std::env::var("LLVM_SYS_211_PREFIX")
        .expect("LLVM_SYS_211_PREFIX must be set — point it to the solx-llvm build output");

    let lib_path = std::path::PathBuf::from(&prefix).join("lib");
    println!("cargo:rustc-link-search=native={}", lib_path.display());

    // LLD C API — LLVMAssembleEVM/LLVMLinkEVM and friends, referenced by
    // inkwell's memory_buffer. The directives must come from this crate
    // (the one that owns the inkwell dependency): strict single-pass
    // linkers (bfd on aarch64-linux and mingw) resolve archives in
    // command-line order, and directives emitted by an unrelated crate
    // (solx-mlir) can land before the inkwell rlib that needs them.
    println!("cargo:rustc-link-lib=static=lldC");
    println!("cargo:rustc-link-lib=static=lldCommon");
    println!("cargo:rustc-link-lib=static=lldELF");
}
