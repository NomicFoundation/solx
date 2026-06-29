// RUN: solx --emit-mlir=sol %s | FileCheck %s

// Auto-getter returning a file-level constant: solc's print-init aborts in getLocalVarAddr, assertion (it != localVarAddrMap.end()), so this is solx-only.

// CHECK: sol.func @{{.*B.*}}() -> ui256 attributes {{.*}}selector = 854050239 : i32{{.*}}#Pure
// CHECK:   %[[C:.*]] = sol.constant 7 : ui8
// CHECK:   %[[V:.*]] = sol.cast %[[C]] : ui8 to ui256
// CHECK:   sol.return %[[V]] : ui256

uint256 constant A = 7;

contract C {
    uint256 public constant B = A;
}
