// RUN: solx --emit-mlir=sol %s | FileCheck %s

// require() with a runtime (non-constant) string message: solc's print-init
// aborts on the errorCall assertion in genExprs (SolidityToMLIR.cpp:2162), so solx-only.

// CHECK: sol.func @{{.*check.*}}
// CHECK:   sol.require %{{.*}}, "Error(string)"(%{{.*}} : !sol.string<Memory>) {call}

contract C {
    function check(bool condition, string memory message) public pure {
        require(condition, message);
    }
}
