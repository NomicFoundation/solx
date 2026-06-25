// RUN: solx --emit-mlir=sol %s | FileCheck %s --check-prefix=CHECK-SOLX
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s --check-prefix=CHECK-SOLC

// `bytes.push()` with no argument takes the slot-returning `push_slot` path
// (unlike `bytes.push(x)`, which passes the byte value directly via
// `sol.push_string`). `push_slot` resolves the pushed element type through
// `Type::dynamic_array_element`'s `Bytes` arm, producing a single-byte element
// place that the returned `sol.push` reference addresses.
//
// The two backends spell the single-byte element type differently — solx as
// `!sol.fixedbytes<1>`, solc as the dedicated `!sol.byte` — so the element type
// is checked per-backend; the `sol.push` over the storage `!sol.string<Storage>`
// receiver is identical.

contract C {
    bytes data;

    function pushEmptyByte() public {
        data.push();
    }
}

// CHECK-SOLX: sol.func @{{.*pushEmptyByte.*}}
// CHECK-SOLX:   %[[B:.*]] = sol.addr_of @{{.*data.*}} : !sol.string<Storage>
// CHECK-SOLX:   %{{.*}} = sol.push %[[B]] : !sol.string<Storage> -> !sol.ptr<!sol.fixedbytes<1>, Storage>

// CHECK-SOLC: sol.func @{{.*pushEmptyByte.*}}
// CHECK-SOLC:   %[[B:.*]] = sol.addr_of @{{.*data.*}} : !sol.string<Storage>
// CHECK-SOLC:   %{{.*}} = sol.push %[[B]] : !sol.string<Storage> -> !sol.ptr<!sol.byte, Storage>
