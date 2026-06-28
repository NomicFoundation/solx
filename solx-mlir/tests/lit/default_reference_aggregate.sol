// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*calldata_ret.*}}
// CHECK:   sol.default_calldata : !sol.array<? x ui256, CallData>
// CHECK: sol.func @{{.*storage_ret.*}}
// CHECK:   sol.default_storage : !sol.array<? x ui256, Storage>

contract C {
    uint256[] s;

    function calldata_ret(uint256[] calldata a) internal pure returns (uint256[] calldata r) {
        r = a;
        return r;
    }

    function storage_ret() internal returns (uint256[] storage r) {
        r = s;
        return r;
    }
}
