// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: %[[T:.*]] = sol.this : !sol.contract<{{.*}}>
// CHECK: %[[A:.*]] = sol.address_cast %[[T]] : !sol.contract<{{.*}}> to !sol.address
// CHECK: %{{.*}}, %[[R:.*]]:2 = sol.ext_call "{{.*}}"() at %[[A]] gas %{{.*}} value %{{.*}} selector %{{.*}} {callee_type = () -> (ui256, ui256), static_call} : !sol.address, () -> (i1, ui256, ui256)
// CHECK: sol.return %[[R]]#0, %[[R]]#1 : ui256, ui256

contract C {
    struct S { uint256 a; uint256 b; }
    S public s;

    function g() external returns (uint256, uint256) {
        return this.s();
    }
}
