// RUN: solx --emit-mlir=sol %s | FileCheck %s

// Discarded new-expression result: solc's print-init aborts NYI (UNREACHABLE at SolidityToMLIR.cpp:2610), so this is solx-only.

// CHECK-LABEL: sol.func @"f()"
// CHECK-NOT: sol.new
// CHECK: sol.return

contract D {}

contract C {
    function f() public {
        new D;
    }
}
