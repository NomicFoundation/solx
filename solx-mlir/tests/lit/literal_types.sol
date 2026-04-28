// RUN: solx --emit-mlir %s | FileCheck %s

// CHECK: sol.func @"address_literal()"
// CHECK:   sol.constant 255 : ui160
// CHECK:   sol.address_cast %{{.*}} : ui160 to !sol.address

// CHECK: sol.func @"neg_int8()"
// CHECK:   %[[POS:.*]] = sol.constant 1 : ui8
// CHECK:   %[[CAST:.*]] = sol.cast %[[POS]] : ui8 to si8
// CHECK:   %[[ZERO:.*]] = sol.constant 0 : si8
// CHECK:   sol.sub %[[ZERO]], %[[CAST]] : si8

// CHECK: sol.func @"ether_rational()"
// CHECK:   sol.constant 500000000000000000 : ui64

// CHECK: sol.func @"scientific()"
// CHECK:   sol.constant 1000000000000000000 : ui64

contract C {
    function address_literal() public pure returns (address) {
        return 0x00000000000000000000000000000000000000Ff;
    }

    function neg_int8() public pure returns (int8) {
        return -1;
    }

    function ether_rational() public pure returns (uint256) {
        return 0.5 ether;
    }

    function scientific() public pure returns (uint256) {
        return 1e18;
    }
}
