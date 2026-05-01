// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK-DAG: sol.state_var @{{.*}} slot 0 offset 0 : ui256
// CHECK-DAG: sol.state_var @{{.*}} slot 1 offset 0 : ui256

// Read: addr_of -> load -> return
// CHECK: sol.func @{{.*get_x.*}}
// CHECK:   %[[PTR:.*]] = sol.addr_of @{{.*}} : !sol.ptr<ui256, Storage>
// CHECK:   sol.load %[[PTR]] : !sol.ptr<ui256, Storage>, ui256
// CHECK:   sol.return

// Write: addr_of -> store
// CHECK: sol.func @{{.*set_x.*}}
// CHECK:   %[[PTR:.*]] = sol.addr_of @{{.*}} : !sol.ptr<ui256, Storage>
// CHECK:   sol.store %{{.*}}, %[[PTR]] : ui256, !sol.ptr<ui256, Storage>

// Swap: touches both state vars
// CHECK: sol.func @{{.*swap.*}}
// CHECK-DAG: sol.addr_of
// CHECK-DAG: sol.store

contract C {
    uint256 x;
    uint256 y;

    function get_x() public view returns (uint256) {
        return x;
    }

    function set_x(uint256 val) public {
        x = val;
    }

    function get_y() public view returns (uint256) {
        return y;
    }

    function swap() public {
        uint256 tmp = x;
        x = y;
        y = tmp;
    }
}
