// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// A checksummed 20-byte address literal folds to a ui160 constant that is
// `sol.address_cast` to !sol.address; `address(0)` instead materialises a small
// ui8 zero widened through ui160. Functions are alphabetically ordered so the
// solx (alphabetical) and solc (source-order) walks agree.

// CHECK: sol.func @{{.*}}addr_checksummed
// CHECK:   sol.constant 471360049350540672339372329809862569580528312039 : ui160
// CHECK:   sol.address_cast %{{.*}} : ui160 to !sol.address
// CHECK: sol.func @{{.*}}addr_zero
// CHECK:   sol.constant 0 : ui8
// CHECK:   sol.cast %{{.*}} : ui8 to ui160
// CHECK:   sol.address_cast %{{.*}} : ui160 to !sol.address

contract C {
    function addr_checksummed() public pure returns (address) {
        return 0x52908400098527886E0F7030069857D2E4169EE7;
    }
    function addr_zero() public pure returns (address) {
        return address(0);
    }
}
