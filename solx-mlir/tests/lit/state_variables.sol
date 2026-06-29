// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.state_var @{{.*}} slot 0 offset 0 : ui256
// CHECK: sol.state_var @{{.*}} slot 1 offset 0 : ui256

// CHECK: sol.func @{{.*get_x.*}}
// CHECK:   %[[PX:.*]] = sol.addr_of @{{.*}} : !sol.ptr<ui256, Storage>
// CHECK:   sol.load %[[PX]] : !sol.ptr<ui256, Storage>, ui256
// CHECK:   sol.return

// CHECK: sol.func @{{.*get_y.*}}
// CHECK:   %[[PY:.*]] = sol.addr_of @{{.*}} : !sol.ptr<ui256, Storage>
// CHECK:   sol.load %[[PY]] : !sol.ptr<ui256, Storage>, ui256
// CHECK:   sol.return

// CHECK: sol.func @{{.*set_x.*}}
// CHECK:   %[[PS:.*]] = sol.addr_of @{{.*}} : !sol.ptr<ui256, Storage>
// CHECK:   sol.store %{{.*}}, %[[PS]] : ui256, !sol.ptr<ui256, Storage>

// CHECK: sol.func @{{.*swap.*}}
// CHECK:   sol.store %{{.*}} : ui256, !sol.ptr<ui256, Stack>
// CHECK:   sol.store %{{.*}} : ui256, !sol.ptr<ui256, Storage>
// CHECK:   sol.store %{{.*}} : ui256, !sol.ptr<ui256, Storage>

contract C {
    uint256 x;
    uint256 y;

    function get_x() public view returns (uint256) {
        return x;
    }

    function get_y() public view returns (uint256) {
        return y;
    }

    function set_x(uint256 value) public {
        x = value;
    }

    function swap() public {
        uint256 temporary = x;
        x = y;
        y = temporary;
    }
}
