// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK-DAG: sol.func @{{.*plain_revert.*}}
// CHECK-DAG:   sol.revert ""

// CHECK-DAG: sol.func @{{.*message_revert.*}}
// CHECK-DAG:   sol.revert "oops"

// CHECK-DAG: sol.func @{{.*custom_error.*}}
// CHECK-DAG:   sol.revert "TooLow(uint256,uint256)" %{{.*}}, %{{.*}} : ui256, ui256 {call}

// CHECK-DAG: sol.func @{{.*custom_error_named.*}}
// CHECK-DAG:   %[[X:.*]] = sol.load
// CHECK-DAG:   sol.constant 100
// CHECK-DAG:   sol.revert "TooLow(uint256,uint256)" %[[X]], %{{.*}} : ui256, ui256 {call}

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
