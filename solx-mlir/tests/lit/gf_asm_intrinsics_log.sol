// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// log0..log4 all lower to a single variadic `yul.log` op carrying 0..4 topic
// operands (rule 16). Both frontends agree on the topic count; the concrete
// topic SSA values are matched loosely with %{{.*}} because solx and solc
// evaluate the topic arguments in opposite order.

// CHECK: sol.func @{{.*f.*}}
// CHECK: yul.log %{{.*}}, %{{.*}}
// CHECK: yul.log %{{.*}}, %{{.*}} topics(%{{.*}})
// CHECK: yul.log %{{.*}}, %{{.*}} topics(%{{.*}}, %{{.*}})
// CHECK: yul.log %{{.*}}, %{{.*}} topics(%{{.*}}, %{{.*}}, %{{.*}})
// CHECK: yul.log %{{.*}}, %{{.*}} topics(%{{.*}}, %{{.*}}, %{{.*}}, %{{.*}})

contract C {
    function f(uint256 t) public {
        assembly {
            log0(0, 32)
            log1(0, 32, t)
            log2(0, 32, t, t)
            log3(0, 32, t, t, t)
            log4(0, 32, t, t, t, t)
        }
    }
}
