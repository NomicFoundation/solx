// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// Assignment of a whole struct `s = m` from a memory parameter into a storage
// state variable (the ReferenceCopy assignment target): the destination is the
// storage reference, the RHS its memory contents, and the deep copy is a single
// `sol.copy` Memory -> Storage. Both backends emit the identical addr_of / load /
// copy shape.

// CHECK: sol.func @{{.*f.*}}(%arg0: !sol.struct<(ui256, ui256), Memory>)
// CHECK:   %[[DST:.*]] = sol.addr_of @{{.*s.*}} : !sol.struct<(ui256, ui256), Storage>
// CHECK:   %[[SRC:.*]] = sol.load %{{.*}} : !sol.ptr<!sol.struct<(ui256, ui256), Memory>, Stack>, !sol.struct<(ui256, ui256), Memory>
// CHECK:   sol.copy %[[SRC]], %[[DST]] : !sol.struct<(ui256, ui256), Memory>, !sol.struct<(ui256, ui256), Storage>

contract C {
    struct S { uint256 a; uint256 b; }
    S s;
    function f(S memory m) public { s = m; }
}
