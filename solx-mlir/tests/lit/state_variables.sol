// RUN: solx --emit-mlir %s | FileCheck %s

// CHECK-DAG: sol.state_var @slot_0 slot 0 offset 0 : ui256
// CHECK-DAG: sol.state_var @slot_1 slot 1 offset 0 : ui256

// Read: addr_of -> load -> return
// CHECK: sol.func @"get_x()"
// CHECK:   %[[PTR:.*]] = sol.addr_of @slot_0 : !sol.ptr<ui256, Storage>
// CHECK:   %[[VAL:.*]] = sol.load %[[PTR]] : !sol.ptr<ui256, Storage>, ui256
// CHECK:   sol.return %[[VAL]]

// Write: addr_of -> store
// CHECK: sol.func @"set_x(uint256)"
// CHECK:   %[[PTR:.*]] = sol.addr_of @slot_0 : !sol.ptr<ui256, Storage>
// CHECK:   sol.store %{{.*}}, %[[PTR]] : ui256, !sol.ptr<ui256, Storage>

// Swap: load both, store crossed
// CHECK: sol.func @"swap()"
// CHECK:   %[[P0:.*]] = sol.addr_of @slot_0
// CHECK:   sol.load %[[P0]]
// CHECK:   %[[P1:.*]] = sol.addr_of @slot_1
// CHECK:   sol.load %[[P1]]
// CHECK:   sol.addr_of @slot_0
// CHECK:   sol.store
// CHECK:   sol.addr_of @slot_1
// CHECK:   sol.store

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
