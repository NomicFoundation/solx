// RUN: solx --emit-mlir=sol %s | FileCheck %s --check-prefixes=CHECK,CHECK-SOLX
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s --check-prefixes=CHECK,CHECK-SOLC

// Base-ctor arg `Lib.val()` in an inheritance specifier: solx materializes the val()
// call in @Derived's ctor and threads it to the base ctor as (ui256); solc's print-init
// emits the base-ctor call with no argument (() -> ()), deferring the wiring. Both still
// copy the val helper (constant 7) into the concrete contract.

// CHECK: sol.contract @{{.*Derived.*}}
// CHECK: sol.func @{{.*}} attributes {kind = #Constructor
// CHECK-SOLX: sol.call @{{.*val.*}}() : () -> ui256
// CHECK-SOLX: sol.call @{{.*constructor.*}}(%{{.*}}) : (ui256) -> ()
// CHECK-SOLC: sol.call @{{.*}}() : () -> ()
// CHECK: sol.return
// CHECK: sol.func @{{.*val.*}}() -> ui256
// CHECK: sol.constant 7 : ui8

contract Lib {
    function val() internal pure returns (uint256) {
        return 7;
    }
}

contract Base {
    uint256 public x;
    constructor(uint256 v) {
        x = v;
    }
}

contract Derived is Lib, Base(Lib.val()) {
}
