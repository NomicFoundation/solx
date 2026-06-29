// RUN: solx --emit-mlir=sol %s | FileCheck %s
// RUN: solc --mlir-action=print-init %s 2>/dev/null | FileCheck %s

// CHECK: sol.func @{{.*balance.*}}(%{{.*}}: !sol.address) -> ui256
// CHECK:   sol.balance %{{.*}} : !sol.address -> ui256
// CHECK: sol.func @{{.*ch.*}}(%{{.*}}: !sol.address) -> !sol.fixedbytes<32>
// CHECK:   sol.code_hash %{{.*}} : !sol.address -> ui256
// CHECK: sol.func @{{.*thisbalance.*}}() -> ui256
// CHECK:   sol.this : !sol.contract<"C{{.*}}">

contract C {
    function balance(address a) public view returns (uint256) { return a.balance; }

    function ch(address a) public view returns (bytes32) { return a.codehash; }

    function thisbalance() public view returns (uint256) { return address(this).balance; }
}
