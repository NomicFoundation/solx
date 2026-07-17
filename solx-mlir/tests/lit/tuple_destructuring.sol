// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*sum.*}}
// CHECK: sol.store %arg0
// CHECK: sol.store %arg1
// CHECK: sol.store %arg2
// CHECK: sol.load
// CHECK-NEXT: sol.load
// CHECK-NEXT: sol.load
// CHECK: sol.alloca
// CHECK-NEXT: sol.store %{{[0-9]+}}
// CHECK: sol.alloca
// CHECK-NEXT: sol.store %{{[0-9]+}}
// CHECK: sol.alloca
// CHECK-NEXT: sol.store %{{[0-9]+}}

// CHECK: sol.func @{{.*literal_fold.*}}
// CHECK: sol.constant 1633837924
// CHECK: sol.bytes_cast %{{.*}} : ui32 to !sol.fixedbytes<4>
// CHECK: sol.cast %{{.*}} : ui8 to ui256

contract C {
    function sum(uint256 x, uint256 y, uint256 z) public pure returns (uint256) {
        (uint256 a, uint256 b, uint256 c) = (x, y, z);
        return a + b + c;
    }

    function literal_fold() public pure returns (bytes4, uint256) {
        (bytes4 p, uint256 q) = ("abcd", 42);
        return (p, q);
    }
}
