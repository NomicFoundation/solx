// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK-LABEL: sol.func @{{.*se.*}}(%{{.*}}: i1) -> ui256
// CHECK: %[[SLOT:.*]] = sol.alloca : !sol.ptr<ui256, Stack>
// CHECK: sol.if %{{.*}} {
// CHECK:   %[[T:.*]] = sol.call @{{.*inc.*}}() : () -> ui256
// CHECK:   sol.store %[[T]], %[[SLOT]] : ui256, !sol.ptr<ui256, Stack>
// CHECK:   sol.yield
// CHECK: } else {
// CHECK:   %[[F:.*]] = sol.call @{{.*dec.*}}() : () -> ui256
// CHECK:   sol.store %[[F]], %[[SLOT]] : ui256, !sol.ptr<ui256, Stack>
// CHECK:   sol.yield
// CHECK: }
// CHECK: sol.load %[[SLOT]] : !sol.ptr<ui256, Stack>, ui256

contract C {
    uint256 s;
    function inc() internal returns (uint256) { s += 1; return s; }
    function dec() internal returns (uint256) { s -= 1; return s; }
    function se(bool c) public returns (uint256) { return c ? inc() : dec(); }
}
