// RUN: solx --emit-mlir=sol %s | FileCheck %s --check-prefix=CHECK-SOLX
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s --check-prefix=CHECK-SOLC

// A base-qualified internal call placed in an inheritance-specifier argument
// (`contract Derived is Lib, Base(Lib.val())`) - a call that lives in no function
// body; the synthesised @Derived constructor evaluates it.
//
// Backends diverge on how base-constructor arguments are threaded at this stage:
// solx materialises the `Lib.val()` call inside the @Derived constructor and passes
// its result to the inherited base constructor, while solc emits the base-constructor
// body and the `val` helper but defers wiring the argument to a later lowering. Both still copy the `val` helper (constant 7) into
// the concrete contract, so the prefixes are split to reflect that divergence.

// CHECK-SOLX: sol.contract @{{.*Derived.*}}
// CHECK-SOLX: sol.func @"constructor()"()
// CHECK-SOLX:   sol.call @{{.*val.*}}() : () -> ui256
// CHECK-SOLX:   sol.call @{{.*constructor.*}}(%{{.*}}) : (ui256) -> ()
// CHECK-SOLX:   sol.return
// CHECK-SOLX: sol.func @{{.*val.*}}() -> ui256
// CHECK-SOLX:   sol.constant 7 : ui8

// CHECK-SOLC: sol.contract @{{.*Derived.*}}
// CHECK-SOLC: sol.func @{{.*val.*}}() -> ui256
// CHECK-SOLC:   sol.constant 7 : ui8

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
