// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*maxValue.*}}() -> !sol.enum<2>
// CHECK:   %[[MAX:.*]] = sol.constant 2 : ui256
// CHECK:   sol.enum_cast %[[MAX]] : ui256 to !sol.enum<2>

// CHECK: sol.func @{{.*minValue.*}}() -> !sol.enum<2>
// CHECK:   %[[MIN:.*]] = sol.constant 0 : ui256
// CHECK:   sol.enum_cast %[[MIN]] : ui256 to !sol.enum<2>

contract C {
    enum Color { Red, Green, Blue }

    function maxValue() public pure returns (Color) {
        return type(Color).max;
    }

    function minValue() public pure returns (Color) {
        return type(Color).min;
    }
}
