// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*nested_scope.*}}
// CHECK:   sol.constant 1 : ui8
// CHECK:   sol.store %{{.*}}, %[[X:[0-9]+]] : ui256, !sol.ptr<ui256, Stack>
// CHECK:   sol.constant 2 : ui8
// CHECK:   sol.store %{{.*}}, %[[Y:[0-9]+]] : ui256, !sol.ptr<ui256, Stack>
// CHECK:   sol.load %[[Y]] : !sol.ptr<ui256, Stack>, ui256
// CHECK:   sol.cadd
// CHECK:   sol.store %{{.*}}, %[[X]] : ui256, !sol.ptr<ui256, Stack>
// CHECK:   sol.load %[[X]] : !sol.ptr<ui256, Stack>, ui256
// CHECK:   sol.return

// CHECK: sol.func @{{.*default_return.*}}
// CHECK:   %[[ZERO:.*]] = sol.constant 0 : ui256
// CHECK:   sol.store %[[ZERO]], %[[Z_PTR:[0-9]+]] :
// CHECK:   %[[R:.*]] = sol.load %[[Z_PTR]] :
// CHECK:   sol.return %[[R]] : ui256

contract C {
    function nested_scope() public pure returns (uint256) {
        uint256 x = 1;
        {
            uint256 y = 2;
            x = x + y;
        }
        return x;
    }

    function default_return() public pure returns (uint256) {
        uint256 x = 42;
    }
}
