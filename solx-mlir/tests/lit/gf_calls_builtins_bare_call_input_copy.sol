// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// `addr.call`'s input must be a memory buffer, so an argument sourced from
// storage or calldata is copied to memory with `sol.data_loc_cast` before the
// `sol.bare_call`. The two functions emit in different orders (solx
// alphabetical, solc source), so match each copy with CHECK-DAG.

// CHECK-DAG: sol.data_loc_cast %{{.*}} : !sol.string<Storage>, !sol.string<Memory>
// CHECK-DAG: sol.data_loc_cast %{{.*}} : !sol.string<CallData>, !sol.string<Memory>
// CHECK-DAG: sol.bare_call %{{.*}} gas %{{.*}} value %{{.*}} input %{{.*}} : !sol.address, ui256, ui256, !sol.string<Memory> -> i1, !sol.string<Memory>

contract C {
    bytes data;
    function callStored(address a) public returns (bool) {
        (bool ok, ) = a.call(data);
        return ok;
    }
    function callCalldata(address a, bytes calldata d) public returns (bool) {
        (bool ok, ) = a.call(d);
        return ok;
    }
}
