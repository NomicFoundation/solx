// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*or_element.*}}
// CHECK:   %[[OR_PTR:.*]] = sol.gep %{{.*}}, %{{.*}} : !sol.string<Memory>, ui8, !sol.ptr<!sol.byte, Memory>
// CHECK:   sol.load %[[OR_PTR]] : !sol.ptr<!sol.byte, Memory>, !sol.byte
// CHECK:   sol.or %{{.*}}, %{{.*}} : !sol.{{(fixedbytes<1>|byte)}}
// CHECK:   sol.store %{{.*}}, %[[OR_PTR]] : !sol.byte, !sol.ptr<!sol.byte, Memory>

// CHECK: sol.func @{{.*and_element.*}}
// CHECK:   %[[AND_PTR:.*]] = sol.gep %{{.*}}, %{{.*}} : !sol.string<Memory>, ui8, !sol.ptr<!sol.byte, Memory>
// CHECK:   sol.load %[[AND_PTR]] : !sol.ptr<!sol.byte, Memory>, !sol.byte
// CHECK:   sol.and %{{.*}}, %{{.*}} : !sol.{{(fixedbytes<1>|byte)}}
// CHECK:   sol.store %{{.*}}, %[[AND_PTR]] : !sol.byte, !sol.ptr<!sol.byte, Memory>

// CHECK: sol.func @{{.*xor_element.*}}
// CHECK:   %[[XOR_PTR:.*]] = sol.gep %{{.*}}, %{{.*}} : !sol.string<Memory>, ui8, !sol.ptr<!sol.byte, Memory>
// CHECK:   sol.load %[[XOR_PTR]] : !sol.ptr<!sol.byte, Memory>, !sol.byte
// CHECK:   sol.xor %{{.*}}, %{{.*}} : !sol.{{(fixedbytes<1>|byte)}}
// CHECK:   sol.store %{{.*}}, %[[XOR_PTR]] : !sol.byte, !sol.ptr<!sol.byte, Memory>

// CHECK: sol.func @{{.*or_storage_element.*}}
// CHECK:   %[[SPTR:.*]] = sol.gep %{{.*}}, %{{.*}} : !sol.string<Storage>, ui8, !sol.ptr<!sol.byte, Storage>
// CHECK:   sol.load %[[SPTR]] : !sol.ptr<!sol.byte, Storage>, !sol.byte
// CHECK:   sol.or %{{.*}}, %{{.*}} : !sol.{{(fixedbytes<1>|byte)}}
// CHECK:   sol.store %{{.*}}, %[[SPTR]] : !sol.byte, !sol.ptr<!sol.byte, Storage>

contract C {
    bytes data;

    function or_element(bytes memory a) public pure returns (bytes1) {
        a[0] |= 0x05;
        return a[0];
    }

    function and_element(bytes memory a) public pure returns (bytes1) {
        a[1] &= 0xf8;
        return a[1];
    }

    function xor_element(bytes memory a) public pure returns (bytes1) {
        a[2] ^= 0x07;
        return a[2];
    }

    function or_storage_element(bytes1 x) public returns (bytes1) {
        data[0] |= x;
        return data[0];
    }
}
