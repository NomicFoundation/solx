// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*makeLiteral.*}}
// CHECK:   sol.array_lit %{{.*}}, %{{.*}}, %{{.*}} : (ui256, ui256, ui256) -> !sol.array<3 x ui256, Memory>

// CHECK: sol.func @{{.*popByte.*}}
// CHECK:   sol.pop %{{.*}} : !sol.string<Storage>

// CHECK: sol.func @{{.*popLast.*}}
// CHECK:   sol.pop %{{.*}} : !sol.array<? x ui256, Storage>

// CHECK: sol.func @{{.*pushEmpty.*}}
// CHECK:   sol.push %{{.*}} : !sol.array<? x ui256, Storage> -> !sol.ptr<ui256, Storage>

// CHECK: sol.func @{{.*pushValue.*}}
// CHECK:   sol.push %{{.*}} : !sol.array<? x ui256, Storage> -> !sol.ptr<ui256, Storage>
// CHECK:   sol.store %{{.*}}, %{{.*}} : ui256, !sol.ptr<ui256, Storage>

contract C {
    uint256[] arr;
    bytes data;

    function makeLiteral(uint256 a, uint256 b, uint256 c) public pure returns (uint256[3] memory) {
        return [a, b, c];
    }

    function popByte() public {
        data.pop();
    }

    function popLast() public {
        arr.pop();
    }

    function pushEmpty() public {
        arr.push();
    }

    function pushValue(uint256 x) public {
        arr.push(x);
    }
}
