// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*address_literal.*}}
// CHECK:   sol.constant 255 : ui160
// CHECK:   sol.address_cast %{{.*}} : ui160 to !sol.address

// solc folds `return -1;` directly to `sol.constant -1 : si8`; solx emits
// `constant 1 : ui8` -> `cast` -> `sub 0 - 1` (catalog item 9). Both yield -1.
// CHECK: sol.func @{{.*neg_int8.*}}
// CHECK:   sol.return %{{.*}} : si8

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
