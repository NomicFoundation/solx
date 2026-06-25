// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// Auto-generated getter for a mapping whose KEY is a reference type (`string`).
// The key is decoded into a Memory reference and passed as the getter argument
// (`!sol.string<Memory>`), exercising the reference-typed-key arm. Both backends
// emit the identical signature, selector, and addr_of -> map -> load -> return
// body. (The mapping declaration's key annotation regexes over Storage/Memory,
// the only nominal difference; solx names it `scores(string)`, solc
// `get_scores_<id>`.)

// CHECK: sol.func @{{.*scores.*}}(%arg0: !sol.string<Memory>) -> ui256 attributes {{.*}}selector = -846305981 : i32
// CHECK:   %[[M:.*]] = sol.addr_of @{{.*scores.*}} : !sol.mapping<!sol.string<{{.*}}>, ui256>
// CHECK:   %[[SLOT:.*]] = sol.map %[[M]], %arg0 : !sol.mapping<!sol.string<{{.*}}>, ui256>, !sol.string<Memory>, !sol.ptr<ui256, Storage>
// CHECK:   %[[V:.*]] = sol.load %[[SLOT]] : !sol.ptr<ui256, Storage>, ui256
// CHECK:   sol.return %[[V]] : ui256

contract C {
    mapping(string => uint256) public scores;
}
