// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.contract @{{.*Derived.*}}
// CHECK:   sol.state_var @{{.*stored.*}} slot 0 offset 0 : ui256
// CHECK:   sol.func @{{.*compute.*}}(%{{.*}}: ui256) -> ui256
// CHECK-DAG:     sol.call @{{.*helper.*}}(%{{.*}}) : (ui256) -> ui256
// CHECK-DAG:     sol.load %{{.*}} : !sol.ptr<ui256, Storage>, ui256
// CHECK:     sol.cadd
// CHECK:     sol.return
// CHECK:   sol.func @{{.*helper.*}}(%{{.*}}: ui256) -> ui256
// CHECK:     sol.cadd
// CHECK:     sol.return

contract Base {
    uint256 internal stored;

    function helper(uint256 a) internal pure returns (uint256) {
        return a + a;
    }
}

contract Derived is Base {
    function compute(uint256 x) public view returns (uint256) {
        return helper(x) + stored;
    }
}
