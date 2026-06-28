// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*pushEmptyByte.*}}
// CHECK:   %[[B:.*]] = sol.addr_of @{{.*data.*}} : !sol.string<Storage>
// CHECK:   %{{.*}} = sol.push %[[B]] : !sol.string<Storage> -> !sol.ptr<!sol.byte, Storage>

contract C {
    bytes data;

    function pushEmptyByte() public {
        data.push();
    }
}
