// RUN: solx --emit-mlir=sol %s | FileCheck %s

// solc print-init types the default function constant as the unit signature and stores it into a
// slot of another type; solx types it from the target so the store is consistent.

// CHECK: sol.func @{{.*reset.*}}
// CHECK:   %[[DEFAULT:.*]] = sol.default_func_constant : !sol.func_ref<() -> ui256>
// CHECK:   sol.store %[[DEFAULT]], %[[SLOT:.*]] : !sol.func_ref<() -> ui256>, !sol.ptr<!sol.func_ref<() -> ui256>, Stack>
// CHECK:   %[[G:.*]] = sol.func_constant @{{.*g.*}} : !sol.func_ref<() -> ui256>
// CHECK:   %[[CLEARED:.*]] = sol.load %[[SLOT]] : !sol.ptr<!sol.func_ref<() -> ui256>, Stack>, !sol.func_ref<() -> ui256>
// CHECK:   sol.cmp eq, %[[CLEARED]], %[[G]] : !sol.func_ref<() -> ui256>

contract C {
    function g() internal pure returns (uint256) {
        return 1;
    }

    function reset() public pure returns (uint256) {
        function() internal pure returns (uint256) functionPointer = g;
        delete functionPointer;
        return functionPointer == g ? 1 : 0;
    }
}
