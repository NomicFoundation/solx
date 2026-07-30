// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*variant.*}}-> !sol.enum<2>
// CHECK:   sol.constant 1 : ui256
// CHECK:   sol.enum_cast %{{.*}} : ui256 to !sol.enum<2>

// CHECK: sol.func @{{.*qualified_variant.*}}-> !sol.enum<2>
// CHECK:   sol.constant 2 : ui256
// CHECK:   sol.enum_cast %{{.*}} : ui256 to !sol.enum<2>

// CHECK: sol.func @{{.*type_min.*}}-> !sol.enum<2>
// CHECK:   sol.constant 0 : ui256
// CHECK:   sol.enum_cast %{{.*}} : ui256 to !sol.enum<2>

// CHECK: sol.func @{{.*type_max.*}}-> !sol.enum<2>
// CHECK:   sol.constant 2 : ui256
// CHECK:   sol.enum_cast %{{.*}} : ui256 to !sol.enum<2>

contract C {
    enum E { First, Second, Third }

    function variant() public pure returns (E) {
        return E.Second;
    }

    function qualified_variant() public pure returns (E) {
        return C.E.Third;
    }

    function type_min() public pure returns (E) {
        return type(E).min;
    }

    function type_max() public pure returns (E) {
        return type(E).max;
    }
}
