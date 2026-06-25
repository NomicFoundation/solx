// RUN: solx --emit-mlir=sol %s | FileCheck %s

// Two destructuring shapes the recursive `destructure` walk handles:
//   * a NESTED tuple lhs/rhs `((a, b), c) = ((1, 2), 3)`, which recurses into the
//     inner `(a, b) = (1, 2)` and flattens it with the outer `c = 3`;
//   * a BLANK slot `(a, ) = (7, 8)`, where the second lhs item is empty and its
//     RHS `8` is discarded (no store emitted for it).
// solc's nascent MLIR backend crashes on both shapes (an lvalue-count assertion
// / segfault), so these are solx-only checks.

// solx emits the functions alphabetically, so `blank` is checked before
// `nested`.

// `blank` stores only the kept slot `a = 7`; the discarded `8` constant is
// materialised but never stored.
// CHECK: sol.func @{{.*blank.*}}
// CHECK: sol.constant 7 : ui8
// CHECK: sol.constant 8 : ui8
// CHECK: %[[B:.*]] = sol.cast %{{.*}} : ui8 to ui256
// CHECK: sol.store %[[B]], %{{[0-9]+}} : ui256, !sol.ptr<ui256, Stack>

// `nested` flattens to three scalar stores; stores run right-to-left so the
// outer `c = 3` store is emitted first, then the inner pair.
// CHECK: sol.func @{{.*nested.*}}
// CHECK-DAG: %[[V1:.*]] = sol.cast %{{.*}} : ui8 to ui256
// CHECK-DAG: %[[V2:.*]] = sol.cast %{{.*}} : ui8 to ui256
// CHECK-DAG: %[[V3:.*]] = sol.cast %{{.*}} : ui8 to ui256
// CHECK-DAG: sol.store %[[V1]], %{{[0-9]+}} : ui256, !sol.ptr<ui256, Stack>
// CHECK-DAG: sol.store %[[V2]], %{{[0-9]+}} : ui256, !sol.ptr<ui256, Stack>
// CHECK-DAG: sol.store %[[V3]], %{{[0-9]+}} : ui256, !sol.ptr<ui256, Stack>

contract C {
    function nested() public pure returns (uint256, uint256, uint256) {
        uint256 a; uint256 b; uint256 c;
        ((a, b), c) = ((1, 2), 3);
        return (a, b, c);
    }
    function blank() public pure returns (uint256) {
        uint256 a;
        (a, ) = (7, 8);
        return a;
    }
}
