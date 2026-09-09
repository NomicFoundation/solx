// RUN: solx --emit-mlir=sol %s | FileCheck %s

// solx defines an overridden body right after the function that first names it, where solc's
// print-init lists it with its own contract's functions, so this is solx-only.

// CHECK: sol.contract @{{.*}}Leaf
// CHECK: sol.func @{{.*}}f{{.*}}() -> ui256 attributes {id = {{.*}}, orig_fn_type = () -> ui256, selector
// CHECK:   sol.call @[[RIGHT:.*]]() : () -> ui256
// CHECK: sol.func @[[RIGHT]]() -> ui256 attributes {id = {{[0-9]+}} : i64, state_mutability
// CHECK:   sol.call @[[LEFT:.*]]() : () -> ui256
// CHECK: sol.func @[[LEFT]]() -> ui256 attributes {id = {{[0-9]+}} : i64, state_mutability
// CHECK:   sol.call @[[ROOT:.*]]() : () -> ui256
// CHECK: sol.func @[[ROOT]]() -> ui256 attributes {id = {{[0-9]+}} : i64, state_mutability
// CHECK: sol.func @{{.*}}pointer{{.*}}
// CHECK:   sol.func_constant @[[RIGHT]] : !sol.func_ref<() -> ui256>
// CHECK: sol.func @{{.*}}parenthesized{{.*}}
// CHECK:   sol.call @[[RIGHT]]() : () -> ui256
// CHECK: sol.func @{{.*}}qualified{{.*}}
// CHECK:   sol.func_constant @[[ROOT]] : !sol.func_ref<() -> ui256>

abstract contract Root {
    function f() public virtual returns (uint256) {
        return 1;
    }
}

abstract contract Left is Root {
    function f() public virtual override returns (uint256) {
        return super.f() + 10;
    }
}

abstract contract Unimplemented {
    function f() public virtual returns (uint256);
}

abstract contract Right is Root {
    function f() public virtual override returns (uint256) {
        return super.f() + 100;
    }
}

contract Leaf is Left, Unimplemented, Right {
    function f() public override(Left, Unimplemented, Right) returns (uint256) {
        return super.f() + 1000;
    }

    function pointer() public returns (uint256) {
        function() internal returns (uint256) next = super.f;
        return next();
    }

    function parenthesized() public returns (uint256) {
        return (super).f();
    }

    function qualified() public returns (uint256) {
        function() internal returns (uint256) root = Root.f;
        return root();
    }
}
