// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// Inline-assembly comparisons lower to `yul.cmp` with the matching predicate;
// `iszero(x)` is `yul.cmp eq, x, 0`.

// CHECK: sol.func @{{.*compare.*}}
// CHECK: yul.cmp ult
// CHECK: yul.cmp ugt
// CHECK: yul.cmp eq
// CHECK: yul.cmp slt
// CHECK: yul.cmp sgt
// CHECK: yul.cmp eq

contract C {
    function compare(uint256 a, uint256 b) public pure returns (uint256 r) {
        assembly {
            r := lt(a, b)
            r := gt(a, b)
            r := eq(a, b)
            r := slt(a, b)
            r := sgt(a, b)
            r := iszero(a)
        }
    }
}
