// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.state_var @{{.*low.*}} transient slot 0 offset 0 : ui64
// CHECK: sol.state_var @{{.*high.*}} transient slot 0 offset 8 : ui64

// CHECK: sol.func @{{.*read.*}}() -> ui64
// CHECK:   %[[PTR:.*]] = sol.addr_of @{{.*low.*}} : !sol.ptr<ui64, Transient>
// CHECK:   sol.load %[[PTR]] : !sol.ptr<ui64, Transient>, ui64

// CHECK: sol.func @{{.*write.*}}
// CHECK:   %[[PTR:.*]] = sol.addr_of @{{.*high.*}} : !sol.ptr<ui64, Transient>
// CHECK:   sol.store %{{.*}}, %[[PTR]] : ui64, !sol.ptr<ui64, Transient>

contract Counter {
    uint64 transient low;
    uint64 transient high;

    function read() public view returns (uint64) {
        return low;
    }

    function write(uint64 value) public {
        high = value;
    }
}
