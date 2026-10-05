// RUN: slang --emit-mlir=sol --debug-info %s | FileCheck %s --implicit-check-not='loc(unknown)'

// CHECK: sol.func @{{.*data.*}}
// CHECK:   sol.string_lit "\01\02" {{.*}} loc(#[[DATA_LITERAL:loc[0-9]*]])
// CHECK:   sol.return %{{.*}} loc(#[[DATA_RETURN:loc[0-9]*]])
// CHECK: } loc(#[[DATA_FN:loc[0-9]*]])
// CHECK: sol.func @{{.*glyphs.*}}
// CHECK:   sol.string_lit "\C3\A9x" {{.*}} loc(#[[GLYPHS_LITERAL:loc[0-9]*]])
// CHECK:   sol.return %{{.*}} loc(#[[GLYPHS_RETURN:loc[0-9]*]])
// CHECK: } loc(#[[GLYPHS_FN:loc[0-9]*]])
// CHECK: sol.func @{{.*text.*}}
// CHECK:   sol.string_lit "abcd" {{.*}} loc(#[[TEXT_LITERAL:loc[0-9]*]])
// CHECK:   sol.return %{{.*}} loc(#[[TEXT_RETURN:loc[0-9]*]])
// CHECK: } loc(#[[TEXT_FN:loc[0-9]*]])

// CHECK-DAG: #[[TEXT_LITERAL]] = loc("{{.*}}debug_location_strings.sol":35:16)
// CHECK-DAG: #[[TEXT_RETURN]] = loc("{{.*}}debug_location_strings.sol":35:9)
// CHECK-DAG: #[[DATA_LITERAL]] = loc("{{.*}}debug_location_strings.sol":39:16)
// CHECK-DAG: #[[DATA_RETURN]] = loc("{{.*}}debug_location_strings.sol":39:9)
// CHECK-DAG: #[[GLYPHS_LITERAL]] = loc("{{.*}}debug_location_strings.sol":43:16)
// CHECK-DAG: #[[GLYPHS_RETURN]] = loc("{{.*}}debug_location_strings.sol":43:9)

// CHECK-DAG: #[[TEXT_FN]] = loc(fused<#[[TEXT_SP:di_subprogram[0-9]*]]>[#[[TEXT:loc[0-9]*]]])
// CHECK-DAG: #[[TEXT]] = loc("{{.*}}debug_location_strings.sol":34:5)
// CHECK-DAG: #[[TEXT_SP]] = #llvm.di_subprogram<{{.*}}name = "text", linkageName = "text", {{.*}}type = #di_subroutine_type>
// CHECK-DAG: #[[DATA_FN]] = loc(fused<#[[DATA_SP:di_subprogram[0-9]*]]>[#[[DATA:loc[0-9]*]]])
// CHECK-DAG: #[[DATA]] = loc("{{.*}}debug_location_strings.sol":38:5)
// CHECK-DAG: #[[DATA_SP]] = #llvm.di_subprogram<{{.*}}name = "data", linkageName = "data", {{.*}}type = #di_subroutine_type>
// CHECK-DAG: #[[GLYPHS_FN]] = loc(fused<#[[GLYPHS_SP:di_subprogram[0-9]*]]>[#[[GLYPHS:loc[0-9]*]]])
// CHECK-DAG: #[[GLYPHS]] = loc("{{.*}}debug_location_strings.sol":42:5)
// CHECK-DAG: #[[GLYPHS_SP]] = #llvm.di_subprogram<{{.*}}name = "glyphs", linkageName = "glyphs", {{.*}}type = #di_subroutine_type>

contract C {
    function text() public pure returns (string memory) {
        return "ab" "cd";
    }

    function data() public pure returns (bytes memory) {
        return hex"01" hex"02";
    }

    function glyphs() public pure returns (string memory) {
        return unicode"é" unicode"x";
    }
}
