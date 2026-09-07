// RUN: solx --emit-mlir=sol %s | FileCheck %s

// solc's print-init aborts on a public state variable shadowing a function from an intermediate
// base, so this is solx-only.

// CHECK: sol.contract @{{.*}}Leaf
// CHECK: sol.func @{{.*}}() attributes {kind = #{{.*}}Constructor
// CHECK-NOT: sol.func @{{.*}}shadowed(){{.*}}id =
// CHECK: sol.func @{{.*}}shadowed(){{.*}} attributes {orig_fn_type = () -> ui256, selector
// CHECK: } {kind = #Contract}

abstract contract Base {
    function shadowed() external virtual returns (uint256);
}

abstract contract Middle is Base {
    uint256 public override shadowed;
}

contract Leaf is Middle {}
