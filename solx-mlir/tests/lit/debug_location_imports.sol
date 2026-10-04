// RUN: slang --emit-mlir=sol --debug-info %s %S/Inputs/debug_location_imported.sol | FileCheck %s --implicit-check-not='loc(unknown)'

// CHECK: #[[CALLER:loc[0-9]*]] = loc("{{.*}}debug_location_imports.sol":31:5)
// CHECK: #[[TRIPLE:loc[0-9]*]] = loc("{{.*}}debug_location_imported.sol":1:1)

// CHECK: sol.func @{{.*run.*}}(%arg0: ui256 loc("{{.*}}debug_location_imports.sol":31:5))
// CHECK:   sol.call @{{.*triple.*}} loc(#[[CALL:loc[0-9]*]])
// CHECK: } loc(#[[CALLER_FN:loc[0-9]*]])
// CHECK: sol.func private @{{.*triple.*}}(%arg0: ui256 loc("{{.*}}debug_location_imported.sol":1:1))
// CHECK:   sol.store %arg0, %{{.*}} loc(#[[X:loc[0-9]*]])
// CHECK:   sol.load %{{.*}} loc(#[[PRODUCT:loc[0-9]*]])
// CHECK:   sol.cmul %{{.*}}, %{{.*}} : ui256 loc(#[[PRODUCT]])
// CHECK:   sol.return %{{.*}} : ui256 loc(#[[RETURN:loc[0-9]*]])
// CHECK: } loc(#[[TRIPLE_FN:loc[0-9]*]])

// CHECK-DAG: #[[FILE:di_file[0-9]*]] = #llvm.di_file<"{{.*}}debug_location_imports.sol" in "">
// CHECK-DAG: #[[IMPORTED_FILE:di_file[0-9]*]] = #llvm.di_file<"{{.*}}debug_location_imported.sol" in "">
// CHECK-DAG: #[[CALL]] = loc("{{.*}}debug_location_imports.sol":32:16)
// CHECK-DAG: #[[X]] = loc("{{.*}}debug_location_imported.sol":1:17)
// CHECK-DAG: #[[PRODUCT]] = loc("{{.*}}debug_location_imported.sol":2:12)
// CHECK-DAG: #[[RETURN]] = loc("{{.*}}debug_location_imported.sol":2:5)

// CHECK-DAG: #[[CALLER_FN]] = loc(fused<#[[CALLER_SP:di_subprogram[0-9]*]]>[#[[CALLER]]])
// CHECK-DAG: #[[CALLER_SP]] = #llvm.di_subprogram<{{.*}}scope = #[[FILE]], name = "run", linkageName = "run", file = #[[FILE]], line = 31, {{.*}}>
// CHECK-DAG: #[[TRIPLE_FN]] = loc(fused<#[[TRIPLE_SP:di_subprogram[0-9]*]]>[#[[TRIPLE]]])
// CHECK-DAG: #[[TRIPLE_SP]] = #llvm.di_subprogram<{{.*}}scope = #[[IMPORTED_FILE]], name = "triple", linkageName = "triple", file = #[[IMPORTED_FILE]], line = 1, {{.*}}>

import {triple} from "./Inputs/debug_location_imported.sol";

contract C {
    function run(uint256 a) public pure returns (uint256) {
        return triple(a);
    }
}
