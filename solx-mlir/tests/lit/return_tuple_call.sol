// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// `return f()` where `f` returns a tuple expands to one `sol.call` with N
// results, then a `sol.return` over all of them — not a single value (which
// would mismatch the enclosing function's result arity).

// CHECK: sol.func @{{.*f.*}}() -> (ui256, ui256)
// CHECK: %[[R:[0-9]+]]:2 = sol.call @{{.*pair.*}}() : () -> (ui256, ui256)
// CHECK: sol.return %[[R]]#0, %[[R]]#1 : ui256, ui256

contract C {
    function pair() internal returns (uint256, uint256) {
        return (1, 2);
    }

    function f() public returns (uint256, uint256) {
        return pair();
    }
}
