// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// A derived contract supplies its base constructor's argument through the header-style
// `constructor(...) Base(args)` invocation (the modifier-list spelling), with the argument derived
// from the derived constructor's own parameter. This must lower exactly like the inline `is Base(args)`
// form: a separate base-constructor `sol.func`, with the most-derived `constructor()` evaluating the
// argument against its spilled parameter and `sol.call`ing the base. The most-derived constructor
// diverges by symbol, matched by `kind = #Constructor`; the constant-vs-load order differs, pinned
// with CHECK-DAG.

// The most-derived constructor: spill its param, evaluate `y + 1`, then call the base constructor.
// CHECK: sol.func @{{.*}}(%arg0: ui256) attributes {{.*}}kind = #Constructor
// CHECK:   sol.store %arg0
// CHECK-DAG:   sol.constant 1 : ui8
// CHECK-DAG:   sol.load %{{.*}} : !sol.ptr<ui256, Stack>, ui256
// CHECK:   sol.cadd
// CHECK:   sol.call @[[BASECTOR:.*]](%{{.*}}) : (ui256) -> ()
// CHECK:   sol.return

// The base constructor: a SEPARATE plain internal func (no `kind`).
// CHECK: sol.func @[[BASECTOR]](%arg0: ui256) attributes {{.*}}state_mutability = #NonPayable
// CHECK-NOT: kind = #Constructor
// CHECK:   sol.store {{.*}}!sol.ptr<ui256, Storage>
// CHECK:   sol.return

contract Base {
    uint256 b;
    constructor(uint256 v) {
        b = v;
    }
}

contract Derived is Base {
    constructor(uint256 y) Base(y + 1) {}
}
