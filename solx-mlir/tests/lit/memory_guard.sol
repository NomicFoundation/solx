// RUN: slang --emit-mlir=yul %s | FileCheck %s

// CHECK-LABEL: yul.object @{{.*:Reached"}}
// CHECK:         yul.memoryguard 128
// CHECK-LABEL: yul.object @{{.*:Reached_deployed"}}
// CHECK-NOT:     yul.memoryguard
// CHECK-LABEL: yul.object @{{.*:Unreached"}}
// CHECK:         yul.memoryguard 128
// CHECK-LABEL: yul.object @{{.*:Unreached_deployed"}}
// CHECK:         yul.memoryguard 128

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
