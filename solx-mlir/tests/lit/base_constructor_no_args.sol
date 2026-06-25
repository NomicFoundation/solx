// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// A base with a no-argument constructor still gets a SEPARATE internal `sol.func` and an explicit
// (empty-argument) `sol.call` from the most-derived constructor — solc never inlines the base body,
// even when there is nothing to evaluate. The most-derived constructor symbol diverges, matched by
// `kind = #Constructor`; the base constructor is a plain internal func (no `kind`).

// The most-derived constructor: a bare `sol.call` (no operands) into the base constructor.
// CHECK: sol.func @{{.*}} attributes {{.*}}kind = #Constructor
// CHECK:   sol.call @[[BASECTOR:.*]]() : () -> ()
// CHECK:   sol.return

// The base constructor: a SEPARATE plain internal func running the base body `b = 5`.
// CHECK: sol.func @[[BASECTOR]]() attributes {{.*}}state_mutability = #NonPayable
// CHECK-NOT: kind = #Constructor
// CHECK:   sol.constant 5 : ui8
// CHECK:   sol.store {{.*}}!sol.ptr<ui256, Storage>
// CHECK:   sol.return

contract Base {
    uint256 b;
    constructor() {
        b = 5;
    }
}

contract Derived is Base {
    constructor() {}
}
