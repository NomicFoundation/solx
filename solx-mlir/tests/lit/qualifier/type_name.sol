// RUN: solx --emit-mlir=sol %qualifier_type_name/main.sol %qualifier_type_name/module.sol | FileCheck %s

// solc's print-init substitutes `sol.timestamp` for module member accesses, so this is solx-only.

// CHECK: sol.func @{{.*enumMember.*}}
// CHECK:   sol.call @{{.*mark.*}}
// CHECK:   sol.enum_cast

// CHECK: sol.func @{{.*interfaceEnum.*}}
// CHECK-NOT:   sol.call
// CHECK:   sol.enum_cast

// CHECK: sol.func @{{.*stateRead.*}}
// CHECK-NOT:   sol.call
// CHECK:   sol.return

// CHECK: sol.func @{{.*stateWrite.*}}
// CHECK-NOT:   sol.call @{{.*mark.*}}
// CHECK:   sol.call @{{.*nine.*}}
// CHECK-NOT:   sol.call @{{.*mark.*}}
// CHECK:   sol.return

// CHECK: sol.func @{{.*stateCompound.*}}
// CHECK-NOT:   sol.call @{{.*mark.*}}
// CHECK:   sol.call @{{.*nine.*}}
// CHECK:   sol.cadd
// CHECK-NOT:   sol.call @{{.*mark.*}}
// CHECK:   sol.return

// CHECK: sol.func @{{.*stateDelete.*}}
// CHECK-NOT:   sol.call
// CHECK:   sol.return

// CHECK: sol.func @{{.*fieldRead.*}}
// CHECK-NOT:   sol.call
// CHECK:   sol.return

// CHECK: sol.func @{{.*fieldWrite.*}}
// CHECK-NOT:   sol.call @{{.*mark.*}}
// CHECK:   sol.call @{{.*nine.*}}
// CHECK-NOT:   sol.call @{{.*mark.*}}
// CHECK:   sol.return

// CHECK: sol.func @{{.*constantMember.*}}
// CHECK-NOT:   sol.call
// CHECK:   sol.constant 7

// CHECK: sol.func @{{.*immutableMember.*}}
// CHECK-NOT:   sol.call
// CHECK:   sol.load_immutable

// CHECK: sol.func @{{.*internalCall.*}}
// CHECK-NOT:   sol.call @{{.*mark.*}}
// CHECK:   sol.call @{{.*seven.*}}
// CHECK-NOT:   sol.call @{{.*mark.*}}
// CHECK:   sol.return

// CHECK: sol.func @{{.*libraryCall.*}}
// CHECK-NOT:   sol.call @{{.*mark.*}}
// CHECK:   sol.call @{{.*eight.*}}
// CHECK:   sol.call @{{.*half.*}}
// CHECK-NOT:   sol.call @{{.*mark.*}}
// CHECK:   sol.return

// CHECK: sol.func @{{.*libraryConstant.*}}
// CHECK-NOT:   sol.call
// CHECK:   sol.constant 7

// CHECK: sol.func @{{.*wrap.*}}
// CHECK-NOT:   sol.call @{{.*mark.*}}
// CHECK:   sol.call @{{.*nine.*}}
// CHECK-NOT:   sol.call @{{.*mark.*}}
// CHECK:   sol.return

// CHECK: sol.func @{{.*construction.*}}
// CHECK-NOT:   sol.call @{{.*mark.*}}
// CHECK:   sol.malloc
// CHECK:   sol.call @{{.*nine.*}}
// CHECK-NOT:   sol.call @{{.*mark.*}}
// CHECK:   sol.return
