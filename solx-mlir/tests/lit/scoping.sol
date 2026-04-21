// RUN: solx --emit-mlir %s | FileCheck %s

// CHECK: sol.func @"nested_scope()"
// CHECK:   sol.alloca : !sol.ptr<ui256, Stack>
// CHECK:   sol.alloca : !sol.ptr<ui256, Stack>

// CHECK: sol.func @"default_return()"
// CHECK:   sol.constant 42
// CHECK:   %[[ZERO:.*]] = sol.constant 0 : ui256
// CHECK:   sol.return %[[ZERO]] : ui256

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
