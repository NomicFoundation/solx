// RUN: solx --emit-mlir=sol %qualifier_library_external/main.sol %qualifier_library_external/library.sol | FileCheck %s
// RUN: solc --mlir-action=print-init %qualifier_library_external/main.sol %qualifier_library_external/library.sol 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*externalCall.*}}
// CHECK:   sol.lib_addr
// CHECK:   sol.ext_call "{{.*half.*}}"({{.*}}library_call

// CHECK: sol.func @{{.*tried.*}}
// CHECK:   sol.lib_addr
// CHECK:   sol.ext_call "{{.*half.*}}"({{.*}}try_call
// CHECK:   sol.try
