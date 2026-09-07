// RUN: solx --emit-mlir=sol %s | FileCheck %s

// CHECK: sol.contract @{{.*}}Leaf
// CHECK: sol.func @{{.*}}(%arg0: ui256) attributes {kind = #{{.*}}Constructor, orig_fn_type = (ui256) -> (), state_mutability = #{{.*}}Payable
// CHECK:   sol.store %arg0
// CHECK:   %[[MIDDLE_ARGUMENT:.*]] = sol.cadd
// CHECK:   sol.call @[[MIDDLE:.*]](%[[MIDDLE_ARGUMENT]], %arg0) : (ui256, ui256) -> ()
// CHECK: sol.func @[[MIDDLE]](%arg0: ui256, %arg1: ui256) attributes {state_mutability = #{{.*}}NonPayable
// CHECK:   sol.store %arg0, %[[MIDDLE_PARAMETER:.*]] : ui256, !sol.ptr<ui256, Stack>
// CHECK:   sol.store %arg1, %[[SEED:.*]] : ui256, !sol.ptr<ui256, Stack>
// CHECK:   %[[ROOT_ARGUMENT:.*]] = sol.load %[[SEED]]
// CHECK:   sol.call @[[ROOT:.*]](%[[ROOT_ARGUMENT]]) : (ui256) -> ()
// CHECK:   %[[MIDDLE_VALUE:.*]] = sol.load %[[MIDDLE_PARAMETER]]
// CHECK:   sol.store %[[MIDDLE_VALUE]], %{{.*}} : ui256, !sol.ptr<ui256, Storage>
// CHECK: sol.func @[[ROOT]](%arg0: ui256) attributes {state_mutability = #{{.*}}Payable
// CHECK:   sol.store %arg0
// CHECK:   sol.addr_of {{.*}} : !sol.ptr<ui256, Immutable>
// CHECK:   sol.store

abstract contract Root {
    uint256 immutable root;

    constructor(uint256 value) payable {
        root = value;
    }
}

abstract contract Middle is Root {
    uint256 middle;

    constructor(uint256 value) {
        middle = value;
    }
}

contract Leaf is Middle {
    constructor(uint256 seed) payable Root(seed) Middle(seed + 1) {}
}
