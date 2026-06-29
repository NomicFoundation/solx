// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*}} attributes {{.*}}kind = #Constructor
// CHECK:   sol.constant 7 : ui8
// CHECK:   sol.store {{.*}}!sol.ptr<ui256, Storage>
// CHECK:   sol.store %arg0
// CHECK:   sol.constant 2 : ui8
// CHECK:   sol.cmul
// CHECK:   sol.call @[[B_CTOR:.*]](%{{.*}}) : (ui256) -> ()
// CHECK:   sol.return

// CHECK: sol.func @[[B_CTOR]](%arg0: ui256) attributes {{.*}}state_mutability = #NonPayable
// CHECK-NOT: kind = #Constructor
// CHECK:   sol.store %arg0
// CHECK:   sol.constant 10 : ui8
// CHECK:   sol.cadd
// CHECK:   sol.call @[[A_CTOR:.*]](%{{.*}}) : (ui256) -> ()
// CHECK:   sol.store {{.*}}!sol.ptr<ui256, Storage>
// CHECK:   sol.return

// CHECK: sol.func @[[A_CTOR]](%arg0: ui256) attributes {{.*}}state_mutability = #NonPayable
// CHECK-NOT: sol.call
// CHECK:   sol.store {{.*}}!sol.ptr<ui256, Storage>
// CHECK:   sol.return

contract A {
    uint256 a;

    constructor(uint256 va) {
        a = va;
    }
}

contract B is A {
    uint256 bb = 7;

    constructor(uint256 vb) A(vb + 10) {
        bb = vb;
    }
}

contract C is B {
    constructor(uint256 vc) B(vc * 2) {}
}
