// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*deleteMemoryArray.*}}
// CHECK:   sol.malloc zero_init : {{.*}}!sol.array<? x ui256, Memory>
// CHECK:   sol.store %{{.*}}, %{{.*}} : !sol.array<? x ui256, Memory>

// CHECK: sol.func @{{.*deleteString.*}}
// CHECK:   sol.malloc : {{.*}}!sol.string<Memory>
// CHECK:   sol.store %{{.*}}, %{{.*}} : !sol.string<Memory>

contract C {
    function deleteMemoryArray() public pure {
        uint256[] memory a = new uint256[](3);
        delete a;
    }

    function deleteString() public pure {
        string memory s = "hello";
        delete s;
    }
}
