// RUN: slang --emit-mlir=sol --debug-info --debug-info-runtime %s | FileCheck %s --implicit-check-not='loc(unknown)'

// CHECK: sol.func @{{.*constructor.*}}
// CHECK:   sol.return loc(#[[CONSTRUCTOR_CLOSE:loc[0-9]*]])
// CHECK: } loc(#[[CONSTRUCTOR_FN:loc[0-9]*]])

// CHECK-DAG: #[[CONSTRUCTOR:loc[0-9]*]] = loc("{{.*}}debug_location_keyword_functions.sol":53:5)
// CHECK-DAG: #[[CONSTRUCTOR_CLOSE]] = loc("{{.*}}debug_location_keyword_functions.sol":53:20)
// CHECK-DAG: #[[CONSTRUCTOR_FN]] = loc(fused<#[[CONSTRUCTOR_SP:di_subprogram[0-9]*]]>[#[[CONSTRUCTOR]]])
// CHECK-DAG: #[[CONSTRUCTOR_SP]] = #llvm.di_subprogram<{{.*}}name = "constructor", linkageName = "constructor", file = #di_file, line = 53, scopeLine = 53, subprogramFlags = "LocalToUnit|Definition", type = #di_subroutine_type>

// CHECK: sol.func @{{.*fallback.*}}
// CHECK:   sol.return loc(#[[FALLBACK_CLOSE:loc[0-9]*]])
// CHECK: } loc(#[[FALLBACK_FN:loc[0-9]*]])
// CHECK: sol.func @{{.*receive.*}}
// CHECK:   sol.return loc(#[[RECEIVE_CLOSE:loc[0-9]*]])
// CHECK: } loc(#[[RECEIVE_FN:loc[0-9]*]])

// CHECK-DAG: #[[FALLBACK:loc[0-9]*]] = loc("{{.*}}debug_location_keyword_functions.sol":55:5)
// CHECK-DAG: #[[FALLBACK_CLOSE]] = loc("{{.*}}debug_location_keyword_functions.sol":55:26)
// CHECK-DAG: #[[RECEIVE:loc[0-9]*]] = loc("{{.*}}debug_location_keyword_functions.sol":57:5)
// CHECK-DAG: #[[RECEIVE_CLOSE]] = loc("{{.*}}debug_location_keyword_functions.sol":57:33)
// CHECK-DAG: #[[FALLBACK_FN]] = loc(fused<#[[FALLBACK_SP:di_subprogram[0-9]*]]>[#[[FALLBACK]]])
// CHECK-DAG: #[[FALLBACK_SP]] = #llvm.di_subprogram<{{.*}}name = "fallback", linkageName = "fallback", file = #di_file, line = 55, scopeLine = 55, subprogramFlags = "LocalToUnit|Definition", type = #di_subroutine_type>
// CHECK-DAG: #[[RECEIVE_FN]] = loc(fused<#[[RECEIVE_SP:di_subprogram[0-9]*]]>[#[[RECEIVE]]])
// CHECK-DAG: #[[RECEIVE_SP]] = #llvm.di_subprogram<{{.*}}name = "receive", linkageName = "receive", file = #di_file, line = 57, scopeLine = 57, subprogramFlags = "LocalToUnit|Definition", type = #di_subroutine_type>

// CHECK-LABEL: :Leaf =======
// CHECK-DAG: #[[ROOT_CONSTRUCTOR:loc[0-9]*]] = loc("{{.*}}debug_location_keyword_functions.sol":61:5)
// CHECK: sol.func @"@constructor()"()
// CHECK:   sol.constant 7 : ui8 loc(#[[BASE_ARGUMENT:loc[0-9]*]])
// CHECK:   sol.cast %{{.*}} : ui8 to ui256 loc(#[[ROOT_CONSTRUCTOR]])
// CHECK:   sol.call @{{.*constructor.*}}(%{{.*}}) : (ui256) -> () loc(#[[ROOT_CONSTRUCTOR]])
// CHECK: sol.func @{{.*constructor.*}}(%arg0: ui256
// CHECK: } loc(#[[ROOT_CONSTRUCTOR_FN:loc[0-9]*]])

// CHECK-DAG: #[[BASE_ARGUMENT]] = loc("{{.*}}debug_location_keyword_functions.sol":64:23)
// CHECK-DAG: #[[ROOT_CONSTRUCTOR_FN]] = loc(fused<#[[ROOT_CONSTRUCTOR_SP:di_subprogram[0-9]*]]>[#[[ROOT_CONSTRUCTOR]]])
// CHECK-DAG: #[[ROOT_CONSTRUCTOR_SP]] = #llvm.di_subprogram<{{.*}}name = "constructor", linkageName = "constructor", file = #di_file, line = 61, scopeLine = 61, subprogramFlags = "LocalToUnit|Definition", type = #di_subroutine_type>

// CHECK-LABEL: :Tail =======
// CHECK-DAG: #[[TAIL_ROOT_CONSTRUCTOR:loc[0-9]*]] = loc("{{.*}}debug_location_keyword_functions.sol":61:5)
// CHECK: sol.func @{{.*constructor.*}}
// CHECK:   sol.constant 8 : ui8 loc(#[[MIDDLE_ARGUMENT:loc[0-9]*]])
// CHECK:   sol.cast %{{.*}} : ui8 to ui256 loc(#[[TAIL_ROOT_CONSTRUCTOR]])
// CHECK:   sol.call @{{.*constructor.*}}(%{{.*}}) : (ui256) -> () loc(#[[TAIL_ROOT_CONSTRUCTOR]])
// CHECK:   sol.return loc(#[[TAIL_CONSTRUCTOR_CLOSE:loc[0-9]*]])

// CHECK-DAG: #[[MIDDLE_ARGUMENT]] = loc("{{.*}}debug_location_keyword_functions.sol":66:34)
// CHECK-DAG: #[[TAIL_CONSTRUCTOR_CLOSE]] = loc("{{.*}}debug_location_keyword_functions.sol":69:20)

contract C {
    constructor() {}

    fallback() external {}

    receive() external payable {}
}

contract Root {
    constructor(uint256 s) {}
}

contract Leaf is Root(7) {}

abstract contract Middle is Root(8) {}

contract Tail is Middle {
    constructor() {}
}
