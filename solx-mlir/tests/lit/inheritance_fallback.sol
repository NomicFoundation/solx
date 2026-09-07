// RUN: solx --emit-mlir=sol %s | FileCheck %s

// solc's print-init emits a base's overridden fallback beside the one the leaf resolves to, so this
// is solx-only.

// CHECK: sol.contract @{{.*}}Leaf
// CHECK: sol.func @{{.*}}() attributes {kind = #{{.*}}Constructor
// CHECK: sol.func @{{.*}}() attributes {kind = #{{.*}}Fallback
// CHECK: sol.func @{{.*}}() attributes {kind = #{{.*}}Receive, state_mutability = #{{.*}}Payable
// CHECK-NOT: kind = #{{.*}}Fallback
// CHECK: } {kind = #Contract}

abstract contract Base {
    fallback(bytes calldata input) external virtual returns (bytes memory) {
        return input;
    }
}

contract Leaf is Base {
    fallback() external override {}

    receive() external payable {}
}
