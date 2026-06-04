// RUN: solx --emit-mlir=sol %s | FileCheck %s --check-prefixes=CHECK,CHECK-SOLX
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s --check-prefixes=CHECK,CHECK-SOLC

// The auto-generated getter for a public, value-typed state variable reads its
// storage slot and returns the scalar. solx names it by canonical signature,
// solc by `get_<name>_<id>`; both share the storage-load body.

// CHECK-SOLX: sol.func @"value()"() -> ui256
// CHECK-SOLC: sol.func @get_value_{{[0-9]+}}() -> ui256
// CHECK:   sol.addr_of @value_{{[0-9]+}} : !sol.ptr<ui256, Storage>
// CHECK:   sol.load %{{.*}} : !sol.ptr<ui256, Storage>, ui256
// CHECK:   sol.return %{{.*}} : ui256

contract C {
    uint256 public value;
}
