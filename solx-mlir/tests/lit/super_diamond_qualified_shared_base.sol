// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*aggregate.*}}() -> ui256
// CHECK:   sol.call @{{.*}}() : () -> ui256
// CHECK:   sol.call @{{.*}}() : () -> ui256
// CHECK:   sol.cadd
// CHECK:   sol.call @{{.*}}() : () -> ui256
// CHECK:   sol.cadd

// CHECK: sol.func @{{.*contributeLarge.*}}() -> ui256
// CHECK:   sol.call @{{.*}}() : () -> ui256
// CHECK:   sol.constant 100 : ui8
// CHECK:   sol.cadd

// CHECK: sol.func @{{.*contributeSmall.*}}() -> ui256
// CHECK:   sol.call @{{.*}}() : () -> ui256
// CHECK:   sol.constant 10 : ui8
// CHECK:   sol.cadd

// CHECK: sol.func @{{.*shared.*}}() -> ui256
// CHECK:   sol.constant 1 : ui8

contract A {
    function shared() internal pure virtual returns (uint256) { return 1; }
}

contract B is A {
    function contributeSmall() internal pure returns (uint256) { return A.shared() + 10; }
}

contract C is A {
    function contributeLarge() internal pure returns (uint256) { return A.shared() + 100; }
}

contract D is B, C {
    function aggregate() public pure returns (uint256) { return contributeSmall() + contributeLarge() + A.shared(); }
}
