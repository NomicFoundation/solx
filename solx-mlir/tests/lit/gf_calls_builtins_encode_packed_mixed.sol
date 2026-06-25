// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// `abi.encodePacked` over mixed dynamic / value operands lowers to one
// `sol.encode {packed}` carrying every operand type in order.

// CHECK: sol.encode %{{.*}}, %{{.*}}, %{{.*}} :{{ +}}!sol.string<Memory>, ui8, !sol.string<Memory> : !sol.string<Memory> {packed}

contract C {
    function packed(string memory s, uint8 n, bytes memory b) public pure returns (bytes memory) {
        return abi.encodePacked(s, n, b);
    }
}
