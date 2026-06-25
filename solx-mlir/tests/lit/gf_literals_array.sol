// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// A fixed-size array literal `[a, b, c]` lowers each element to a constant and
// gathers them with `sol.array_lit` into a fixed memory array.

// CHECK: sol.func @{{.*}}arr
// CHECK:   sol.constant 1 : ui8
// CHECK:   sol.constant 2 : ui8
// CHECK:   sol.constant 3 : ui8
// CHECK:   sol.array_lit %{{.*}}, %{{.*}}, %{{.*}} : (ui8, ui8, ui8) -> !sol.array<3 x ui8, Memory>

contract C {
    function arr() public pure returns (uint8) {
        uint8[3] memory a = [uint8(1), 2, 3];
        return a[1];
    }
}
