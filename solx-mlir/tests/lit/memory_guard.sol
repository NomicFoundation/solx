// RUN: solx --emit-mlir=llvm %s | FileCheck %s

// The free memory pointer's initializer stays symbolic through the pipeline;
// the EVM backend folds it to the sum of the two region module flags.

// CHECK: llvm.func @__entry()
// CHECK: "llvm.intrcall"() <{id = {{[0-9]+}} : i32, name = "evm.memoryguard"}> : () -> i256
// CHECK: llvm.module_flags [{{.*}}"evm-memory-guard", 128 : i64>, {{.*}}"evm-stack-region-size", 0 : i64>]

contract C {
  function f(uint256 a) public pure returns (uint256) {
    return a + 1;
  }
}
