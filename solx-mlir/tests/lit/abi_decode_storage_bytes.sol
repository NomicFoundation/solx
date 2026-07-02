// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: %[[P:.*]] = sol.addr_of @{{.*}} : !sol.string<Storage>
// CHECK: %[[M:.*]] = sol.data_loc_cast %[[P]] : !sol.string<Storage>, !sol.string<Memory>
// CHECK: sol.decode %[[M]] : !sol.string<Memory> -> ui256

contract C {
    bytes stored;

    function f() external returns (uint256) {
        return abi.decode(stored, (uint256));
    }
}
