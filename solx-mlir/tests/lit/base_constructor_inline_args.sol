// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// A derived contract supplies its base constructor's argument via an inline `is Base(args)` on the
// header (a compile-time-constant argument, as the header has no constructor parameters in scope).
// solc (and now solx) lowers the base constructor into a *separate* internal `sol.func` and has the
// most-derived `constructor()` evaluate the invocation argument and `sol.call` it — it does NOT inline
// the base body. The most-derived constructor symbol diverges between backends, so it is matched by
// `kind = #Constructor`; the base constructor is a plain internal func (no `kind`), matched by its
// `state_mutability` and body.

// The most-derived constructor: materialize the literal argument `41`, then call the base constructor.
// CHECK: sol.func @{{.*}} attributes {{.*}}kind = #Constructor
// CHECK:   sol.constant 41 : ui8
// CHECK:   sol.call @[[BASECTOR:.*]](%{{.*}}) : (ui8) -> ()
// CHECK:   sol.return

// The base constructor: a SEPARATE plain internal func (no `kind`), running the base body `b = v`.
// CHECK: sol.func @[[BASECTOR]](%arg0: ui256) attributes {{.*}}state_mutability = #NonPayable
// CHECK-NOT: kind = #Constructor
// CHECK:   sol.store %arg0
// CHECK-DAG:   sol.addr_of @{{.*}} : !sol.ptr<ui256, Storage>
// CHECK-DAG:   sol.load %{{.*}} : !sol.ptr<ui256, Stack>, ui256
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
