// RUN: solx --emit-mlir=sol %s | FileCheck %s --check-prefixes=CHECK,CHECK-SOLX
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s --check-prefixes=CHECK,CHECK-SOLC

// Base-constructor argument `Library.value()` in an inheritance specifier: solx materializes the value()
// call in @Derived's constructor and threads it to the base constructor as (ui256); solc's print-init
// emits the base-constructor call with no argument (() -> ()), deferring the wiring. Both still
// copy the value helper (constant 7) into the concrete contract.

// CHECK: sol.contract @{{.*Derived.*}}
// CHECK: sol.func @{{.*}} attributes {kind = #Constructor
// CHECK-SOLX: sol.call @{{.*value.*}}() : () -> ui256
// CHECK-SOLX: sol.call @{{.*constructor.*}}(%{{.*}}) : (ui256) -> ()
// CHECK-SOLC: sol.call @{{.*}}() : () -> ()
// CHECK: sol.return
// CHECK: sol.func @{{.*value.*}}() -> ui256
// CHECK: sol.constant 7 : ui8

contract Library {
    function value() internal pure returns (uint256) {
        return 7;
    }
}

contract Base {
    uint256 public x;

    constructor(uint256 v) {
        x = v;
    }
}

contract Derived is Library, Base(Library.value()) {
}
