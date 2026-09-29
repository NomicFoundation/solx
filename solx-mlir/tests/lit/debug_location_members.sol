// RUN: solx --emit-mlir=sol --debug-info %s | FileCheck %s --implicit-check-not='loc(unknown)'

// CHECK-DAG: #[[VALUES:loc[0-9]*]] = loc("{{.*}}debug_location_members.sol":108:5)
// CHECK-DAG: #[[BALANCES:loc[0-9]*]] = loc("{{.*}}debug_location_members.sol":109:5)
// CHECK: #[[SET:loc[0-9]*]] = loc("{{.*}}debug_location_members.sol":118:5)

// CHECK: sol.state_var @{{.*}} loc(#[[VALUE:loc[0-9]*]])
// CHECK: sol.immutable @{{.*}} loc(#[[START:loc[0-9]*]])
// CHECK: sol.state_var @{{.*values.*}} loc(#[[VALUES]])
// CHECK: sol.state_var @{{.*balances.*}} loc(#[[BALANCES]])
// CHECK: sol.func @{{.*constructor.*}}
// CHECK:   sol.addr_of @{{.*}} loc(#[[VALUE]])
// CHECK:   sol.constant 7 : ui8 loc(#[[SEVEN:loc[0-9]*]])
// CHECK:   sol.store %{{.*}}Storage{{.*}} loc(#[[VALUE]])
// CHECK:   sol.addr_of @{{.*}} loc(#[[START]])
// CHECK:   sol.constant 3 : ui8 loc(#[[THREE:loc[0-9]*]])
// CHECK:   sol.store %{{.*}}Immutable{{.*}} loc(#[[START]])
// CHECK:   sol.return loc(#[[CONTRACT:loc[0-9]*]])
// CHECK: } loc(#[[CONTRACT_FN:loc[0-9]*]])
// CHECK: sol.func @{{.*builds.*}}(
// CHECK:   sol.malloc {{.*}} loc(#[[POSITIONAL:loc[0-9]*]])
// CHECK:   sol.malloc {{.*}} loc(#[[NAMED:loc[0-9]*]])
// CHECK:   sol.cadd %{{.*}}, %{{.*}} : ui256 loc(#[[BUILT_SUM:loc[0-9]*]])
// CHECK: sol.func @{{.*bump.*}}(
// CHECK:   sol.addr_of @{{.*}} loc(#[[BUMPED:loc[0-9]*]])
// CHECK:   sol.cadd %{{.*}}, %{{.*}} : ui256 loc(#[[INCREMENT:loc[0-9]*]])
// CHECK:   sol.load %{{.*}} loc(#[[SWAP_OTHER:loc[0-9]*]])
// CHECK:   sol.addr_of @{{.*}} loc(#[[SWAP_VALUE:loc[0-9]*]])
// CHECK:   sol.addr_of @{{.*}} loc(#[[SWAP_TARGET:loc[0-9]*]])
// CHECK:   sol.store %{{.*}}, %{{.*}} loc(#[[SWAP:loc[0-9]*]])
// CHECK: sol.func @{{.*counts.*}}(
// CHECK:   sol.addr_of @{{.*}} loc(#[[MEMBER_READ:loc[0-9]*]])
// CHECK:   sol.load %{{.*}}Storage{{.*}} loc(#[[MEMBER_READ]])
// CHECK: sol.func @{{.*seeds.*}}(
// CHECK:   sol.string_lit "seed" {{.*}} loc(#[[SEED_TEXT:loc[0-9]*]])
// CHECK:   "sol.keccak256"(%{{.*}}) {{.*}} loc(#[[SEED_HASH:loc[0-9]*]])
// CHECK: sol.func @{{.*set.*}}(%arg0: ui256 loc("{{.*}}debug_location_members.sol":118:5))
// CHECK:   sol.store %arg0, %{{.*}} loc(#[[V:loc[0-9]*]])
// CHECK:   sol.load %{{.*}} loc(#[[READ_V:loc[0-9]*]])
// CHECK:   sol.addr_of @{{.*}} loc(#[[ASSIGN:loc[0-9]*]])
// CHECK:   sol.store %{{.*}}Storage{{.*}} loc(#[[ASSIGN]])
// CHECK:   sol.return loc(#[[CLOSE:loc[0-9]*]])
// CHECK: } loc(#[[SET_FN:loc[0-9]*]])
// CHECK: sol.func @{{.*value.*}}
// CHECK:   sol.load %{{.*}} loc(#[[VALUE]])
// CHECK:   sol.return %{{.*}} : ui256 loc(#[[VALUE]])
// CHECK: } loc(#[[VALUE_FN:loc[0-9]*]])
// CHECK: sol.func @{{.*LIMIT.*}}
// CHECK:   sol.constant 10 : ui8 loc(#[[TEN:loc[0-9]*]])
// CHECK:   sol.return %{{.*}} : ui256 loc(#[[LIMIT:loc[0-9]*]])
// CHECK: } loc(#[[LIMIT_FN:loc[0-9]*]])
// CHECK: sol.func @{{.*start.*}}
// CHECK:   sol.load_immutable @{{.*}} loc(#[[START]])
// CHECK:   sol.return %{{.*}} : ui256 loc(#[[START]])
// CHECK: } loc(#[[START_FN:loc[0-9]*]])
// CHECK: sol.func @{{.*values.*}}
// CHECK:   sol.gep %{{.*}} loc(#[[VALUES]])
// CHECK:   sol.return %{{.*}} : ui256 loc(#[[VALUES]])
// CHECK: } loc(#[[VALUES_FN:loc[0-9]*]])
// CHECK: sol.func @{{.*balances.*}}
// CHECK:   sol.map %{{.*}} loc(#[[BALANCES]])
// CHECK:   sol.return %{{.*}} : ui256 loc(#[[BALANCES]])
// CHECK: } loc(#[[BALANCES_FN:loc[0-9]*]])

// CHECK-DAG: #[[CONTRACT]] = loc("{{.*}}debug_location_members.sol":104:1)
// CHECK-DAG: #[[VALUE]] = loc("{{.*}}debug_location_members.sol":105:5)
// CHECK-DAG: #[[SEVEN]] = loc("{{.*}}debug_location_members.sol":105:28)
// CHECK-DAG: #[[LIMIT]] = loc("{{.*}}debug_location_members.sol":106:5)
// CHECK-DAG: #[[TEN]] = loc("{{.*}}debug_location_members.sol":106:37)
// CHECK-DAG: #[[START]] = loc("{{.*}}debug_location_members.sol":107:5)
// CHECK-DAG: #[[THREE]] = loc("{{.*}}debug_location_members.sol":107:38)
// CHECK-DAG: #[[V]] = loc("{{.*}}debug_location_members.sol":118:18)
// CHECK-DAG: #[[ASSIGN]] = loc("{{.*}}debug_location_members.sol":119:9)
// CHECK-DAG: #[[READ_V]] = loc("{{.*}}debug_location_members.sol":119:17)
// CHECK-DAG: #[[CLOSE]] = loc("{{.*}}debug_location_members.sol":120:5)
// CHECK-DAG: #[[BUMPED]] = loc("{{.*}}debug_location_members.sol":123:11)
// CHECK-DAG: #[[INCREMENT]] = loc("{{.*}}debug_location_members.sol":123:9)
// CHECK-DAG: #[[SWAP_OTHER]] = loc("{{.*}}debug_location_members.sol":124:27)
// CHECK-DAG: #[[SWAP_VALUE]] = loc("{{.*}}debug_location_members.sol":124:34)
// CHECK-DAG: #[[SWAP_TARGET]] = loc("{{.*}}debug_location_members.sol":124:10)
// CHECK-DAG: #[[SWAP]] = loc("{{.*}}debug_location_members.sol":124:9)
// CHECK-DAG: #[[POSITIONAL]] = loc("{{.*}}debug_location_members.sol":128:31)
// CHECK-DAG: #[[NAMED]] = loc("{{.*}}debug_location_members.sol":129:26)
// CHECK-DAG: #[[BUILT_SUM]] = loc("{{.*}}debug_location_members.sol":130:16)
// CHECK-DAG: #[[MEMBER_READ]] = loc("{{.*}}debug_location_members.sol":134:20)
// CHECK-DAG: #[[SEED_TEXT]] = loc("{{.*}}debug_location_members.sol":110:39)
// CHECK-DAG: #[[SEED_HASH]] = loc("{{.*}}debug_location_members.sol":110:29)

// CHECK-DAG: #[[CONTRACT_FN]] = loc(fused<#[[CONTRACT_SP:di_subprogram[0-9]*]]>[#[[CONTRACT]]])
// CHECK-DAG: #[[CONTRACT_SP]] = #llvm.di_subprogram<{{.*}}name = "constructor", linkageName = "constructor", file = #di_file, line = 104, scopeLine = 104, subprogramFlags = "LocalToUnit|Definition", type = #di_subroutine_type, flags = Artificial>
// CHECK-DAG: #[[SET_FN]] = loc(fused<#[[SET_SP:di_subprogram[0-9]*]]>[#[[SET]]])
// CHECK-DAG: #[[SET_SP]] = #llvm.di_subprogram<{{.*}}name = "set", linkageName = "set", file = #di_file, line = 118, scopeLine = 118, subprogramFlags = "LocalToUnit|Definition", type = #di_subroutine_type>
// CHECK-DAG: #[[VALUE_FN]] = loc(fused<#[[VALUE_SP:di_subprogram[0-9]*]]>[#[[VALUE]]])
// CHECK-DAG: #[[VALUE_SP]] = #llvm.di_subprogram<{{.*}}name = "value", linkageName = "value", file = #di_file, line = 105, scopeLine = 105, subprogramFlags = "LocalToUnit|Definition", type = #di_subroutine_type>
// CHECK-DAG: #[[LIMIT_FN]] = loc(fused<#[[LIMIT_SP:di_subprogram[0-9]*]]>[#[[LIMIT]]])
// CHECK-DAG: #[[LIMIT_SP]] = #llvm.di_subprogram<{{.*}}name = "LIMIT", linkageName = "LIMIT", file = #di_file, line = 106, scopeLine = 106, subprogramFlags = "LocalToUnit|Definition", type = #di_subroutine_type>
// CHECK-DAG: #[[START_FN]] = loc(fused<#[[START_SP:di_subprogram[0-9]*]]>[#[[START]]])
// CHECK-DAG: #[[START_SP]] = #llvm.di_subprogram<{{.*}}name = "start", linkageName = "start", file = #di_file, line = 107, scopeLine = 107, subprogramFlags = "LocalToUnit|Definition", type = #di_subroutine_type>
// CHECK-DAG: #[[VALUES_FN]] = loc(fused<#[[VALUES_SP:di_subprogram[0-9]*]]>[#[[VALUES]]])
// CHECK-DAG: #[[VALUES_SP]] = #llvm.di_subprogram<{{.*}}name = "values", linkageName = "values", file = #di_file, line = 108, scopeLine = 108, subprogramFlags = "LocalToUnit|Definition", type = #di_subroutine_type>
// CHECK-DAG: #[[BALANCES_FN]] = loc(fused<#[[BALANCES_SP:di_subprogram[0-9]*]]>[#[[BALANCES]]])
// CHECK-DAG: #[[BALANCES_SP]] = #llvm.di_subprogram<{{.*}}name = "balances", linkageName = "balances", file = #di_file, line = 109, scopeLine = 109, subprogramFlags = "LocalToUnit|Definition", type = #di_subroutine_type>

contract C {
    uint256 public value = 7;
    uint256 public constant LIMIT = 10;
    uint256 public immutable start = 3;
    uint256[] public values;
    mapping(address => uint256) public balances;
    bytes32 constant SEED = keccak256("seed");
    S s;

    struct S {
        uint256 count;
        uint256 extra;
    }

    function set(uint256 v) public {
        value = v;
    }

    function bump(uint256 other) public {
        ++value;
        (value, other) = (other, value);
    }

    function builds() public pure returns (uint256) {
        S memory positional = S(1, 2);
        S memory named = S({count: 3, extra: 4});
        return positional.count + named.extra;
    }

    function counts(uint256 i) public view returns (uint256) {
        return i + s.count;
    }

    function seeds() public pure returns (bytes32) {
        return SEED;
    }
}
