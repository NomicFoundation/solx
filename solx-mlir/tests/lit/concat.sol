// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func {{.*}}bytes_concat{{.*}}-> !sol.string<Memory>
// CHECK:   sol.concat %{{.*}}, %{{.*}} : !sol.string<Memory>, !sol.string<Memory> -> <Memory>

// CHECK: sol.func {{.*}}string_concat{{.*}}-> !sol.string<Memory>
// CHECK:   sol.concat %{{.*}}, %{{.*}} : !sol.string<Memory>, !sol.string<Memory> -> <Memory>

// CHECK: sol.func {{.*}}mixed{{.*}}-> !sol.string<Memory>
// CHECK:   sol.concat %{{.*}}, %{{.*}}, %{{.*}} : !sol.string<Memory>, !sol.fixedbytes<4>, !sol.string<Memory> -> <Memory>

// CHECK: sol.func {{.*}}empty{{.*}}-> !sol.string<Memory>
// CHECK:   sol.concat -> <Memory>

contract Concat {
    function bytes_concat(bytes memory a, bytes memory b) public pure returns (bytes memory) {
        return bytes.concat(a, b);
    }

    function string_concat(string memory a, string memory b) public pure returns (string memory) {
        return string.concat(a, b);
    }

    function mixed(bytes memory a, bytes4 t, bytes memory b) public pure returns (bytes memory) {
        return bytes.concat(a, t, b);
    }

    function empty() public pure returns (bytes memory) {
        return bytes.concat();
    }
}
