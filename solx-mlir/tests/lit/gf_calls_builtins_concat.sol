// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// `bytes.concat(...)` / `string.concat(...)` lower to a variadic `sol.concat`
// over the operand values, yielding a fresh memory buffer.

// CHECK-DAG: sol.concat %{{.*}}, %{{.*}} : !sol.string<Memory>, !sol.string<Memory> -> <Memory>

contract C {
    function bc(bytes memory a, bytes memory b) public pure returns (bytes memory) {
        return bytes.concat(a, b);
    }
    function sc(string memory a, string memory b) public pure returns (string memory) {
        return string.concat(a, b);
    }
}
