// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// A library (or free) function used as an internal function pointer must carry
// an `id` like any other referenceable function, even though it is emitted under
// a node-id symbol override. Without it, the `sol.func_constant` lowering's
// `fn.getId()` assertion fires.

// CHECK-DAG: sol.func {{@.*x.*attributes \{id = [0-9]+}}
// CHECK-DAG: sol.func_constant @{{.*}} : !sol.func_ref<(ui256) -> ui256>
// CHECK-DAG: sol.icall %{{[0-9]+}}(%{{[0-9]+}}) : !sol.func_ref<(ui256) -> ui256>, (ui256) -> ui256

library L {
    function x(uint256 a) internal returns (uint256) {
        return a + 1;
    }

    function apply_(function(uint256) internal returns (uint256) f, uint256 a)
        internal
        returns (uint256)
    {
        return f(a);
    }

    function run(uint256 a) internal returns (uint256) {
        return apply_(x, a);
    }
}

contract C {
    function f(uint256 a) public returns (uint256) {
        return L.run(a);
    }
}
