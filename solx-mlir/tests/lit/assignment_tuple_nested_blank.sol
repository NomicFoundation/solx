// RUN: solx --emit-mlir=sol %s | FileCheck %s

// CHECK: sol.func @{{.*blank.*}}
// CHECK: sol.constant 7 : ui8
// CHECK: sol.constant 8 : ui8
// CHECK: %[[B:.*]] = sol.cast %{{.*}} : ui8 to ui256
// CHECK: sol.store %[[B]], %{{[0-9]+}} : ui256, !sol.ptr<ui256, Stack>

// CHECK: sol.func @{{.*nested.*}}
// CHECK-DAG: %[[V1:.*]] = sol.cast %{{.*}} : ui8 to ui256
// CHECK-DAG: %[[V2:.*]] = sol.cast %{{.*}} : ui8 to ui256
// CHECK-DAG: %[[V3:.*]] = sol.cast %{{.*}} : ui8 to ui256
// CHECK-DAG: sol.store %[[V1]], %{{[0-9]+}} : ui256, !sol.ptr<ui256, Stack>
// CHECK-DAG: sol.store %[[V2]], %{{[0-9]+}} : ui256, !sol.ptr<ui256, Stack>
// CHECK-DAG: sol.store %[[V3]], %{{[0-9]+}} : ui256, !sol.ptr<ui256, Stack>

contract C {
    function nested() public pure returns (uint256, uint256, uint256) {
        uint256 a; uint256 b; uint256 c;
        ((a, b), c) = ((1, 2), 3);
        return (a, b, c);
    }

    function blank() public pure returns (uint256) {
        uint256 a;
        (a, ) = (7, 8);
        return a;
    }
}
