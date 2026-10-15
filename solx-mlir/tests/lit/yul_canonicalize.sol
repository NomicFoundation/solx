// RUN: slang --emit-mlir=llvm %s | FileCheck %s

// The Yul module is canonicalized before its lowering: `iszero(lt(a, b))`
// reaches the LLVM dialect as one comparison.

// CHECK-LABEL: llvm.func private @"ge(uint256,uint256)
// CHECK-NOT: llvm.icmp
// CHECK: llvm.icmp "uge" %{{.*}}, %{{.*}} : i256
// CHECK-NOT: llvm.icmp
// CHECK: llvm.return

contract C {
  function ge(uint256 a, uint256 b) public pure returns (bool r) {
    assembly {
      r := iszero(lt(a, b))
    }
  }
}
