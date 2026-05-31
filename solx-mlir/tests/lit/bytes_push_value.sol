// RUN: solx --emit-mlir=sol %s | FileCheck %s

// `bytes.push(x)` lowers to the dedicated sol.push_string op (which handles
// the in-place -> out-of-place storage-encoding transition), not generic
// sol.push.

// CHECK: sol.func @{{.*}}f
// CHECK: sol.push_string

contract C {
    bytes data;

    function f(bytes1 x) public {
        data.push(x);
    }
}
