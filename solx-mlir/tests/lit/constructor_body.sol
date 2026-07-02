// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*}} attributes {{.*}}kind = #{{.*}}Constructor
// CHECK: sol.store %arg0
// CHECK: sol.store %arg1
// CHECK: sol.cadd
// CHECK: sol.store {{.*}}!sol.ptr<ui256, Storage>
// CHECK: sol.return

contract ConstructorTest {
    uint256 c;

    constructor(uint256 a, uint256 b) {
        c = a + b;
    }
}
