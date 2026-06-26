// RUN: solx --emit-mlir=sol %s | FileCheck %s

// A public mapping / array getter called on another instance `other.m(key)` lowers
// the key/index argument against the getter's `(K) -> (V)` signature. solc emits the
// same selector and signature but through a symbol-callee `sol.ext_call`, so this is
// pinned solx-only (the `ext_func_constant` + `ext_icall` form is a pre-existing
// benign divergence shared with the `this.m(key)` path).
// CHECK: sol.ext_func_constant %{{.*}} : !sol.address -> !sol.ext_func_ref<(ui256) -> ui256>
// CHECK: sol.ext_icall %{{.*}}(%{{.*}})

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
