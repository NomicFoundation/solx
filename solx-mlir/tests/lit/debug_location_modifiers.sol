// RUN: solx --emit-mlir=sol --debug-info %s | FileCheck %s --implicit-check-not='loc(unknown)'

// CHECK-DAG: #[[SET:loc[0-9]*]] = loc("{{.*}}debug_location_modifiers.sol":44:5)
// CHECK-DAG: #[[MODIFIER:loc[0-9]*]] = loc("{{.*}}debug_location_modifiers.sol":38:5)

// CHECK: sol.state_var @{{.*}} loc(#[[VALUE:loc[0-9]*]])
// CHECK: sol.func @{{.*set.*}}(%arg0: ui256 loc("{{.*}}debug_location_modifiers.sol":44:5))
// CHECK:   sol.modifier_invocation @{{.*positive.*}} {
// CHECK:     sol.load %{{.*}} loc(#[[ARGUMENT:loc[0-9]*]])
// CHECK:     sol.yield %{{.*}} loc(#[[INVOCATION:loc[0-9]*]])
// CHECK:   } loc(#[[INVOCATION]])
// CHECK:   sol.store %{{.*}}Storage{{.*}} loc(#[[ASSIGN:loc[0-9]*]])
// CHECK: } loc(#[[SET_FN:loc[0-9]*]])
// CHECK: sol.modifier @{{.*positive.*}}(%arg0: ui256 loc("{{.*}}debug_location_modifiers.sol":38:5))
// CHECK:   sol.store %arg0, %{{.*}} loc(#[[X:loc[0-9]*]])
// CHECK:   sol.require %{{.*}} loc(#[[REQUIRE:loc[0-9]*]])
// CHECK:   sol.placeholder loc(#[[PLACEHOLDER:loc[0-9]*]])
// CHECK:   sol.cadd %{{.*}}, %{{.*}} : ui256 loc(#[[INCREMENT:loc[0-9]*]])
// CHECK:   sol.return loc(#[[MODIFIER_CLOSE:loc[0-9]*]])
// CHECK: } loc(#[[MODIFIER]])

// CHECK-DAG: #[[VALUE]] = loc("{{.*}}debug_location_modifiers.sol":36:5)
// CHECK-DAG: #[[INVOCATION]] = loc("{{.*}}debug_location_modifiers.sol":44:36)
// CHECK-DAG: #[[ARGUMENT]] = loc("{{.*}}debug_location_modifiers.sol":44:45)
// CHECK-DAG: #[[ASSIGN]] = loc("{{.*}}debug_location_modifiers.sol":45:9)
// CHECK-DAG: #[[X]] = loc("{{.*}}debug_location_modifiers.sol":38:23)
// CHECK-DAG: #[[REQUIRE]] = loc("{{.*}}debug_location_modifiers.sol":39:9)
// CHECK-DAG: #[[PLACEHOLDER]] = loc("{{.*}}debug_location_modifiers.sol":40:9)
// CHECK-DAG: #[[INCREMENT]] = loc("{{.*}}debug_location_modifiers.sol":41:9)
// CHECK-DAG: #[[MODIFIER_CLOSE]] = loc("{{.*}}debug_location_modifiers.sol":42:5)

// CHECK-DAG: #[[SET_FN]] = loc(fused<#[[SET_SP:di_subprogram[0-9]*]]>[#[[SET]]])
// CHECK-DAG: #[[SET_SP]] = #llvm.di_subprogram<{{.*}}name = "set", linkageName = "set", {{.*}}type = #di_subroutine_type>

contract C {
    uint256 value;

    modifier positive(uint256 x) {
        require(x > 0);
        _;
        value += 1;
    }

    function set(uint256 x) public positive(x) {
        value = x;
    }
}
