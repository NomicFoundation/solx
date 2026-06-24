// RUN: solx --emit-mlir=sol %s | FileCheck %s --check-prefix=CHECK-SOLX
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s --check-prefix=CHECK-SOLC

// Both backends fold `return -1;` to `sol.constant -1 : si8`. The CHECK blocks
// are split only because solx walks functions alphabetically and solc in source
// order, so each backend's sequence follows its own function order.

// CHECK-SOLX: sol.func @{{.*address_literal.*}}
// CHECK-SOLX:   sol.constant 255 : ui160
// CHECK-SOLX:   sol.address_cast %{{.*}} : ui160 to !sol.address
// CHECK-SOLX: sol.func @{{.*ether_rational.*}}
// CHECK-SOLX:   sol.constant 500000000000000000 : ui64
// CHECK-SOLX: sol.func @{{.*neg_int8.*}}
// CHECK-SOLX:   %[[N:.*]] = sol.constant -1 : si8
// CHECK-SOLX:   sol.return %[[N]] : si8
// CHECK-SOLX: sol.func @{{.*scientific.*}}
// CHECK-SOLX:   sol.constant 1000000000000000000 : ui64

// CHECK-SOLC: sol.func @{{.*address_literal.*}}
// CHECK-SOLC:   sol.constant 255 : ui160
// CHECK-SOLC:   sol.address_cast %{{.*}} : ui160 to !sol.address
// CHECK-SOLC: sol.func @{{.*neg_int8.*}}
// CHECK-SOLC:   %[[N:.*]] = sol.constant -1 : si8
// CHECK-SOLC:   sol.return %[[N]] : si8
// CHECK-SOLC: sol.func @{{.*ether_rational.*}}
// CHECK-SOLC:   sol.constant 500000000000000000 : ui64
// CHECK-SOLC: sol.func @{{.*scientific.*}}
// CHECK-SOLC:   sol.constant 1000000000000000000 : ui64

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
