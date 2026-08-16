// RUN: solx --emit-mlir=sol %qualifier_library_external/main.sol %qualifier_library_external/library.sol | FileCheck %s

// solc's print-init substitutes `sol.timestamp` for module member accesses, so this is solx-only.

// CHECK: sol.func @{{.*externalCall.*}}
// CHECK:   sol.call @{{.*mark.*}}
// CHECK:   sol.lib_addr
// CHECK:   sol.call @{{.*eight.*}}
// CHECK:   sol.ext_call

// CHECK: sol.func @{{.*tried.*}}
// CHECK:   sol.call @{{.*mark.*}}
// CHECK:   sol.lib_addr
// CHECK:   sol.call @{{.*eight.*}}
// CHECK:   sol.ext_call "{{.*half.*}}"({{.*}}try_call
// CHECK:   sol.try
