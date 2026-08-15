// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*library_address.*}}() -> !sol.address
// CHECK:   %[[ADDR:.*]] = sol.lib_addr "{{[^"]*}}Lib" : !sol.address
// CHECK:   sol.return %[[ADDR]] : !sol.address

library Lib {}

contract User {
    function library_address() public pure returns (address) {
        return address(Lib);
    }
}
