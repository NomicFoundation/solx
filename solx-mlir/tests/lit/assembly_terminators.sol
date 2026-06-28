// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*a_ret.*}}
// CHECK: yul.gas
// CHECK: yul.return
// CHECK: sol.func @{{.*b_rev.*}}
// CHECK: yul.revert
// CHECK: sol.func @{{.*c_stop.*}}
// CHECK: yul.stop
// CHECK: sol.func @{{.*d_inv.*}}
// CHECK: yul.invalid

contract C {
    function a_ret() public { assembly { pop(gas()) return(0, 32) } }
    function b_rev() public { assembly { revert(0, 32) } }
    function c_stop() public { assembly { stop() } }
    function d_inv() public { assembly { invalid() } }
}
