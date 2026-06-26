// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// The keyed getter signature is multi-level and matches solc: a nested
// `mapping(=>uint[])` getter takes two keys, a reference-keyed `mapping(string=>…)`
// decodes the key into Memory, and a `mapping(=>Struct)` getter flattens the struct's
// returnable members. solx names them `nested(...)`/`byName(...)`/`structs(...)`, solc
// `get_nested_<id>` etc.
// CHECK-DAG: sol.func @{{.*}}nested{{.*}}(%arg0: ui256, %arg1: ui256) -> ui256
// CHECK-DAG: sol.func @{{.*}}byName{{.*}}(%arg0: !sol.string<Memory>) -> ui256
// CHECK-DAG: sol.func @{{.*}}structs{{.*}}(%arg0: ui256) -> (ui256, i1)

contract C {
    struct S {
        uint256 a;
        bool b;
    }

    mapping(uint256 => uint256[]) public nested;
    mapping(string => uint256) public byName;
    mapping(uint256 => S) public structs;
}
