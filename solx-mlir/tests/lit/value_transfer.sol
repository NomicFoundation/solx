// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK-DAG: sol.func {{.*}}pay_send{{.*}}-> i1
// CHECK-DAG:   sol.send {{.*}}, {{.*}} : !sol.address, ui256 -> i1
// CHECK-DAG: sol.func {{.*}}pay_transfer{{.*}}!sol.address{{.*}}ui256
// CHECK-DAG:   sol.transfer {{.*}}, {{.*}} : !sol.address, ui256

// send / transfer take a ui256 amount, so a narrow literal argument is widened
// with sol.cast first.
// CHECK-DAG: sol.cast %{{.*}} : ui8 to ui256

contract C {
    function pay_send(address payable r, uint256 v) public returns (bool) { return r.send(v); }
    function pay_transfer(address payable r, uint256 v) public { r.transfer(v); }
    function send_literal(address payable r) public returns (bool) { return r.send(0); }
    function transfer_literal(address payable r) public { r.transfer(1); }
}
