// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*thisbalance.*}}() -> ui256
// CHECK:   sol.this : !sol.contract<"C{{.*}}">
// CHECK:   sol.balance %{{.*}} : !sol.address -> ui256

contract C {
    function thisbalance() public view returns (uint256) { return address(this).balance; }
}
