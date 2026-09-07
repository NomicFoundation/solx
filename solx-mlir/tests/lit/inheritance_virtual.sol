// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.contract @{{.*}}Leaf
// CHECK: sol.func @[[OVERRIDE:.*]]() -> ui256 attributes {id = {{.*}}, orig_fn_type = () -> ui256, selector
// CHECK: sol.func @{{.*}}qualified{{.*}}
// CHECK:   sol.call @[[DECLARATION:.*]]() : () -> ui256
// CHECK: sol.func @[[DECLARATION]]() -> ui256 attributes {id = {{[0-9]+}} : i64, state_mutability
// CHECK: sol.func @{{.*}}caller{{.*}}
// CHECK:   sol.call @[[OVERRIDE]]() : () -> ui256
// CHECK:   sol.call @{{.*}}f{{.*}}(%{{.*}}) : (ui256) -> ui256

abstract contract Base {
    function f() public virtual returns (uint256) {
        return 1;
    }

    function f(uint256 x) public pure returns (uint256) {
        return x;
    }

    function caller() public returns (uint256) {
        uint256 dispatched = f();
        return dispatched + f(2);
    }
}

contract Leaf is Base {
    function f() public override returns (uint256) {
        return 2;
    }

    function qualified() public returns (uint256) {
        return Base.f();
    }
}
