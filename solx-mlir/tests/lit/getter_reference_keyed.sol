// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*}}tags{{.*}} -> !sol.string<Memory>
// CHECK: sol.func @{{.*}}blobs{{.*}} -> !sol.string<Memory>
// CHECK: sol.data_loc_cast {{.*}} : !sol.string<Storage>, !sol.string<Memory>

contract C {
    string[] public tags;
    mapping(uint256 => bytes) public blobs;
}
