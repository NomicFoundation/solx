// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// A derived contract with NO constructor of its own, inheriting a base that has one. solc (and now
// solx) still synthesises a most-derived `constructor()` (`kind = #Constructor`) that runs the
// hierarchy's state-variable initializers (`d = 3`) and `sol.call`s the base constructor, which lives
// in its OWN separate plain internal `sol.func` — the base body is never inlined. The synthesised
// constructor symbol diverges between backends (regex); the base constructor's `sol.constant 5` vs.
// `sol.addr_of` order differs, pinned with CHECK-DAG.

// The synthesised most-derived constructor: run `d = 3`, then chain into the base constructor.
// CHECK: sol.func @{{.*}} attributes {{.*}}kind = #Constructor
// CHECK:   sol.constant 3 : ui8
// CHECK:   sol.store {{.*}}!sol.ptr<ui256, Storage>
// CHECK:   sol.call @[[BASECTOR:.*]]() : () -> ()
// CHECK:   sol.return

// The base constructor: a SEPARATE plain internal func running the base body `b = 5`.
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
