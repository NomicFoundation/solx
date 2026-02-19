//!
//! The MLIR-to-LLVM translation integration tests.
//!

use solx_mlir::Context;

#[test]
fn default() {
    // LLVM-dialect MLIR that stores 42 at heap offset 0 and calls EVM RETURN(0, 32).
    const MLIR_SOURCE: &str = r#"
    module {
      llvm.func @llvm.evm.return(!llvm.ptr<1>, i256)

      llvm.func @__entry() {
        %c42 = llvm.mlir.constant(42 : i256) : i256
        %c0 = llvm.mlir.constant(0 : i256) : i256
        %ptr = llvm.inttoptr %c0 : i256 to !llvm.ptr<1>
        llvm.store %c42, %ptr : i256, !llvm.ptr<1>
        %c32 = llvm.mlir.constant(32 : i256) : i256
        llvm.call @llvm.evm.return(%ptr, %c32) : (!llvm.ptr<1>, i256) -> ()
        llvm.unreachable
      }
    }
    "#;

    let llvm_module = Context::new()
        .try_into_llvm_module_from_source(MLIR_SOURCE)
        .expect("MLIR to LLVM translation failed");
    assert!(
        !llvm_module.as_raw().is_null(),
        "LLVM module pointer must not be null"
    );
}
