// RUN: solx --emit-mlir=llvm %s | FileCheck %s

// The free memory pointer's initializer stays symbolic through the pipeline;
// the EVM backend folds it to `guard + spill region size`.

// CHECK: llvm.func @__entry()
// CHECK: "llvm.intrcall"({{.*}}) <{id = {{[0-9]+}} : i32, name = "evm.memoryguard"}> : (i256) -> i256

contract C {
  function f(uint256 a) public pure returns (uint256) {
    return a + 1;
  }
}
