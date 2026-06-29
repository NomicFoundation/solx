// RUN: solx --emit-mlir=sol %s | FileCheck %s

// Ternary with void-typed arms: solc's print-init aborts on the assertion
// !lVals.empty() in genRValExprs (SolidityToMLIR.cpp:2652), so this is solx-only.

// CHECK-LABEL: sol.func @"choose(bool)"
// CHECK: sol.if %{{[0-9]+}} {
// CHECK-NEXT: sol.call @"a()"() : () -> ()
// CHECK-NEXT: sol.yield
// CHECK-NEXT: } else {
// CHECK-NEXT: sol.call @"b()"() : () -> ()
// CHECK-NEXT: sol.yield
// CHECK-NEXT: }
// CHECK-NEXT: sol.return

contract C {
    uint256 x;
    function a() internal { x = 1; }
    function b() internal { x = 2; }
    function choose(bool c) public {
        c ? a() : b();
    }
}
