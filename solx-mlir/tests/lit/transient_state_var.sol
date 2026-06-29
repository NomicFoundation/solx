// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.state_var @{{.*}} transient slot 0 offset 0 : ui256

// CHECK: sol.func @{{.*get_t.*}}
// CHECK:   sol.load %{{.*}} : !sol.ptr<ui256, Transient>, ui256

// CHECK: sol.func @{{.*set_t.*}}
// CHECK:   sol.store %{{.*}}, %{{.*}} : ui256, !sol.ptr<ui256, Transient>

contract C {
    uint256 transient t;

    function get_t() public view returns (uint256) {
        return t;
    }

    function set_t(uint256 v) public {
        t = v;
    }
}
