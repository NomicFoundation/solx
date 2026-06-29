// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*a_return.*}}
// CHECK: yul.gas
// CHECK: yul.return
// CHECK: sol.func @{{.*b_revert.*}}
// CHECK: yul.revert
// CHECK: sol.func @{{.*c_stop.*}}
// CHECK: yul.stop
// CHECK: sol.func @{{.*d_invalid.*}}
// CHECK: yul.invalid

contract C {
    function a_return() public { assembly { pop(gas()) return(0, 32) } }

    function b_revert() public { assembly { revert(0, 32) } }

    function c_stop() public { assembly { stop() } }

    function d_invalid() public { assembly { invalid() } }
}
