// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// A conditional whose two branches are string literals forces the literal's
// *type* to be resolved (not just materialised): the ternary's result slot is
// typed from the common type of the two `LiteralKind::String` operands, driving
// `Type::resolve`'s string-literal arm to `!sol.string<Memory>`. The shared
// result slot and the per-branch `sol.string_lit` stores prove the arm fired.
// Both backends agree.

contract C {
    function pick(bool c) public pure returns (string memory) {
        return c ? "yes" : "no";
    }
}

// CHECK: sol.func @{{.*pick.*}}(%{{.*}}: i1) -> !sol.string<Memory>
// CHECK:   %[[SLOT:.*]] = sol.alloca : !sol.ptr<!sol.string<Memory>, Stack>
// CHECK:   sol.if
// CHECK:     %[[Y:.*]] = sol.string_lit "yes" -> !sol.string<Memory>
// CHECK:     sol.store %[[Y]], %[[SLOT]] : !sol.string<Memory>, !sol.ptr<!sol.string<Memory>, Stack>
// CHECK:     %[[N:.*]] = sol.string_lit "no" -> !sol.string<Memory>
// CHECK:     sol.store %[[N]], %[[SLOT]] : !sol.string<Memory>, !sol.ptr<!sol.string<Memory>, Stack>
