// RUN: solx --emit-mlir=sol %s | FileCheck %s

// CHECK: sol.contract @{{.*}}Leaf
// CHECK: sol.func @{{.*}}() attributes {kind = #{{.*}}Constructor
// CHECK:   sol.call @[[PROVIDER:.*]]() : () -> ()
// CHECK: sol.func private @[[PROVIDER]]() attributes {state_mutability = #{{.*}}NonPayable
// CHECK:   sol.constant 5
// CHECK:   sol.call @[[ROOT:.*]](%{{.*}}) : (ui256) -> ()
// CHECK: sol.func private @[[ROOT]](%arg0: ui256) attributes {state_mutability = #{{.*}}NonPayable

abstract contract Root {
    constructor(uint256 initial) {}
}

abstract contract Provider is Root(5) {
    constructor() {}
}

contract Leaf is Provider {}
