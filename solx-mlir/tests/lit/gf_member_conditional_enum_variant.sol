// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// E.Variant (and qualified C.E.Variant) not in call position: the variant's
// ordinal as a ui256 constant, bridged to the enum type via sol.enum_cast.
// solx walks functions alphabetically, solc in source order; CHECK-DAG covers both.

// CHECK-DAG: sol.func @{{.*ev.*}}() -> !sol.enum<2>
// CHECK-DAG:   %[[V1:.*]] = sol.constant 1 : ui256
// CHECK-DAG:   sol.enum_cast %[[V1]] : ui256 to !sol.enum<2>
// CHECK-DAG: sol.func @{{.*evq.*}}() -> !sol.enum<2>
// CHECK-DAG:   %[[V2:.*]] = sol.constant 2 : ui256
// CHECK-DAG:   sol.enum_cast %[[V2]] : ui256 to !sol.enum<2>

contract C {
    enum E { A, B, C }
    function ev() public pure returns (E) { return E.B; }
    function evq() public pure returns (E) { return C.E.C; }
}
