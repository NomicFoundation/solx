// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func {{.*}}pay_send{{.*}}-> i1
// CHECK:   %[[SR:.*]] = sol.address_cast %{{.*}} : !sol.address<payable> to !sol.address
// CHECK:   sol.send %[[SR]], %{{.*}} : !sol.address, ui256 -> i1

// CHECK: sol.func {{.*}}pay_transfer{{.*}}!sol.address<payable>{{.*}}ui256
// CHECK:   %[[TR:.*]] = sol.address_cast %{{.*}} : !sol.address<payable> to !sol.address
// CHECK:   sol.transfer %[[TR]], %{{.*}} : !sol.address, ui256

// CHECK: sol.func {{.*}}send_literal{{.*}}-> i1
// CHECK:   %[[SL:.*]] = sol.constant 1 : ui8
// CHECK:   %[[SA:.*]] = sol.cast %[[SL]] : ui8 to ui256
// CHECK:   sol.send %{{.*}}, %[[SA]] : !sol.address, ui256 -> i1

// CHECK: sol.func {{.*}}transfer_literal
// CHECK:   %[[TL:.*]] = sol.constant 2 : ui8
// CHECK:   %[[TA:.*]] = sol.cast %[[TL]] : ui8 to ui256
// CHECK:   sol.transfer %{{.*}}, %[[TA]] : !sol.address, ui256

contract C {
    function pay_send(address payable r, uint256 v) public returns (bool) { return r.send(v); }
    function pay_transfer(address payable r, uint256 v) public { r.transfer(v); }

    function send_literal(address payable r) public returns (bool) { return r.send(1 wei); }
    function transfer_literal(address payable r) public { r.transfer(2 wei); }
}
