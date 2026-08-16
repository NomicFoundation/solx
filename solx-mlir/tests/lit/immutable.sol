// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.immutable @{{.*x.*}} : ui256
// CHECK: sol.state_var @{{.*y.*}} slot 0 offset 0 : ui256

// CHECK: sol.func @{{.*}}(%arg0: ui256) attributes {{.*}}kind = #{{.*}}Constructor
// CHECK:   %[[XW:.*]] = sol.addr_of @{{.*x.*}} : !sol.ptr<ui256, Immutable>
// CHECK:   sol.store %{{.*}}, %[[XW]] : ui256, !sol.ptr<ui256, Immutable>
// CHECK:   %[[XR:.*]] = sol.addr_of @{{.*x.*}} : !sol.ptr<ui256, Immutable>
// CHECK:   %[[XV:.*]] = sol.load %[[XR]] : !sol.ptr<ui256, Immutable>, ui256
// CHECK:   sol.store %{{.*}}, %{{.*}} : ui256, !sol.ptr<ui256, Storage>

// CHECK: sol.func @{{.*get.*}}() -> ui256 attributes {{.*}}selector = 1833756220 : i32
// CHECK:   %[[V:.*]] = sol.load_immutable @{{.*x.*}} : ui256
// CHECK:   sol.return %[[V]] : ui256

contract Immutable {
    uint256 immutable x;
    uint256 y;

    constructor(uint256 seed) {
        x = seed;
        y = x + 1;
    }

    function get() public view returns (uint256) {
        return x;
    }
}
