// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: %[[T:.*]] = sol.this : !sol.contract<{{.*}}>
// CHECK: %[[A:.*]] = sol.address_cast %[[T]] : !sol.contract<{{.*}}> to !sol.address
// CHECK: sol.ext_call "{{.*}}"(%{{.*}}) at %[[A]] gas %{{.*}} value %{{.*}} selector %{{.*}} {callee_type = (ui256) -> ui256, static_call} : !sol.address, (ui256) -> (i1, ui256)

contract C {
    mapping(uint256 => uint256) public m;

    function g(uint256 k) external returns (uint256) {
        return this.m(k);
    }
}
