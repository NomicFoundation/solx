// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*f.*}}
// CHECK:   sol.bytes_cast %{{.*}} : ui32 to !sol.fixedbytes<4>
// CHECK:   sol.encode selector(%{{.*}}) %{{.*}}, %{{.*}} : !sol.fixedbytes<4> ui256, !sol.string<Memory> : !sol.string<Memory>

// CHECK: sol.func @{{.*viaPointer.*}}
// CHECK:   sol.ext_func_selector %{{.*}} : !sol.ext_func_ref<(ui256) -> ()> -> !sol.fixedbytes<4>
// CHECK:   sol.encode selector(%{{.*}}) %{{.*}} : !sol.fixedbytes<4> ui256 : !sol.string<Memory>

contract C {
    function f() public returns (bytes memory) {
        return abi.encodeCall(this.g, (1, "xy"));
    }

    function g(uint256 a, bytes memory b) public {}

    function viaPointer(function(uint256) external fp) public pure returns (bytes memory) {
        return abi.encodeCall(fp, (7));
    }
}
