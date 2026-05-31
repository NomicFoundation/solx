// RUN: solx --emit-mlir=sol %s | FileCheck %s

// CHECK: sol.func @{{.*}}f
// CHECK: sol.new "B"

contract B {
    uint public x;
    constructor(uint v) {
        x = v;
    }
}

contract C {
    function f() public returns (address) {
        B b = new B(7);
        return address(b);
    }
}
