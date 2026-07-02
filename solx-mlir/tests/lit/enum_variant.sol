// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*ev.*}}() -> !sol.enum<2>
// CHECK:   %[[V1:.*]] = sol.constant 1 : ui256
// CHECK:   sol.enum_cast %[[V1]] : ui256 to !sol.enum<2>
// CHECK: sol.func @{{.*evq.*}}() -> !sol.enum<2>
// CHECK:   %[[V2:.*]] = sol.constant 2 : ui256
// CHECK:   sol.enum_cast %[[V2]] : ui256 to !sol.enum<2>

contract C {
    enum E { A, B, C }

    function ev() public pure returns (E) { return E.B; }

    function evq() public pure returns (E) { return C.E.C; }
}
