// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// `address.send(v)` / `address.transfer(v)` take a `ui256` amount, so a narrow
// literal argument is widened with `sol.cast` first. `send` yields an `i1`
// success flag; `transfer` reverts on failure and yields nothing.

// CHECK-DAG: sol.cast %{{.*}} : ui8 to ui256
// CHECK-DAG: sol.send %{{.*}}, %{{.*}} : !sol.address, ui256 -> i1
// CHECK-DAG: sol.transfer %{{.*}}, %{{.*}} : !sol.address, ui256

contract C {
    function s0(address payable r) public returns (bool) { return r.send(0); }
    function t0(address payable r) public { r.transfer(1); }
}
