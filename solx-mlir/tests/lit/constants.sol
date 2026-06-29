// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*getDouble.*}}() -> ui8
// CHECK:   sol.constant 42 : ui8
// CHECK:   sol.cmul %{{.*}}, %{{.*}} : ui8

// CHECK: sol.func @{{.*read.*}}() -> ui256
// CHECK:   sol.constant 42 : ui8
// CHECK:   sol.return %{{.*}} : ui256

// CHECK: sol.func @{{.*sum.*}}() -> ui256
// CHECK:   sol.cadd %{{.*}}, %{{.*}} : ui256

contract C {
    uint256 constant FOO = 42;
    uint8 constant DOUBLE = uint8(FOO) * 2;

    function getDouble() public pure returns (uint8) {
        return DOUBLE;
    }

    function read() public pure returns (uint256) {
        return FOO;
    }

    function sum() public pure returns (uint256) {
        return FOO + FOO;
    }
}
