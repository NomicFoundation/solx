// RUN: solx --emit-mlir=sol %qualifier_module/main.sol %qualifier_module/module.sol %qualifier_module/nested.sol | FileCheck %s

// solc's print-init drops a computed module qualifier's effect, which legacy keeps, so this is
// solx-only.

// CHECK: sol.func @{{.*constantMember.*}}
// CHECK:   sol.call @{{.*mark.*}}
// CHECK:   sol.constant 7

// CHECK: sol.func @{{.*freeCall.*}}
// CHECK:   sol.call @{{.*eight.*}}
// CHECK:   sol.call @{{.*mark.*}}
// CHECK:   sol.call @{{.*half.*}}

// CHECK: sol.func @{{.*designator.*}}
// CHECK:   sol.call @{{.*eight.*}}
// CHECK:   sol.call @{{.*mark.*}}
// CHECK:   sol.call @{{.*half.*}}

// CHECK: sol.func @{{.*chain.*}}
// CHECK:   sol.call @{{.*mark.*}}
// CHECK:   sol.constant 7

// CHECK: sol.func @{{.*aliased.*}}
// CHECK:   sol.call @{{.*mark.*}}
// CHECK:   sol.constant 7

// CHECK: sol.func @{{.*stray.*}}
// CHECK:   sol.call @{{.*mark.*}}

// CHECK: sol.func @{{.*plain.*}}
// CHECK:   sol.call @{{.*half.*}}

// CHECK: sol.func @{{.*chained.*}}
// CHECK:   sol.call @{{.*half.*}}

// CHECK: sol.func @{{.*starred.*}}
// CHECK:   sol.call @{{.*half.*}}
