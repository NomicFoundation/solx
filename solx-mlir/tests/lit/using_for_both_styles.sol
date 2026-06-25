// RUN: solx --emit-mlir=sol %s | FileCheck %s --check-prefix=CHECK-SOLX
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s --check-prefix=CHECK-SOLC

// One library internal function `inc` invoked two ways from the same contract:
//   * `L.inc(x)`  (explicit, `viaDirect`) -> PARITY: both emit
//                 `sol.call @inc(%x) : (ui256) -> ui256`.
//   * `x.inc()`   (method-style via `using L for uint256`, `viaMethod`) ->
//                 DIVERGES: solx forwards the receiver `sol.call @inc(%x)`,
//                 solc's print-init drops it `sol.call @inc() : () -> ui256`.
// Both backends inline `inc` once and share it between both call sites.
// Split prefixes are required because (a) the method-style arity diverges and
// (b) solx walks functions alphabetically (viaDirect, viaMethod) while solc
// keeps source order (viaMethod, viaDirect).

// CHECK-SOLX: sol.func @{{.*viaDirect.*}}(%{{.*}}: ui256) -> ui256
// CHECK-SOLX:   %[[X:.*]] = sol.load %{{.*}} : !sol.ptr<ui256, Stack>, ui256
// CHECK-SOLX:   sol.call @{{.*inc.*}}(%[[X]]) : (ui256) -> ui256
// CHECK-SOLX: sol.func @{{.*viaMethod.*}}(%{{.*}}: ui256) -> ui256
// CHECK-SOLX:   %[[Y:.*]] = sol.load %{{.*}} : !sol.ptr<ui256, Stack>, ui256
// CHECK-SOLX:   sol.call @{{.*inc.*}}(%[[Y]]) : (ui256) -> ui256

// CHECK-SOLC: sol.func @{{.*viaMethod.*}}(%{{.*}}: ui256) -> ui256
// CHECK-SOLC:   sol.call @{{.*inc.*}}() : () -> ui256
// CHECK-SOLC: sol.func @{{.*viaDirect.*}}(%{{.*}}: ui256) -> ui256
// CHECK-SOLC:   %[[X:.*]] = sol.load %{{.*}} : !sol.ptr<ui256, Stack>, ui256
// CHECK-SOLC:   sol.call @{{.*inc.*}}(%[[X]]) : (ui256) -> ui256

library L {
    function inc(uint256 a) internal pure returns (uint256) {
        return a + 1;
    }
}

contract C {
    using L for uint256;

    function viaMethod(uint256 x) public pure returns (uint256) {
        return x.inc();
    }

    function viaDirect(uint256 x) public pure returns (uint256) {
        return L.inc(x);
    }
}
