// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*}}fire{{.*}}(%arg0: !sol.address, %arg1: ui256)
// CHECK:   %[[CALLER:.*]] = sol.caller
// CHECK:   %[[TO:.*]] = sol.load
// CHECK:   %[[AMT:.*]] = sol.load
// CHECK:   sol.emit "Transfer(address,address,uint256)" indexed = [%[[CALLER]], %[[TO]]] non_indexed = [%[[AMT]]] : !sol.address, !sol.address, ui256

// CHECK: sol.func @{{.*}}fireAnonymous
// CHECK:   sol.emit non_indexed = [%{{.*}}] : ui256

// CHECK: sol.func @{{.*}}fireReference
// CHECK:   sol.emit "Reference(string,uint256[],bytes,uint256)" indexed = [%{{.*}}, %{{.*}}, %{{.*}}] non_indexed = [%{{.*}}] : !sol.string<Memory>, !sol.array<? x ui256, Memory>, !sol.string<Memory>, ui256

contract C {
    event Transfer(address indexed from, address indexed to, uint256 value);
    event Anonymous(uint256 v) anonymous;
    event Reference(string indexed s, uint256[] indexed array, bytes indexed b, uint256 v);

    function fire(address to, uint256 amount) public {
        emit Transfer(msg.sender, to, amount);
    }

    function fireAnonymous(uint256 v) public {
        emit Anonymous(v);
    }

    function fireReference(string calldata s, uint256[] calldata array, bytes calldata b) public {
        emit Reference(s, array, b, 42);
    }
}
