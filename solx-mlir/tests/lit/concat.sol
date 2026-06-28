// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*bc.*}}
// CHECK:   sol.concat %{{.*}}, %{{.*}} : !sol.string<Memory>, !sol.string<Memory> -> <Memory>
// CHECK: sol.func @{{.*sc.*}}
// CHECK:   sol.concat %{{.*}}, %{{.*}} : !sol.string<Memory>, !sol.string<Memory> -> <Memory>

contract C {
    function bc(bytes memory a, bytes memory b) public pure returns (bytes memory) {
        return bytes.concat(a, b);
    }

    function sc(string memory a, string memory b) public pure returns (string memory) {
        return string.concat(a, b);
    }
}
