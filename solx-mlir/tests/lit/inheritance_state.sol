// RUN: solx --emit-mlir=sol %s | FileCheck %s

// solc's print-init declares a hierarchy's state variables most-derived first, where solx keeps
// storage order, so this is solx-only.

// CHECK: sol.contract @{{.*}}Leaf
// CHECK: sol.state_var @{{.*}}base{{.*}} slot 0 offset 0 : ui256
// CHECK: sol.immutable @{{.*}}frozen{{.*}} : ui256
// CHECK: sol.state_var @{{.*}}leaf{{.*}} slot 1 offset 0 : ui256
// CHECK: sol.func @{{.*}}() attributes {kind = #{{.*}}Constructor
// CHECK:   sol.addr_of @{{.*}}base{{.*}} : !sol.ptr<ui256, Storage>
// CHECK:   sol.store
// CHECK:   sol.addr_of @{{.*}}frozen{{.*}} : !sol.ptr<ui256, Immutable>
// CHECK:   sol.store
// CHECK:   sol.addr_of @{{.*}}leaf{{.*}} : !sol.ptr<ui256, Storage>
// CHECK:   sol.call @[[READ:.*]]() : () -> ui256
// CHECK:   sol.store
// CHECK: sol.func @[[READ]]() -> ui256
// CHECK:   sol.addr_of @{{.*}}base{{.*}} : !sol.ptr<ui256, Storage>
// CHECK:   sol.load_immutable @{{.*}}frozen{{.*}} : ui256
// CHECK: sol.func @{{.*}}base(){{.*}} attributes {orig_fn_type = () -> ui256, selector
// CHECK: sol.func @{{.*}}leaf(){{.*}} attributes {orig_fn_type = () -> ui256, selector

abstract contract Base {
    uint256 public base = 1;
    uint256 immutable frozen = 2;
    uint256 constant CONSTANT = 3;

    function read() internal view returns (uint256) {
        return frozen + base;
    }
}

contract Leaf is Base {
    uint256 public leaf = read();
}
