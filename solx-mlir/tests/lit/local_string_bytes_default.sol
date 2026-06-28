// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*f.*}}()
// CHECK:   %[[SLOT_S:.*]] = sol.alloca : !sol.ptr<!sol.string<Memory>, Stack>
// CHECK:   %[[BUF_S:.*]] = sol.malloc :  !sol.string<Memory>
// CHECK:   sol.store %[[BUF_S]], %[[SLOT_S]] : !sol.string<Memory>, !sol.ptr<!sol.string<Memory>, Stack>
// CHECK:   %[[SLOT_B:.*]] = sol.alloca : !sol.ptr<!sol.string<Memory>, Stack>
// CHECK:   %[[BUF_B:.*]] = sol.malloc :  !sol.string<Memory>
// CHECK:   sol.store %[[BUF_B]], %[[SLOT_B]] : !sol.string<Memory>, !sol.ptr<!sol.string<Memory>, Stack>

contract C {
    function f() public pure {
        string memory s;
        bytes memory b;
        s;
        b;
    }
}
