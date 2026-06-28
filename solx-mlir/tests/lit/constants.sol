// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK-DAG: sol.func @{{.*read.*}}() -> ui256
// CHECK-DAG:   %{{.*}} = sol.constant 42 : ui8
// CHECK-DAG:   %{{.*}} = sol.cast %{{.*}} : ui8 to ui256
// CHECK-DAG:   sol.return %{{.*}} : ui256

// CHECK-DAG:      sol.func @{{.*sum.*}}() -> ui256
// CHECK-DAG:    sol.constant 42 : ui8
// CHECK-DAG:    sol.constant 42 : ui8
// CHECK-DAG:        sol.cadd %{{.*}}, %{{.*}} : ui256
// CHECK-DAG:        sol.return %{{.*}} : ui256

// CHECK-DAG:      sol.func @{{.*getDouble.*}}() -> ui8
// CHECK-DAG:    sol.constant 42 : ui8
// CHECK-DAG:    sol.constant 2 : ui8
// CHECK-DAG:        sol.cmul %{{.*}}, %{{.*}} : ui8

contract C {
    uint256 constant FOO = 42;
    uint8 constant DOUBLE = uint8(FOO) * 2;

    function read() public pure returns (uint256) {
        return FOO;
    }

    function sum() public pure returns (uint256) {
        return FOO + FOO;
    }

    function getDouble() public pure returns (uint8) {
        return DOUBLE;
    }
}
