// RUN: solx --emit-mlir=sol %s | FileCheck %s

// solx-only: solc down-casts the payable operand to a plain `address` before send/transfer (an
// extra `sol.address_cast`), so it diverges here; solx keeps the operand payable.

// CHECK: sol.func {{.*}}pay_send{{.*}}-> i1
// CHECK:   sol.send {{.*}}, {{.*}} : !sol.address<payable>, ui256 -> i1
// CHECK: sol.func {{.*}}pay_transfer{{.*}}!sol.address<payable>{{.*}}ui256
// CHECK:   sol.transfer {{.*}}, {{.*}} : !sol.address<payable>, ui256

contract C {
    function pay_send(address payable r, uint256 v) public returns (bool) { return r.send(v); }
    function pay_transfer(address payable r, uint256 v) public { r.transfer(v); }
}
