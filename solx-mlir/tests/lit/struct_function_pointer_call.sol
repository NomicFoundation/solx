// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// Calling through a function-pointer struct field (`s.f()`) is classified by the
// member's function type (a `StructMember` definition that is `Function`-typed),
// then lowered through the indirect-call path: `sol.gep` + `sol.load` of the
// `func_ref` field, then `sol.icall` (`sol.ext_icall` for an external field).

// CHECK: sol.func @{{.*run.*}}
// CHECK: sol.gep %{{[0-9]+}}{{.*}} : !sol.struct<(!sol.func_ref<() -> ui256>), Storage>, {{.*}}!sol.ptr<!sol.func_ref<() -> ui256>, Storage>
// CHECK: sol.load %{{[0-9]+}} : !sol.ptr<!sol.func_ref<() -> ui256>, Storage>, !sol.func_ref<() -> ui256>
// CHECK: sol.icall %{{[0-9]+}}() : !sol.func_ref<() -> ui256>, () -> ui256

contract C {
    struct S {
        function () internal returns (uint256) f;
    }

    S s;

    function g() internal returns (uint256) {
        return 42;
    }

    function set() public {
        s.f = g;
    }

    function run() public returns (uint256) {
        return s.f();
    }
}
