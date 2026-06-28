// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.contract @D{{.*}} {
// CHECK-DAG:   sol.constant 1 : ui8
// CHECK-DAG:   sol.constant 10 : ui8
// CHECK-DAG:   sol.constant 100 : ui8
// CHECK-DAG:   sol.func @{{.*go.*}}() -> ui256
// CHECK-DAG:     sol.call @{{.*}}() : () -> ui256
// CHECK-DAG:     sol.call @{{.*}}() : () -> ui256
// CHECK-DAG:     sol.call @{{.*}}() : () -> ui256

// CHECK-NOT: sol.constant 2 : ui8

contract A {
    function base() internal pure virtual returns (uint256) { return 1; }
}

contract B is A {
    function fromB() internal pure returns (uint256) { return A.base() + 10; }
}

contract C is A {
    function fromC() internal pure returns (uint256) { return A.base() + 100; }
}

contract D is B, C {
    function go() public pure returns (uint256) { return fromB() + fromC() + A.base(); }
}
