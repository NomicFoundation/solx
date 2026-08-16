// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*qualified_calldata_argument.*}}
// CHECK:   sol.ext_call "{{.*look.*}}"(%{{.*}}) at %{{.*}} gas %{{.*}} value %c0_ui256 selector %c1335285351_ui256 {callee_type = (!sol.string<Memory>) -> ui256, delegate_call, library_call, static_call} : !sol.address, (!sol.string<CallData>) -> (i1, ui256)

// CHECK: sol.func @{{.*qualified_calldata_return.*}}
// CHECK:   sol.ext_call "{{.*tail.*}}"(%{{.*}}) at %{{.*}} gas %{{.*}} value %c0_ui256 selector %c3126462307_ui256 {callee_type = (!sol.string<Memory>) -> !sol.string<Memory>, delegate_call, library_call, static_call} : !sol.address, (!sol.string<CallData>) -> (i1, !sol.string<Memory>)

// CHECK: sol.func @{{.*qualified_storage.*}}
// CHECK:   %[[SLOT:.*]] = sol.addr_of @{{.*stored.*}} : !sol.string<Storage>
// CHECK:   sol.ext_call "{{.*keep.*}}"(%[[SLOT]]) at %{{.*}} gas %{{.*}} value %c0_ui256 selector %c4077198112_ui256 {callee_type = (!sol.string<Storage>) -> ui256, delegate_call, library_call, static_call} : !sol.address, (!sol.string<Storage>) -> (i1, ui256)

// CHECK: sol.func @{{.*attached_calldata_receiver.*}}
// CHECK:   sol.ext_call "{{.*look.*}}"(%{{.*}}) at %{{.*}} gas %{{.*}} value %c0_ui256 selector %c1335285351_ui256 {callee_type = (!sol.string<CallData>) -> ui256, delegate_call, library_call, static_call} : !sol.address, (!sol.string<CallData>) -> (i1, ui256)

// CHECK: sol.func @{{.*attached_calldata_result.*}}
// CHECK:   sol.ext_call "{{.*tail.*}}"(%{{.*}}) at %{{.*}} gas %{{.*}} value %c0_ui256 selector %c3126462307_ui256 {callee_type = (!sol.string<CallData>) -> !sol.string<CallData>, delegate_call, library_call, static_call} : !sol.address, (!sol.string<CallData>) -> (i1, !sol.string<Memory>)

// CHECK: sol.func @{{.*attached_storage_receiver.*}}
// CHECK:   %[[SLOT:.*]] = sol.addr_of @{{.*stored.*}} : !sol.string<Storage>
// CHECK:   sol.ext_call "{{.*keep.*}}"(%[[SLOT]]) at %{{.*}} gas %{{.*}} value %c0_ui256 selector %c4077198112_ui256 {callee_type = (!sol.string<Storage>) -> ui256, delegate_call, library_call, static_call} : !sol.address, (!sol.string<Storage>) -> (i1, ui256)

library Lib {
    function keep(bytes storage b) external view returns (uint256) {
        return b.length;
    }

    function look(bytes calldata b) external pure returns (uint256) {
        return b.length;
    }

    function tail(bytes calldata b) external pure returns (bytes calldata) {
        return b[1:];
    }
}

contract User {
    using Lib for bytes;

    bytes stored;

    function qualified_calldata_argument(bytes calldata b) public pure returns (uint256) {
        return Lib.look(b);
    }

    function qualified_calldata_return(bytes calldata b) public pure returns (bytes memory) {
        return Lib.tail(b);
    }

    function qualified_storage() public view returns (uint256) {
        return Lib.keep(stored);
    }

    function attached_calldata_receiver(bytes calldata b) public pure returns (uint256) {
        return b.look();
    }

    function attached_calldata_result(bytes calldata b) public pure returns (bytes memory) {
        return b.tail();
    }

    function attached_storage_receiver() public view returns (uint256) {
        return stored.keep();
    }
}
