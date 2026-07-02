// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.new "{{.*Created.*}}" value = %{{.*}} ctor(%{{.*}} : ui256) : !sol.contract<"{{.*Created.*}}">

contract C {
    function make(uint256 v) public returns (Created) {
        return new Created(v);
    }
}

contract Created {
    uint256 public x;

    constructor(uint256 _x) { x = _x; }
}
