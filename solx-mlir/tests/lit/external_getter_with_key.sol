// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.ext_call "{{.*}}"(%{{.*}}) at %{{.*}} gas %{{.*}} value %{{.*}} selector %{{.*}} {callee_type = (ui256) -> ui256, static_call} : !sol.address, (ui256) -> (i1, ui256)
// CHECK: sol.ext_call "{{.*}}"(%{{.*}}) at %{{.*}} gas %{{.*}} value %{{.*}} selector %{{.*}} {callee_type = (ui256) -> ui256, static_call} : !sol.address, (ui256) -> (i1, ui256)

contract C {
    function readArray(Other o, uint256 index) external view returns (uint256) {
        return o.array(index);
    }

    function readMapping(Other o, uint256 key) external view returns (uint256) {
        return o.m(key);
    }
}

contract Other {
    mapping(uint256 => uint256) public m;
    uint256[] public array;
}
