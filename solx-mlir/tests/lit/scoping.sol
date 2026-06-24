// RUN: solx --emit-mlir=sol %s | FileCheck %s --check-prefix=CHECK-SOLX
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s --check-prefix=CHECK-SOLC

// `default_return`: function declares `uint256 x = 42;` and falls off the end.
// solc stages the trailing zero through a return-slot alloca/store/load triple;
// solx returns the constant zero directly. Pin the actual zero return on both
// sides. solx walks functions alphabetically and solc in source order, so each
// backend's CHECK sequence follows its own function order.

// CHECK-SOLX: sol.func @{{.*default_return.*}}
// CHECK-SOLX:   sol.constant 42
// CHECK-SOLX:   %[[X_PTR:.*]] = sol.alloca : !sol.ptr<ui256, Stack>
// CHECK-SOLX:   sol.store %{{.*}}, %[[X_PTR]]
// CHECK-SOLX:   %[[ZERO:.*]] = sol.constant 0 : ui256
// CHECK-SOLX:   sol.return %[[ZERO]] : ui256
// CHECK-SOLX: sol.func @{{.*nested_scope.*}}
// CHECK-SOLX:   sol.alloca : !sol.ptr<ui256, Stack>
// CHECK-SOLX:   sol.alloca : !sol.ptr<ui256, Stack>

// CHECK-SOLC: sol.func @{{.*nested_scope.*}}
// CHECK-SOLC:   sol.alloca : !sol.ptr<ui256, Stack>
// CHECK-SOLC:   sol.alloca : !sol.ptr<ui256, Stack>
// CHECK-SOLC: sol.func @{{.*default_return.*}}
// CHECK-SOLC:   sol.constant 42
// CHECK-SOLC:   %[[X_PTR:.*]] = sol.alloca : !sol.ptr<ui256, Stack>
// CHECK-SOLC:   sol.store %{{.*}}, %[[X_PTR]]
// CHECK-SOLC:   %[[Z_PTR:.*]] = sol.alloca : !sol.ptr<ui256, Stack>
// CHECK-SOLC:   %[[ZERO:.*]] = sol.constant 0 : ui256
// CHECK-SOLC:   sol.store %[[ZERO]], %[[Z_PTR]]
// CHECK-SOLC:   %[[R:.*]] = sol.load %[[Z_PTR]]
// CHECK-SOLC:   sol.return %[[R]] : ui256

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
