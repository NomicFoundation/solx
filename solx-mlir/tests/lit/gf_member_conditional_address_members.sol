// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// address.balance / .codehash / address(this).balance: a unary intrinsic over the
// receiver. .codehash yields a ui256 bridged to bytes4 via sol.bytes_cast. solx
// walks functions alphabetically, solc in source order; the `sol.this` contract
// name carries a solc node-id suffix, so it is matched with a regex. CHECK-DAG.

// CHECK-DAG: sol.func @{{.*bal.*}}(%{{.*}}: !sol.address) -> ui256
// CHECK-DAG:   sol.balance %{{.*}} : !sol.address -> ui256
// CHECK-DAG: sol.func @{{.*ch.*}}(%{{.*}}: !sol.address) -> !sol.fixedbytes<32>
// CHECK-DAG:   sol.code_hash %{{.*}} : !sol.address -> ui256
// CHECK-DAG: sol.func @{{.*thisbal.*}}() -> ui256
// CHECK-DAG:   sol.this : !sol.contract<"C{{.*}}">

contract C {
    function bal(address a) public view returns (uint256) { return a.balance; }
    function ch(address a) public view returns (bytes32) { return a.codehash; }
    function thisbal() public view returns (uint256) { return address(this).balance; }
}
