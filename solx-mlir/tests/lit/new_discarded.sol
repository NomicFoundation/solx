// RUN: solx --emit-mlir=sol %s | FileCheck %s

// CHECK: sol.func @{{.*discarded_creation.*}}
// CHECK:   sol.new "{{[^"]*}}Child{{[^"]*}}"

// CHECK: sol.func @{{.*discarded_allocation.*}}
// CHECK:   sol.malloc %{{.*}} zero_init : ui256 !sol.array<? x ui256, Memory>

// CHECK: sol.func @{{.*discarded_reference.*}}
// CHECK-NEXT: sol.return

contract C {
    function discarded_creation() public {
        new Child(1);
    }

    function discarded_allocation(uint256 n) public pure {
        new uint256[](n);
    }

    function discarded_reference() public pure {
        new Child;
    }
}

contract Child {
    uint256 stored;

    constructor(uint256 a) {
        stored = a;
    }
}
