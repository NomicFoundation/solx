// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// A keyed getter of a `string[]` / `mapping(=>bytes)` public state variable returns
// the element/value as a Memory copy via `sol.data_loc_cast`, matching solc (the
// external ABI returns reference types in memory). solx names them `tags(uint256)` /
// `blobs(uint256)`, solc `get_tags_<id>` / `get_blobs_<id>`.
// CHECK-DAG: sol.func @{{.*}}tags{{.*}} -> !sol.string<Memory>
// CHECK-DAG: sol.func @{{.*}}blobs{{.*}} -> !sol.string<Memory>
// CHECK-DAG: sol.data_loc_cast {{.*}} : !sol.string<Storage>, !sol.string<Memory>

contract C {
    string[] public tags;
    mapping(uint256 => bytes) public blobs;
}
