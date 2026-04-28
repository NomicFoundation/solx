// RUN: solx --emit-mlir %s | FileCheck %s

// CHECK: sol.func @"plain_revert()"
// CHECK:   sol.revert ""

// CHECK: sol.func @"message_revert()"
// CHECK:   sol.revert "oops"

// CHECK: sol.func @"custom_error(uint256)"
// CHECK:   sol.revert "TooLow(uint256,uint256)" %{{.*}}, %{{.*}} : ui256, {{.*}} {call}

// CHECK: sol.func @"custom_error_named(uint256)"
// CHECK:   %[[X:.*]] = sol.load
// CHECK:   %[[C:.*]] = sol.constant 100
// CHECK:   sol.revert "TooLow(uint256,uint256)" %[[X]], %[[C]] : ui256, {{.*}} {call}

contract C {
    error TooLow(uint256 supplied, uint256 minimum);

    function plain_revert() public pure {
        revert();
    }

    function message_revert() public pure {
        revert("oops");
    }

    function custom_error(uint256 x) public pure {
        revert TooLow(x, 100);
    }

    function custom_error_named(uint256 x) public pure {
        revert TooLow({minimum: 100, supplied: x});
    }
}
