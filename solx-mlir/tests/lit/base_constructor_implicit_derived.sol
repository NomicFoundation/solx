// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*}} attributes {{.*}}kind = #Constructor
// CHECK:   sol.constant 3 : ui8
// CHECK:   sol.store {{.*}}!sol.ptr<ui256, Storage>
// CHECK:   sol.call @[[BASECTOR:.*]]() : () -> ()
// CHECK:   sol.return

// CHECK: sol.func @[[BASECTOR]]() attributes {{.*}}state_mutability = #NonPayable
// CHECK-NOT: kind = #Constructor
// CHECK-DAG:   sol.constant 5 : ui8
// CHECK-DAG:   sol.addr_of @{{.*}} : !sol.ptr<ui256, Storage>
// CHECK:   sol.store {{.*}}!sol.ptr<ui256, Storage>
// CHECK:   sol.return

contract Base {
    uint256 b;
    constructor() {
        b = 5;
    }
}

contract Derived is Base {
    uint256 d = 3;
}
