// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// type(intN).min/max fold to a compile-time integer constant of the named type.
// solx walks functions alphabetically, solc in source order; CHECK-DAG covers both.

// CHECK-DAG: sol.func @{{.*tmin.*}}() -> ui8
// CHECK-DAG:   sol.constant 0 : ui8
// CHECK-DAG: sol.func @{{.*tmax.*}}() -> ui8
// CHECK-DAG:   sol.constant 255 : ui8
// CHECK-DAG: sol.func @{{.*smin.*}}() -> si16
// CHECK-DAG:   sol.constant -32768 : si16
// CHECK-DAG: sol.func @{{.*smax.*}}() -> si16
// CHECK-DAG:   sol.constant 32767 : si16
// CHECK-DAG: sol.func @{{.*bigmax.*}}() -> ui256
// CHECK-DAG:   sol.constant 115792089237316195423570985008687907853269984665640564039457584007913129639935 : ui256

contract C {
    function tmin() public pure returns (uint8) { return type(uint8).min; }
    function tmax() public pure returns (uint8) { return type(uint8).max; }
    function smin() public pure returns (int16) { return type(int16).min; }
    function smax() public pure returns (int16) { return type(int16).max; }
    function bigmax() public pure returns (uint256) { return type(uint256).max; }
}
