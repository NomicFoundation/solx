// RUN: solx --emit-mlir=sol %s | FileCheck %s

// A discarded `new C` is an uncalled contract creator: it has no effect and
// emits no sol.new. solc's MLIR backend asserts on it, so this is a solx-only
// check; the deploying `new C()` form is covered by gf_calls_builtins_new_contract.

// CHECK-LABEL: sol.func @"f()"
// CHECK-NOT: sol.new
// CHECK: sol.return

contract D {}

contract C {
    function f() public {
        new D;
    }
}
