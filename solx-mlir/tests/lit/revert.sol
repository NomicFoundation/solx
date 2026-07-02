// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*custom_error.*}}
// CHECK:   sol.revert "TooLow(uint256,uint256)" %{{.*}}, %{{.*}} : ui256, ui256 {call}

// CHECK: sol.func @{{.*custom_error_named.*}}
// CHECK:   %[[X:.*]] = sol.load
// CHECK:   sol.constant 100
// CHECK:   sol.revert "TooLow(uint256,uint256)" %[[X]], %{{.*}} : ui256, ui256 {call}

// CHECK: sol.func @{{.*empty_named_revert.*}}
// CHECK:   sol.revert ""

// CHECK: sol.func @{{.*empty_string_revert.*}}
// CHECK:   sol.revert ""

// CHECK: sol.func @{{.*message_revert.*}}
// CHECK:   sol.revert "oops"

// CHECK: sol.func @{{.*plain_revert.*}}
// CHECK:   sol.revert ""

contract C {
    error TooLow(uint256 supplied, uint256 minimum);

    function custom_error(uint256 x) public pure {
        revert TooLow(x, 100);
    }

    function custom_error_named(uint256 x) public pure {
        revert TooLow({minimum: 100, supplied: x});
    }

    function empty_named_revert(bool b) public pure {
        if (b) revert({});
    }

    function empty_string_revert() public pure {
        revert("");
    }

    function message_revert() public pure {
        revert("oops");
    }

    function plain_revert() public pure {
        revert();
    }
}
