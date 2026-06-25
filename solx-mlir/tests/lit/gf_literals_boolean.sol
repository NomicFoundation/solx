// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// Boolean literals lower to `sol.constant false/true` at type i1. The two
// functions are alphabetically ordered so solx (alphabetical) and solc (source
// order) agree on the sequence.

// CHECK: sol.func @{{.*}}bfalse
// CHECK:   %false = sol.constant false
// CHECK:   sol.return %false : i1
// CHECK: sol.func @{{.*}}btrue
// CHECK:   %true = sol.constant true
// CHECK:   sol.return %true : i1

contract C {
    function bfalse() public pure returns (bool) {
        return false;
    }
    function btrue() public pure returns (bool) {
        return true;
    }
}
