// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.contract @{{.*Derived.*}}
// CHECK: sol.func @{{.*call.*}}() -> ui256
// CHECK:   sol.call @{{.*foo.*}}() : () -> ui256
// CHECK:   sol.call @{{.*foo.*}}() : () -> ui256
// CHECK:   sol.cadd
// CHECK:   sol.return
// CHECK: sol.func @{{.*foo.*}}() -> ui256
// CHECK:   sol.constant 2 : ui8
// CHECK: sol.func @{{.*foo.*}}() -> ui256
// CHECK:   sol.constant 1 : ui8

contract Base {
    function foo() internal pure virtual returns (uint256) {
        return 1;
    }
}

contract Derived is Base {
    function call() public pure returns (uint256) {
        return Base.foo() + foo();
    }

    function foo() internal pure override returns (uint256) {
        return 2;
    }
}
