// RUN: solx --emit-mlir=sol %s | FileCheck %s --check-prefixes=CHECK,CHECK-SOLX
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s --check-prefixes=CHECK,CHECK-SOLC

// CHECK: sol.func {{.*}}readArray{{.*}}-> ui256
// CHECK:   sol.gep %{{.*}}, %{{.*}} : !sol.array<? x ui256, Memory>, ui256, !sol.ptr<ui256, Memory>
// CHECK:   sol.load %{{.*}} : !sol.ptr<ui256, Memory>, ui256

// CHECK: sol.func {{.*}}readBytes{{.*}}-> !sol.fixedbytes<1>
// CHECK-SOLX:   sol.gep %{{.*}}, %{{.*}} : !sol.string<Memory>, ui256, !sol.ptr<!sol.fixedbytes<1>, Memory>
// CHECK-SOLX:   sol.load %{{.*}} : !sol.ptr<!sol.fixedbytes<1>, Memory>, !sol.fixedbytes<1>
// CHECK-SOLC:   sol.gep %{{.*}}, %{{.*}} : !sol.string<Memory>, ui256, !sol.ptr<!sol.byte, Memory>
// CHECK-SOLC:   sol.load %{{.*}} : !sol.ptr<!sol.byte, Memory>, !sol.byte
// CHECK-SOLC:   sol.bytes_cast %{{.*}} : !sol.byte to !sol.fixedbytes<1>

// CHECK: sol.func {{.*}}readMapping{{.*}}-> ui256
// CHECK:   sol.map %{{.*}}, %{{.*}} : !sol.mapping<ui256, ui256>, ui256, !sol.ptr<ui256, Storage>
// CHECK:   sol.load %{{.*}} : !sol.ptr<ui256, Storage>, ui256

contract C {
    mapping(uint256 => uint256) m;

    function readArray(uint256[] memory a, uint256 i) public pure returns (uint256) {
        return a[i];
    }

    function readBytes(bytes memory data, uint256 i) public pure returns (bytes1) {
        return data[i];
    }

    function readMapping(uint256 k) public view returns (uint256) {
        return m[k];
    }
}
