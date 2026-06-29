// RUN: solx --emit-mlir=sol %s | FileCheck %s

// Assignment to a contract-qualified state variable (C.x = v): solc's print-init
// aborts NYI via llvm_unreachable at SolidityToMLIR.cpp:1698 (exit 134), so this is solx-only.

// CHECK: sol.func @{{.*setNs.*}}
// CHECK: %[[V:.*]] = sol.load %{{[0-9]+}} : !sol.ptr<ui256, Stack>, ui256
// CHECK: %[[SLOT:.*]] = sol.addr_of @{{x.*}} : !sol.ptr<ui256, Storage>
// CHECK: sol.store %[[V]], %[[SLOT]] : ui256, !sol.ptr<ui256, Storage>

contract C {
    uint256 x;
    function setNs(uint256 v) public { C.x = v; }
}
