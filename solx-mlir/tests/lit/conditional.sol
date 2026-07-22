// RUN: solx --emit-mlir=sol %s | FileCheck %s

// solc's print-init aborts on this file's conditional lowering on some platforms, emitting
// nothing, so this is solx-only. TODO: restore solc's RUN line once its MLIR backend stops
// crashing on it.

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

// CHECK: sol.func @{{.*ternary_statement.*}}
// CHECK:   sol.if %{{.*}} {
// CHECK:     sol.call @{{.*effect_a.*}}()
// CHECK:     sol.yield
// CHECK:   } else {
// CHECK:     sol.call @{{.*effect_b.*}}()
// CHECK:     sol.yield
// CHECK:   }

contract C {
    function ternary_scalar(bool c, uint256 a, uint256 b) public pure returns (uint256) {
        return c ? a : b;
    }

    function ternary_string(bool c) public pure returns (string memory) {
        return c ? "yes" : "no";
    }

    function effect_a() internal pure {}

    function effect_b() internal pure {}

    function ternary_statement(bool c) public pure {
        c ? effect_a() : effect_b();
    }
}
