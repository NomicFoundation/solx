// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*pushByte.*}}
// CHECK: sol.push_string %{{[0-9]+}}, %{{[0-9]+}} : <Storage>, !sol.fixedbytes<1>

contract C {
    bytes data;

    function pushByte(bytes1 x) public {
        data.push(x);
    }
}
