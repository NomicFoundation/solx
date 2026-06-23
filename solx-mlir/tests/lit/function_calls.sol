// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK-DAG: sol.func @{{.*add.*}}(%{{.*}}: ui256, %{{.*}}: ui256) -> ui256
// CHECK-DAG:   sol.cadd

// CHECK-DAG: sol.func @{{.*double.*}}
// CHECK-DAG:   sol.call @{{.*add.*}}(%{{.*}}, %{{.*}}) : (ui256, ui256) -> ui256

// CHECK-DAG: sol.func @{{.*chain.*}}
// CHECK-DAG:   sol.call @{{.*double.*}}
// CHECK-DAG:   sol.call @{{.*add.*}}

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
