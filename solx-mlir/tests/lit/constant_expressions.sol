// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*add.*}}() -> ui256
// CHECK:   %{{.*}} = sol.constant 5 : ui8
// CHECK:   %{{.*}} = sol.cast %{{.*}} : ui8 to ui256
// CHECK:   sol.return %{{.*}} : ui256

// CHECK: sol.func @{{.*rational.*}}() -> ui256
// CHECK:   %{{.*}} = sol.constant 6 : ui8
// CHECK:   %{{.*}} = sol.cast %{{.*}} : ui8 to ui256
// CHECK:   sol.return %{{.*}} : ui256

// CHECK: sol.func @{{.*signed_mod.*}}() -> si256
// CHECK:   %{{.*}} = sol.constant -2 : si8
// CHECK:   %{{.*}} = sol.cast %{{.*}} : si8 to si256
// CHECK:   sol.return %{{.*}} : si256

// CHECK: sol.func @{{.*full_word.*}}() -> ui256
// CHECK:   %[[MAX:.*]] = sol.constant 115792089237316195423570985008687907853269984665640564039457584007913129639935 : ui256
// CHECK:   sol.return %[[MAX]] : ui256

// CHECK: sol.func @{{.*shifted.*}}() -> ui256
// CHECK:   %[[SHIFT:.*]] = sol.constant 1809251394333065553493296640760748560207343510400633813116524750123642650624 : ui256
// CHECK:   sol.return %[[SHIFT]] : ui256

// CHECK: sol.func @{{.*complemented.*}}() -> si256
// CHECK:   %{{.*}} = sol.constant -6 : si8
// CHECK:   %{{.*}} = sol.cast %{{.*}} : si8 to si256
// CHECK:   sol.return %{{.*}} : si256

// CHECK: sol.func @{{.*denominated.*}}() -> ui256
// CHECK:   %{{.*}} = sol.constant 1000000000000000001 : ui64
// CHECK:   %{{.*}} = sol.cast %{{.*}} : ui64 to ui256
// CHECK:   sol.return %{{.*}} : ui256

// CHECK: sol.func @{{.*narrow.*}}() -> ui8
// CHECK:   %[[FULL:.*]] = sol.constant 255 : ui8
// CHECK:   sol.return %[[FULL]] : ui8

// CHECK: sol.func @{{.*lowest.*}}() -> si256
// CHECK:   %[[MIN:.*]] = sol.constant -57896044618658097711785492504343953926634992332820282019728792003956564819968 : si256
// CHECK:   sol.return %[[MIN]] : si256

// CHECK: sol.func @{{.*compared.*}}() -> i1
// CHECK:   sol.constant 6 : ui8
// CHECK:   sol.constant 6 : ui8
// CHECK:   %{{.*}} = sol.cmp ge, %{{.*}}, %{{.*}} : ui8
// CHECK:   sol.return %{{.*}} : i1

contract C {
    function add() public pure returns (uint256) {
        return 2 + 3;
    }

    function rational() public pure returns (uint256) {
        return 3 / 2 * 4;
    }

    function signed_mod() public pure returns (int256) {
        return -5 % 3;
    }

    function full_word() public pure returns (uint256) {
        return 2**256 - 1;
    }

    function shifted() public pure returns (uint256) {
        return 1 << 250;
    }

    function complemented() public pure returns (int256) {
        return ~5;
    }

    function denominated() public pure returns (uint256) {
        return 1 ether + 1 wei;
    }

    function narrow() public pure returns (uint8) {
        return 250 + 5;
    }

    function lowest() public pure returns (int256) {
        return -2**255;
    }

    function compared() public pure returns (bool) {
        return 3 / 2 * 4 >= 12 / 2;
    }
}
