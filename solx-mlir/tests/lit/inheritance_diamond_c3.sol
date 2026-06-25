// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// Diamond inheritance D is B, C; B is A; C is A. Each level overrides `who` and
// chains through `super.who()`. The C3 linearization of D is D -> C -> B -> A, so
// the most-derived contract @D must emit its four `who` bodies in that exact order,
// with each body's `super` lowering to a direct `sol.call` of the next link in the
// chain. Both backends agree on the linearization; only symbol names differ
// (solc `who_<id>`, solx `<Contract>.who()`), matched via regex. The constant vs.
// call ordering within a body is backend-dependent, pinned with CHECK-DAG.

// CHECK: sol.contract @{{.*D.*}}
// D's own override (the only one carrying a public `selector`) -> calls C.who
// CHECK: sol.func @{{.*who.*}}() -> ui256 attributes {{.*}}selector = -690872835
// CHECK-DAG:   sol.constant 1000 : ui16
// CHECK-DAG:   sol.call @{{.*who.*}}() : () -> ui256
// CHECK:   sol.return
// C.who -> calls B.who
// CHECK: sol.func @{{.*who.*}}() -> ui256
// CHECK-DAG:   sol.constant 100 : ui8
// CHECK-DAG:   sol.call @{{.*who.*}}() : () -> ui256
// CHECK:   sol.return
// B.who -> calls A.who
// CHECK: sol.func @{{.*who.*}}() -> ui256
// CHECK-DAG:   sol.constant 10 : ui8
// CHECK-DAG:   sol.call @{{.*who.*}}() : () -> ui256
// CHECK:   sol.return
// A.who -> leaf, returns 1
// CHECK: sol.func @{{.*who.*}}() -> ui256
// CHECK:   sol.constant 1 : ui8
// CHECK-NOT: sol.call
// CHECK:   sol.return

contract A {
    function who() public pure virtual returns (uint256) {
        return 1;
    }
}

contract B is A {
    function who() public pure virtual override returns (uint256) {
        return super.who() + 10;
    }
}

contract C is A {
    function who() public pure virtual override returns (uint256) {
        return super.who() + 100;
    }
}

contract D is B, C {
    function who() public pure override(B, C) returns (uint256) {
        return super.who() + 1000;
    }
}
