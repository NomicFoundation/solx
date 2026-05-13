// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*pair.*}}
// CHECK: sol.return %{{.*}}, %{{.*}}
// CHECK: sol.func @{{.*widen.*}}
// CHECK: sol.return %{{.*}}, %{{.*}} : ui256, i1

contract C {
    function pair() public pure returns (uint256, uint256) {
        return (3, 7);
    }

    function widen(uint8 a, bool b) public pure returns (uint256, bool) {
        return (a, b);
    }
}
