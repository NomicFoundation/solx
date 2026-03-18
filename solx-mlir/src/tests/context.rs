//! Tests for [`crate::Context`].

use melior::ir::operation::OperationLike;

use crate::Context;

#[test]
fn new_empty_module_verifies() {
    let context = Context::create_mlir_context();
    let module = melior::ir::Module::parse(&context, "module {}").expect("MLIR should parse");
    assert!(module.as_operation().verify());
}

#[test]
fn run_sol_passes_empty_contract_succeeds() {
    let context = Context::create_mlir_context();
    let mlir_text = r#"
module {
  sol.contract @Test {
  } {kind = #sol<ContractKind Contract>}
}
"#;
    let mut module = melior::ir::Module::parse(&context, mlir_text).expect("parse failed");
    assert!(
        module.as_operation().verify(),
        "pre-pass verification failed"
    );
    Context::run_sol_passes(&context, &mut module).expect("sol passes should succeed");
}

#[test]
fn run_sol_passes_with_function_succeeds() {
    let context = Context::create_mlir_context();
    let mlir_text = r#"
module attributes {sol.evm_version = #sol<EvmVersion Cancun>} {
  sol.contract @Test {
    sol.func @first() -> (i256) attributes {
        selector = 1039457780 : i32,
        orig_fn_type = () -> (i256),
        state_mutability = #sol<StateMutability Pure>
    } {
      %ptr = sol.alloca : !sol.ptr<i256, Stack>
      %c42 = sol.constant 42 : i256
      sol.store %c42, %ptr : i256, !sol.ptr<i256, Stack>
      %v = sol.load %ptr : !sol.ptr<i256, Stack>, i256
      sol.return %v : i256
    }
  } {kind = #sol<ContractKind Contract>}
}
"#;
    let mut module = melior::ir::Module::parse(&context, mlir_text).expect("parse failed");
    assert!(
        module.as_operation().verify(),
        "pre-pass verification failed"
    );
    Context::run_sol_passes(&context, &mut module).expect("sol passes should succeed");
}

#[test]
fn translate_llvm_dialect_to_llvm_module() {
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

    let context = Context::create_mlir_context();
    let llvm_module = Context::translate_source_to_llvm_module(&context, MLIR_SOURCE)
        .expect("MLIR to LLVM translation failed");
    assert!(
        !llvm_module.as_raw().is_null(),
        "LLVM module pointer must not be null"
    );
}
