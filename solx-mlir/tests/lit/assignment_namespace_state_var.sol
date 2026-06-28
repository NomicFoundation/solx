// RUN: solx --emit-mlir=sol %s | FileCheck %s

// CHECK: sol.func @{{.*setNs.*}}
// CHECK: %[[V:.*]] = sol.load %{{[0-9]+}} : !sol.ptr<ui256, Stack>, ui256
// CHECK: %[[SLOT:.*]] = sol.addr_of @{{x.*}} : !sol.ptr<ui256, Storage>
// CHECK: sol.store %[[V]], %[[SLOT]] : ui256, !sol.ptr<ui256, Storage>

contract C {
    uint256 x;
    function setNs(uint256 v) public { C.x = v; }
}
