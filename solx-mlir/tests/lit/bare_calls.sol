// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK-DAG: sol.bare_call %{{.*}} gas %{{.*}} value %{{.*}} input %{{.*}} : !sol.address, ui256, ui256, !sol.string<Memory> -> i1, !sol.string<Memory>
// CHECK-DAG: sol.bare_delegate_call %{{.*}} gas %{{.*}} input %{{.*}} : !sol.address, ui256, !sol.string<Memory> -> i1, !sol.string<Memory>
// CHECK-DAG: sol.bare_static_call %{{.*}} gas %{{.*}} input %{{.*}} : !sol.address, ui256, !sol.string<Memory> -> i1, !sol.string<Memory>
// CHECK-DAG: sol.data_loc_cast %{{.*}} : !sol.string<Storage>, !sol.string<Memory>
// CHECK-DAG: sol.data_loc_cast %{{.*}} : !sol.string<CallData>, !sol.string<Memory>

contract C {
    bytes data;

    function bare_call(address a, bytes memory d) public returns (bool, bytes memory) {
        return a.call(d);
    }

    function bare_delegate(address a, bytes memory d) public returns (bool, bytes memory) {
        return a.delegatecall(d);
    }

    function bare_static(address a, bytes memory d) public view returns (bool, bytes memory) {
        return a.staticcall(d);
    }

    function call_storage(address a) public returns (bool) {
        (bool ok, ) = a.call(data);
        return ok;
    }

    function call_calldata(address a, bytes calldata d) public returns (bool) {
        (bool ok, ) = a.call(d);
        return ok;
    }
}
