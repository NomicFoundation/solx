// RUN: slang --emit-mlir=sol --debug-info --debug-info-runtime %s | FileCheck %s --implicit-check-not='loc(unknown)'

// CHECK: #[[NESTED:loc[0-9]*]] = loc("{{.*}}debug_location_index_access.sol":96:5)
// CHECK: #[[READ:loc[0-9]*]] = loc("{{.*}}debug_location_index_access.sol":92:5)

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

// CHECK: sol.func @{{.*pushes.*}}(
// CHECK:   sol.push %{{.*}} loc(#[[PUSH_READ:loc[0-9]*]])
// CHECK:   sol.load %{{.*}} loc(#[[PUSH_READ]])

// CHECK: sol.func @{{.*read.*}}(
// CHECK:   sol.addr_of @{{.*}} loc(#[[ELEMENT:loc[0-9]*]])
// CHECK:   sol.load %{{.*}} loc(#[[READ_I:loc[0-9]*]])
// CHECK:   sol.gep %{{.*}} loc(#[[ELEMENT]])
// CHECK:   sol.load %{{.*}} loc(#[[ELEMENT]])
// CHECK:   sol.return %{{.*}} : ui256 loc(#[[READ_RETURN:loc[0-9]*]])
// CHECK: } loc(#[[READ_FN:loc[0-9]*]])

// CHECK-DAG: #[[DATA]] = loc("{{.*}}debug_location_index_access.sol":76:5)
// CHECK-DAG: #[[S]] = loc("{{.*}}debug_location_index_access.sol":77:5)
// CHECK-DAG: #[[ELEMENT]] = loc("{{.*}}debug_location_index_access.sol":93:16)
// CHECK-DAG: #[[READ_I]] = loc("{{.*}}debug_location_index_access.sol":93:21)
// CHECK-DAG: #[[READ_RETURN]] = loc("{{.*}}debug_location_index_access.sol":93:9)
// CHECK-DAG: #[[NESTED_ELEMENT]] = loc("{{.*}}debug_location_index_access.sol":97:16)
// CHECK-DAG: #[[NESTED_I]] = loc("{{.*}}debug_location_index_access.sol":97:25)
// CHECK-DAG: #[[NESTED_RETURN]] = loc("{{.*}}debug_location_index_access.sol":97:9)

// CHECK-DAG: #[[CLEARED]] = loc("{{.*}}debug_location_index_access.sol":101:16)
// CHECK-DAG: #[[DELETE_ELEMENT]] = loc("{{.*}}debug_location_index_access.sol":101:9)
// CHECK-DAG: #[[CLEARED_MEMBER]] = loc("{{.*}}debug_location_index_access.sol":102:16)
// CHECK-DAG: #[[DELETE_MEMBER]] = loc("{{.*}}debug_location_index_access.sol":102:9)

// CHECK-DAG: #[[NEW_ARRAY]] = loc("{{.*}}debug_location_index_access.sol":81:16)
// CHECK-DAG: #[[PUSHED]] = loc("{{.*}}debug_location_index_access.sol":85:17)
// CHECK-DAG: #[[TUPLE_ASSIGN]] = loc("{{.*}}debug_location_index_access.sol":85:9)
// CHECK-DAG: #[[ARRAY_LITERAL]] = loc("{{.*}}debug_location_index_access.sol":89:16)
// CHECK-DAG: #[[PUSH_READ]] = loc("{{.*}}debug_location_index_access.sol":106:16)

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

    function pushes() public returns (uint256) {
        return grown.push();
    }
}
