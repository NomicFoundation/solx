// RUN: solx --emit-mlir=sol %s | FileCheck %s

// `delete` on a reference-typed storage variable resets it to its zero value.
// Dynamic `bytes`/`string` copy a freshly allocated zero-length memory buffer
// into the slot; arrays and structs emit `sol.delete`, which recursively clears
// every occupied storage slot (nested dynamic members reset to empty).

contract C {
    struct S {
        uint a;
        uint[] b;
    }

    bytes data;
    uint[] arr;
    S s;

    // CHECK-LABEL: sol.func {{.*}}f
    function f() public {
        // CHECK: sol.malloc
        // CHECK: sol.copy
        delete data;
        // CHECK: sol.addr_of
        // CHECK: sol.delete
        delete arr;
        // CHECK: sol.addr_of
        // CHECK: sol.delete
        delete s;
    }
}
