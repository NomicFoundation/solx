// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func {{.*}}pay_send{{.*}}-> i1
// CHECK:   sol.send {{.*}}, {{.*}} : !sol.address, ui256 -> i1
// CHECK: sol.func {{.*}}pay_transfer{{.*}}!sol.address{{.*}}ui256
// CHECK:   sol.transfer {{.*}}, {{.*}} : !sol.address, ui256

contract C {
    function pay_send(address payable r, uint256 v) public returns (bool) { return r.send(v); }
    function pay_transfer(address payable r, uint256 v) public { r.transfer(v); }
}
