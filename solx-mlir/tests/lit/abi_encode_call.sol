// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// `abi.encodeCall(C.f, (args))` folds the callee's selector to a compile-time
// `ui32` constant, casts it to `bytes4`, and `sol.encode`s it ahead of the
// arguments. The arguments are coerced to the callee's parameter types, so the
// integer literal `1` encodes at `ui256` (the declared width), not `ui8`.

// CHECK: sol.func @{{.*f.*}}
// CHECK: sol.constant {{[0-9]+}} : ui32
// CHECK: sol.bytes_cast %{{.*}} : ui32 to !sol.fixedbytes<4>
// CHECK: sol.encode selector(%{{[0-9]+}}) {{.*}} : !sol.fixedbytes<4> ui256, !sol.string<Memory>

// Over a runtime function-pointer callee the selector is read from the pointer
// via sol.ext_func_selector (no compile-time fold) before the sol.encode.
// CHECK-DAG: sol.ext_func_selector %{{.*}} : !sol.ext_func_ref<(ui256) -> ()> -> !sol.fixedbytes<4>
// CHECK-DAG: sol.encode selector(%{{.*}}) %{{.*}} : !sol.fixedbytes<4> ui256 : !sol.string<Memory>

contract C {
    function g(uint256 a, bytes memory b) public {}

    function f() public returns (bytes memory) {
        return abi.encodeCall(this.g, (1, "xy"));
    }

    function viaPointer(function(uint256) external fp) public pure returns (bytes memory) {
        return abi.encodeCall(fp, (7));
    }
}
