// RUN: solx --emit-mlir=sol %s | FileCheck %s

// A library name as an abi.decode target type is valid (a library is its
// address): Type::resolve maps the library type to an address, so the decode
// yields an address. solc's MLIR backend types it as a contract, so solx-only.

// CHECK: sol.decode {{.*}} -> !sol.address

library L {
    function g() internal pure returns (uint256) {
        return 1;
    }
}

contract C {
    function f(bytes calldata d) external pure {
        abi.decode(d, (L));
    }
}
