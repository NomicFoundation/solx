// RUN: solx --emit-mlir=sol %s | FileCheck %s

// solc emits a getter before the functions and solx after them, so this test
// is solx-only.

// CHECK: sol.state_var @{{.*low.*}} slot 100 offset 0 : ui64
// CHECK: sol.state_var @{{.*high.*}} slot 100 offset 8 : ui64

// CHECK: sol.func @{{.*write.*}}
// CHECK:   %[[PTR:.*]] = sol.addr_of @{{.*high.*}} : !sol.ptr<ui64, Storage>
// CHECK:   sol.store %{{.*}}, %[[PTR]] : ui64, !sol.ptr<ui64, Storage>

// CHECK: sol.func @{{.*high.*}}
// CHECK:   %[[GETTER:.*]] = sol.addr_of @{{.*high.*}} : !sol.ptr<ui64, Storage>
// CHECK:   %[[VALUE:.*]] = sol.load %[[GETTER]] : !sol.ptr<ui64, Storage>, ui64
// CHECK:   sol.return %[[VALUE]] : ui64

contract Based layout at 100 {
    uint64 low;
    uint64 public high;

    function write(uint64 value) public {
        high = value;
    }
}
