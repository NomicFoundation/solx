// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// Bare low-level calls without options: `addr.call(data)` forwards `gas` and a
// zero `value` then the memory input; `addr.delegatecall` / `addr.staticcall`
// drop the value operand. Each yields `(i1 status, bytes ret_data)`.

// CHECK-DAG: sol.bare_call %{{.*}} gas %{{.*}} value %{{.*}} input %{{.*}} : !sol.address, ui256, ui256, !sol.string<Memory> -> i1, !sol.string<Memory>
// CHECK-DAG: sol.bare_delegate_call %{{.*}} gas %{{.*}} input %{{.*}} : !sol.address, ui256, !sol.string<Memory> -> i1, !sol.string<Memory>
// CHECK-DAG: sol.bare_static_call %{{.*}} gas %{{.*}} input %{{.*}} : !sol.address, ui256, !sol.string<Memory> -> i1, !sol.string<Memory>

contract C {
    function bare_call(address a, bytes memory d) public returns (bool, bytes memory) {
        return a.call(d);
    }
    function bare_delegate(address a, bytes memory d) public returns (bool, bytes memory) {
        return a.delegatecall(d);
    }
    function bare_static(address a, bytes memory d) public view returns (bool, bytes memory) {
        return a.staticcall(d);
    }
}
