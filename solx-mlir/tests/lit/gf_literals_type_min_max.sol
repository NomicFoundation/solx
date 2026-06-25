// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// `type(uintN/intN).min/max` fold to the boundary constant at the integer's own
// type. Functions are in alphabetical order (maxi, maxu, mini, minu) so the
// solx alphabetical walk and the solc source-order walk agree.

// CHECK: sol.func @{{.*}}maxi
// CHECK:   sol.constant 57896044618658097711785492504343953926634992332820282019728792003956564819967 : si256
// CHECK: sol.func @{{.*}}maxu
// CHECK:   sol.constant 115792089237316195423570985008687907853269984665640564039457584007913129639935 : ui256
// CHECK: sol.func @{{.*}}mini
// CHECK:   sol.constant -128 : si8
// CHECK: sol.func @{{.*}}minu
// CHECK:   sol.constant 0 : ui8

contract C {
    function maxi() public pure returns (int256) {
        return type(int256).max;
    }
    function maxu() public pure returns (uint256) {
        return type(uint256).max;
    }
    function mini() public pure returns (int8) {
        return type(int8).min;
    }
    function minu() public pure returns (uint8) {
        return type(uint8).min;
    }
}
