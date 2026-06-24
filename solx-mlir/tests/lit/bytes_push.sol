// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// `bytes.push(x)` appends a single byte in place via `sol.push_string` (the value
// is passed directly; the packed element has no separate slot), unlike an array
// `push` which returns a slot reference to store into.

// CHECK: sol.func @{{.*pushByte.*}}
// CHECK: sol.push_string %{{[0-9]+}}, %{{[0-9]+}} : <Storage>, !sol.fixedbytes<1>

contract C {
    bytes data;

    function pushByte(bytes1 x) public {
        data.push(x);
    }
}
