// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK-DAG: sol.pop %{{.*}} : !sol.array<{{.*}}ui256, Storage>
// CHECK-DAG: sol.push %{{.*}} : !sol.array<{{.*}}ui256, Storage>

contract C {
    uint256[] arr;

    function popEmpty() external {
        arr.pop({});
    }

    function pushEmpty() external {
        arr.push({});
    }
}
