// RUN: slang --emit-mlir=yul %s | FileCheck %s

// CHECK: yul.object @{{.*:C"}}
// CHECK: yul.func @__entry
// CHECK: yul.memoryguard 128
// CHECK: yul.object @{{.*:Reached_deployed"}}
// CHECK-NOT: yul.memoryguard
// CHECK: yul.object @{{.*:Unreached"}}
// CHECK: yul.object @{{.*:Unreached_deployed"}}
// CHECK: yul.memoryguard 128

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
