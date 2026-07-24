// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.selfdestruct %{{.*}} : !sol.address<payable>

contract C {
    function destroy(address payable recipient) external {
        selfdestruct(recipient);
    }
}
