// RUN: solx --emit-mlir=sol %s | FileCheck %s

// CHECK-LABEL: sol.func @"f()"
// CHECK-NOT: sol.new
// CHECK: sol.return

contract D {}

contract C {
    function f() public {
        new D;
    }
}
