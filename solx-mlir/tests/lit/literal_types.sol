// RUN: solx --emit-mlir=sol %s | FileCheck %s --check-prefixes=CHECK,CHECK-SOLX
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s --check-prefixes=CHECK,CHECK-SOLC

// CHECK: sol.func @{{.*address_literal.*}}
// CHECK:   sol.constant 255 : ui160
// CHECK:   sol.address_cast %{{.*}} : ui160 to !sol.address

// solc folds `return -1;` directly to `sol.constant -1 : si8`; solx
// materializes `1 : ui8` -> cast to si8 -> `sub 0 - 1` (catalog item 9).
// Pin both shapes so a regression on either side fails.
// CHECK: sol.func @{{.*neg_int8.*}}
// CHECK-SOLC:   %[[N:.*]] = sol.constant -1 : si8
// CHECK-SOLC:   sol.return %[[N]] : si8
// CHECK-SOLX:   %[[ONE:.*]] = sol.constant 1 : ui8
// CHECK-SOLX:   %[[ONE_S:.*]] = sol.cast %[[ONE]] : ui8 to si8
// CHECK-SOLX:   %[[ZERO:.*]] = sol.constant 0 : si8
// CHECK-SOLX:   %[[N:.*]] = sol.sub %[[ZERO]], %[[ONE_S]] : si8
// CHECK-SOLX:   sol.return %[[N]] : si8

// CHECK: sol.func @{{.*ether_rational.*}}
// CHECK:   sol.constant 500000000000000000 : ui64

// CHECK: sol.func @{{.*scientific.*}}
// CHECK:   sol.constant 1000000000000000000 : ui64

contract C {
    function address_literal() public pure returns (address) {
        return 0x00000000000000000000000000000000000000ff;
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
