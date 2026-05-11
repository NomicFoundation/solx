// RUN: solx --emit-mlir=sol %s | FileCheck %s

// CHECK: sol.func {{.*}}requireLiteral
// CHECK:   sol.require %{{.*}}, "literal message"()

// CHECK: sol.func {{.*}}requireRuntime
// CHECK:   sol.require %{{.*}}, "Error(string)"(%{{.*}} : !sol.string<{{.*}}>) {call}

contract C {
    function requireLiteral(bool cond) public pure {
        require(cond, "literal message");
    }

    function requireRuntime(bool cond, string memory message) public pure {
        require(cond, message);
    }
}
