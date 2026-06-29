// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.contract @{{.*D.*}}
// CHECK: sol.func @{{.*who.*}}() -> ui256 attributes {{.*}}selector = -690872835
// CHECK:   sol.call @{{.*who.*}}() : () -> ui256
// CHECK:   sol.constant 1000 : ui16
// CHECK:   sol.cadd

// CHECK: sol.func @{{.*who.*}}() -> ui256
// CHECK:   sol.call @{{.*who.*}}() : () -> ui256
// CHECK:   sol.constant 100 : ui8
// CHECK:   sol.cadd

// CHECK: sol.func @{{.*who.*}}() -> ui256
// CHECK:   sol.call @{{.*who.*}}() : () -> ui256
// CHECK:   sol.constant 10 : ui8
// CHECK:   sol.cadd

// CHECK: sol.func @{{.*who.*}}() -> ui256
// CHECK:   sol.constant 1 : ui8
// CHECK-NOT: sol.call
// CHECK:   sol.return

contract A {
    function who() public pure virtual returns (uint256) {
        return 1;
    }
}

contract B is A {
    function who() public pure virtual override returns (uint256) {
        return super.who() + 10;
    }
}

contract C is A {
    function who() public pure virtual override returns (uint256) {
        return super.who() + 100;
    }
}

contract D is B, C {
    function who() public pure override(B, C) returns (uint256) {
        return super.who() + 1000;
    }
}
