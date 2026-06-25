// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// Library internal functions invoked with the explicit `L.foo(args)` syntax are
// inlined into the calling contract and dispatched through a plain `sol.call`
// with the full argument list. This direct form is parity-clean: both backends
// emit the same chained internal calls (the receiver is an ordinary argument).
// Symbol names differ (solc appends `_<nodeid>`, solx appends `#NodeId(..)`),
// so the callee labels are matched with a regex.

// CHECK: sol.func @{{.*f.*}}(%{{.*}}: ui256) -> ui256
// CHECK:   %[[X:.*]] = sol.load %{{.*}} : !sol.ptr<ui256, Stack>, ui256
// CHECK:   %[[A:.*]] = sol.call @{{.*neg.*}}(%[[X]]) : (ui256) -> ui256
// CHECK:   %[[B:.*]] = sol.call @{{.*half.*}}(%[[A]]) : (ui256) -> ui256
// CHECK:   sol.return %[[B]] : ui256

library L {
    function neg(uint256 a) internal pure returns (uint256) {
        return 0 - a;
    }
    function half(uint256 a) internal pure returns (uint256) {
        return a / 2;
    }
}

contract C {
    function f(uint256 x) public pure returns (uint256) {
        return L.half(L.neg(x));
    }
}
