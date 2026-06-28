// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// A named return of a reference-typed aggregate is default-initialized at entry:
// a storage/transient aggregate to sol.default_storage, a calldata aggregate to
// sol.default_calldata (a memory aggregate uses sol.malloc zero_init).

// CHECK-DAG: sol.func @{{.*storage_ret.*}}
// CHECK-DAG:   sol.default_storage : !sol.array<? x ui256, Storage>
// CHECK-DAG: sol.func @{{.*calldata_ret.*}}
// CHECK-DAG:   sol.default_calldata : !sol.array<? x ui256, CallData>

contract C {
    uint256[] s;

    function storage_ret() internal returns (uint256[] storage r) {
        r = s;
        return r;
    }

    function calldata_ret(uint256[] calldata a) internal pure returns (uint256[] calldata r) {
        r = a;
        return r;
    }
}
