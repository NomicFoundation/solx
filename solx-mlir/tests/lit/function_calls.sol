// RUN: solx --emit-mlir %s | FileCheck %s

// CHECK: sol.func @"add(uint256,uint256)"(%{{.*}}: ui256, %{{.*}}: ui256) -> ui256
// CHECK:   sol.cadd

// CHECK: sol.func @"double
// CHECK:   sol.call @"add(uint256,uint256)"(%{{.*}}, %{{.*}}) : (ui256, ui256) -> ui256

// CHECK: sol.func @"chain
// CHECK:   sol.call @"double(uint256)"
// CHECK:   sol.call @"add(uint256,uint256)"

contract C {
    function add(uint256 a, uint256 b) public pure returns (uint256) {
        return a + b;
    }

    function double(uint256 x) public pure returns (uint256) {
        return add(x, x);
    }

    function chain(uint256 x) public pure returns (uint256) {
        return add(double(x), x);
    }
}
