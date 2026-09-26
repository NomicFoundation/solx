// RUN: solx --emit-mlir=sol %s | FileCheck %s

// CHECK: sol.selfdestruct %{{.*}} : !sol.address<payable>

contract C {
    function destroy(address payable recipient) external {
        selfdestruct(recipient);
    }
}
