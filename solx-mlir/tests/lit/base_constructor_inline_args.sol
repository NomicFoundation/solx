// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*}} attributes {{.*}}kind = #Constructor
// CHECK:   sol.constant 41 : ui8
// CHECK:   sol.call @[[BASECTOR:.*]](%{{.*}}) : (ui8) -> ()
// CHECK:   sol.return

// CHECK: sol.func @[[BASECTOR]](%arg0: ui256) attributes {{.*}}state_mutability = #NonPayable
// CHECK-NOT: kind = #Constructor
// CHECK:   sol.store %arg0
// CHECK:   sol.store {{.*}}!sol.ptr<ui256, Storage>
// CHECK:   sol.return

contract Base {
    uint256 b;
    constructor(uint256 v) {
        b = v;
    }
}

contract Derived is Base(41) {
    constructor() {}
}
