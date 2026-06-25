// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// Negative decimal / hex literals fold to a signed constant at the literal's
// natural narrow type (widened by `sol.cast` to the return type); a negative
// literal equal to the type minimum stays at that exact width. Time-unit
// literals (days/hours/minutes/seconds/weeks) fold to their second count and
// sum at compile time. Functions are alphabetically ordered so the solx
// (alphabetical) and solc (source-order) walks agree.

// CHECK: sol.func @{{.*}}neg256
// CHECK:   sol.constant -42 : si8
// CHECK:   sol.cast %{{.*}} : si8 to si256
// CHECK: sol.func @{{.*}}neg_hex
// CHECK:   sol.constant -16 : si8
// CHECK:   sol.cast %{{.*}} : si8 to si256
// CHECK: sol.func @{{.*}}neg_min
// CHECK:   sol.constant -32768 : si16
// CHECK: sol.func @{{.*}}time_units
// CHECK:   sol.constant 694861 : ui24

contract C {
    function neg256() public pure returns (int256) {
        return -42;
    }
    function neg_hex() public pure returns (int256) {
        return -0x10;
    }
    function neg_min() public pure returns (int16) {
        return -32768;
    }
    function time_units() public pure returns (uint256) {
        return 1 days + 1 hours + 1 minutes + 1 seconds + 1 weeks;
    }
}
