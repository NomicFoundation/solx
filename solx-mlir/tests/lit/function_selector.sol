// RUN: solx --emit-mlir=sol %s | FileCheck %s

// CHECK: sol.func @{{.*}}f
// CHECK: sol.constant {{.*}} : ui32
// CHECK: sol.bytes_cast %{{.*}} : ui32 to !sol.fixedbytes<4>

contract C {
    function g() public {}

    function f() public view returns (bytes4) {
        return this.g.selector;
    }
}
