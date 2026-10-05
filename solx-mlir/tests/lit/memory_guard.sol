// RUN: slang --emit-mlir=llvm %s | FileCheck %s

// CHECK: llvm.func @__entry()
// CHECK: "llvm.intrcall"() <{id = {{[0-9]+}} : i32, name = "evm.memoryguard"}> : () -> i256
// CHECK: llvm.module_flags [{{.*}}"evm-memory-guard", 128 : i64>, {{.*}}"evm-stack-region-size", 0 : i64>]
// CHECK: module @{{.*:Reached_deployed"}}
// CHECK-NOT: "evm-memory-guard"
// CHECK: module @{{.*:Unreached"}}
// CHECK: module @{{.*:Unreached_deployed"}}
// CHECK: "evm-memory-guard"

contract C {
  function f(uint256 a) public pure returns (uint256) {
    return a + 1;
  }
}

contract Reached {
  function f(uint256 x) public pure returns (uint256) {
    return g(x);
  }

  function g(uint256 x) internal pure returns (uint256) {
    assembly { mstore(0, x) }
    return x;
  }
}

contract Unreached {
  function g(uint256 x) internal pure returns (uint256) {
    assembly { mstore(0, x) }
    return x;
  }
}
