// RUN: solx --emit-mlir=sol %qualifier_type_name/main.sol %qualifier_type_name/module.sol | FileCheck %s
// RUN: solc --mlir-action=print-init %qualifier_type_name/main.sol %qualifier_type_name/module.sol 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*enumMember.*}}
// CHECK:   sol.enum_cast

// CHECK: sol.func @{{.*interfaceEnum.*}}
// CHECK:   sol.enum_cast

// CHECK: sol.func @{{.*stateRead.*}}
// CHECK:   sol.store %{{.*}} : ui256, !sol.ptr<ui256, Storage>
// CHECK:   sol.load

// CHECK: sol.func @{{.*stateWrite.*}}
// CHECK:   sol.store %{{.*}} : ui256, !sol.ptr<ui256, Storage>
// CHECK:   sol.load

// CHECK: sol.func @{{.*stateCompound.*}}
// CHECK:   sol.cadd
// CHECK:   sol.store

// CHECK: sol.func @{{.*stateDelete.*}}
// CHECK:   sol.store %c0{{.*}} : ui256, !sol.ptr<ui256, Storage>
// CHECK:   sol.load

// CHECK: sol.func @{{.*fieldRead.*}}
// CHECK:   sol.gep
// CHECK:   sol.load

// CHECK: sol.func @{{.*fieldWrite.*}}
// CHECK:   sol.gep
// CHECK:   sol.store

// CHECK: sol.func @{{.*constantMember.*}}
// CHECK:   sol.constant 7

// CHECK: sol.func @{{.*immutableMember.*}}
// CHECK:   sol.load_immutable

// CHECK: sol.func @{{.*internalCall.*}}
// CHECK:   sol.call @{{.*seven.*}}

// CHECK: sol.func @{{.*libraryCall.*}}
// CHECK:   sol.call @{{.*half.*}}

// CHECK: sol.func @{{.*libraryConstant.*}}
// CHECK:   sol.constant 7

// CHECK: sol.func @{{.*wrap.*}}
// CHECK:   sol.constant 9

// CHECK: sol.func @{{.*construction.*}}
// CHECK:   sol.malloc
