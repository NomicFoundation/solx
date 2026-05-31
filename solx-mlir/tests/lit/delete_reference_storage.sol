// RUN: solx --emit-mlir=sol %s | FileCheck %s

// `delete` on a dynamic `bytes`/`string` storage variable resets it to empty
// by copying a freshly allocated zero-length memory buffer into the slot.

// CHECK: sol.func @{{.*}}f
// CHECK: sol.malloc
// CHECK: sol.copy

contract C {
    bytes data;

    function f() public {
        delete data;
    }
}
