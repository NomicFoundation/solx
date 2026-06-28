// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// `new Created(v)` lowers to `sol.new` embedding the contract's deploy object,
// forwarding a zero `value` (a plain create) and the constructor argument
// coerced to its declared `ui256` parameter type. The contract symbol differs
// (solc appends a node id), so match it with a regex.

// CHECK: sol.new "{{.*Created.*}}" value = %{{.*}} ctor(%{{.*}} : ui256) : !sol.contract<"{{.*Created.*}}">

contract Created {
    uint256 public x;
    constructor(uint256 _x) { x = _x; }
}

contract C {
    function make(uint256 v) public returns (Created) {
        return new Created(v);
    }
}
