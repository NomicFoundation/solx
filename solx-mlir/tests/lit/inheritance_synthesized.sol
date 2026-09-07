// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.contract @{{.*}}Leaf
// CHECK: sol.func @{{.*}}() attributes {kind = #{{.*}}Constructor
// CHECK:   sol.call @[[PROVIDER:.*]]() : () -> ()
// CHECK: sol.func @[[PROVIDER]]() attributes {{{(id = [0-9]+ : i64, )?}}state_mutability
// CHECK:   sol.constant 5
// CHECK:   sol.call @[[ROOT:.*]](%{{.*}}) : (ui256) -> ()
// CHECK: sol.func @[[ROOT]](%arg0: ui256) attributes {{{(id = [0-9]+ : i64, )?}}state_mutability

abstract contract Root {
    constructor(uint256 initial) {}
}

abstract contract Provider is Root(5) {
    constructor() {}
}

contract Leaf is Provider {}
