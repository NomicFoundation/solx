// RUN: solx --emit-mlir=sol %s | FileCheck %s --check-prefix=CHECK-SOLX
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s --check-prefix=CHECK-SOLC

// Overload resolution at a call site: `h(h(10), 20)` must resolve the inner
// call to the one-argument overload and the outer call to the two-argument
// overload. Both backends pick the same targets; we pin the resolution via the
// call's lowered signature ((ui256) -> ui256 for the inner, (ui256, ui256) ->
// ui256 for the outer), which a bare `@{{.*h.*}}` regex cannot disambiguate.
//
// Split prefixes are required because solx emits internal functions
// alphabetically (`caller`, then the `h` overloads) while solc emits them in
// source order (the `h` overloads, then `caller`); only the per-backend
// function ordering differs, the call-site resolution is identical.

// CHECK-SOLX: sol.func @{{.*caller.*}}
// CHECK-SOLX:   %[[A:.*]] = sol.call @{{.*h.*}}(%{{.*}}) : (ui256) -> ui256
// CHECK-SOLX:   %{{.*}} = sol.call @{{.*h.*}}(%[[A]], %{{.*}}) : (ui256, ui256) -> ui256

// CHECK-SOLC: sol.func @{{.*caller.*}}
// CHECK-SOLC:   %[[A:.*]] = sol.call @{{.*h.*}}(%{{.*}}) : (ui256) -> ui256
// CHECK-SOLC:   %{{.*}} = sol.call @{{.*h.*}}(%[[A]], %{{.*}}) : (ui256, ui256) -> ui256

contract C {
    function h(uint256 a) internal pure returns (uint256) {
        return a + 1;
    }

    function h(uint256 a, uint256 b) internal pure returns (uint256) {
        return a + b;
    }

    function caller() public pure returns (uint256) {
        return h(h(10), 20);
    }
}
