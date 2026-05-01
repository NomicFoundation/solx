// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*add.*}}(%{{.*}}: ui256, %{{.*}}: ui256) -> ui256
// CHECK:   sol.cadd

// CHECK: sol.func @{{.*double.*}}
// CHECK:   sol.call @{{.*add.*}}(%{{.*}}, %{{.*}}) : (ui256, ui256) -> ui256

// CHECK: sol.func @{{.*chain.*}}
// CHECK:   sol.call @{{.*double.*}}
// CHECK:   sol.call @{{.*add.*}}

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
