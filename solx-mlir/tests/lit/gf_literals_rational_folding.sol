// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// Rational compile-time constant folding: division keeps full precision until a
// final integer is produced, so `1/3*3*3` folds to 3 and `3*(1/2+1/2)` to 3;
// `2**256-1` folds to the ui256 maximum; ether/gwei/wei unit literals sum at
// compile time. Functions are alphabetically ordered (frac, pow, rational_id,
// units) so the solx (alphabetical) and solc (source-order) walks agree.

// CHECK: sol.func @{{.*}}frac
// CHECK:   sol.constant 3 : ui8
// CHECK: sol.func @{{.*}}pow
// CHECK:   sol.constant 115792089237316195423570985008687907853269984665640564039457584007913129639935 : ui256
// CHECK: sol.func @{{.*}}rational_id
// CHECK:   sol.constant 3 : ui8
// CHECK: sol.func @{{.*}}units
// CHECK:   sol.constant 1000000001000000001 : ui64

contract C {
    function frac() public pure returns (uint256) {
        return 3 * (1 / 2 + 1 / 2);
    }
    function pow() public pure returns (uint256) {
        return 2 ** 256 - 1;
    }
    function rational_id() public pure returns (uint256) {
        return 1 / 3 * 3 * 3;
    }
    function units() public pure returns (uint256) {
        return 1 ether + 1 gwei + 1 wei;
    }
}
