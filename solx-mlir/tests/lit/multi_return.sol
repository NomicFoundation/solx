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
}
