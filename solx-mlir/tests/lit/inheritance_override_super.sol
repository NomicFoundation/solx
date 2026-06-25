// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// Base + derived with a `virtual`/`override` function and a `super.foo()` call.
// Both backends emit two contract modules (Base, then Derived). In Derived the
// overriding `foo` is emitted first, followed by the inherited base body, which
// the override reaches through a direct `sol.call` to the base function. Symbol
// names differ (solc appends `_<nodeid>`, solx prefixes the base name) so they
// are matched with regex. The constant-2 materialization is order-independent
// between the backends, so it is pinned with CHECK-DAG.

// CHECK: sol.contract @{{.*Base.*}}
// CHECK: sol.func @{{.*foo.*}}() -> ui256
// CHECK:   sol.constant 1 : ui8
// CHECK:   sol.return

// CHECK: sol.contract @{{.*Derived.*}}
// CHECK: sol.func @{{.*foo.*}}() -> ui256
// CHECK-DAG:   sol.constant 2 : ui8
// CHECK-DAG:   sol.call @{{.*foo.*}}() : () -> ui256
// CHECK:   sol.cadd
// CHECK:   sol.return

contract Base {
    function foo() public pure virtual returns (uint256) {
        return 1;
    }
}

contract Derived is Base {
    function foo() public pure override returns (uint256) {
        return super.foo() + 2;
    }
}
