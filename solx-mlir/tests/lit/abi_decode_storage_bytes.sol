// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// `abi.decode` requires a memory / calldata buffer, but a storage `bytes`
// payload is a reference. Decoding `abi.decode(storedBytes, (uint256))` first
// copies the storage `bytes` into memory via a `sol.data_loc_cast`
// (Storage -> Memory), then runs `sol.decode` on the memory buffer. Both
// backends emit the identical cast-then-decode sequence.

// CHECK: %[[P:.*]] = sol.addr_of @{{.*}} : !sol.string<Storage>
// CHECK: %[[M:.*]] = sol.data_loc_cast %[[P]] : !sol.string<Storage>, !sol.string<Memory>
// CHECK: sol.decode %[[M]] : !sol.string<Memory> -> ui256

contract C {
    bytes stored;

    function f() external returns (uint256) {
        return abi.decode(stored, (uint256));
    }
}
