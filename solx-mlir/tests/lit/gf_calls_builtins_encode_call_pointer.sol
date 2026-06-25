// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// `abi.encodeCall(fp, (args))` over a runtime function-pointer callee reads the
// selector from the pointer via `sol.ext_func_selector` (no compile-time fold),
// then `sol.encode`s it ahead of the argument coerced to the pointer's
// parameter type.

// CHECK: sol.ext_func_selector %{{.*}} : !sol.ext_func_ref<(ui256) -> ()> -> !sol.fixedbytes<4>
// CHECK: sol.encode selector(%{{.*}}) %{{.*}} : !sol.fixedbytes<4> ui256 : !sol.string<Memory>

contract C {
    function viaPointer(function(uint256) external fp) public pure returns (bytes memory) {
        return abi.encodeCall(fp, (7));
    }
}
