// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// type(E).min and type(E).max fold to the bounding ordinal constants (0 and 2
// for a three-member enum) materialized as `sol.constant N : ui256` and then
// `sol.enum_cast` to the enum type. solx is alphabetical (maxVal, minVal), solc
// source order (minVal, maxVal); CHECK-DAG covers both.

// CHECK-DAG: sol.func @{{.*minVal.*}}() -> !sol.enum<2>
// CHECK-DAG:   %[[MIN:.*]] = sol.constant 0 : ui256
// CHECK-DAG:   sol.enum_cast %[[MIN]] : ui256 to !sol.enum<2>

// CHECK-DAG: sol.func @{{.*maxVal.*}}() -> !sol.enum<2>
// CHECK-DAG:   %[[MAX:.*]] = sol.constant 2 : ui256
// CHECK-DAG:   sol.enum_cast %[[MAX]] : ui256 to !sol.enum<2>

contract C {
    enum Color { Red, Green, Blue }

    function minVal() public pure returns (Color) {
        return type(Color).min;
    }

    function maxVal() public pure returns (Color) {
        return type(Color).max;
    }
}
