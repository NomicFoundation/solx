// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// A 3-level constructor chain A <- B <- C, with a base-state-variable initializer (`bb = 7`) and a
// base constructor that has a body (`bb = vb`). solc (and now solx) emits one `sol.func` per
// constructor in the C3 linearisation and wires them with `sol.call`:
//   * @C (most-derived, kind = #Constructor): runs the WHOLE hierarchy's state-variable initializers
//     first, then calls B's constructor with the evaluated `vc * 2`.
//   * @B (plain internal func, no kind): calls A's constructor with `vb + 10`, THEN runs B's body.
//   * @A (plain internal func, no kind): runs A's body.
// Each base constructor's call to its own base precedes its body, exactly as solc orders them. Symbols
// diverge between backends (regex); the constant-vs-load order is pinned with CHECK-DAG.

// Most-derived @C: state-var init first, then the chained call into B's constructor.
// CHECK: sol.func @{{.*}} attributes {{.*}}kind = #Constructor
// CHECK:   sol.constant 7 : ui8
// CHECK:   sol.store {{.*}}!sol.ptr<ui256, Storage>
// CHECK:   sol.store %arg0
// CHECK-DAG:   sol.constant 2 : ui8
// CHECK-DAG:   sol.load %{{.*}} : !sol.ptr<ui256, Stack>, ui256
// CHECK:   sol.cmul
// CHECK:   sol.call @[[B_CTOR:.*]](%{{.*}}) : (ui256) -> ()
// CHECK:   sol.return

// @B: a separate plain func that calls A's constructor BEFORE running its own body.
// CHECK: sol.func @[[B_CTOR]](%arg0: ui256) attributes {{.*}}state_mutability = #NonPayable
// CHECK-NOT: kind = #Constructor
// CHECK:   sol.store %arg0
// CHECK-DAG:   sol.constant 10 : ui8
// CHECK-DAG:   sol.load %{{.*}} : !sol.ptr<ui256, Stack>, ui256
// CHECK:   sol.cadd
// CHECK:   sol.call @[[A_CTOR:.*]](%{{.*}}) : (ui256) -> ()
// CHECK:   sol.store {{.*}}!sol.ptr<ui256, Storage>
// CHECK:   sol.return

// @A: the leaf constructor — no further call.
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
