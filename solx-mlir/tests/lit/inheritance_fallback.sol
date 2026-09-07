// RUN: solx --emit-mlir=sol %s | FileCheck %s

// CHECK: sol.contract @{{.*}}Leaf
// CHECK: sol.func @{{.*}}() attributes {kind = #{{.*}}Constructor
// CHECK: sol.func @{{.*}}() attributes {kind = #{{.*}}Fallback
// CHECK-NOT: kind = #{{.*}}Fallback
// CHECK: sol.func @{{.*}}() attributes {kind = #{{.*}}Receive, state_mutability = #{{.*}}Payable
// CHECK-NOT: kind = #{{.*}}Fallback
// CHECK: } {kind = #Contract}

abstract contract Base {
    fallback() external virtual {}
}

contract Leaf is Base {
    fallback() external override {}

    receive() external payable {}
}
