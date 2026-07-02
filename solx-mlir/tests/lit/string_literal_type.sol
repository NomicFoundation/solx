// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*pick.*}}(%{{.*}}: i1) -> !sol.string<Memory>
// CHECK:   %[[SLOT:.*]] = sol.alloca : !sol.ptr<!sol.string<Memory>, Stack>
// CHECK:   sol.if
// CHECK:     %[[Y:.*]] = sol.string_lit "yes" -> !sol.string<Memory>
// CHECK:     sol.store %[[Y]], %[[SLOT]] : !sol.string<Memory>, !sol.ptr<!sol.string<Memory>, Stack>
// CHECK:     %[[N:.*]] = sol.string_lit "no" -> !sol.string<Memory>
// CHECK:     sol.store %[[N]], %[[SLOT]] : !sol.string<Memory>, !sol.ptr<!sol.string<Memory>, Stack>

contract C {
    function pick(bool c) public pure returns (string memory) {
        return c ? "yes" : "no";
    }
}
