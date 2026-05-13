// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*}}fire{{.*}}(%arg0: !sol.address, %arg1: ui256)
// CHECK:   %[[CALLER:.*]] = sol.caller
// CHECK:   %[[TO:.*]] = sol.load
// CHECK:   %[[AMT:.*]] = sol.load
// CHECK:   sol.emit "Transfer(address,address,uint256)" indexed = [%[[CALLER]], %[[TO]]] non_indexed = [%[[AMT]]] : !sol.address, !sol.address, ui256

// CHECK: sol.func @{{.*}}fireAnon
// CHECK:   sol.emit non_indexed = [%{{.*}}] : ui256

contract C {
    event Transfer(address indexed from, address indexed to, uint256 value);
    event Anon(uint256 v) anonymous;

    function fire(address to, uint256 amount) public {
        emit Transfer(msg.sender, to, amount);
    }

    function fireAnon(uint256 v) public {
        emit Anon(v);
    }
}
