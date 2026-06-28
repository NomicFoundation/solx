// RUN: solx --emit-mlir=sol %s | FileCheck %s --check-prefix=CHECK-SOLX
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s --check-prefix=CHECK-SOLC

// `bytes.push()` with no argument appends a default single-byte element via
// `sol.push`, returning a reference to the new slot.
//
// The two backends spell the single-byte element type differently - solx as
// `!sol.fixedbytes<1>`, solc as the dedicated `!sol.byte` - so the element type
// is checked per-backend; the `sol.push` over the storage `!sol.string<Storage>`
// receiver is identical.

// CHECK-SOLX: sol.func @{{.*pushEmptyByte.*}}
// CHECK-SOLX:   %[[B:.*]] = sol.addr_of @{{.*data.*}} : !sol.string<Storage>
// CHECK-SOLX:   %{{.*}} = sol.push %[[B]] : !sol.string<Storage> -> !sol.ptr<!sol.fixedbytes<1>, Storage>

// CHECK-SOLC: sol.func @{{.*pushEmptyByte.*}}
// CHECK-SOLC:   %[[B:.*]] = sol.addr_of @{{.*data.*}} : !sol.string<Storage>
// CHECK-SOLC:   %{{.*}} = sol.push %[[B]] : !sol.string<Storage> -> !sol.ptr<!sol.byte, Storage>

contract C {
    bytes data;

    function pushEmptyByte() public {
        data.push();
    }
}
