// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// Each reference to a contract-level `constant` is inlined as if the
// initializer expression appeared at the use site, so `FOO` lowers to
// the same `sol.constant` + `sol.cast` chain as a bare `42` literal.

// CHECK: sol.func @{{.*read.*}}() -> ui256
// CHECK:   %{{.*}} = sol.constant 42 : ui8
// CHECK:   %{{.*}} = sol.cast %{{.*}} : ui8 to ui256
// CHECK:   sol.return %{{.*}} : ui256

// CHECK:      sol.func @{{.*sum.*}}() -> ui256
// CHECK-DAG:    sol.constant 42 : ui8
// CHECK-DAG:    sol.constant 42 : ui8
// CHECK:        sol.cadd %{{.*}}, %{{.*}} : ui256
// CHECK:        sol.return %{{.*}} : ui256

contract C {
    uint256 constant FOO = 42;

    function read() public pure returns (uint256) {
        return FOO;
    }

    function sum() public pure returns (uint256) {
        return FOO + FOO;
    }
}
