// RUN: solx --emit-mlir=sol %s | FileCheck %s --check-prefix=CHECK-SOLX
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s --check-prefix=CHECK-SOLC

// `using L for uint256` with a method-style call `x.add(3)`. The receiver `x`
// is the first argument of the library function. Both backends inline `add`
// into the contract and dispatch via `sol.call`, but they DIVERGE on the
// receiver argument: solx passes the receiver (call type `(ui256, ui256)`),
// while solc's print-init drops the receiver entirely, lowering to a
// `(ui256) -> ui256` call that forwards only the explicit `3`. Split prefixes
// pin each backend's real call arity; the inlined `add` body is identical.

// CHECK-SOLX: sol.func @{{.*f.*}}(%{{.*}}: ui256) -> ui256
// CHECK-SOLX:   %[[X:.*]] = sol.load %{{.*}} : !sol.ptr<ui256, Stack>, ui256
// CHECK-SOLX:   %[[THREE:.*]] = sol.cast %{{.*}} : ui8 to ui256
// CHECK-SOLX:   sol.call @{{.*add.*}}(%[[X]], %[[THREE]]) : (ui256, ui256) -> ui256

// CHECK-SOLC: sol.func @{{.*f.*}}(%{{.*}}: ui256) -> ui256
// CHECK-SOLC:   %[[THREE:.*]] = sol.cast %{{.*}} : ui8 to ui256
// CHECK-SOLC:   sol.call @{{.*add.*}}(%[[THREE]]) : (ui256) -> ui256

library L {
    function add(uint256 a, uint256 b) internal pure returns (uint256) {
        return a + b;
    }
}

contract C {
    using L for uint256;

    function f(uint256 x) public pure returns (uint256) {
        return x.add(3);
    }
}
