// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*ternary_scalar.*}}(%{{.*}}: i1, %{{.*}}: ui256, %{{.*}}: ui256) -> ui256
// CHECK:   sol.if %{{.*}} {
// CHECK:     sol.store %{{.*}}, %[[SLOT:.*]] : ui256, !sol.ptr<ui256, Stack>
// CHECK:   } else {
// CHECK:     sol.store %{{.*}}, %[[SLOT]] : ui256, !sol.ptr<ui256, Stack>
// CHECK:   }
// CHECK:   sol.load %[[SLOT]] : !sol.ptr<ui256, Stack>, ui256

// CHECK: sol.func @{{.*ternary_string.*}}(%{{.*}}: i1) -> !sol.string<Memory>
// CHECK:   %[[STR_SLOT:.*]] = sol.alloca : !sol.ptr<!sol.string<Memory>, Stack>
// CHECK:   sol.if
// CHECK:     %[[Y:.*]] = sol.string_lit "yes" -> !sol.string<Memory>
// CHECK:     sol.store %[[Y]], %[[STR_SLOT]] : !sol.string<Memory>, !sol.ptr<!sol.string<Memory>, Stack>
// CHECK:     %[[N:.*]] = sol.string_lit "no" -> !sol.string<Memory>
// CHECK:     sol.store %[[N]], %[[STR_SLOT]] : !sol.string<Memory>, !sol.ptr<!sol.string<Memory>, Stack>

contract C {
    function ternary_scalar(bool c, uint256 a, uint256 b) public pure returns (uint256) {
        return c ? a : b;
    }

    function ternary_string(bool c) public pure returns (string memory) {
        return c ? "yes" : "no";
    }
}
