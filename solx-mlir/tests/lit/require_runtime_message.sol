// RUN: solx --emit-mlir=sol %s | FileCheck %s

// CHECK: sol.func @{{.*check.*}}
// CHECK:   sol.require %{{.*}}, "Error(string)"(%{{.*}} : !sol.string<Memory>) {call}

contract C {
    function check(bool cond, string memory message) public pure {
        require(cond, message);
    }
}
