// RUN: solx --emit-mlir=sol %s | FileCheck %s

// CHECK: %[[B:.*]] = sol.bytes_cast %{{.*}} : ui256 to !sol.fixedbytes<32>
// CHECK: %[[S:.*]] = sol.bytes_cast %[[B]] : !sol.fixedbytes<32> to ui256
// CHECK: sol.new "Child" value = %{{.*}} salt = %[[S]] ctor() : !sol.contract<"Child">

contract Child {}

contract C {
    function f() external returns (Child) {
        return new Child{salt: hex"01"}();
    }
}
