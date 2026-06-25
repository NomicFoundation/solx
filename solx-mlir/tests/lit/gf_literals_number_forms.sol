// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// Decimal underscores, scientific/exponent notation, and exponent-with-underscore
// all fold to the same narrow typed constant in both backends. Functions are
// declared in alphabetical order so the (alphabetical) solx walk and the
// (source-order) solc walk visit them in the same sequence.

// CHECK: sol.func @{{.*}}expnotation
// CHECK:   sol.constant 20000000000 : ui40
// CHECK: sol.func @{{.*}}expunderscore
// CHECK:   sol.constant 1000000 : ui24
// CHECK: sol.func @{{.*}}hexcaps
// CHECK:   sol.constant 255 : ui8
// CHECK: sol.func @{{.*}}underscores
// CHECK:   sol.constant 1000000 : ui24

contract C {
    function expnotation() public pure returns (uint256) {
        return 2e10;
    }
    function expunderscore() public pure returns (uint256) {
        return 1_000e3;
    }
    function hexcaps() public pure returns (uint256) {
        return 0xFF;
    }
    function underscores() public pure returns (uint256) {
        return 1_000_000;
    }
}
