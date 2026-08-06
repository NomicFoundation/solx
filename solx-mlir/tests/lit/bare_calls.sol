// RUN: solx --emit-mlir=sol %s | FileCheck %s

// solc print-init evaluates a call's options before its receiver, unlike legacy, so this is
// solx-only.

// CHECK: sol.func @{{.*bare_call.*}}
// CHECK:   sol.bare_call %{{.*}} gas %{{.*}} value %{{.*}} input %{{.*}} : !sol.address, ui256, ui256, !sol.string<Memory> -> i1, !sol.string<Memory>

// CHECK: sol.func @{{.*bare_delegate.*}}
// CHECK:   sol.bare_delegate_call %{{.*}} gas %{{.*}} input %{{.*}} : !sol.address, ui256, !sol.string<Memory> -> i1, !sol.string<Memory>

// CHECK: sol.func @{{.*bare_static.*}}
// CHECK:   sol.bare_static_call %{{.*}} gas %{{.*}} input %{{.*}} : !sol.address, ui256, !sol.string<Memory> -> i1, !sol.string<Memory>

// CHECK: sol.func @{{.*call_calldata.*}}
// CHECK:   sol.data_loc_cast %{{.*}} : !sol.string<CallData>, !sol.string<Memory>
// CHECK:   sol.bare_call

// CHECK: sol.func @{{.*call_storage.*}}
// CHECK:   sol.data_loc_cast %{{.*}} : !sol.string<Storage>, !sol.string<Memory>
// CHECK:   sol.bare_call

// CHECK: sol.func @{{.*call_value.*}}
// CHECK:   %[[RECEIVER:.*]] = sol.address_cast %{{.*}} : !sol.address<payable> to !sol.address
// CHECK:   %[[V:.*]] = sol.load %{{.*}} : !sol.ptr<ui256, Stack>, ui256
// CHECK:   %[[LEFT:.*]] = sol.gasleft
// CHECK:   sol.bare_call %[[RECEIVER]] gas %[[LEFT]] value %[[V]] input

// CHECK: sol.func @{{.*call_gas.*}}
// CHECK:   %[[RECEIVER:.*]] = sol.load %{{.*}} : !sol.ptr<!sol.address, Stack>, !sol.address
// CHECK:   %[[G:.*]] = sol.load %{{.*}} : !sol.ptr<ui256, Stack>, ui256
// CHECK:   %[[ZERO:.*]] = sol.constant 0 : ui256
// CHECK:   sol.bare_call %[[RECEIVER]] gas %[[G]] value %[[ZERO]] input

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

    function call_calldata(address a, bytes calldata d) public returns (bool) {
        (bool ok, ) = a.call(d);
        return ok;
    }

    function call_storage(address a) public returns (bool) {
        (bool ok, ) = a.call(data);
        return ok;
    }

    function call_value(address payable a, uint256 v, bytes memory d) public returns (bool) {
        (bool ok, ) = a.call{value: v}(d);
        return ok;
    }

    function call_gas(address a, uint256 g, bytes memory d) public returns (bool) {
        (bool ok, ) = a.call{gas: g}(d);
        return ok;
    }
}
