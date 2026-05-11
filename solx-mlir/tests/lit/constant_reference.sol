// RUN: solx --emit-mlir=sol %s | FileCheck %s

// CHECK: sol.func {{.*}}get
// CHECK:   sol.constant 42 : ui8
// CHECK:   sol.return %{{.*}} : ui8

// CHECK: sol.func {{.*}}getDouble
// CHECK:   sol.constant 42 : ui8
// CHECK:   sol.constant 2 : ui8
// CHECK:   sol.cmul %{{.*}}, %{{.*}} : ui8
// CHECK:   sol.return %{{.*}} : ui8

contract C {
    uint8 constant TEST = 42;
    uint8 constant DOUBLE = TEST * 2;

    function get() public pure returns (uint8) {
        return TEST;
    }

    function getDouble() public pure returns (uint8) {
        return DOUBLE;
    }
}
