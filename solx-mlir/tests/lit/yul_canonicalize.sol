// RUN: slang --emit-mlir=yul %s | FileCheck %s

// The Yul module is canonicalized before its translation: `iszero(lt(a, b))`
// reaches the LLVM IR translation as one comparison.

// CHECK-LABEL: yul.func @"ge(uint256,uint256)
// CHECK-NOT: yul.cmp
// CHECK: yul.cmp uge, %{{.*}}, %{{.*}}
// CHECK-NOT: yul.cmp
// CHECK: yul.func_return

contract C {
  function ge(uint256 a, uint256 b) public pure returns (bool r) {
    assembly {
      r := iszero(lt(a, b))
    }
  }
}
