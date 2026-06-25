// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// `abi.decode(payload, (T1, T2, T3))` lowers to a single `sol.decode` over the
// memory payload, producing one result per requested type in order.

// CHECK: sol.decode %{{.*}} : !sol.string<Memory> -> ui256, i1, !sol.fixedbytes<32>

contract C {
    function dm(bytes memory d) public pure returns (uint256, bool, bytes32) {
        return abi.decode(d, (uint256, bool, bytes32));
    }
}
