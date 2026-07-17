// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*single_element_tuple.*}}
// CHECK:   sol.constant 2004384122 : ui32
// CHECK:   sol.bytes_cast %{{.*}} : ui32 to !sol.fixedbytes<4>

// CHECK: sol.func @{{.*two_constants.*}}
// CHECK: sol.return %{{.*}}, %{{.*}}

// CHECK: sol.func @{{.*widened_elements.*}}
// CHECK: sol.return %{{.*}}, %{{.*}} : ui256, i1

// CHECK: sol.func @{{.*string_and_constant.*}}
// CHECK:   sol.constant 1633837924 : ui32
// CHECK:   sol.bytes_cast %{{.*}} : ui32 to !sol.fixedbytes<4>
// CHECK:   sol.return %{{.*}}, %{{.*}} : !sol.fixedbytes<4>, ui256

// CHECK: sol.func @{{.*string_and_variable.*}}
// CHECK:   sol.constant 1633837924 : ui32
// CHECK:   sol.bytes_cast %{{.*}} : ui32 to !sol.fixedbytes<4>
// CHECK:   sol.return %{{.*}}, %{{.*}} : !sol.fixedbytes<4>, ui256

// CHECK: sol.func @{{.*via_call.*}}
// CHECK: %[[R:[0-9]+]]:2 = sol.call @{{.*two_constants.*}}() : () -> (ui256, ui256)
// CHECK: sol.return %[[R]]#0, %[[R]]#1 : ui256, ui256

// CHECK: sol.func @{{.*via_conditional.*}}
// CHECK: sol.if
// CHECK: sol.return %{{.*}}, %{{.*}} : ui256, ui256

// CHECK: sol.func @{{.*via_conditional_call.*}}
// CHECK: sol.if
// CHECK: sol.return %{{.*}}, %{{.*}} : ui256, ui256

// CHECK: sol.func @{{.*via_nested_conditional.*}}
// CHECK: sol.if
// CHECK: sol.return %{{.*}}, %{{.*}} : ui256, ui256

contract C {
    function single_element_tuple() public pure returns (bytes4) {
        return ("wxyz");
    }

    function two_constants() public pure returns (uint256, uint256) {
        return (3, 7);
    }

    function widened_elements(uint8 a, bool b) public pure returns (uint256, bool) {
        return (a, b);
    }

    function string_and_constant() public pure returns (bytes4, uint256) {
        return ("abcd", 42);
    }

    function string_and_variable(uint256 x) public pure returns (bytes4, uint256) {
        return ("abcd", x);
    }

    function via_call() public pure returns (uint256, uint256) {
        return two_constants();
    }

    function via_conditional(bool c) public pure returns (uint256, uint256) {
        return c ? (1, 2) : (3, 4);
    }

    function via_conditional_call(bool c) public pure returns (uint256, uint256) {
        return c ? two_constants() : two_constants();
    }

    function via_nested_conditional(bool a, bool b) public pure returns (uint256, uint256) {
        return a ? (1, 2) : b ? (3, 4) : (5, 6);
    }
}
