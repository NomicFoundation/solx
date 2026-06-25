// RUN: solx --emit-mlir=sol %s | FileCheck %s

// A namespace-qualified state-variable lvalue `C.x = v`. The left-hand side is a
// `MemberAccessExpression` whose member resolves to a `StateVariable`, so it is
// treated as the bare `x = v` (a storage scalar store) rather than a struct-field
// access. solc's nascent MLIR backend rejects this form (NYI), so this is a
// solx-only check.

// CHECK: sol.func @{{.*setNs.*}}
// CHECK: %[[V:.*]] = sol.load %{{[0-9]+}} : !sol.ptr<ui256, Stack>, ui256
// CHECK: %[[SLOT:.*]] = sol.addr_of @{{x.*}} : !sol.ptr<ui256, Storage>
// CHECK: sol.store %[[V]], %[[SLOT]] : ui256, !sol.ptr<ui256, Storage>

contract C {
    uint256 x;
    function setNs(uint256 v) public { C.x = v; }
}
