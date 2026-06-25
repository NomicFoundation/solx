// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// Explicit conversions between `address` and a contract / interface type lower
// to `sol.address_cast` in both directions. The contract symbol differs (solc
// appends a node id), so match it with a regex. The two functions emit in
// different orders (solx alphabetical, solc source), so use CHECK-DAG.

// CHECK-DAG: sol.address_cast %{{.*}} : !sol.address to !sol.contract<"{{.*I.*}}">
// CHECK-DAG: sol.address_cast %{{.*}} : !sol.contract<"{{.*I.*}}"> to !sol.address

interface I {}

contract C {
    function toI(address a) public pure returns (I) { return I(a); }
    function toAddr(I i) public pure returns (address) { return address(i); }
}
