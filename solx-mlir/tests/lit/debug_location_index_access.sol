// RUN: solx --emit-mlir=sol --debug-info %s | FileCheck %s --implicit-check-not='loc(unknown)'

// CHECK: #[[NESTED:loc[0-9]*]] = loc("{{.*}}debug_location_index_access.sol":91:5)
// CHECK: #[[READ:loc[0-9]*]] = loc("{{.*}}debug_location_index_access.sol":87:5)

// CHECK: sol.state_var @{{.*}} loc(#[[DATA:loc[0-9]*]])
// CHECK: sol.state_var @{{.*}} loc(#[[S:loc[0-9]*]])

// CHECK: sol.func @{{.*allocates.*}}(
// CHECK:   sol.malloc %{{.*}} zero_init {{.*}} loc(#[[NEW_ARRAY:loc[0-9]*]])

// CHECK: sol.func @{{.*appends.*}}(
// CHECK:   sol.push %{{.*}} loc(#[[PUSHED:loc[0-9]*]])
// CHECK:   sol.store %{{.*}}, %{{.*}} loc(#[[TUPLE_ASSIGN:loc[0-9]*]])

// CHECK: sol.func @{{.*clears.*}}(
// CHECK:   sol.gep %{{.*}} loc(#[[CLEARED:loc[0-9]*]])
// CHECK:   sol.store %{{.*}}, %{{.*}} loc(#[[DELETE_ELEMENT:loc[0-9]*]])
// CHECK:   sol.gep %{{.*}} loc(#[[CLEARED_MEMBER:loc[0-9]*]])
// CHECK:   sol.delete %{{.*}} loc(#[[DELETE_MEMBER:loc[0-9]*]])

// CHECK: sol.func @{{.*literal.*}}(
// CHECK:   sol.array_lit {{.*}} loc(#[[ARRAY_LITERAL:loc[0-9]*]])

// CHECK: sol.func @{{.*nested.*}}(
// CHECK:   sol.addr_of @{{.*}} loc(#[[NESTED_ELEMENT:loc[0-9]*]])
// CHECK:   sol.gep %{{.*}} loc(#[[NESTED_ELEMENT]])
// CHECK:   sol.load %{{.*}} loc(#[[NESTED_I:loc[0-9]*]])
// CHECK:   sol.gep %{{.*}} loc(#[[NESTED_ELEMENT]])
// CHECK:   sol.load %{{.*}} loc(#[[NESTED_ELEMENT]])
// CHECK:   sol.return %{{.*}} : ui256 loc(#[[NESTED_RETURN:loc[0-9]*]])
// CHECK: } loc(#[[NESTED_FN:loc[0-9]*]])

// CHECK: sol.func @{{.*read.*}}(
// CHECK:   sol.addr_of @{{.*}} loc(#[[ELEMENT:loc[0-9]*]])
// CHECK:   sol.load %{{.*}} loc(#[[READ_I:loc[0-9]*]])
// CHECK:   sol.gep %{{.*}} loc(#[[ELEMENT]])
// CHECK:   sol.load %{{.*}} loc(#[[ELEMENT]])
// CHECK:   sol.return %{{.*}} : ui256 loc(#[[READ_RETURN:loc[0-9]*]])
// CHECK: } loc(#[[READ_FN:loc[0-9]*]])

// CHECK-DAG: #[[DATA]] = loc("{{.*}}debug_location_index_access.sol":71:5)
// CHECK-DAG: #[[S]] = loc("{{.*}}debug_location_index_access.sol":72:5)
// CHECK-DAG: #[[ELEMENT]] = loc("{{.*}}debug_location_index_access.sol":88:16)
// CHECK-DAG: #[[READ_I]] = loc("{{.*}}debug_location_index_access.sol":88:21)
// CHECK-DAG: #[[READ_RETURN]] = loc("{{.*}}debug_location_index_access.sol":88:9)
// CHECK-DAG: #[[NESTED_ELEMENT]] = loc("{{.*}}debug_location_index_access.sol":92:16)
// CHECK-DAG: #[[NESTED_I]] = loc("{{.*}}debug_location_index_access.sol":92:25)
// CHECK-DAG: #[[NESTED_RETURN]] = loc("{{.*}}debug_location_index_access.sol":92:9)

// CHECK-DAG: #[[CLEARED]] = loc("{{.*}}debug_location_index_access.sol":96:16)
// CHECK-DAG: #[[DELETE_ELEMENT]] = loc("{{.*}}debug_location_index_access.sol":96:9)
// CHECK-DAG: #[[CLEARED_MEMBER]] = loc("{{.*}}debug_location_index_access.sol":97:16)
// CHECK-DAG: #[[DELETE_MEMBER]] = loc("{{.*}}debug_location_index_access.sol":97:9)

// CHECK-DAG: #[[NEW_ARRAY]] = loc("{{.*}}debug_location_index_access.sol":76:16)
// CHECK-DAG: #[[PUSHED]] = loc("{{.*}}debug_location_index_access.sol":80:17)
// CHECK-DAG: #[[TUPLE_ASSIGN]] = loc("{{.*}}debug_location_index_access.sol":80:9)
// CHECK-DAG: #[[ARRAY_LITERAL]] = loc("{{.*}}debug_location_index_access.sol":84:16)

// CHECK-DAG: #[[READ_FN]] = loc(fused<#[[READ_SP:di_subprogram[0-9]*]]>[#[[READ]]])
// CHECK-DAG: #[[READ_SP]] = #llvm.di_subprogram<{{.*}}name = "read", linkageName = "read", {{.*}}type = #di_subroutine_type>
// CHECK-DAG: #[[NESTED_FN]] = loc(fused<#[[NESTED_SP:di_subprogram[0-9]*]]>[#[[NESTED]]])
// CHECK-DAG: #[[NESTED_SP]] = #llvm.di_subprogram<{{.*}}name = "nested", linkageName = "nested", {{.*}}type = #di_subroutine_type>

contract C {
    struct S {
        uint256[2] values;
    }

    uint256[3] private data;
    S private s;
    uint256[] private grown;

    function allocates(uint256 n) public pure returns (uint256) {
        return new uint256[](n).length;
    }

    function appends(uint256 v) public returns (uint256 other) {
        (other, grown.push()) = (v, v);
    }

    function literal() public pure returns (uint256) {
        return [uint256(1), 2, 3][1];
    }

    function read(uint256 i) public view returns (uint256) {
        return data[i];
    }

    function nested(uint256 i) public view returns (uint256) {
        return s.values[i];
    }

    function clears(uint256 i) public {
        delete data[i];
        delete s.values;
    }
}
