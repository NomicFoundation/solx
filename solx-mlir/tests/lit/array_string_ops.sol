// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// Order-independent (solx walks functions alphabetically, solc in source order).
// `pushValue` appends and stores; `pushEmpty` appends a default and is the only
// other push, so the module carries exactly two pushes and a single store.
// CHECK-DAG: sol.push %{{.*}} : !sol.array<? x ui256, Storage> -> !sol.ptr<ui256, Storage>
// CHECK-DAG: sol.push %{{.*}} : !sol.array<? x ui256, Storage> -> !sol.ptr<ui256, Storage>
// CHECK-DAG: sol.store %{{.*}}, %{{.*}} : ui256, !sol.ptr<ui256, Storage>
// CHECK-DAG: sol.pop %{{.*}} : !sol.array<? x ui256, Storage>
// CHECK-DAG: sol.pop %{{.*}} : !sol.string<Storage>
// CHECK-DAG: sol.array_lit %{{.*}}, %{{.*}}, %{{.*}} : (ui256, ui256, ui256) -> !sol.array<3 x ui256, Memory>

contract C {
    uint256[] arr;
    bytes data;

    function pushValue(uint256 x) public {
        arr.push(x);
    }

    function pushEmpty() public {
        arr.push();
    }

    function popLast() public {
        arr.pop();
    }

    function popByte() public {
        data.pop();
    }

    function makeLiteral(uint256 a, uint256 b, uint256 c) public pure returns (uint256[3] memory) {
        return [a, b, c];
    }
}
