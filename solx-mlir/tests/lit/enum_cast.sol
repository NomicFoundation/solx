// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*fromUint.*}}(%{{.*}}: ui8) -> !sol.enum<2>
// CHECK:   sol.enum_cast %{{.*}} : ui8 to !sol.enum<2>

// CHECK: sol.func @{{.*toUint.*}}(%{{.*}}: !sol.enum<2>) -> ui8
// CHECK:   sol.enum_cast %{{.*}} : !sol.enum<2> to ui8

// CHECK: sol.func @{{.*toWide.*}}(%{{.*}}: !sol.enum<2>) -> ui256
// CHECK:   sol.enum_cast %{{.*}} : !sol.enum<2> to ui8
// CHECK:   sol.cast %{{.*}} : ui8 to ui256

contract C {
    enum Color { Red, Green, Blue }

    function fromUint(uint8 v) public pure returns (Color) {
        return Color(v);
    }

    function toUint(Color a) public pure returns (uint8) {
        return uint8(a);
    }

    function toWide(Color a) public pure returns (uint256) {
        return uint256(uint8(a));
    }
}
