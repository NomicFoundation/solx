// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// `**` takes an unsigned exponent of its own width alongside a possibly-signed
// base. The exponent must not be coerced to the (signed) result type the way a
// symmetric operator's operands are — `sol.cexp` requires operand #1 unsigned.

// CHECK: sol.func @{{.*pow.*}}
// CHECK: sol.cexp %{{[0-9]+}}, %{{[0-9a-z_]+}} : si256, ui8 -> si256

contract C {
    function pow(int256 b) public pure returns (int256) {
        return b ** 3;
    }
}
