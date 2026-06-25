// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// A `uint256 transient t` state var: the state_var carries the `transient`
// keyword + slot, and read/write go through a !sol.ptr<_, Transient> address
// space (EIP-1153). solx and solc agree op-for-op.

// CHECK: sol.state_var @{{.*}} transient slot 0 offset 0 : ui256

// Read: addr_of (Transient) -> load -> return
// CHECK: sol.func @{{.*get_t.*}}
// CHECK:   %[[PTR:.*]] = sol.addr_of @{{.*}} : !sol.ptr<ui256, Transient>
// CHECK:   sol.load %[[PTR]] : !sol.ptr<ui256, Transient>, ui256
// CHECK:   sol.return

// Write: addr_of (Transient) -> store
// CHECK: sol.func @{{.*set_t.*}}
// CHECK-DAG:   sol.addr_of @{{.*}} : !sol.ptr<ui256, Transient>
// CHECK-DAG:   sol.store %{{.*}}, %{{.*}} : ui256, !sol.ptr<ui256, Transient>

contract C {
    uint256 transient t;

    function get_t() public view returns (uint256) {
        return t;
    }

    function set_t(uint256 v) public {
        t = v;
    }
}
