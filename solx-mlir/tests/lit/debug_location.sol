// RUN: slang --emit-mlir=sol --debug-info --debug-info-runtime %s | FileCheck %s --implicit-check-not='loc(unknown)'

// CHECK: } {kind = #Contract} loc(#[[CONTRACT:loc[0-9]*]])
// CHECK-NEXT: } loc(#[[CONTRACT_CU:loc[0-9]*]])

// CHECK-DAG: #[[FILE:di_file[0-9]*]] = #llvm.di_file<"{{.*}}debug_location.sol" in "">
// CHECK-DAG: #[[CU:di_compile_unit[0-9]*]] = #llvm.di_compile_unit<id = distinct[{{[0-9]+}}]<>, sourceLanguage = DW_LANG_Assembly, file = #[[FILE]], producer = "", isOptimized = true, emissionKind = Full>
// CHECK-DAG: #[[CONTRACT]] = loc("{{.*}}debug_location.sol":23:1)
// CHECK-DAG: #[[CONTRACT_CU]] = loc(fused<#[[CU]]>[#[[CONTRACT]]])

// CHECK: } {kind = #Contract, runtime} loc(#[[RUNTIME:loc[0-9]*]])
// CHECK-NEXT: } loc(#[[RUNTIME_CU:loc[0-9]*]])

// CHECK-DAG: #[[RUNTIME]] = loc("{{.*}}debug_location.sol":23:1)
// CHECK-DAG: #[[RUNTIME_CU]] = loc(fused<#{{di_compile_unit[0-9]*}}>[#[[RUNTIME]]])

// CHECK: } {kind = #Library} loc(#[[LIBRARY:loc[0-9]*]])
// CHECK-NEXT: } loc(#[[LIBRARY_CU:loc[0-9]*]])

// CHECK-DAG: #[[LIBRARY]] = loc("{{.*}}debug_location.sol":25:1)
// CHECK-DAG: #[[LIBRARY_CU]] = loc(fused<#{{di_compile_unit[0-9]*}}>[#[[LIBRARY]]])

contract C {}

library L {}
