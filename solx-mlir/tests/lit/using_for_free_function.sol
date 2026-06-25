// RUN: solx --emit-mlir=sol %s | FileCheck %s --check-prefix=CHECK-SOLX
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s --check-prefix=CHECK-SOLC

// A free (file-level) function attached with `using {dbl} for uint256` and
// invoked method-style `x.dbl()`. Like the library case, both backends inline
// the free function into the contract and dispatch via `sol.call`, but the
// receiver argument diverges: solx forwards it `sol.call @dbl(%x) : (ui256)`,
// solc's print-init drops it `sol.call @dbl() : () -> ui256`. The inlined `dbl`
// body (cmul by 2) is identical on both sides. Split prefixes pin each
// backend's real call arity.

// CHECK-SOLX: sol.func @{{.*f.*}}(%{{.*}}: ui256) -> ui256
// CHECK-SOLX:   %[[X:.*]] = sol.load %{{.*}} : !sol.ptr<ui256, Stack>, ui256
// CHECK-SOLX:   sol.call @{{.*dbl.*}}(%[[X]]) : (ui256) -> ui256
// CHECK-SOLX: sol.func @{{.*dbl.*}}(%{{.*}}: ui256) -> ui256
// CHECK-SOLX:   sol.cmul

// CHECK-SOLC: sol.func @{{.*dbl.*}}(%{{.*}}: ui256) -> ui256
// CHECK-SOLC:   sol.cmul
// CHECK-SOLC: sol.func @{{.*f.*}}(%{{.*}}: ui256) -> ui256
// CHECK-SOLC:   sol.call @{{.*dbl.*}}() : () -> ui256

function dbl(uint256 a) pure returns (uint256) {
    return a * 2;
}

contract C {
    using {dbl} for uint256;

    function f(uint256 x) public pure returns (uint256) {
        return x.dbl();
    }
}
