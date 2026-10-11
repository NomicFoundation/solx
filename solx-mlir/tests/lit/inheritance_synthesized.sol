// RUN: slang --emit-mlir=sol %s | FileCheck %s

// CHECK: sol.contract @{{.*}}Leaf
// CHECK: sol.func @{{.*}}() attributes {kind = #{{.*}}Constructor
// CHECK:   sol.constant 5
// CHECK:   sol.store %{{.*}}, %[[ROOT_SLOT:.*]] : ui256, !sol.ptr<ui256, Stack>
// CHECK:   %[[ROOT_ARGUMENT:.*]] = sol.load %[[ROOT_SLOT]] : !sol.ptr<ui256, Stack>, ui256
// CHECK:   sol.call @[[PROVIDER:.*]](%[[ROOT_ARGUMENT]]) : (ui256) -> ()
// CHECK: sol.func private @[[PROVIDER]](%arg0: ui256) attributes {state_mutability = #{{.*}}NonPayable
// CHECK:   sol.call @[[ROOT:.*]](%arg0) : (ui256) -> ()
// CHECK: sol.func private @[[ROOT]](%arg0: ui256) attributes {state_mutability = #{{.*}}NonPayable

abstract contract Root {
    constructor(uint256 initial) {}
}

abstract contract Provider is Root(5) {
    constructor() {}
}

contract Leaf is Provider {}
