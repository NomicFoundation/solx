// RUN: solx --emit-mlir=sol %s | FileCheck %s --check-prefixes=CHECK,CHECK-SOLX
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s --check-prefixes=CHECK,CHECK-SOLC

// CHECK: sol.func @{{.*nested_scope.*}}
// CHECK:   sol.alloca : !sol.ptr<ui256, Stack>
// CHECK:   sol.alloca : !sol.ptr<ui256, Stack>

// CHECK: sol.func @{{.*default_return.*}}
// CHECK:   sol.constant 42
// CHECK:   %[[X_PTR:.*]] = sol.alloca : !sol.ptr<ui256, Stack>
// CHECK:   sol.store %{{.*}}, %[[X_PTR]]
// CHECK-SOLC:   %[[Z_PTR:.*]] = sol.alloca : !sol.ptr<ui256, Stack>
// CHECK-SOLC:   %[[ZERO:.*]] = sol.constant 0 : ui256
// CHECK-SOLC:   sol.store %[[ZERO]], %[[Z_PTR]]
// CHECK-SOLC:   %[[R:.*]] = sol.load %[[Z_PTR]]
// CHECK-SOLC:   sol.return %[[R]] : ui256
// CHECK-SOLX:   %[[ZERO:.*]] = sol.constant 0 : ui256
// CHECK-SOLX:   sol.return %[[ZERO]] : ui256

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
