// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// `type(I).interfaceId` (EIP-165) folds to the XOR of the selectors of `I`'s
// directly-declared *functions*. The interface here also carries an `event` and an
// `error`; those non-function members are skipped when computing the id, so the
// result is the selector of the single function `ping()` alone. Both backends fold
// it to the same `bytes4` constant (a `ui32` bridged via `sol.bytes_cast`); only
// the symbol names differ.

// CHECK: sol.func @{{.*id.*}}() -> !sol.fixedbytes<4>
// CHECK:   sol.constant 1547088262 : ui32
// CHECK:   sol.bytes_cast %{{.*}} : ui32 to !sol.fixedbytes<4>
// CHECK:   sol.return

interface I {
    event Ping(uint256 x);
    error Bad(uint256 y);
    function ping() external returns (uint256);
}

contract C {
    function id() public pure returns (bytes4) {
        return type(I).interfaceId;
    }
}
