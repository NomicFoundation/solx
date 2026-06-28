// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.load %{{.*}} : !sol.ptr<!sol.byte, Storage>, !sol.byte
// CHECK: sol.bytes_cast %{{.*}} : !sol.byte to !sol.fixedbytes<1>
// CHECK: sol.bytes_cast %{{.*}} : ui8 to !sol.fixedbytes<1>
// CHECK: sol.cmp eq, %{{.*}}, %{{.*}} : !sol.fixedbytes<1>

contract C {
    bytes data;

    function f() public returns (bool) {
        data.push(0x05);
        return data[0] == 0x05;
    }
}
