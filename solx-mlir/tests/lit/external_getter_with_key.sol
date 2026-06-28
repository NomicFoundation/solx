// RUN: solx --emit-mlir=sol %s | FileCheck %s --check-prefixes=CHECK,CHECK-SOLX
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s --check-prefixes=CHECK,CHECK-SOLC

// Two public getters called on another contract instance, a mapping getter and an array
// getter. solx lowers the external call to sol.ext_func_constant + sol.ext_icall (an
// ext_func_ref callee); solc emits a sol.constant selector plus a symbol-callee sol.ext_call.
// The synthesized array getter also diverges: solx bounds-checks the index with sol.length +
// sol.cmp lt + sol.require then a plain sol.gep, while solc folds the guard into
// sol.gep ... no_panic_bounds. The mapping getter body agrees. CHECK-SOLX pins the solx ops,
// CHECK-SOLC the solc ops.

// CHECK-SOLX: sol.ext_func_constant %{{.*}} : !sol.address -> !sol.ext_func_ref<(ui256) -> ui256>
// CHECK-SOLX: sol.ext_icall %{{.*}}(%{{.*}}) gas %{{.*}} value %{{.*}} : !sol.ext_func_ref<(ui256) -> ui256>, (ui256) -> (i1, ui256)
// CHECK: sol.map %{{.*}}, %{{.*}} : !sol.mapping<ui256, ui256>, ui256, !sol.ptr<ui256, Storage>
// CHECK-SOLC: sol.gep %{{.*}}, %{{.*}} no_panic_bounds : !sol.array<? x ui256, Storage>, ui256, !sol.ptr<ui256, Storage>
// CHECK-SOLC: sol.ext_call "{{.*}}"(%{{.*}}) at %{{.*}} gas %{{.*}} value %{{.*}} selector %{{.*}} {callee_type = (ui256) -> ui256, static_call} : !sol.address, (ui256) -> (i1, ui256)
// CHECK-SOLX: sol.length %{{.*}} : !sol.array<? x ui256, Storage>
// CHECK-SOLX: sol.cmp lt, %{{.*}}, %{{.*}} : ui256
// CHECK-SOLX: sol.require %{{.*}}()
// CHECK-SOLX: sol.gep %{{.*}}, %{{.*}} : !sol.array<? x ui256, Storage>, ui256, !sol.ptr<ui256, Storage>

contract Other {
    mapping(uint256 => uint256) public m;
    uint256[] public arr;
}

contract C {
    function readMapping(Other o, uint256 key) external view returns (uint256) {
        return o.m(key);
    }

    function readArray(Other o, uint256 index) external view returns (uint256) {
        return o.arr(index);
    }
}
