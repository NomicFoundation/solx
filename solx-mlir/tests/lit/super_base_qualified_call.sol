// RUN: solx --emit-mlir=sol %s | FileCheck %s --check-prefix=CHECK-SOLX
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s --check-prefix=CHECK-SOLC

// An explicit base-qualified internal call `Base.foo()` (not `super.foo()`) names
// Base's own implementation, bypassing the most-derived `override`. In @Derived,
// `call()` issues two direct `sol.call`s: one to the override `foo` and one to the
// base implementation, which is copied into the concrete contract under its own
// symbol. Both backends lower this to two calls plus a `sol.cadd`, and emit both
// `foo` bodies (constant 1 for Base's, constant 2 for the override). The structure
// is identical; only symbol names (solc node-id suffix vs. solx qualified name) and
// the order in which the three functions are emitted differ, so the prefixes are
// split purely to follow each backend's emission order while pinning the same ops.

// CHECK-SOLX: sol.contract @{{.*Derived.*}}
// CHECK-SOLX: sol.func @{{.*call.*}}() -> ui256
// CHECK-SOLX-DAG:   sol.call @{{.*foo.*}}() : () -> ui256
// CHECK-SOLX-DAG:   sol.call @{{.*[Bb]ase.*foo.*}}() : () -> ui256
// CHECK-SOLX:   sol.cadd
// CHECK-SOLX:   sol.return
// CHECK-SOLX:   sol.constant 2 : ui8
// CHECK-SOLX:   sol.constant 1 : ui8

// CHECK-SOLC: sol.contract @{{.*Derived.*}}
// CHECK-SOLC:   sol.constant 2 : ui8
// CHECK-SOLC: sol.func @{{.*call.*}}() -> ui256
// CHECK-SOLC-DAG:   sol.call @{{.*foo.*}}() : () -> ui256
// CHECK-SOLC-DAG:   sol.call @{{.*foo.*}}() : () -> ui256
// CHECK-SOLC:   sol.cadd
// CHECK-SOLC:   sol.return
// CHECK-SOLC:   sol.constant 1 : ui8

contract Base {
    function foo() internal pure virtual returns (uint256) {
        return 1;
    }
}

contract Derived is Base {
    function foo() internal pure override returns (uint256) {
        return 2;
    }

    function call() public pure returns (uint256) {
        return Base.foo() + foo();
    }
}
