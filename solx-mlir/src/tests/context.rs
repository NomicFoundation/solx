//! Tests for [`crate::Context`].

use melior::ir::operation::OperationLike;

use crate::Context;

#[test]
fn new_empty_module_verifies() {
    let context = Context::new();
    let module = melior::ir::Module::parse(context.mlir(), "module {}").expect("MLIR should parse");
    assert!(module.as_operation().verify());
}

#[test]
fn run_sol_passes_empty_contract_succeeds() {
    let context = Context::new();
    let mlir_text = r#"
module {
  sol.contract @Test {
  } {kind = #sol<ContractKind Contract>}
}
"#;
    let mut module = melior::ir::Module::parse(context.mlir(), mlir_text).expect("parse failed");
    assert!(
        module.as_operation().verify(),
        "pre-pass verification failed"
    );
    Context::run_sol_passes(context.mlir(), &mut module).expect("sol passes should succeed");
}

#[test]
fn run_sol_passes_with_function_succeeds() {
    let context = Context::new();
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
    let mut module = melior::ir::Module::parse(context.mlir(), mlir_text).expect("parse failed");
    assert!(
        module.as_operation().verify(),
        "pre-pass verification failed"
    );
    Context::run_sol_passes(context.mlir(), &mut module).expect("sol passes should succeed");
}
