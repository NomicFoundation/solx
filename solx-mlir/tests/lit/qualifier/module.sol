// RUN: solx --emit-mlir=sol %qualifier_module/main.sol %qualifier_module/module.sol %qualifier_module/nested.sol | FileCheck %s

// CHECK: sol.func @{{.*plain.*}}
// CHECK:   sol.call @{{.*half.*}}

// CHECK: sol.func @{{.*chained.*}}
// CHECK:   sol.call @{{.*half.*}}

// CHECK: sol.func @{{.*starred.*}}
// CHECK:   sol.call @{{.*half.*}}

// CHECK: sol.func @{{.*chain.*}}
// CHECK:   sol.constant 7
