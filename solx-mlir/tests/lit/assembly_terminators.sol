// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// Block-terminating Yul opcodes lower to their own Yul-dialect ops (rule 16):
// return, revert, stop, invalid. `pop(x)` evaluates and discards its argument,
// emitting no op of its own (here the discarded `gas()` shows up as `yul.gas`).
// Functions are named a_/b_/c_/d_ so solx's alphabetical walk and solc's
// source-order walk produce the same CHECK-LABEL sequence. `selfdestruct` is
// intentionally excluded: solx does not implement YulSelfdestruct (see
// divergences).

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
