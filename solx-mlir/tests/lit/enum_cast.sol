// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// Explicit enum <-> uint8 conversions. Both directions lower to `sol.enum_cast`
// (ui8 to !sol.enum<2> and !sol.enum<2> to ui8). solx orders functions
// alphabetically (fromUint, toUint), solc in source order (toUint, fromUint);
// CHECK-DAG tolerates the order swap since the bodies are distinct.

// CHECK-DAG: sol.func @{{.*fromUint.*}}(%{{.*}}: ui8) -> !sol.enum<2>
// CHECK-DAG:   sol.enum_cast %{{.*}} : ui8 to !sol.enum<2>

// CHECK-DAG: sol.func @{{.*toUint.*}}(%{{.*}}: !sol.enum<2>) -> ui8
// CHECK-DAG:   sol.enum_cast %{{.*}} : !sol.enum<2> to ui8

contract C {
    enum Color { Red, Green, Blue }

    function toUint(Color a) public pure returns (uint8) {
        return uint8(a);
    }

    function fromUint(uint8 v) public pure returns (Color) {
        return Color(v);
    }
}
