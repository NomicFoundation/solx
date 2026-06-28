// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*LIMIT.*}}() -> ui256 attributes {{.*}}selector = -1350429457 : i32{{.*}}
// CHECK:   %[[C:.*]] = sol.constant 42 : ui8
// CHECK:   %[[V:.*]] = sol.cast %[[C]] : ui8 to ui256
// CHECK:   sol.return %[[V]] : ui256

contract C {
    uint256 public constant LIMIT = 42;
}
