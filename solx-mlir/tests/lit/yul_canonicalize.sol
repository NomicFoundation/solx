// RUN: slang --emit-mlir=yul %s | FileCheck %s

// CHECK-LABEL: yul.func @"lt(uint256,uint256)
// CHECK-NOT:     yul.cmp
// CHECK:         yul.cmp uge
// CHECK-NOT:     yul.cmp
// CHECK:         yul.func_return

contract C {
  function lt(uint256 a, uint256 b) public pure {
    require(a < b);
  }
}
