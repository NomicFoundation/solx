// RUN: solx --emit-mlir=sol --debug-info %s | FileCheck %s --implicit-check-not='loc(unknown)'

// CHECK: } {kind = #Contract} loc(#[[CONTRACT_CU:loc[0-9]*]])
// CHECK-NEXT: } loc(#[[CONTRACT_CU]])

// CHECK-DAG: #[[FILE:di_file[0-9]*]] = #llvm.di_file<"{{.*}}debug_location.sol" in "">
// CHECK-DAG: #[[CU:di_compile_unit[0-9]*]] = #llvm.di_compile_unit<id = distinct[{{[0-9]+}}]<>, sourceLanguage = DW_LANG_Assembly, file = #[[FILE]], producer = "", isOptimized = true, emissionKind = Full>
// CHECK-DAG: #[[CONTRACT:loc[0-9]*]] = loc("{{.*}}debug_location.sol":17:1)
// CHECK-DAG: #[[CONTRACT_CU]] = loc(fused<#[[CU]]>[#[[CONTRACT]]])

// CHECK: } {kind = #Library} loc(#[[LIBRARY_CU:loc[0-9]*]])
// CHECK-NEXT: } loc(#[[LIBRARY_CU]])

// CHECK-DAG: #[[LIBRARY:loc[0-9]*]] = loc("{{.*}}debug_location.sol":19:1)
// CHECK-DAG: #[[LIBRARY_CU]] = loc(fused<#{{di_compile_unit[0-9]*}}>[#[[LIBRARY]]])

contract C {}

library L {}
