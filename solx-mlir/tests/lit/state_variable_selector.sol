// RUN: solx --emit-mlir=sol %s | FileCheck %s

// CHECK: sol.func @{{.*s.*}}() -> !sol.fixedbytes<4>
// CHECK:   sol.constant 1067774533 : ui32
// CHECK:   sol.bytes_cast %{{.*}} : ui32 to !sol.fixedbytes<4>
// CHECK:   sol.return

contract C {
    uint256 public value;

    function s() public view returns (bytes4) {
        return this.value.selector;
    }
}
