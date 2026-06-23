// RUN: solx --emit-mlir=sol %s | FileCheck %s --check-prefix=CHECK-SOLX
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s --check-prefix=CHECK-SOLC

// `return -1;`: solc folds it to `sol.constant -1 : si8`, while solx materializes
// the negation as `1 : ui8` cast to `si8` then subtracted from `0 : si8`. Both
// backends walk functions in source order, so each CHECK sequence runs
// address_literal, neg_int8, ether_rational, scientific.

// CHECK-SOLX: sol.func @{{.*address_literal.*}}
// CHECK-SOLX:   sol.constant 255 : ui160
// CHECK-SOLX:   sol.address_cast %{{.*}} : ui160 to !sol.address
// CHECK-SOLX: sol.func @{{.*neg_int8.*}}
// CHECK-SOLX:   %[[ONE:.*]] = sol.constant 1 : ui8
// CHECK-SOLX:   %[[ONE_S:.*]] = sol.cast %[[ONE]] : ui8 to si8
// CHECK-SOLX:   %[[ZERO:.*]] = sol.constant 0 : si8
// CHECK-SOLX:   %[[N:.*]] = sol.sub %[[ZERO]], %[[ONE_S]] : si8
// CHECK-SOLX:   sol.return %[[N]] : si8
// CHECK-SOLX: sol.func @{{.*ether_rational.*}}
// CHECK-SOLX:   sol.constant 500000000000000000 : ui64
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
